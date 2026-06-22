use crate::cli::Args;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq)]
/// Character set for rendering simulation trails.
pub enum Charset {
    /// Half-block characters (▀, ▄, █) for 2× vertical resolution.
    HalfBlock,
    /// Dual-color half-block mode: uses ▀ with independent fg/bg colors for true 2× color resolution.
    HalfBlockDual,
    /// Standard ASCII characters mapped by density.
    Ascii,
    /// Braille patterns for 4×2 subpixel resolution.
    Braille,
    /// Unicode quadrant characters for 2×2 subpixel resolution.
    Quadrant,
    /// Shade block characters (░▒▓█) for smooth density gradients.
    Shade,
    /// Point grid using ▪ for sparse particle visualization.
    Points,
    /// Sculpted mode: solid interior blocks with shape-aware outlines using
    /// triangle fills, quadrant blocks, and half blocks.
    Sculpted,
    /// User-defined ASCII character set.
    CustomAscii(Vec<char>),
}

impl Charset {
    /// Selects the appropriate character set based on command-line arguments.
    pub fn from_args(args: &Args) -> Self {
        if args.sculpted {
            Charset::Sculpted
        } else if args.quadrant {
            Charset::Quadrant
        } else if args.shade {
            Charset::Shade
        } else if args.points {
            Charset::Points
        } else if args.braille {
            Charset::Braille
        } else if args.half_block_dual {
            Charset::HalfBlockDual
        } else if let Some(ref custom_chars) = args.ascii_chars {
            // Create custom ASCII charset from user-provided characters
            Charset::from_custom_string(custom_chars)
        } else if args.ascii {
            Charset::Ascii
        } else {
            Charset::HalfBlockDual
        }
    }

    /// Creates a CustomAscii charset from a string of characters, sorted by visual density
    pub fn from_custom_string(chars: &str) -> Self {
        let mut unique_chars: Vec<char> = chars.chars().collect();

        // Remove duplicates while preserving order
        unique_chars.dedup();

        // Sort by estimated visual density
        unique_chars.sort_by(|a, b| {
            estimate_char_density(*a)
                .partial_cmp(&estimate_char_density(*b))
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        Charset::CustomAscii(unique_chars)
    }
}

/// Character-selection strategy for the glyph-by-shape lever (#34, lever 10).
/// `None` (via [`GlyphConfig`]) preserves each charset's native behavior.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum GlyphSelection {
    /// Force the tonal brightness ramp (`map_brightness`) on shape-capable charsets.
    Brightness,
    /// Use the charset's native shape path (today's behavior for Ascii/Braille/Sculpted).
    Shape,
    /// Sobel edge-orientation: directional glyphs on high-gradient cells,
    /// brightness buckets in flat regions. ASCII-family only.
    Hybrid,
}

/// Bundled glyph-selection config threaded through the render path (one param,
/// mirroring `PaletteCycle`). `selection: None` = native per-charset behavior.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GlyphConfig {
    /// Glyph-selection strategy (e.g., Brightness, Shape, Hybrid).
    pub selection: Option<GlyphSelection>,
    /// Sobel gradient-magnitude threshold for edge-glyph selection (Hybrid mode).
    pub edge_threshold: f32,
}

impl Default for GlyphConfig {
    fn default() -> Self {
        Self {
            selection: None,
            edge_threshold: crate::config_defaults::glyph_consts::DEFAULT_GLYPH_EDGE_THRESHOLD,
        }
    }
}

/// Sobel edge-orientation glyph for a row-major 3×3 brightness neighborhood
/// (`n[4]` = center). Returns a directional glyph (`| / - \`) when gradient
/// magnitude exceeds `threshold`, else `None` (caller falls back to a
/// brightness bucket). Gradients are normalized to `[-1,1]`. (#34, lever 10.)
pub fn sobel_edge_glyph(n: &[f32; 9], threshold: f32) -> Option<char> {
    let gx = ((n[2] + 2.0 * n[5] + n[8]) - (n[0] + 2.0 * n[3] + n[6])) / 4.0;
    let gy = ((n[6] + 2.0 * n[7] + n[8]) - (n[0] + 2.0 * n[1] + n[2])) / 4.0;
    let mag = (gx * gx + gy * gy).sqrt();
    if mag <= threshold {
        return None;
    }
    let mut a = gx.atan2(gy).to_degrees();
    if a < 0.0 {
        a += 180.0;
    }
    let glyph = if !(22.5..157.5).contains(&a) {
        '-'
    } else if a < 67.5 {
        '/'
    } else if a < 112.5 {
        '|'
    } else {
        '\\'
    };
    Some(glyph)
}

/// List of all available charsets for cycling.
/// This is the single source of truth for charset enumeration.
pub const ALL_CHARSETS: [Charset; 7] = [
    Charset::HalfBlock,
    Charset::HalfBlockDual,
    Charset::Ascii,
    Charset::Braille,
    Charset::Quadrant,
    Charset::Shade,
    Charset::Points,
];

/// Number of distinct charsets. Single source of truth for any per-charset
/// array (e.g. color-AA strength) so they stay locked to [`ALL_CHARSETS`].
pub const NUM_CHARSETS: usize = ALL_CHARSETS.len();

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

/// Shade block characters for smooth density gradients (░▒▓█).
const SHADE_CHARS: [char; 5] = [' ', '░', '▒', '▓', '█'];

/// Point grid character for sparse visualization (▪ Black Small Square U+25AA).
const POINT_CHAR: char = '▪';

/// Estimates the visual density/weight of a character for sorting
/// Returns a value from 0.0 (lightest) to 1.0 (darkest)
#[inline]
fn estimate_char_density(c: char) -> f32 {
    match c {
        // Whitespace and very light characters
        ' ' => 0.0,
        '\t' | '\n' => 0.0,

        // Very light punctuation
        '.' | ',' | '\'' | '`' | '´' | '°' | '·' | '˙' => 0.1,

        // Light punctuation and symbols
        ':' | ';' | '!' | '|' | 'i' | 'l' | 'I' | '1' => 0.2,
        '/' | '\\' | '(' | ')' | '[' | ']' | '{' | '}' => 0.25,

        // Medium-light characters
        '-' | '_' | '~' | '^' | '"' | 'r' | 'c' | 'v' | 'f' | 'j' | 't' => 0.3,
        '<' | '>' | 'L' | 'T' | 'Y' | 'J' | '7' => 0.35,

        // Medium characters
        '=' | '+' | '?' | 's' | 'z' | 'x' | 'k' | 'n' | 'u' | 'y' => 0.4,
        'a' | 'e' | 'o' | 'h' | 'p' | 'q' | 'b' | 'd' | 'g' => 0.45,
        'F' | 'P' | 'E' | 'Z' | '2' | '3' | '5' => 0.5,

        // Medium-dark characters
        '*' | '×' | 'w' | 'm' | 'V' | 'X' | 'K' => 0.55,
        'A' | 'S' | 'C' | 'U' | 'R' | 'G' | '4' | '6' | '9' => 0.6,

        // Dark characters
        '#' | 'H' | 'N' | 'D' | 'B' | 'O' | 'Q' | '8' | '0' => 0.7,
        'W' | 'M' => 0.75,

        // Very dark characters
        '@' | '%' | '&' | '$' => 0.8,
        '▓' | '▒' => 0.9,
        '■' | '▪' | '▬' => 0.95,

        // Unicode block characters (darkest)
        '█' => 1.0, // Full block (U+2588)
        '\u{2587}' => 0.875,
        '\u{2586}' => 0.75,
        '\u{2585}' => 0.625,
        '\u{2584}' => 0.5,
        '\u{2583}' => 0.375,
        '\u{2582}' => 0.25,
        '\u{2581}' => 0.125,

        // Default: estimate based on Unicode category
        _ => {
            if c.is_ascii_uppercase() {
                0.55
            } else if c.is_ascii_lowercase() {
                0.4
            } else if c.is_ascii_digit() {
                0.5
            } else if c.is_ascii_punctuation() {
                0.3
            } else {
                // Unknown character - assume medium density
                0.5
            }
        }
    }
}

/// Maps two vertical subpixels to a Braille character.
///
/// Simplified intensity mapping: `top` fills the left dot column and `bottom`
/// the right column (0-3 dots each), approximating brightness rather than
/// true subpixel layout.
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

