//! Frame buffer for terminal rendering.
//!
//! This module provides a double-buffered screen buffer for terminal rendering,
//! storing character and color information for each cell in the terminal grid.

use crate::cli::ColorMode;
use crate::cli::Palette;
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
    pub(crate) char: char,
    /// Foreground color in ANSI 256 format.
    pub(crate) fg_color_256: Option<u8>,
    /// Background color in ANSI 256 format.
    pub(crate) bg_color_256: Option<u8>,
    /// Foreground color in RGB format.
    pub(crate) fg_color_rgb: Option<RgbColor>,
    /// Background color in RGB format.
    pub(crate) bg_color_rgb: Option<RgbColor>,
}

/// A double-buffered screen buffer for terminal rendering.
///
/// Stores character and color information for each cell in the terminal grid.
/// Handles efficient updates and string building for output.
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
        }
    }

    fn set_cell(&mut self, x: usize, y: usize, cell: Cell) {
        if x < self.width && y < self.height {
            self.cells[y * self.width + x] = cell;
        }
    }

    #[cfg(test)]
    fn get_cell(&self, x: usize, y: usize) -> &Cell {
        &self.cells[y * self.width + x]
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
        let height = if width > 0 {
            downsampled.len() / width
        } else {
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

        // Only render grid where there's no (or very dim) simulation content
        // Check if this cell is essentially empty (dark background)
        // A cell is empty if it displays a space character OR has no/dark foreground color
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
            // Apply grid with opacity to empty cells
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
        // If cell is not empty, don't render grid (simulation takes precedence)
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
            if w > 0 && start_y < self.height {
                let panel_width = text_lines
                    .iter()
                    .map(|l| l.as_ref().chars().count())
                    .max()
                    .unwrap_or(0)
                    .saturating_add(start_x)
                    .min(self.width - start_x);

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
    pub fn draw_rich_overlay(&mut self, rich: &[Vec<RichCell>], start_x: usize, start_y: usize) {
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
                // Write character (space = transparent, let sim show through)
                if ch != ' ' {
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

    /// Calculates the maximum brightness in a frame.
    ///
    /// This helper function is currently only used in tests but is retained
    /// as it may be useful for future features like automatic brightness normalization
    /// or exposure adjustment.
    #[allow(dead_code)]
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
    ) -> Self {
        let mut buffer = Self::new(width, height, color_mode, background_color);
        buffer.species_colors_enabled = species_colors_enabled;
        buffer.ascii_contrast = ascii_contrast;

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
            let cell_hue_shift = if let Some(aux) = aux_frame {
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
                if trail_age_enabled {
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
                }
            } else {
                hue_shift
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
            );
            buffer.set_cell(x, y, cell);
        }

        buffer.species_rgb_colors = species_rgb_colors.unwrap_or_default();

        buffer
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
            let char = if top_adj > THRESHOLD && bottom_adj > THRESHOLD {
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

            let brightness = if top_adj > THRESHOLD && bottom_adj > THRESHOLD {
                (top_adj + bottom_adj) / 2.0
            } else if top_adj > bottom_adj {
                top_adj
            } else {
                bottom_adj
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
                    palette::map_brightness_rgb(
                        brightness,
                        palette.clone(),
                        reverse_palette,
                        invert_palette,
                        hue_shift,
                        intensity_mapping,
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
                    palette::map_brightness(
                        brightness,
                        palette.clone(),
                        reverse_palette,
                        invert_palette,
                        intensity_mapping,
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

    #[allow(dead_code)]
    fn ansi_color_code(color: u8, is_fg: bool) -> String {
        if is_fg {
            format!("\x1b[38;5;{}m", color)
        } else {
            format!("\x1b[48;5;{}m", color)
        }
    }

    #[allow(dead_code)]
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
            output.push_str("\x1b[H");
        }

        let mut last_fg_256: Option<u8> = None;
        let mut last_bg_256: Option<u8> = None;
        let mut last_fg_rgb: Option<RgbColor> = None;
        let mut last_bg_rgb: Option<RgbColor> = None;

        for y in 0..self.height {
            for x in 0..self.width {
                let cell = self.cells[y * self.width + x];

                if !plain_output {
                    output.push_str(&format!("\x1b[{};{}H", y + 1, x + 1));

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

        output
    }

    /// Darken all cells and add scanlines for VCR freeze-frame look.
    ///
    /// Called after `from_downsampled` when the simulation is paused.
    /// - All cells: multiply RGB by `DIM` (~0.22)
    /// - Even rows: multiply again by `SCANLINE` (~0.6) to create CRT scanline effect
    pub fn apply_vcr_pause_effect(&mut self, frame_counter: u64) {
        const DIM: f32 = 0.40;
        const SCANLINE_DARK: f32 = 0.55; // even rows get this multiplier on top of DIM
        const NOISE_AMOUNT: f32 = 0.08; // ±8% brightness jitter

        for y in 0..self.height {
            let scanline = if y % 2 == 0 { SCANLINE_DARK } else { 1.0 };
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
#[allow(dead_code)]
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
        assert!(frame_str.starts_with("\x1b[H"));
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
        assert!(frame_str.starts_with("\x1b[H"));
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
}
