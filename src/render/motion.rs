//! Pure motion-math helpers for overlay animation.
//!
//! No state, no I/O — just math primitives. `lerp_rgb` is used across the
//! renderer / frame_buffer / ambient; `breath` drives the title-box accent and
//! the ambient TUNE pulse.

use crate::render::palette::RgbColor;

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
}
