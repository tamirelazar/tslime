use crate::cli::Palette;
use crate::render::charset::{self, Charset};
use crate::render::downsample::Cell as DownsampleCell;
use crate::render::palette;
use crate::render::palette::RgbColor;
use crate::cli::ColorMode;
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
        }
    }

    fn set_cell(&mut self, x: usize, y: usize, cell: Cell) {
        if x < self.width && y < self.height {
            self.cells[y * self.width + x] = cell;
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
    ) -> Self {
        let mut buffer = Self::new(width, height, color_mode);

        for (idx, dcell) in downsampled.iter().enumerate() {
            if idx >= width * height {
                break;
            }

            let x = idx % width;
            let y = idx / width;

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
                top_brightness,
                bottom_brightness,
                &palette,
                charset,
                reverse_palette,
                invert_palette,
                color_mode,
            );
            buffer.set_cell(x, y, cell);
        }

        buffer
    }

    #[allow(dead_code)]
    #[allow(clippy::too_many_arguments)]
    fn create_cell(
        &self,
        top: f32,
        bottom: f32,
        palette: &Palette,
        charset: Charset,
        reverse_palette: bool,
        invert_palette: bool,
        color_mode: ColorMode,
    ) -> Cell {
        const THRESHOLD: f32 = 0.05;

        if top < THRESHOLD && bottom < THRESHOLD {
            Cell {
                char: ' ',
                fg_color_256: None,
                bg_color_256: None,
                fg_color_rgb: None,
                bg_color_rgb: None,
            }
        } else if top > THRESHOLD && bottom > THRESHOLD {
            let char = match charset {
                Charset::HalfBlock => charset::map_vertical_block(top, bottom),
                _ => charset::map_brightness((top + bottom) / 2.0, charset),
            };
            let brightness = (top + bottom) / 2.0;
            match color_mode {
                ColorMode::TrueColor => {
                    let rgb = palette::map_brightness_rgb(
                        brightness,
                        palette.clone(),
                        reverse_palette,
                        invert_palette,
                    );
                    Cell {
                        char,
                        fg_color_256: None,
                        bg_color_256: None,
                        fg_color_rgb: Some(rgb),
                        bg_color_rgb: None,
                    }
                }
                _ => {
                    let color = palette::map_brightness(
                        brightness,
                        palette.clone(),
                        reverse_palette,
                        invert_palette,
                    );
                    Cell {
                        char,
                        fg_color_256: Some(color),
                        bg_color_256: None,
                        fg_color_rgb: None,
                        bg_color_rgb: None,
                    }
                }
            }
        } else if top > bottom {
            let brightness = top;
            let char = match charset {
                Charset::Braille => charset::map_brightness(brightness, charset),
                Charset::HalfBlock => charset::map_vertical_block(top, bottom),
                Charset::Ascii => charset::map_ascii_directional(brightness, true),
            };
            match color_mode {
                ColorMode::TrueColor => {
                    let rgb = palette::map_brightness_rgb(
                        brightness,
                        palette.clone(),
                        reverse_palette,
                        invert_palette,
                    );
                    Cell {
                        char,
                        fg_color_256: None,
                        bg_color_256: None,
                        fg_color_rgb: Some(rgb),
                        bg_color_rgb: None,
                    }
                }
                _ => {
                    let color = palette::map_brightness(
                        brightness,
                        palette.clone(),
                        reverse_palette,
                        invert_palette,
                    );
                    Cell {
                        char,
                        fg_color_256: Some(color),
                        bg_color_256: None,
                        fg_color_rgb: None,
                        bg_color_rgb: None,
                    }
                }
            }
        } else {
            let brightness = bottom;
            let char = match charset {
                Charset::Braille => charset::map_brightness(brightness, charset),
                Charset::HalfBlock => charset::map_vertical_block(top, bottom),
                Charset::Ascii => charset::map_ascii_directional(brightness, false),
            };
            match color_mode {
                ColorMode::TrueColor => {
                    let rgb = palette::map_brightness_rgb(
                        brightness,
                        palette.clone(),
                        reverse_palette,
                        invert_palette,
                    );
                    Cell {
                        char,
                        fg_color_256: None,
                        bg_color_256: None,
                        fg_color_rgb: Some(rgb),
                        bg_color_rgb: None,
                    }
                }
                _ => {
                    let color = palette::map_brightness(
                        brightness,
                        palette.clone(),
                        reverse_palette,
                        invert_palette,
                    );
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

        if !plain_output && (last_fg_256.is_some() || last_bg_256.is_some() || last_fg_rgb.is_some() || last_bg_rgb.is_some()) {
            output.push_str("\x1b[0m");
        }

        output
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
) -> io::Result<()> {
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
    pub fn set_charset(&mut self, charset: Charset) {
        self.charset = charset;
    }

    pub fn render(
        &mut self,
        downsampled: &[DownsampleCell],
        max_trail_value: f32,
    ) -> io::Result<()> {
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
        );

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
        let buffer = FrameBuffer::new(10, 10, ColorMode::Bits256);
        let cell = buffer.create_cell(
            0.0,
            0.0,
            &Palette::Organic,
            Charset::HalfBlock,
            false,
            false,
            ColorMode::Bits256,
        );
        assert_eq!(cell.char, ' ');
        assert!(cell.fg_color_256.is_none());
        assert!(cell.bg_color_256.is_none());
        assert!(cell.fg_color_rgb.is_none());
        assert!(cell.bg_color_rgb.is_none());
    }

    #[test]
    fn test_create_cell_full() {
        let buffer = FrameBuffer::new(10, 10, ColorMode::Bits256);
        let cell = buffer.create_cell(
            1.0,
            1.0,
            &Palette::Organic,
            Charset::HalfBlock,
            false,
            false,
            ColorMode::Bits256,
        );
        assert_eq!(cell.char, '\u{2588}');
        assert!(cell.fg_color_256.is_some());
        assert!(cell.bg_color_256.is_none());
        assert!(cell.fg_color_rgb.is_none());
        assert!(cell.bg_color_rgb.is_none());
    }

    #[test]
    fn test_create_cell_full_truecolor() {
        let buffer = FrameBuffer::new(10, 10, ColorMode::TrueColor);
        let cell = buffer.create_cell(
            1.0,
            1.0,
            &Palette::Organic,
            Charset::HalfBlock,
            false,
            false,
            ColorMode::TrueColor,
        );
        assert_eq!(cell.char, '\u{2588}');
        assert!(cell.fg_color_256.is_none());
        assert!(cell.bg_color_256.is_none());
        assert!(cell.fg_color_rgb.is_some());
        assert!(cell.bg_color_rgb.is_none());
    }

    #[test]
    fn test_create_cell_halfblock_top_only_uses_half_height() {
        let buffer = FrameBuffer::new(10, 10, ColorMode::Bits256);
        let cell = buffer.create_cell(
            1.0,
            0.0,
            &Palette::Organic,
            Charset::HalfBlock,
            false,
            false,
            ColorMode::Bits256,
        );
        assert_eq!(cell.char, '▀');
        assert!(cell.fg_color_256.is_some());
        assert!(cell.bg_color_256.is_none());
    }

    #[test]
    fn test_create_cell_halfblock_bottom_only_uses_half_height() {
        let buffer = FrameBuffer::new(10, 10, ColorMode::Bits256);
        let cell = buffer.create_cell(
            0.0,
            1.0,
            &Palette::Organic,
            Charset::HalfBlock,
            false,
            false,
            ColorMode::Bits256,
        );
        assert_eq!(cell.char, '▄');
        assert!(cell.fg_color_256.is_some());
        assert!(cell.bg_color_256.is_none());
    }

    #[test]
    fn test_create_cell_halfblock_top_half_brightness() {
        let buffer = FrameBuffer::new(10, 10, ColorMode::Bits256);
        let cell = buffer.create_cell(
            0.5,
            0.0,
            &Palette::Organic,
            Charset::HalfBlock,
            false,
            false,
            ColorMode::Bits256,
        );
        assert_eq!(cell.char, '▀');
        assert!(cell.fg_color_256.is_some());
        assert!(cell.bg_color_256.is_none());
    }

    #[test]
    fn test_create_cell_bottom_only() {
        let buffer = FrameBuffer::new(10, 10, ColorMode::Bits256);
        let cell = buffer.create_cell(
            0.0,
            1.0,
            &Palette::Organic,
            Charset::HalfBlock,
            false,
            false,
            ColorMode::Bits256,
        );
        assert_eq!(cell.char, '▄');
        assert!(cell.fg_color_256.is_some());
        assert!(cell.bg_color_256.is_none());
    }

    #[test]
    fn test_create_cell_braille_top_only() {
        let buffer = FrameBuffer::new(10, 10, ColorMode::Bits256);
        let cell = buffer.create_cell(1.0, 0.0, &Palette::Organic, Charset::Braille, false, false, ColorMode::Bits256);
        assert_eq!(cell.char, '\u{287B}');
        assert!(cell.fg_color_256.is_some());
        assert!(cell.bg_color_256.is_none());
    }

    #[test]
    fn test_create_cell_braille_bottom_only() {
        let buffer = FrameBuffer::new(10, 10, ColorMode::Bits256);
        let cell = buffer.create_cell(0.0, 1.0, &Palette::Organic, Charset::Braille, false, false, ColorMode::Bits256);
        assert_eq!(cell.char, '\u{287B}');
        assert!(cell.fg_color_256.is_some());
        assert!(cell.bg_color_256.is_none());
    }

    #[test]
    fn test_create_cell_braille_top_half_brightness() {
        let buffer = FrameBuffer::new(10, 10, ColorMode::Bits256);
        let cell = buffer.create_cell(0.5, 0.0, &Palette::Organic, Charset::Braille, false, false, ColorMode::Bits256);
        assert!(cell.char >= '\u{2800}' && cell.char <= '\u{28FF}');
        assert!(cell.fg_color_256.is_some());
        assert!(cell.bg_color_256.is_none());
    }

    #[test]
    fn test_create_cell_braille_bottom_half_brightness() {
        let buffer = FrameBuffer::new(10, 10, ColorMode::Bits256);
        let cell = buffer.create_cell(0.0, 0.5, &Palette::Organic, Charset::Braille, false, false, ColorMode::Bits256);
        assert!(cell.char >= '\u{2800}' && cell.char <= '\u{28FF}');
        assert!(cell.fg_color_256.is_some());
        assert!(cell.bg_color_256.is_none());
    }

    #[test]
    fn test_create_cell_ascii_top_only() {
        let buffer = FrameBuffer::new(10, 10, ColorMode::Bits256);
        let cell = buffer.create_cell(1.0, 0.0, &Palette::Organic, Charset::Ascii, false, false, ColorMode::Bits256);
        assert_eq!(cell.char, '^');
        assert!(cell.fg_color_256.is_some());
        assert!(cell.bg_color_256.is_none());
    }

    #[test]
    fn test_create_cell_ascii_bottom_only() {
        let buffer = FrameBuffer::new(10, 10, ColorMode::Bits256);
        let cell = buffer.create_cell(0.0, 1.0, &Palette::Organic, Charset::Ascii, false, false, ColorMode::Bits256);
        assert_eq!(cell.char, 'v');
        assert!(cell.fg_color_256.is_some());
        assert!(cell.bg_color_256.is_none());
    }

    #[test]
    fn test_create_cell_ascii_top_half_brightness() {
        let buffer = FrameBuffer::new(10, 10, ColorMode::Bits256);
        let cell = buffer.create_cell(0.5, 0.0, &Palette::Organic, Charset::Ascii, false, false, ColorMode::Bits256);
        assert_eq!(cell.char, '=');
        assert!(cell.fg_color_256.is_some());
        assert!(cell.bg_color_256.is_none());
    }

    #[test]
    fn test_create_cell_ascii_bottom_half_brightness() {
        let buffer = FrameBuffer::new(10, 10, ColorMode::Bits256);
        let cell = buffer.create_cell(0.0, 0.5, &Palette::Organic, Charset::Ascii, false, false, ColorMode::Bits256);
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
            fg_color_rgb: Some(RgbColor { r: 255, g: 128, b: 64 }),
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
        );

        assert_eq!(buffer.cells[0].char, '▀');
        assert_eq!(buffer.cells[1].char, '▄');
        assert_eq!(buffer.cells[2].char, '█');
    }

    #[test]
    fn test_terminal_renderer_creation() {
        let renderer =
            TerminalRenderer::new(80, 24, Palette::Organic, Charset::HalfBlock, false, false, ColorMode::Bits256);
        assert_eq!(renderer.width, 80);
        assert_eq!(renderer.height, 24);
    }

    #[test]
    fn test_terminal_renderer_creation_truecolor() {
        let renderer =
            TerminalRenderer::new(80, 24, Palette::Organic, Charset::HalfBlock, false, false, ColorMode::TrueColor);
        assert_eq!(renderer.width, 80);
        assert_eq!(renderer.height, 24);
    }

    #[test]
    fn test_terminal_renderer_set_dimensions() {
        let mut renderer =
            TerminalRenderer::new(80, 24, Palette::Organic, Charset::HalfBlock, false, false, ColorMode::Bits256);
        renderer.set_dimensions(100, 30);
        assert_eq!(renderer.width, 100);
        assert_eq!(renderer.height, 30);
    }

    #[test]
    fn test_terminal_renderer_set_palette() {
        let mut renderer =
            TerminalRenderer::new(80, 24, Palette::Organic, Charset::HalfBlock, false, false, ColorMode::Bits256);
        renderer.set_palette(Palette::Heat);
        assert_eq!(renderer.palette, Palette::Heat);
    }

    #[test]
    fn test_terminal_renderer_set_charset() {
        let mut renderer =
            TerminalRenderer::new(80, 24, Palette::Organic, Charset::HalfBlock, false, false, ColorMode::Bits256);
        renderer.set_charset(Charset::Ascii);
        assert_eq!(renderer.charset, Charset::Ascii);
    }
}
