use crate::cli::Args;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Charset {
    HalfBlock,
    Ascii,
    Braille,
}

impl Charset {
    pub fn from_args(args: &Args) -> Self {
        if args.braille {
            Charset::Braille
        } else if args.ascii {
            Charset::Ascii
        } else {
            Charset::HalfBlock
        }
    }
}

const HALF_BLOCK_CHARS: [char; 9] = [
    ' ', '\u{2581}', '\u{2582}', '\u{2583}', '\u{2584}', '\u{2585}', '\u{2586}', '\u{2587}',
    '\u{2588}',
];

const ASCII_CHARS: [char; 10] = [' ', '.', ':', '-', '=', '+', '*', '#', '%', '@'];

const ASCII_TOP_CHARS: [char; 10] = [' ', ',', '.', ':', '-', '=', '+', '*', '#', '^'];

const ASCII_BOTTOM_CHARS: [char; 10] = [' ', '\'', '.', ':', '-', '=', '+', '*', '#', 'v'];

const BRAILLE_PATTERNS: [char; 64] = [
    '\u{2800}', '\u{2801}', '\u{2808}', '\u{2809}', '\u{2802}', '\u{2803}', '\u{280A}', '\u{280B}',
    '\u{2810}', '\u{2811}', '\u{2818}', '\u{2819}', '\u{2812}', '\u{2813}', '\u{281A}', '\u{281B}',
    '\u{2840}', '\u{2841}', '\u{2848}', '\u{2849}', '\u{2842}', '\u{2843}', '\u{284A}', '\u{284B}',
    '\u{2850}', '\u{2851}', '\u{2858}', '\u{2859}', '\u{2852}', '\u{2853}', '\u{285A}', '\u{285B}',
    '\u{2820}', '\u{2821}', '\u{2828}', '\u{2829}', '\u{2822}', '\u{2823}', '\u{282A}', '\u{282B}',
    '\u{2830}', '\u{2831}', '\u{2838}', '\u{2839}', '\u{2832}', '\u{2833}', '\u{283A}', '\u{283B}',
    '\u{2860}', '\u{2861}', '\u{2868}', '\u{2869}', '\u{2862}', '\u{2863}', '\u{286A}', '\u{286B}',
    '\u{2870}', '\u{2871}', '\u{2878}', '\u{2879}', '\u{2872}', '\u{2873}', '\u{287A}', '\u{287B}',
];

pub fn map_brightness(brightness: f32, charset: Charset) -> char {
    let brightness = brightness.clamp(0.0, 1.0);

    match charset {
        Charset::HalfBlock => {
            let index = (brightness * (HALF_BLOCK_CHARS.len() - 1) as f32).round() as usize;
            HALF_BLOCK_CHARS[index]
        }
        Charset::Ascii => {
            let index = (brightness * (ASCII_CHARS.len() - 1) as f32).round() as usize;
            ASCII_CHARS[index]
        }
        Charset::Braille => {
            let index = (brightness * (BRAILLE_PATTERNS.len() - 1) as f32).round() as usize;
            BRAILLE_PATTERNS[index]
        }
    }
}

pub fn map_vertical_block(top: f32, bottom: f32) -> char {
    const THRESHOLD: f32 = 0.05;
    let top_above = top > THRESHOLD;
    let bottom_above = bottom > THRESHOLD;

    match (top_above, bottom_above) {
        (true, true) => '█',
        (true, false) => '▀',
        (false, true) => '▄',
        (false, false) => ' ',
    }
}

