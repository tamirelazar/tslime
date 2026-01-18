use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
/// Matrix size for ordered dithering.
pub enum DitherMatrix {
    /// Standard 4×4 Bayer matrix.
    #[default]
    Bayer4x4,
    /// Larger 8×8 Bayer matrix for smoother gradients.
    Bayer8x8,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Default)]
/// Algorithm used for color quantization and dithering.
pub enum DitherMode {
    /// No dithering (nearest neighbor quantization).
    #[default]
    None,
    /// Ordered dithering using a Bayer matrix.
    Ordered {
        /// Intensity of the dither noise (0.0-1.0).
        intensity: f32,
        /// The matrix pattern to use.
        matrix: DitherMatrix,
    },
    /// Error diffusion (Floyd-Steinberg).
    ErrorDiffusion {
        /// Whether to alternate scan direction (serpentine) to reduce artifacts.
        serpentine: bool,
    },
    /// Hybrid approach combining ordered dithering and error diffusion.
    Hybrid {
        /// Threshold for edge detection (to switch modes).
        edge_threshold: f32,
        /// Intensity of the ordered dither component.
        intensity: f32,
        /// The matrix pattern to use.
        matrix: DitherMatrix,
    },
}

impl DitherMode {
    /// Returns the display name of the dither mode.
    pub fn name(&self) -> &str {
        match self {
            DitherMode::None => "None",
            DitherMode::Ordered { .. } => "Ordered",
            DitherMode::ErrorDiffusion { .. } => "ErrorDiff",
            DitherMode::Hybrid { .. } => "Hybrid",
        }
    }
}

/// 4×4 Bayer ordered dithering matrix.
pub const BAYER_4X4: [[u8; 4]; 4] = [[0, 8, 2, 10], [12, 4, 14, 6], [3, 11, 1, 9], [15, 7, 13, 5]];

/// 8×8 Bayer ordered dithering matrix.
pub const BAYER_8X8: [[u8; 8]; 8] = [
    [0, 48, 12, 60, 3, 51, 15, 63],
    [32, 16, 44, 28, 35, 19, 47, 31],
    [8, 56, 4, 52, 11, 59, 7, 55],
    [40, 24, 36, 20, 43, 27, 39, 23],
    [2, 50, 14, 62, 1, 49, 13, 61],
    [34, 18, 46, 30, 33, 17, 45, 29],
    [10, 58, 6, 54, 9, 57, 5, 53],
    [42, 26, 38, 22, 41, 25, 37, 21],
];

fn bayer_threshold(x: usize, y: usize, matrix: DitherMatrix) -> f32 {
    match matrix {
        DitherMatrix::Bayer4x4 => BAYER_4X4[y % 4][x % 4] as f32 / 16.0,
        DitherMatrix::Bayer8x8 => BAYER_8X8[y % 8][x % 8] as f32 / 64.0,
    }
}

/// Applies ordered dithering to a pixel.
///
/// Modulates the pixel brightness based on its coordinate and the dither matrix.
pub fn apply_ordered_dither(
    x: usize,
    y: usize,
    brightness: f32,
    intensity: f32,
    matrix: DitherMatrix,
) -> f32 {
    let threshold = bayer_threshold(x, y, matrix);
    let dithered = brightness + (threshold - 0.5) * intensity;
    dithered.clamp(0.0, 1.0)
}

/// Applies ordered dithering with temporal modulation (animation).
///
/// Shifts the dither threshold based on the frame number to create animated noise.
#[allow(dead_code)]
pub fn apply_ordered_dither_with_frame(
    x: usize,
    y: usize,
    brightness: f32,
    intensity: f32,
    matrix: DitherMatrix,
    frame: usize,
) -> f32 {
    let threshold = bayer_threshold(x, y, matrix);
    let phase = (frame as f32 * 0.1) % 1.0;
    let modulated = if threshold < phase { 1.0 } else { 0.0 };
    let dithered = brightness + (modulated - 0.5) * intensity;
    dithered.clamp(0.0, 1.0)
}

/// Quantizes a brightness value to a specific number of discrete levels.
pub fn quantize_to_levels(brightness: f32, num_levels: usize) -> f32 {
    if num_levels <= 1 {
        return 0.0;
    }
    let levels_minus_one = num_levels - 1;
    let quantized = (brightness * levels_minus_one as f32).round() as usize;
    quantized as f32 / levels_minus_one as f32
}

/// Calculates the local variance of brightness in a region.
///
/// Used for edge detection in hybrid dithering modes.
pub fn local_variance(
    downsampled: &[crate::render::downsample::Cell],
    width: usize,
    x: usize,
    y: usize,
    radius: usize,
) -> f32 {
    if radius == 0 {
        return 0.0;
    }
    let mut sum = 0.0;
    let mut sum_sq = 0.0;
    let mut count = 0;

    for dy in -(radius as i32)..=radius as i32 {
        for dx in -(radius as i32)..=radius as i32 {
            let nx = x as i32 + dx;
            let ny = y as i32 + dy;
            if nx >= 0 && nx < width as i32 && ny >= 0 {
                let idx = (ny as usize) * width + nx as usize;
                if idx < downsampled.len() {
                    let brightness = (downsampled[idx].top + downsampled[idx].bottom) / 2.0;
                    sum += brightness;
                    sum_sq += brightness * brightness;
                    count += 1;
                }
            }
        }
    }

    if count == 0 {
        return 0.0;
    }

    let mean = sum / count as f32;
    let variance = (sum_sq / count as f32) - (mean * mean);
    variance.sqrt()
}

