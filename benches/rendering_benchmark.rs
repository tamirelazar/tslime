use criterion::{black_box, criterion_group, criterion_main, Criterion};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RgbColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub enum Palette {
    Organic,
    Heat,
    Ocean,
    Mono,
    Forest,
    Neon,
    Warm,
    Vibrant,
    LegibleMono,
}

#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub enum ColorMode {
    TrueColor,
    Bits8,
    Bits16,
    Bits256,
}

const ORGANIC_RGB: [RgbColor; 11] = [
    RgbColor {
        r: 18,
        g: 18,
        b: 18,
    },
    RgbColor {
        r: 40,
        g: 40,
        b: 40,
    },
    RgbColor {
        r: 70,
        g: 20,
        b: 20,
    },
    RgbColor {
        r: 100,
        g: 40,
        b: 40,
    },
    RgbColor {
        r: 130,
        g: 50,
        b: 40,
    },
    RgbColor {
        r: 160,
        g: 50,
        b: 50,
    },
    RgbColor {
        r: 120,
        g: 100,
        b: 50,
    },
    RgbColor {
        r: 100,
        g: 130,
        b: 60,
    },
    RgbColor {
        r: 80,
        g: 160,
        b: 80,
    },
    RgbColor {
        r: 100,
        g: 190,
        b: 130,
    },
    RgbColor {
        r: 150,
        g: 220,
        b: 200,
    },
];

const OCEAN_RGB: [RgbColor; 11] = [
    RgbColor {
        r: 18,
        g: 18,
        b: 18,
    },
    RgbColor {
        r: 20,
        g: 20,
        b: 50,
    },
    RgbColor {
        r: 20,
        g: 25,
        b: 60,
    },
    RgbColor {
        r: 20,
        g: 30,
        b: 70,
    },
    RgbColor {
        r: 20,
        g: 40,
        b: 80,
    },
    RgbColor {
        r: 25,
        g: 50,
        b: 100,
    },
    RgbColor {
        r: 30,
        g: 70,
        b: 130,
    },
    RgbColor {
        r: 40,
        g: 90,
        b: 160,
    },
    RgbColor {
        r: 50,
        g: 110,
        b: 190,
    },
    RgbColor {
        r: 60,
        g: 140,
        b: 220,
    },
    RgbColor {
        r: 80,
        g: 170,
        b: 240,
    },
];

fn get_rgb_gradient(palette: Palette) -> &'static [RgbColor; 11] {
    match palette {
        Palette::Organic => &ORGANIC_RGB,
        Palette::Heat => &ORGANIC_RGB,
        Palette::Ocean => &OCEAN_RGB,
        Palette::Mono => &ORGANIC_RGB,
        Palette::Forest => &ORGANIC_RGB,
        Palette::Neon => &ORGANIC_RGB,
        Palette::Warm => &ORGANIC_RGB,
        Palette::Vibrant => &ORGANIC_RGB,
        Palette::LegibleMono => &ORGANIC_RGB,
    }
}

pub fn map_brightness_rgb(
    brightness: f32,
    palette: Palette,
    _reverse: bool,
    _invert: bool,
) -> RgbColor {
    let brightness = brightness.clamp(0.0, 1.0);
    let gradient = get_rgb_gradient(palette);

    let position = brightness * (gradient.len() - 1) as f32;
    let lower = position.floor() as usize;
    let upper = position.ceil() as usize;
    let fraction = position - lower as f32;

    let lower_color = gradient[lower];
    let upper_color = gradient[upper];

    let r = if upper == lower {
        lower_color.r
    } else {
        (lower_color.r as f32 + (upper_color.r as f32 - lower_color.r as f32) * fraction) as u8
    };
    let g = if upper == lower {
        lower_color.g
    } else {
        (lower_color.g as f32 + (upper_color.g as f32 - lower_color.g as f32) * fraction) as u8
    };
    let b = if upper == lower {
        lower_color.b
    } else {
        (lower_color.b as f32 + (upper_color.b as f32 - lower_color.b as f32) * fraction) as u8
    };

    RgbColor { r, g, b }
}

pub fn truecolor_ansi_fg(r: u8, g: u8, b: u8) -> String {
    format!("\x1b[38;2;{};{};{}m", r, g, b)
}

pub fn truecolor_ansi_bg(r: u8, g: u8, b: u8) -> String {
    format!("\x1b[48;2;{};{};{}m", r, g, b)
}

#[derive(Clone)]
struct DownsampleCell {
    top: f32,
    bottom: f32,
}

#[derive(Clone)]
struct Cell {
    char: char,
    fg_color_rgb: Option<RgbColor>,
}

struct FrameBuffer {
    width: usize,
    height: usize,
    cells: Vec<Cell>,
    color_mode: ColorMode,
}

impl FrameBuffer {
    fn new(width: usize, height: usize, color_mode: ColorMode) -> Self {
        Self {
            width,
            height,
            cells: vec![
                Cell {
                    char: ' ',
                    fg_color_rgb: None,
                };
                width * height
            ],
            color_mode,
        }
    }

    fn create_cell(&self, top: f32, bottom: f32, palette: Palette) -> Cell {
        const THRESHOLD: f32 = 0.05;
        let brightness = if top > bottom { top } else { bottom };

        if top < THRESHOLD && bottom < THRESHOLD {
            Cell {
                char: ' ',
                fg_color_rgb: None,
            }
        } else {
            let char = if brightness > 0.5 { '█' } else { '▀' };
            let rgb = match self.color_mode {
                ColorMode::TrueColor => Some(map_brightness_rgb(brightness, palette, false, false)),
                _ => None,
            };
            Cell {
                char,
                fg_color_rgb: rgb,
            }
        }
    }

