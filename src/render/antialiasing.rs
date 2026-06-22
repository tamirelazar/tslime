//! Color anti-aliasing for subcell-shape charsets.
//!
//! Subcell-shape charsets (Braille/Quadrant/HalfBlock/Ascii) render shape at
//! 2–4× resolution but carry one fg color per terminal cell, so a thin diagonal
//! vein staircases in color at cell frequency. Low-passing the per-cell color
//! field washes that into a gradient while the glyph (shape) stays crisp.

use serde::{Deserialize, Serialize};

use crate::render::charset::Charset;

/// Per-charset color anti-aliasing strength.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AaStrength {
    /// No color blur — raw per-cell color (today's behavior).
    #[default]
    Off,
    /// Weighted-center 3×3: `0.5·self + 0.5·mean(neighbors)`. Gentle.
    Subtle,
    /// Uniform 3×3 box blur. Strongest wash.
    Strong,
}

impl AaStrength {
    /// Cycle Off → Subtle → Strong → Off.
    pub fn cycle(self) -> Self {
        match self {
            AaStrength::Off => AaStrength::Subtle,
            AaStrength::Subtle => AaStrength::Strong,
            AaStrength::Strong => AaStrength::Off,
        }
    }

    /// Title-case label for UI/notifications.
    pub fn as_label(self) -> &'static str {
        match self {
            AaStrength::Off => "Off",
            AaStrength::Subtle => "Subtle",
            AaStrength::Strong => "Strong",
        }
    }

    /// Lowercase token for CLI/config serialization.
    pub fn as_cli(self) -> &'static str {
        match self {
            AaStrength::Off => "off",
            AaStrength::Subtle => "subtle",
            AaStrength::Strong => "strong",
        }
    }

    /// Parse a CLI/config token (case-insensitive). `None` if unrecognized.
    pub fn parse_cli(s: &str) -> Option<Self> {
        match s.trim().to_ascii_lowercase().as_str() {
            "off" => Some(AaStrength::Off),
            "subtle" => Some(AaStrength::Subtle),
            "strong" => Some(AaStrength::Strong),
            _ => None,
        }
    }
}

impl std::str::FromStr for AaStrength {
    type Err = String;

    /// Parse a CLI token, rejecting unrecognized values with a helpful message
    /// (used by clap so bad `--color-aa` input errors instead of silently
    /// falling back).
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse_cli(s).ok_or_else(|| {
            format!("invalid color-aa mode: {s}. Valid options: off, subtle, strong")
        })
    }
}

/// True for charsets whose shape resolution exceeds their color resolution and
/// therefore benefit from color anti-aliasing.
pub fn charset_aa_eligible(charset: &Charset) -> bool {
    matches!(
        charset,
        Charset::Braille | Charset::Quadrant | Charset::HalfBlock | Charset::Ascii
    )
}

