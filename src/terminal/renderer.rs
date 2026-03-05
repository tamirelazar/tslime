//! Terminal renderer with overlay support.
//!
//! This module provides high-level rendering functionality that manages
//! terminal state and handles complex overlay rendering scenarios.

use crate::cli::ColorMode;
use crate::cli::Palette;
use crate::cli::PauseStyle;
use crate::config_defaults::TrailAgeMode;
use crate::render::charset::Charset;
use crate::render::dither::DitherMode;
use crate::render::downsample::{downsample_multi_species, Cell as DownsampleCell};
use crate::render::error_diffusion::ErrorDiffusion;
use crate::render::overlay::OverlayConfig;
use crate::render::palette;
use crate::render::palette::IntensityMapping;
use crate::render::palette::RgbColor;
use crate::render::panel::RenderedOverlay;
use crate::render::theme::PanelStyle;
use crate::terminal::frame_buffer::FrameBuffer;
use crossterm::execute;
use std::io::{self, Stdout};

/// Status line data: (text, x_position, colored_spans).
/// Each span is `(column_offset, color)`.
type StatusLineData = Option<(String, usize, Vec<(usize, RgbColor)>)>;

/// Handles the state and logic for rendering frames to the terminal.
///
/// Maintains persistent state like error diffusion buffers and configuration.
pub struct TerminalRenderer {
    stdout: Stdout,
    width: usize,
    height: usize,
    palette: Palette,
    charset: Charset,
    reverse_palette: bool,
    invert_palette: bool,
    color_mode: ColorMode,
    hue_shift: f32,
    dither_mode: DitherMode,
    intensity_mapping: Option<IntensityMapping>,
    error_diffusion: Option<ErrorDiffusion>,
    species_colors_enabled: bool,
    species_rgb_colors: Vec<RgbColor>,
    background_color: Option<RgbColor>,
    ascii_contrast: f32,
    aux_frame: Option<crate::render::downsample::AuxFrame>,
    trail_age_enabled: bool,
    trail_delta_enabled: bool,
    trail_age_hue_range: f32,
    trail_age_blend: f32,
    trail_age_mode: TrailAgeMode,
    trail_age_reverse: bool,
    trail_delta_strength: f32,
    gradient_magnitude_enabled: bool,
    gradient_strength: f32,
}

