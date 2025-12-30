use crate::cli::Palette;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RgbColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

const ORGANIC_GRADIENT: [u8; 11] = [232, 22, 28, 34, 40, 46, 82, 118, 154, 190, 226];

const ORGANIC_RGB: [RgbColor; 11] = [
    RgbColor { r: 18, g: 18, b: 18 },
    RgbColor { r: 40, g: 40, b: 40 },
    RgbColor { r: 70, g: 20, b: 20 },
    RgbColor { r: 100, g: 40, b: 40 },
    RgbColor { r: 130, g: 50, b: 40 },
    RgbColor { r: 160, g: 50, b: 50 },
    RgbColor { r: 120, g: 100, b: 50 },
    RgbColor { r: 100, g: 130, b: 60 },
    RgbColor { r: 80, g: 160, b: 80 },
    RgbColor { r: 100, g: 190, b: 130 },
    RgbColor { r: 150, g: 220, b: 200 },
];

const HEAT_GRADIENT: [u8; 11] = [232, 52, 88, 124, 160, 196, 202, 208, 214, 220, 226];

const HEAT_RGB: [RgbColor; 11] = [
    RgbColor { r: 40, g: 20, b: 20 },
    RgbColor { r: 40, g: 20, b: 20 },
    RgbColor { r: 70, g: 20, b: 20 },
    RgbColor { r: 110, g: 20, b: 20 },
    RgbColor { r: 150, g: 20, b: 20 },
    RgbColor { r: 190, g: 40, b: 30 },
    RgbColor { r: 200, g: 70, b: 40 },
    RgbColor { r: 210, g: 100, b: 50 },
    RgbColor { r: 220, g: 140, b: 60 },
    RgbColor { r: 230, g: 180, b: 80 },
    RgbColor { r: 240, g: 220, b: 180 },
];

const OCEAN_GRADIENT: [u8; 11] = [232, 17, 18, 19, 20, 21, 27, 33, 39, 45, 51];

const OCEAN_RGB: [RgbColor; 11] = [
    RgbColor { r: 18, g: 18, b: 18 },
    RgbColor { r: 20, g: 20, b: 50 },
    RgbColor { r: 20, g: 25, b: 60 },
    RgbColor { r: 20, g: 30, b: 70 },
    RgbColor { r: 20, g: 40, b: 80 },
    RgbColor { r: 25, g: 50, b: 100 },
    RgbColor { r: 30, g: 70, b: 130 },
    RgbColor { r: 40, g: 90, b: 160 },
    RgbColor { r: 50, g: 110, b: 190 },
    RgbColor { r: 60, g: 140, b: 220 },
    RgbColor { r: 80, g: 170, b: 240 },
];

const MONO_GRADIENT: [u8; 11] = [232, 234, 236, 238, 240, 242, 244, 246, 248, 250, 252];

const MONO_RGB: [RgbColor; 11] = [
    RgbColor { r: 18, g: 18, b: 18 },
    RgbColor { r: 35, g: 35, b: 35 },
    RgbColor { r: 55, g: 55, b: 55 },
    RgbColor { r: 75, g: 75, b: 75 },
    RgbColor { r: 95, g: 95, b: 95 },
    RgbColor { r: 115, g: 115, b: 115 },
    RgbColor { r: 135, g: 135, b: 135 },
    RgbColor { r: 155, g: 155, b: 155 },
    RgbColor { r: 175, g: 175, b: 175 },
    RgbColor { r: 195, g: 195, b: 195 },
    RgbColor { r: 215, g: 215, b: 215 },
];

const FOREST_GRADIENT: [u8; 11] = [22, 22, 34, 34, 40, 40, 118, 118, 154, 118, 40];

const FOREST_RGB: [RgbColor; 11] = [
    RgbColor { r: 20, g: 40, b: 20 },
    RgbColor { r: 30, g: 50, b: 25 },
    RgbColor { r: 40, g: 60, b: 30 },
    RgbColor { r: 50, g: 80, b: 35 },
    RgbColor { r: 60, g: 100, b: 40 },
    RgbColor { r: 70, g: 120, b: 50 },
    RgbColor { r: 80, g: 140, b: 60 },
    RgbColor { r: 100, g: 160, b: 80 },
    RgbColor { r: 120, g: 180, b: 100 },
    RgbColor { r: 150, g: 200, b: 130 },
    RgbColor { r: 180, g: 220, b: 170 },
];

