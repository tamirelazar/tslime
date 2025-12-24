use crate::cli::Palette;

const ORGANIC_GRADIENT: [u8; 11] = [232, 22, 28, 34, 40, 46, 82, 118, 154, 190, 226];

const HEAT_GRADIENT: [u8; 11] = [232, 52, 88, 124, 160, 196, 202, 208, 214, 220, 226];

const OCEAN_GRADIENT: [u8; 11] = [232, 17, 18, 19, 20, 21, 27, 33, 39, 45, 51];

const MONO_GRADIENT: [u8; 11] = [232, 234, 236, 238, 240, 242, 244, 246, 248, 250, 252];

fn get_gradient(palette: Palette) -> &'static [u8; 11] {
    match palette {
        Palette::Organic => &ORGANIC_GRADIENT,
        Palette::Heat => &HEAT_GRADIENT,
        Palette::Ocean => &OCEAN_GRADIENT,
        Palette::Mono => &MONO_GRADIENT,
    }
}

pub fn map_brightness(brightness: f32, palette: Palette) -> u8 {
    let brightness = brightness.clamp(0.0, 1.0);
    let gradient = get_gradient(palette);

    let position = brightness * (gradient.len() - 1) as f32;
    let lower = position.floor() as usize;
    let upper = position.ceil() as usize;
    let fraction = position - lower as f32;

    if upper == lower {
        return gradient[lower];
    }

    if fraction < 0.5 {
        gradient[lower]
    } else {
        gradient[upper]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_map_brightness_min() {
        assert_eq!(map_brightness(0.0, Palette::Organic), 232);
        assert_eq!(map_brightness(0.0, Palette::Heat), 232);
        assert_eq!(map_brightness(0.0, Palette::Ocean), 232);
        assert_eq!(map_brightness(0.0, Palette::Mono), 232);
    }

    #[test]
    fn test_map_brightness_max() {
        assert_eq!(map_brightness(1.0, Palette::Organic), 226);
        assert_eq!(map_brightness(1.0, Palette::Heat), 226);
        assert_eq!(map_brightness(1.0, Palette::Ocean), 51);
        assert_eq!(map_brightness(1.0, Palette::Mono), 252);
    }

    #[test]
    fn test_map_brightness_mid() {
        let color = map_brightness(0.5, Palette::Organic);
        assert_eq!(color, 46);

        let color = map_brightness(0.5, Palette::Heat);
        assert_eq!(color, 196);

        let color = map_brightness(0.5, Palette::Ocean);
        assert_eq!(color, 21);

        let color = map_brightness(0.5, Palette::Mono);
        assert_eq!(color, 242);
    }

    #[test]
    fn test_map_brightness_clamped() {
        assert_eq!(map_brightness(-0.5, Palette::Organic), 232);
        assert_eq!(map_brightness(1.5, Palette::Organic), 226);
    }

    #[test]
    fn test_map_brightness_quarter() {
        let color = map_brightness(0.25, Palette::Organic);
        assert_eq!(color, 34);

        let color = map_brightness(0.25, Palette::Heat);
        assert_eq!(color, 124);
    }

    #[test]
    fn test_map_brightness_three_quarter() {
        let color = map_brightness(0.75, Palette::Organic);
        assert_eq!(color, 154);

        let color = map_brightness(0.75, Palette::Heat);
        assert_eq!(color, 214);
    }
}
