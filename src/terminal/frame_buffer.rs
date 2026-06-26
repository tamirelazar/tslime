//! Frame buffer for terminal rendering.
//!
//! This module provides an off-screen cell grid that is rebuilt each frame,
//! storing character and color information for each cell in the terminal grid,
//! then emitted as a single ANSI string.

use crate::cli::ColorMode;
use crate::cli::Palette;
use crate::cli::PauseStyle;
use crate::config_defaults::TrailAgeMode;
use crate::render::charset::{self, Charset};
use crate::render::dither::{self, DitherMode};
use crate::render::downsample::Cell as DownsampleCell;
use crate::render::error_diffusion::ErrorDiffusion;
use crate::render::palette;
use crate::render::palette::IntensityMapping;
use crate::render::palette::RgbColor;
use crate::render::panel::RichCell;
use crossterm::{execute, Command};
use std::fmt;
use std::io;

#[derive(Clone, Copy)]
/// A single cell in the frame buffer.
pub struct Cell {
    /// Character to display.
    pub char: char,
    /// Foreground color in ANSI 256 format.
    pub fg_color_256: Option<u8>,
    /// Background color in ANSI 256 format.
    pub bg_color_256: Option<u8>,
    /// Foreground color in RGB format.
    pub fg_color_rgb: Option<RgbColor>,
    /// Background color in RGB format.
    pub bg_color_rgb: Option<RgbColor>,
}

impl Cell {
    /// Creates a new cell with the given character and no colors.
    pub fn new(ch: char) -> Self {
        Self {
            char: ch,
            fg_color_256: None,
            bg_color_256: None,
            fg_color_rgb: None,
            bg_color_rgb: None,
        }
    }

    /// Sets the foreground RGB color and returns self for chaining.
    pub fn with_fg(mut self, color: RgbColor) -> Self {
        self.fg_color_rgb = Some(color);
        self
    }

    /// Sets the background RGB color and returns self for chaining.
    pub fn with_bg(mut self, color: RgbColor) -> Self {
        self.bg_color_rgb = Some(color);
        self
    }
}

/// An off-screen cell grid for terminal rendering.
///
/// Stores character and color information for each cell in the terminal grid;
/// overlays draw into it before the whole frame is emitted as one ANSI string.
pub struct FrameBuffer {
    /// Width of the frame buffer in cells.
    pub(crate) width: usize,
    /// Height of the frame buffer in cells.
    pub(crate) height: usize,
    /// Direct access to cells for rendering purposes.
    pub(crate) cells: Vec<Cell>,
    /// Color mode used for rendering.
    pub(crate) color_mode: ColorMode,
    /// Whether species colors are enabled.
    pub(crate) species_colors_enabled: bool,
    /// RGB colors for species rendering.
    pub(crate) species_rgb_colors: Vec<RgbColor>,
    background_color: Option<RgbColor>,
    ascii_contrast: f32,
    /// Resolved color-AA strength for this frame's charset (Off when disabled
    /// or charset ineligible). Drives the blurred color fields below.
    aa_strength: crate::render::antialiasing::AaStrength,
    /// Per-cell blurred base brightness (color only); empty when AA inactive.
    aa_brightness: Vec<f32>,
    /// Per-cell blurred temporal diff_norm (accent color); empty when inactive.
    aa_diff_norm: Vec<f32>,
}

impl FrameBuffer {
    /// Get the width of the frame buffer.
    pub fn width(&self) -> usize {
        self.width
    }

    /// Get the height of the frame buffer.
    pub fn height(&self) -> usize {
        self.height
    }

    /// Create a new empty frame buffer.
    pub fn new(
        width: usize,
        height: usize,
        color_mode: ColorMode,
        background_color: Option<RgbColor>,
    ) -> Self {
        let (bg_color_256, bg_color_rgb) = if let Some(bg) = background_color {
            match color_mode {
                ColorMode::TrueColor => (None, Some(bg)),
                _ => (Some(palette::rgb_to_256(bg)), None),
            }
        } else {
            (None, None)
        };

        Self {
            width,
            height,
            cells: vec![
                Cell {
                    char: ' ',
                    fg_color_256: None,
                    bg_color_256,
                    fg_color_rgb: None,
                    bg_color_rgb,
                };
                width * height
            ],
            color_mode,
            species_colors_enabled: false,
            species_rgb_colors: Vec::new(),
            background_color,
            ascii_contrast: 1.5,
            aa_strength: crate::render::antialiasing::AaStrength::Off,
            aa_brightness: Vec::new(),
            aa_diff_norm: Vec::new(),
        }
    }

    /// Sets a cell at the specified coordinates.
    pub fn set_cell(&mut self, x: usize, y: usize, cell: Cell) {
        if x < self.width && y < self.height {
            self.cells[y * self.width + x] = cell;
        }
    }

    /// Resets every cell to the matte background (blank space + background color),
    /// wiping any simulation content and window frame already drawn. Used to give
    /// a clean backdrop behind a modal overlay that would otherwise overlap the
    /// frame border and read as visual noise.
    pub fn fill_background(&mut self) {
        let (bg_color_256, bg_color_rgb) = match (self.background_color, self.color_mode) {
            (Some(bg), ColorMode::TrueColor) => (None, Some(bg)),
            (Some(bg), _) => (Some(palette::rgb_to_256(bg)), None),
            (None, _) => (None, None),
        };
        let blank = Cell {
            char: ' ',
            fg_color_256: None,
            bg_color_256,
            fg_color_rgb: None,
            bg_color_rgb,
        };
        for cell in &mut self.cells {
            *cell = blank;
        }
    }

    #[cfg(test)]
    pub(crate) fn get_cell(&self, x: usize, y: usize) -> &Cell {
        &self.cells[y * self.width + x]
    }

    /// Renders a window frame onto the frame buffer.
    ///
    /// Uses the provided window frame mode and accent color from the theme.
    /// `background_color` fills the inner separator/negative ring.
    pub fn render_window_frame(
        &mut self,
        mode: crate::simulation::config::WindowFrame,
        accent_color: RgbColor,
        background_color: Option<RgbColor>,
        ring_cols: usize,
        ring_rows: usize,
    ) {
        use crate::render::window_frame::WindowFrameRenderer;
        let renderer =
            WindowFrameRenderer::new(mode, accent_color, background_color, ring_cols, ring_rows);
        renderer.render(self);
    }

    /// Renders a window frame at an arbitrary position within the buffer.
    ///
    /// Draws a `w × h` frame at `(x, y)` within `self`. Only non-blank cells from
    /// the frame are blitted so the sim content behind transparent frame areas is
    /// preserved.
    #[allow(clippy::too_many_arguments)]
    pub fn render_window_frame_at(
        &mut self,
        mode: crate::simulation::config::WindowFrame,
        accent_color: RgbColor,
        x: usize,
        y: usize,
        w: usize,
        h: usize,
        background_color: Option<RgbColor>,
        ring_cols: usize,
        ring_rows: usize,
    ) {
        use crate::render::window_frame::WindowFrameRenderer;
        // Render into a sub-buffer, then blit non-blank cells into self at (x, y)
        let mut sub = Self::new(w, h, self.color_mode, None);
        let renderer =
            WindowFrameRenderer::new(mode, accent_color, background_color, ring_cols, ring_rows);
        renderer.render(&mut sub);
        for sy in 0..h {
            for sx in 0..w {
                let src_cell = &sub.cells[sy * w + sx];
                let dst_x = x + sx;
                let dst_y = y + sy;
                let non_blank = src_cell.char != ' '
                    || src_cell.fg_color_rgb.is_some()
                    || src_cell.bg_color_rgb.is_some()
                    || src_cell.fg_color_256.is_some()
                    || src_cell.bg_color_256.is_some();
                if non_blank && dst_x < self.width && dst_y < self.height {
                    self.cells[dst_y * self.width + dst_x] = *src_cell;
                }
            }
        }
    }

    /// Checks whether a cell is at the outline/edge of a shape.
    ///
    /// A cell is an outline cell if any of its 4-connected neighbors (up, down,
    /// left, right) is empty (below threshold). Interior cells are fully
    /// surrounded by other active cells.
    fn is_outline_cell(
        downsampled: &[DownsampleCell],
        width: usize,
        x: usize,
        y: usize,
        max_trail_value: f32,
    ) -> bool {
        const EDGE_THRESHOLD: f32 = 0.05;
        let Some(height) = downsampled.len().checked_div(width) else {
            return true;
        };

        // Edge of the grid is always an outline
        if x == 0 || y == 0 || x + 1 >= width || y + 1 >= height {
            return true;
        }

        let threshold = EDGE_THRESHOLD * max_trail_value;

        // Check 4-connected neighbors
        let neighbors = [
            (x, y.wrapping_sub(1)), // up
            (x, y + 1),             // down
            (x.wrapping_sub(1), y), // left
            (x + 1, y),             // right
        ];

        for (nx, ny) in &neighbors {
            if *nx >= width || *ny >= height {
                return true; // Out of bounds = empty neighbor
            }
            let nidx = ny * width + nx;
            if nidx >= downsampled.len() {
                return true;
            }
            let ncell = &downsampled[nidx];
            let avg = (ncell.top + ncell.bottom) / 2.0;
            if avg <= threshold {
                return true; // Neighbor is empty, so we're on the outline
            }
        }

        false
    }

    /// Render a grid background pattern into the buffer.
    ///
    /// Used for overlaying a visual grid on top of empty space.
    pub fn render_grid_background(
        &mut self,
        x: usize,
        y: usize,
        grid_color: RgbColor,
        opacity: f32,
        on_vertical: bool,
        on_horizontal: bool,
    ) {
        if x >= self.width || y >= self.height {
            return;
        }

        let idx = y * self.width + x;
        let cell = &self.cells[idx];

        // Only render grid where there's no (or very dim) simulation content:
        // a space character, or a foreground color close to black.
        let is_empty = cell.char == ' '
            || match self.color_mode {
                ColorMode::TrueColor => cell.fg_color_rgb.map_or(true, |c| {
                    // Check if color is very dark (close to black)
                    (c.r as u32 + c.g as u32 + c.b as u32) < 30
                }),
                _ => {
                    // ANSI colors: check if it maps to something very dark
                    cell.fg_color_256.map_or(true, |c| {
                        let rgb = palette::ANSI_256_TO_RGB[c as usize];
                        (rgb.r as u32 + rgb.g as u32 + rgb.b as u32) < 30
                    })
                }
            };

        if is_empty {
            let dimmed_color = RgbColor {
                r: (grid_color.r as f32 * opacity) as u8,
                g: (grid_color.g as f32 * opacity) as u8,
                b: (grid_color.b as f32 * opacity) as u8,
            };

            // Choose character based on which lines intersect at this position
            let grid_char = match (on_vertical, on_horizontal) {
                (true, true) => '┼',   // Intersection
                (true, false) => '│',  // Vertical line
                (false, true) => '─',  // Horizontal line
                (false, false) => ' ', // Should not happen
            };

            let target_cell = &mut self.cells[idx];
            match self.color_mode {
                ColorMode::TrueColor => {
                    target_cell.fg_color_rgb = Some(dimmed_color);
                    target_cell.char = grid_char;
                    // Preserve background color if set
                    target_cell.bg_color_rgb = self.background_color;
                }
                _ => {
                    target_cell.fg_color_256 = Some(palette::rgb_to_256(dimmed_color));
                    target_cell.char = grid_char;
                    // Preserve background color if set
                    target_cell.bg_color_256 = self.background_color.map(palette::rgb_to_256);
                }
            }
        }
        // Non-empty cells are left alone — simulation content takes precedence.
    }

    /// Draw a solid panel background with left focus indicator.
    ///
    /// Used for rendering OpenCode-style panels with colored left edge.
    #[allow(clippy::too_many_arguments)]
    pub fn draw_panel_background(
        &mut self,
        x: usize,
        y: usize,
        width: usize,
        height: usize,
        bg_color: RgbColor,
        indicator_color: RgbColor,
        indicator_width: usize,
    ) {
        for dy in 0..height {
            let py = y + dy;
            if py >= self.height {
                break;
            }
            for dx in 0..width {
                let px = x + dx;
                if px >= self.width {
                    break;
                }

                let idx = py * self.width + px;
                let is_indicator = dx < indicator_width;

                let color = if is_indicator {
                    indicator_color
                } else {
                    bg_color
                };

                match self.color_mode {
                    ColorMode::TrueColor => {
                        self.cells[idx].bg_color_rgb = Some(color);
                    }
                    _ => {
                        self.cells[idx].bg_color_256 = Some(palette::rgb_to_256(color));
                    }
                }
            }
        }
    }

    /// Draw text directly onto the frame buffer.
    ///
    /// Used for UI overlays like help text, status lines, etc.
    pub fn draw_text_overlay<T: AsRef<str>>(
        &mut self,
        text_lines: &[T],
        start_x: usize,
        start_y: usize,
        fg_color: u8,
        bg_color: Option<u8>,
    ) {
        self.draw_text_overlay_with_panel(
            text_lines, start_x, start_y, fg_color, bg_color, None, None, 0,
        )
    }

    /// Draw text with optional panel background onto the frame buffer.
    ///
    /// Supports OpenCode-style panels with colored left focus indicator.
    #[allow(clippy::too_many_arguments)]
    pub fn draw_text_overlay_with_panel<T: AsRef<str>>(
        &mut self,
        text_lines: &[T],
        start_x: usize,
        start_y: usize,
        fg_color: u8,
        bg_color: Option<u8>,
        panel_bg_color: Option<RgbColor>,
        indicator_color: Option<RgbColor>,
        indicator_width: usize,
    ) {
        // Convert ANSI 256 colors to RGB for TrueColor mode
        let fg_rgb = palette::ANSI_256_TO_RGB[fg_color as usize];
        let bg_rgb = bg_color.map(|c| palette::ANSI_256_TO_RGB[c as usize]);

        // Draw panel background first if specified
        if let (Some(bg), Some(ind), w) = (panel_bg_color, indicator_color, indicator_width) {
            if w > 0 && start_y < self.height && start_x < self.width {
                let panel_width = text_lines
                    .iter()
                    .map(|l| l.as_ref().chars().count())
                    .max()
                    .unwrap_or(0)
                    .saturating_add(start_x)
                    .min(self.width.saturating_sub(start_x));

                let panel_height = text_lines.len().min(self.height.saturating_sub(start_y));

                for dy in 0..panel_height {
                    let y = start_y + dy;
                    for dx in 0..panel_width {
                        let x = start_x + dx;
                        if x >= self.width || y >= self.height {
                            break;
                        }
                        let idx = y * self.width + x;
                        let is_indicator = dx < w;
                        let color = if is_indicator { ind } else { bg };
                        match self.color_mode {
                            ColorMode::TrueColor => {
                                self.cells[idx].bg_color_rgb = Some(color);
                            }
                            _ => {
                                self.cells[idx].bg_color_256 = Some(palette::rgb_to_256(color));
                            }
                        }
                    }
                }
            }
        }

        for (dy, line) in text_lines.iter().enumerate() {
            let y = start_y + dy;
            if y >= self.height {
                break;
            }
            let line = line.as_ref();
            for (dx, ch) in line.chars().enumerate() {
                let x = start_x + dx;
                if x >= self.width {
                    break;
                }
                self.cells[y * self.width + x] = match self.color_mode {
                    ColorMode::TrueColor => Cell {
                        char: ch,
                        fg_color_256: None,
                        bg_color_256: None,
                        fg_color_rgb: Some(fg_rgb),
                        bg_color_rgb: bg_rgb,
                    },
                    _ => Cell {
                        char: ch,
                        fg_color_256: Some(fg_color),
                        bg_color_256: bg_color,
                        fg_color_rgb: None,
                        bg_color_rgb: None,
                    },
                };
            }
        }
    }

    /// Overlay per-cell color overrides produced by rich overlay builders.
    ///
    /// `rich` is a slice of rows; each row is a vec of `(char, fg_override, bg_override)`.
    /// Only cells whose override is `Some(…)` are modified; the underlying character and
    /// the existing `bg_color` from a previous `draw_text_overlay` call are preserved for
    /// cells whose override is `None`.
    ///
    /// Thin wrapper around [`draw_rich_overlay_solid`] with `solid = false`.
    pub fn draw_rich_overlay(&mut self, rich: &[Vec<RichCell>], start_x: usize, start_y: usize) {
        self.draw_rich_overlay_solid(rich, start_x, start_y, false);
    }

    /// Like [`draw_rich_overlay`] but with a `solid` flag controlling space cells.
    ///
    /// `solid = false` — space chars are transparent (the sim shows through).
    /// `solid = true` — space chars are written too, so a bordered modal reads
    /// as a filled box rather than a colored mask.
    pub fn draw_rich_overlay_solid(
        &mut self,
        rich: &[Vec<RichCell>],
        start_x: usize,
        start_y: usize,
        solid: bool,
    ) {
        for (dy, row) in rich.iter().enumerate() {
            let y = start_y + dy;
            if y >= self.height {
                break;
            }
            for (dx, &(ch, fg_override, bg_override)) in row.iter().enumerate() {
                let x = start_x + dx;
                if x >= self.width {
                    break;
                }
                let idx = y * self.width + x;
                // Space stays transparent unless `solid` (see method doc).
                if ch != ' ' || solid {
                    self.cells[idx].char = ch;
                }
                if let Some(fg) = fg_override {
                    match self.color_mode {
                        ColorMode::TrueColor => self.cells[idx].fg_color_rgb = Some(fg),
                        _ => self.cells[idx].fg_color_256 = Some(palette::rgb_to_256(fg)),
                    }
                }
                if let Some(bg) = bg_override {
                    match self.color_mode {
                        ColorMode::TrueColor => self.cells[idx].bg_color_rgb = Some(bg),
                        _ => self.cells[idx].bg_color_256 = Some(palette::rgb_to_256(bg)),
                    }
                }
            }
        }
    }

