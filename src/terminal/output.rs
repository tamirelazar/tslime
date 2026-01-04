use crate::cli::ColorMode;
use crate::cli::Palette;
use crate::render::charset::{self, Charset};
use crate::render::dither::{self, DitherMode};
use crate::render::downsample::{downsample_multi_species, Cell as DownsampleCell};
use crate::render::error_diffusion::ErrorDiffusion;
use crate::render::palette;
use crate::render::palette::RgbColor;
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

pub struct FrameBuffer {
    width: usize,
    height: usize,
    cells: Vec<Cell>,
    color_mode: ColorMode,
    species_colors_enabled: bool,
    species_rgb_colors: Vec<RgbColor>,
}

impl FrameBuffer {
    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn new(width: usize, height: usize, color_mode: ColorMode) -> Self {
        Self {
            width,
            height,
            cells: vec![
                Cell {
                    char: ' ',
                    fg_color_256: None,
                    bg_color_256: None,
                    fg_color_rgb: None,
                    bg_color_rgb: None,
                };
                width * height
            ],
            color_mode,
            species_colors_enabled: false,
            species_rgb_colors: Vec::new(),
        }
    }

    fn set_cell(&mut self, x: usize, y: usize, cell: Cell) {
        if x < self.width && y < self.height {
            self.cells[y * self.width + x] = cell;
        }
    }

    pub fn draw_text_overlay<T: AsRef<str>>(
        &mut self,
        text_lines: &[T],
        start_x: usize,
        start_y: usize,
        fg_color: u8,
        bg_color: Option<u8>,
    ) {
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
                self.cells[y * self.width + x] = Cell {
                    char: ch,
                    fg_color_256: Some(fg_color),
                    bg_color_256: bg_color,
                    fg_color_rgb: None,
                    bg_color_rgb: None,
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
        species_colors_enabled: bool,
        species_rgb_colors: Option<Vec<RgbColor>>,
    ) -> Self {
        let mut buffer = Self::new(width, height, color_mode);
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
                &palette,
                charset,
                reverse_palette,
                invert_palette,
                color_mode,
                hue_shift,
                dither_mode,
                error_diffusion,
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
        palette: &Palette,
        charset: Charset,
        reverse_palette: bool,
        invert_palette: bool,
        color_mode: ColorMode,
        hue_shift: f32,
        dither_mode: DitherMode,
        error_diffusion: &mut Option<ErrorDiffusion>,
        species_colors_enabled: bool,
        species_rgb_colors: Option<&[RgbColor]>,
    ) -> Cell {
        const THRESHOLD: f32 = 0.05;

        let levels = charset::charset_level_count(charset);

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
                bg_color_256: None,
                fg_color_rgb: None,
                bg_color_rgb: None,
            }
        } else {
            let char = if top_adj > THRESHOLD && bottom_adj > THRESHOLD {
                match charset {
                    Charset::HalfBlock => charset::map_vertical_block(top_adj, bottom_adj),
                    Charset::Braille => charset::map_brightness(top_adj, Some(bottom_adj), charset),
                    _ => charset::map_brightness((top_adj + bottom_adj) / 2.0, None, charset),
                }
            } else if top_adj > bottom_adj {
                match charset {
                    Charset::Braille => charset::map_brightness(top_adj, Some(bottom_adj), charset),
                    Charset::HalfBlock => charset::map_vertical_block(top_adj, bottom_adj),
                    Charset::Ascii => charset::map_ascii_directional(top_adj, true),
                }
            } else {
                match charset {
                    Charset::Braille => charset::map_brightness(top_adj, Some(bottom_adj), charset),
                    Charset::HalfBlock => charset::map_vertical_block(top_adj, bottom_adj),
                    Charset::Ascii => charset::map_ascii_directional(bottom_adj, false),
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
                    )
                };
                Cell {
                    char,
                    fg_color_256: None,
                    bg_color_256: None,
                    fg_color_rgb: Some(rgb),
                    bg_color_rgb: None,
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
                    )
                };
                Cell {
                    char,
                    fg_color_256: Some(color),
                    bg_color_256: None,
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
    species_colors_enabled: bool,
    species_rgb_colors: Option<Vec<RgbColor>>,
    error_diffusion: &mut Option<ErrorDiffusion>,
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
        species_colors_enabled,
        species_rgb_colors,
    );