    fn from_downsampled(
        downsampled: &[DownsampleCell],
        width: usize,
        height: usize,
        palette: Palette,
        color_mode: ColorMode,
    ) -> Self {
        let mut buffer = Self::new(width, height, color_mode);

        for (idx, dcell) in downsampled.iter().enumerate() {
            if idx >= width * height {
                break;
            }

            let cell = buffer.create_cell(dcell.top, dcell.bottom, palette);
            buffer.cells[idx] = cell;
        }

        buffer
    }

    fn build_frame_string(&self, _plain_output: bool) -> String {
        let mut output = String::new();

        for y in 0..self.height {
            for x in 0..self.width {
                let cell = &self.cells[y * self.width + x];

                if let Some(fg) = cell.fg_color_rgb {
                    output.push_str(&truecolor_ansi_fg(fg.r, fg.g, fg.b));
                }

                output.push(cell.char);
            }
        }

        output
    }
}

fn generate_downsampled_cells(width: usize, height: usize) -> Vec<DownsampleCell> {
    let mut cells = Vec::with_capacity(width * height);
    for y in 0..height {
        for x in 0..width {
            let t = ((x + y) as f32 / (width + height) as f32).min(1.0);
            let b = ((x + height - y) as f32 / (width + height) as f32).min(1.0);
            cells.push(DownsampleCell {
                top: t * 20.0,
                bottom: b * 20.0,
            });
        }
    }
    cells
}

fn bench_framebuffer_256color(c: &mut Criterion) {
    let width = 80;
    let height = 24;
    let downsampled = generate_downsampled_cells(width, height);
    let palette = Palette::Organic;
    let color_mode = ColorMode::Bits256;

    c.bench_function("framebuffer_256color_80x24", |b| {
        b.iter(|| {
            let buffer = FrameBuffer::from_downsampled(
                black_box(&downsampled),
                black_box(width),
                black_box(height),
                black_box(palette),
                black_box(color_mode),
            );
            black_box(buffer.build_frame_string(true));
        });
    });
}

fn bench_framebuffer_truecolor(c: &mut Criterion) {
    let width = 80;
    let height = 24;
    let downsampled = generate_downsampled_cells(width, height);
    let palette = Palette::Organic;
    let color_mode = ColorMode::TrueColor;

    c.bench_function("framebuffer_truecolor_80x24", |b| {
        b.iter(|| {
            let buffer = FrameBuffer::from_downsampled(
                black_box(&downsampled),
                black_box(width),
                black_box(height),
                black_box(palette),
                black_box(color_mode),
            );
            black_box(buffer.build_frame_string(true));
        });
    });
}

fn bench_framebuffer_large_256color(c: &mut Criterion) {
    let width = 200;
    let height = 60;
    let downsampled = generate_downsampled_cells(width, height);
    let palette = Palette::Ocean;
    let color_mode = ColorMode::Bits256;

    c.bench_function("framebuffer_256color_200x60", |b| {
        b.iter(|| {
            let buffer = FrameBuffer::from_downsampled(
                black_box(&downsampled),
                black_box(width),
                black_box(height),
                black_box(palette),
                black_box(color_mode),
            );
            black_box(buffer.build_frame_string(true));
        });
    });
}

fn bench_framebuffer_large_truecolor(c: &mut Criterion) {
    let width = 200;
    let height = 60;
    let downsampled = generate_downsampled_cells(width, height);
    let palette = Palette::Ocean;
    let color_mode = ColorMode::TrueColor;

    c.bench_function("framebuffer_truecolor_200x60", |b| {
        b.iter(|| {
            let buffer = FrameBuffer::from_downsampled(
                black_box(&downsampled),
                black_box(width),
                black_box(height),
                black_box(palette),
                black_box(color_mode),
            );
            black_box(buffer.build_frame_string(true));
        });
    });
}

fn bench_palette_rgb_mapping(c: &mut Criterion) {
    let palette = Palette::Ocean;

    c.bench_function("palette_rgb_mapping", |b| {
        b.iter(|| {
            for brightness in (0..=100).map(|i| i as f32 / 100.0) {
                black_box(map_brightness_rgb(
                    black_box(brightness),
                    black_box(palette),
                    black_box(false),
                    black_box(false),
                ));
            }
        });
    });
}

fn bench_truecolor_ansi_generation(c: &mut Criterion) {
    c.bench_function("truecolor_ansi_fg_generation", |b| {
        b.iter(|| {
            for i in 0..=255 {
                black_box(truecolor_ansi_fg(i, i / 2, i / 4));
            }
        });
    });
}

fn bench_color_mode_comparison(c: &mut Criterion) {
    let mut group = c.benchmark_group("color_mode_render");

    let width = 120;
    let height = 40;
    let downsampled = generate_downsampled_cells(width, height);
    let palette = Palette::Ocean;

    group.bench_function("256color", |b| {
        b.iter(|| {
            let buffer = FrameBuffer::from_downsampled(
                black_box(&downsampled),
                black_box(width),
                black_box(height),
                black_box(palette),
                black_box(ColorMode::Bits256),
            );
            black_box(buffer);
        });
    });

    group.bench_function("truecolor", |b| {
        b.iter(|| {
            let buffer = FrameBuffer::from_downsampled(
                black_box(&downsampled),
                black_box(width),
                black_box(height),
                black_box(palette),
                black_box(ColorMode::TrueColor),
            );
            black_box(buffer);
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_framebuffer_256color,
    bench_framebuffer_truecolor,
    bench_framebuffer_large_256color,
    bench_framebuffer_large_truecolor,
    bench_palette_rgb_mapping,
    bench_truecolor_ansi_generation,
    bench_color_mode_comparison
);
criterion_main!(benches);
