//! Terminal output rendering and buffer management.
//!
//! This module handles the low-level details of converting simulation data into
//! ANSI escape sequences or character grids for display in the terminal.
//! It supports double-buffering, color mapping, downsampling, and various
//! character sets (Block, Braille, ASCII).

use crate::cli::ColorMode;
use crate::cli::Palette;
use crate::render::charset::{self, Charset};
use crate::render::dither::{self, DitherMode};
use crate::render::downsample::{downsample_multi_species, Cell as DownsampleCell};
use crate::render::error_diffusion::ErrorDiffusion;
use crate::render::overlay::OverlayConfig;
use crate::render::panel::RenderedOverlay;
use crate::render::palette;
use crate::render::palette::IntensityMapping;
use crate::render::palette::RgbColor;
use crate::render::theme::PanelStyle;
use crossterm::{execute, Command};
use std::fmt;
use std::io::{self, Stdout};

#[derive(Clone, Copy)]
struct Cell {
    char: char,
    fg_color_256: Option<u8>,
    bg_color_256: Option<u8>,
    fg_color_rgb: Option<RgbColor>,
    bg_color_rgb: Option<RgbColor>,
}

/// A double-buffered screen buffer for terminal rendering.
///
/// Stores character and color information for each cell in the terminal grid.
/// Handles efficient updates and string building for output.
pub struct FrameBuffer {
    width: usize,
    height: usize,
    cells: Vec<Cell>,
    color_mode: ColorMode,
    species_colors_enabled: bool,
    species_rgb_colors: Vec<RgbColor>,
    background_color: Option<RgbColor>,
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
        let is_empty = match self.color_mode {
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
    ) -> Self {
        let mut buffer = Self::new(width, height, color_mode, background_color);
        buffer.species_colors_enabled = species_colors_enabled;

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

            let top_brightness = if max_trail_value > 0.0 {
                dcell.top / max_trail_value
            } else {
                0.0
            };
            let bottom_brightness = if max_trail_value > 0.0 {
                dcell.bottom / max_trail_value
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
                hue_shift,
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
        const THRESHOLD: f32 = 0.05;

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
                    Charset::Braille => {
                        charset::map_brightness(top_adj, Some(bottom_adj), charset.clone())
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
                    Charset::Ascii | Charset::CustomAscii(_) => {
                        charset::map_brightness((top_adj + bottom_adj) / 2.0, None, charset.clone())
                    }
                }
            } else if top_adj > bottom_adj {
                match charset {
                    Charset::Braille => {
                        charset::map_brightness(top_adj, Some(bottom_adj), charset.clone())
                    }
                    Charset::HalfBlock => charset::map_vertical_block(top_adj, bottom_adj),
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
                    Charset::Ascii => charset::map_ascii_directional(top_adj, true),
                    Charset::CustomAscii(_) => {
                        charset::map_brightness(top_adj, None, charset.clone())
                    }
                }
            } else {
                match charset {
                    Charset::Braille => {
                        charset::map_brightness(top_adj, Some(bottom_adj), charset.clone())
                    }
                    Charset::HalfBlock => charset::map_vertical_block(top_adj, bottom_adj),
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
                    Charset::Ascii => charset::map_ascii_directional(bottom_adj, false),
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
    );

    execute!(std::io::stdout(), &buffer)
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
    #[allow(dead_code)]
    pub fn dither_mode(&self) -> DitherMode {
        self.dither_mode
    }

    /// Reset error diffusion error accumulators.
    ///
    /// Should be called at the start of each frame.
    #[allow(dead_code)]
    pub fn reset_error_diffusion(&mut self) {
        if let Some(ref mut ed) = self.error_diffusion {
            ed.reset();
        }
    }

    /// Resize error diffusion buffers.
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
    #[allow(dead_code)]
    pub fn set_palette(&mut self, palette: Palette) {
        self.palette = palette;
    }

    /// Set the hue shift amount (0.0 to 1.0).
    #[allow(dead_code)]
    pub fn set_hue_shift(&mut self, hue_shift: f32) {
        self.hue_shift = hue_shift;
    }

    /// Update the character set used for rendering.
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

    /// Set specific colors for multi-species rendering.
    pub fn set_species_colors(&mut self, enabled: bool, colors: Vec<RgbColor>) {
        self.species_colors_enabled = enabled;
        self.species_rgb_colors = colors;
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
        controls_lines: Option<(&RenderedOverlay, usize, usize)>,
        status_line: Option<(String, usize)>,
        notification_line: Option<(String, usize)>,
        stats_lines: Option<(&RenderedOverlay, usize)>,
        info_lines: Option<(&RenderedOverlay, usize, usize)>,
        grid_renderer: Option<&crate::render::grid::GridRenderer>,
        config_browser_lines: Option<(&RenderedOverlay, usize, usize)>,
        config_save_lines: Option<(&RenderedOverlay, usize, usize)>,
        keyboard_hints_lines: Option<(&RenderedOverlay, usize, usize)>,
        preset_comparison_lines: Option<(&RenderedOverlay, usize, usize)>,
        panel_style: Option<&crate::render::theme::PanelStyle>,
        _focused_overlay: Option<crate::terminal::control::OverlayType>,
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
        );

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

        // Controls overlay at top-left
        if let Some((overlay, x, y)) = controls_lines {
            let config = &OverlayConfig::CONTROLS;
            let (fg, bg, panel_bg, ind, w) = get_overlay_colors(config, panel_style);
            if panel_bg.is_some() {
                buffer.draw_text_overlay_with_panel(&overlay.lines, x, y, fg, bg, panel_bg, ind, w);
            } else {
                buffer.draw_text_overlay(&overlay.lines, x, y, fg, bg);
            }
            if let Some(tb) = &overlay.title_box {
                let mini_x = x + 1 + tb.col_offset;
                let mini_y = y.saturating_sub(1);
                if panel_bg.is_some() {
                    buffer.draw_text_overlay_with_panel(&tb.lines, mini_x, mini_y, fg, bg, panel_bg, ind, w);
                } else {
                    buffer.draw_text_overlay(&tb.lines, mini_x, mini_y, fg, bg);
                }
            }
        }

        // Status line at bottom
        if let Some((line, x)) = status_line {
            let config = &OverlayConfig::STATUS;
            let line_chars: Vec<char> = line.chars().collect();
            buffer.draw_text_overlay(
                &[&line_chars.iter().collect::<String>()],
                x,
                self.height.saturating_sub(2),
                config.text_color_256,
                Some(config.bg_color_256),
            );
        }

        // Notification at bottom-center
        if let Some((text, x)) = notification_line {
            let config = &OverlayConfig::NOTIFICATION;
            let text_chars: Vec<char> = text.chars().collect();
            buffer.draw_text_overlay(
                &[&text_chars.iter().collect::<String>()],
                x,
                self.height.saturating_sub(4),
                config.text_color_256,
                Some(config.bg_color_256),
            );
        }

        // Stats overlay at top-right
        if let Some((overlay, x)) = stats_lines {
            let config = &OverlayConfig::STATS;
            let (fg, bg, panel_bg, ind, w) = get_overlay_colors(config, panel_style);
            if panel_bg.is_some() {
                buffer.draw_text_overlay_with_panel(&overlay.lines, x, 2, fg, bg, panel_bg, ind, w);
            } else {
                buffer.draw_text_overlay(&overlay.lines, x, 2, fg, bg);
            }
            if let Some(tb) = &overlay.title_box {
                let mini_x = x + 1 + tb.col_offset;
                if panel_bg.is_some() {
                    buffer.draw_text_overlay_with_panel(&tb.lines, mini_x, 1, fg, bg, panel_bg, ind, w);
                } else {
                    buffer.draw_text_overlay(&tb.lines, mini_x, 1, fg, bg);
                }
            }
        }

        // Info overlay at top-right (below stats)
        if let Some((overlay, x, y)) = info_lines {
            let config = &OverlayConfig::INFO;
            let (fg, bg, panel_bg, ind, w) = get_overlay_colors(config, panel_style);
            if panel_bg.is_some() {
                buffer.draw_text_overlay_with_panel(&overlay.lines, x, y, fg, bg, panel_bg, ind, w);
            } else {
                buffer.draw_text_overlay(&overlay.lines, x, y, fg, bg);
            }
            if let Some(tb) = &overlay.title_box {
                let mini_x = x + 1 + tb.col_offset;
                let mini_y = y.saturating_sub(1);
                if panel_bg.is_some() {
                    buffer.draw_text_overlay_with_panel(&tb.lines, mini_x, mini_y, fg, bg, panel_bg, ind, w);
                } else {
                    buffer.draw_text_overlay(&tb.lines, mini_x, mini_y, fg, bg);
                }
            }
        }

        // Config browser overlay (modal, on top)
        if let Some((overlay, x, y)) = config_browser_lines {
            let config = &OverlayConfig::CONFIG_BROWSER;
            let (fg, bg, panel_bg, ind, w) = get_overlay_colors(config, panel_style);
            if panel_bg.is_some() {
                buffer.draw_text_overlay_with_panel(&overlay.lines, x, y, fg, bg, panel_bg, ind, w);
            } else {
                buffer.draw_text_overlay(&overlay.lines, x, y, fg, bg);
            }
            if let Some(tb) = &overlay.title_box {
                let mini_x = x + 1 + tb.col_offset;
                let mini_y = y.saturating_sub(1);
                if panel_bg.is_some() {
                    buffer.draw_text_overlay_with_panel(&tb.lines, mini_x, mini_y, fg, bg, panel_bg, ind, w);
                } else {
                    buffer.draw_text_overlay(&tb.lines, mini_x, mini_y, fg, bg);
                }
            }
        }

        // Config save overlay (modal, on top) in render_with_overlay
        if let Some((overlay, x, y)) = config_save_lines {
            let config = &OverlayConfig::CONFIG_SAVE;
            let (fg, bg, panel_bg, ind, w) = get_overlay_colors(config, panel_style);
            if panel_bg.is_some() {
                buffer.draw_text_overlay_with_panel(&overlay.lines, x, y, fg, bg, panel_bg, ind, w);
            } else {
                buffer.draw_text_overlay(&overlay.lines, x, y, fg, bg);
            }
            if let Some(tb) = &overlay.title_box {
                let mini_x = x + 1 + tb.col_offset;
                let mini_y = y.saturating_sub(1);
                if panel_bg.is_some() {
                    buffer.draw_text_overlay_with_panel(&tb.lines, mini_x, mini_y, fg, bg, panel_bg, ind, w);
                } else {
                    buffer.draw_text_overlay(&tb.lines, mini_x, mini_y, fg, bg);
                }
            }
        }

        // Keyboard hints overlay (modal, on top)
        if let Some((overlay, x, y)) = keyboard_hints_lines {
            let config = &OverlayConfig::KEYBOARD_HINTS;
            let (fg, bg, panel_bg, ind, w) = get_overlay_colors(config, panel_style);
            if panel_bg.is_some() {
                buffer.draw_text_overlay_with_panel(&overlay.lines, x, y, fg, bg, panel_bg, ind, w);
            } else {
                buffer.draw_text_overlay(&overlay.lines, x, y, fg, bg);
            }
            if let Some(tb) = &overlay.title_box {
                let mini_x = x + 1 + tb.col_offset;
                let mini_y = y.saturating_sub(1);
                if panel_bg.is_some() {
                    buffer.draw_text_overlay_with_panel(&tb.lines, mini_x, mini_y, fg, bg, panel_bg, ind, w);
                } else {
                    buffer.draw_text_overlay(&tb.lines, mini_x, mini_y, fg, bg);
                }
            }
        }

        // Preset comparison overlay (modal, on top)
        if let Some((overlay, x, y)) = preset_comparison_lines {
            let config = &OverlayConfig::PRESET_COMPARISON;
            let (fg, bg, panel_bg, ind, w) = get_overlay_colors(config, panel_style);
            if panel_bg.is_some() {
                buffer.draw_text_overlay_with_panel(&overlay.lines, x, y, fg, bg, panel_bg, ind, w);
            } else {
                buffer.draw_text_overlay(&overlay.lines, x, y, fg, bg);
            }
            if let Some(tb) = &overlay.title_box {
                let mini_x = x + 1 + tb.col_offset;
                let mini_y = y.saturating_sub(1);
                if panel_bg.is_some() {
                    buffer.draw_text_overlay_with_panel(&tb.lines, mini_x, mini_y, fg, bg, panel_bg, ind, w);
                } else {
                    buffer.draw_text_overlay(&tb.lines, mini_x, mini_y, fg, bg);
                }
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
        controls_lines: Option<(&RenderedOverlay, usize, usize)>,
        status_line: Option<(String, usize)>,
        notification_line: Option<(String, usize)>,
        stats_lines: Option<(&RenderedOverlay, usize)>,
        info_lines: Option<(&RenderedOverlay, usize, usize)>,
        grid_renderer: Option<&crate::render::grid::GridRenderer>,
        config_browser_lines: Option<(&RenderedOverlay, usize, usize)>,
        config_save_lines: Option<(&RenderedOverlay, usize, usize)>,
        keyboard_hints_lines: Option<(&RenderedOverlay, usize, usize)>,
        preset_comparison_lines: Option<(&RenderedOverlay, usize, usize)>,
        _panel_style: Option<&crate::render::theme::PanelStyle>,
        _focused_overlay: Option<crate::terminal::control::OverlayType>,
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
            );

            for (i, cell) in species_buffer.cells.iter().enumerate() {
                if cell.char != ' ' {
                    buffer.cells[i] = *cell;
                }
            }
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

        // Controls overlay at top-left
        if let Some((overlay, x, y)) = controls_lines {
            let config = &OverlayConfig::CONTROLS;
            buffer.draw_text_overlay(
                &overlay.lines,
                x,
                y,
                config.text_color_256,
                Some(config.bg_color_256),
            );
            if let Some(tb) = &overlay.title_box {
                let mini_x = x + 1 + tb.col_offset;
                let mini_y = y.saturating_sub(1);
                buffer.draw_text_overlay(&tb.lines, mini_x, mini_y, config.text_color_256, Some(config.bg_color_256));
            }
        }

        // Status line at bottom
        if let Some((line, x)) = status_line {
            let config = &OverlayConfig::STATUS;
            let line_chars: Vec<char> = line.chars().collect();
            buffer.draw_text_overlay(
                &[&line_chars.iter().collect::<String>()],
                x,
                self.height.saturating_sub(2),
                config.text_color_256,
                Some(config.bg_color_256),
            );
        }

        // Notification at bottom-center
        if let Some((text, x)) = notification_line {
            let config = &OverlayConfig::NOTIFICATION;
            let text_chars: Vec<char> = text.chars().collect();
            buffer.draw_text_overlay(
                &[&text_chars.iter().collect::<String>()],
                x,
                self.height.saturating_sub(4),
                config.text_color_256,
                Some(config.bg_color_256),
            );
        }

        // Stats overlay at top-right
        if let Some((overlay, x)) = stats_lines {
            let config = &OverlayConfig::STATS;
            buffer.draw_text_overlay(&overlay.lines, x, 2, config.text_color_256, Some(config.bg_color_256));
            if let Some(tb) = &overlay.title_box {
                let mini_x = x + 1 + tb.col_offset;
                buffer.draw_text_overlay(&tb.lines, mini_x, 1, config.text_color_256, Some(config.bg_color_256));
            }
        }

        // Info overlay at top-right (below stats)
        if let Some((overlay, x, y)) = info_lines {
            let config = &OverlayConfig::INFO;
            buffer.draw_text_overlay(&overlay.lines, x, y, config.text_color_256, Some(config.bg_color_256));
            if let Some(tb) = &overlay.title_box {
                let mini_x = x + 1 + tb.col_offset;
                let mini_y = y.saturating_sub(1);
                buffer.draw_text_overlay(&tb.lines, mini_x, mini_y, config.text_color_256, Some(config.bg_color_256));
            }
        }

        // Config browser overlay (modal, on top)
        if let Some((overlay, x, y)) = config_browser_lines {
            let config = &OverlayConfig::CONFIG_BROWSER;
            buffer.draw_text_overlay(&overlay.lines, x, y, config.text_color_256, Some(config.bg_color_256));
            if let Some(tb) = &overlay.title_box {
                let mini_x = x + 1 + tb.col_offset;
                let mini_y = y.saturating_sub(1);
                buffer.draw_text_overlay(&tb.lines, mini_x, mini_y, config.text_color_256, Some(config.bg_color_256));
            }
        }

        // Config save overlay (modal, on top) in render_multi_species_with_overlay
        if let Some((overlay, x, y)) = config_save_lines {
            let config = &OverlayConfig::CONFIG_SAVE;
            buffer.draw_text_overlay(&overlay.lines, x, y, config.text_color_256, Some(config.bg_color_256));
            if let Some(tb) = &overlay.title_box {
                let mini_x = x + 1 + tb.col_offset;
                let mini_y = y.saturating_sub(1);
                buffer.draw_text_overlay(&tb.lines, mini_x, mini_y, config.text_color_256, Some(config.bg_color_256));
            }
        }

        // Keyboard hints overlay (modal, on top)
        if let Some((overlay, x, y)) = keyboard_hints_lines {
            let config = &OverlayConfig::KEYBOARD_HINTS;
            buffer.draw_text_overlay(&overlay.lines, x, y, config.text_color_256, Some(config.bg_color_256));
            if let Some(tb) = &overlay.title_box {
                let mini_x = x + 1 + tb.col_offset;
                let mini_y = y.saturating_sub(1);
                buffer.draw_text_overlay(&tb.lines, mini_x, mini_y, config.text_color_256, Some(config.bg_color_256));
            }
        }

        // Preset comparison overlay (modal, on top)
        if let Some((overlay, x, y)) = preset_comparison_lines {
            let config = &OverlayConfig::PRESET_COMPARISON;
            buffer.draw_text_overlay(&overlay.lines, x, y, config.text_color_256, Some(config.bg_color_256));
            if let Some(tb) = &overlay.title_box {
                let mini_x = x + 1 + tb.col_offset;
                let mini_y = y.saturating_sub(1);
                buffer.draw_text_overlay(&tb.lines, mini_x, mini_y, config.text_color_256, Some(config.bg_color_256));
            }
        }

        execute!(self.stdout, &buffer)
    }
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
        );
        assert_eq!(fb.width, 10);
        assert_eq!(fb.height, 10);

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
        );
        assert_ne!(fb.cells[0].fg_color_rgb, fb_rev.cells[0].fg_color_rgb);
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
        );
        assert!(result.is_ok());
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
                ..Default::default()
            },
            DownsampleCell {
                top: 0.0,
                bottom: 5.0,
                ..Default::default()
            },
            DownsampleCell {
                top: 5.0,
                bottom: 5.0,
                ..Default::default()
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

        let cell = buffer.cells[1 * 5 + 2];
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

        let cell = buffer.cells[1 * 5 + 2];
        assert_eq!(cell.char, '│');
        assert!(cell.fg_color_rgb.is_some());
    }

    #[test]
    fn test_render_grid_background_non_empty_cell() {
        let mut buffer = FrameBuffer::new(5, 3, ColorMode::Bits256, None);

        // Simulate existing content
        buffer.cells[1 * 5 + 2] = Cell {
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

        let cell = buffer.cells[1 * 5 + 2];
        // Should NOT be overwritten
        assert_eq!(cell.char, '#');
        assert_eq!(cell.fg_color_256, Some(200));
    }
}