impl TerminalRenderer {
    /// Create a new terminal renderer.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        width: usize,
        height: usize,
        palette: Palette,
        charset: Charset,
        reverse_palette: bool,
        invert_palette: bool,
        color_mode: ColorMode,
        background_color: Option<RgbColor>,
    ) -> Self {
        Self {
            stdout: std::io::stdout(),
            width,
            height,
            palette,
            charset,
            reverse_palette,
            invert_palette,
            color_mode,
            hue_shift: 0.0,
            dither_mode: DitherMode::None,
            intensity_mapping: None,
            error_diffusion: None,
            species_colors_enabled: false,
            species_rgb_colors: Vec::new(),
            background_color,
            ascii_contrast: 1.5,
            aux_frame: None,
            trail_age_enabled: false,
            trail_delta_enabled: false,
            trail_age_hue_range: 15.0,
            trail_age_blend: 0.5,
            trail_age_mode: TrailAgeMode::Bidirectional,
            trail_age_reverse: true,
            trail_delta_strength: 0.5,
            gradient_magnitude_enabled: false,
            gradient_strength: 0.3,
        }
    }

    /// Set the dithering mode.
    ///
    /// This may allocate or resize error diffusion buffers.
    pub fn set_dither_mode(&mut self, mode: DitherMode) {
        self.dither_mode = mode;
        self.error_diffusion = match mode {
            DitherMode::ErrorDiffusion { .. } | DitherMode::Hybrid { .. } => {
                let mut ed = ErrorDiffusion::new(self.width, self.height);
                ed.resize(self.width, self.height);
                Some(ed)
            }
            _ => None,
        };
    }

    /// Set the intensity mapping for non-linear color distribution.
    pub fn set_intensity_mapping(&mut self, mapping: Option<IntensityMapping>) {
        self.intensity_mapping = mapping;
    }

    /// Get the current dithering mode.
    ///
    /// Part of the public API but currently unused internally.
    /// Retained for potential external use or testing.
    #[allow(dead_code)]
    pub fn dither_mode(&self) -> DitherMode {
        self.dither_mode
    }

    /// Reset error diffusion error accumulators.
    ///
    /// Should be called at the start of each frame.
    /// Currently reset automatically in `render_with_overlay`, but exposed
    /// as public API for advanced use cases.
    #[allow(dead_code)]
    pub fn reset_error_diffusion(&mut self) {
        if let Some(ref mut ed) = self.error_diffusion {
            ed.reset();
        }
    }

    /// Resize error diffusion buffers.
    ///
    /// Part of the public API but currently unused internally.
    /// Retained for potential external use when terminal size changes.
    #[allow(dead_code)]
    pub fn resize_error_diffusion(&mut self, width: usize, height: usize) {
        if let Some(ref mut ed) = self.error_diffusion {
            ed.resize(width, height);
        }
    }

    /// Update the renderer dimensions.
    pub fn set_dimensions(&mut self, width: usize, height: usize) {
        self.width = width;
        self.height = height;
    }

    /// Update the color palette.
    ///
    /// Part of the public API but currently unused internally.
    /// Retained for potential runtime palette switching features.
    #[allow(dead_code)]
    pub fn set_palette(&mut self, palette: Palette) {
        self.palette = palette;
    }

    /// Set the hue shift amount (0.0 to 1.0).
    ///
    /// Part of the public API but currently unused internally.
    /// Retained for potential runtime color adjustment features.
    #[allow(dead_code)]
    pub fn set_hue_shift(&mut self, hue_shift: f32) {
        self.hue_shift = hue_shift;
    }

    /// Update the character set used for rendering.
    ///
    /// Part of the public API but currently unused internally.
    /// Retained for potential runtime charset switching features.
    #[allow(dead_code)]
    pub fn set_charset(&mut self, charset: Charset) {
        self.charset = charset;
    }

    /// Enable or disable palette inversion (light <-> dark).
    pub fn set_invert_palette(&mut self, invert: bool) {
        self.invert_palette = invert;
    }

    /// Enable or disable palette reversal (start <-> end color).
    pub fn set_reverse_palette(&mut self, reverse: bool) {
        self.reverse_palette = reverse;
    }

    /// Set the contrast exponent for shape-vector ASCII rendering.
    ///
    /// Values > 1.0 sharpen edges; 1.0 = no enhancement; 2.0+ = strong.
    pub fn set_ascii_contrast(&mut self, contrast: f32) {
        self.ascii_contrast = contrast;
    }

    /// Set specific colors for multi-species rendering.
    pub fn set_species_colors(&mut self, enabled: bool, colors: Vec<RgbColor>) {
        self.species_colors_enabled = enabled;
        self.species_rgb_colors = colors;
    }

    /// Set visual effects data for trail age, temporal delta, and gradient magnitude.
    #[allow(clippy::too_many_arguments)]
    pub fn set_visual_fx(
        &mut self,
        aux: Option<crate::render::downsample::AuxFrame>,
        age: bool,
        delta: bool,
        age_hue_range: f32,
        age_blend: f32,
        delta_strength: f32,
        gradient: bool,
        gradient_strength: f32,
        age_mode: TrailAgeMode,
        age_reverse: bool,
    ) {
        self.aux_frame = aux;
        self.trail_age_enabled = age;
        self.trail_delta_enabled = delta;
        self.trail_age_hue_range = age_hue_range;
        self.trail_age_blend = age_blend;
        self.trail_delta_strength = delta_strength;
        self.gradient_magnitude_enabled = gradient;
        self.gradient_strength = gradient_strength;
        self.trail_age_mode = age_mode;
        self.trail_age_reverse = age_reverse;
    }

    /// Get a mutable reference to the standard output.
    #[allow(dead_code)]
    pub fn stdout_mut(&mut self) -> &mut Stdout {
        &mut self.stdout
    }

    /// Render a frame to the terminal.
    #[allow(dead_code)]
    pub fn render(
        &mut self,
        downsampled: &[DownsampleCell],
        max_trail_value: f32,
    ) -> io::Result<()> {
        if let Some(ref mut ed) = self.error_diffusion {
            ed.reset();
        }
        let buffer = FrameBuffer::from_downsampled(
            downsampled,
            self.width,
            self.height,
            max_trail_value,
            self.palette.clone(),
            self.charset.clone(),
            self.reverse_palette,
            self.invert_palette,
            self.color_mode,
            self.hue_shift,
            self.dither_mode,
            &mut self.error_diffusion,
            self.intensity_mapping.as_ref(),
            self.species_colors_enabled,
            if self.species_colors_enabled {
                Some(self.species_rgb_colors.clone())
            } else {
                None
            },
            self.background_color,
            self.ascii_contrast,
            self.aux_frame.as_ref(),
            self.trail_age_enabled,
            self.trail_delta_enabled,
            self.trail_age_hue_range,
            self.trail_age_blend,
            self.trail_delta_strength,
            self.gradient_magnitude_enabled,
            self.gradient_strength,
            self.trail_age_mode,
            self.trail_age_reverse,
        );

        execute!(self.stdout, &buffer)
    }

    /// Render a frame with text overlays.
    ///
    /// Supports various overlay types: help, controls, status, stats, info, config, etc.
    #[allow(clippy::too_many_arguments)]
    pub fn render_with_overlay(
        &mut self,
        downsampled: &[DownsampleCell],
        max_trail_value: f32,
        pause_frame: Option<u64>,
        pause_logo_overlay: Option<(&RenderedOverlay, usize, usize)>,
        pause_badge_overlay: Option<(&RenderedOverlay, usize, usize)>,
        controls_lines: Option<(&RenderedOverlay, usize, usize)>,
        status_line: StatusLineData,
        notification_line: Option<(&RenderedOverlay, usize, usize)>,
        dashboard_lines: Option<(&RenderedOverlay, usize, usize)>,
        grid_renderer: Option<&crate::render::grid::GridRenderer>,
        config_browser_lines: Option<(&RenderedOverlay, usize, usize)>,
        config_save_lines: Option<(&RenderedOverlay, usize, usize)>,
        keyboard_hints_lines: Option<(&RenderedOverlay, usize, usize)>,
        preset_comparison_lines: Option<(&RenderedOverlay, usize, usize)>,
        palette_editor_overlay: Option<(&RenderedOverlay, usize, usize)>,
        panel_style: Option<&PanelStyle>,
        _focused_overlay: Option<crate::overlay::OverlayType>,
        pause_style: PauseStyle,
        pause_pulse_draw_mode: bool,
    ) -> io::Result<()> {
        if let Some(ref mut ed) = self.error_diffusion {
            ed.reset();
        }
        let mut buffer = FrameBuffer::from_downsampled(
            downsampled,
            self.width,
            self.height,
            max_trail_value,
            self.palette.clone(),
            self.charset.clone(),
            self.reverse_palette,
            self.invert_palette,
            self.color_mode,
            self.hue_shift,
            self.dither_mode,
            &mut self.error_diffusion,
            self.intensity_mapping.as_ref(),
            self.species_colors_enabled,
            if self.species_colors_enabled {
                Some(self.species_rgb_colors.clone())
            } else {
                None
            },
            self.background_color,
            self.ascii_contrast,
            self.aux_frame.as_ref(),
            self.trail_age_enabled,
            self.trail_delta_enabled,
            self.trail_age_hue_range,
            self.trail_age_blend,
            self.trail_delta_strength,
            self.gradient_magnitude_enabled,
            self.gradient_strength,
            self.trail_age_mode,
            self.trail_age_reverse,
        );

        // Apply pause effect based on selected style
        if let Some(fc) = pause_frame {
            buffer.apply_pause_effect(pause_style, fc, pause_pulse_draw_mode);
        }

        // Apply grid rendering if enabled
        if let Some(mut grid) = grid_renderer.cloned() {
            grid.initialize(self.width, self.height);

            // Calculate average brightness for adaptive opacity
            let total_brightness: f32 = downsampled
                .iter()
                .map(|cell| cell.top.max(cell.bottom))
                .sum();
            let avg_brightness = if !downsampled.is_empty() && max_trail_value > 0.0 {
                (total_brightness / (downsampled.len() as f32)) / max_trail_value
            } else {
                0.0
            };

            // Apply grid to each position
            for y in 0..self.height {
                for x in 0..self.width {
                    if grid.is_grid_position(x, y, self.width, self.height) {
                        let (on_vertical, on_horizontal) = grid.get_grid_lines(x, y);
                        let opacity =
                            grid.calculate_opacity(x, y, self.width, self.height, avg_brightness);
                        buffer.render_grid_background(
                            x,
                            y,
                            grid.color,
                            opacity,
                            on_vertical,
                            on_horizontal,
                        );
                    }
                }
            }
        }

        // Palette accent color used for accented title badges.
        let accent = palette::palette_accent_color(
            &self.palette,
            self.reverse_palette,
            self.invert_palette,
            self.hue_shift,
            self.intensity_mapping.as_ref(),
        );

        // Helper to get colors from OverlayConfig and PanelStyle
        let get_overlay_colors =
            |config: &OverlayConfig,
             style: Option<&PanelStyle>|
             -> (u8, Option<u8>, Option<RgbColor>, Option<RgbColor>, usize) {
                if let Some(s) = style {
                    (
                        config.text_color_256,
                        Some(config.bg_color_256),
                        Some(s.bg_color),
                        Some(s.border_color),
                        s.indicator_width,
                    )
                } else {
                    (
                        config.text_color_256,
                        Some(config.bg_color_256),
                        None,
                        None,
                        0,
                    )
                }
            };

        // Unified draw helper: main panel + rich_lines + accented title badge.
        let draw_overlay = |buf: &mut FrameBuffer,
                            overlay: &RenderedOverlay,
                            x: usize,
                            y: usize,
                            config: &OverlayConfig| {
            let (fg, bg, panel_bg, _border_col, _w) = get_overlay_colors(config, panel_style);
            if panel_bg.is_some() {
                buf.draw_text_overlay_with_panel(&overlay.lines, x, y, fg, bg, panel_bg, None, 0);
            } else {
                buf.draw_text_overlay(&overlay.lines, x, y, fg, bg);
            }
            // Apply theme colors per character — border chars get border_color,
            // all other chars get text_primary. Done before rich_lines so per-overlay
            // overrides (e.g. notification accent, key bindings) still win.
            if let Some(style) = panel_style {
                for (line_idx, line) in overlay.lines.iter().enumerate() {
                    for (col, c) in line.chars().enumerate() {
                        let cell_x = x + col;
                        let cell_y = y + line_idx;
                        if cell_x < buf.width && cell_y < buf.height {
                            let idx = cell_y * buf.width + cell_x;
                            let color = if matches!(c, '█' | '▀' | '▄') {
                                style.border_color
                            } else {
                                style.text_primary
                            };
                            match buf.color_mode {
                                ColorMode::TrueColor => {
                                    buf.cells[idx].fg_color_rgb = Some(color);
                                }
                                _ => {
                                    buf.cells[idx].fg_color_256 = Some(palette::rgb_to_256(color));
                                }
                            }
                        }
                    }
                }
            }
            if let Some(ref rich) = overlay.rich_lines {
                buf.draw_rich_overlay(rich, x, y);
            }
            if let Some(tb) = &overlay.title_box {
                let mini_x = x + 1 + tb.col_offset;
                let mini_y = y.saturating_sub(1);
                let badge_fg = if let Some(style) = panel_style {
                    palette::rgb_to_256(style.text_primary)
                } else {
                    palette::rgb_to_256(palette::RgbColor {
                        r: 255,
                        g: 255,
                        b: 255,
                    })
                };
                if panel_bg.is_some() {
                    buf.draw_text_overlay_with_panel(
                        &tb.lines, mini_x, mini_y, badge_fg, bg, panel_bg, None, 0,
                    );
                } else {
                    buf.draw_text_overlay(&tb.lines, mini_x, mini_y, badge_fg, bg);
                }
                // Apply accent color to border chars
                let num_lines = tb.lines.len();
                for (line_idx, line) in tb.lines.iter().enumerate() {
                    let width = line.chars().count();
                    if width < 2 {
                        continue;
                    }
                    // Top and bottom borders: color all columns
                    // Middle lines: color only first and last column (vertical borders)
                    let cols_to_color: Vec<usize> = if line_idx == 0 || line_idx == num_lines - 1 {
                        (0..width).collect()
                    } else {
                        vec![0, width - 1]
                    };
                    for col in cols_to_color {
                        let cell_x = mini_x + col;
                        let cell_y = mini_y + line_idx;
                        if cell_x < buf.width && cell_y < buf.height {
                            let idx = cell_y * buf.width + cell_x;
                            match buf.color_mode {
                                ColorMode::TrueColor => buf.cells[idx].fg_color_rgb = Some(accent),
                                _ => {
                                    buf.cells[idx].fg_color_256 = Some(palette::rgb_to_256(accent))
                                }
                            }
                        }
                    }
                }
            }
        };

        // Pause logo overlay (drawn first so status/controls appear on top)
        if let Some((overlay, x, y)) = pause_logo_overlay {
            if let Some(ref rich) = overlay.rich_lines {
                buffer.draw_rich_overlay(rich, x, y);
            }
        }

        // Pause badge overlay
        if let Some((overlay, x, y)) = pause_badge_overlay {
            draw_overlay(&mut buffer, overlay, x, y, &OverlayConfig::NOTIFICATION);
        }

        // Controls overlay at top-left
        if let Some((overlay, x, y)) = controls_lines {
            draw_overlay(&mut buffer, overlay, x, y, &OverlayConfig::CONTROLS);
        }

        // Status line at bottom
        if let Some((line, x, color_overrides)) = status_line {
            let config = &OverlayConfig::STATUS;
            let status_y = self.height.saturating_sub(2);
            let line_chars: Vec<char> = line.chars().collect();
            buffer.draw_text_overlay(
                &[&line_chars.iter().collect::<String>()],
                x,
                status_y,
                config.text_color_256,
                Some(config.bg_color_256),
            );
            // Apply theme colors: status_bar_bg across the full row, text_primary on text cells.
            if let Some(style) = panel_style {
                if status_y < buffer.height {
                    let text_end = (x + line_chars.len()).min(buffer.width);
                    for cell_x in 0..buffer.width {
                        let idx = status_y * buffer.width + cell_x;
                        match buffer.color_mode {
                            ColorMode::TrueColor => {
                                buffer.cells[idx].bg_color_rgb = Some(style.status_bar_bg);
                                if cell_x >= x && cell_x < text_end {
                                    buffer.cells[idx].fg_color_rgb = Some(style.text_primary);
                                }
                            }
                            _ => {
                                buffer.cells[idx].bg_color_256 =
                                    Some(palette::rgb_to_256(style.status_bar_bg));
                                if cell_x >= x && cell_x < text_end {
                                    buffer.cells[idx].fg_color_256 =
                                        Some(palette::rgb_to_256(style.text_primary));
                                }
                            }
                        }
                    }
                }
            }
            // Apply per-column color overrides (swatch, ↺↻ undo/redo, PAUSED badge, ? hint)
            for (col, fg) in &color_overrides {
                let cell_x = x + col;
                if cell_x < buffer.width && status_y < buffer.height {
                    let idx = status_y * buffer.width + cell_x;
                    match buffer.color_mode {
                        ColorMode::TrueColor => buffer.cells[idx].fg_color_rgb = Some(*fg),
                        _ => buffer.cells[idx].fg_color_256 = Some(palette::rgb_to_256(*fg)),
                    }
                }
            }
        }

        // Notification at bottom-center
        if let Some((overlay, x, y)) = notification_line {
            draw_overlay(&mut buffer, overlay, x, y, &OverlayConfig::NOTIFICATION);
        }

        // Dashboard overlay
        if let Some((overlay, x, y)) = dashboard_lines {
            draw_overlay(&mut buffer, overlay, x, y, &OverlayConfig::DASHBOARD);
        }

        // Config browser overlay (modal, on top)
        if let Some((overlay, x, y)) = config_browser_lines {
            draw_overlay(&mut buffer, overlay, x, y, &OverlayConfig::CONFIG_BROWSER);
        }

        // Config save overlay (modal, on top)
        if let Some((overlay, x, y)) = config_save_lines {
            draw_overlay(&mut buffer, overlay, x, y, &OverlayConfig::CONFIG_SAVE);
        }

        // Keyboard hints overlay (modal, on top)
        if let Some((overlay, x, y)) = keyboard_hints_lines {
            draw_overlay(&mut buffer, overlay, x, y, &OverlayConfig::KEYBOARD_HINTS);
        }

        // Preset comparison overlay (modal, on top)
        if let Some((overlay, x, y)) = preset_comparison_lines {
            draw_overlay(
                &mut buffer,
                overlay,
                x,
                y,
                &OverlayConfig::PRESET_COMPARISON,
            );
        }

        // Palette editor overlay (modal, on top)
        if let Some((overlay, x, y)) = palette_editor_overlay {
            draw_overlay(&mut buffer, overlay, x, y, &OverlayConfig::PALETTE_EDITOR);
        }

        execute!(self.stdout, &buffer)
    }

    /// Render a multi-species frame with text overlays.
    ///
    /// Combines multiple trail maps, assigning a distinct color to each species.
    #[allow(clippy::too_many_arguments)]
    pub fn render_multi_species_with_overlay(
        &mut self,
        trail_maps: &[(&[f32], RgbColor)],
        sim_width: usize,
        sim_height: usize,
        max_trail_value: f32,
        pause_frame: Option<u64>,
        pause_logo_overlay: Option<(&RenderedOverlay, usize, usize)>,
        pause_badge_overlay: Option<(&RenderedOverlay, usize, usize)>,
        controls_lines: Option<(&RenderedOverlay, usize, usize)>,
        status_line: StatusLineData,
        notification_line: Option<(&RenderedOverlay, usize, usize)>,
        dashboard_lines: Option<(&RenderedOverlay, usize, usize)>,
        grid_renderer: Option<&crate::render::grid::GridRenderer>,
        config_browser_lines: Option<(&RenderedOverlay, usize, usize)>,
        config_save_lines: Option<(&RenderedOverlay, usize, usize)>,
        keyboard_hints_lines: Option<(&RenderedOverlay, usize, usize)>,
        preset_comparison_lines: Option<(&RenderedOverlay, usize, usize)>,
        palette_editor_overlay: Option<(&RenderedOverlay, usize, usize)>,
        panel_style_ms: Option<&PanelStyle>,
        _focused_overlay: Option<crate::overlay::OverlayType>,
        pause_style: PauseStyle,
        pause_pulse_draw_mode: bool,
    ) -> io::Result<()> {
        if let Some(ref mut ed) = self.error_diffusion {
            ed.reset();
        }

        let mut buffer = FrameBuffer::new(
            self.width,
            self.height,
            self.color_mode,
            self.background_color,
        );
        buffer.species_colors_enabled = true;
        buffer.species_rgb_colors = self.species_rgb_colors.clone();

        // Keep track of downsampled cells for grid brightness calculation
        let mut all_downsampled_cells = Vec::new();

        for (trail_map, species_color) in trail_maps {
            let downsampled = downsample_multi_species(
                &[(trail_map, 0)],
                sim_width,
                sim_height,
                self.width,
                self.height,
            );

            // Store for brightness calculation
            if all_downsampled_cells.is_empty() {
                all_downsampled_cells = downsampled.cells().to_vec();
            } else {
                // Sum cells from different species
                for (i, cell) in downsampled.cells().iter().enumerate() {
                    if i < all_downsampled_cells.len() {
                        all_downsampled_cells[i].top += cell.top;
                        all_downsampled_cells[i].bottom += cell.bottom;
                    }
                }
            }

            let species_color_vec = vec![*species_color];
            let species_buffer = FrameBuffer::from_downsampled(
                downsampled.cells(),
                self.width,
                self.height,
                max_trail_value,
                self.palette.clone(),
                self.charset.clone(),
                self.reverse_palette,
                self.invert_palette,
                self.color_mode,
                self.hue_shift,
                self.dither_mode,
                &mut self.error_diffusion,
                self.intensity_mapping.as_ref(),
                true,
                Some(species_color_vec),
                self.background_color,
                self.ascii_contrast,
                None, // aux_frame not supported for multi-species
                false,
                false,
                60.0,  // default hue range
                1.0,   // default blend
                0.5,   // default delta strength
                false, // gradient disabled
                0.3,   // default gradient strength
                TrailAgeMode::Bidirectional,
                false,
            );

            for (i, cell) in species_buffer.cells.iter().enumerate() {
                if cell.char != ' ' {
                    buffer.cells[i] = *cell;
                }
            }
        }

        // Apply pause effect based on selected style
        if let Some(fc) = pause_frame {
            buffer.apply_pause_effect(pause_style, fc, pause_pulse_draw_mode);
        }

        // Apply grid rendering if enabled
        if let Some(mut grid) = grid_renderer.cloned() {
            grid.initialize(self.width, self.height);

            // Calculate average brightness from all species combined
            let total_brightness: f32 = all_downsampled_cells
                .iter()
                .map(|cell| cell.top.max(cell.bottom))
                .sum();
            let avg_brightness = if !all_downsampled_cells.is_empty() && max_trail_value > 0.0 {
                (total_brightness / (all_downsampled_cells.len() as f32)) / max_trail_value
            } else {
                0.0
            };

            // Apply grid to each position
            for y in 0..self.height {
                for x in 0..self.width {
                    if grid.is_grid_position(x, y, self.width, self.height) {
                        let (on_vertical, on_horizontal) = grid.get_grid_lines(x, y);
                        let opacity =
                            grid.calculate_opacity(x, y, self.width, self.height, avg_brightness);
                        buffer.render_grid_background(
                            x,
                            y,
                            grid.color,
                            opacity,
                            on_vertical,
                            on_horizontal,
                        );
                    }
                }
            }
        }

        // Palette accent color for accented title badges (same approach as single-species).
        let accent = palette::palette_accent_color(
            &self.palette,
            self.reverse_palette,
            self.invert_palette,
            self.hue_shift,
            self.intensity_mapping.as_ref(),
        );

        // Unified draw helper: main panel + rich_lines + accented title badge.
        let draw_ms_overlay = |buf: &mut FrameBuffer,
                               overlay: &RenderedOverlay,
                               x: usize,
                               y: usize,
                               config: &OverlayConfig| {
            buf.draw_text_overlay(
                &overlay.lines,
                x,
                y,
                config.text_color_256,
                Some(config.bg_color_256),
            );
            // Apply theme colors per character — border chars get border_color,
            // all other chars get text_primary.
            if let Some(style) = panel_style_ms {
                for (line_idx, line) in overlay.lines.iter().enumerate() {
                    for (col, c) in line.chars().enumerate() {
                        let cell_x = x + col;
                        let cell_y = y + line_idx;
                        if cell_x < buf.width && cell_y < buf.height {
                            let idx = cell_y * buf.width + cell_x;
                            let color = if matches!(c, '█' | '▀' | '▄') {
                                style.border_color
                            } else {
                                style.text_primary
                            };
                            match buf.color_mode {
                                ColorMode::TrueColor => {
                                    buf.cells[idx].fg_color_rgb = Some(color);
                                }
                                _ => {
                                    buf.cells[idx].fg_color_256 = Some(palette::rgb_to_256(color));
                                }
                            }
                        }
                    }
                }
            }
            if let Some(ref rich) = overlay.rich_lines {
                buf.draw_rich_overlay(rich, x, y);
            }
            if let Some(tb) = &overlay.title_box {
                let mini_x = x + 1 + tb.col_offset;
                let mini_y = y.saturating_sub(1);
                let badge_fg = if let Some(style) = panel_style_ms {
                    palette::rgb_to_256(style.text_primary)
                } else {
                    palette::rgb_to_256(palette::RgbColor {
                        r: 255,
                        g: 255,
                        b: 255,
                    })
                };
                buf.draw_text_overlay(
                    &tb.lines,
                    mini_x,
                    mini_y,
                    badge_fg,
                    Some(config.bg_color_256),
                );
                // Apply accent color to border chars
                let num_lines = tb.lines.len();
                for (line_idx, line) in tb.lines.iter().enumerate() {
                    let width = line.chars().count();
                    if width < 2 {
                        continue;
                    }
                    // Top and bottom borders: color all columns
                    // Middle lines: color only first and last column (vertical borders)
                    let cols_to_color: Vec<usize> = if line_idx == 0 || line_idx == num_lines - 1 {
                        (0..width).collect()
                    } else {
                        vec![0, width - 1]
                    };
                    for col in cols_to_color {
                        let cell_x = mini_x + col;
                        let cell_y = mini_y + line_idx;
                        if cell_x < buf.width && cell_y < buf.height {
                            let idx = cell_y * buf.width + cell_x;
                            match buf.color_mode {
                                ColorMode::TrueColor => buf.cells[idx].fg_color_rgb = Some(accent),
                                _ => {
                                    buf.cells[idx].fg_color_256 = Some(palette::rgb_to_256(accent))
                                }
                            }
                        }
                    }
                }
            }
        };

        // Pause logo overlay (drawn first so status/controls appear on top)
        if let Some((overlay, x, y)) = pause_logo_overlay {
            if let Some(ref rich) = overlay.rich_lines {
                buffer.draw_rich_overlay(rich, x, y);
            }
        }

        // Pause badge overlay
        if let Some((overlay, x, y)) = pause_badge_overlay {
            draw_ms_overlay(&mut buffer, overlay, x, y, &OverlayConfig::NOTIFICATION);
        }

        // Controls overlay at top-left
        if let Some((overlay, x, y)) = controls_lines {
            draw_ms_overlay(&mut buffer, overlay, x, y, &OverlayConfig::CONTROLS);
        }

        // Status line at bottom
        if let Some((line, x, color_overrides)) = status_line {
            let config = &OverlayConfig::STATUS;
            let status_y = self.height.saturating_sub(2);
            let line_chars: Vec<char> = line.chars().collect();
            buffer.draw_text_overlay(
                &[&line_chars.iter().collect::<String>()],
                x,
                status_y,
                config.text_color_256,
                Some(config.bg_color_256),
            );
            // Apply theme colors: status_bar_bg across the full row, text_primary on text cells.
            if let Some(style) = panel_style_ms {
                if status_y < buffer.height {
                    let text_end = (x + line_chars.len()).min(buffer.width);
                    for cell_x in 0..buffer.width {
                        let idx = status_y * buffer.width + cell_x;
                        match buffer.color_mode {
                            ColorMode::TrueColor => {
                                buffer.cells[idx].bg_color_rgb = Some(style.status_bar_bg);
                                if cell_x >= x && cell_x < text_end {
                                    buffer.cells[idx].fg_color_rgb = Some(style.text_primary);
                                }
                            }
                            _ => {
                                buffer.cells[idx].bg_color_256 =
                                    Some(palette::rgb_to_256(style.status_bar_bg));
                                if cell_x >= x && cell_x < text_end {
                                    buffer.cells[idx].fg_color_256 =
                                        Some(palette::rgb_to_256(style.text_primary));
                                }
                            }
                        }
                    }
                }
            }
            for (col, fg) in &color_overrides {
                let cell_x = x + col;
                if cell_x < buffer.width && status_y < buffer.height {
                    let idx = status_y * buffer.width + cell_x;
                    match buffer.color_mode {
                        ColorMode::TrueColor => buffer.cells[idx].fg_color_rgb = Some(*fg),
                        _ => buffer.cells[idx].fg_color_256 = Some(palette::rgb_to_256(*fg)),
                    }
                }
            }
        }

        // Notification at bottom-center
        if let Some((overlay, x, y)) = notification_line {
            draw_ms_overlay(&mut buffer, overlay, x, y, &OverlayConfig::NOTIFICATION);
        }

        // Dashboard overlay
        if let Some((overlay, x, y)) = dashboard_lines {
            draw_ms_overlay(&mut buffer, overlay, x, y, &OverlayConfig::DASHBOARD);
        }

        // Config browser overlay (modal, on top)
        if let Some((overlay, x, y)) = config_browser_lines {
            draw_ms_overlay(&mut buffer, overlay, x, y, &OverlayConfig::CONFIG_BROWSER);
        }

        // Config save overlay (modal, on top)
        if let Some((overlay, x, y)) = config_save_lines {
            draw_ms_overlay(&mut buffer, overlay, x, y, &OverlayConfig::CONFIG_SAVE);
        }

        // Keyboard hints overlay (modal, on top)
        if let Some((overlay, x, y)) = keyboard_hints_lines {
            draw_ms_overlay(&mut buffer, overlay, x, y, &OverlayConfig::KEYBOARD_HINTS);
        }

        // Preset comparison overlay (modal, on top)
        if let Some((overlay, x, y)) = preset_comparison_lines {
            draw_ms_overlay(
                &mut buffer,
                overlay,
                x,
                y,
                &OverlayConfig::PRESET_COMPARISON,
            );
        }

        // Palette editor overlay (modal, on top)
        if let Some((overlay, x, y)) = palette_editor_overlay {
            draw_ms_overlay(&mut buffer, overlay, x, y, &OverlayConfig::PALETTE_EDITOR);
        }

        execute!(self.stdout, &buffer)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_terminal_renderer_setters() {
        let mut renderer = TerminalRenderer::new(
            80,
            24,
            Palette::Organic,
            Charset::HalfBlock,
            false,
            false,
            ColorMode::TrueColor,
            None,
        );
        renderer.set_invert_palette(true);
        renderer.set_reverse_palette(true);
        renderer.set_species_colors(true, vec![RgbColor { r: 255, g: 0, b: 0 }]);
        renderer.set_charset(Charset::Ascii);
        renderer.set_palette(Palette::Heat);
        renderer.set_dimensions(100, 40);
        assert_eq!(renderer.width, 100);
        assert_eq!(renderer.height, 40);
    }

    #[test]
    fn test_render_multi_species_with_overlay() {
        let mut renderer = TerminalRenderer::new(
            80,
            24,
            Palette::Organic,
            Charset::HalfBlock,
            false,
            false,
            ColorMode::TrueColor,
            None,
        );
        let trail = vec![1.0; 100];
        let color = RgbColor {
            r: 255,
            g: 255,
            b: 255,
        };
        let trail_maps = vec![(&trail[..], color)];
        let result = renderer.render_multi_species_with_overlay(
            &trail_maps,
            10,
            10,
            1.0,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            PauseStyle::Vignette,
            false,
        );

        assert!(result.is_ok());
    }
}