/// Maps brightness (0.0-1.0) to one of 5 shade characters (' ' ░ ▒ ▓ █)
/// by rounding to the nearest level (boundaries at 12.5%, 37.5%, 62.5%, 87.5%).
pub fn map_shade(brightness: f32) -> char {
    let b = brightness.clamp(0.0, 1.0);
    let index = (b * (SHADE_CHARS.len() - 1) as f32).round() as usize;
    SHADE_CHARS[index]
}

/// Returns ▪ if `brightness` exceeds `threshold`, space otherwise.
/// Best for sparse particle visualization.
pub fn map_point(brightness: f32, threshold: f32) -> char {
    if brightness > threshold {
        POINT_CHAR
    } else {
        ' '
    }
}

/// Maps four quadrant brightness values to a Unicode block element
/// (U+2580-U+259F). Each quadrant is on or off based on `threshold`.
pub fn map_quadrant(
    top_left: f32,
    top_right: f32,
    bottom_left: f32,
    bottom_right: f32,
    threshold: f32,
) -> char {
    let threshold = threshold.clamp(0.0, 1.0);

    let tl = top_left > threshold;
    let tr = top_right > threshold;
    let bl = bottom_left > threshold;
    let br = bottom_right > threshold;

    // Map quadrants to bit pattern (TL=bit0, TR=bit1, BL=bit2, BR=bit3)
    let index = (tl as u32) | ((tr as u32) << 1) | ((bl as u32) << 2) | ((br as u32) << 3);

    // Standard Unicode quadrant characters (U+2580-U+259F)
    match index {
        0x0 => ' ',        // 0000: Empty
        0x1 => '\u{2598}', // 0001: TL
        0x2 => '\u{259D}', // 0010: TR
        0x3 => '\u{2580}', // 0011: TL+TR (▀)
        0x4 => '\u{2596}', // 0100: BL
        0x5 => '\u{258C}', // 0101: TL+BL (▌)
        0x6 => '\u{259E}', // 0110: TR+BL (▞)
        0x7 => '\u{259B}', // 0111: TL+TR+BL (▛)
        0x8 => '\u{2597}', // 1000: BR
        0x9 => '\u{259A}', // 1001: TL+BR (▚)
        0xA => '\u{2590}', // 1010: TR+BR (▐)
        0xB => '\u{259C}', // 1011: TL+TR+BR (▜)
        0xC => '\u{2584}', // 1100: BL+BR (▄)
        0xD => '\u{2599}', // 1101: TL+BL+BR (▙)
        0xE => '\u{259F}', // 1110: TR+BL+BR (▟)
        0xF => '\u{2588}', // 1111: Full (█)
        _ => ' ',
    }
}

/// Maps a brightness value to a character from the selected charset.
///
/// `top` is the top-subpixel (or overall cell) brightness; `bottom` is the
/// optional bottom-subpixel brightness used by braille/shade/points modes.
pub fn map_brightness(top: f32, bottom: Option<f32>, charset: Charset) -> char {
    match charset {
        Charset::HalfBlock | Charset::HalfBlockDual => {
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
            use crate::config_defaults::threshold;
            if let Some(bottom_val) = bottom {
                map_braille_subpixel(top, bottom_val, threshold::DEFAULT_BRAILLE_THRESHOLD)
            } else {
                let brightness = top.clamp(0.0, 1.0);
                let index = (brightness * 15.0).round() as usize;
                char::from_u32(0x2800 + index as u32).unwrap_or(' ')
            }
        }
        Charset::Quadrant => {
            // For quadrant mode without explicit quadrant values, treat as simple brightness
            // This will be overridden when using the downsampler with quadrant support
            let brightness = top.clamp(0.0, 1.0);
            if brightness < 0.25 {
                ' '
            } else if brightness < 0.5 {
                '\u{1FB00}' // Single quadrant
            } else if brightness < 0.75 {
                '\u{1FB02}' // Half filled
            } else {
                '\u{1FB0E}' // Full block
            }
        }
        Charset::Shade => {
            // Average top and bottom for single-cell density
            let avg = if let Some(bottom_val) = bottom {
                (top + bottom_val) / 2.0
            } else {
                top
            };
            map_shade(avg)
        }
        Charset::Points => {
            use crate::config_defaults::threshold;
            let avg = if let Some(bottom_val) = bottom {
                (top + bottom_val) / 2.0
            } else {
                top
            };
            map_point(avg, threshold::DEFAULT_POINT_THRESHOLD) // Higher threshold for sparse dots
        }
        Charset::Sculpted => {
            // Fallback: same as HalfBlock when no quadrant data available
            let brightness = top.clamp(0.0, 1.0);
            let index = (brightness * (HALF_BLOCK_CHARS.len() - 1) as f32).round() as usize;
            HALF_BLOCK_CHARS[index]
        }
        Charset::CustomAscii(ref chars) => {
            if chars.is_empty() {
                return ' ';
            }
            let brightness = top.clamp(0.0, 1.0);
            let index = (brightness * (chars.len() - 1) as f32).round() as usize;
            chars[index.min(chars.len() - 1)]
        }
    }
}

/// Maps two vertical subpixels for dual-color half-block mode.
///
/// Returns '▀', '▄', or ' '. Unlike `map_vertical_block`, this never returns '█'
/// because both halves get independent colors via fg/bg. The character choice tells
/// the renderer which half is foreground:
/// - '▀': foreground = top, background = bottom
/// - '▄': foreground = bottom (used when only bottom is lit)
/// - ' ': both halves dark
pub fn map_half_block_dual(top: f32, bottom: f32, threshold: f32) -> char {
    let top_above = top > threshold;
    let bottom_above = bottom > threshold;

    match (top_above, bottom_above) {
        (true, true) => '▀',  // fg = top color, bg = bottom color
        (true, false) => '▀', // fg = top color, bg = default
        (false, true) => '▄', // fg = bottom color, bg = default
        (false, false) => ' ',
    }
}

/// Maps two vertical subpixel values to a block character.
///
/// Returns '█', '▀', '▄', or ' ' based on which subpixels exceed the threshold.
pub fn map_vertical_block(top: f32, bottom: f32) -> char {
    use crate::config_defaults::threshold;
    let top_above = top > threshold::DEFAULT_VERTICAL_BLOCK_THRESHOLD;
    let bottom_above = bottom > threshold::DEFAULT_VERTICAL_BLOCK_THRESHOLD;

    match (top_above, bottom_above) {
        (true, true) => '█',
        (true, false) => '▀',
        (false, true) => '▄',
        (false, false) => ' ',
    }
}

/// Maps brightness to an ASCII character, selecting from top-heavy or bottom-heavy sets.
///
/// This provides a pseudo-vertical-resolution effect using characters like `'` vs `.`.
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

// ===== Shape Vector ASCII Rendering =====
//
// Based on Alex Harri's technique: instead of mapping a single brightness
// value to a character by density, we match the *spatial distribution* of
// brightness within each cell to the visual shape of ASCII characters.
//
// Each character is described by a 6D "shape vector" representing ink density
// across a 2×3 grid:
//   [top_left, top_right, mid_left, mid_right, bot_left, bot_right]
//
// The input cell is sampled the same way and matched to the nearest character
// by Euclidean distance, so edge-like patterns get edge-like characters
// (/, \, |, -, etc.) rather than uniform fill characters.

/// Shape vector entry: a character paired with its 2×3 spatial density profile.
struct ShapeEntry {
    ch: char,
    vector: [f32; 6],
}