const NEON_GRADIENT: [u8; 11] = [17, 27, 39, 51, 87, 123, 159, 195, 201, 225, 195];

const NEON_RGB: [RgbColor; 11] = [
    RgbColor { r: 30, g: 0, b: 50 },
    RgbColor { r: 40, g: 10, b: 60 },
    RgbColor { r: 50, g: 20, b: 80 },
    RgbColor { r: 60, g: 40, b: 100 },
    RgbColor { r: 80, g: 70, b: 130 },
    RgbColor { r: 100, g: 100, b: 160 },
    RgbColor { r: 120, g: 130, b: 190 },
    RgbColor { r: 140, g: 160, b: 220 },
    RgbColor { r: 170, g: 190, b: 240 },
    RgbColor { r: 200, g: 220, b: 255 },
    RgbColor { r: 150, g: 60, b: 200 },
];

const WARM_GRADIENT: [u8; 11] = [52, 94, 130, 166, 202, 208, 214, 220, 226, 226, 226];

const WARM_RGB: [RgbColor; 11] = [
    RgbColor { r: 40, g: 20, b: 20 },
    RgbColor { r: 60, g: 30, b: 20 },
    RgbColor { r: 80, g: 40, b: 25 },
    RgbColor { r: 110, g: 55, b: 30 },
    RgbColor { r: 140, g: 70, b: 35 },
    RgbColor { r: 170, g: 90, b: 45 },
    RgbColor { r: 200, g: 110, b: 60 },
    RgbColor { r: 210, g: 140, b: 80 },
    RgbColor { r: 220, g: 170, b: 100 },
    RgbColor { r: 230, g: 200, b: 140 },
    RgbColor { r: 240, g: 230, b: 200 },
];

const VIBRANT_GRADIENT: [u8; 11] = [197, 209, 221, 193, 157, 121, 85, 49, 51, 87, 231];

const VIBRANT_RGB: [RgbColor; 11] = [
    RgbColor { r: 50, g: 20, b: 60 },
    RgbColor { r: 60, g: 40, b: 80 },
    RgbColor { r: 80, g: 60, b: 100 },
    RgbColor { r: 100, g: 80, b: 80 },
    RgbColor { r: 120, g: 100, b: 60 },
    RgbColor { r: 140, g: 120, b: 40 },
    RgbColor { r: 160, g: 140, b: 30 },
    RgbColor { r: 180, g: 160, b: 30 },
    RgbColor { r: 200, g: 150, b: 40 },
    RgbColor { r: 220, g: 140, b: 60 },
    RgbColor { r: 240, g: 130, b: 80 },
];

const LEGIBLEMONO_GRADIENT: [u8; 11] = [236, 240, 244, 248, 250, 251, 252, 253, 254, 255, 255];

const LEGIBLEMONO_RGB: [RgbColor; 11] = [
    RgbColor { r: 30, g: 30, b: 30 },
    RgbColor { r: 50, g: 50, b: 50 },
    RgbColor { r: 70, g: 70, b: 70 },
    RgbColor { r: 90, g: 90, b: 90 },
    RgbColor { r: 110, g: 110, b: 110 },
    RgbColor { r: 130, g: 130, b: 130 },
    RgbColor { r: 150, g: 150, b: 150 },
    RgbColor { r: 170, g: 170, b: 170 },
    RgbColor { r: 190, g: 190, b: 190 },
    RgbColor { r: 210, g: 210, b: 210 },
    RgbColor { r: 230, g: 230, b: 230 },
];

fn get_256_gradient(palette: Palette) -> &'static [u8; 11] {
    match palette {
        Palette::Organic => &ORGANIC_GRADIENT,
        Palette::Heat => &HEAT_GRADIENT,
        Palette::Ocean => &OCEAN_GRADIENT,
        Palette::Mono => &MONO_GRADIENT,
        Palette::Forest => &FOREST_GRADIENT,
        Palette::Neon => &NEON_GRADIENT,
        Palette::Warm => &WARM_GRADIENT,
        Palette::Vibrant => &VIBRANT_GRADIENT,
        Palette::LegibleMono => &LEGIBLEMONO_GRADIENT,
    }
}

