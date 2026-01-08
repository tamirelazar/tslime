use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum DitherMatrix {
    #[default]
    Bayer4x4,
    Bayer8x8,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Default)]
pub enum DitherMode {
    #[default]
    None,
    Ordered {
        intensity: f32,
        matrix: DitherMatrix,
    },
    ErrorDiffusion {
        serpentine: bool,
    },
    Hybrid {
        edge_threshold: f32,
        intensity: f32,
        matrix: DitherMatrix,
    },
}

impl DitherMode {
    pub fn name(&self) -> &str {
        match self {
            DitherMode::None => "None",
            DitherMode::Ordered { .. } => "Ordered",
            DitherMode::ErrorDiffusion { .. } => "ErrorDiff",
            DitherMode::Hybrid { .. } => "Hybrid",
        }
    }
}

pub const BAYER_4X4: [[u8; 4]; 4] = [[0, 8, 2, 10], [12, 4, 14, 6], [3, 11, 1, 9], [15, 7, 13, 5]];

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

pub fn quantize_to_levels(brightness: f32, num_levels: usize) -> f32 {
    if num_levels <= 1 {
        return 0.0;
    }
    let levels_minus_one = num_levels - 1;
    let quantized = (brightness * levels_minus_one as f32).round() as usize;
    quantized as f32 / levels_minus_one as f32
}

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
}
