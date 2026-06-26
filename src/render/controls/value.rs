//! Pure value-display widgets: gauge bar, heatmap slider, sparkline.
//!
//! All functions are side-effect-free and produce styled character sequences
//! rather than writing to any output stream.

use crate::render::palette::RgbColor;

/// Renders a filled gauge bar of exactly `width` characters.
///
/// Each character is one of:
/// - `█` (fill) for cells within the current ratio.
/// - `░` (dim) for cells beyond the current ratio.
/// - `│` (tick) at the `def_ratio` position to mark the default value.
///
/// When the tick position coincides with a filled cell, the tick takes
/// precedence so the default is always visible.
pub fn gauge(
    ratio: f32,
    def_ratio: f32,
    width: usize,
    fill: RgbColor,
    hi: RgbColor,
    dim: RgbColor,
) -> Vec<(char, RgbColor)> {
    let filled = (ratio * width as f32).round() as usize;
    let defc = (def_ratio * (width as f32 - 1.0)).round() as usize;
    (0..width)
        .map(|i| {
            if i == defc {
                ('│', hi)
            } else if i < filled {
                ('█', fill)
            } else {
                ('░', dim)
            }
        })
        .collect()
}

/// Renders a heatmap slider of exactly `width` characters.
///
/// Characters produced:
/// - `●` knob at the filled position (current value).
/// - `│` notch at `def_ratio` (default value marker).
/// - `━` fill for cells between 0 and the knob.
/// - `━` dim for cells beyond the knob.
///
/// When `truecolor` is `true`, filled cells use a green→red gradient keyed on
/// position within the track (matches the prototype's value-magnitude heatmap).
/// When `truecolor` is `false`, filled cells use the solid `accent` colour so
/// the widget degrades gracefully on 256-colour terminals.
pub fn heatmap_slider(
    ratio: f32,
    def_ratio: f32,
    width: usize,
    truecolor: bool,
    accent: RgbColor,
    dim: RgbColor,
) -> Vec<(char, RgbColor)> {
    let filled = (ratio * width as f32).round() as usize;
    let knob_pos = filled.min(width.saturating_sub(1));
    let defc = (def_ratio * (width as f32 - 1.0)).round() as usize;
    (0..width)
        .map(|i| {
            if i == knob_pos {
                ('●', accent)
            } else if i == defc {
                ('│', dim)
            } else if i < filled {
                let col = if truecolor {
                    let t = i as f32 / width as f32;
                    RgbColor {
                        r: (90.0 + 140.0 * t) as u8,
                        g: 200,
                        b: (120.0 - 60.0 * t) as u8,
                    }
                } else {
                    accent
                };
                ('━', col)
            } else {
                ('━', dim)
            }
        })
        .collect()
}

/// Renders a sparkline string from a history slice of normalised `[0.0, 1.0]`
/// values.
///
/// Each sample maps to one of `▁▂▃▄▅▆▇█` (8 levels). Values are clamped
/// before mapping so out-of-range inputs are safe.
pub fn sparkline(hist: &[f32]) -> String {
    const BARS: [char; 8] = ['▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];
    hist.iter()
        .map(|v| BARS[((v.clamp(0.0, 1.0)) * 7.0).round() as usize])
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::render::palette::RgbColor;
    const A: RgbColor = RgbColor { r: 0, g: 200, b: 0 };
    const D: RgbColor = RgbColor {
        r: 80,
        g: 80,
        b: 80,
    };

    #[test]
    fn gauge_width_and_default_tick() {
        let g = gauge(0.5, 0.25, 8, A, A, D);
        assert_eq!(g.len(), 8);
        assert!(g.iter().any(|(c, _)| *c == '│')); // default tick present
        assert!(g.iter().any(|(c, _)| *c == '█')); // some fill
    }

    #[test]
    fn heatmap_falls_back_to_solid_in_256() {
        let s = heatmap_slider(0.8, 0.5, 10, false, A, D);
        // every fill cell uses the solid accent (no gradient) under 256-color
        assert!(s
            .iter()
            .filter(|(c, _)| *c == '━')
            .all(|(_, col)| *col == A || *col == D));
    }

    #[test]
    fn sparkline_maps_range() {
        assert_eq!(sparkline(&[0.0, 1.0]).chars().next().unwrap(), '▁');
    }
}