fn get_rgb_gradient(palette: Palette) -> &'static [RgbColor; 11] {
    match palette {
        Palette::Organic => &ORGANIC_RGB,
        Palette::Heat => &HEAT_RGB,
        Palette::Ocean => &OCEAN_RGB,
        Palette::Mono => &MONO_RGB,
        Palette::Forest => &FOREST_RGB,
        Palette::Neon => &NEON_RGB,
        Palette::Warm => &WARM_RGB,
        Palette::Vibrant => &VIBRANT_RGB,
        Palette::LegibleMono => &LEGIBLEMONO_RGB,
    }
}

fn invert_color(color_code: u8) -> u8 {
    255 - color_code
}

pub fn map_brightness(brightness: f32, palette: Palette, reverse: bool, invert: bool) -> u8 {
    let mut brightness = brightness.clamp(0.0, 1.0);
    let gradient = get_256_gradient(palette);

    if reverse {
        brightness = 1.0 - brightness;
    }

    let position = brightness * (gradient.len() - 1) as f32;
    let lower = position.floor() as usize;
    let upper = position.ceil() as usize;
    let fraction = position - lower as f32;

    let color = if upper == lower || fraction < 0.5 {
        gradient[lower]
    } else {
        gradient[upper]
    };

    let mut final_color = color;

    if invert {
        final_color = invert_color(final_color);
    }

    final_color
}

fn invert_rgb(rgb: RgbColor) -> RgbColor {
    RgbColor {
        r: 255 - rgb.r,
        g: 255 - rgb.g,
        b: 255 - rgb.b,
    }
}

