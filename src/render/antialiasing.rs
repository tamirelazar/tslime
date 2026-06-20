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

/// True for charsets whose shape resolution exceeds their color resolution and
/// therefore benefit from color anti-aliasing.
pub fn charset_aa_eligible(charset: &Charset) -> bool {
    matches!(
        charset,
        Charset::Braille | Charset::Quadrant | Charset::HalfBlock | Charset::Ascii
    )
}

/// Dev A/B: 2-tap blur along the local low-gradient axis. Averages the two
/// neighbors on the axis with the SMALLER intensity difference (i.e. along the
/// vein), preserving the sharp cross-vein edge. Not wired to config — env only.
pub fn blur_field_directional(src: &[f32], width: usize, height: usize) -> Vec<f32> {
    let mut out = src.to_vec();
    for y in 0..height {
        for x in 0..width {
            let idx = y * width + x;
            if idx >= src.len() {
                continue;
            }
            let at = |xx: i32, yy: i32| -> Option<f32> {
                if xx >= 0 && xx < width as i32 && yy >= 0 && yy < height as i32 {
                    src.get(yy as usize * width + xx as usize).copied()
                } else {
                    None
                }
            };
            let c = src[idx];
            let h = match (at(x as i32 - 1, y as i32), at(x as i32 + 1, y as i32)) {
                (Some(l), Some(r)) => Some(((l + r) * 0.5, (l - r).abs())),
                _ => None,
            };
            let v = match (at(x as i32, y as i32 - 1), at(x as i32, y as i32 + 1)) {
                (Some(u), Some(d)) => Some(((u + d) * 0.5, (u - d).abs())),
                _ => None,
            };
            out[idx] = match (h, v) {
                (Some((ha, hd)), Some((va, vd))) => {
                    let along = if hd <= vd { ha } else { va };
                    0.5 * c + 0.5 * along
                }
                (Some((ha, _)), None) => 0.5 * c + 0.5 * ha,
                (None, Some((va, _))) => 0.5 * c + 0.5 * va,
                (None, None) => c,
            };
        }
    }
    out
}

/// Low-pass a row-major `width*height` color field with a 3×3 kernel.
///
/// Edge cells average only the neighbors that exist (no wrap, no padding). The
/// shape/glyph path does NOT use this — only the per-cell color does. Allocates
/// exactly one output `Vec<f32>`; no per-cell allocation.
///
/// When `strength` is `Subtle` and the environment variable
/// `TSLIME_AA_SUBTLE_DIRECTIONAL` is set, delegates to
/// [`blur_field_directional`] for A/B comparison purposes.
pub fn blur_field(src: &[f32], width: usize, height: usize, strength: AaStrength) -> Vec<f32> {
    if strength == AaStrength::Off || width == 0 || height == 0 {
        return src.to_vec();
    }
    // Read env var once here, outside the cell loop, to avoid per-cell syscalls.
    let directional =
        strength == AaStrength::Subtle && std::env::var("TSLIME_AA_SUBTLE_DIRECTIONAL").is_ok();
    if directional {
        return blur_field_directional(src, width, height);
    }
    let mut out = vec![0.0_f32; src.len()];
    for y in 0..height {
        for x in 0..width {
            let idx = y * width + x;
            if idx >= src.len() {
                continue;
            }
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
                        if nidx < src.len() {
                            neighbor_sum += src[nidx];
                            neighbor_count += 1;
                        }
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

    /// Directional kernel should preserve a horizontal ridge's center brightness
    /// better than weighted-center Subtle, because it blurs ALONG the ridge
    /// (horizontally) rather than across it (vertically).
    #[test]
    fn directional_preserves_horizontal_ridge_better_than_weighted_center() {
        // 3 rows × 5 cols. Middle row is the "ridge" (lit), top/bottom are dark.
        // width=5, height=3; middle row = row index 1.
        let width = 5;
        let height = 3;
        let mut src = vec![0.0_f32; width * height];
        for x in 0..width {
            src[width + x] = 1.0; // middle row (y=1) fully lit
        }

        // Weighted-center Subtle: blurs in all 8 directions, so vertical dark
        // neighbors drag the lit row's center cell down significantly.
        let wc = blur_field(&src, width, height, AaStrength::Subtle);
        // Directional: blurs only along the low-gradient axis (horizontal for a
        // horizontal ridge), so the lit row stays brighter at its center.
        let dir = blur_field_directional(&src, width, height);

        // The center cell of the lit row is (x=2, y=1) → idx = width+2 = 7.
        let center_idx = width + 2;
        assert!(
            dir[center_idx] > wc[center_idx],
            "directional ({}) should be brighter than weighted-center ({}) at ridge center",
            dir[center_idx],
            wc[center_idx]
        );
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