/// Low-pass a row-major `width*height` color field with a 3×3 kernel.
///
/// Edge cells average only the neighbors that exist (no wrap, no padding). The
/// shape/glyph path does NOT use this — only the per-cell color does. Allocates
/// exactly one output `Vec<f32>`; no per-cell allocation.
pub fn blur_field(src: &[f32], width: usize, height: usize, strength: AaStrength) -> Vec<f32> {
    if strength == AaStrength::Off || width == 0 || height == 0 {
        return src.to_vec();
    }
    // Invariant: src is exactly the row-major field. With it, idx = y*width+x and
    // every in-range nidx are provably < src.len(), so the inner indexing needs no
    // per-cell bounds check (debug builds catch a caller that violates this).
    debug_assert_eq!(
        src.len(),
        width * height,
        "blur_field: src must be width*height"
    );
    let mut out = vec![0.0_f32; src.len()];
    for y in 0..height {
        for x in 0..width {
            let idx = y * width + x;
            let center = src[idx];
            let mut neighbor_sum = 0.0_f32;
            let mut neighbor_count = 0u32;
            for dy in -1i32..=1 {
                for dx in -1i32..=1 {
                    if dx == 0 && dy == 0 {
                        continue;
                    }
                    let nx = x as i32 + dx;
                    let ny = y as i32 + dy;
                    if nx >= 0 && nx < width as i32 && ny >= 0 && ny < height as i32 {
                        let nidx = ny as usize * width + nx as usize;
                        neighbor_sum += src[nidx];
                        neighbor_count += 1;
                    }
                }
            }
            out[idx] = match strength {
                AaStrength::Strong => {
                    // Uniform mean over self + present neighbors.
                    (center + neighbor_sum) / (neighbor_count as f32 + 1.0)
                }
                AaStrength::Subtle => {
                    let neighbor_avg = if neighbor_count > 0 {
                        neighbor_sum / neighbor_count as f32
                    } else {
                        center
                    };
                    0.5 * center + 0.5 * neighbor_avg
                }
                AaStrength::Off => center,
            };
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::render::charset::Charset;

    #[test]
    fn off_returns_clone() {
        let src = vec![0.1, 0.9, 0.4, 0.6];
        let out = blur_field(&src, 2, 2, AaStrength::Off);
        assert_eq!(out, src);
    }

    #[test]
    fn uniform_field_is_unchanged() {
        let src = vec![0.5_f32; 9];
        let strong = blur_field(&src, 3, 3, AaStrength::Strong);
        for v in strong {
            assert!((v - 0.5).abs() < 1e-6);
        }
    }

    #[test]
    fn strong_spreads_a_center_spike() {
        // 3×3, only the center is lit.
        let mut src = vec![0.0_f32; 9];
        src[4] = 1.0;
        let out = blur_field(&src, 3, 3, AaStrength::Strong);
        // Center is averaged over all 9 neighbors → 1/9.
        assert!((out[4] - (1.0 / 9.0)).abs() < 1e-6);
        // A corner sees 4 neighbors (incl. itself); the spike is one of them → 1/4.
        assert!((out[0] - 0.25).abs() < 1e-6);
    }

    #[test]
    fn subtle_preserves_center_more_than_strong() {
        let mut src = vec![0.0_f32; 9];
        src[4] = 1.0;
        let subtle = blur_field(&src, 3, 3, AaStrength::Subtle);
        let strong = blur_field(&src, 3, 3, AaStrength::Strong);
        // Subtle keeps half the center weight, so its center stays brighter.
        assert!(subtle[4] > strong[4]);
        assert!((subtle[4] - 0.5).abs() < 1e-6); // 0.5*1.0 + 0.5*neighbor_avg(=0)
    }

    #[test]
    fn cycle_wraps() {
        assert_eq!(AaStrength::Off.cycle(), AaStrength::Subtle);
        assert_eq!(AaStrength::Subtle.cycle(), AaStrength::Strong);
        assert_eq!(AaStrength::Strong.cycle(), AaStrength::Off);
    }

    #[test]
    fn parse_cli_roundtrip() {
        for s in [AaStrength::Off, AaStrength::Subtle, AaStrength::Strong] {
            assert_eq!(AaStrength::parse_cli(s.as_cli()), Some(s));
        }
        assert_eq!(AaStrength::parse_cli("STRONG"), Some(AaStrength::Strong));
        assert_eq!(AaStrength::parse_cli("nope"), None);
    }

    #[test]
    fn eligibility() {
        assert!(charset_aa_eligible(&Charset::Braille));
        assert!(charset_aa_eligible(&Charset::Quadrant));
        assert!(charset_aa_eligible(&Charset::HalfBlock));
        assert!(charset_aa_eligible(&Charset::Ascii));
        assert!(!charset_aa_eligible(&Charset::HalfBlockDual));
        assert!(!charset_aa_eligible(&Charset::Sculpted));
        assert!(!charset_aa_eligible(&Charset::Shade));
    }
}