/// Precomputed shape vectors for ~55 printable ASCII characters.
///
/// Values represent approximate ink density in a 2×3 grid layout:
///   [TL, TR, ML, MR, BL, BR]
/// where each value ∈ [0.0, 1.0].
///
/// These are tuned for typical monospace terminal fonts (Consolas, DejaVu Mono,
/// etc.) and emphasize characters useful for rendering organic contours.
const SHAPE_TABLE: [ShapeEntry; 55] = [
    // === Empty / very light ===
    ShapeEntry {
        ch: ' ',
        vector: [0.00, 0.00, 0.00, 0.00, 0.00, 0.00],
    },
    ShapeEntry {
        ch: '.',
        vector: [0.00, 0.00, 0.00, 0.00, 0.05, 0.05],
    },
    ShapeEntry {
        ch: ',',
        vector: [0.00, 0.00, 0.00, 0.00, 0.02, 0.10],
    },
    ShapeEntry {
        ch: '\'',
        vector: [0.05, 0.08, 0.00, 0.00, 0.00, 0.00],
    },
    ShapeEntry {
        ch: '`',
        vector: [0.10, 0.03, 0.00, 0.00, 0.00, 0.00],
    },
    ShapeEntry {
        ch: '"',
        vector: [0.10, 0.10, 0.00, 0.00, 0.00, 0.00],
    },
    ShapeEntry {
        ch: ':',
        vector: [0.00, 0.00, 0.08, 0.08, 0.08, 0.08],
    },
    ShapeEntry {
        ch: ';',
        vector: [0.00, 0.00, 0.06, 0.06, 0.04, 0.10],
    },
    // === Horizontal lines ===
    ShapeEntry {
        ch: '-',
        vector: [0.00, 0.00, 0.35, 0.35, 0.00, 0.00],
    },
    ShapeEntry {
        ch: '_',
        vector: [0.00, 0.00, 0.00, 0.00, 0.35, 0.35],
    },
    ShapeEntry {
        ch: '~',
        vector: [0.00, 0.00, 0.28, 0.28, 0.00, 0.00],
    },
    ShapeEntry {
        ch: '=',
        vector: [0.12, 0.12, 0.30, 0.30, 0.12, 0.12],
    },
    // === Vertical lines ===
    ShapeEntry {
        ch: '|',
        vector: [0.15, 0.15, 0.18, 0.18, 0.15, 0.15],
    },
    ShapeEntry {
        ch: '!',
        vector: [0.12, 0.12, 0.12, 0.12, 0.03, 0.08],
    },
    ShapeEntry {
        ch: 'i',
        vector: [0.08, 0.08, 0.12, 0.12, 0.12, 0.12],
    },
    // === Diagonals ===
    ShapeEntry {
        ch: '/',
        vector: [0.03, 0.30, 0.18, 0.18, 0.30, 0.03],
    },
    ShapeEntry {
        ch: '\\',
        vector: [0.30, 0.03, 0.18, 0.18, 0.03, 0.30],
    },
    // === Corners and brackets ===
    ShapeEntry {
        ch: '(',
        vector: [0.10, 0.00, 0.22, 0.00, 0.10, 0.00],
    },
    ShapeEntry {
        ch: ')',
        vector: [0.00, 0.10, 0.00, 0.22, 0.00, 0.10],
    },
    ShapeEntry {
        ch: '[',
        vector: [0.22, 0.00, 0.22, 0.00, 0.22, 0.00],
    },
    ShapeEntry {
        ch: ']',
        vector: [0.00, 0.22, 0.00, 0.22, 0.00, 0.22],
    },
    ShapeEntry {
        ch: '{',
        vector: [0.08, 0.00, 0.22, 0.00, 0.08, 0.00],
    },
    ShapeEntry {
        ch: '}',
        vector: [0.00, 0.08, 0.00, 0.22, 0.00, 0.08],
    },
    ShapeEntry {
        ch: '<',
        vector: [0.00, 0.18, 0.22, 0.00, 0.00, 0.18],
    },
    ShapeEntry {
        ch: '>',
        vector: [0.18, 0.00, 0.00, 0.22, 0.18, 0.00],
    },
    // === Top-heavy / bottom-heavy ===
    ShapeEntry {
        ch: '^',
        vector: [0.12, 0.12, 0.00, 0.00, 0.00, 0.00],
    },
    ShapeEntry {
        ch: 'v',
        vector: [0.00, 0.00, 0.00, 0.00, 0.12, 0.12],
    },
    ShapeEntry {
        ch: 'T',
        vector: [0.30, 0.30, 0.12, 0.12, 0.12, 0.12],
    },
    ShapeEntry {
        ch: 'L',
        vector: [0.15, 0.00, 0.15, 0.00, 0.25, 0.25],
    },
    ShapeEntry {
        ch: 'J',
        vector: [0.00, 0.15, 0.00, 0.15, 0.22, 0.22],
    },
    ShapeEntry {
        ch: 'Y',
        vector: [0.22, 0.22, 0.12, 0.12, 0.12, 0.12],
    },
    // === Medium fill / structural ===
    ShapeEntry {
        ch: '+',
        vector: [0.10, 0.10, 0.38, 0.38, 0.10, 0.10],
    },
    ShapeEntry {
        ch: '*',
        vector: [0.18, 0.18, 0.25, 0.25, 0.18, 0.18],
    },
    ShapeEntry {
        ch: 'x',
        vector: [0.25, 0.25, 0.10, 0.10, 0.25, 0.25],
    },
    ShapeEntry {
        ch: 'o',
        vector: [0.15, 0.15, 0.22, 0.22, 0.15, 0.15],
    },
    ShapeEntry {
        ch: 'c',
        vector: [0.12, 0.15, 0.18, 0.00, 0.12, 0.15],
    },
    ShapeEntry {
        ch: 'n',
        vector: [0.22, 0.22, 0.18, 0.18, 0.18, 0.18],
    },
    ShapeEntry {
        ch: 'u',
        vector: [0.18, 0.18, 0.18, 0.18, 0.22, 0.22],
    },
    ShapeEntry {
        ch: 's',
        vector: [0.08, 0.25, 0.18, 0.18, 0.25, 0.08],
    },
    ShapeEntry {
        ch: 'r',
        vector: [0.12, 0.18, 0.18, 0.05, 0.15, 0.00],
    },
    ShapeEntry {
        ch: 'z',
        vector: [0.20, 0.28, 0.18, 0.18, 0.28, 0.20],
    },
    ShapeEntry {
        ch: 't',
        vector: [0.12, 0.00, 0.25, 0.10, 0.12, 0.12],
    },
    // === Heavy fill ===
    ShapeEntry {
        ch: '#',
        vector: [0.55, 0.55, 0.60, 0.60, 0.55, 0.55],
    },
    ShapeEntry {
        ch: '@',
        vector: [0.58, 0.58, 0.65, 0.65, 0.55, 0.55],
    },
    ShapeEntry {
        ch: '%',
        vector: [0.40, 0.18, 0.25, 0.25, 0.18, 0.40],
    },
    ShapeEntry {
        ch: '&',
        vector: [0.30, 0.20, 0.40, 0.30, 0.35, 0.30],
    },
    ShapeEntry {
        ch: '$',
        vector: [0.20, 0.30, 0.35, 0.25, 0.30, 0.20],
    },
    ShapeEntry {
        ch: 'H',
        vector: [0.35, 0.35, 0.50, 0.50, 0.35, 0.35],
    },
    ShapeEntry {
        ch: 'N',
        vector: [0.42, 0.35, 0.38, 0.38, 0.35, 0.42],
    },
    ShapeEntry {
        ch: 'M',
        vector: [0.50, 0.50, 0.42, 0.42, 0.38, 0.38],
    },
    ShapeEntry {
        ch: 'W',
        vector: [0.38, 0.38, 0.50, 0.50, 0.35, 0.35],
    },
    ShapeEntry {
        ch: 'O',
        vector: [0.28, 0.28, 0.38, 0.38, 0.28, 0.28],
    },
    ShapeEntry {
        ch: 'B',
        vector: [0.48, 0.38, 0.48, 0.38, 0.48, 0.38],
    },
    ShapeEntry {
        ch: 'D',
        vector: [0.48, 0.32, 0.48, 0.38, 0.48, 0.32],
    },
    ShapeEntry {
        ch: 'S',
        vector: [0.15, 0.35, 0.30, 0.30, 0.35, 0.15],
    },
];

/// Applies global contrast enhancement to a shape vector.
///
/// Normalizes the vector to its peak, applies an exponent to amplify
/// differences between bright and dim regions, then denormalizes.
/// Uniform regions are unaffected; regions with spatial variation
/// get sharper separation.
#[inline]
fn enhance_contrast(v: &mut [f32; 6], exponent: f32) {
    let max = v.iter().cloned().fold(0.0_f32, f32::max);
    if max < 1e-6 {
        return;
    }
    for val in v.iter_mut() {
        let normalized = *val / max;
        *val = normalized.powf(exponent) * max;
    }
}