pub fn map_ascii_directional(brightness: f32, is_top: bool) -> char {
    let brightness = brightness.clamp(0.0, 1.0);
    let chars = if is_top {
        ASCII_TOP_CHARS
    } else {
        ASCII_BOTTOM_CHARS
    };
    let index = (brightness * (chars.len() - 1) as f32).round() as usize;
    chars[index]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_charset_from_args_default() {
        let args = Args {
            ascii: false,
            braille: false,
            ..Default::default()
        };
        assert_eq!(Charset::from_args(&args), Charset::HalfBlock);
    }

    #[test]
    fn test_charset_from_args_ascii() {
        let args = Args {
            ascii: true,
            braille: false,
            ..Default::default()
        };
        assert_eq!(Charset::from_args(&args), Charset::Ascii);
    }

    #[test]
    fn test_charset_from_args_braille() {
        let args = Args {
            ascii: false,
            braille: true,
            ..Default::default()
        };
        assert_eq!(Charset::from_args(&args), Charset::Braille);
    }

    #[test]
    fn test_map_brightness_halfblock_min() {
        assert_eq!(map_brightness(0.0, Charset::HalfBlock), ' ');
    }

    #[test]
    fn test_map_brightness_halfblock_max() {
        assert_eq!(map_brightness(1.0, Charset::HalfBlock), '\u{2588}');
    }

    #[test]
    fn test_map_brightness_ascii_min() {
        assert_eq!(map_brightness(0.0, Charset::Ascii), ' ');
    }

    #[test]
    fn test_map_brightness_ascii_max() {
        assert_eq!(map_brightness(1.0, Charset::Ascii), '@');
    }

    #[test]
    fn test_map_brightness_braille_min() {
        assert_eq!(map_brightness(0.0, Charset::Braille), '\u{2800}');
    }

    #[test]
    fn test_map_brightness_braille_max() {
        assert_eq!(map_brightness(1.0, Charset::Braille), '\u{287B}');
    }

    #[test]
    fn test_map_brightness_clamped() {
        assert_eq!(map_brightness(-0.5, Charset::HalfBlock), ' ');
        assert_eq!(map_brightness(1.5, Charset::Ascii), '@');
    }

    #[test]
    fn test_map_brightness_halfblock_mid() {
        let char = map_brightness(0.5, Charset::HalfBlock);
        assert_eq!(char, '\u{2584}');
    }

    #[test]
    fn test_map_brightness_ascii_mid() {
        let char = map_brightness(0.5, Charset::Ascii);
        assert_eq!(char, '+');
    }

    #[test]
    fn test_all_halfblock_chars() {
        for (i, _) in HALF_BLOCK_CHARS.iter().enumerate() {
            let brightness = i as f32 / (HALF_BLOCK_CHARS.len() - 1) as f32;
            let char = map_brightness(brightness, Charset::HalfBlock);
            assert_eq!(char, HALF_BLOCK_CHARS[i]);
        }
    }

    #[test]
    fn test_halfblock_chars_expanded() {
        assert_eq!(HALF_BLOCK_CHARS.len(), 9);
        assert_eq!(HALF_BLOCK_CHARS[0], ' ');
        assert_eq!(HALF_BLOCK_CHARS[1], '\u{2581}');
        assert_eq!(HALF_BLOCK_CHARS[4], '\u{2584}');
        assert_eq!(HALF_BLOCK_CHARS[7], '\u{2587}');
        assert_eq!(HALF_BLOCK_CHARS[8], '\u{2588}');
    }

    #[test]
    fn test_map_vertical_block_empty() {
        assert_eq!(map_vertical_block(0.0, 0.0), ' ');
        assert_eq!(map_vertical_block(0.01, 0.01), ' ');
    }

    #[test]
    fn test_map_vertical_block_top_only() {
        assert_eq!(map_vertical_block(1.0, 0.0), '▀');
        assert_eq!(map_vertical_block(0.5, 0.01), '▀');
        assert_eq!(map_vertical_block(0.06, 0.0), '▀');
    }

    #[test]
    fn test_map_vertical_block_bottom_only() {
        assert_eq!(map_vertical_block(0.0, 1.0), '▄');
        assert_eq!(map_vertical_block(0.01, 0.5), '▄');
        assert_eq!(map_vertical_block(0.0, 0.06), '▄');
    }

    #[test]
    fn test_map_vertical_block_full() {
        assert_eq!(map_vertical_block(1.0, 1.0), '█');
        assert_eq!(map_vertical_block(0.5, 0.5), '█');
        assert_eq!(map_vertical_block(0.06, 0.06), '█');
    }

    #[test]
    fn test_map_vertical_block_threshold_edge() {
        assert_eq!(map_vertical_block(0.05, 0.0), ' ');
        assert_eq!(map_vertical_block(0.06, 0.0), '▀');
        assert_eq!(map_vertical_block(0.0, 0.05), ' ');
        assert_eq!(map_vertical_block(0.0, 0.06), '▄');
    }

    #[test]
    fn test_map_ascii_directional_top_min() {
        assert_eq!(map_ascii_directional(0.0, true), ' ');
    }

    #[test]
    fn test_map_ascii_directional_top_max() {
        assert_eq!(map_ascii_directional(1.0, true), '^');
    }

    #[test]
    fn test_map_ascii_directional_bottom_min() {
        assert_eq!(map_ascii_directional(0.0, false), ' ');
    }

    #[test]
    fn test_map_ascii_directional_bottom_max() {
        assert_eq!(map_ascii_directional(1.0, false), 'v');
    }

    #[test]
    fn test_map_ascii_directional_top_mid() {
        assert_eq!(map_ascii_directional(0.5, true), '=');
    }

    #[test]
    fn test_map_ascii_directional_bottom_mid() {
        assert_eq!(map_ascii_directional(0.5, false), '=');
    }

    #[test]
    fn test_map_ascii_directional_clamped() {
        assert_eq!(map_ascii_directional(-0.5, true), ' ');
        assert_eq!(map_ascii_directional(1.5, false), 'v');
    }

    #[test]
    fn test_all_ascii_top_chars() {
        for (i, expected) in ASCII_TOP_CHARS.iter().enumerate() {
            let brightness = i as f32 / (ASCII_TOP_CHARS.len() - 1) as f32;
            let char = map_ascii_directional(brightness, true);
            assert_eq!(char, *expected);
        }
    }

    #[test]
    fn test_all_ascii_bottom_chars() {
        for (i, expected) in ASCII_BOTTOM_CHARS.iter().enumerate() {
            let brightness = i as f32 / (ASCII_BOTTOM_CHARS.len() - 1) as f32;
            let char = map_ascii_directional(brightness, false);
            assert_eq!(char, *expected);
        }
    }
}