#[deprecated(since = "0.1.0", note = "Use apply_ordered_dither instead")]
#[allow(dead_code)]
/// Legacy dither function (deprecated).
pub fn apply_dither(x: usize, y: usize, brightness: f32, intensity: f32) -> f32 {
    apply_ordered_dither(x, y, brightness, intensity, DitherMatrix::Bayer4x4)
}

#[cfg(test)]
#[allow(deprecated)]
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
        for row in &BAYER_4X4 {
            for &val in row {
                let threshold = val as f32 / 16.0;
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

    #[test]
    fn test_ordered_dither_different_matrices() {
        let brightness = 0.5;
        let result_bayer = apply_ordered_dither(0, 0, brightness, 1.0, DitherMatrix::Bayer4x4);
        let result_bayer_8x8 = apply_ordered_dither(0, 0, brightness, 1.0, DitherMatrix::Bayer8x8);
        assert!(result_bayer >= 0.0 && result_bayer <= 1.0);
        assert!(result_bayer_8x8 >= 0.0 && result_bayer_8x8 <= 1.0);
    }

    #[test]
    fn test_dither_mode_name() {
        assert_eq!(DitherMode::None.name(), "None");
        assert_eq!(
            DitherMode::Ordered {
                intensity: 1.0,
                matrix: DitherMatrix::Bayer4x4
            }
            .name(),
            "Ordered"
        );
        assert_eq!(
            DitherMode::ErrorDiffusion { serpentine: true }.name(),
            "ErrorDiff"
        );
        assert_eq!(
            DitherMode::Hybrid {
                edge_threshold: 0.5,
                intensity: 1.0,
                matrix: DitherMatrix::Bayer4x4
            }
            .name(),
            "Hybrid"
        );
    }

    #[test]
    fn test_dither_mode_default() {
        assert_eq!(DitherMode::default(), DitherMode::None);
    }

    #[test]
    fn test_ordered_dither_tile_consistency() {
        let brightness = 0.5;
        for matrix in [DitherMatrix::Bayer4x4, DitherMatrix::Bayer8x8] {
            for y in 0..4 {
                for x in 0..4 {
                    let result = apply_ordered_dither(x, y, brightness, 1.0, matrix);
                    assert!(
                        result >= 0.0 && result <= 1.0,
                        "Result out of bounds for matrix {:?}",
                        matrix
                    );
                }
            }
        }
    }

    #[test]
    fn test_ordered_dither_low_brightness() {
        let result = apply_ordered_dither(0, 0, 0.1, 1.0, DitherMatrix::Bayer4x4);
        assert!(result <= 0.1);
    }

    #[test]
    fn test_ordered_dither_high_brightness() {
        let result = apply_ordered_dither(0, 0, 0.9, 1.0, DitherMatrix::Bayer4x4);
        assert!(result >= 0.0 && result <= 1.0);
    }

    #[test]
    fn test_local_variance_basic() {
        use crate::render::downsample::Cell;

        let mut downsampled = vec![
            Cell {
                top: 0.5,
                bottom: 0.5,
                top_left: 0.5,
                top_right: 0.5,
                bottom_left: 0.5,
                bottom_right: 0.5,
            };
            100
        ];
        downsampled[50] = Cell {
            top: 1.0,
            bottom: 0.0,
            top_left: 1.0,
            top_right: 0.0,
            bottom_left: 0.5,
            bottom_right: 0.5,
        };

        let variance = local_variance(&downsampled, 10, 5, 5, 1);
        assert!(variance >= 0.0);
        assert!(variance.is_finite());
    }

    #[test]
    fn test_local_variance_edge_case() {
        use crate::render::downsample::Cell;

        let downsampled = vec![
            Cell {
                top: 0.5,
                bottom: 0.5,
                top_left: 0.5,
                top_right: 0.5,
                bottom_left: 0.5,
                bottom_right: 0.5,
            };
            4
        ];
        let variance = local_variance(&downsampled, 2, 0, 0, 1);
        assert!(variance >= 0.0);
    }

    #[test]
    fn test_local_variance_empty_region() {
        use crate::render::downsample::Cell;

        let downsampled: Vec<Cell> = vec![];
        let variance = local_variance(&downsampled, 0, 0, 0, 1);
        assert_eq!(variance, 0.0);
    }

    #[test]
    fn test_local_variance_zero_radius() {
        use crate::render::downsample::Cell;

        let downsampled = vec![
            Cell {
                top: 0.5,
                bottom: 0.5,
                top_left: 0.5,
                top_right: 0.5,
                bottom_left: 0.5,
                bottom_right: 0.5,
            };
            100
        ];
        let variance = local_variance(&downsampled, 10, 5, 5, 0);
        assert_eq!(variance, 0.0);
    }

    #[test]
    fn test_quantize_to_levels() {
        assert_eq!(quantize_to_levels(0.0, 2), 0.0);
        assert_eq!(quantize_to_levels(1.0, 2), 1.0);
        let result = quantize_to_levels(0.5, 2);
        assert!(result == 0.0 || result == 1.0);
    }

    #[test]
    fn test_quantize_to_levels_more_levels() {
        assert_eq!(quantize_to_levels(0.0, 4), 0.0);
        assert_eq!(quantize_to_levels(1.0, 4), 1.0);
        assert!((quantize_to_levels(0.33, 4) - 0.333).abs() < 0.01);
    }
}