/// Applies directional contrast enhancement using neighboring cell values.
///
/// For each of the 6 sampling regions, we compare the internal value to
/// the corresponding region of a neighboring cell. If the neighbor is
/// brighter in that direction, we push the internal value darker,
/// sharpening the boundary between cells.
#[inline]
fn enhance_directional_contrast(v: &mut [f32; 6], neighbors: &[[f32; 6]], exponent: f32) {
    let max = v.iter().cloned().fold(0.0_f32, f32::max);
    if max < 1e-6 {
        return;
    }
    for (i, val) in v.iter_mut().enumerate() {
        let normalized = *val / max;
        // Find the maximum neighboring value for this sampling region
        let max_neighbor = neighbors.iter().map(|n| n[i]).fold(0.0_f32, f32::max);
        let neighbor_norm = if max > 1e-6 {
            (max_neighbor / max).min(1.0)
        } else {
            0.0
        };
        // If this region is dimmer than the neighbor's corresponding region,
        // push it darker (higher exponent = more suppression)
        let contrast_exp = if neighbor_norm > normalized + 0.05 {
            exponent * 1.5
        } else {
            exponent
        };
        *val = normalized.powf(contrast_exp) * max;
    }
}

/// Maps spatial brightness distribution to an ASCII character using shape vectors.
///
/// Constructs a 6D shape vector from the cell's quadrant brightness values
/// (0.0-1.0) and finds the character whose visual density distribution best
/// matches, using squared Euclidean distance. `contrast` is the enhancement
/// exponent (1.0 = none, 2.0 = strong).
pub fn map_shape_ascii(tl: f32, tr: f32, bl: f32, br: f32, contrast: f32) -> char {
    // Construct 2×3 shape vector from 2×2 quadrant data
    // Middle row is estimated as the average of top and bottom halves
    let mut v = [tl, tr, (tl + bl) * 0.5, (tr + br) * 0.5, bl, br];

    // Apply contrast enhancement if requested
    if contrast > 1.01 {
        enhance_contrast(&mut v, contrast);
    }

    // Normalize to unit scale for shape matching (we care about distribution,
    // not absolute magnitude). The overall brightness is handled by color.
    let max = v.iter().cloned().fold(0.0_f32, f32::max);
    if max < 0.01 {
        return ' ';
    }
    let inv_max = 1.0 / max;
    for val in v.iter_mut() {
        *val *= inv_max;
    }

    // Find the character with the smallest squared Euclidean distance
    // Early termination threshold: stop if we find a very close match
    const EARLY_TERMINATION_THRESHOLD: f32 = 0.001;
    let mut best_char = ' ';
    let mut best_dist = f32::MAX;

    for entry in &SHAPE_TABLE {
        // Skip space — we already handle fully empty cells above
        if entry.ch == ' ' {
            continue;
        }

        // Normalize the character vector the same way
        let cv = &entry.vector;
        let c_max = cv.iter().cloned().fold(0.0_f32, f32::max);
        if c_max < 1e-6 {
            continue;
        }
        let c_inv = 1.0 / c_max;

        let mut dist = 0.0_f32;
        for i in 0..6 {
            let diff = v[i] - cv[i] * c_inv;
            dist += diff * diff;
        }

        // Early termination: if we've found a very close match, stop searching
        if dist < EARLY_TERMINATION_THRESHOLD {
            best_char = entry.ch;
            break;
        }

        if dist < best_dist {
            best_dist = dist;
            best_char = entry.ch;
        }
    }

    best_char
}

/// Shape-vector ASCII matching with directional contrast using neighbor data.
///
/// Like `map_shape_ascii`, but also sharpens boundaries against adjacent
/// cells: `neighbor_quads` holds `[tl, tr, bl, br]` arrays for each neighbor.
pub fn map_shape_ascii_with_neighbors(
    tl: f32,
    tr: f32,
    bl: f32,
    br: f32,
    neighbor_quads: &[[f32; 4]],
    contrast: f32,
) -> char {
    // Construct 2×3 shape vector
    let mut v = [tl, tr, (tl + bl) * 0.5, (tr + br) * 0.5, bl, br];

    // Build neighbor shape vectors
    let neighbor_vecs: Vec<[f32; 6]> = neighbor_quads
        .iter()
        .map(|q| {
            [
                q[0],
                q[1],
                (q[0] + q[2]) * 0.5,
                (q[1] + q[3]) * 0.5,
                q[2],
                q[3],
            ]
        })
        .collect();

    // Apply contrast enhancement
    if contrast > 1.01 {
        enhance_contrast(&mut v, contrast);
        enhance_directional_contrast(&mut v, &neighbor_vecs, contrast);
    }

    // Normalize to unit scale
    let max = v.iter().cloned().fold(0.0_f32, f32::max);
    if max < 0.01 {
        return ' ';
    }
    let inv_max = 1.0 / max;
    for val in v.iter_mut() {
        *val *= inv_max;
    }

    // Find nearest character with early termination
    const EARLY_TERMINATION_THRESHOLD: f32 = 0.001;
    let mut best_char = ' ';
    let mut best_dist = f32::MAX;

    for entry in &SHAPE_TABLE {
        if entry.ch == ' ' {
            continue;
        }

        let cv = &entry.vector;
        let c_max = cv.iter().cloned().fold(0.0_f32, f32::max);
        if c_max < 1e-6 {
            continue;
        }
        let c_inv = 1.0 / c_max;

        let mut dist = 0.0_f32;
        for i in 0..6 {
            let diff = v[i] - cv[i] * c_inv;
            dist += diff * diff;
        }

        // Early termination: if we've found a very close match, stop searching
        if dist < EARLY_TERMINATION_THRESHOLD {
            best_char = entry.ch;
            break;
        }

        if dist < best_dist {
            best_dist = dist;
            best_char = entry.ch;
        }
    }

    best_char
}

// ===== Shape-Vector Braille Rendering =====
//
// Instead of mapping just 2 values (top/bottom) to ~9 braille patterns,
// we interpolate 8 sampling positions across the 2×4 braille dot grid
// from the 4 quadrant values, then independently threshold each dot.
// This unlocks all 256 braille patterns for precise edge rendering.

/// Maps spatial brightness to a braille character using 8-position sampling.
///
/// Interpolates the 4 quadrant brightness values (0.0-1.0) to 8 positions
/// matching the 2×4 braille dot layout, thresholds each dot independently,
/// and returns the corresponding braille character (U+2800-U+28FF).
pub fn map_shape_braille(tl: f32, tr: f32, bl: f32, br: f32, threshold: f32) -> char {
    // Interpolate 8 positions across the 2×4 braille grid.
    // Row weights: row0=1.0/0.0, row1=0.67/0.33, row2=0.33/0.67, row3=0.0/1.0
    // (fraction of top vs bottom quadrant contribution)
    let samples = [
        tl,                    // row 0, left  (dot 1)
        tr,                    // row 0, right (dot 4)
        tl * 0.67 + bl * 0.33, // row 1, left  (dot 2)
        tr * 0.67 + br * 0.33, // row 1, right (dot 5)
        tl * 0.33 + bl * 0.67, // row 2, left  (dot 3)
        tr * 0.33 + br * 0.67, // row 2, right (dot 6)
        bl,                    // row 3, left  (dot 7)
        br,                    // row 3, right (dot 8)
    ];

    // Build the 8-bit braille dot pattern.
    // Braille encoding: dots 1-3 are bits 0-2 (left column, top to bottom),
    // dots 4-6 are bits 3-5 (right column), dots 7-8 are bits 6-7 (bottom row).
    let mut pattern: u8 = 0;
    if samples[0] > threshold {
        pattern |= 0x01;
    } // dot 1 (row 0, left)
    if samples[2] > threshold {
        pattern |= 0x02;
    } // dot 2 (row 1, left)
    if samples[4] > threshold {
        pattern |= 0x04;
    } // dot 3 (row 2, left)
    if samples[1] > threshold {
        pattern |= 0x08;
    } // dot 4 (row 0, right)
    if samples[3] > threshold {
        pattern |= 0x10;
    } // dot 5 (row 1, right)
    if samples[5] > threshold {
        pattern |= 0x20;
    } // dot 6 (row 2, right)
    if samples[6] > threshold {
        pattern |= 0x40;
    } // dot 7 (row 3, left)
    if samples[7] > threshold {
        pattern |= 0x80;
    } // dot 8 (row 3, right)

    char::from_u32(0x2800 + pattern as u32).unwrap_or(' ')
}