    execute!(std::io::stdout(), &buffer)
}

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
    error_diffusion: Option<ErrorDiffusion>,
    species_colors_enabled: bool,
    species_rgb_colors: Vec<RgbColor>,
}

impl TerminalRenderer {
    pub fn new(
        width: usize,
        height: usize,
        palette: Palette,
        charset: Charset,
        reverse_palette: bool,
        invert_palette: bool,
        color_mode: ColorMode,
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
            error_diffusion: None,
            species_colors_enabled: false,
            species_rgb_colors: Vec::new(),
        }
    }

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

    #[allow(dead_code)]
    pub fn dither_mode(&self) -> DitherMode {
        self.dither_mode
    }

    #[allow(dead_code)]
    pub fn reset_error_diffusion(&mut self) {
        if let Some(ref mut ed) = self.error_diffusion {
            ed.reset();
        }
    }

    #[allow(dead_code)]
    pub fn resize_error_diffusion(&mut self, width: usize, height: usize) {
        if let Some(ref mut ed) = self.error_diffusion {
            ed.resize(width, height);
        }
    }

    pub fn set_dimensions(&mut self, width: usize, height: usize) {
        self.width = width;
        self.height = height;
    }

    #[allow(dead_code)]
    pub fn set_palette(&mut self, palette: Palette) {
        self.palette = palette;
    }

    #[allow(dead_code)]
    pub fn set_hue_shift(&mut self, hue_shift: f32) {
        self.hue_shift = hue_shift;
    }

    #[allow(dead_code)]
    pub fn set_charset(&mut self, charset: Charset) {
        self.charset = charset;
    }

    pub fn set_invert_palette(&mut self, invert: bool) {
        self.invert_palette = invert;
    }

    pub fn set_reverse_palette(&mut self, reverse: bool) {
        self.reverse_palette = reverse;
    }

    pub fn set_species_colors(&mut self, enabled: bool, colors: Vec<RgbColor>) {
        self.species_colors_enabled = enabled;
        self.species_rgb_colors = colors;
    }

    #[allow(dead_code)]
    pub fn stdout_mut(&mut self) -> &mut Stdout {
        &mut self.stdout
    }

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
            self.charset,
            self.reverse_palette,
            self.invert_palette,
            self.color_mode,
            self.hue_shift,
            self.dither_mode,
            &mut self.error_diffusion,
            self.species_colors_enabled,
            if self.species_colors_enabled {
                Some(self.species_rgb_colors.clone())
            } else {
                None
            },
        );

        execute!(self.stdout, &buffer)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn render_with_overlay<T: AsRef<str>>(
        &mut self,
        downsampled: &[DownsampleCell],
        max_trail_value: f32,
        help_lines: Option<(&[T], usize, usize)>,
        status_line: Option<(String, usize)>,
        paused_line: Option<(String, usize)>,
        notification_line: Option<(String, usize)>,
        stats_lines: Option<&[String]>,
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
            self.charset,
            self.reverse_palette,
            self.invert_palette,
            self.color_mode,
            self.hue_shift,
            self.dither_mode,
            &mut self.error_diffusion,
            self.species_colors_enabled,
            if self.species_colors_enabled {
                Some(self.species_rgb_colors.clone())
            } else {
                None
            },
        );

        if let Some((lines, x, y)) = help_lines {
            buffer.draw_text_overlay(lines, x, y, 15, Some(236));
        }

        if let Some((line, x)) = status_line {
            let line_chars: Vec<char> = line.chars().collect();
            buffer.draw_text_overlay(
                &[&line_chars.iter().collect::<String>()],
                x,
                self.height.saturating_sub(2),
                14,
                Some(234),
            );
        }

        if let Some((text, x)) = paused_line {
            let text_chars: Vec<char> = text.chars().collect();
            buffer.draw_text_overlay(
                &[&text_chars.iter().collect::<String>()],
                x,
                2,
                15,
                Some(196),
            );
        }

        if let Some((text, x)) = notification_line {
            let text_chars: Vec<char> = text.chars().collect();
            buffer.draw_text_overlay(
                &[&text_chars.iter().collect::<String>()],
                x,
                self.height.saturating_sub(4),
                15,
                Some(22),
            );
        }

        if let Some(lines) = stats_lines {
            buffer.draw_text_overlay(lines, 1, 2, 14, Some(236));
        }

        execute!(self.stdout, &buffer)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn render_multi_species_with_overlay<T: AsRef<str>>(
        &mut self,
        trail_maps: &[(&[f32], RgbColor)],
        sim_width: usize,
        sim_height: usize,
        max_trail_value: f32,
        help_lines: Option<(&[T], usize, usize)>,
        status_line: Option<(String, usize)>,
        paused_line: Option<(String, usize)>,
        notification_line: Option<(String, usize)>,
        stats_lines: Option<&[String]>,
    ) -> io::Result<()> {
        if let Some(ref mut ed) = self.error_diffusion {
            ed.reset();
        }

        let mut buffer = FrameBuffer::new(self.width, self.height, self.color_mode);
        buffer.species_colors_enabled = true;
        buffer.species_rgb_colors = self.species_rgb_colors.clone();

        for (trail_map, species_color) in trail_maps {
            let downsampled = downsample_multi_species(
                &[(trail_map, 0)],
                sim_width,
                sim_height,
                self.width,
                self.height,
            );

            let species_color_vec = vec![*species_color];
            let species_buffer = FrameBuffer::from_downsampled(
                downsampled.cells(),
                self.width,
                self.height,
                max_trail_value,
                self.palette.clone(),
                self.charset,
                self.reverse_palette,
                self.invert_palette,
                self.color_mode,
                self.hue_shift,
                self.dither_mode,
                &mut self.error_diffusion,
                true,
                Some(species_color_vec),
            );

            for (i, cell) in species_buffer.cells.iter().enumerate() {
                if cell.char != ' ' {
                    buffer.cells[i] = *cell;
                }
            }
        }

        if let Some((lines, x, y)) = help_lines {
            buffer.draw_text_overlay(lines, x, y, 15, Some(236));
        }

        if let Some((line, x)) = status_line {
            let line_chars: Vec<char> = line.chars().collect();
            buffer.draw_text_overlay(
                &[&line_chars.iter().collect::<String>()],
                x,
                self.height.saturating_sub(2),
                14,
                Some(234),
            );
        }

        if let Some((text, x)) = paused_line {
            let text_chars: Vec<char> = text.chars().collect();
            buffer.draw_text_overlay(
                &[&text_chars.iter().collect::<String>()],
                x,
                2,
                15,
                Some(196),
            );
        }

        if let Some((text, x)) = notification_line {
            let text_chars: Vec<char> = text.chars().collect();
            buffer.draw_text_overlay(
                &[&text_chars.iter().collect::<String>()],
                x,
                self.height.saturating_sub(4),
                15,
                Some(22),
            );
        }

        if let Some(lines) = stats_lines {
            buffer.draw_text_overlay(lines, 1, 2, 14, Some(236));
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
        let buffer = FrameBuffer::new(80, 24, ColorMode::Bits256);
        assert_eq!(buffer.width(), 80);
        assert_eq!(buffer.height(), 24);
    }

    #[test]
    fn test_frame_buffer_set_cell() {
        let mut buffer = FrameBuffer::new(10, 10, ColorMode::Bits256);
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
        let mut buffer = FrameBuffer::new(10, 10, ColorMode::Bits256);
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
            },
            DownsampleCell {
                top: 5.0,
                bottom: 2.0,
            },
            DownsampleCell {
                top: 3.0,
                bottom: 7.0,
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
        let mut buffer = FrameBuffer::new(10, 10, ColorMode::Bits256);
        let cell = buffer.create_cell(
            0,
            0,
            0.0,
            0.0,
            &[],
            &Palette::Organic,
            Charset::HalfBlock,
            false,
            false,
            ColorMode::Bits256,
            0.0,
            DitherMode::None,
            &mut None,
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
        let mut buffer = FrameBuffer::new(10, 10, ColorMode::Bits256);
        let cell = buffer.create_cell(
            0,
            0,
            1.0,
            1.0,
            &[],
            &Palette::Organic,
            Charset::HalfBlock,
            false,
            false,
            ColorMode::Bits256,
            0.0,
            DitherMode::None,
            &mut None,
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
        let mut buffer = FrameBuffer::new(10, 10, ColorMode::TrueColor);
        let cell = buffer.create_cell(
            0,
            0,
            1.0,
            1.0,
            &[],
            &Palette::Organic,
            Charset::HalfBlock,
            false,
            false,
            ColorMode::TrueColor,
            0.0,
            DitherMode::None,
            &mut None,
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
        let mut buffer = FrameBuffer::new(10, 10, ColorMode::Bits256);
        let cell = buffer.create_cell(
            0,
            0,
            1.0,
            0.0,
            &[],
            &Palette::Organic,
            Charset::HalfBlock,
            false,
            false,
            ColorMode::Bits256,
            0.0,
            DitherMode::None,
            &mut None,
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
        let mut buffer = FrameBuffer::new(10, 10, ColorMode::Bits256);
        let cell = buffer.create_cell(
            0,
            0,
            0.0,
            1.0,
            &[],
            &Palette::Organic,
            Charset::HalfBlock,
            false,
            false,
            ColorMode::Bits256,
            0.0,
            DitherMode::None,
            &mut None,
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
        let mut buffer = FrameBuffer::new(10, 10, ColorMode::Bits256);
        let cell = buffer.create_cell(
            0,
            0,
            0.5,
            0.0,
            &[],
            &Palette::Organic,
            Charset::HalfBlock,
            false,
            false,
            ColorMode::Bits256,
            0.0,
            DitherMode::None,
            &mut None,
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
        let mut buffer = FrameBuffer::new(10, 10, ColorMode::Bits256);
        let cell = buffer.create_cell(
            0,
            0,
            0.0,
            1.0,
            &[],
            &Palette::Organic,
            Charset::HalfBlock,
            false,
            false,
            ColorMode::Bits256,
            0.0,
            DitherMode::None,
            &mut None,
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
        let mut buffer = FrameBuffer::new(10, 10, ColorMode::Bits256);
        let cell = buffer.create_cell(
            0,
            0,
            1.0,
            0.0,
            &[],
            &Palette::Organic,
            Charset::Braille,
            false,
            false,
            ColorMode::Bits256,
            0.0,
            DitherMode::None,
            &mut None,
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
        let mut buffer = FrameBuffer::new(10, 10, ColorMode::Bits256);
        let cell = buffer.create_cell(
            0,
            0,
            0.0,
            1.0,
            &[],
            &Palette::Organic,
            Charset::Braille,
            false,
            false,
            ColorMode::Bits256,
            0.0,
            DitherMode::None,
            &mut None,
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
        let mut buffer = FrameBuffer::new(10, 10, ColorMode::Bits256);
        let cell = buffer.create_cell(
            0,
            0,
            0.5,
            0.0,
            &[],
            &Palette::Organic,
            Charset::Braille,
            false,
            false,
            ColorMode::Bits256,
            0.0,
            DitherMode::None,
            &mut None,
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
        let mut buffer = FrameBuffer::new(10, 10, ColorMode::Bits256);
        let cell = buffer.create_cell(
            0,
            0,
            0.0,
            0.5,
            &[],
            &Palette::Organic,
            Charset::Braille,
            false,
            false,
            ColorMode::Bits256,
            0.0,
            DitherMode::None,
            &mut None,
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
        let mut buffer = FrameBuffer::new(10, 10, ColorMode::Bits256);
        let cell = buffer.create_cell(
            0,
            0,
            1.0,
            0.0,
            &[],
            &Palette::Organic,
            Charset::Ascii,
            false,
            false,
            ColorMode::Bits256,
            0.0,
            DitherMode::None,
            &mut None,
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
        let mut buffer = FrameBuffer::new(10, 10, ColorMode::Bits256);
        let cell = buffer.create_cell(
            0,
            0,
            0.0,
            1.0,
            &[],
            &Palette::Organic,
            Charset::Ascii,
            false,
            false,
            ColorMode::Bits256,
            0.0,
            DitherMode::None,
            &mut None,
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
        let mut buffer = FrameBuffer::new(10, 10, ColorMode::Bits256);
        let cell = buffer.create_cell(
            0,
            0,
            0.5,
            0.0,
            &[],
            &Palette::Organic,
            Charset::Ascii,
            false,
            false,
            ColorMode::Bits256,
            0.0,
            DitherMode::None,
            &mut None,
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
        let mut buffer = FrameBuffer::new(10, 10, ColorMode::Bits256);
        let cell = buffer.create_cell(
            0,
            0,
            0.0,
            0.5,
            &[],
            &Palette::Organic,
            Charset::Ascii,
            false,
            false,
            ColorMode::Bits256,
            0.0,
            DitherMode::None,
            &mut None,
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
    fn test_truecolor_code_fg() {
        let code = FrameBuffer::truecolor_code(42, 128, 255, true);
        assert_eq!(code, "\x1b[38;2;42;128;255m");
    }

    #[test]
    fn test_truecolor_code_bg() {
        let code = FrameBuffer::truecolor_code(42, 128, 255, false);
        assert_eq!(code, "\x1b[48;2;42;128;255m");
    }

    #[test]
    fn test_build_frame_string_cursor_home() {
        let buffer = FrameBuffer::new(5, 3, ColorMode::Bits256);
        let frame_str = buffer.build_frame_string(false, ColorMode::Bits256);
        assert!(frame_str.starts_with("\x1b[H"));
    }

    #[test]
    fn test_build_frame_string_plain_output() {
        let buffer = FrameBuffer::new(5, 3, ColorMode::Bits256);
        let frame_str = buffer.build_frame_string(true, ColorMode::Bits256);
        assert!(!frame_str.contains("\x1b"));
    }

    #[test]
    fn test_build_frame_string_truecolor() {
        let mut buffer = FrameBuffer::new(5, 3, ColorMode::TrueColor);
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
                bottom: 0.0
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
            false,
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
            },
            DownsampleCell {
                top: 0.0,
                bottom: 5.0,
            },
            DownsampleCell {
                top: 5.0,
                bottom: 5.0,
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
            false,
            None,
        );

        assert_eq!(buffer.cells[0].char, '▀');
        assert_eq!(buffer.cells[1].char, '▄');
        assert_eq!(buffer.cells[2].char, '█');
    }

    #[test]
    fn test_terminal_renderer_creation() {
        let renderer = TerminalRenderer::new(
            80,
            24,
            Palette::Organic,
            Charset::HalfBlock,
            false,
            false,
            ColorMode::Bits256,
        );
        assert_eq!(renderer.width, 80);
        assert_eq!(renderer.height, 24);
    }

    #[test]
    fn test_terminal_renderer_creation_truecolor() {
        let renderer = TerminalRenderer::new(
            80,
            24,
            Palette::Organic,
            Charset::HalfBlock,
            false,
            false,
            ColorMode::TrueColor,
        );
        assert_eq!(renderer.width, 80);
        assert_eq!(renderer.height, 24);
    }

    #[test]
    fn test_terminal_renderer_set_dimensions() {
        let mut renderer = TerminalRenderer::new(
            80,
            24,
            Palette::Organic,
            Charset::HalfBlock,
            false,
            false,
            ColorMode::Bits256,
        );
        renderer.set_dimensions(100, 30);
        assert_eq!(renderer.width, 100);
        assert_eq!(renderer.height, 30);
    }

    #[test]
    fn test_terminal_renderer_set_palette() {
        let mut renderer = TerminalRenderer::new(
            80,
            24,
            Palette::Organic,
            Charset::HalfBlock,
            false,
            false,
            ColorMode::Bits256,
        );
        renderer.set_palette(Palette::Heat);
        assert_eq!(renderer.palette, Palette::Heat);
    }

    #[test]
    fn test_terminal_renderer_set_charset() {
        let mut renderer = TerminalRenderer::new(
            80,
            24,
            Palette::Organic,
            Charset::HalfBlock,
            false,
            false,
            ColorMode::Bits256,
        );
        renderer.set_charset(Charset::Ascii);
        assert_eq!(renderer.charset, Charset::Ascii);
    }

    #[test]
    fn test_downsample_multi_species() {
        let trail1 = vec![10.0; 100];
        let trail2 = vec![5.0; 100];

        let result = downsample_multi_species(&[(&trail1, 0), (&trail2, 1)], 10, 10, 5, 5);

        for cell in result.cells() {
            assert!(
                (cell.top - 15.0).abs() < 0.001,
                "Expected 15.0, got {}",
                cell.top
            );
            assert!(
                (cell.bottom - 15.0).abs() < 0.001,
                "Expected 15.0, got {}",
                cell.bottom
            );
        }
    }

    #[test]
    fn test_downsample_multi_species_empty() {
        let trail1 = vec![0.0; 100];
        let trail2 = vec![0.0; 100];

        let result = downsample_multi_species(&[(&trail1, 0), (&trail2, 1)], 10, 10, 5, 5);

        for cell in result.cells() {
            assert_eq!(cell.top, 0.0);
            assert_eq!(cell.bottom, 0.0);
        }
    }
}
