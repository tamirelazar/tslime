use crate::cli::Palette;

const ORGANIC_GRADIENT: [u8; 11] = [232, 22, 28, 34, 40, 46, 82, 118, 154, 190, 226];

const HEAT_GRADIENT: [u8; 11] = [232, 52, 88, 124, 160, 196, 202, 208, 214, 220, 226];

const OCEAN_GRADIENT: [u8; 11] = [232, 17, 18, 19, 20, 21, 27, 33, 39, 45, 51];

const MONO_GRADIENT: [u8; 11] = [232, 234, 236, 238, 240, 242, 244, 246, 248, 250, 252];

const FOREST_GRADIENT: [u8; 11] = [22, 34, 46, 82, 118, 154, 190, 194, 230, 230, 230];

const NEON_GRADIENT: [u8; 11] = [17, 27, 39, 51, 87, 123, 159, 195, 201, 225, 195];

const WARM_GRADIENT: [u8; 11] = [52, 94, 130, 166, 202, 208, 214, 220, 226, 226, 226];

const VIBRANT_GRADIENT: [u8; 11] = [197, 209, 221, 193, 157, 121, 85, 49, 51, 87, 231];

const LEGIBLEMONO_GRADIENT: [u8; 11] = [236, 240, 244, 248, 250, 251, 252, 253, 254, 255, 255];

fn get_gradient(palette: Palette) -> &'static [u8; 11] {
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

fn invert_color(color_code: u8) -> u8 {
    255 - color_code
}

pub fn map_brightness(brightness: f32, palette: Palette, reverse: bool, invert: bool) -> u8 {
    let mut brightness = brightness.clamp(0.0, 1.0);
    let gradient = get_gradient(palette);

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
        assert_eq!(map_brightness(1.0, Palette::Forest, false, false), 230);
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
        assert_eq!(color, 154);

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
        assert_eq!(map_brightness(1.5, Palette::Forest, false, false), 230);
    }

    #[test]
    fn test_map_brightness_quarter() {
        let color = map_brightness(0.25, Palette::Organic, false, false);
        assert_eq!(color, 34);

        let color = map_brightness(0.25, Palette::Heat, false, false);
        assert_eq!(color, 124);

        let color = map_brightness(0.25, Palette::Forest, false, false);
        assert_eq!(color, 82);

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
        assert_eq!(color, 230);

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
}
