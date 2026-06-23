//! Terminal renderer with overlay support.
//!
//! This module provides high-level rendering functionality that manages
//! terminal state and handles complex overlay rendering scenarios.

use crate::cli::ColorMode;
use crate::cli::Palette;
use crate::cli::PauseStyle;
use crate::config_defaults::TrailAgeMode;
use crate::render::charset::{self as charset, Charset};
use crate::render::dither::DitherMode;
use crate::render::downsample::{downsample_multi_species, Cell as DownsampleCell};
use crate::render::error_diffusion::ErrorDiffusion;
use crate::render::motion::{breath, lerp_rgb};
use crate::render::overlay::{ExpandedChromeOverlay, OverlayConfig};
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

/// State snapshot provided by the runner for rendering expanded window chrome.
///
/// Populated each frame so the renderer can draw title/footer rows without
/// reaching back into the runner's mutable state.
pub struct ChromeSnapshot {
    /// Current chrome state (Minimal, Expanded, ModalPane).
    pub chrome_state: crate::terminal::state::ChromeState,
    /// Active simulation preset.
    pub preset: crate::simulation::config::Preset,
    /// Active color palette.
    pub palette: crate::cli::Palette,
    /// Human-readable charset name (e.g. "HalfBlock").
    pub charset_str: String,
    /// Number of simulation agents.
    pub population: usize,
    /// Simulation time scale multiplier.
    pub time_scale: f32,
    /// Current dithering mode.
    pub dither_mode: crate::render::dither::DitherMode,
    /// Diffusion kernel description (e.g. "Mean3x3").
    pub diffusion_kernel: Option<String>,
    /// Whether undo history is non-empty.
    pub can_undo: bool,
    /// Whether redo history is non-empty.
    pub can_redo: bool,
    /// Whether simulation is paused.
    pub is_paused: bool,
}

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
    color_aa: crate::render::antialiasing::AaStrength,
    aux_frame: Option<crate::render::downsample::AuxFrame>,
    /// Pre-allocated frame buffer to avoid per-frame allocations
    frame_buffer: Option<crate::render::downsample::DownsampledFrame>,
    trail_age_enabled: bool,
    trail_delta_enabled: bool,
    trail_age_hue_range: f32,
    trail_age_blend: f32,
    trail_age_mode: TrailAgeMode,
    trail_age_reverse: bool,
    trail_delta_strength: f32,
    gradient_magnitude_enabled: bool,
    gradient_strength: f32,
    window_frame: crate::simulation::config::WindowFrame,
    window_frame_accent_color: RgbColor,
    window_layout: Option<crate::render::window::WindowLayout>,
    chrome_snapshot: Option<ChromeSnapshot>,
    temporal_strength: f32,
    temporal_mode: palette::TemporalMode,
    temporal_accent: Option<palette::RgbColor>,
    palette_cycle: palette::PaletteCycle,
    glyph: charset::GlyphConfig,
    /// Monotonic seconds for overlay motion (breath on title-box accent). Set each frame.
    phase_clock: f32,
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
            color_aa: crate::render::antialiasing::AaStrength::Off,
            aux_frame: None,
            frame_buffer: None,
            trail_age_enabled: false,
            trail_delta_enabled: false,
            trail_age_hue_range: 15.0,
            trail_age_blend: 0.5,
            trail_age_mode: TrailAgeMode::Bidirectional,
            trail_age_reverse: true,
            trail_delta_strength: 0.5,
            gradient_magnitude_enabled: false,
            gradient_strength: 0.3,
            window_frame: crate::simulation::config::WindowFrame::None,
            window_frame_accent_color: RgbColor::new(0xFA, 0xBD, 0x2F),
            window_layout: None,
            chrome_snapshot: None,
            temporal_strength: 0.0,
            temporal_mode: palette::TemporalMode::Hue,
            temporal_accent: None,
            palette_cycle: palette::PaletteCycle::default(),
            glyph: charset::GlyphConfig::default(),
            phase_clock: 0.0,
        }
    }

    /// Set the phase clock for overlay motion (title-box breath, eases).
    pub fn set_phase_clock(&mut self, t: f32) {
        self.phase_clock = t;
    }

    /// Set the window frame mode for rendering.
    pub fn set_window_frame(&mut self, mode: crate::simulation::config::WindowFrame) {
        self.window_frame = mode;
    }

    /// Set the window frame accent color.
    pub fn set_window_frame_accent_color(&mut self, color: RgbColor) {
        self.window_frame_accent_color = color;
    }

    /// Set the window layout for windowed rendering mode.
    ///
    /// When `Some`, the simulation is rendered at `layout.sim_w × layout.sim_h` and
    /// composited into the full terminal buffer at `(layout.sim_x, layout.sim_y)`.
    /// When `None`, the simulation fills the terminal edge-to-edge (fullscreen mode).
    pub fn set_window_layout(&mut self, layout: Option<crate::render::window::WindowLayout>) {
        self.window_layout = layout;
        // Resize error diffusion buffer to match new sim dimensions
        let (ed_w, ed_h) = self
            .window_layout
            .as_ref()
            .map(|l| (l.sim_w, l.sim_h))
            .unwrap_or((self.width, self.height));
        if let Some(ref mut ed) = self.error_diffusion {
            ed.resize(ed_w, ed_h);
        }
    }

    /// Set a snapshot of runtime state for expanded chrome rendering.
    ///
    /// Call this once per frame before `render_with_overlay` when using windowed
    /// mode so the renderer can draw title/footer rows without needing mutable
    /// access to `RuntimeState`.
    pub fn set_chrome_snapshot(&mut self, s: ChromeSnapshot) {
        self.chrome_snapshot = Some(s);
    }

    /// Set the dithering mode.
    ///
    /// This may allocate or resize error diffusion buffers.
    pub fn set_dither_mode(&mut self, mode: DitherMode) {
        self.dither_mode = mode;
        let (ed_w, ed_h) = self
            .window_layout
            .as_ref()
            .map(|l| (l.sim_w, l.sim_h))
            .unwrap_or((self.width, self.height));
        self.error_diffusion = match mode {
            DitherMode::ErrorDiffusion { .. } | DitherMode::Hybrid { .. } => {
                let mut ed = ErrorDiffusion::new(ed_w, ed_h);
                ed.resize(ed_w, ed_h);
                Some(ed)
            }
            _ => None,
        };
    }

    /// Set the intensity mapping for non-linear color distribution.
    pub fn set_intensity_mapping(&mut self, mapping: Option<IntensityMapping>) {
        self.intensity_mapping = mapping;
    }

    /// Set the spatial palette-repeat cycle configuration.
    pub fn set_palette_cycle(&mut self, cycle: palette::PaletteCycle) {
        self.palette_cycle = cycle;
    }

    /// Get the current dithering mode.
    pub fn dither_mode(&self) -> DitherMode {
        self.dither_mode
    }

    /// Reset error diffusion accumulators.
    ///
    /// The render methods already do this at the start of each frame; exposed
    /// for callers that drive `FrameBuffer` construction themselves.
    pub fn reset_error_diffusion(&mut self) {
        if let Some(ref mut ed) = self.error_diffusion {
            ed.reset();
        }
    }

    /// Resize error diffusion buffers (e.g. after a terminal resize).
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
    pub fn set_palette(&mut self, palette: Palette) {
        self.palette = palette;
    }

    /// Set the hue shift in degrees, applied to palette colors at render time.
    pub fn set_hue_shift(&mut self, hue_shift: f32) {
        self.hue_shift = hue_shift;
    }

    /// Update the character set used for rendering.
    pub fn set_charset(&mut self, charset: Charset) {
        self.charset = charset;
    }

    /// Returns the active character set.
    pub fn charset(&self) -> &Charset {
        &self.charset
    }

    /// Returns the active intensity mapping, if any.
    pub fn intensity_mapping(&self) -> Option<&IntensityMapping> {
        self.intensity_mapping.as_ref()
    }

    /// Returns the active palette cycle.
    pub fn palette_cycle(&self) -> palette::PaletteCycle {
        self.palette_cycle
    }

    /// Set the glyph-selection config (lever 10).
    pub fn set_glyph(&mut self, glyph: charset::GlyphConfig) {
        self.glyph = glyph;
    }

    /// Returns the active glyph-selection config.
    pub fn glyph(&self) -> charset::GlyphConfig {
        self.glyph
    }

    /// Returns the active window frame style.
    pub fn window_frame(&self) -> crate::simulation::config::WindowFrame {
        self.window_frame
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

    /// Sets the resolved color-AA strength for the current charset.
    pub fn set_color_aa(&mut self, aa: crate::render::antialiasing::AaStrength) {
        self.color_aa = aa;
    }

    /// Returns the resolved color-AA strength for the current charset.
    pub fn color_aa(&self) -> crate::render::antialiasing::AaStrength {
        self.color_aa
    }

    /// Update the background color (the color drawn behind empty cells).
    pub fn set_background_color(&mut self, bg: Option<RgbColor>) {
        self.background_color = bg;
    }

    /// Returns the active background color, if any.
    pub fn background_color(&self) -> Option<RgbColor> {
        self.background_color
    }

    /// Returns the active palette.
    pub fn palette(&self) -> &Palette {
        &self.palette
    }

    /// Returns whether the palette is reversed.
    pub fn reverse_palette(&self) -> bool {
        self.reverse_palette
    }

    /// Returns whether the palette is inverted.
    pub fn invert_palette(&self) -> bool {
        self.invert_palette
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

    /// Set temporal-color modulation (lever 3). strength 0.0 disables it.
    pub fn set_temporal(
        &mut self,
        strength: f32,
        mode: palette::TemporalMode,
        accent: Option<palette::RgbColor>,
    ) {
        self.temporal_strength = strength;
        self.temporal_mode = mode;
        self.temporal_accent = accent;
    }

    /// Get a mutable reference to the standard output.
    pub fn stdout_mut(&mut self) -> &mut Stdout {
        &mut self.stdout
    }

    /// Render a frame to the terminal.
    pub fn render(
        &mut self,
        downsampled: &[DownsampleCell],
        max_trail_value: f32,
    ) -> io::Result<()> {
        if let Some(ref mut ed) = self.error_diffusion {
            ed.reset();
        }
        let species_colors = if self.species_colors_enabled {
            Some(self.species_rgb_colors.clone())
        } else {
            None
        };
        let mut buffer = if let Some(ref layout) = self.window_layout {
            FrameBuffer::from_downsampled_at(
                downsampled,
                layout.sim_w,
                layout.sim_h,
                self.width,
                self.height,
                layout.sim_x,
                layout.sim_y,
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
                species_colors,
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
                self.temporal_strength,
                self.temporal_mode,
                self.palette_cycle,
                self.glyph,
                self.temporal_accent,
                self.color_aa,
            )
        } else {
            FrameBuffer::from_downsampled(
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
                species_colors,
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
                self.temporal_strength,
                self.temporal_mode,
                self.palette_cycle,
                self.glyph,
                self.temporal_accent,
                self.color_aa,
            )
        };

        if let Some(ref layout) = self.window_layout {
            use crate::render::window::FallbackMode;
            if !matches!(layout.fallback, FallbackMode::Fullscreen)
                && self.window_frame.is_visible()
            {
                let accent = palette::palette_accent_color(
                    &self.palette,
                    self.reverse_palette,
                    self.invert_palette,
                    self.hue_shift,
                    self.intensity_mapping.as_ref(),
                );
                buffer.render_window_frame_at(
                    self.window_frame,
                    accent,
                    layout.frame_x,
                    layout.frame_y,
                    layout.frame_w,
                    layout.frame_h,
                    self.background_color,
                    layout.sim_x - layout.frame_x,
                    layout.sim_y - layout.frame_y,
                );
            }
        }

        execute!(self.stdout, &buffer)
    }

    /// Render a frame with text overlays.
    ///
    /// Draws the sim frame, then composites overlays on top in z-order:
    /// pause effects, window frame/chrome, controls, status line, notification,
    /// dashboard, and modals (config browser/save, hints, comparison, palette editor).
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
        dirty_guard_lines: Option<(&RenderedOverlay, usize, usize)>,
        keyboard_hints_lines: Option<(&RenderedOverlay, usize, usize)>,
        preset_comparison_lines: Option<(&RenderedOverlay, usize, usize)>,
        palette_editor_overlay: Option<(&RenderedOverlay, usize, usize)>,
        ambient_overlay: Option<(&RenderedOverlay, usize, usize)>,
        panel_style: Option<&PanelStyle>,
        _focused_overlay: Option<crate::overlay::OverlayType>,
        pause_style: PauseStyle,
        pause_pulse_draw_mode: bool,
    ) -> io::Result<()> {
        if let Some(ref mut ed) = self.error_diffusion {
            ed.reset();
        }
        let species_colors_rwo = if self.species_colors_enabled {
            Some(self.species_rgb_colors.clone())
        } else {
            None
        };
        let mut buffer = if let Some(ref layout) = self.window_layout {
            FrameBuffer::from_downsampled_at(
                downsampled,
                layout.sim_w,
                layout.sim_h,
                self.width,
                self.height,
                layout.sim_x,
                layout.sim_y,
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
                species_colors_rwo,
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
                self.temporal_strength,
                self.temporal_mode,
                self.palette_cycle,
                self.glyph,
                self.temporal_accent,
                self.color_aa,
            )
        } else {
            FrameBuffer::from_downsampled(
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
                species_colors_rwo,
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
                self.temporal_strength,
                self.temporal_mode,
                self.palette_cycle,
                self.glyph,
                self.temporal_accent,
                self.color_aa,
            )
        };

        if let Some(fc) = pause_frame {
            buffer.apply_pause_effect(pause_style, fc, pause_pulse_draw_mode);
        }

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

        // Palette accent color used for accented title badges and border.
        let accent = palette::palette_accent_color(
            &self.palette,
            self.reverse_palette,
            self.invert_palette,
            self.hue_shift,
            self.intensity_mapping.as_ref(),
        );

        if self.window_frame.is_visible() {
            if let Some(ref layout) = self.window_layout {
                use crate::render::window::FallbackMode;
                if !matches!(layout.fallback, FallbackMode::Fullscreen) {
                    buffer.render_window_frame_at(
                        self.window_frame,
                        accent,
                        layout.frame_x,
                        layout.frame_y,
                        layout.frame_w,
                        layout.frame_h,
                        self.background_color,
                        layout.sim_x - layout.frame_x,
                        layout.sim_y - layout.frame_y,
                    );
                }
            } else {
                buffer.render_window_frame(
                    self.window_frame,
                    accent,
                    self.background_color,
                    crate::render::window::FRAME_RING_COLS,
                    crate::render::window::FRAME_RING_ROWS,
                );
            }
        }

        // Draw expanded chrome (title block + footer) when in non-Minimal chrome state.
        // This includes Expanded, ModalPane, and FadingOut (fade has content to alpha-blend).
        if let (Some(ref layout), Some(ref snap)) = (&self.window_layout, &self.chrome_snapshot) {
            use crate::terminal::state::ChromeState;
            if snap.chrome_state != ChromeState::Minimal {
                let title = ExpandedChromeOverlay::build_title_block(
                    snap.preset,
                    snap.palette.clone(),
                    &snap.charset_str,
                    snap.population,
                    layout.sim_w,
                );
                let footer_st = panel_style.unwrap_or(&crate::render::theme::GRUVBOX_DARK);
                let (footer_status, footer_colors) = ExpandedChromeOverlay::build_footer_status(
                    snap.is_paused,
                    snap.preset,
                    snap.time_scale,
                    snap.palette.clone(),
                    snap.dither_mode,
                    layout.sim_w,
                    Some(snap.population),
                    snap.diffusion_kernel.as_deref(),
                    snap.can_undo,
                    snap.can_redo,
                    Some(accent),
                    footer_st,
                );
                let footer_keys = ExpandedChromeOverlay::build_footer_keybinds(
                    snap.chrome_state == ChromeState::ModalPane,
                    layout.sim_w,
                );
                // Dim text color for footer secondary row
                let text_color = RgbColor::new(168, 153, 132);
                buffer.draw_expanded_chrome(
                    layout.sim_x,
                    layout.sim_y,
                    layout.sim_w,
                    layout.sim_h,
                    &title,
                    &footer_status,
                    &footer_colors,
                    &footer_keys,
                    accent,
                    text_color,
                );
            }
            // Apply fade-out alpha when chrome is collapsing.
            if let ChromeState::FadingOut(progress) = snap.chrome_state {
                buffer.fade_chrome_rows(
                    layout.sim_x,
                    layout.sim_y,
                    layout.sim_w,
                    layout.sim_h,
                    progress,
                );
            }
        }

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
                // Apply breathed accent color to border chars (title-box signature).
                // Breath: lerp accent toward muted by (1 - breath(clock, 5s, 12%)).
                let title_accent = if let Some(style) = panel_style {
                    let pulse = breath(self.phase_clock, 5.0, 0.12);
                    lerp_rgb(style.muted, accent, pulse)
                } else {
                    accent
                };
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
                                ColorMode::TrueColor => {
                                    buf.cells[idx].fg_color_rgb = Some(title_accent)
                                }
                                _ => {
                                    buf.cells[idx].fg_color_256 =
                                        Some(palette::rgb_to_256(title_accent))
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

        if let Some((overlay, x, y)) = pause_badge_overlay {
            draw_overlay(&mut buffer, overlay, x, y, &OverlayConfig::NOTIFICATION);
        }

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

        if let Some((overlay, x, y)) = notification_line {
            draw_overlay(&mut buffer, overlay, x, y, &OverlayConfig::NOTIFICATION);
        }

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

        // Dirty-state guard overlay (modal, on top)
        if let Some((overlay, x, y)) = dirty_guard_lines {
            draw_overlay(&mut buffer, overlay, x, y, &OverlayConfig::DIRTY_GUARD);
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

        // Ambient BASE surface — bottom-docked always-on strip (Task 12).
        // Composited last: the ambient strip paints over the bottom rows, above all other overlays.
        if let Some((overlay, x, y)) = ambient_overlay {
            if let Some(ref rich) = overlay.rich_lines {
                buffer.draw_rich_overlay_dim(rich, x, y, 1.0);
            }
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
        dirty_guard_lines: Option<(&RenderedOverlay, usize, usize)>,
        keyboard_hints_lines: Option<(&RenderedOverlay, usize, usize)>,
        preset_comparison_lines: Option<(&RenderedOverlay, usize, usize)>,
        palette_editor_overlay: Option<(&RenderedOverlay, usize, usize)>,
        ambient_overlay: Option<(&RenderedOverlay, usize, usize)>,
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

        // Determine render dimensions: use sim_w/sim_h from window layout if present,
        // otherwise fill the full terminal.
        let (render_w, render_h, render_x, render_y) = if let Some(ref layout) = self.window_layout
        {
            (layout.sim_w, layout.sim_h, layout.sim_x, layout.sim_y)
        } else {
            (self.width, self.height, 0, 0)
        };

        // Get or create pre-allocated frame buffer at sim render dimensions
        let width = render_w;
        let height = render_h;
        if self
            .frame_buffer
            .as_ref()
            .map(|f| f.width() != width || f.height() != height)
            .unwrap_or(true)
        {
            self.frame_buffer = Some(crate::render::downsample::DownsampledFrame::new(
                width, height,
            ));
        }

        for (trail_map, species_color) in trail_maps {
            if let Some(ref mut downsampled) = self.frame_buffer {
                downsample_multi_species(
                    &[(trail_map, 0)],
                    sim_width,
                    sim_height,
                    width,
                    height,
                    downsampled,
                );

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
                    width,
                    height,
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
                    0.0, // temporal OFF for multi-species
                    palette::TemporalMode::Hue,
                    self.palette_cycle,
                    self.glyph,
                    None,
                    crate::render::antialiasing::AaStrength::Off,
                );

                // Blit non-blank cells from species_buffer into main buffer at (render_x, render_y)
                for sy in 0..height {
                    for sx in 0..width {
                        let src_idx = sy * width + sx;
                        let src_cell = &species_buffer.cells[src_idx];
                        if src_cell.char != ' ' {
                            let dst_x = render_x + sx;
                            let dst_y = render_y + sy;
                            if dst_x < self.width && dst_y < self.height {
                                let dst_idx = dst_y * self.width + dst_x;
                                buffer.cells[dst_idx] = *src_cell;
                            }
                        }
                    }
                }
            }
        }

        if let Some(fc) = pause_frame {
            buffer.apply_pause_effect(pause_style, fc, pause_pulse_draw_mode);
        }

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

        if self.window_frame.is_visible() {
            if let Some(ref layout) = self.window_layout {
                use crate::render::window::FallbackMode;
                if !matches!(layout.fallback, FallbackMode::Fullscreen) {
                    buffer.render_window_frame_at(
                        self.window_frame,
                        accent,
                        layout.frame_x,
                        layout.frame_y,
                        layout.frame_w,
                        layout.frame_h,
                        self.background_color,
                        layout.sim_x - layout.frame_x,
                        layout.sim_y - layout.frame_y,
                    );
                }
            } else {
                buffer.render_window_frame(
                    self.window_frame,
                    accent,
                    self.background_color,
                    crate::render::window::FRAME_RING_COLS,
                    crate::render::window::FRAME_RING_ROWS,
                );
            }
        }

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
                // Apply breathed accent color to border chars (title-box signature).
                let title_accent = if let Some(style) = panel_style_ms {
                    let pulse = breath(self.phase_clock, 5.0, 0.12);
                    lerp_rgb(style.muted, accent, pulse)
                } else {
                    accent
                };
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
                                ColorMode::TrueColor => {
                                    buf.cells[idx].fg_color_rgb = Some(title_accent)
                                }
                                _ => {
                                    buf.cells[idx].fg_color_256 =
                                        Some(palette::rgb_to_256(title_accent))
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

        if let Some((overlay, x, y)) = pause_badge_overlay {
            draw_ms_overlay(&mut buffer, overlay, x, y, &OverlayConfig::NOTIFICATION);
        }

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

        if let Some((overlay, x, y)) = notification_line {
            draw_ms_overlay(&mut buffer, overlay, x, y, &OverlayConfig::NOTIFICATION);
        }

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

        // Dirty-state guard overlay (modal, on top)
        if let Some((overlay, x, y)) = dirty_guard_lines {
            draw_ms_overlay(&mut buffer, overlay, x, y, &OverlayConfig::DIRTY_GUARD);
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

        // Ambient BASE surface — bottom-docked always-on strip (Task 12).
        // Composited last: the ambient strip paints over the bottom rows, above all other overlays.
        if let Some((overlay, x, y)) = ambient_overlay {
            if let Some(ref rich) = overlay.rich_lines {
                buffer.draw_rich_overlay_dim(rich, x, y, 1.0);
            }
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
    fn setters_are_observable_via_getters() {
        use crate::simulation::config::WindowFrame;
        let mut r = TerminalRenderer::new(
            80,
            24,
            Palette::Organic,
            Charset::HalfBlock,
            false,
            false,
            ColorMode::TrueColor,
            None,
        );
        r.set_charset(Charset::Ascii);
        assert_eq!(r.charset(), &Charset::Ascii);
        r.set_window_frame(WindowFrame::Glow);
        assert_eq!(r.window_frame(), WindowFrame::Glow);
        r.set_intensity_mapping(None);
        assert!(r.intensity_mapping().is_none());
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
            None, // ambient_overlay
            None,
            PauseStyle::Vignette,
            false,
        );

        assert!(result.is_ok());
    }
}