// ===== Sculpted Outline Rendering =====
//
// Used by the Sculpted charset mode for cells at the outline/edge of
// shapes. Interior cells use solid blocks (▀▄█) while outline cells
// use this function to select shape-aware characters.
//
// Characters used:
// - Quadrant blocks (▘▝▖▗) for single-corner shapes
// - Half blocks (▀▄▌▐) for straight edges
// - Triangle fills (◢◣◤◥) for diagonal contours
// - Full block (█) for fully filled cells

/// Selects an outline character based on quadrant brightness distribution.
///
/// Uses quadrant blocks for corners, half blocks for straight edges, and
/// triangle fills (◢◣◤◥) for diagonal contours. Used by Sculpted mode
/// for cells at the boundary of shapes.
pub fn map_sculpted_outline(tl: f32, tr: f32, bl: f32, br: f32) -> char {
    use crate::config_defaults::threshold;

    let tl_on = tl > threshold::DEFAULT_SCULPTED_OUTLINE_THRESHOLD;
    let tr_on = tr > threshold::DEFAULT_SCULPTED_OUTLINE_THRESHOLD;
    let bl_on = bl > threshold::DEFAULT_SCULPTED_OUTLINE_THRESHOLD;
    let br_on = br > threshold::DEFAULT_SCULPTED_OUTLINE_THRESHOLD;

    let index =
        (tl_on as u32) | ((tr_on as u32) << 1) | ((bl_on as u32) << 2) | ((br_on as u32) << 3);

    match index {
        0x0 => ' ',
        0x1 => '\u{2598}', // ▘ TL
        0x2 => '\u{259D}', // ▝ TR
        0x3 => '\u{2580}', // ▀ top half
        0x4 => '\u{2596}', // ▖ BL
        0x5 => '\u{258C}', // ▌ left half
        0x6 => '\u{25E3}', // ◣ lower-left triangle (TR+BL diagonal)
        0x7 => '\u{25E4}', // ◤ upper-left triangle (TL+TR+BL → missing BR)
        0x8 => '\u{2597}', // ▗ BR
        0x9 => '\u{25E2}', // ◢ lower-right triangle (TL+BR diagonal)
        0xA => '\u{2590}', // ▐ right half
        0xB => '\u{25E5}', // ◥ upper-right triangle (TL+TR+BR → missing BL)
        0xC => '\u{2584}', // ▄ bottom half
        0xD => '\u{25E3}', // ◣ lower-left triangle (TL+BL+BR → missing TR)
        0xE => '\u{25E2}', // ◢ lower-right triangle (TR+BL+BR → missing TL)
        0xF => '\u{2588}', // █ full
        _ => ' ',
    }
}

