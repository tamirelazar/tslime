//! Pure motion-math helpers for overlay animation.
//!
//! No state, no I/O — just math primitives consumed by Tasks 9, 13, 14, 19.

use crate::render::palette::RgbColor;

use crate::render::panel::RichCell;

/// Linearly interpolates between two [`RgbColor`] values, rounding each channel.
///
/// `t` is clamped to `[0, 1]`.
pub fn lerp_rgb(a: RgbColor, b: RgbColor, t: f32) -> RgbColor {
    let t = t.clamp(0.0, 1.0);
    RgbColor {
        r: (a.r as f32 + (b.r as f32 - a.r as f32) * t).round() as u8,
        g: (a.g as f32 + (b.g as f32 - a.g as f32) * t).round() as u8,
        b: (a.b as f32 + (b.b as f32 - a.b as f32) * t).round() as u8,
    }
}

/// Ease-out cubic: slow deceleration at end.
///
/// `t` is clamped to `[0, 1]`. Returns values in `[0, 1]`.
pub fn ease_out_cubic(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    1.0 - (1.0 - t).powi(3)
}

/// Ease-in-out: slow at both ends.
///
/// `t` is clamped to `[0, 1]`. Returns values in `[0, 1]`.
pub fn ease_in_out(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    if t < 0.5 {
        4.0 * t * t * t
    } else {
        1.0 - (-2.0 * t + 2.0).powi(3) / 2.0
    }
}

/// Oscillating breath envelope.
///
/// Returns `1.0 - depth * 0.5 * (1.0 - sin(2π * phase / period))`,
/// which stays in `[1 - depth, 1]` as `phase` advances.
///
/// - `phase`  — current time/frame counter (any unit matching `period`)
/// - `period` — full cycle length in same units
/// - `depth`  — modulation depth in `[0, 1]`; 0 = flat, 1 = full swing
pub fn breath(phase: f32, period: f32, depth: f32) -> f32 {
    1.0 - depth * 0.5 * (1.0 - (std::f32::consts::TAU * phase / period).sin())
}

/// Dims every set `fg`/`bg` colour in a [`RichCell`] grid toward `toward`.
///
/// Each colour `c` becomes `lerp_rgb(c, toward, 1.0 - dim)`.
/// `dim = 1.0` → no change; `dim = 0.0` → fully replaced by `toward`.
pub fn dim_overlay(rich: &mut [Vec<RichCell>], dim: f32, toward: RgbColor) {
    let blend = 1.0 - dim;
    for row in rich.iter_mut() {
        for (_, fg, bg) in row.iter_mut() {
            if let Some(c) = fg {
                *c = lerp_rgb(*c, toward, blend);
            }
            if let Some(c) = bg {
                *c = lerp_rgb(*c, toward, blend);
            }
        }
    }
}

/// Crossfades two [`RichCell`] grids.
///
/// Lerps cell-by-cell at parameter `t` (`0.0` = all `a`, `1.0` = all `b`).
pub fn crossfade(a: &[Vec<RichCell>], b: &[Vec<RichCell>], t: f32) -> Vec<Vec<RichCell>> {
    if a.len() != b.len() {
        return b.to_vec();
    }
    let t = t.clamp(0.0, 1.0);
    let mut out = Vec::with_capacity(a.len());
    for (row_a, row_b) in a.iter().zip(b.iter()) {
        if row_a.len() != row_b.len() {
            out.push(row_b.clone());
            continue;
        }
        let mut row = Vec::with_capacity(row_a.len());
        for ((ch_a, fg_a, bg_a), (ch_b, fg_b, bg_b)) in row_a.iter().zip(row_b.iter()) {
            // Blend character: use `b` once t >= 0.5, else `a`.
            let ch = if t >= 0.5 { *ch_b } else { *ch_a };
            let fg = match (*fg_a, *fg_b) {
                (Some(ca), Some(cb)) => Some(lerp_rgb(ca, cb, t)),
                (None, Some(cb)) => Some(lerp_rgb(cb, cb, t)),
                (Some(ca), None) => Some(lerp_rgb(ca, ca, 1.0 - t)),
                (None, None) => None,
            };
            let bg = match (*bg_a, *bg_b) {
                (Some(ca), Some(cb)) => Some(lerp_rgb(ca, cb, t)),
                (None, Some(cb)) => Some(lerp_rgb(cb, cb, t)),
                (Some(ca), None) => Some(lerp_rgb(ca, ca, 1.0 - t)),
                (None, None) => None,
            };
            row.push((ch, fg, bg));
        }
        out.push(row);
    }
    out
}