pub fn map_brightness_rgb(brightness: f32, palette: Palette, reverse: bool, invert: bool) -> RgbColor {
    let mut brightness = brightness.clamp(0.0, 1.0);
    let gradient = get_rgb_gradient(palette);

    if reverse {
        brightness = 1.0 - brightness;
    }

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

    let final_color = RgbColor { r, g, b };

    if invert {
        invert_rgb(final_color)
    } else {
        final_color
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_map_brightness_min() {
        assert_eq!(map_brightness(0.0, Palette::Organic, false, false), 232);
        assert_eq!(map_brightness(0.0, Palette::Heat, false, false), 232);
        assert_eq!(map_brightness(0.0, Palette::Ocean, false, false), 232);
        assert_eq!(map_brightness(0.0, Palette::Mono, false, false), 232);
        assert_eq!(map_brightness(0.0, Palette::Forest, false, false), 22);
        assert_eq!(map_brightness(0.0, Palette::Neon, false, false), 17);
        assert_eq!(map_brightness(0.0, Palette::Warm, false, false), 52);
        assert_eq!(map_brightness(0.0, Palette::Vibrant, false, false), 197);
        assert_eq!(map_brightness(0.0, Palette::LegibleMono, false, false), 236);
    }

    #[test]
    fn test_map_brightness_max() {
        assert_eq!(map_brightness(1.0, Palette::Organic, false, false), 226);
        assert_eq!(map_brightness(1.0, Palette::Heat, false, false), 226);
        assert_eq!(map_brightness(1.0, Palette::Ocean, false, false), 51);
        assert_eq!(map_brightness(1.0, Palette::Mono, false, false), 252);
        assert_eq!(map_brightness(1.0, Palette::Forest, false, false), 40);
        assert_eq!(map_brightness(1.0, Palette::Neon, false, false), 195);
        assert_eq!(map_brightness(1.0, Palette::Warm, false, false), 226);
        assert_eq!(map_brightness(1.0, Palette::Vibrant, false, false), 231);
        assert_eq!(map_brightness(1.0, Palette::LegibleMono, false, false), 255);
    }

    #[test]
    fn test_map_brightness_mid() {
        let color = map_brightness(0.5, Palette::Organic, false, false);
        assert_eq!(color, 46);

        let color = map_brightness(0.5, Palette::Heat, false, false);
        assert_eq!(color, 196);

        let color = map_brightness(0.5, Palette::Ocean, false, false);
        assert_eq!(color, 21);

        let color = map_brightness(0.5, Palette::Mono, false, false);
        assert_eq!(color, 242);

        let color = map_brightness(0.5, Palette::Forest, false, false);
        assert_eq!(color, 40);

        let color = map_brightness(0.5, Palette::Neon, false, false);
        assert_eq!(color, 123);

        let color = map_brightness(0.5, Palette::Warm, false, false);
        assert_eq!(color, 208);

        let color = map_brightness(0.5, Palette::Vibrant, false, false);
        assert_eq!(color, 121);

        let color = map_brightness(0.5, Palette::LegibleMono, false, false);
        assert_eq!(color, 251);
    }

    #[test]
    fn test_map_brightness_clamped() {
        assert_eq!(map_brightness(-0.5, Palette::Organic, false, false), 232);
        assert_eq!(map_brightness(1.5, Palette::Organic, false, false), 226);
        assert_eq!(map_brightness(-0.5, Palette::Forest, false, false), 22);
        assert_eq!(map_brightness(1.5, Palette::Forest, false, false), 40);
    }

    #[test]
    fn test_map_brightness_quarter() {
        let color = map_brightness(0.25, Palette::Organic, false, false);
        assert_eq!(color, 34);

        let color = map_brightness(0.25, Palette::Heat, false, false);
        assert_eq!(color, 124);

        let color = map_brightness(0.25, Palette::Forest, false, false);
        assert_eq!(color, 34);

        let color = map_brightness(0.25, Palette::Neon, false, false);
        assert_eq!(color, 51);

        let color = map_brightness(0.25, Palette::Warm, false, false);
        assert_eq!(color, 166);
    }

    #[test]
    fn test_map_brightness_three_quarter() {
        let color = map_brightness(0.75, Palette::Organic, false, false);
        assert_eq!(color, 154);

        let color = map_brightness(0.75, Palette::Heat, false, false);
        assert_eq!(color, 214);

        let color = map_brightness(0.75, Palette::Forest, false, false);
        assert_eq!(color, 154);

        let color = map_brightness(0.75, Palette::Neon, false, false);
        assert_eq!(color, 201);

        let color = map_brightness(0.75, Palette::Warm, false, false);
        assert_eq!(color, 226);
    }

    #[test]
    fn test_reverse_palette() {
        assert_eq!(map_brightness(0.0, Palette::Organic, true, false), 226);
        assert_eq!(map_brightness(1.0, Palette::Organic, true, false), 232);
    }

    #[test]
    fn test_invert_palette() {
        let normal = map_brightness(0.5, Palette::Organic, false, false);
        let inverted = map_brightness(0.5, Palette::Organic, false, true);
        assert_eq!(inverted, 255 - normal);
    }

    #[test]
    fn test_reverse_and_invert_palette() {
        let reversed = map_brightness(0.0, Palette::Organic, true, false);
        let reversed_and_inverted = map_brightness(0.0, Palette::Organic, true, true);
        assert_eq!(reversed_and_inverted, invert_color(reversed));
    }

    #[test]
    fn test_map_brightness_rgb_min() {
        let color = map_brightness_rgb(0.0, Palette::Organic, false, false);
        assert_eq!(color.r, 18);
        assert_eq!(color.g, 18);
        assert_eq!(color.b, 18);
    }

    #[test]
    fn test_map_brightness_rgb_max() {
        let color = map_brightness_rgb(1.0, Palette::Organic, false, false);
        assert_eq!(color.r, 150);
        assert_eq!(color.g, 220);
        assert_eq!(color.b, 200);
    }

    #[test]
    fn test_map_brightness_rgb_interpolation() {
        let color = map_brightness_rgb(0.5, Palette::Organic, false, false);
        assert!(color.r >= 18 && color.r <= 160);
        assert!(color.g >= 18 && color.g <= 220);
        assert!(color.b >= 18 && color.b <= 200);

        let color = map_brightness_rgb(0.5, Palette::Ocean, false, false);
        assert!(color.r >= 18 && color.r <= 80);
        assert!(color.g >= 18 && color.g <= 170);
        assert!(color.b >= 18 && color.b <= 240);
    }

    #[test]
    fn test_map_brightness_rgb_heat() {
        let min_color = map_brightness_rgb(0.0, Palette::Heat, false, false);
        let max_color = map_brightness_rgb(1.0, Palette::Heat, false, false);
        assert_eq!(min_color.r, 40);
        assert_eq!(min_color.g, 20);
        assert_eq!(min_color.b, 20);
        assert_eq!(max_color.r, 240);
        assert_eq!(max_color.g, 220);
        assert_eq!(max_color.b, 180);
    }

    #[test]
    fn test_map_brightness_rgb_ocean() {
        let min_color = map_brightness_rgb(0.0, Palette::Ocean, false, false);
        let max_color = map_brightness_rgb(1.0, Palette::Ocean, false, false);
        assert_eq!(min_color.r, 18);
        assert_eq!(min_color.g, 18);
        assert_eq!(min_color.b, 18);
        assert_eq!(max_color.r, 80);
        assert_eq!(max_color.g, 170);
        assert_eq!(max_color.b, 240);
    }

    #[test]
    fn test_map_brightness_rgb_forest() {
        let min_color = map_brightness_rgb(0.0, Palette::Forest, false, false);
        let max_color = map_brightness_rgb(1.0, Palette::Forest, false, false);
        assert_eq!(min_color.r, 20);
        assert_eq!(min_color.g, 40);
        assert_eq!(min_color.b, 20);
        assert_eq!(max_color.r, 180);
        assert_eq!(max_color.g, 220);
        assert_eq!(max_color.b, 170);
    }

    #[test]
    fn test_map_brightness_rgb_reverse() {
        let normal = map_brightness_rgb(0.0, Palette::Organic, false, false);
        let reversed = map_brightness_rgb(1.0, Palette::Organic, true, false);
        assert_eq!(normal.r, reversed.r);
        assert_eq!(normal.g, reversed.g);
        assert_eq!(normal.b, reversed.b);
    }

    #[test]
    fn test_map_brightness_rgb_invert() {
        let normal = map_brightness_rgb(0.5, Palette::Organic, false, false);
        let inverted = map_brightness_rgb(0.5, Palette::Organic, false, true);
        assert_eq!(inverted.r, 255 - normal.r);
        assert_eq!(inverted.g, 255 - normal.g);
        assert_eq!(inverted.b, 255 - normal.b);
    }

    #[test]
    fn test_map_brightness_rgb_all_palettes() {
        let palettes = [
            Palette::Organic,
            Palette::Heat,
            Palette::Ocean,
            Palette::Mono,
            Palette::Forest,
            Palette::Neon,
            Palette::Warm,
            Palette::Vibrant,
            Palette::LegibleMono,
        ];

        for palette in palettes {
            let color = map_brightness_rgb(0.5, palette, false, false);
            assert!(color.r <= 255 && color.g <= 255 && color.b <= 255);
            assert!(color.r >= 0 && color.g >= 0 && color.b >= 0);
        }
    }

    #[test]
    fn test_map_brightness_rgb_clamped() {
        let min = map_brightness_rgb(-0.5, Palette::Heat, false, false);
        let max = map_brightness_rgb(1.5, Palette::Heat, false, false);
        let normal = map_brightness_rgb(0.5, Palette::Heat, false, false);
        assert_eq!(min.r, 40);
        assert_eq!(max.r, 240);
        assert!(min.r <= normal.r && normal.r <= max.r);
    }

    #[test]
    fn test_truecolor_ansi_fg() {
        let code = truecolor_ansi(255, 128, 64, true);
        assert_eq!(code, "\x1b[38;2;255;128;64m");
    }

    #[test]
    fn test_truecolor_ansi_bg() {
        let code = truecolor_ansi(255, 128, 64, false);
        assert_eq!(code, "\x1b[48;2;255;128;64m");
    }

    #[test]
    fn test_truecolor_ansi_fg_specific() {
        let code = truecolor_ansi_fg(42, 42, 42);
        assert_eq!(code, "\x1b[38;2;42;42;42m");
    }

    #[test]
    fn test_truecolor_ansi_bg_specific() {
        let code = truecolor_ansi_bg(42, 42, 42);
        assert_eq!(code, "\x1b[48;2;42;42;42m");
    }

    #[test]
    fn test_truecolor_ansi_zeros() {
        let code = truecolor_ansi(0, 0, 0, true);
        assert_eq!(code, "\x1b[38;2;0;0;0m");
    }

    #[test]
    fn test_truecolor_ansi_max_values() {
        let code = truecolor_ansi(255, 255, 255, true);
        assert_eq!(code, "\x1b[38;2;255;255;255m");
    }
}

#[allow(dead_code)]
pub fn truecolor_ansi(r: u8, g: u8, b: u8, is_fg: bool) -> String {
    if is_fg {
        format!("\x1b[38;2;{};{};{}m", r, g, b)
    } else {
        format!("\x1b[48;2;{};{};{}m", r, g, b)
    }
}

#[allow(dead_code)]
pub fn truecolor_ansi_fg(r: u8, g: u8, b: u8) -> String {
    format!("\x1b[38;2;{};{};{}m", r, g, b)
}

#[allow(dead_code)]
pub fn truecolor_ansi_bg(r: u8, g: u8, b: u8) -> String {
    format!("\x1b[48;2;{};{};{}m", r, g, b)
}