/// Returns the number of distinct brightness levels supported by the charset.
pub fn charset_level_count(charset: Charset) -> usize {
    use crate::config_defaults::charset_levels;
    match charset {
        Charset::HalfBlock => charset_levels::HALF_BLOCK,
        Charset::HalfBlockDual => 256, // Continuous color; effectively unlimited levels
        Charset::Ascii => charset_levels::ASCII,
        Charset::Braille => charset_levels::BRAILLE,
        Charset::Quadrant => charset_levels::QUADRANT, // 2^4 combinations of 4 quadrants
        Charset::Shade => charset_levels::SHADE,       // space, ░, ▒, ▓, █
        Charset::Points => charset_levels::POINTS,     // space or ▪
        Charset::Sculpted => charset_levels::SCULPTED, // 16 quadrant patterns + graduated fills
        Charset::CustomAscii(ref chars) => chars.len().max(2), // At least 2 levels
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
        assert_eq!(Charset::from_args(&args), Charset::HalfBlockDual);
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
    fn test_map_quadrant_basic() {
        assert_eq!(map_quadrant(0.0, 0.0, 0.0, 0.0, 0.05), ' ');
        assert_eq!(map_quadrant(1.0, 0.0, 0.0, 0.0, 0.05), '\u{2598}');
        assert_eq!(map_quadrant(0.0, 1.0, 0.0, 0.0, 0.05), '\u{259D}');
        assert_eq!(map_quadrant(1.0, 1.0, 0.0, 0.0, 0.05), '\u{2580}');
        assert_eq!(map_quadrant(1.0, 1.0, 1.0, 1.0, 0.05), '\u{2588}');
    }

    #[test]
    fn test_map_quadrant_threshold() {
        assert_eq!(map_quadrant(0.04, 0.04, 0.04, 0.04, 0.05), ' ');
        assert_eq!(map_quadrant(0.06, 0.0, 0.0, 0.0, 0.05), '\u{2598}');
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
    fn test_charset_from_args_quadrant() {
        let args = Args {
            quadrant: true,
            ..Default::default()
        };
        assert_eq!(Charset::from_args(&args), Charset::Quadrant);
    }

    #[test]
    fn test_charset_from_args_custom() {
        let args = Args {
            ascii_chars: Some("abc".to_string()),
            ..Default::default()
        };
        assert!(matches!(Charset::from_args(&args), Charset::CustomAscii(_)));
    }

    #[test]
    fn test_from_custom_string_sorting() {
        let charset = Charset::from_custom_string("@.");
        if let Charset::CustomAscii(chars) = charset {
            assert_eq!(chars[0], '.');
            assert_eq!(chars[1], '@');
        } else {
            panic!("Expected CustomAscii");
        }
    }

    #[test]
    fn test_map_brightness_quadrant() {
        assert_eq!(map_brightness(0.0, None, Charset::Quadrant), ' ');
        assert_eq!(map_brightness(1.0, None, Charset::Quadrant), '\u{1FB0E}');
    }

    #[test]
    fn test_map_brightness_custom() {
        let charset = Charset::CustomAscii(vec!['a', 'b', 'c']);
        assert_eq!(map_brightness(0.0, None, charset.clone()), 'a');
        assert_eq!(map_brightness(1.0, None, charset.clone()), 'c');
        assert_eq!(map_brightness(0.5, None, charset.clone()), 'b');
    }

    #[test]
    fn test_charset_level_count_extended() {
        assert_eq!(charset_level_count(Charset::Quadrant), 16);
        assert_eq!(charset_level_count(Charset::CustomAscii(vec!['a', 'b'])), 2);
    }

    // ===== Shade Mode Tests =====

    #[test]
    fn test_map_shade_levels() {
        assert_eq!(map_shade(0.0), ' ');
        assert_eq!(map_shade(0.25), '░');
        assert_eq!(map_shade(0.5), '▒');
        assert_eq!(map_shade(0.75), '▓');
        assert_eq!(map_shade(1.0), '█');
    }

    #[test]
    fn test_map_shade_clamping() {
        assert_eq!(map_shade(-0.5), ' ');
        assert_eq!(map_shade(1.5), '█');
    }

    #[test]
    fn test_map_shade_boundaries() {
        // Test boundary values for each shade level
        // Formula: index = (brightness * 4).round()
        // Space: index 0 (brightness 0.0-0.125)
        // ░: index 1 (brightness ~0.125-0.375)
        // ▒: index 2 (brightness ~0.375-0.625)
        // ▓: index 3 (brightness ~0.625-0.875)
        // █: index 4 (brightness ~0.875-1.0)
        assert_eq!(map_shade(0.1), ' '); // 0.1 * 4 = 0.4, rounds to 0
        assert_eq!(map_shade(0.15), '░'); // 0.15 * 4 = 0.6, rounds to 1
        assert_eq!(map_shade(0.4), '▒'); // 0.4 * 4 = 1.6, rounds to 2
        assert_eq!(map_shade(0.6), '▒'); // 0.6 * 4 = 2.4, rounds to 2
        assert_eq!(map_shade(0.7), '▓'); // 0.7 * 4 = 2.8, rounds to 3
        assert_eq!(map_shade(0.9), '█'); // 0.9 * 4 = 3.6, rounds to 4
    }

    // ===== Point Grid Tests =====

    #[test]
    fn test_map_point_threshold() {
        assert_eq!(map_point(0.1, 0.15), ' ');
        assert_eq!(map_point(0.2, 0.15), '▪');
        assert_eq!(map_point(1.0, 0.15), '▪');
    }

    #[test]
    fn test_map_point_at_threshold() {
        // Exactly at threshold should return space (uses > not >=)
        assert_eq!(map_point(0.15, 0.15), ' ');
        // Just above threshold
        assert_eq!(map_point(0.151, 0.15), '▪');
    }

    #[test]
    fn test_map_point_zero_threshold() {
        // With zero threshold, any positive brightness shows a point
        assert_eq!(map_point(0.0, 0.0), ' ');
        assert_eq!(map_point(0.001, 0.0), '▪');
    }

    // ===== Charset::from_args Tests =====

    #[test]
    fn test_charset_from_args_shade() {
        let args = Args {
            shade: true,
            ..Default::default()
        };
        assert_eq!(Charset::from_args(&args), Charset::Shade);
    }

    #[test]
    fn test_charset_from_args_points() {
        let args = Args {
            points: true,
            ..Default::default()
        };
        assert_eq!(Charset::from_args(&args), Charset::Points);
    }

    #[test]
    fn test_charset_from_args_shade_priority() {
        // Shade should take priority over braille and ascii
        let args = Args {
            shade: true,
            braille: true,
            ascii: true,
            ..Default::default()
        };
        assert_eq!(Charset::from_args(&args), Charset::Shade);
    }

    #[test]
    fn test_charset_from_args_quadrant_priority_over_shade() {
        // Quadrant should take priority over shade
        let args = Args {
            quadrant: true,
            shade: true,
            ..Default::default()
        };
        assert_eq!(Charset::from_args(&args), Charset::Quadrant);
    }

    // ===== charset_level_count Tests =====

    #[test]
    fn test_charset_level_count_shade() {
        assert_eq!(charset_level_count(Charset::Shade), 5);
    }

    #[test]
    fn test_charset_level_count_points() {
        assert_eq!(charset_level_count(Charset::Points), 2);
    }

    // ===== map_brightness with Shade/Points =====

    #[test]
    fn test_map_brightness_shade() {
        assert_eq!(map_brightness(0.0, None, Charset::Shade), ' ');
        assert_eq!(map_brightness(0.5, None, Charset::Shade), '▒');
        assert_eq!(map_brightness(1.0, None, Charset::Shade), '█');
    }

    #[test]
    fn test_map_brightness_shade_with_bottom() {
        // When bottom is provided, should average top and bottom
        assert_eq!(map_brightness(1.0, Some(0.0), Charset::Shade), '▒'); // avg = 0.5
        assert_eq!(map_brightness(0.5, Some(0.5), Charset::Shade), '▒'); // avg = 0.5
        assert_eq!(map_brightness(1.0, Some(1.0), Charset::Shade), '█'); // avg = 1.0
    }

    #[test]
    fn test_map_brightness_points() {
        assert_eq!(map_brightness(0.0, None, Charset::Points), ' ');
        assert_eq!(map_brightness(0.1, None, Charset::Points), ' '); // below threshold
        assert_eq!(map_brightness(0.2, None, Charset::Points), '▪'); // above threshold
        assert_eq!(map_brightness(1.0, None, Charset::Points), '▪');
    }

    #[test]
    fn test_map_brightness_points_with_bottom() {
        // When bottom is provided, should average top and bottom
        assert_eq!(map_brightness(0.2, Some(0.1), Charset::Points), ' '); // avg = 0.15, at threshold
        assert_eq!(map_brightness(0.3, Some(0.1), Charset::Points), '▪'); // avg = 0.2, above threshold
    }

    #[test]
    fn test_half_block_dual_both_dark() {
        assert_eq!(map_half_block_dual(0.0, 0.0, 0.05), ' ');
        assert_eq!(map_half_block_dual(0.04, 0.04, 0.05), ' ');
    }

    #[test]
    fn test_half_block_dual_top_only() {
        assert_eq!(map_half_block_dual(0.5, 0.0, 0.05), '▀');
        assert_eq!(map_half_block_dual(1.0, 0.03, 0.05), '▀');
    }

    #[test]
    fn test_half_block_dual_bottom_only() {
        assert_eq!(map_half_block_dual(0.0, 0.5, 0.05), '▄');
        assert_eq!(map_half_block_dual(0.03, 1.0, 0.05), '▄');
    }

    #[test]
    fn test_half_block_dual_both_bright() {
        assert_eq!(map_half_block_dual(0.5, 0.5, 0.05), '▀');
        assert_eq!(map_half_block_dual(1.0, 0.1, 0.05), '▀');
        assert_eq!(map_half_block_dual(0.1, 1.0, 0.05), '▀');
    }

    #[test]
    fn test_half_block_dual_threshold_edge() {
        assert_eq!(map_half_block_dual(0.05, 0.0, 0.05), ' ');
        assert_eq!(map_half_block_dual(0.051, 0.0, 0.05), '▀');
        assert_eq!(map_half_block_dual(0.0, 0.05, 0.05), ' ');
        assert_eq!(map_half_block_dual(0.0, 0.051, 0.05), '▄');
    }

    #[test]
    fn test_charset_from_args_half_block_dual() {
        let args = Args {
            half_block_dual: true,
            ..Default::default()
        };
        assert_eq!(Charset::from_args(&args), Charset::HalfBlockDual);
    }

    #[test]
    fn test_charset_level_count_half_block_dual() {
        assert_eq!(charset_level_count(Charset::HalfBlockDual), 256);
    }

    #[test]
    fn test_map_brightness_half_block_dual_uses_half_block_chars() {
        // HalfBlockDual shares the same char array as HalfBlock for map_brightness
        assert_eq!(map_brightness(0.0, None, Charset::HalfBlockDual), ' ');
        assert_eq!(
            map_brightness(1.0, None, Charset::HalfBlockDual),
            '\u{2588}'
        );
    }

    // ===== Shape Vector ASCII Tests =====

    #[test]
    fn test_shape_ascii_empty_returns_space() {
        assert_eq!(map_shape_ascii(0.0, 0.0, 0.0, 0.0, 1.5), ' ');
    }

    #[test]
    fn test_shape_ascii_uniform_returns_dense_char() {
        // Uniform brightness should match a uniform/dense character
        let ch = map_shape_ascii(0.8, 0.8, 0.8, 0.8, 1.0);
        // Should be a heavy fill char, not a directional one
        assert!(
            "#@HMWOn*".contains(ch),
            "Uniform brightness should produce a dense fill character, got '{ch}'"
        );
    }

    #[test]
    fn test_shape_ascii_diagonal_forward_slash() {
        // Bright in top-right and bottom-left = forward slash pattern
        let ch = map_shape_ascii(0.0, 0.8, 0.8, 0.0, 1.0);
        assert_eq!(
            ch, '/',
            "Top-right + bottom-left diagonal should produce '/'"
        );
    }

    #[test]
    fn test_shape_ascii_diagonal_backslash() {
        // Bright in top-left and bottom-right = backslash pattern
        let ch = map_shape_ascii(0.8, 0.0, 0.0, 0.8, 1.0);
        assert_eq!(
            ch, '\\',
            "Top-left + bottom-right diagonal should produce '\\'"
        );
    }

    #[test]
    fn test_shape_ascii_top_heavy() {
        // Bright on top, dark on bottom
        let ch = map_shape_ascii(0.8, 0.8, 0.0, 0.0, 1.0);
        // Should pick a top-heavy character like ^, ", T, Y
        assert!(
            "^\"TY".contains(ch),
            "Top-heavy pattern should produce a top-heavy character, got '{ch}'"
        );
    }

    #[test]
    fn test_shape_ascii_bottom_heavy() {
        // Dark on top, bright on bottom
        let ch = map_shape_ascii(0.0, 0.0, 0.8, 0.8, 1.0);
        // Should pick a bottom-leaning character. Due to 2x2→2x3 interpolation,
        // the middle row also gets weight, so chars like ; (mid+bottom dots)
        // are also valid matches alongside v, _, u, J, L
        assert!(
            "v_u;JL:".contains(ch),
            "Bottom-heavy pattern should produce a bottom-leaning character, got '{ch}'"
        );
    }

    #[test]
    fn test_shape_ascii_left_heavy() {
        // Bright on left, dark on right
        let ch = map_shape_ascii(0.8, 0.0, 0.8, 0.0, 1.0);
        // Should pick a left-heavy character like [, (, {
        assert!(
            "[({BDLt".contains(ch),
            "Left-heavy pattern should produce a left-side character, got '{ch}'"
        );
    }

    #[test]
    fn test_shape_ascii_right_heavy() {
        // Dark on left, bright on right
        let ch = map_shape_ascii(0.0, 0.8, 0.0, 0.8, 1.0);
        // Should pick a right-heavy character like ], ), }
        assert!(
            "])}.!i".contains(ch),
            "Right-heavy pattern should produce a right-side character, got '{ch}'"
        );
    }

    #[test]
    fn test_shape_ascii_horizontal_band() {
        // Bright in middle row only (approximated: equal TL=BL, TR=BR)
        // When all 4 corners are equal, the middle row = average of all = same
        // Instead, test with slight middle emphasis
        let ch = map_shape_ascii(0.1, 0.1, 0.1, 0.1, 1.0);
        // Low uniform brightness should still produce some character (not space)
        assert_ne!(
            ch, ' ',
            "Low but nonzero uniform brightness should not be space"
        );
    }

    #[test]
    fn test_shape_ascii_contrast_enhancement() {
        // With contrast enhancement, characters should still be valid
        let ch_low = map_shape_ascii(0.0, 0.8, 0.8, 0.0, 1.0);
        let ch_high = map_shape_ascii(0.0, 0.8, 0.8, 0.0, 3.0);
        // Both should produce a valid character (not space or panic)
        assert_ne!(ch_low, ' ');
        assert_ne!(ch_high, ' ');
    }

    #[test]
    fn test_shape_ascii_near_zero_not_space() {
        // Very low but nonzero values should still produce a character
        let ch = map_shape_ascii(0.02, 0.0, 0.0, 0.0, 1.0);
        // At 0.02 max, should be above the 0.01 threshold
        assert_ne!(ch, ' ', "Brightness 0.02 should produce a character");
    }

    #[test]
    fn test_shape_ascii_just_below_threshold() {
        // Below the 0.01 threshold should return space
        let ch = map_shape_ascii(0.005, 0.005, 0.005, 0.005, 1.0);
        assert_eq!(ch, ' ', "Brightness below 0.01 threshold should be space");
    }

    #[test]
    fn test_enhance_contrast_uniform_unchanged() {
        // A uniform vector should remain uniform after enhancement
        let mut v = [0.5, 0.5, 0.5, 0.5, 0.5, 0.5];
        enhance_contrast(&mut v, 2.0);
        for val in &v {
            assert!(
                (*val - 0.5).abs() < 1e-5,
                "Uniform vector should be unchanged by contrast, got {val}"
            );
        }
    }

    #[test]
    fn test_enhance_contrast_zero_unchanged() {
        let mut v = [0.0, 0.0, 0.0, 0.0, 0.0, 0.0];
        enhance_contrast(&mut v, 2.0);
        for val in &v {
            assert!(val.abs() < 1e-6, "Zero vector should stay zero");
        }
    }

    #[test]
    fn test_enhance_contrast_amplifies_differences() {
        let mut v = [1.0, 0.5, 0.0, 0.0, 0.0, 0.0];
        enhance_contrast(&mut v, 2.0);
        // After contrast with exponent 2: normalized [1, 0.5, 0, 0, 0, 0]
        // raised to power 2: [1, 0.25, 0, 0, 0, 0], then * max(1.0)
        assert!((v[0] - 1.0).abs() < 1e-5, "Max value should stay at 1.0");
        assert!(
            (v[1] - 0.25).abs() < 1e-5,
            "Half value should become 0.25 with exponent 2"
        );
    }

    // ===== Shape Braille Tests =====

    #[test]
    fn test_shape_braille_empty() {
        assert_eq!(map_shape_braille(0.0, 0.0, 0.0, 0.0, 0.05), '\u{2800}');
    }

    #[test]
    fn test_shape_braille_full() {
        // All quadrants bright → all 8 dots on
        assert_eq!(map_shape_braille(1.0, 1.0, 1.0, 1.0, 0.05), '\u{28FF}');
    }

    #[test]
    fn test_shape_braille_top_only() {
        // Only top half bright → dots 1,2,3 (left) and 4,5,6 (right) on
        // rows 0,1 fully on (from tl/tr), row 2 partially on (0.33*tl), row 3 off
        let ch = map_shape_braille(1.0, 1.0, 0.0, 0.0, 0.05);
        // Dots 1,2 on left (tl=1.0, 0.67*tl=0.67), dot 3 (0.33*tl=0.33 > 0.05)
        // Dots 4,5 on right (tr=1.0, 0.67*tr=0.67), dot 6 (0.33*tr=0.33 > 0.05)
        // Dots 7,8 off (bl=br=0)
        assert_eq!(ch, '\u{283F}'); // all top 6 dots on, bottom 2 off
    }

    #[test]
    fn test_shape_braille_bottom_only() {
        // Only bottom half bright → bottom dots on
        let ch = map_shape_braille(0.0, 0.0, 1.0, 1.0, 0.05);
        // Row 0: tl=0 (off), Row 1: 0.67*0+0.33*1=0.33 (on)
        // Row 2: 0.33*0+0.67*1=0.67 (on), Row 3: bl=1 (on)
        // Left dots: 1=off, 2=on, 3=on, 7=on → bits 1,2,6 = 0x46
        // Right dots: 4=off, 5=on, 6=on, 8=on → bits 4,5,7 = 0xB0
        // Pattern = 0x46 | 0xB0 = 0xF6 -- hmm let me recalculate
        // dot 2 = bit 1, dot 3 = bit 2, dot 7 = bit 6: 0b01000110 = 0x46
        // dot 5 = bit 4, dot 6 = bit 5, dot 8 = bit 7: 0b10110000 = 0xB0
        // 0x46 | 0xB0 = 0xF6
        assert_eq!(ch, char::from_u32(0x2800 + 0xF6).unwrap());
    }

    #[test]
    fn test_shape_braille_left_only() {
        // Only left half bright
        let ch = map_shape_braille(1.0, 0.0, 1.0, 0.0, 0.05);
        // Left column all on (dots 1,2,3,7 = bits 0,1,2,6 = 0x47)
        // Right column all off
        assert_eq!(ch, char::from_u32(0x2800 + 0x47).unwrap());
    }

    #[test]
    fn test_shape_braille_right_only() {
        // Only right half bright
        let ch = map_shape_braille(0.0, 1.0, 0.0, 1.0, 0.05);
        // Right column all on (dots 4,5,6,8 = bits 3,4,5,7 = 0xB8)
        assert_eq!(ch, char::from_u32(0x2800 + 0xB8).unwrap());
    }

    #[test]
    fn test_shape_braille_diagonal_tl_br() {
        // Top-left and bottom-right bright = diagonal pattern
        let ch = map_shape_braille(1.0, 0.0, 0.0, 1.0, 0.05);
        // TL quadrant: tl=1.0, tr=0.0, bl=0.0, br=1.0
        // Left col: row0=1.0(on), row1=0.67(on), row2=0.33(on), row3=0(off)
        // Right col: row0=0(off), row1=0.33(on), row2=0.67(on), row3=1(on)
        // Left: dots 1,2,3 on, 7 off → bits 0,1,2 = 0x07
        // Right: dots 4 off, 5,6 on, 8 on → bits 4,5,7 = 0xB0
        assert_eq!(ch, char::from_u32(0x2800 + 0x07 + 0xB0).unwrap());
    }

    #[test]
    fn test_shape_braille_threshold() {
        // Values below threshold produce no dots
        assert_eq!(map_shape_braille(0.04, 0.04, 0.04, 0.04, 0.05), '\u{2800}');
        // Values above threshold produce dots
        assert_ne!(map_shape_braille(0.06, 0.06, 0.06, 0.06, 0.05), '\u{2800}');
    }

    #[test]
    fn test_shape_braille_produces_more_patterns_than_old() {
        // Verify that different quadrant distributions produce different braille patterns
        let patterns: std::collections::HashSet<char> = [
            (1.0, 0.0, 0.0, 0.0),
            (0.0, 1.0, 0.0, 0.0),
            (0.0, 0.0, 1.0, 0.0),
            (0.0, 0.0, 0.0, 1.0),
            (1.0, 1.0, 0.0, 0.0),
            (0.0, 0.0, 1.0, 1.0),
            (1.0, 0.0, 1.0, 0.0),
            (0.0, 1.0, 0.0, 1.0),
            (1.0, 0.0, 0.0, 1.0),
            (0.0, 1.0, 1.0, 0.0),
            (1.0, 1.0, 1.0, 1.0),
        ]
        .iter()
        .map(|(tl, tr, bl, br)| map_shape_braille(*tl, *tr, *bl, *br, 0.05))
        .collect();
        // Should produce at least 6 distinct patterns (old approach produced ~4)
        assert!(
            patterns.len() >= 6,
            "Expected at least 6 distinct braille patterns, got {}",
            patterns.len()
        );
    }

    // ===== Sculpted Outline Tests =====

    #[test]
    fn test_sculpted_outline_empty() {
        assert_eq!(map_sculpted_outline(0.0, 0.0, 0.0, 0.0), ' ');
    }

    #[test]
    fn test_sculpted_outline_full() {
        assert_eq!(map_sculpted_outline(1.0, 1.0, 1.0, 1.0), '\u{2588}');
    }

    #[test]
    fn test_sculpted_outline_top_half() {
        let ch = map_sculpted_outline(0.9, 0.9, 0.0, 0.0);
        assert_eq!(ch, '\u{2580}', "Top-heavy should produce ▀, got '{ch}'");
    }

    #[test]
    fn test_sculpted_outline_top_half_dim() {
        // Dim top → still ▀ (no graduated fills)
        let ch = map_sculpted_outline(0.1, 0.1, 0.0, 0.0);
        assert_eq!(ch, '\u{2580}', "Dim top should produce ▀, got '{ch}'");
    }

    #[test]
    fn test_sculpted_outline_bottom_half() {
        let ch = map_sculpted_outline(0.0, 0.0, 0.9, 0.9);
        assert_eq!(ch, '\u{2584}', "Bottom half should produce ▄, got '{ch}'");
    }

    #[test]
    fn test_sculpted_outline_left_half() {
        let ch = map_sculpted_outline(0.9, 0.0, 0.9, 0.0);
        assert_eq!(ch, '\u{258C}', "Left half should produce ▌, got '{ch}'");
    }

    #[test]
    fn test_sculpted_outline_right_half() {
        let ch = map_sculpted_outline(0.0, 0.9, 0.0, 0.9);
        assert_eq!(ch, '\u{2590}', "Right-heavy should produce ▐, got '{ch}'");
    }

    #[test]
    fn test_sculpted_outline_tl_quadrant() {
        let ch = map_sculpted_outline(0.9, 0.0, 0.0, 0.0);
        assert_eq!(ch, '\u{2598}', "Top-left only should produce ▘, got '{ch}'");
    }

    #[test]
    fn test_sculpted_outline_br_quadrant() {
        let ch = map_sculpted_outline(0.0, 0.0, 0.0, 0.9);
        assert_eq!(
            ch, '\u{2597}',
            "Bottom-right only should produce ▗, got '{ch}'"
        );
    }

    #[test]
    fn test_sculpted_outline_diagonal_backslash() {
        // TL+BR diagonal → ◢ (lower-right triangle fill)
        let ch = map_sculpted_outline(0.9, 0.0, 0.0, 0.9);
        assert_eq!(
            ch, '\u{25E2}',
            "TL+BR diagonal should produce ◢, got '{ch}'"
        );
    }

    #[test]
    fn test_sculpted_outline_diagonal_forwardslash() {
        // TR+BL diagonal → ◣ (lower-left triangle fill)
        let ch = map_sculpted_outline(0.0, 0.9, 0.9, 0.0);
        assert_eq!(
            ch, '\u{25E3}',
            "TR+BL diagonal should produce ◣, got '{ch}'"
        );
    }

    #[test]
    fn test_sculpted_outline_three_quarter_tl_tr_bl() {
        // TL+TR+BL → ◤ upper-left triangle (missing BR)
        let ch = map_sculpted_outline(0.9, 0.9, 0.9, 0.0);
        assert_eq!(ch, '\u{25E4}', "TL+TR+BL should produce ◤, got '{ch}'");
    }

    #[test]
    fn test_sculpted_outline_three_quarter_tl_tr_br() {
        // TL+TR+BR → ◥ upper-right triangle (missing BL)
        let ch = map_sculpted_outline(0.9, 0.9, 0.0, 0.9);
        assert_eq!(ch, '\u{25E5}', "TL+TR+BR should produce ◥, got '{ch}'");
    }

    #[test]
    fn test_sculpted_outline_three_quarter_tl_bl_br() {
        // TL+BL+BR → ◣ lower-left triangle (missing TR)
        let ch = map_sculpted_outline(0.9, 0.0, 0.9, 0.9);
        assert_eq!(ch, '\u{25E3}', "TL+BL+BR should produce ◣, got '{ch}'");
    }

    #[test]
    fn test_sculpted_outline_three_quarter_tr_bl_br() {
        // TR+BL+BR → ◢ lower-right triangle (missing TL)
        let ch = map_sculpted_outline(0.0, 0.9, 0.9, 0.9);
        assert_eq!(ch, '\u{25E2}', "TR+BL+BR should produce ◢, got '{ch}'");
    }

    #[test]
    fn test_sculpted_outline_near_zero() {
        assert_eq!(
            map_sculpted_outline(0.005, 0.005, 0.005, 0.005),
            ' ',
            "Below threshold should be space"
        );
    }

    #[test]
    fn test_sculpted_outline_produces_distinct_patterns() {
        let patterns: std::collections::HashSet<char> = [
            (0.9, 0.0, 0.0, 0.0), // TL only
            (0.0, 0.9, 0.0, 0.0), // TR only
            (0.0, 0.0, 0.9, 0.0), // BL only
            (0.0, 0.0, 0.0, 0.9), // BR only
            (0.9, 0.9, 0.0, 0.0), // top half
            (0.0, 0.0, 0.9, 0.9), // bottom half
            (0.9, 0.0, 0.9, 0.0), // left half
            (0.0, 0.9, 0.0, 0.9), // right half
            (0.9, 0.0, 0.0, 0.9), // TL+BR diagonal
            (0.0, 0.9, 0.9, 0.0), // TR+BL diagonal
            (0.9, 0.9, 0.9, 0.9), // full
        ]
        .iter()
        .map(|(tl, tr, bl, br)| map_sculpted_outline(*tl, *tr, *bl, *br))
        .collect();
        assert!(
            patterns.len() >= 8,
            "Expected at least 8 distinct outline patterns, got {}",
            patterns.len()
        );
    }

    #[test]
    fn glyph_config_default_is_identity() {
        let g = GlyphConfig::default();
        assert_eq!(g.selection, None);
        assert_eq!(
            g.edge_threshold,
            crate::config_defaults::glyph_consts::DEFAULT_GLYPH_EDGE_THRESHOLD
        );
    }

    #[test]
    fn sobel_flat_field_is_none() {
        // Uniform brightness → zero gradient → below any positive threshold.
        let n = [0.5_f32; 9];
        assert_eq!(sobel_edge_glyph(&n, 0.05), None);
    }

    #[test]
    fn sobel_below_threshold_is_none() {
        // Tiny step, high threshold → None.
        let n = [0.0, 0.0, 0.0, 0.0, 0.01, 0.0, 0.0, 0.0, 0.0];
        assert_eq!(sobel_edge_glyph(&n, 0.5), None);
    }

    #[test]
    fn sobel_vertical_edge_is_pipe() {
        // Left column dark, right column bright → horizontal gradient → vertical edge.
        let n = [0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 0.0, 0.0, 1.0];
        assert_eq!(sobel_edge_glyph(&n, 0.1), Some('|'));
    }

    #[test]
    fn sobel_horizontal_edge_is_dash() {
        // Top dark, bottom bright → vertical gradient → horizontal edge.
        let n = [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0];
        assert_eq!(sobel_edge_glyph(&n, 0.1), Some('-'));
    }

    #[test]
    fn sobel_diagonal_edges() {
        // Gradient pointing down-right → "/" edge.
        let down_right = [0.0, 0.0, 0.0, 0.0, 0.5, 1.0, 0.0, 1.0, 1.0];
        assert_eq!(sobel_edge_glyph(&down_right, 0.1), Some('/'));
        // Gradient pointing down-left → "\" edge.
        let down_left = [0.0, 0.0, 0.0, 1.0, 0.5, 0.0, 1.0, 1.0, 0.0];
        assert_eq!(sobel_edge_glyph(&down_left, 0.1), Some('\\'));
    }
}