/// Truncates a [`RichCell`] grid to at most `visible` rows.
pub fn clip_rows(rich: &[Vec<RichCell>], visible: usize) -> Vec<Vec<RichCell>> {
    rich[..rich.len().min(visible)].to_vec()
}

#[cfg(test)]
mod motion_tests {
    use super::*;
    use crate::render::palette::RgbColor;

    // lerp_rgb

    #[test]
    fn lerp_endpoints() {
        let a = RgbColor::new(0, 0, 0);
        let b = RgbColor::new(100, 100, 100);
        assert_eq!(lerp_rgb(a, b, 0.0), a);
        assert_eq!(lerp_rgb(a, b, 1.0), b);
        assert_eq!(lerp_rgb(a, b, 0.5), RgbColor::new(50, 50, 50));
    }

    #[test]
    fn lerp_clamps_t() {
        let a = RgbColor::new(10, 20, 30);
        let b = RgbColor::new(200, 200, 200);
        assert_eq!(lerp_rgb(a, b, -1.0), a);
        assert_eq!(lerp_rgb(a, b, 2.0), b);
    }

    #[test]
    fn lerp_midpoint_rounds() {
        // 0 -> 255 at t=0.5 -> 127.5 rounds to 128
        let a = RgbColor::new(0, 0, 0);
        let b = RgbColor::new(255, 255, 255);
        let mid = lerp_rgb(a, b, 0.5);
        assert_eq!(mid.r, 128);
        assert_eq!(mid.g, 128);
        assert_eq!(mid.b, 128);
    }

    // ease_out_cubic

    #[test]
    fn ease_endpoints_clamped() {
        assert_eq!(ease_out_cubic(0.0), 0.0);
        assert_eq!(ease_out_cubic(1.0), 1.0);
        assert_eq!(ease_out_cubic(-1.0), 0.0); // clamped
    }

    #[test]
    fn ease_out_cubic_midpoint() {
        let v = ease_out_cubic(0.5);
        // 1 - (0.5)^3 = 0.875
        assert!((v - 0.875).abs() < 1e-6);
    }

    // ease_in_out

    #[test]
    fn ease_in_out_endpoints() {
        assert_eq!(ease_in_out(0.0), 0.0);
        assert_eq!(ease_in_out(1.0), 1.0);
    }

    #[test]
    fn ease_in_out_midpoint() {
        // Symmetric -> exactly 0.5 at t=0.5
        assert!((ease_in_out(0.5) - 0.5).abs() < 1e-6);
    }

    #[test]
    fn ease_in_out_clamped() {
        assert_eq!(ease_in_out(-5.0), 0.0);
        assert_eq!(ease_in_out(5.0), 1.0);
    }

    // breath

    #[test]
    fn breath_bounded() {
        for i in 0..100 {
            let v = breath(i as f32 * 0.1, 4.0, 0.3);
            assert!(
                (0.7..=1.0).contains(&v),
                "breath at phase {} out of range: {}",
                i as f32 * 0.1,
                v
            );
        }
    }

    #[test]
    fn breath_no_depth_is_flat() {
        for i in 0..100 {
            let v = breath(i as f32 * 0.5, 10.0, 0.0);
            assert!((v - 1.0).abs() < 1e-6, "flat breath should be 1.0, got {v}");
        }
    }

