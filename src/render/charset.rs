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

const BRAILLE_DOT_MASKS: [u8; 5] = [
    0x00, // 0 dots
    0x01, // 1 dot
    0x03, // 2 dots
    0x07, // 3 dots
    0x07, // 3 dots (max)
];

pub fn map_braille_subpixel(top: f32, bottom: f32, threshold: f32) -> char {
    let top = top.clamp(0.0, 1.0);
    let bottom = bottom.clamp(0.0, 1.0);

    let top_level = if top < threshold {
        0
    } else {
        ((top - threshold) / (1.0 - threshold) * 4.0).ceil() as usize
    }
    .min(4);

    let bottom_level = if bottom < threshold {
        0
    } else {
        ((bottom - threshold) / (1.0 - threshold) * 4.0).ceil() as usize
    }
    .min(4);

    let left_mask = BRAILLE_DOT_MASKS[top_level];
    let right_mask = BRAILLE_DOT_MASKS[bottom_level] << 3;
    let combined_mask = left_mask | right_mask;

    char::from_u32(0x2800 + combined_mask as u32).unwrap_or(' ')
}

pub fn map_brightness(top: f32, bottom: Option<f32>, charset: Charset) -> char {
    match charset {
        Charset::HalfBlock => {
            let brightness = top.clamp(0.0, 1.0);
            let index = (brightness * (HALF_BLOCK_CHARS.len() - 1) as f32).round() as usize;
            HALF_BLOCK_CHARS[index]
        }
        Charset::Ascii => {
            let brightness = top.clamp(0.0, 1.0);
            let index = (brightness * (ASCII_CHARS.len() - 1) as f32).round() as usize;
            ASCII_CHARS[index]
        }
        Charset::Braille => {
            if let Some(bottom_val) = bottom {
                map_braille_subpixel(top, bottom_val, 0.05)
            } else {
                let brightness = top.clamp(0.0, 1.0);
                let index = (brightness * 15.0).round() as usize;
                char::from_u32(0x2800 + index as u32).unwrap_or(' ')
            }
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

pub fn charset_level_count(charset: Charset) -> usize {
    match charset {
        Charset::HalfBlock => 9,
        Charset::Ascii => 10,
        Charset::Braille => 16,
    }
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
        assert_eq!(map_brightness(0.0, None, Charset::HalfBlock), ' ');
    }

    #[test]
    fn test_map_brightness_halfblock_max() {
        assert_eq!(map_brightness(1.0, None, Charset::HalfBlock), '\u{2588}');
    }

    #[test]
    fn test_map_brightness_ascii_min() {
        assert_eq!(map_brightness(0.0, None, Charset::Ascii), ' ');
    }

    #[test]
    fn test_map_brightness_ascii_max() {
        assert_eq!(map_brightness(1.0, None, Charset::Ascii), '@');
    }

    #[test]
    fn test_map_brightness_braille_min() {
        assert_eq!(map_brightness(0.0, None, Charset::Braille), '\u{2800}');
    }

    #[test]
    fn test_map_brightness_braille_max() {
        assert_eq!(map_brightness(1.0, None, Charset::Braille), '\u{280F}');
    }

    #[test]
    fn test_map_brightness_clamped() {
        assert_eq!(map_brightness(-0.5, None, Charset::HalfBlock), ' ');
        assert_eq!(map_brightness(1.5, None, Charset::Ascii), '@');
    }

    #[test]
    fn test_map_brightness_halfblock_mid() {
        let char = map_brightness(0.5, None, Charset::HalfBlock);
        assert_eq!(char, '\u{2584}');
    }

    #[test]
    fn test_map_brightness_ascii_mid() {
        let char = map_brightness(0.5, None, Charset::Ascii);
        assert_eq!(char, '+');
    }

    #[test]
    fn test_all_halfblock_chars() {
        for (i, _) in HALF_BLOCK_CHARS.iter().enumerate() {
            let brightness = i as f32 / (HALF_BLOCK_CHARS.len() - 1) as f32;
            let char = map_brightness(brightness, None, Charset::HalfBlock);
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

    #[test]
    fn test_braille_subpixel_empty() {
        assert_eq!(map_braille_subpixel(0.0, 0.0, 0.05), '\u{2800}');
    }

    #[test]
    fn test_braille_subpixel_top_only() {
        assert_eq!(map_braille_subpixel(1.0, 0.0, 0.05), '\u{2807}');
    }

    #[test]
    fn test_braille_subpixel_bottom_only() {
        assert_eq!(map_braille_subpixel(0.0, 1.0, 0.05), '\u{2838}');
    }

    #[test]
    fn test_braille_subpixel_full() {
        assert_eq!(map_braille_subpixel(1.0, 1.0, 0.05), '\u{283F}');
    }

    #[test]
    fn test_braille_subpixel_top_levels() {
        assert_eq!(map_braille_subpixel(0.0, 0.0, 0.05), '\u{2800}');
        assert_eq!(map_braille_subpixel(0.3, 0.0, 0.05), '\u{2803}');
        assert_eq!(map_braille_subpixel(0.55, 0.0, 0.05), '\u{2807}');
        assert_eq!(map_braille_subpixel(1.0, 0.0, 0.05), '\u{2807}');
    }

    #[test]
    fn test_braille_subpixel_bottom_levels() {
        assert_eq!(map_braille_subpixel(0.0, 0.0, 0.05), '\u{2800}');
        assert_eq!(map_braille_subpixel(0.0, 0.3, 0.05), '\u{2818}');
        assert_eq!(map_braille_subpixel(0.0, 0.55, 0.05), '\u{2838}');
        assert_eq!(map_braille_subpixel(0.0, 1.0, 0.05), '\u{2838}');
    }

    #[test]
    fn test_braille_subpixel_all_combinations() {
        assert_eq!(map_braille_subpixel(0.0, 0.0, 0.05), '\u{2800}');
        assert_eq!(map_braille_subpixel(0.0, 0.3, 0.05), '\u{2818}');
        assert_eq!(map_braille_subpixel(0.0, 0.55, 0.05), '\u{2838}');
        assert_eq!(map_braille_subpixel(0.0, 1.0, 0.05), '\u{2838}');
        assert_eq!(map_braille_subpixel(0.3, 0.0, 0.05), '\u{2803}');
        assert_eq!(map_braille_subpixel(0.3, 0.3, 0.05), '\u{281B}');
        assert_eq!(map_braille_subpixel(0.3, 0.55, 0.05), '\u{283B}');
        assert_eq!(map_braille_subpixel(0.3, 1.0, 0.05), '\u{283B}');
        assert_eq!(map_braille_subpixel(0.55, 0.0, 0.05), '\u{2807}');
        assert_eq!(map_braille_subpixel(0.55, 0.3, 0.05), '\u{281F}');
        assert_eq!(map_braille_subpixel(0.55, 0.55, 0.05), '\u{283F}');
        assert_eq!(map_braille_subpixel(0.55, 1.0, 0.05), '\u{283F}');
        assert_eq!(map_braille_subpixel(1.0, 0.0, 0.05), '\u{2807}');
        assert_eq!(map_braille_subpixel(1.0, 0.3, 0.05), '\u{281F}');
        assert_eq!(map_braille_subpixel(1.0, 0.55, 0.05), '\u{283F}');
        assert_eq!(map_braille_subpixel(1.0, 1.0, 0.05), '\u{283F}');
    }

    #[test]
    fn test_braille_subpixel_threshold() {
        assert_eq!(map_braille_subpixel(0.04, 0.04, 0.05), '\u{2800}');
        assert_eq!(map_braille_subpixel(0.06, 0.0, 0.05), '\u{2801}');
        assert_eq!(map_braille_subpixel(0.0, 0.06, 0.05), '\u{2808}');
    }

    #[test]
    fn test_braille_subpixel_clamping() {
        assert_eq!(map_braille_subpixel(-0.5, 0.0, 0.05), '\u{2800}');
        assert_eq!(map_braille_subpixel(0.0, -0.5, 0.05), '\u{2800}');
        assert_eq!(map_braille_subpixel(1.5, 1.5, 0.05), '\u{283F}');
    }

    #[test]
    fn test_braille_dot_masks_values() {
        assert_eq!(BRAILLE_DOT_MASKS[0], 0x00);
        assert_eq!(BRAILLE_DOT_MASKS[1], 0x01);
        assert_eq!(BRAILLE_DOT_MASKS[2], 0x03);
        assert_eq!(BRAILLE_DOT_MASKS[3], 0x07);
        assert_eq!(BRAILLE_DOT_MASKS[4], 0x07);
    }

    #[test]
    fn test_map_brightness_braille_with_bottom() {
        assert_eq!(map_brightness(1.0, Some(1.0), Charset::Braille), '\u{283F}');
        assert_eq!(map_brightness(1.0, Some(0.0), Charset::Braille), '\u{2807}');
        assert_eq!(map_brightness(0.0, Some(1.0), Charset::Braille), '\u{2838}');
        assert_eq!(map_brightness(0.0, Some(0.0), Charset::Braille), '\u{2800}');
    }

    #[test]
    fn test_charset_level_count_halfblock() {
        assert_eq!(charset_level_count(Charset::HalfBlock), 9);
    }

    #[test]
    fn test_charset_level_count_ascii() {
        assert_eq!(charset_level_count(Charset::Ascii), 10);
    }

    #[test]
    fn test_charset_level_count_braille() {
        assert_eq!(charset_level_count(Charset::Braille), 16);
    }
}
