pub const BAYER_4X4: [[u8; 4]; 4] = [[0, 8, 2, 10], [12, 4, 14, 6], [3, 11, 1, 9], [15, 7, 13, 5]];

pub fn apply_dither(x: usize, y: usize, brightness: f32, intensity: f32) -> f32 {
    let threshold = (BAYER_4X4[y % 4][x % 4] as f32) / 16.0;
    let dithered = brightness + (threshold - 0.5) * intensity * 0.1;
    dithered.clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bayer_4x4_values() {
        assert_eq!(BAYER_4X4[0][0], 0);
        assert_eq!(BAYER_4X4[0][1], 8);
        assert_eq!(BAYER_4X4[0][2], 2);
        assert_eq!(BAYER_4X4[0][3], 10);
        assert_eq!(BAYER_4X4[1][0], 12);
        assert_eq!(BAYER_4X4[1][1], 4);
        assert_eq!(BAYER_4X4[1][2], 14);
        assert_eq!(BAYER_4X4[1][3], 6);
        assert_eq!(BAYER_4X4[2][0], 3);
        assert_eq!(BAYER_4X4[2][1], 11);
        assert_eq!(BAYER_4X4[2][2], 1);
        assert_eq!(BAYER_4X4[2][3], 9);
        assert_eq!(BAYER_4X4[3][0], 15);
        assert_eq!(BAYER_4X4[3][1], 7);
        assert_eq!(BAYER_4X4[3][2], 13);
        assert_eq!(BAYER_4X4[3][3], 5);
    }

    #[test]
    fn test_bayer_threshold_range() {
        for y in 0..4 {
            for x in 0..4 {
                let threshold = BAYER_4X4[y][x] as f32 / 16.0;
                assert!(threshold >= 0.0);
                assert!(threshold <= 1.0);
            }
        }
    }

    #[test]
    fn test_apply_dither_no_intensity() {
        let brightness = 0.5;
        let dithered = apply_dither(0, 0, brightness, 0.0);
        assert_eq!(dithered, brightness);
    }

    #[test]
    fn test_apply_dither_clamping_min() {
        let dithered = apply_dither(0, 0, 0.0, 1.0);
        assert!(dithered >= 0.0);
    }

    #[test]
    fn test_apply_dither_clamping_max() {
        let dithered = apply_dither(0, 0, 1.0, 1.0);
        assert!(dithered <= 1.0);
    }

    #[test]
    fn test_apply_dither_tiling_x() {
        let brightness = 0.5;
        let dithered0 = apply_dither(0, 0, brightness, 1.0);
        let dithered4 = apply_dither(4, 0, brightness, 1.0);
        assert_eq!(dithered0, dithered4);
    }

    #[test]
    fn test_apply_dither_tiling_y() {
        let brightness = 0.5;
        let dithered0 = apply_dither(0, 0, brightness, 1.0);
        let dithered4 = apply_dither(0, 4, brightness, 1.0);
        assert_eq!(dithered0, dithered4);
    }

    #[test]
    fn test_apply_dither_pattern_consistency() {
        let brightness = 0.5;
        let intensity = 1.0;

        let results: Vec<f32> = (0..16)
            .map(|i| {
                let x = i % 4;
                let y = i / 4;
                apply_dither(x, y, brightness, intensity)
            })
            .collect();

        assert_eq!(results.len(), 16);

        for &result in &results {
            assert!(result >= 0.0);
            assert!(result <= 1.0);
        }
    }

    #[test]
    fn test_apply_dither_intensity_scaling() {
        let brightness = 0.5;
        let dithered_low = apply_dither(0, 0, brightness, 0.25);
        let dithered_high = apply_dither(0, 0, brightness, 1.0);

        assert_ne!(dithered_low, dithered_high);
    }

    #[test]
    fn test_apply_dither_mid_brightness() {
        let brightness = 0.5;
        let dithered = apply_dither(0, 0, brightness, 0.5);

        assert!(dithered >= 0.0);
        assert!(dithered <= 1.0);
        assert_ne!(dithered, brightness);
    }

    #[test]
    fn test_apply_dither_extreme_thresholds() {
        let brightness = 0.5;
        let intensity = 1.0;

        let min_threshold = apply_dither(0, 0, brightness, intensity);
        let max_threshold = apply_dither(3, 0, brightness, intensity);

        assert_ne!(min_threshold, max_threshold);
    }

    #[test]
    fn test_apply_dither_negative_brightness() {
        let dithered = apply_dither(0, 0, -0.5, 1.0);
        assert_eq!(dithered, 0.0);
    }

    #[test]
    fn test_apply_dither_above_one_brightness() {
        let dithered = apply_dither(0, 0, 1.5, 1.0);
        assert_eq!(dithered, 1.0);
    }
}