    /// Calculates the maximum brightness in a frame. Currently only used in tests.
    #[cfg_attr(not(test), allow(dead_code))]
    fn max_brightness(frame: &[DownsampleCell]) -> f32 {
        frame
            .iter()
            .map(|c| c.top.max(c.bottom))
            .fold(0.0, |acc, v| acc.max(v))
    }

    /// Create a frame buffer from downsampled simulation data.
    ///
    /// This is the main method for converting simulation state into renderable cells.
    /// It handles color mapping, character selection, dithering, and multi-species blending.
    #[allow(clippy::too_many_arguments)]
    pub fn from_downsampled(
        downsampled: &[DownsampleCell],
        width: usize,
        height: usize,
        max_trail_value: f32,
        palette: Palette,
        charset: Charset,
        reverse_palette: bool,
        invert_palette: bool,
        color_mode: ColorMode,
        hue_shift: f32,
        dither_mode: DitherMode,
        error_diffusion: &mut Option<ErrorDiffusion>,
        intensity_mapping: Option<&IntensityMapping>,
        species_colors_enabled: bool,
        species_rgb_colors: Option<Vec<RgbColor>>,
        background_color: Option<RgbColor>,
        ascii_contrast: f32,
        aux_frame: Option<&crate::render::downsample::AuxFrame>,
        trail_age_enabled: bool,
        trail_delta_enabled: bool,
        trail_age_hue_range: f32,
        trail_age_blend: f32,
        trail_delta_strength: f32,
        gradient_magnitude_enabled: bool,
        gradient_strength: f32,
        trail_age_mode: TrailAgeMode,
        trail_age_reverse: bool,
        temporal_strength: f32,
        temporal_mode: palette::TemporalMode,
        palette_cycle: palette::PaletteCycle,
        glyph: charset::GlyphConfig,
        temporal_accent: Option<palette::RgbColor>,
        aa_strength: crate::render::antialiasing::AaStrength,
    ) -> Self {
        let mut buffer = Self::new(width, height, color_mode, background_color);
        buffer.species_colors_enabled = species_colors_enabled;
        buffer.ascii_contrast = ascii_contrast;

        // Color anti-aliasing: when active for this (frame-uniform) charset,
        // pre-blur two per-cell color fields. The glyph/shape path below reads
        // raw quadrant data and is unaffected.
        // PERF: when active this allocates ~3 Vec<f32> per frame (base, diff, and
        // each blur_field output). Only runs for AA-eligible charsets (braille by
        // default). Revisit with reusable scratch buffers if a frame-time profile
        // shows braille-AA as hot; not worth the restructure unprofiled.
        use crate::render::antialiasing::{blur_field, charset_aa_eligible};
        let aa_active = aa_strength != crate::render::antialiasing::AaStrength::Off
            && charset_aa_eligible(&charset);
        buffer.aa_strength = if aa_active {
            aa_strength
        } else {
            crate::render::antialiasing::AaStrength::Off
        };
        if aa_active {
            let inv = if max_trail_value > 0.0 {
                1.0 / max_trail_value
            } else {
                0.0
            };
            // NOTE: this AA color field samples the raw normalized trail value,
            // intentionally BEFORE the per-cell trail-delta and edge-glow (gradient
            // magnitude) brightness boosts applied later in create_cell. So with AA
            // active AND those FX enabled, the AA-blurred base tone reflects the
            // un-boosted trail; the glyph and temporal-accent paths still see the
            // boosts. This is a deliberate, minor tonal divergence, not a bug.
            // Base color field: mean of top/bottom trail, normalized.
            let base: Vec<f32> = downsampled
                .iter()
                .take(width * height)
                .map(|d| ((d.top + d.bottom) * 0.5 * inv).clamp(0.0, 1.0))
                .collect();
            buffer.aa_brightness = blur_field(&base, width, height, aa_strength);
            // Temporal accent field: signed diff normalized (0 when no aux).
            if let Some(aux) = aux_frame {
                let diff: Vec<f32> = (0..width * height)
                    .map(|i| {
                        let cell = &aux.cells[i.min(aux.cells.len().saturating_sub(1))];
                        cell.signed_diff * inv
                    })
                    .collect();
                buffer.aa_diff_norm = blur_field(&diff, width, height, aa_strength);
            }
        }

        let species_colors_slice = species_rgb_colors.as_deref();

        for (idx, dcell) in downsampled.iter().enumerate() {
            if idx >= width * height {
                break;
            }

            let x = idx % width;
            let y = idx / width;

            if let Some(ref mut ed) = error_diffusion {
                if x == 0 {
                    ed.start_row(y);
                }
            }

            let mut top_brightness = if max_trail_value > 0.0 {
                dcell.top / max_trail_value
            } else {
                0.0
            };
            let mut bottom_brightness = if max_trail_value > 0.0 {
                dcell.bottom / max_trail_value
            } else {
                0.0
            };

            // Per-cell hue shift and brightness boost from aux data
            let (cell_hue_shift, cell_signed_diff) = if let Some(aux) = aux_frame {
                let aux_cell = &aux.cells[idx.min(aux.cells.len().saturating_sub(1))];

                // Delta → brightness boost
                if trail_delta_enabled {
                    let boost = aux_cell.delta * trail_delta_strength;
                    top_brightness = (top_brightness + boost).clamp(0.0, 1.0);
                    bottom_brightness = (bottom_brightness + boost).clamp(0.0, 1.0);
                }

                // Gradient magnitude → edge glow brightness boost
                if gradient_magnitude_enabled {
                    let boost = aux_cell.gradient * gradient_strength;
                    top_brightness = (top_brightness + boost).clamp(0.0, 1.0);
                    bottom_brightness = (bottom_brightness + boost).clamp(0.0, 1.0);
                }

                // Age → hue shift (blended with original hue shift)
                let computed_hue_shift = if trail_age_enabled {
                    let age_hue_shift = match trail_age_mode {
                        TrailAgeMode::Bidirectional => {
                            // Center around 0: age=0 → -range/2, age=0.5 → 0, age=1 → +range/2
                            // When reversed: age=0 → +range/2, age=1 → -range/2
                            let shift = (aux_cell.age - 0.5) * trail_age_hue_range;
                            if trail_age_reverse {
                                -shift
                            } else {
                                shift
                            }
                        }
                        TrailAgeMode::Alternating => {
                            // Spatial alternating: direction depends on cell position
                            let direction = if (x + y) % 2 == 0 { 1.0 } else { -1.0 };
                            aux_cell.age * trail_age_hue_range * direction
                        }
                    };
                    hue_shift + age_hue_shift * trail_age_blend
                } else {
                    hue_shift
                };

                (computed_hue_shift, aux_cell.signed_diff)
            } else {
                (hue_shift, 0.0)
            };

            let diff_norm = if buffer.aa_strength != crate::render::antialiasing::AaStrength::Off
                && !buffer.aa_diff_norm.is_empty()
                && idx < buffer.aa_diff_norm.len()
            {
                buffer.aa_diff_norm[idx]
            } else if max_trail_value > 0.0 {
                cell_signed_diff / max_trail_value
            } else {
                0.0
            };

            let cell = buffer.create_cell(
                x,
                y,
                top_brightness,
                bottom_brightness,
                downsampled,
                max_trail_value,
                &palette,
                charset.clone(),
                reverse_palette,
                invert_palette,
                color_mode,
                cell_hue_shift,
                dither_mode,
                error_diffusion,
                intensity_mapping,
                species_colors_enabled,
                species_colors_slice,
                diff_norm,
                temporal_strength,
                temporal_mode,
                palette_cycle,
                glyph,
                temporal_accent,
            );
            buffer.set_cell(x, y, cell);
        }

        buffer.species_rgb_colors = species_rgb_colors.unwrap_or_default();

        buffer
    }

    /// Create a frame buffer from downsampled simulation data, blitting the sim into a
    /// sub-region `(sim_x, sim_y)` of a `term_w × term_h` outer buffer.
    ///
    /// This enables windowed mode: the simulation renders at its natural `sim_w × sim_h`
    /// size, then the result is composited into the full terminal-sized buffer with the
    /// surrounding cells left blank.
    ///
    /// When `sim_x == 0 && sim_y == 0 && sim_w == term_w && sim_h == term_h` the inner
    /// `from_downsampled` result is returned directly (fast path — no copy).
    #[allow(clippy::too_many_arguments)]
    pub fn from_downsampled_at(
        downsampled: &[DownsampleCell],
        sim_w: usize,
        sim_h: usize,
        term_w: usize,
        term_h: usize,
        sim_x: usize,
        sim_y: usize,
        max_trail_value: f32,
        palette: Palette,
        charset: Charset,
        reverse_palette: bool,
        invert_palette: bool,
        color_mode: ColorMode,
        hue_shift: f32,
        dither_mode: DitherMode,
        error_diffusion: &mut Option<ErrorDiffusion>,
        intensity_mapping: Option<&IntensityMapping>,
        species_colors_enabled: bool,
        species_rgb_colors: Option<Vec<RgbColor>>,
        background_color: Option<RgbColor>,
        ascii_contrast: f32,
        aux_frame: Option<&crate::render::downsample::AuxFrame>,
        trail_age_enabled: bool,
        trail_delta_enabled: bool,
        trail_age_hue_range: f32,
        trail_age_blend: f32,
        trail_delta_strength: f32,
        gradient_magnitude_enabled: bool,
        gradient_strength: f32,
        trail_age_mode: TrailAgeMode,
        trail_age_reverse: bool,
        temporal_strength: f32,
        temporal_mode: palette::TemporalMode,
        palette_cycle: palette::PaletteCycle,
        glyph: charset::GlyphConfig,
        temporal_accent: Option<palette::RgbColor>,
        aa_strength: crate::render::antialiasing::AaStrength,
    ) -> Self {
        // Build sim buffer at sim dimensions
        let sim_buffer = Self::from_downsampled(
            downsampled,
            sim_w,
            sim_h,
            max_trail_value,
            palette,
            charset,
            reverse_palette,
            invert_palette,
            color_mode,
            hue_shift,
            dither_mode,
            error_diffusion,
            intensity_mapping,
            species_colors_enabled,
            species_rgb_colors,
            background_color,
            ascii_contrast,
            aux_frame,
            trail_age_enabled,
            trail_delta_enabled,
            trail_age_hue_range,
            trail_age_blend,
            trail_delta_strength,
            gradient_magnitude_enabled,
            gradient_strength,
            trail_age_mode,
            trail_age_reverse,
            temporal_strength,
            temporal_mode,
            palette_cycle,
            glyph,
            temporal_accent,
            aa_strength,
        );

        // Fast path: fullscreen — no blitting needed
        if sim_x == 0 && sim_y == 0 && sim_w == term_w && sim_h == term_h {
            return sim_buffer;
        }

        // Create outer buffer filled with blank cells, then blit sim into it
        let mut outer = Self::new(term_w, term_h, color_mode, background_color);
        for y in 0..sim_h {
            for x in 0..sim_w {
                let src_idx = y * sim_w + x;
                let dst_x = sim_x + x;
                let dst_y = sim_y + y;
                if dst_x < term_w && dst_y < term_h {
                    let dst_idx = dst_y * term_w + dst_x;
                    outer.cells[dst_idx] = sim_buffer.cells[src_idx];
                }
            }
        }
        outer
    }