    // dim_overlay
    #[test]
    fn dim_overlay_full_dim_is_noop() {
        let white = RgbColor::new(255, 255, 255);
        let black = RgbColor::new(0, 0, 0);
        let mut grid: Vec<Vec<crate::render::panel::RichCell>> =
            vec![vec![('a', Some(white), Some(black))]];
        dim_overlay(&mut grid, 1.0, RgbColor::new(128, 0, 0));
        let (_, fg, bg) = grid[0][0];
        assert_eq!(fg, Some(white));
        assert_eq!(bg, Some(black));
    }
    #[test]
    fn dim_overlay_zero_dim_replaces_color() {
        let white = RgbColor::new(255, 255, 255);
        let toward = RgbColor::new(0, 0, 0);
        let mut grid: Vec<Vec<crate::render::panel::RichCell>> =
            vec![vec![('x', Some(white), None)]];
        dim_overlay(&mut grid, 0.0, toward);
        let (_, fg, bg) = grid[0][0];
        assert_eq!(fg, Some(toward));
        assert_eq!(bg, None);
    }
    #[test]
    fn dim_overlay_half_blend() {
        let a = RgbColor::new(200, 100, 0);
        let toward = RgbColor::new(0, 0, 0);
        let mut grid: Vec<Vec<crate::render::panel::RichCell>> = vec![vec![('z', Some(a), None)]];
        dim_overlay(&mut grid, 0.5, toward);
        let (_, fg, _) = grid[0][0];
        // lerp(a, toward, 0.5): (100, 50, 0)
        assert_eq!(fg, Some(RgbColor::new(100, 50, 0)));
    }

    // crossfade
    #[test]
    fn crossfade_t0_is_a() {
        let red = RgbColor::new(255, 0, 0);
        let blue = RgbColor::new(0, 0, 255);
        let a: Vec<Vec<crate::render::panel::RichCell>> = vec![vec![('a', Some(red), None)]];
        let b: Vec<Vec<crate::render::panel::RichCell>> = vec![vec![('b', Some(blue), None)]];
        let out = crossfade(&a, &b, 0.0);
        let (ch, fg, _) = out[0][0];
        assert_eq!(ch, 'a');
        assert_eq!(fg, Some(red));
    }
    #[test]
    fn crossfade_t1_is_b() {
        let red = RgbColor::new(255, 0, 0);
        let blue = RgbColor::new(0, 0, 255);
        let a: Vec<Vec<crate::render::panel::RichCell>> = vec![vec![('a', Some(red), None)]];
        let b: Vec<Vec<crate::render::panel::RichCell>> = vec![vec![('b', Some(blue), None)]];
        let out = crossfade(&a, &b, 1.0);
        let (ch, fg, _) = out[0][0];
        assert_eq!(ch, 'b');
        assert_eq!(fg, Some(blue));
    }
    #[test]
    fn crossfade_dim_mismatch_returns_b() {
        let a: Vec<Vec<crate::render::panel::RichCell>> =
            vec![vec![('a', None, None)], vec![('b', None, None)]];
        let b: Vec<Vec<crate::render::panel::RichCell>> = vec![vec![('c', None, None)]];
        let out = crossfade(&a, &b, 0.5);
        assert_eq!(out.len(), 1);
        assert_eq!(out[0][0].0, 'c');
    }

    // clip_rows
    #[test]
    fn clip_rows_truncates() {
        let grid: Vec<Vec<crate::render::panel::RichCell>> = (0..5)
            .map(|i| vec![(char::from_digit(i, 10).unwrap(), None, None)])
            .collect();
        let clipped = clip_rows(&grid, 3);
        assert_eq!(clipped.len(), 3);
    }
    #[test]
    fn clip_rows_no_truncation_needed() {
        let grid: Vec<Vec<crate::render::panel::RichCell>> =
            vec![vec![('a', None, None)], vec![('b', None, None)]];
        let clipped = clip_rows(&grid, 10);
        assert_eq!(clipped.len(), 2);
    }
    #[test]
    fn clip_rows_zero_visible() {
        let grid: Vec<Vec<crate::render::panel::RichCell>> = vec![vec![('a', None, None)]];
        let clipped = clip_rows(&grid, 0);
        assert_eq!(clipped.len(), 0);
    }
}