    #[allow(clippy::too_many_arguments)]
    fn glyph_override(
        glyph: charset::GlyphConfig,
        cs: &Charset,
        top_adj: f32,
        bottom_adj: f32,
        downsampled: &[DownsampleCell],
        width: usize,
        x: usize,
        y: usize,
        max_trail_value: f32,
    ) -> Option<char> {
        use charset::GlyphSelection;
        let sel = glyph.selection?;
        let inv = if max_trail_value > 0.0 {
            1.0 / max_trail_value
        } else {
            0.0
        };
        let brightness_bucket =
            || charset::map_brightness((top_adj + bottom_adj) / 2.0, None, cs.clone());
        match sel {
            GlyphSelection::Shape => None,
            GlyphSelection::Brightness => match cs {
                Charset::Ascii | Charset::Braille | Charset::Sculpted => Some(brightness_bucket()),
                _ => None,
            },
            GlyphSelection::Hybrid => match cs {
                Charset::Ascii | Charset::CustomAscii(_) => {
                    let center = (top_adj + bottom_adj) / 2.0;
                    let mut n = [center; 9];
                    let height = downsampled.len().checked_div(width).unwrap_or(0);
                    for (k, (dy, dx)) in [
                        (-1i32, -1i32),
                        (-1, 0),
                        (-1, 1),
                        (0, -1),
                        (0, 0),
                        (0, 1),
                        (1, -1),
                        (1, 0),
                        (1, 1),
                    ]
                    .iter()
                    .enumerate()
                    {
                        let nx = x as i32 + dx;
                        let ny = y as i32 + dy;
                        if nx >= 0 && nx < width as i32 && ny >= 0 && ny < height as i32 {
                            let idx = (ny as usize) * width + nx as usize;
                            if idx < downsampled.len() {
                                let d = &downsampled[idx];
                                n[k] = ((d.top + d.bottom) * 0.5 * inv).clamp(0.0, 1.0);
                            }
                        }
                    }
                    Some(
                        charset::sobel_edge_glyph(&n, glyph.edge_threshold)
                            .unwrap_or_else(brightness_bucket),
                    )
                }
                _ => None,
            },
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn create_cell(
        &mut self,
        x: usize,
        y: usize,
        top: f32,
        bottom: f32,
        downsampled: &[DownsampleCell],
        max_trail_value: f32,
        palette: &Palette,
        charset: Charset,
        reverse_palette: bool,
        invert_palette: bool,
        color_mode: ColorMode,
        hue_shift: f32,
        dither_mode: DitherMode,
        error_diffusion: &mut Option<ErrorDiffusion>,
        intensity_mapping: Option<&IntensityMapping>,
        species_colors_enabled: bool,
        species_rgb_colors: Option<&[RgbColor]>,
        diff_norm: f32,
        temporal_strength: f32,
        temporal_mode: palette::TemporalMode,
        palette_cycle: palette::PaletteCycle,
        glyph: charset::GlyphConfig,
        temporal_accent: Option<palette::RgbColor>,
    ) -> Cell {
        const THRESHOLD: f32 = 0.01;
        let log_gaps = std::env::var("TSLIME_LOG_GAPS").is_ok();

        let levels = charset::charset_level_count(charset.clone());

        let (top_adj, bottom_adj) = match dither_mode {
            DitherMode::None => (top, bottom),
            DitherMode::Ordered { intensity, matrix } => (
                dither::apply_ordered_dither(x, y, top, intensity, matrix),
                dither::apply_ordered_dither(x, y, bottom, intensity, matrix),
            ),
            DitherMode::ErrorDiffusion { serpentine } => {
                if let Some(ed) = error_diffusion {
                    let top_quantized = dither::quantize_to_levels(top, levels);
                    let bottom_quantized = dither::quantize_to_levels(bottom, levels);
                    (
                        ed.apply_and_distribute(x, y, top, top_quantized, true, serpentine),
                        ed.apply_and_distribute(x, y, bottom, bottom_quantized, false, serpentine),
                    )
                } else {
                    (top, bottom)
                }
            }
            DitherMode::Hybrid {
                edge_threshold,
                intensity,
                matrix,
            } => {
                let variance = dither::local_variance(downsampled, self.width, x, y, 1);
                let top_adj = if variance > edge_threshold {
                    dither::apply_ordered_dither(x, y, top, intensity, matrix)
                } else if let Some(ed) = error_diffusion {
                    let quantized = dither::quantize_to_levels(top, levels);
                    ed.apply_and_distribute(x, y, top, quantized, true, true)
                } else {
                    top
                };
                let bottom_adj = if variance > edge_threshold {
                    dither::apply_ordered_dither(x, y, bottom, intensity, matrix)
                } else if let Some(ed) = error_diffusion {
                    let quantized = dither::quantize_to_levels(bottom, levels);
                    ed.apply_and_distribute(x, y, bottom, quantized, false, true)
                } else {
                    bottom
                };
                (top_adj, bottom_adj)
            }
        };

        if top_adj < THRESHOLD && bottom_adj < THRESHOLD {
            if log_gaps {
                eprintln!(
                    "Empty cell at ({}, {}): top_adj={}, bottom_adj={}, charset={:?}",
                    x, y, top_adj, bottom_adj, charset
                );
            }
            Cell {
                char: ' ',
                fg_color_256: None,
                bg_color_256: self.background_color.map(palette::rgb_to_256),
                fg_color_rgb: None,
                bg_color_rgb: self.background_color,
            }
        } else {
            let char = if let Some(c) = Self::glyph_override(
                glyph,
                &charset,
                top_adj,
                bottom_adj,
                downsampled,
                self.width,
                x,
                y,
                max_trail_value,
            ) {
                c
            } else if top_adj > THRESHOLD && bottom_adj > THRESHOLD {
                match charset {
                    Charset::HalfBlock => charset::map_vertical_block(top_adj, bottom_adj),
                    Charset::HalfBlockDual => charset::map_vertical_block(top_adj, bottom_adj),
                    Charset::Sculpted => {
                        let idx = y * self.width + x;
                        if idx < downsampled.len() {
                            let dcell = &downsampled[idx];
                            let inv = if max_trail_value > 0.0 {
                                1.0 / max_trail_value
                            } else {
                                0.0
                            };
                            if Self::is_outline_cell(downsampled, self.width, x, y, max_trail_value)
                            {
                                charset::map_sculpted_outline(
                                    dcell.top_left * inv,
                                    dcell.top_right * inv,
                                    dcell.bottom_left * inv,
                                    dcell.bottom_right * inv,
                                )
                            } else {
                                charset::map_vertical_block(top_adj, bottom_adj)
                            }
                        } else {
                            charset::map_vertical_block(top_adj, bottom_adj)
                        }
                    }
                    Charset::Braille => {
                        let idx = y * self.width + x;
                        if idx < downsampled.len() {
                            let dcell = &downsampled[idx];
                            let inv = if max_trail_value > 0.0 {
                                1.0 / max_trail_value
                            } else {
                                0.0
                            };
                            charset::map_shape_braille(
                                dcell.top_left * inv,
                                dcell.top_right * inv,
                                dcell.bottom_left * inv,
                                dcell.bottom_right * inv,
                                THRESHOLD,
                            )
                        } else {
                            charset::map_brightness(top_adj, Some(bottom_adj), charset.clone())
                        }
                    }
                    Charset::Quadrant => {
                        // Use quadrant values from downsampled cell
                        let idx = y * self.width + x;
                        if idx < downsampled.len() {
                            let dcell = &downsampled[idx];
                            let tl = if max_trail_value > 0.0 {
                                dcell.top_left / max_trail_value
                            } else {
                                0.0
                            };
                            let tr = if max_trail_value > 0.0 {
                                dcell.top_right / max_trail_value
                            } else {
                                0.0
                            };
                            let bl = if max_trail_value > 0.0 {
                                dcell.bottom_left / max_trail_value
                            } else {
                                0.0
                            };
                            let br = if max_trail_value > 0.0 {
                                dcell.bottom_right / max_trail_value
                            } else {
                                0.0
                            };
                            charset::map_quadrant(tl, tr, bl, br, THRESHOLD)
                        } else {
                            ' '
                        }
                    }
                    Charset::Shade => {
                        let avg = (top_adj + bottom_adj) / 2.0;
                        charset::map_shade(avg)
                    }
                    Charset::Points => {
                        let avg = (top_adj + bottom_adj) / 2.0;
                        charset::map_point(avg, 0.15)
                    }
                    Charset::Ascii => {
                        let idx = y * self.width + x;
                        if idx < downsampled.len() {
                            let dcell = &downsampled[idx];
                            let inv = if max_trail_value > 0.0 {
                                1.0 / max_trail_value
                            } else {
                                0.0
                            };
                            charset::map_shape_ascii(
                                dcell.top_left * inv,
                                dcell.top_right * inv,
                                dcell.bottom_left * inv,
                                dcell.bottom_right * inv,
                                self.ascii_contrast,
                            )
                        } else {
                            ' '
                        }
                    }
                    Charset::CustomAscii(_) => {
                        charset::map_brightness((top_adj + bottom_adj) / 2.0, None, charset.clone())
                    }
                }
            } else if top_adj > bottom_adj {
                match charset {
                    Charset::Braille => {
                        let idx = y * self.width + x;
                        if idx < downsampled.len() {
                            let dcell = &downsampled[idx];
                            let inv = if max_trail_value > 0.0 {
                                1.0 / max_trail_value
                            } else {
                                0.0
                            };
                            charset::map_shape_braille(
                                dcell.top_left * inv,
                                dcell.top_right * inv,
                                dcell.bottom_left * inv,
                                dcell.bottom_right * inv,
                                THRESHOLD,
                            )
                        } else {
                            charset::map_brightness(top_adj, Some(bottom_adj), charset.clone())
                        }
                    }
                    Charset::HalfBlockDual => charset::map_vertical_block(top_adj, bottom_adj),
                    Charset::HalfBlock => charset::map_vertical_block(top_adj, bottom_adj),
                    Charset::Sculpted => {
                        let idx = y * self.width + x;
                        if idx < downsampled.len() {
                            let dcell = &downsampled[idx];
                            let inv = if max_trail_value > 0.0 {
                                1.0 / max_trail_value
                            } else {
                                0.0
                            };
                            if Self::is_outline_cell(downsampled, self.width, x, y, max_trail_value)
                            {
                                charset::map_sculpted_outline(
                                    dcell.top_left * inv,
                                    dcell.top_right * inv,
                                    dcell.bottom_left * inv,
                                    dcell.bottom_right * inv,
                                )
                            } else {
                                charset::map_vertical_block(top_adj, bottom_adj)
                            }
                        } else {
                            charset::map_vertical_block(top_adj, bottom_adj)
                        }
                    }
                    Charset::Shade => {
                        let avg = (top_adj + bottom_adj) / 2.0;
                        charset::map_shade(avg)
                    }
                    Charset::Points => {
                        let avg = (top_adj + bottom_adj) / 2.0;
                        charset::map_point(avg, 0.15)
                    }
                    Charset::Quadrant => {
                        // Use quadrant values from downsampled cell
                        let idx = y * self.width + x;
                        if idx < downsampled.len() {
                            let dcell = &downsampled[idx];
                            let tl = if max_trail_value > 0.0 {
                                dcell.top_left / max_trail_value
                            } else {
                                0.0
                            };
                            let tr = if max_trail_value > 0.0 {
                                dcell.top_right / max_trail_value
                            } else {
                                0.0
                            };
                            let bl = if max_trail_value > 0.0 {
                                dcell.bottom_left / max_trail_value
                            } else {
                                0.0
                            };
                            let br = if max_trail_value > 0.0 {
                                dcell.bottom_right / max_trail_value
                            } else {
                                0.0
                            };
                            charset::map_quadrant(tl, tr, bl, br, THRESHOLD)
                        } else {
                            ' '
                        }
                    }
                    Charset::Ascii => {
                        let idx = y * self.width + x;
                        if idx < downsampled.len() {
                            let dcell = &downsampled[idx];
                            let inv = if max_trail_value > 0.0 {
                                1.0 / max_trail_value
                            } else {
                                0.0
                            };
                            charset::map_shape_ascii(
                                dcell.top_left * inv,
                                dcell.top_right * inv,
                                dcell.bottom_left * inv,
                                dcell.bottom_right * inv,
                                self.ascii_contrast,
                            )
                        } else {
                            charset::map_ascii_directional(top_adj, true)
                        }
                    }
                    Charset::CustomAscii(_) => {
                        charset::map_brightness(top_adj, None, charset.clone())
                    }
                }
            } else {
                match charset {
                    Charset::Braille => {
                        let idx = y * self.width + x;
                        if idx < downsampled.len() {
                            let dcell = &downsampled[idx];
                            let inv = if max_trail_value > 0.0 {
                                1.0 / max_trail_value
                            } else {
                                0.0
                            };
                            charset::map_shape_braille(
                                dcell.top_left * inv,
                                dcell.top_right * inv,
                                dcell.bottom_left * inv,
                                dcell.bottom_right * inv,
                                THRESHOLD,
                            )
                        } else {
                            charset::map_brightness(top_adj, Some(bottom_adj), charset.clone())
                        }
                    }
                    Charset::HalfBlockDual => charset::map_vertical_block(top_adj, bottom_adj),
                    Charset::HalfBlock => charset::map_vertical_block(top_adj, bottom_adj),
                    Charset::Sculpted => {
                        let idx = y * self.width + x;
                        if idx < downsampled.len() {
                            let dcell = &downsampled[idx];
                            let inv = if max_trail_value > 0.0 {
                                1.0 / max_trail_value
                            } else {
                                0.0
                            };
                            if Self::is_outline_cell(downsampled, self.width, x, y, max_trail_value)
                            {
                                charset::map_sculpted_outline(
                                    dcell.top_left * inv,
                                    dcell.top_right * inv,
                                    dcell.bottom_left * inv,
                                    dcell.bottom_right * inv,
                                )
                            } else {
                                charset::map_vertical_block(top_adj, bottom_adj)
                            }
                        } else {
                            charset::map_vertical_block(top_adj, bottom_adj)
                        }
                    }
                    Charset::Shade => {
                        let avg = (top_adj + bottom_adj) / 2.0;
                        charset::map_shade(avg)
                    }
                    Charset::Points => {
                        let avg = (top_adj + bottom_adj) / 2.0;
                        charset::map_point(avg, 0.15)
                    }
                    Charset::Quadrant => {
                        // Use quadrant values from downsampled cell
                        let idx = y * self.width + x;
                        if idx < downsampled.len() {
                            let dcell = &downsampled[idx];
                            let tl = if max_trail_value > 0.0 {
                                dcell.top_left / max_trail_value
                            } else {
                                0.0
                            };
                            let tr = if max_trail_value > 0.0 {
                                dcell.top_right / max_trail_value
                            } else {
                                0.0
                            };
                            let bl = if max_trail_value > 0.0 {
                                dcell.bottom_left / max_trail_value
                            } else {
                                0.0
                            };
                            let br = if max_trail_value > 0.0 {
                                dcell.bottom_right / max_trail_value
                            } else {
                                0.0
                            };
                            charset::map_quadrant(tl, tr, bl, br, THRESHOLD)
                        } else {
                            ' '
                        }
                    }
                    Charset::Ascii => {
                        let idx = y * self.width + x;
                        if idx < downsampled.len() {
                            let dcell = &downsampled[idx];
                            let inv = if max_trail_value > 0.0 {
                                1.0 / max_trail_value
                            } else {
                                0.0
                            };
                            charset::map_shape_ascii(
                                dcell.top_left * inv,
                                dcell.top_right * inv,
                                dcell.bottom_left * inv,
                                dcell.bottom_right * inv,
                                self.ascii_contrast,
                            )
                        } else {
                            charset::map_ascii_directional(bottom_adj, false)
                        }
                    }
                    Charset::CustomAscii(_) => {
                        charset::map_brightness(bottom_adj, None, charset.clone())
                    }
                }
            };

            let cell_avg = if top_adj > THRESHOLD && bottom_adj > THRESHOLD {
                (top_adj + bottom_adj) / 2.0
            } else if top_adj > bottom_adj {
                top_adj
            } else {
                bottom_adj
            };

            // Color anti-aliasing: when active, color subcell-shape charsets from
            // the pre-blurred base-brightness field (set in from_downsampled).
            // The glyph above used raw quadrant data, so shape stays crisp.
            let aa_idx = y * self.width + x;
            let brightness = if self.aa_strength != crate::render::antialiasing::AaStrength::Off
                && aa_idx < self.aa_brightness.len()
            {
                self.aa_brightness[aa_idx]
            } else {
                cell_avg
            };

            self.render_colored_cell(
                char,
                brightness,
                palette,
                reverse_palette,
                invert_palette,
                color_mode,
                hue_shift,
                intensity_mapping,
                species_colors_enabled,
                species_rgb_colors,
                diff_norm,
                temporal_strength,
                temporal_mode,
                palette_cycle,
                temporal_accent,
            )
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn render_colored_cell(
        &self,
        char: char,
        brightness: f32,
        palette: &Palette,
        reverse_palette: bool,
        invert_palette: bool,
        color_mode: ColorMode,
        hue_shift: f32,
        intensity_mapping: Option<&IntensityMapping>,
        species_colors_enabled: bool,
        species_rgb_colors: Option<&[RgbColor]>,
        diff_norm: f32,
        temporal_strength: f32,
        temporal_mode: palette::TemporalMode,
        palette_cycle: palette::PaletteCycle,
        temporal_accent: Option<palette::RgbColor>,
    ) -> Cell {
        match color_mode {
            ColorMode::TrueColor => {
                let rgb = if species_colors_enabled {
                    let base_color = species_rgb_colors
                        .and_then(|colors| colors.first())
                        .copied()
                        .unwrap_or(RgbColor {
                            r: 128,
                            g: 128,
                            b: 128,
                        });
                    palette::map_species_brightness_rgb(brightness, base_color, reverse_palette)
                } else {
                    palette::colorize_subpixel(
                        brightness,
                        palette.clone(),
                        reverse_palette,
                        invert_palette,
                        hue_shift,
                        intensity_mapping,
                        diff_norm,
                        temporal_strength,
                        temporal_mode,
                        palette_cycle,
                        temporal_accent,
                    )
                };
                Cell {
                    char,
                    fg_color_256: None,
                    bg_color_256: None,
                    fg_color_rgb: Some(rgb),
                    bg_color_rgb: self.background_color,
                }
            }
            _ => {
                let color = if species_colors_enabled {
                    let base_color = species_rgb_colors
                        .and_then(|colors| colors.first())
                        .copied()
                        .unwrap_or(RgbColor {
                            r: 128,
                            g: 128,
                            b: 128,
                        });
                    palette::map_species_brightness(brightness, base_color, reverse_palette)
                } else {
                    palette::map_brightness_cycled(
                        brightness,
                        palette.clone(),
                        reverse_palette,
                        invert_palette,
                        intensity_mapping,
                        palette_cycle,
                    )
                };
                Cell {
                    char,
                    fg_color_256: Some(color),
                    bg_color_256: self.background_color.map(palette::rgb_to_256),
                    fg_color_rgb: None,
                    bg_color_rgb: None,
                }
            }
        }
    }

    fn ansi_color_code(color: u8, is_fg: bool) -> String {
        if is_fg {
            format!("\x1b[38;5;{}m", color)
        } else {
            format!("\x1b[48;5;{}m", color)
        }
    }

    fn truecolor_code(r: u8, g: u8, b: u8, is_fg: bool) -> String {
        if is_fg {
            format!("\x1b[38;2;{};{};{}m", r, g, b)
        } else {
            format!("\x1b[48;2;{};{};{}m", r, g, b)
        }
    }

    /// Build the final ANSI string for the frame.
    ///
    /// Optimizes output by only emitting color codes when they change.
    pub fn build_frame_string(&self, plain_output: bool, color_mode: ColorMode) -> String {
        let mut output = String::new();

        if !plain_output {
            // Begin Synchronized Update (DECSET 2026): tell the terminal to buffer
            // this frame and repaint it atomically, preventing mid-write tearing.
            // Terminals that don't support it ignore the sequence harmlessly.
            output.push_str("\x1b[?2026h");
            output.push_str("\x1b[H");
        }

        let mut last_fg_256: Option<u8> = None;
        let mut last_bg_256: Option<u8> = None;
        let mut last_fg_rgb: Option<RgbColor> = None;
        let mut last_bg_rgb: Option<RgbColor> = None;

        for y in 0..self.height {
            if !plain_output {
                // Move to the start of this row once. Cells are emitted left-to-right
                // and every glyph is single-width, so the cursor advances naturally as
                // we print — a per-cell absolute `\x1b[y;xH` move (the old code) added
                // ~7-10 bytes to every cell, bloating the frame ~7x. Smaller frames
                // mean less terminal write back-pressure, which keeps FPS (and thus the
                // fixed-step sim speed) stable while a key is held.
                output.push_str(&format!("\x1b[{};1H", y + 1));
            }

            for x in 0..self.width {
                let cell = self.cells[y * self.width + x];

                if !plain_output {
                    match color_mode {
                        ColorMode::TrueColor => {
                            if let Some(fg) = cell.fg_color_rgb {
                                if last_fg_rgb != Some(fg) {
                                    output.push_str(&Self::truecolor_code(fg.r, fg.g, fg.b, true));
                                    last_fg_rgb = Some(fg);
                                }
                            } else if last_fg_rgb.is_some() {
                                output.push_str("\x1b[39m");
                                last_fg_rgb = None;
                            }

                            if let Some(bg) = cell.bg_color_rgb {
                                if last_bg_rgb != Some(bg) {
                                    output.push_str(&Self::truecolor_code(bg.r, bg.g, bg.b, false));
                                    last_bg_rgb = Some(bg);
                                }
                            } else if last_bg_rgb.is_some() {
                                output.push_str("\x1b[49m");
                                last_bg_rgb = None;
                            }
                        }
                        _ => {
                            if let Some(fg) = cell.fg_color_256 {
                                if last_fg_256 != Some(fg) {
                                    output.push_str(&Self::ansi_color_code(fg, true));
                                    last_fg_256 = Some(fg);
                                }
                            } else if last_fg_256.is_some() {
                                output.push_str("\x1b[39m");
                                last_fg_256 = None;
                            }

                            if let Some(bg) = cell.bg_color_256 {
                                if last_bg_256 != Some(bg) {
                                    output.push_str(&Self::ansi_color_code(bg, false));
                                    last_bg_256 = Some(bg);
                                }
                            } else if last_bg_256.is_some() {
                                output.push_str("\x1b[49m");
                                last_bg_256 = None;
                            }
                        }
                    }
                }

                output.push(cell.char);
            }
        }

        if !plain_output
            && (last_fg_256.is_some()
                || last_bg_256.is_some()
                || last_fg_rgb.is_some()
                || last_bg_rgb.is_some())
        {
            output.push_str("\x1b[0m");
        }

        if !plain_output {
            // End Synchronized Update (DECRST 2026): flush the buffered frame.
            output.push_str("\x1b[?2026l");
        }

        output
    }

    /// Darken all cells and add scanlines for VCR freeze-frame look.
    ///
    /// Called after `from_downsampled` when the simulation is paused.
    /// - All cells: multiply RGB by `DIM` (~0.40)
    /// - Even rows: multiply again by `SCANLINE` (~0.55) to create CRT scanline effect
    /// - Empty cells: draw scanline pattern to make effect visible
    pub fn apply_vcr_pause_effect(&mut self, frame_counter: u64) {
        const DIM: f32 = 0.40;
        const SCANLINE_DARK: f32 = 0.55; // even rows get this multiplier on top of DIM
        const NOISE_AMOUNT: f32 = 0.08; // ±8% brightness jitter
        const SCANLINE_CHAR: char = '▒'; // Block character for scanlines
        const SCANLINE_COLOR: RgbColor = RgbColor {
            r: 40,
            g: 40,
            b: 40,
        };

        for y in 0..self.height {
            let is_scanline_row = y % 2 == 0;
            let scanline = if is_scanline_row { SCANLINE_DARK } else { 1.0 };
            let base_factor = DIM * scanline;

            for x in 0..self.width {
                // Deterministic noise from position + frame
                let hash = ((x as u64).wrapping_mul(2654435761)
                    ^ (y as u64).wrapping_mul(2246822519)
                    ^ frame_counter) as u32;
                let noise = ((hash % 256) as f32 / 255.0 - 0.5) * 2.0 * NOISE_AMOUNT;
                let factor = (base_factor + noise).clamp(0.05, 0.6);

                let idx = y * self.width + x;
                let cell = &mut self.cells[idx];

                let is_empty = cell.char == ' ' && cell.fg_color_rgb.is_none();

                if is_empty && is_scanline_row {
                    // Draw scanline on empty cells in even rows
                    cell.char = SCANLINE_CHAR;
                    cell.fg_color_rgb = Some(SCANLINE_COLOR);
                } else {
                    // Apply dimming to existing cells
                    if let Some(c) = cell.fg_color_rgb {
                        cell.fg_color_rgb = Some(RgbColor {
                            r: (c.r as f32 * factor) as u8,
                            g: (c.g as f32 * factor) as u8,
                            b: (c.b as f32 * factor) as u8,
                        });
                    }
                    if let Some(c) = cell.bg_color_rgb {
                        cell.bg_color_rgb = Some(RgbColor {
                            r: (c.r as f32 * factor) as u8,
                            g: (c.g as f32 * factor) as u8,
                            b: (c.b as f32 * factor) as u8,
                        });
                    }
                    if let Some(c) = cell.fg_color_256 {
                        let rgb = palette::ANSI_256_TO_RGB[c as usize];
                        let dimmed = RgbColor {
                            r: (rgb.r as f32 * factor) as u8,
                            g: (rgb.g as f32 * factor) as u8,
                            b: (rgb.b as f32 * factor) as u8,
                        };
                        cell.fg_color_256 = Some(palette::rgb_to_256(dimmed));
                    }
                    if let Some(c) = cell.bg_color_256 {
                        let rgb = palette::ANSI_256_TO_RGB[c as usize];
                        let dimmed = RgbColor {
                            r: (rgb.r as f32 * factor) as u8,
                            g: (rgb.g as f32 * factor) as u8,
                            b: (rgb.b as f32 * factor) as u8,
                        };
                        cell.bg_color_256 = Some(palette::rgb_to_256(dimmed));
                    }
                }
            }
        }
    }

    /// Apply pause effect based on the selected style.
    ///
    /// Routes to the appropriate effect implementation based on pause_style.
    pub fn apply_pause_effect(
        &mut self,
        pause_style: PauseStyle,
        frame_counter: u64,
        pulse_draw_mode: bool,
    ) {
        match pause_style {
            PauseStyle::Vcr => self.apply_vcr_pause_effect(frame_counter),
            PauseStyle::Vignette => self.apply_vignette_pause_effect(),
            PauseStyle::Minimal => {} // No effect - just freeze
            PauseStyle::Frosted => self.apply_frosted_pause_effect(),
            PauseStyle::Pixelate => self.apply_pixelate_pause_effect(),
            PauseStyle::Edges => self.apply_edges_pause_effect(),
            PauseStyle::Zoom => self.apply_zoom_pause_effect(frame_counter),
            PauseStyle::Pulse => self.apply_pulse_pause_effect(frame_counter, pulse_draw_mode),
            PauseStyle::Snow => self.apply_snow_pause_effect(frame_counter),
            PauseStyle::Starfield => self.apply_starfield_pause_effect(frame_counter),
            PauseStyle::Noise => self.apply_noise_pause_effect(frame_counter),
            PauseStyle::Matrix => self.apply_matrix_pause_effect(frame_counter),
        }
    }

    /// Desaturation + vignette effect (default pause style).
    ///
    /// Slightly desaturates colors and darkens edges for a cinematic look.
    fn apply_vignette_pause_effect(&mut self) {
        const DESATURATE: f32 = 0.25; // Desaturation factor
        const VIGNETTE_STRENGTH: f32 = 0.35; // Edge darkness
        const BRIGHTNESS: f32 = 0.85; // Overall brightness

        let center_x = (self.width as f32) / 2.0;
        let center_y = (self.height as f32) / 2.0;
        let max_radius = ((center_x * center_x + center_y * center_y).sqrt()).max(1.0);

        for y in 0..self.height {
            for x in 0..self.width {
                let idx = y * self.width + x;
                let cell = &mut self.cells[idx];

                // Calculate vignette factor based on distance from center
                let dx = x as f32 - center_x;
                let dy = y as f32 - center_y;
                let dist = (dx * dx + dy * dy).sqrt() / max_radius;
                let vignette = 1.0 - (dist * VIGNETTE_STRENGTH).clamp(0.0, VIGNETTE_STRENGTH);
                let factor = BRIGHTNESS * vignette;

                // Apply to RGB colors
                if let Some(c) = cell.fg_color_rgb {
                    // Desaturate: move toward grayscale
                    let gray = c.r as f32 * 0.299 + c.g as f32 * 0.587 + c.b as f32 * 0.114;
                    let r = (c.r as f32 * (1.0 - DESATURATE) + gray * DESATURATE) * factor;
                    let g = (c.g as f32 * (1.0 - DESATURATE) + gray * DESATURATE) * factor;
                    let b = (c.b as f32 * (1.0 - DESATURATE) + gray * DESATURATE) * factor;
                    cell.fg_color_rgb = Some(RgbColor {
                        r: r as u8,
                        g: g as u8,
                        b: b as u8,
                    });
                }
                if let Some(c) = cell.bg_color_rgb {
                    let gray = c.r as f32 * 0.299 + c.g as f32 * 0.587 + c.b as f32 * 0.114;
                    let r = (c.r as f32 * (1.0 - DESATURATE) + gray * DESATURATE) * factor;
                    let g = (c.g as f32 * (1.0 - DESATURATE) + gray * DESATURATE) * factor;
                    let b = (c.b as f32 * (1.0 - DESATURATE) + gray * DESATURATE) * factor;
                    cell.bg_color_rgb = Some(RgbColor {
                        r: r as u8,
                        g: g as u8,
                        b: b as u8,
                    });
                }
                if let Some(c) = cell.fg_color_256 {
                    let rgb = palette::ANSI_256_TO_RGB[c as usize];
                    let gray = rgb.r as f32 * 0.299 + rgb.g as f32 * 0.587 + rgb.b as f32 * 0.114;
                    let r = (rgb.r as f32 * (1.0 - DESATURATE) + gray * DESATURATE) * factor;
                    let g = (rgb.g as f32 * (1.0 - DESATURATE) + gray * DESATURATE) * factor;
                    let b = (rgb.b as f32 * (1.0 - DESATURATE) + gray * DESATURATE) * factor;
                    cell.fg_color_256 = Some(palette::rgb_to_256(RgbColor {
                        r: r as u8,
                        g: g as u8,
                        b: b as u8,
                    }));
                }
                if let Some(c) = cell.bg_color_256 {
                    let rgb = palette::ANSI_256_TO_RGB[c as usize];
                    let gray = rgb.r as f32 * 0.299 + rgb.g as f32 * 0.587 + rgb.b as f32 * 0.114;
                    let r = (rgb.r as f32 * (1.0 - DESATURATE) + gray * DESATURATE) * factor;
                    let g = (rgb.g as f32 * (1.0 - DESATURATE) + gray * DESATURATE) * factor;
                    let b = (rgb.b as f32 * (1.0 - DESATURATE) + gray * DESATURATE) * factor;
                    cell.bg_color_256 = Some(palette::rgb_to_256(RgbColor {
                        r: r as u8,
                        g: g as u8,
                        b: b as u8,
                    }));
                }
            }
        }
    }

    /// Frosted glass blur effect.
    ///
    /// Applies a subtle blur with blue tint for a modern frozen look.
    fn apply_frosted_pause_effect(&mut self) {
        // Create a copy of current cells for blur sampling
        let original_cells = self.cells.clone();
        const BLUR_FACTOR: f32 = 0.75; // Blend factor for blur
        const TINT_R: f32 = 0.95; // Slightly reduce red
        const TINT_G: f32 = 0.98; // Slightly reduce green
        const TINT_B: f32 = 1.05; // Boost blue slightly

        for y in 0..self.height {
            for x in 0..self.width {
                let idx = y * self.width + x;
                let cell = &mut self.cells[idx];

                // Simple 3x3 box blur by sampling neighbors
                let mut r_sum = 0.0f32;
                let mut g_sum = 0.0f32;
                let mut b_sum = 0.0f32;
                let mut count = 0u32;

                for dy in -1..=1i32 {
                    for dx in -1..=1i32 {
                        let nx = (x as i32 + dx).clamp(0, self.width as i32 - 1) as usize;
                        let ny = (y as i32 + dy).clamp(0, self.height as i32 - 1) as usize;
                        let nidx = ny * self.width + nx;

                        if let Some(c) = original_cells[nidx].fg_color_rgb {
                            r_sum += c.r as f32;
                            g_sum += c.g as f32;
                            b_sum += c.b as f32;
                            count += 1;
                        }
                    }
                }

                if count > 0 {
                    let r_avg = r_sum / count as f32;
                    let g_avg = g_sum / count as f32;
                    let b_avg = b_sum / count as f32;

                    if let Some(c) = cell.fg_color_rgb {
                        // Blend original with blur, then apply blue tint
                        let r = (c.r as f32 * (1.0 - BLUR_FACTOR) + r_avg * BLUR_FACTOR) * TINT_R;
                        let g = (c.g as f32 * (1.0 - BLUR_FACTOR) + g_avg * BLUR_FACTOR) * TINT_G;
                        let b = (c.b as f32 * (1.0 - BLUR_FACTOR) + b_avg * BLUR_FACTOR) * TINT_B;
                        cell.fg_color_rgb = Some(RgbColor {
                            r: r.min(255.0) as u8,
                            g: g.min(255.0) as u8,
                            b: b.min(255.0) as u8,
                        });
                    }
                    if let Some(c) = cell.bg_color_rgb {
                        let r = (c.r as f32 * (1.0 - BLUR_FACTOR) + r_avg * BLUR_FACTOR) * TINT_R;
                        let g = (c.g as f32 * (1.0 - BLUR_FACTOR) + g_avg * BLUR_FACTOR) * TINT_G;
                        let b = (c.b as f32 * (1.0 - BLUR_FACTOR) + b_avg * BLUR_FACTOR) * TINT_B;
                        cell.bg_color_rgb = Some(RgbColor {
                            r: r.min(255.0) as u8,
                            g: g.min(255.0) as u8,
                            b: b.min(255.0) as u8,
                        });
                    }
                }
            }
        }
    }

    /// Pixelate/mosaic effect.
    ///
    /// Reduces resolution appearance by grouping cells into blocks.
    fn apply_pixelate_pause_effect(&mut self) {
        const BLOCK_SIZE: usize = 2; // 2x2 pixel blocks
        const DIM: f32 = 0.90; // Slight dimming

        // Process in blocks
        for block_y in (0..self.height).step_by(BLOCK_SIZE) {
            for block_x in (0..self.width).step_by(BLOCK_SIZE) {
                // Calculate block bounds
                let end_y = (block_y + BLOCK_SIZE).min(self.height);
                let end_x = (block_x + BLOCK_SIZE).min(self.width);

                // Average colors in block
                let mut r_sum = 0.0f32;
                let mut g_sum = 0.0f32;
                let mut b_sum = 0.0f32;
                let mut count = 0u32;

                for y in block_y..end_y {
                    for x in block_x..end_x {
                        let idx = y * self.width + x;
                        if let Some(c) = self.cells[idx].fg_color_rgb {
                            r_sum += c.r as f32;
                            g_sum += c.g as f32;
                            b_sum += c.b as f32;
                            count += 1;
                        }
                    }
                }

                if count > 0 {
                    let avg_r = ((r_sum / count as f32) * DIM) as u8;
                    let avg_g = ((g_sum / count as f32) * DIM) as u8;
                    let avg_b = ((b_sum / count as f32) * DIM) as u8;

                    // Apply average to all cells in block
                    for y in block_y..end_y {
                        for x in block_x..end_x {
                            let idx = y * self.width + x;
                            if self.cells[idx].fg_color_rgb.is_some() {
                                self.cells[idx].fg_color_rgb = Some(RgbColor {
                                    r: avg_r,
                                    g: avg_g,
                                    b: avg_b,
                                });
                            }
                        }
                    }
                }
            }
        }
    }

    /// Edge detection outline effect.
    ///
    /// Shows only high-contrast edges like a sketch.
    fn apply_edges_pause_effect(&mut self) {
        const THRESHOLD: f32 = 30.0; // Brightness difference threshold
        const EDGE_COLOR: RgbColor = RgbColor {
            r: 220,
            g: 220,
            b: 220,
        };
        const BG_DIM: f32 = 0.3;

        // Create a copy for sampling
        let original_cells = self.cells.clone();

        for y in 1..self.height.saturating_sub(1) {
            for x in 1..self.width.saturating_sub(1) {
                let idx = y * self.width + x;

                let _current_brightness = if let Some(c) = original_cells[idx].fg_color_rgb {
                    (c.r as f32 + c.g as f32 + c.b as f32) / 3.0
                } else {
                    0.0
                };

                // Check gradient with neighbors
                let left_idx = y * self.width + (x - 1);
                let right_idx = y * self.width + (x + 1);
                let up_idx = (y - 1) * self.width + x;
                let down_idx = (y + 1) * self.width + x;

                let left_brightness = original_cells[left_idx]
                    .fg_color_rgb
                    .map_or(0.0, |c| (c.r + c.g + c.b) as f32 / 3.0);
                let right_brightness = original_cells[right_idx]
                    .fg_color_rgb
                    .map_or(0.0, |c| (c.r + c.g + c.b) as f32 / 3.0);
                let up_brightness = original_cells[up_idx]
                    .fg_color_rgb
                    .map_or(0.0, |c| (c.r + c.g + c.b) as f32 / 3.0);
                let down_brightness = original_cells[down_idx]
                    .fg_color_rgb
                    .map_or(0.0, |c| (c.r + c.g + c.b) as f32 / 3.0);

                let dx = (right_brightness - left_brightness).abs();
                let dy = (down_brightness - up_brightness).abs();
                let gradient = (dx * dx + dy * dy).sqrt();

                if gradient > THRESHOLD {
                    // This is an edge - make it bright
                    self.cells[idx].fg_color_rgb = Some(EDGE_COLOR);
                } else {
                    // Not an edge - dim significantly
                    if let Some(c) = self.cells[idx].fg_color_rgb {
                        self.cells[idx].fg_color_rgb = Some(RgbColor {
                            r: (c.r as f32 * BG_DIM) as u8,
                            g: (c.g as f32 * BG_DIM) as u8,
                            b: (c.b as f32 * BG_DIM) as u8,
                        });
                    }
                }
            }
        }
    }

    /// Radial zoom blur effect.
    ///
    /// Creates motion blur radiating from center.
    fn apply_zoom_pause_effect(&mut self, _frame_counter: u64) {
        const BLUR_STRENGTH: f32 = 0.4;
        const RADIAL_SAMPLES: i32 = 4;

        let center_x = (self.width as f32) / 2.0;
        let center_y = (self.height as f32) / 2.0;

        // Create a copy for sampling
        let original_cells = self.cells.clone();

        for y in 0..self.height {
            for x in 0..self.width {
                let idx = y * self.width + x;

                // Calculate direction from center
                let dx = x as f32 - center_x;
                let dy = y as f32 - center_y;
                let dist = (dx * dx + dy * dy).sqrt();

                if dist < 1.0 {
                    continue; // Skip center
                }

                // Normalize direction
                let dir_x = dx / dist;
                let dir_y = dy / dist;

                // Sample along radial direction
                let mut r_sum = 0.0f32;
                let mut g_sum = 0.0f32;
                let mut b_sum = 0.0f32;
                let mut count = 0u32;

                for i in 0..RADIAL_SAMPLES {
                    let sample_dist = i as f32 * 0.5;
                    let sx = (x as f32 + dir_x * sample_dist).clamp(0.0, (self.width - 1) as f32)
                        as usize;
                    let sy = (y as f32 + dir_y * sample_dist).clamp(0.0, (self.height - 1) as f32)
                        as usize;
                    let sidx = sy * self.width + sx;

                    if let Some(c) = original_cells[sidx].fg_color_rgb {
                        r_sum += c.r as f32;
                        g_sum += c.g as f32;
                        b_sum += c.b as f32;
                        count += 1;
                    }
                }

                if count > 0 && self.cells[idx].fg_color_rgb.is_some() {
                    let r_avg = r_sum / count as f32;
                    let g_avg = g_sum / count as f32;
                    let b_avg = b_sum / count as f32;

                    // Blend original with radial blur
                    if let Some(c) = self.cells[idx].fg_color_rgb {
                        self.cells[idx].fg_color_rgb = Some(RgbColor {
                            r: (c.r as f32 * (1.0 - BLUR_STRENGTH) + r_avg * BLUR_STRENGTH) as u8,
                            g: (c.g as f32 * (1.0 - BLUR_STRENGTH) + g_avg * BLUR_STRENGTH) as u8,
                            b: (c.b as f32 * (1.0 - BLUR_STRENGTH) + b_avg * BLUR_STRENGTH) as u8,
                        });
                    }
                }
            }
        }
    }

    /// Pulse/wave animation effect.
    ///
    /// Creates expanding circular waves that animate outward from center.
    fn apply_pulse_pause_effect(&mut self, frame_counter: u64, draw_on_empty: bool) {
        const WAVE_SPEED: f32 = 0.15;
        const WAVE_COUNT: usize = 3;
        const WAVE_WIDTH: f32 = 3.0;
        const DIM: f32 = 0.75;

        let center_x = self.width as f32 / 2.0;
        let center_y = self.height as f32 / 2.0;
        let max_dist = (center_x * center_x + center_y * center_y).sqrt();

        // Draw wave rings on empty cells (only in debug draw mode)
        for y in 0..self.height {
            for x in 0..self.width {
                let idx = y * self.width + x;
                let cell = &self.cells[idx];

                let is_empty = cell.char == ' ' && cell.fg_color_rgb.is_none();

                let dx = x as f32 - center_x;
                let dy = y as f32 - center_y;
                let dist = (dx * dx + dy * dy).sqrt();

                // Calculate wave intensity
                let mut wave_intensity = 0.0f32;
                for i in 0..WAVE_COUNT {
                    let wave_offset = frame_counter as f32 * WAVE_SPEED
                        + i as f32 * (max_dist / WAVE_COUNT as f32);
                    let wave_pos = wave_offset % max_dist;
                    let dist_from_wave = (dist - wave_pos).abs();
                    if dist_from_wave < WAVE_WIDTH {
                        let intensity = 1.0 - (dist_from_wave / WAVE_WIDTH);
                        wave_intensity = wave_intensity.max(intensity);
                    }
                }

                if draw_on_empty && is_empty && wave_intensity > 0.1 {
                    // Draw wave ring on empty cell (debug mode only)
                    let brightness = (100.0 + wave_intensity * 100.0) as u8;
                    self.cells[idx].char = '·';
                    self.cells[idx].fg_color_rgb = Some(RgbColor {
                        r: brightness,
                        g: brightness,
                        b: (brightness as f32 * 1.2).min(255.0) as u8,
                    });
                } else if let Some(c) = cell.fg_color_rgb {
                    // Pulse existing cells - always apply brightness modulation
                    let pulse = 1.0 + wave_intensity * 0.3;
                    self.cells[idx].fg_color_rgb = Some(RgbColor {
                        r: ((c.r as f32 * pulse * DIM).min(255.0)) as u8,
                        g: ((c.g as f32 * pulse * DIM).min(255.0)) as u8,
                        b: ((c.b as f32 * pulse * DIM).min(255.0)) as u8,
                    });
                }
            }
        }
    }

    /// Falling snowflakes effect.
    ///
    /// Draws animated snowflakes on empty cells that slowly fall.
    fn apply_snow_pause_effect(&mut self, frame_counter: u64) {
        const SNOW_CHARS: [char; 4] = ['❄', '·', '∙', '•'];
        const SNOW_COLOR: RgbColor = RgbColor {
            r: 220,
            g: 230,
            b: 255,
        };
        const DIM: f32 = 0.80;
        const FALL_SPEED: f32 = 0.3; // Rows per frame

        // First, dim existing cells
        for y in 0..self.height {
            for x in 0..self.width {
                let idx = y * self.width + x;
                if let Some(c) = self.cells[idx].fg_color_rgb {
                    self.cells[idx].fg_color_rgb = Some(RgbColor {
                        r: (c.r as f32 * DIM) as u8,
                        g: (c.g as f32 * DIM) as u8,
                        b: (c.b as f32 * DIM) as u8,
                    });
                }
            }
        }

        // Draw snowflakes on empty cells
        // Use deterministic pseudo-random based on position and frame
        for y in 0..self.height {
            for x in 0..self.width {
                let idx = y * self.width + x;
                let cell = &self.cells[idx];

                let is_empty = cell.char == ' ' && cell.fg_color_rgb.is_none();

                if is_empty {
                    // Deterministic "random" snow based on column and time
                    let col_hash = (x as u64).wrapping_mul(2654435761u64);
                    let snow_chance = ((col_hash ^ (frame_counter / 10)) % 100) as u8;

                    if snow_chance < 5 {
                        // This column has snow at this time
                        let fall_offset =
                            ((frame_counter as f32 * FALL_SPEED) as usize + x) % self.height;
                        let snow_y = (self.height - 1) - ((y + fall_offset) % self.height);

                        if y == snow_y {
                            let char_idx = (col_hash % SNOW_CHARS.len() as u64) as usize;
                            let brightness_var = ((col_hash % 40) as u8) + 180;
                            self.cells[idx].char = SNOW_CHARS[char_idx];
                            self.cells[idx].fg_color_rgb = Some(RgbColor {
                                r: (SNOW_COLOR.r as f32 * brightness_var as f32 / 255.0) as u8,
                                g: (SNOW_COLOR.g as f32 * brightness_var as f32 / 255.0) as u8,
                                b: (SNOW_COLOR.b as f32 * brightness_var as f32 / 255.0) as u8,
                            });
                        }
                    }
                }
            }
        }
    }

    /// Twinkling starfield effect.
    ///
    /// Draws twinkling stars on empty cells.
    fn apply_starfield_pause_effect(&mut self, frame_counter: u64) {
        const STAR_CHARS: [char; 3] = ['·', '•', '✦'];
        const DIM: f32 = 0.75;

        // Star colors (white, blue-white, yellow-white)
        const STAR_COLORS: [RgbColor; 3] = [
            RgbColor {
                r: 255,
                g: 255,
                b: 255,
            },
            RgbColor {
                r: 200,
                g: 220,
                b: 255,
            },
            RgbColor {
                r: 255,
                g: 250,
                b: 220,
            },
        ];

        // First, dim existing cells
        for y in 0..self.height {
            for x in 0..self.width {
                let idx = y * self.width + x;
                if let Some(c) = self.cells[idx].fg_color_rgb {
                    self.cells[idx].fg_color_rgb = Some(RgbColor {
                        r: (c.r as f32 * DIM) as u8,
                        g: (c.g as f32 * DIM) as u8,
                        b: (c.b as f32 * DIM) as u8,
                    });
                }
            }
        }

        // Draw stars on empty cells
        for y in 0..self.height {
            for x in 0..self.width {
                let idx = y * self.width + x;
                let cell = &self.cells[idx];

                let is_empty = cell.char == ' ' && cell.fg_color_rgb.is_none();

                if is_empty {
                    // Deterministic star positions
                    let pos_hash =
                        ((x * 374761) ^ (y * 668265) ^ (frame_counter / 30) as usize) as u64;
                    let star_density = 80; // 1 in 80 cells has a star

                    if (pos_hash % star_density as u64) == 0 {
                        let twinkle = ((pos_hash / star_density as u64) % 10) as u8;
                        let brightness = if twinkle < 5 {
                            150 + twinkle * 20
                        } else {
                            250 - (twinkle - 5) * 20
                        };

                        let char_idx = (pos_hash % STAR_CHARS.len() as u64) as usize;
                        let color_idx = (pos_hash % STAR_COLORS.len() as u64) as usize;
                        let color = STAR_COLORS[color_idx];

                        self.cells[idx].char = STAR_CHARS[char_idx];
                        self.cells[idx].fg_color_rgb = Some(RgbColor {
                            r: (color.r as f32 * brightness as f32 / 255.0) as u8,
                            g: (color.g as f32 * brightness as f32 / 255.0) as u8,
                            b: (color.b as f32 * brightness as f32 / 255.0) as u8,
                        });
                    }
                }
            }
        }
    }

    /// TV static noise effect.
    ///
    /// Draws random noise on empty cells like TV static.
    fn apply_noise_pause_effect(&mut self, frame_counter: u64) {
        const NOISE_CHARS: [char; 5] = ['·', ':', '·', '∙', ' '];
        const DIM: f32 = 0.70;

        // First, dim existing cells
        for y in 0..self.height {
            for x in 0..self.width {
                let idx = y * self.width + x;
                if let Some(c) = self.cells[idx].fg_color_rgb {
                    self.cells[idx].fg_color_rgb = Some(RgbColor {
                        r: (c.r as f32 * DIM) as u8,
                        g: (c.g as f32 * DIM) as u8,
                        b: (c.b as f32 * DIM) as u8,
                    });
                }
            }
        }

        // Draw noise on empty cells
        for y in 0..self.height {
            for x in 0..self.width {
                let idx = y * self.width + x;
                let cell = &self.cells[idx];

                let is_empty = cell.char == ' ' && cell.fg_color_rgb.is_none();

                if is_empty {
                    // Deterministic noise based on position and frame
                    let noise_hash =
                        ((x * 73856093) ^ (y * 19349663) ^ (frame_counter as usize)) as u64;
                    let noise_threshold = 60; // 60% of cells get noise

                    if (noise_hash % 100) < noise_threshold {
                        let char_idx = (noise_hash % NOISE_CHARS.len() as u64) as usize;
                        let brightness = 100 + ((noise_hash / 100) % 100) as u8;

                        self.cells[idx].char = NOISE_CHARS[char_idx];
                        self.cells[idx].fg_color_rgb = Some(RgbColor {
                            r: brightness,
                            g: brightness,
                            b: brightness,
                        });
                    }
                }
            }
        }
    }

    /// Matrix-style falling characters effect.
    ///
    /// Draws falling characters on empty cells like the Matrix digital rain.
    fn apply_matrix_pause_effect(&mut self, frame_counter: u64) {
        const MATRIX_CHARS: [char; 10] = ['0', '1', '2', '3', '4', '5', '6', '7', '8', '9'];
        const MATRIX_COLOR: RgbColor = RgbColor {
            r: 0,
            g: 200,
            b: 50,
        };
        const HEAD_COLOR: RgbColor = RgbColor {
            r: 200,
            g: 255,
            b: 200,
        };
        const DIM: f32 = 0.65;
        const FALL_SPEED: f32 = 0.4;

        // First, dim existing cells with green tint
        for y in 0..self.height {
            for x in 0..self.width {
                let idx = y * self.width + x;
                if let Some(c) = self.cells[idx].fg_color_rgb {
                    self.cells[idx].fg_color_rgb = Some(RgbColor {
                        r: (c.r as f32 * DIM * 0.5) as u8,
                        g: (c.g as f32 * DIM) as u8,
                        b: (c.b as f32 * DIM * 0.5) as u8,
                    });
                }
            }
        }

        // Draw matrix rain on empty cells
        for x in 0..self.width {
            // Each column has its own stream
            let col_hash = (x as u64).wrapping_mul(2654435761u64);
            let stream_length = 5 + (col_hash % 15) as usize;
            let stream_speed = ((col_hash % 3) as f32 + 1.0) * FALL_SPEED;
            let stream_offset = (frame_counter as f32 * stream_speed) as usize;

            // Determine if this column has an active stream
            if (col_hash % 100) < 30 {
                // 30% of columns have streams
                let head_y = (stream_offset % (self.height + stream_length + 10)) as i32
                    - stream_length as i32;

                for y in 0..self.height {
                    let idx = y * self.width + x;
                    let cell = &self.cells[idx];

                    let is_empty = cell.char == ' ' && cell.fg_color_rgb.is_none();

                    if is_empty {
                        let dist_from_head = (y as i32 - head_y).abs();

                        if dist_from_head >= 0 && dist_from_head < stream_length as i32 {
                            let char_idx =
                                ((col_hash + y as u64) % MATRIX_CHARS.len() as u64) as usize;
                            let brightness = if dist_from_head == 0 {
                                // Head is brighter
                                255
                            } else {
                                // Tail fades
                                150 - (dist_from_head * 10).min(100)
                            };

                            let color = if dist_from_head == 0 {
                                HEAD_COLOR
                            } else {
                                MATRIX_COLOR
                            };

                            self.cells[idx].char = MATRIX_CHARS[char_idx];
                            self.cells[idx].fg_color_rgb = Some(RgbColor {
                                r: (color.r as f32 * brightness as f32 / 255.0) as u8,
                                g: (color.g as f32 * brightness as f32 / 255.0) as u8,
                                b: (color.b as f32 * brightness as f32 / 255.0) as u8,
                            });
                        }
                    }
                }
            }
        }
    }

    /// Draws title block and footer rows as overlays on the outer sim rows.
    ///
    /// Title rows are written at `(sim_x, sim_y)` and `(sim_x, sim_y+1)`.
    /// Footer rows at `(sim_x, sim_y+sim_h-2)` and `(sim_x, sim_y+sim_h-1)`.
    /// No-op if `sim_h < 4`.
    #[allow(clippy::too_many_arguments)]
    pub fn draw_expanded_chrome(
        &mut self,
        sim_x: usize,
        sim_y: usize,
        sim_w: usize,
        sim_h: usize,
        title_rows: &[String; 2],
        footer_status: &str,
        footer_status_colors: &[(usize, crate::render::palette::RgbColor)],
        footer_keybinds: &str,
        accent: crate::render::palette::RgbColor,
        text: crate::render::palette::RgbColor,
        dim: crate::render::palette::RgbColor,
    ) {
        if sim_h < 4 {
            return;
        }
        let bw = self.width;
        let bh = self.height;

        // Helper closure: write a plain string at (x_start, row_y) with given fg color.
        let write_row = |cells: &mut Vec<Cell>,
                         row_y: usize,
                         x_start: usize,
                         s: &str,
                         fg: crate::render::palette::RgbColor,
                         max_w: usize,
                         total_w: usize| {
            for (i, ch) in s.chars().enumerate() {
                if i >= max_w {
                    break;
                }
                let x = x_start + i;
                if x < total_w && row_y < bh {
                    let idx = row_y * total_w + x;
                    if idx < cells.len() {
                        cells[idx].char = ch;
                        cells[idx].fg_color_rgb = Some(fg);
                    }
                }
            }
        };

        write_row(
            &mut self.cells,
            sim_y,
            sim_x,
            &title_rows[0],
            accent,
            sim_w,
            bw,
        );
        write_row(
            &mut self.cells,
            sim_y + 1,
            sim_x,
            &title_rows[1],
            text,
            sim_w,
            bw,
        );
        write_row(
            &mut self.cells,
            sim_y + sim_h - 2,
            sim_x,
            footer_status,
            text,
            sim_w,
            bw,
        );

        // Apply per-column color overrides for footer status
        for (col, color) in footer_status_colors {
            let x = sim_x + col;
            let y = sim_y + sim_h - 2;
            if x < bw && y < bh {
                let idx = y * bw + x;
                if idx < self.cells.len() {
                    self.cells[idx].fg_color_rgb = Some(*color);
                }
            }
        }

        write_row(
            &mut self.cells,
            sim_y + sim_h - 1,
            sim_x,
            footer_keybinds,
            dim,
            sim_w,
            bw,
        );
    }

    /// Applies an alpha multiplier to fg colors in the chrome rows (top 2 and bottom 2 rows of the sim area).
    ///
    /// Called each frame when `ChromeState::FadingOut` is active to blend the chrome toward black.
    ///
    /// # Parameters
    /// - `sim_x`, `sim_y`: Top-left corner of the sim area.
    /// - `sim_w`, `sim_h`: Width and height of the sim area.
    /// - `alpha`: Multiplier in `[0.0, 1.0]`; 1.0 = fully visible, 0.0 = invisible.
    pub fn fade_chrome_rows(
        &mut self,
        sim_x: usize,
        sim_y: usize,
        sim_w: usize,
        sim_h: usize,
        alpha: f32,
    ) {
        if sim_h < 4 {
            return;
        }
        let fade_rows = [sim_y, sim_y + 1, sim_y + sim_h - 2, sim_y + sim_h - 1];
        for row in fade_rows {
            for x in sim_x..sim_x + sim_w {
                if x < self.width && row < self.height {
                    let idx = row * self.width + x;
                    if let Some(ref mut fg) = self.cells[idx].fg_color_rgb {
                        fg.r = (fg.r as f32 * alpha) as u8;
                        fg.g = (fg.g as f32 * alpha) as u8;
                        fg.b = (fg.b as f32 * alpha) as u8;
                    }
                }
            }
        }
    }

    /// Get the raw RGB values for all pixels in the frame buffer.
    ///
    /// Useful for exporting the frame to an image file.
    pub fn get_rgb_pixels(&self) -> Vec<u8> {
        let mut pixels = Vec::with_capacity(self.width * self.height * 3);
        for cell in &self.cells {
            let rgb = if let Some(c) = cell.fg_color_rgb {
                c
            } else if let Some(c) = cell.bg_color_rgb {
                c
            } else if let Some(c) = cell.fg_color_256 {
                palette::ANSI_256_TO_RGB[c as usize]
            } else {
                RgbColor { r: 0, g: 0, b: 0 }
            };
            pixels.push(rgb.r);
            pixels.push(rgb.g);
            pixels.push(rgb.b);
        }
        pixels
    }
}

/// Render a single frame to stdout.
///
/// This is a convenience wrapper around creating a `FrameBuffer` and writing it.
#[allow(clippy::too_many_arguments)]
pub fn render_frame(
    downsampled: &[DownsampleCell],
    width: usize,
    height: usize,
    max_trail_value: f32,
    palette: Palette,
    charset: Charset,
    reverse_palette: bool,
    invert_palette: bool,
    color_mode: ColorMode,
    hue_shift: f32,
    dither_mode: DitherMode,
    intensity_mapping: Option<&IntensityMapping>,
    species_colors_enabled: bool,
    species_rgb_colors: Option<Vec<RgbColor>>,
    error_diffusion: &mut Option<ErrorDiffusion>,
    background_color: Option<RgbColor>,
) -> io::Result<()> {
    if let Some(ref mut ed) = error_diffusion {
        ed.reset();
    }
    let buffer = FrameBuffer::from_downsampled(
        downsampled,
        width,
        height,
        max_trail_value,
        palette,
        charset,
        reverse_palette,
        invert_palette,
        color_mode,
        hue_shift,
        dither_mode,
        error_diffusion,
        intensity_mapping,
        species_colors_enabled,
        species_rgb_colors,
        background_color,
        1.5,
        None,
        false,
        false,
        60.0,
        1.0,
        0.5,
        false,
        0.3,
        TrailAgeMode::Bidirectional,
        false,
        0.0,
        palette::TemporalMode::Hue,
        palette::PaletteCycle::default(),
        charset::GlyphConfig::default(),
        None,
        crate::render::antialiasing::AaStrength::Off,
    );

    execute!(std::io::stdout(), &buffer)
}

impl Command for &FrameBuffer {
    fn write_ansi(&self, f: &mut impl fmt::Write) -> fmt::Result {
        let frame_str = self.build_frame_string(false, self.color_mode);
        write!(f, "{}", frame_str)
    }

    #[cfg(windows)]
    fn execute_winapi(&self) -> io::Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_frame_buffer_creation() {
        let buffer = FrameBuffer::new(80, 24, ColorMode::Bits256, None);
        assert_eq!(buffer.width(), 80);
        assert_eq!(buffer.height(), 24);
    }

    #[test]
    fn test_frame_buffer_set_cell() {
        let mut buffer = FrameBuffer::new(10, 10, ColorMode::Bits256, None);
        let cell = Cell {
            char: 'A',
            fg_color_256: Some(10),
            bg_color_256: Some(20),
            fg_color_rgb: None,
            bg_color_rgb: None,
        };
        buffer.set_cell(5, 3, cell);

        assert_eq!(buffer.cells[3 * 10 + 5].char, 'A');
        assert_eq!(buffer.cells[3 * 10 + 5].fg_color_256, Some(10));
        assert_eq!(buffer.cells[3 * 10 + 5].bg_color_256, Some(20));
    }

    #[test]
    fn test_frame_buffer_set_cell_out_of_bounds() {
        let mut buffer = FrameBuffer::new(10, 10, ColorMode::Bits256, None);
        let cell = Cell {
            char: 'A',
            fg_color_256: Some(10),
            bg_color_256: None,
            fg_color_rgb: None,
            bg_color_rgb: None,
        };

        buffer.set_cell(15, 5, cell);
        buffer.set_cell(5, 15, cell);

        assert_eq!(buffer.cells[0].char, ' ');
    }

    #[test]
    fn test_max_brightness() {
        let cells = vec![
            DownsampleCell {
                top: 0.0,
                bottom: 0.0,
                ..Default::default()
            },
            DownsampleCell {
                top: 5.0,
                bottom: 2.0,
                ..Default::default()
            },
            DownsampleCell {
                top: 3.0,
                bottom: 7.0,
                ..Default::default()
            },
        ];
        let max = FrameBuffer::max_brightness(&cells);
        assert_eq!(max, 7.0);
    }

    #[test]
    fn test_max_brightness_empty() {
        let cells: Vec<DownsampleCell> = vec![];
        let max = FrameBuffer::max_brightness(&cells);
        assert_eq!(max, 0.0);
    }

    #[test]
    fn test_create_cell_empty() {
        use crate::render::dither::DitherMode;
        let mut buffer = FrameBuffer::new(10, 10, ColorMode::Bits256, None);
        let cell = buffer.create_cell(
            0,
            0,
            0.0,
            0.0,
            &[],
            1.0,
            &Palette::Organic,
            Charset::HalfBlock,
            false,
            false,
            ColorMode::Bits256,
            0.0,
            DitherMode::None,
            &mut None,
            None,
            false,
            None,
            0.0,
            0.0,
            palette::TemporalMode::Hue,
            palette::PaletteCycle::default(),
            charset::GlyphConfig::default(),
            None,
        );
        assert_eq!(cell.char, ' ');
        assert!(cell.fg_color_256.is_none());
        assert!(cell.bg_color_256.is_none());
        assert!(cell.fg_color_rgb.is_none());
        assert!(cell.bg_color_rgb.is_none());
    }

    #[test]
    fn test_create_cell_full() {
        use crate::render::dither::DitherMode;
        let mut buffer = FrameBuffer::new(10, 10, ColorMode::Bits256, None);
        let cell = buffer.create_cell(
            0,
            0,
            1.0,
            1.0,
            &[],
            1.0,
            &Palette::Organic,
            Charset::HalfBlock,
            false,
            false,
            ColorMode::Bits256,
            0.0,
            DitherMode::None,
            &mut None,
            None,
            false,
            None,
            0.0,
            0.0,
            palette::TemporalMode::Hue,
            palette::PaletteCycle::default(),
            charset::GlyphConfig::default(),
            None,
        );
        assert_eq!(cell.char, '\u{2588}');
        assert!(cell.fg_color_256.is_some());
        assert!(cell.bg_color_256.is_none());
        assert!(cell.fg_color_rgb.is_none());
        assert!(cell.bg_color_rgb.is_none());
    }

    #[test]
    fn test_create_cell_full_truecolor() {
        use crate::render::dither::DitherMode;
        let mut buffer = FrameBuffer::new(10, 10, ColorMode::TrueColor, None);
        let cell = buffer.create_cell(
            0,
            0,
            1.0,
            1.0,
            &[],
            1.0,
            &Palette::Organic,
            Charset::HalfBlock,
            false,
            false,
            ColorMode::TrueColor,
            0.0,
            DitherMode::None,
            &mut None,
            None,
            false,
            None,
            0.0,
            0.0,
            palette::TemporalMode::Hue,
            palette::PaletteCycle::default(),
            charset::GlyphConfig::default(),
            None,
        );
        assert_eq!(cell.char, '\u{2588}');
        assert!(cell.fg_color_256.is_none());
        assert!(cell.bg_color_256.is_none());
        assert!(cell.fg_color_rgb.is_some());
        assert!(cell.bg_color_rgb.is_none());
    }

    #[test]
    fn test_create_cell_halfblock_top_only_uses_half_height() {
        use crate::render::dither::DitherMode;
        let mut buffer = FrameBuffer::new(10, 10, ColorMode::Bits256, None);
        let cell = buffer.create_cell(
            0,
            0,
            1.0,
            0.0,
            &[],
            1.0,
            &Palette::Organic,
            Charset::HalfBlock,
            false,
            false,
            ColorMode::Bits256,
            0.0,
            DitherMode::None,
            &mut None,
            None,
            false,
            None,
            0.0,
            0.0,
            palette::TemporalMode::Hue,
            palette::PaletteCycle::default(),
            charset::GlyphConfig::default(),
            None,
        );
        assert_eq!(cell.char, '▀');
        assert!(cell.fg_color_256.is_some());
        assert!(cell.bg_color_256.is_none());
    }

    #[test]
    fn test_create_cell_halfblock_bottom_only_uses_half_height() {
        use crate::render::dither::DitherMode;
        let mut buffer = FrameBuffer::new(10, 10, ColorMode::Bits256, None);
        let cell = buffer.create_cell(
            0,
            0,
            0.0,
            1.0,
            &[],
            1.0,
            &Palette::Organic,
            Charset::HalfBlock,
            false,
            false,
            ColorMode::Bits256,
            0.0,
            DitherMode::None,
            &mut None,
            None,
            false,
            None,
            0.0,
            0.0,
            palette::TemporalMode::Hue,
            palette::PaletteCycle::default(),
            charset::GlyphConfig::default(),
            None,
        );
        assert_eq!(cell.char, '▄');
        assert!(cell.fg_color_256.is_some());
        assert!(cell.bg_color_256.is_none());
    }

    #[test]
    fn test_create_cell_halfblock_top_half_brightness() {
        use crate::render::dither::DitherMode;
        let mut buffer = FrameBuffer::new(10, 10, ColorMode::Bits256, None);
        let cell = buffer.create_cell(
            0,
            0,
            0.5,
            0.0,
            &[],
            1.0,
            &Palette::Organic,
            Charset::HalfBlock,
            false,
            false,
            ColorMode::Bits256,
            0.0,
            DitherMode::None,
            &mut None,
            None,
            false,
            None,
            0.0,
            0.0,
            palette::TemporalMode::Hue,
            palette::PaletteCycle::default(),
            charset::GlyphConfig::default(),
            None,
        );
        assert_eq!(cell.char, '▀');
        assert!(cell.fg_color_256.is_some());
        assert!(cell.bg_color_256.is_none());
    }

    #[test]
    fn test_create_cell_bottom_only() {
        use crate::render::dither::DitherMode;
        let mut buffer = FrameBuffer::new(10, 10, ColorMode::Bits256, None);
        let cell = buffer.create_cell(
            0,
            0,
            0.0,
            1.0,
            &[],
            1.0,
            &Palette::Organic,
            Charset::HalfBlock,
            false,
            false,
            ColorMode::Bits256,
            0.0,
            DitherMode::None,
            &mut None,
            None,
            false,
            None,
            0.0,
            0.0,
            palette::TemporalMode::Hue,
            palette::PaletteCycle::default(),
            charset::GlyphConfig::default(),
            None,
        );
        assert_eq!(cell.char, '▄');
        assert!(cell.fg_color_256.is_some());
        assert!(cell.bg_color_256.is_none());
    }

    #[test]
    fn test_create_cell_braille_top_only() {
        use crate::render::dither::DitherMode;
        let mut buffer = FrameBuffer::new(10, 10, ColorMode::Bits256, None);
        let cell = buffer.create_cell(
            0,
            0,
            1.0,
            0.0,
            &[],
            1.0,
            &Palette::Organic,
            Charset::Braille,
            false,
            false,
            ColorMode::Bits256,
            0.0,
            DitherMode::None,
            &mut None,
            None,
            false,
            None,
            0.0,
            0.0,
            palette::TemporalMode::Hue,
            palette::PaletteCycle::default(),
            charset::GlyphConfig::default(),
            None,
        );
        assert_eq!(cell.char, '\u{2807}');
        assert!(cell.fg_color_256.is_some());
        assert!(cell.bg_color_256.is_none());
    }

    #[test]
    fn test_create_cell_braille_bottom_only() {
        use crate::render::dither::DitherMode;
        let mut buffer = FrameBuffer::new(10, 10, ColorMode::Bits256, None);
        let cell = buffer.create_cell(
            0,
            0,
            0.0,
            1.0,
            &[],
            1.0,
            &Palette::Organic,
            Charset::Braille,
            false,
            false,
            ColorMode::Bits256,
            0.0,
            DitherMode::None,
            &mut None,
            None,
            false,
            None,
            0.0,
            0.0,
            palette::TemporalMode::Hue,
            palette::PaletteCycle::default(),
            charset::GlyphConfig::default(),
            None,
        );
        assert_eq!(cell.char, '\u{2838}');
        assert!(cell.fg_color_256.is_some());
        assert!(cell.bg_color_256.is_none());
    }

    #[test]
    fn test_create_cell_braille_top_half_brightness() {
        use crate::render::dither::DitherMode;
        let mut buffer = FrameBuffer::new(10, 10, ColorMode::Bits256, None);
        let cell = buffer.create_cell(
            0,
            0,
            0.5,
            0.0,
            &[],
            1.0,
            &Palette::Organic,
            Charset::Braille,
            false,
            false,
            ColorMode::Bits256,
            0.0,
            DitherMode::None,
            &mut None,
            None,
            false,
            None,
            0.0,
            0.0,
            palette::TemporalMode::Hue,
            palette::PaletteCycle::default(),
            charset::GlyphConfig::default(),
            None,
        );
        assert!(cell.char >= '\u{2800}' && cell.char <= '\u{28FF}');
        assert!(cell.fg_color_256.is_some());
        assert!(cell.bg_color_256.is_none());
    }

    #[test]
    fn test_create_cell_braille_bottom_half_brightness() {
        use crate::render::dither::DitherMode;
        let mut buffer = FrameBuffer::new(10, 10, ColorMode::Bits256, None);
        let cell = buffer.create_cell(
            0,
            0,
            0.0,
            0.5,
            &[],
            1.0,
            &Palette::Organic,
            Charset::Braille,
            false,
            false,
            ColorMode::Bits256,
            0.0,
            DitherMode::None,
            &mut None,
            None,
            false,
            None,
            0.0,
            0.0,
            palette::TemporalMode::Hue,
            palette::PaletteCycle::default(),
            charset::GlyphConfig::default(),
            None,
        );
        assert!(cell.char >= '\u{2800}' && cell.char <= '\u{28FF}');
        assert!(cell.fg_color_256.is_some());
        assert!(cell.bg_color_256.is_none());
    }

    #[test]
    fn test_create_cell_ascii_top_only() {
        use crate::render::dither::DitherMode;
        let mut buffer = FrameBuffer::new(10, 10, ColorMode::Bits256, None);
        let cell = buffer.create_cell(
            0,
            0,
            1.0,
            0.0,
            &[],
            1.0,
            &Palette::Organic,
            Charset::Ascii,
            false,
            false,
            ColorMode::Bits256,
            0.0,
            DitherMode::None,
            &mut None,
            None,
            false,
            None,
            0.0,
            0.0,
            palette::TemporalMode::Hue,
            palette::PaletteCycle::default(),
            charset::GlyphConfig::default(),
            None,
        );
        assert_eq!(cell.char, '^');
        assert!(cell.fg_color_256.is_some());
        assert!(cell.bg_color_256.is_none());
    }

    #[test]
    fn test_create_cell_ascii_bottom_only() {
        use crate::render::dither::DitherMode;
        let mut buffer = FrameBuffer::new(10, 10, ColorMode::Bits256, None);
        let cell = buffer.create_cell(
            0,
            0,
            0.0,
            1.0,
            &[],
            1.0,
            &Palette::Organic,
            Charset::Ascii,
            false,
            false,
            ColorMode::Bits256,
            0.0,
            DitherMode::None,
            &mut None,
            None,
            false,
            None,
            0.0,
            0.0,
            palette::TemporalMode::Hue,
            palette::PaletteCycle::default(),
            charset::GlyphConfig::default(),
            None,
        );
        assert_eq!(cell.char, 'v');
        assert!(cell.fg_color_256.is_some());
        assert!(cell.bg_color_256.is_none());
    }

    #[test]
    fn test_create_cell_ascii_top_half_brightness() {
        use crate::render::dither::DitherMode;
        let mut buffer = FrameBuffer::new(10, 10, ColorMode::Bits256, None);
        let cell = buffer.create_cell(
            0,
            0,
            0.5,
            0.0,
            &[],
            1.0,
            &Palette::Organic,
            Charset::Ascii,
            false,
            false,
            ColorMode::Bits256,
            0.0,
            DitherMode::None,
            &mut None,
            None,
            false,
            None,
            0.0,
            0.0,
            palette::TemporalMode::Hue,
            palette::PaletteCycle::default(),
            charset::GlyphConfig::default(),
            None,
        );
        assert_eq!(cell.char, '=');
        assert!(cell.fg_color_256.is_some());
        assert!(cell.bg_color_256.is_none());
    }

    #[test]
    fn test_create_cell_ascii_bottom_half_brightness() {
        use crate::render::dither::DitherMode;
        let mut buffer = FrameBuffer::new(10, 10, ColorMode::Bits256, None);
        let cell = buffer.create_cell(
            0,
            0,
            0.0,
            0.5,
            &[],
            1.0,
            &Palette::Organic,
            Charset::Ascii,
            false,
            false,
            ColorMode::Bits256,
            0.0,
            DitherMode::None,
            &mut None,
            None,
            false,
            None,
            0.0,
            0.0,
            palette::TemporalMode::Hue,
            palette::PaletteCycle::default(),
            charset::GlyphConfig::default(),
            None,
        );
        assert_eq!(cell.char, '=');
        assert!(cell.fg_color_256.is_some());
        assert!(cell.bg_color_256.is_none());
    }

    #[test]
    fn test_ansi_color_code_fg() {
        let code = FrameBuffer::ansi_color_code(42, true);
        assert_eq!(code, "\x1b[38;5;42m");
    }

    #[test]
    fn test_ansi_color_code_bg() {
        let code = FrameBuffer::ansi_color_code(128, false);
        assert_eq!(code, "\x1b[48;5;128m");
    }

    #[test]
    fn test_truecolor_code_bg_extended() {
        let _color = RgbColor {
            r: 10,
            g: 20,
            b: 30,
        };
        assert_eq!(
            FrameBuffer::truecolor_code(10, 20, 30, false),
            "\x1b[48;2;10;20;30m"
        );
    }

    #[test]
    fn test_truecolor_code_bg_final() {
        assert_eq!(
            FrameBuffer::truecolor_code(10, 20, 30, false),
            "\x1b[48;2;10;20;30m"
        );
    }

    #[test]
    fn test_frame_buffer_grid_out_of_bounds() {
        let mut fb = FrameBuffer::new(10, 10, ColorMode::TrueColor, None);
        fb.render_grid_background(
            15,
            15,
            RgbColor {
                r: 255,
                g: 255,
                b: 255,
            },
            1.0,
            true,
            true,
        );
        // Should not panic
    }

    #[test]
    fn test_from_downsampled_options() {
        let cells = vec![
            DownsampleCell {
                top: 5.0,
                bottom: 2.0,
                ..Default::default()
            };
            100
        ];
        let mut ed = None;
        let fb = FrameBuffer::from_downsampled(
            &cells,
            10,
            10,
            10.0,
            Palette::Organic,
            Charset::HalfBlock,
            false,
            false,
            ColorMode::TrueColor,
            0.0,
            DitherMode::None,
            &mut ed,
            None,
            false,
            None,
            None,
            1.5,
            None,
            false,
            false,
            60.0,
            1.0,
            0.5,
            false,
            0.3,
            TrailAgeMode::Bidirectional,
            false,
            0.0,
            palette::TemporalMode::Hue,
            palette::PaletteCycle::default(),
            charset::GlyphConfig::default(),
            None,
            crate::render::antialiasing::AaStrength::Off,
        );
        assert_eq!(fb.width(), 10);
        assert_eq!(fb.height(), 10);

        let fb_rev = FrameBuffer::from_downsampled(
            &cells,
            10,
            10,
            10.0,
            Palette::Organic,
            Charset::HalfBlock,
            true,
            false,
            ColorMode::TrueColor,
            0.0,
            DitherMode::None,
            &mut ed,
            None,
            false,
            None,
            None,
            1.5,
            None,
            false,
            false,
            60.0,
            1.0,
            0.5,
            false,
            0.3,
            TrailAgeMode::Bidirectional,
            false,
            0.0,
            palette::TemporalMode::Hue,
            palette::PaletteCycle::default(),
            charset::GlyphConfig::default(),
            None,
            crate::render::antialiasing::AaStrength::Off,
        );
        assert_ne!(fb.cells[0].fg_color_rgb, fb_rev.cells[0].fg_color_rgb);
    }

    #[test]
    fn test_frame_buffer_grid_full() {
        let mut fb = FrameBuffer::new(10, 10, ColorMode::TrueColor, None);
        let color = RgbColor {
            r: 255,
            g: 255,
            b: 255,
        };
        fb.render_grid_background(5, 5, color, 1.0, true, true);
        assert!(fb.cells[5 * 10 + 5].fg_color_rgb.is_some());
    }

    #[test]
    fn test_frame_buffer_grid_8bit() {
        let mut fb = FrameBuffer::new(10, 10, ColorMode::Bits256, None);
        let color = RgbColor {
            r: 255,
            g: 255,
            b: 255,
        };
        fb.render_grid_background(5, 5, color, 1.0, true, true);
        assert!(fb.cells[5 * 10 + 5].fg_color_256.is_some());
    }

    #[test]
    fn test_build_frame_string_cursor_home() {
        let buffer = FrameBuffer::new(5, 3, ColorMode::Bits256, None);
        let frame_str = buffer.build_frame_string(false, ColorMode::Bits256);
        // Frame is wrapped in a synchronized-update region; cursor-home follows BSU.
        assert!(frame_str.starts_with("\x1b[?2026h\x1b[H"));
        assert!(frame_str.ends_with("\x1b[?2026l"));
    }

    #[test]
    fn test_build_frame_string_emits_one_cursor_move_per_row() {
        // Regression: the renderer used to emit an absolute `\x1b[y;xH` move for
        // every cell, bloating the frame ~7x and inducing terminal write
        // back-pressure that slowed the fixed-step sim while a key was held.
        // Cursor moves should now appear only at the start of each row (column 1),
        // never mid-row (e.g. column 2+).
        let buffer = FrameBuffer::new(5, 3, ColorMode::Bits256, None);
        let frame_str = buffer.build_frame_string(false, ColorMode::Bits256);

        // No per-cell move into an inner column.
        assert!(!frame_str.contains("\x1b[1;2H"));
        assert!(!frame_str.contains("\x1b[2;3H"));
        // One explicit move per row, to column 1.
        for y in 1..=3 {
            assert!(frame_str.contains(&format!("\x1b[{};1H", y)));
        }
        // Total cursor-position sequences (the leading `\x1b[H` home + one per
        // row) stay proportional to height, not width*height.
        let moves = frame_str.matches('H').count();
        assert!(
            moves <= 3 + 1,
            "expected <= height+1 cursor moves, got {moves}"
        );
    }

    #[test]
    fn test_build_frame_string_plain_output() {
        let buffer = FrameBuffer::new(5, 3, ColorMode::Bits256, None);
        let frame_str = buffer.build_frame_string(true, ColorMode::Bits256);
        assert!(!frame_str.contains("\x1b"));
    }

    #[test]
    fn test_build_frame_string_truecolor() {
        let mut buffer = FrameBuffer::new(5, 3, ColorMode::TrueColor, None);
        buffer.cells[0] = Cell {
            char: '█',
            fg_color_256: None,
            bg_color_256: None,
            fg_color_rgb: Some(RgbColor {
                r: 255,
                g: 128,
                b: 64,
            }),
            bg_color_rgb: None,
        };
        let frame_str = buffer.build_frame_string(false, ColorMode::TrueColor);
        assert!(frame_str.starts_with("\x1b[?2026h\x1b[H"));
        assert!(frame_str.ends_with("\x1b[?2026l"));
        assert!(frame_str.contains("\x1b[38;2;255;128;64m"));
    }

    #[test]
    fn test_from_downsampled_empty() {
        let downsampled = vec![
            DownsampleCell {
                top: 0.0,
                bottom: 0.0,
                ..Default::default()
            };
            10
        ];
        let buffer = FrameBuffer::from_downsampled(
            &downsampled,
            10,
            1,
            1.0,
            Palette::Organic,
            Charset::HalfBlock,
            false,
            false,
            ColorMode::Bits256,
            0.0,
            DitherMode::None,
            &mut None,
            None,
            false,
            None,
            None,
            1.5,
            None,
            false,
            false,
            60.0,
            1.0,
            0.5,
            false,
            0.3,
            TrailAgeMode::Bidirectional,
            false,
            0.0,
            palette::TemporalMode::Hue,
            palette::PaletteCycle::default(),
            charset::GlyphConfig::default(),
            None,
            crate::render::antialiasing::AaStrength::Off,
        );
        assert_eq!(buffer.width(), 10);
        assert_eq!(buffer.height(), 1);
    }

    #[test]
    fn test_from_downsampled_with_values() {
        let downsampled = vec![
            DownsampleCell {
                top: 5.0,
                bottom: 0.0,
                top_left: 5.0,
                top_right: 5.0,
                bottom_left: 0.0,
                bottom_right: 0.0,
            },
            DownsampleCell {
                top: 0.0,
                bottom: 5.0,
                top_left: 0.0,
                top_right: 0.0,
                bottom_left: 5.0,
                bottom_right: 5.0,
            },
            DownsampleCell {
                top: 5.0,
                bottom: 5.0,
                top_left: 5.0,
                top_right: 5.0,
                bottom_left: 5.0,
                bottom_right: 5.0,
            },
        ];
        let buffer = FrameBuffer::from_downsampled(
            &downsampled,
            3,
            1,
            5.0,
            Palette::Organic,
            Charset::HalfBlock,
            false,
            false,
            ColorMode::Bits256,
            0.0,
            DitherMode::None,
            &mut None,
            None,
            false,
            None,
            None,
            1.5,
            None,
            false,
            false,
            60.0,
            1.0,
            0.5,
            false,
            0.3,
            TrailAgeMode::Bidirectional,
            false,
            0.0,
            palette::TemporalMode::Hue,
            palette::PaletteCycle::default(),
            charset::GlyphConfig::default(),
            None,
            crate::render::antialiasing::AaStrength::Off,
        );

        assert_eq!(buffer.cells[0].char, '▀');
        assert_eq!(buffer.cells[1].char, '▄');
        assert_eq!(buffer.cells[2].char, '█');
    }

    #[test]
    fn test_draw_text_overlay() {
        let mut buffer = FrameBuffer::new(10, 5, ColorMode::Bits256, None);
        let text = vec!["Hello", "World"];

        buffer.draw_text_overlay(&text, 0, 0, 15, None);

        assert_eq!(buffer.get_cell(0, 0).char, 'H');
        assert_eq!(buffer.get_cell(4, 0).char, 'o');
        assert_eq!(buffer.get_cell(0, 1).char, 'W');
        assert_eq!(buffer.get_cell(4, 1).char, 'd');
        assert_eq!(buffer.get_cell(0, 0).fg_color_256, Some(15));
    }

    #[test]
    fn test_render_grid_background() {
        let mut buffer = FrameBuffer::new(10, 5, ColorMode::TrueColor, None);
        let grid_color = RgbColor {
            r: 100,
            g: 100,
            b: 100,
        };

        // Render intersection
        buffer.render_grid_background(5, 2, grid_color, 0.5, true, true);
        assert_eq!(buffer.get_cell(5, 2).char, '┼');

        // Render vertical
        buffer.render_grid_background(5, 1, grid_color, 0.5, true, false);
        assert_eq!(buffer.get_cell(5, 1).char, '│');

        // Render horizontal
        buffer.render_grid_background(4, 2, grid_color, 0.5, false, true);
        assert_eq!(buffer.get_cell(4, 2).char, '─');
    }

    #[test]
    fn test_frame_buffer_background_color() {
        let bg = RgbColor {
            r: 20,
            g: 20,
            b: 20,
        };
        let buffer = FrameBuffer::new(10, 10, ColorMode::TrueColor, Some(bg));
        assert_eq!(buffer.cells[0].bg_color_rgb, Some(bg));
        assert_eq!(buffer.cells[0].char, ' ');
    }

    #[test]
    fn test_frame_buffer_species_colors() {
        let mut fb = FrameBuffer::new(10, 10, ColorMode::TrueColor, None);
        fb.species_colors_enabled = true;
        fb.species_rgb_colors = vec![
            RgbColor { r: 255, g: 0, b: 0 },
            RgbColor { r: 0, g: 255, b: 0 },
        ];

        let cell = fb.render_colored_cell(
            '#',
            1.0,
            &Palette::Organic,
            false,
            false,
            ColorMode::TrueColor,
            0.0,
            None,
            true,
            Some(&fb.species_rgb_colors),
            0.0,
            0.0,
            palette::TemporalMode::Hue,
            palette::PaletteCycle::default(),
            None,
        );

        // Should be reddish (based on first species color)
        if let Some(rgb) = cell.fg_color_rgb {
            assert!(rgb.r > 100);
            assert!(rgb.g < 100);
            assert!(rgb.b < 100);
        }
    }

    #[test]
    fn test_frame_buffer_species_colors_indexed() {
        let mut fb = FrameBuffer::new(10, 10, ColorMode::Bits256, None);
        fb.species_colors_enabled = true;
        fb.species_rgb_colors = vec![RgbColor { r: 255, g: 0, b: 0 }];

        let cell = fb.render_colored_cell(
            '#',
            1.0,
            &Palette::Organic,
            false,
            false,
            ColorMode::Bits256,
            0.0,
            None,
            true,
            Some(&fb.species_rgb_colors),
            0.0,
            0.0,
            palette::TemporalMode::Hue,
            palette::PaletteCycle::default(),
            None,
        );

        // Should be an index close to red (196 or similar)
        if let Some(idx) = cell.fg_color_256 {
            // Can be standard red (9 or 1) or cube red (196, etc)
            assert!(idx > 0);
        }
    }

    #[test]
    fn test_render_grid_background_empty_cell() {
        let _buffer = FrameBuffer::new(5, 3, ColorMode::Bits256, None);
        // ... (rest of test logic needs mutable buffer)
    }

    #[test]
    fn test_render_grid_background_updates_cell() {
        let mut buffer = FrameBuffer::new(5, 3, ColorMode::Bits256, None);
        let grid_color = RgbColor {
            r: 255,
            g: 255,
            b: 255,
        };

        buffer.render_grid_background(2, 1, grid_color, 0.5, true, true);

        let cell = buffer.cells[7];
        assert_eq!(cell.char, '┼');
        assert!(cell.fg_color_256.is_some());
    }

    #[test]
    fn test_render_grid_background_truecolor() {
        let mut buffer = FrameBuffer::new(5, 3, ColorMode::TrueColor, None);
        let grid_color = RgbColor {
            r: 255,
            g: 255,
            b: 255,
        };

        buffer.render_grid_background(2, 1, grid_color, 0.5, true, false);

        let cell = buffer.cells[7];
        assert_eq!(cell.char, '│');
        assert!(cell.fg_color_rgb.is_some());
    }

    #[test]
    fn test_render_grid_background_non_empty_cell() {
        let mut buffer = FrameBuffer::new(5, 3, ColorMode::Bits256, None);

        // Simulate existing content
        buffer.cells[7] = Cell {
            char: '#',
            fg_color_256: Some(200), // Bright color
            bg_color_256: None,
            fg_color_rgb: None,
            bg_color_rgb: None,
        };

        let grid_color = RgbColor {
            r: 255,
            g: 255,
            b: 255,
        };
        buffer.render_grid_background(2, 1, grid_color, 0.5, true, true);

        let cell = buffer.cells[7];
        // Should NOT be overwritten
        assert_eq!(cell.char, '#');
        assert_eq!(cell.fg_color_256, Some(200));
    }

    #[test]
    fn test_from_downsampled_at_places_cells_at_offset() {
        // A 10×10 terminal buffer with a 4×4 sim at offset (3, 3).
        // Cells at (3,3) and nearby should be non-blank; (0,0) should be blank.
        let sim_w = 4;
        let sim_h = 4;
        let downsampled: Vec<DownsampleCell> = (0..sim_w * sim_h)
            .map(|_| DownsampleCell {
                top: 0.8,
                bottom: 0.8,
                ..Default::default()
            })
            .collect();

        let buffer = FrameBuffer::from_downsampled_at(
            &downsampled,
            sim_w,
            sim_h,
            10,
            10, // term_w, term_h
            3,
            3, // sim_x, sim_y
            1.0,
            Palette::Organic,
            Charset::HalfBlock,
            false,
            false,
            ColorMode::TrueColor,
            0.0,
            DitherMode::None,
            &mut None,
            None,
            false,
            None,
            None,
            1.5,
            None,
            false,
            false,
            15.0,
            0.5,
            0.5,
            false,
            0.3,
            TrailAgeMode::Bidirectional,
            false,
            0.0,
            palette::TemporalMode::Hue,
            palette::PaletteCycle::default(),
            charset::GlyphConfig::default(),
            None,
            crate::render::antialiasing::AaStrength::Off,
        );
        // The buffer should be 10×10
        assert_eq!(buffer.width, 10);
        assert_eq!(buffer.height, 10);
        // Cell at offset (3,3) should have sim content (non-space character or colored)
        let cell_at_offset = &buffer.cells[3 * 10 + 3];
        let cell_at_origin = &buffer.cells[0];
        // Origin should be blank/default
        assert_eq!(cell_at_origin.char, ' ');
        // Note: the exact character depends on palette/charset; just verify it differs from blank
        // (fg_color_rgb will be Some if any trail value was rendered)
        assert!(
            cell_at_offset.fg_color_rgb.is_some() || cell_at_offset.char != ' ',
            "Expected non-blank cell at sim offset but got character='{}' fg={:?}",
            cell_at_offset.char,
            cell_at_offset.fg_color_rgb
        );
    }

    /// Regression: from_downsampled_at must thread aa_strength through to the
    /// inner from_downsampled call.  Before the fix AaStrength::Off was
    /// hardcoded, so the window-layout path ignored the caller's strength.
    /// Uses a 3×3 fullscreen layout (sim == term dims, so no blit) with a
    /// bright center cell surrounded by dim neighbors — the same contrast
    /// pattern as `color_aa_changes_color_not_glyph_for_braille` so the blur
    /// has enough contrast to shift the center cell's color.
    #[test]
    fn from_downsampled_at_honors_aa_strength() {
        use crate::render::antialiasing::AaStrength;

        let w = 3usize;
        let h = 3usize;
        let dim = 0.3f32;
        let mut downsampled = vec![
            DownsampleCell {
                top: dim,
                bottom: dim,
                top_left: dim,
                top_right: dim,
                bottom_left: dim,
                bottom_right: dim,
            };
            w * h
        ];
        // Center cell (index 4) is full brightness; all neighbors are dim.
        let c = &mut downsampled[4];
        c.top = 1.0;
        c.bottom = 1.0;
        c.top_left = 1.0;
        c.top_right = 1.0;
        c.bottom_left = 1.0;
        c.bottom_right = 1.0;

        let build_at = |aa: AaStrength| {
            FrameBuffer::from_downsampled_at(
                &downsampled,
                w,
                h,
                w, // term_w == sim_w (fast path, no blit)
                h, // term_h == sim_h
                0,
                0,
                1.0,
                Palette::Mono,
                Charset::Braille,
                false,
                false,
                ColorMode::TrueColor,
                0.0,
                DitherMode::None,
                &mut None,
                None,
                false,
                None,
                None,
                1.5,
                None,
                false,
                false,
                60.0,
                1.0,
                0.5,
                false,
                0.3,
                TrailAgeMode::Bidirectional,
                false,
                0.0,
                palette::TemporalMode::Hue,
                palette::PaletteCycle::default(),
                charset::GlyphConfig::default(),
                None,
                aa,
            )
        };

        let off_buf = build_at(AaStrength::Off);
        let strong_buf = build_at(AaStrength::Strong);

        // The center cell (index 4) has raw brightness 1.0 but with Strong AA
        // its blurred brightness is pulled toward its dim (0.3) neighbors.
        assert_ne!(
            off_buf.cells[4].fg_color_rgb, strong_buf.cells[4].fg_color_rgb,
            "from_downsampled_at: center cell fg must differ between Off and Strong \
             AA — aa_strength is not being threaded through to from_downsampled"
        );
    }

    /// Contract test: colorize_subpixel produces a different color when
    /// temporal_strength > 0 vs. 0.  This guards the gate that strength=0 is
    /// byte-identical to the non-temporal path while strength>0 changes color.
    #[test]
    fn from_downsampled_applies_temporal_color() {
        use crate::render::palette::{self, Palette, TemporalMode};
        let off = palette::colorize_subpixel(
            0.6,
            Palette::Organic,
            false,
            false,
            0.0,
            None,
            0.5,
            0.0,
            TemporalMode::Hue,
            palette::PaletteCycle::default(),
            None,
        );
        let on = palette::colorize_subpixel(
            0.6,
            Palette::Organic,
            false,
            false,
            0.0,
            None,
            0.5,
            1.0,
            TemporalMode::Hue,
            palette::PaletteCycle::default(),
            None,
        );
        assert_ne!(
            off, on,
            "temporal_strength=1.0 with diff_norm=0.5 must shift color"
        );
    }

    /// End-to-end test: from_downsampled with a non-zero signed_diff in the aux
    /// frame AND temporal_strength > 0 must produce a different cell color than
    /// the temporal-off (strength=0.0) render of the same frame.
    #[test]
    fn from_downsampled_temporal_strength_changes_cell_color() {
        use crate::render::downsample::{AuxCell, AuxFrame};
        use palette::TemporalMode;

        let cells = vec![
            DownsampleCell {
                top: 6.0,
                bottom: 6.0,
                top_left: 6.0,
                top_right: 6.0,
                bottom_left: 6.0,
                bottom_right: 6.0,
            };
            1
        ];
        let max_trail = 10.0_f32;
        // Non-zero signed_diff to drive temporal modulation
        let aux = AuxFrame {
            width: 1,
            height: 1,
            cells: vec![AuxCell {
                age: 0.5,
                delta: 0.0,
                gradient: 0.0,
                signed_diff: 5.0, // diff_norm = 5.0 / 10.0 = 0.5
            }],
        };

        let mut ed = None;
        let fb_off = FrameBuffer::from_downsampled(
            &cells,
            1,
            1,
            max_trail,
            Palette::Organic,
            Charset::HalfBlock,
            false,
            false,
            ColorMode::TrueColor,
            0.0,
            DitherMode::None,
            &mut ed,
            None,
            false,
            None,
            None,
            1.5,
            Some(&aux),
            false,
            false,
            60.0,
            1.0,
            0.5,
            false,
            0.3,
            TrailAgeMode::Bidirectional,
            false,
            0.0, // temporal OFF
            TemporalMode::Hue,
            palette::PaletteCycle::default(),
            charset::GlyphConfig::default(),
            None,
            crate::render::antialiasing::AaStrength::Off,
        );

        let fb_on = FrameBuffer::from_downsampled(
            &cells,
            1,
            1,
            max_trail,
            Palette::Organic,
            Charset::HalfBlock,
            false,
            false,
            ColorMode::TrueColor,
            0.0,
            DitherMode::None,
            &mut ed,
            None,
            false,
            None,
            None,
            1.5,
            Some(&aux),
            false,
            false,
            60.0,
            1.0,
            0.5,
            false,
            0.3,
            TrailAgeMode::Bidirectional,
            false,
            1.0, // temporal ON (strength = 1.0)
            TemporalMode::Hue,
            palette::PaletteCycle::default(),
            charset::GlyphConfig::default(),
            None,
            crate::render::antialiasing::AaStrength::Off,
        );

        assert_ne!(
            fb_off.cells[0].fg_color_rgb, fb_on.cells[0].fg_color_rgb,
            "temporal_strength=1.0 with signed_diff=5.0/max_trail=10.0 must shift cell color"
        );
    }

    #[test]
    fn from_downsampled_cycle_changes_pixels_vs_identity() {
        // Same downsampled field, cycles=1 (identity) vs cycles=3 mirror must differ somewhere.
        use crate::render::palette::{Palette, PaletteCycle, PaletteCycleMode};
        let w = 8usize;
        let h = 4usize;
        let cells: Vec<DownsampleCell> = (0..w * h)
            .map(|i| {
                let v = (i as f32 + 1.0) / (w * h) as f32;
                DownsampleCell {
                    top: v,
                    bottom: v,
                    top_left: v,
                    top_right: v,
                    bottom_left: v,
                    bottom_right: v,
                }
            })
            .collect();
        let id = PaletteCycle::default();
        let active = PaletteCycle {
            cycles: 3,
            mode: PaletteCycleMode::Mirror,
        };
        let mut ed = None;
        let fb_id = FrameBuffer::from_downsampled(
            &cells,
            w,
            h,
            1.0,
            Palette::Organic,
            Charset::HalfBlock,
            false,
            false,
            ColorMode::TrueColor,
            0.0,
            DitherMode::None,
            &mut ed,
            None,
            false,
            None,
            None,
            1.5,
            None,
            false,
            false,
            15.0,
            0.5,
            0.5,
            false,
            0.3,
            TrailAgeMode::Bidirectional,
            false,
            0.0,
            palette::TemporalMode::Hue,
            id,
            charset::GlyphConfig::default(),
            None,
            crate::render::antialiasing::AaStrength::Off,
        );
        let fb_on = FrameBuffer::from_downsampled(
            &cells,
            w,
            h,
            1.0,
            Palette::Organic,
            Charset::HalfBlock,
            false,
            false,
            ColorMode::TrueColor,
            0.0,
            DitherMode::None,
            &mut ed,
            None,
            false,
            None,
            None,
            1.5,
            None,
            false,
            false,
            15.0,
            0.5,
            0.5,
            false,
            0.3,
            TrailAgeMode::Bidirectional,
            false,
            0.0,
            palette::TemporalMode::Hue,
            active,
            charset::GlyphConfig::default(),
            None,
            crate::render::antialiasing::AaStrength::Off,
        );
        // At least one cell must differ in fg_color_rgb between identity and active cycle
        let differs = fb_id
            .cells
            .iter()
            .zip(fb_on.cells.iter())
            .any(|(a, b)| a.fg_color_rgb != b.fg_color_rgb);
        assert!(
            differs,
            "cycles=3 mirror must change rendered colors vs identity"
        );
    }

    fn sample_edge_grid() -> (Vec<DownsampleCell>, usize, usize, f32) {
        let w = 6usize;
        let h = 4usize;
        let cells: Vec<DownsampleCell> = (0..w * h)
            .map(|i| {
                let x = i % w;
                let v = if x < 3 { 0.0f32 } else { 1.0f32 };
                DownsampleCell {
                    top: v,
                    bottom: v,
                    top_left: v,
                    top_right: v,
                    bottom_left: v,
                    bottom_right: v,
                }
            })
            .collect();
        (cells, w, h, 1.0)
    }

    #[test]
    fn glyph_identity_default_matches_native_ascii() {
        let (cells, w, h, maxv) = sample_edge_grid();
        let native = FrameBuffer::from_downsampled(
            &cells,
            w,
            h,
            maxv,
            Palette::Organic,
            Charset::Ascii,
            false,
            false,
            ColorMode::TrueColor,
            0.0,
            DitherMode::None,
            &mut None,
            None,
            false,
            None,
            None,
            1.0,
            None,
            false,
            false,
            0.0,
            0.0,
            0.0,
            false,
            0.0,
            TrailAgeMode::Bidirectional,
            false,
            0.0,
            palette::TemporalMode::Hue,
            palette::PaletteCycle::default(),
            charset::GlyphConfig::default(),
            None,
            crate::render::antialiasing::AaStrength::Off,
        );
        assert!(native.cells.iter().any(|c| c.char != ' '));
    }

    #[test]
    fn glyph_hybrid_emits_directional_on_vertical_edge_ascii() {
        let (cells, w, h, maxv) = sample_edge_grid();
        let hybrid = charset::GlyphConfig {
            selection: Some(charset::GlyphSelection::Hybrid),
            edge_threshold: 0.1,
        };
        let fb = FrameBuffer::from_downsampled(
            &cells,
            w,
            h,
            maxv,
            Palette::Organic,
            Charset::Ascii,
            false,
            false,
            ColorMode::TrueColor,
            0.0,
            DitherMode::None,
            &mut None,
            None,
            false,
            None,
            None,
            1.0,
            None,
            false,
            false,
            0.0,
            0.0,
            0.0,
            false,
            0.0,
            TrailAgeMode::Bidirectional,
            false,
            0.0,
            palette::TemporalMode::Hue,
            palette::PaletteCycle::default(),
            hybrid,
            None,
            crate::render::antialiasing::AaStrength::Off,
        );
        assert!(fb
            .cells
            .iter()
            .any(|c| matches!(c.char, '|' | '/' | '-' | '\\')));
    }

    #[test]
    fn glyph_brightness_forces_ramp_not_shape_braille() {
        let (cells, w, h, maxv) = sample_edge_grid();
        let bri = charset::GlyphConfig {
            selection: Some(charset::GlyphSelection::Brightness),
            edge_threshold: 0.15,
        };
        let fb = FrameBuffer::from_downsampled(
            &cells,
            w,
            h,
            maxv,
            Palette::Organic,
            Charset::Braille,
            false,
            false,
            ColorMode::TrueColor,
            0.0,
            DitherMode::None,
            &mut None,
            None,
            false,
            None,
            None,
            1.0,
            None,
            false,
            false,
            0.0,
            0.0,
            0.0,
            false,
            0.0,
            TrailAgeMode::Bidirectional,
            false,
            0.0,
            palette::TemporalMode::Hue,
            palette::PaletteCycle::default(),
            bri,
            None,
            crate::render::antialiasing::AaStrength::Off,
        );
        assert!(fb
            .cells
            .iter()
            .filter(|c| c.char != ' ')
            .all(|c| ('\u{2800}'..='\u{28FF}').contains(&c.char)));
    }

    #[test]
    fn glyph_noop_on_tonal_halfblock() {
        let (cells, w, h, maxv) = sample_edge_grid();
        let hyb = charset::GlyphConfig {
            selection: Some(charset::GlyphSelection::Hybrid),
            edge_threshold: 0.1,
        };
        let native = charset::GlyphConfig::default();
        let mk = |g| {
            FrameBuffer::from_downsampled(
                &cells,
                w,
                h,
                maxv,
                Palette::Organic,
                Charset::HalfBlock,
                false,
                false,
                ColorMode::TrueColor,
                0.0,
                DitherMode::None,
                &mut None,
                None,
                false,
                None,
                None,
                1.0,
                None,
                false,
                false,
                0.0,
                0.0,
                0.0,
                false,
                0.0,
                TrailAgeMode::Bidirectional,
                false,
                0.0,
                palette::TemporalMode::Hue,
                palette::PaletteCycle::default(),
                g,
                None,
                crate::render::antialiasing::AaStrength::Off,
            )
        };
        let a = mk(hyb);
        let b = mk(native);
        let chars_a: Vec<char> = a.cells.iter().map(|c| c.char).collect();
        let chars_b: Vec<char> = b.cells.iter().map(|c| c.char).collect();
        assert_eq!(
            chars_a, chars_b,
            "glyph-selection must be a no-op on HalfBlock"
        );
    }

    #[test]
    fn color_aa_changes_color_not_glyph_for_braille() {
        use crate::render::antialiasing::AaStrength;
        use crate::render::charset::Charset;
        use crate::render::palette::Palette;

        // 3×3 frame: center cell at full brightness, all others at dim (0.3).
        // With AA on, the center's blurred brightness is pulled down toward its
        // neighbors; with AA off it stays at 1.0. The center cell's color must differ.
        let w = 3usize;
        let h = 3usize;
        let dim = 0.3f32;
        let mut cells = vec![
            DownsampleCell {
                top: dim,
                bottom: dim,
                top_left: dim,
                top_right: dim,
                bottom_left: dim,
                bottom_right: dim,
            };
            w * h
        ];
        let c = &mut cells[4]; // center (y=1, x=1)
        c.top = 1.0;
        c.bottom = 1.0;
        c.top_left = 1.0;
        c.top_right = 1.0;
        c.bottom_left = 1.0;
        c.bottom_right = 1.0;

        let build = |aa: AaStrength| {
            FrameBuffer::from_downsampled(
                &cells,
                w,
                h,
                1.0,
                Palette::Mono,
                Charset::Braille,
                false,
                false,
                ColorMode::TrueColor,
                0.0,
                DitherMode::None,
                &mut None,
                None,
                false,
                None,
                None,
                1.5,
                None,
                false,
                false,
                60.0,
                1.0,
                0.5,
                false,
                0.3,
                crate::config_defaults::TrailAgeMode::Bidirectional,
                false,
                0.0,
                palette::TemporalMode::Hue,
                palette::PaletteCycle::default(),
                charset::GlyphConfig::default(),
                None,
                aa,
            )
        };

        let off = build(AaStrength::Off);
        let strong = build(AaStrength::Strong);

        // The center cell (index 4) has raw brightness 1.0 but with Strong AA its
        // blurred brightness is pulled toward its dim (0.3) neighbors; the resulting
        // color must differ from the non-AA render.
        let off_fg = off.cells[4].fg_color_rgb;
        let strong_fg = strong.cells[4].fg_color_rgb;
        assert_ne!(
            off_fg, strong_fg,
            "AA must change center cell color when neighbors differ"
        );

        // The glyph (shape) of every cell is identical regardless of AA.
        for i in 0..w * h {
            assert_eq!(
                off.cells[i].char, strong.cells[i].char,
                "glyph must not change"
            );
        }
    }

    #[test]
    fn color_aa_off_is_identical_to_no_aa() {
        use crate::render::antialiasing::AaStrength;
        use crate::render::charset::Charset;
        use crate::render::palette::Palette;

        let w = 2usize;
        let h = 2usize;
        let mut cells = vec![DownsampleCell::default(); w * h];
        cells[0].top = 0.8;
        cells[0].bottom = 0.4;

        let build = |cs: Charset, aa: AaStrength| {
            FrameBuffer::from_downsampled(
                &cells,
                w,
                h,
                1.0,
                Palette::Organic,
                cs,
                false,
                false,
                ColorMode::TrueColor,
                0.0,
                DitherMode::None,
                &mut None,
                None,
                false,
                None,
                None,
                1.5,
                None,
                false,
                false,
                60.0,
                1.0,
                0.5,
                false,
                0.3,
                crate::config_defaults::TrailAgeMode::Bidirectional,
                false,
                0.0,
                palette::TemporalMode::Hue,
                palette::PaletteCycle::default(),
                charset::GlyphConfig::default(),
                None,
                aa,
            )
        };

        // For AA-ineligible charsets, AaStrength::Strong must produce byte-identical
        // output to AaStrength::Off because the eligibility gate forces AA off.
        // HalfBlockDual is AA-ineligible (per charset_aa_eligible).
        let a = build(Charset::HalfBlockDual, AaStrength::Strong);
        let b = build(Charset::HalfBlockDual, AaStrength::Off);
        for i in 0..w * h {
            assert_eq!(
                a.cells[i].fg_color_rgb, b.cells[i].fg_color_rgb,
                "AA-ineligible charset: Strong and Off must produce identical fg_color_rgb"
            );
            assert_eq!(
                a.cells[i].char, b.cells[i].char,
                "AA-ineligible charset: Strong and Off must produce identical char"
            );
        }
    }

    /// draw_text_overlay_with_panel must not panic when start_x >= buffer width.
    ///
    /// Before the fix, `.min(self.width - start_x)` underflowed (usize subtraction
    /// wraps), yielding a huge panel width and an out-of-bounds index panic.
    #[test]
    fn draw_text_overlay_with_panel_start_x_gte_width_no_panic() {
        let mut buf = FrameBuffer::new(10, 5, crate::cli::ColorMode::TrueColor, None);
        let lines = ["hello", "world"];

        // start_x exactly equal to width — must return without panic.
        buf.draw_text_overlay_with_panel(
            &lines,
            10, // start_x == width
            0,
            231,
            None,
            Some(crate::render::palette::RgbColor { r: 0, g: 0, b: 0 }),
            Some(crate::render::palette::RgbColor { r: 255, g: 0, b: 0 }),
            1,
        );

        // start_x beyond width — must also return without panic.
        buf.draw_text_overlay_with_panel(
            &lines,
            999, // start_x >> width
            0,
            231,
            None,
            Some(crate::render::palette::RgbColor { r: 0, g: 0, b: 0 }),
            Some(crate::render::palette::RgbColor { r: 255, g: 0, b: 0 }),
            1,
        );
    }
}

#[cfg(test)]
mod rich_overlay_tests {
    use super::*;
    use crate::render::palette::RgbColor;

    #[test]
    fn writes_fg_override_as_is() {
        let mut fb = FrameBuffer::new(2, 1, ColorMode::TrueColor, None);
        let rich = vec![vec![('A', Some(RgbColor::new(200, 200, 200)), None)]];
        fb.draw_rich_overlay_solid(&rich, 0, 0, false);
        assert_eq!(fb.cells[0].fg_color_rgb, Some(RgbColor::new(200, 200, 200)));
    }

    #[test]
    fn solid_false_leaves_space_transparent() {
        // Space char = transparent when not solid; existing char is preserved.
        let mut fb = FrameBuffer::new(2, 1, ColorMode::TrueColor, None);
        fb.cells[0].char = 'X';
        let rich = vec![vec![(' ', Some(RgbColor::new(100, 100, 100)), None)]];
        fb.draw_rich_overlay_solid(&rich, 0, 0, false);
        assert_eq!(fb.cells[0].char, 'X');
    }

    #[test]
    fn solid_true_writes_space() {
        // Space char is written when solid, masking the underlying cell.
        let mut fb = FrameBuffer::new(2, 1, ColorMode::TrueColor, None);
        fb.cells[0].char = 'X';
        let rich = vec![vec![(' ', Some(RgbColor::new(100, 100, 100)), None)]];
        fb.draw_rich_overlay_solid(&rich, 0, 0, true);
        assert_eq!(fb.cells[0].char, ' ');
    }
}
