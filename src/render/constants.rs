//! Centralized constants for rendering parameters.
//!
//! This module consolidates magic numbers related to rendering thresholds,
//! character mappings, color quantization, and other display parameters.

/// Threshold constants for brightness/edge detection.
pub mod threshold {
    /// Edge detection threshold for rendering.
    pub const EDGE: f32 = 0.05;

    /// Minimum brightness threshold for rendering.
    pub const BRIGHTNESS_MIN: f32 = 0.01;

    /// Noise amount for scanline effects (±percentage of brightness).
    pub const NOISE_AMOUNT: f32 = 0.08;

    /// Minimum factor for scanline darkening.
    pub const SCANLINE_FACTOR_MIN: f32 = 0.05;

    /// Maximum factor for scanline darkening.
    pub const SCANLINE_FACTOR_MAX: f32 = 0.6;

    /// Default brightness threshold for point grid mode.
    pub const POINT_DEFAULT: f32 = 0.15;

    /// Default brightness threshold for braille mode.
    pub const BRAILLE_DEFAULT: f32 = 0.05;

    /// Default brightness threshold for quadrant mode.
    pub const QUADRANT_DEFAULT: f32 = 0.05;

    /// Default brightness threshold for vertical block mode.
    pub const VERTICAL_BLOCK_DEFAULT: f32 = 0.05;

    /// Threshold for sculpted outline mode.
    pub const SCULPTED_OUTLINE: f32 = 0.05;

    /// Threshold for dithering edge detection.
    pub const DITHER_EDGE: f32 = 0.15;

    /// Minimum contrast difference for directional enhancement.
    pub const CONTRAST_DELTA: f32 = 0.05;
}

/// Intensity mapping constants.
pub mod intensity {
    /// Number of discrete intensity levels for quantization.
    pub const QUANTIZE_LEVELS: u32 = 6;

    /// Default log base for logarithmic intensity mapping.
    pub const LOG_DEFAULT: f32 = 10.0;

    /// Default exponent for exponential intensity mapping.
    pub const EXP_DEFAULT: f32 = 10.0;

    /// Perlin noise frequency for perlin intensity mapping.
    pub const PERLIN_FREQUENCY: f32 = 0.15;

    /// Perlin noise octaves for perlin intensity mapping.
    pub const PERLIN_OCTAVES: u32 = 4;

    /// Perlin noise seed for perlin intensity mapping.
    pub const PERLIN_SEED: u64 = 42;
}

/// Color palette constants.
pub mod palette {
    /// Default palette name.
    pub const DEFAULT_NAME: &str = "moss";

    /// Minimum number of colors in custom palette.
    pub const CUSTOM_MIN_COLORS: usize = 2;

    /// Maximum number of colors in custom palette.
    pub const CUSTOM_MAX_COLORS: usize = 11;

    /// Default grid color (white).
    pub const DEFAULT_GRID_COLOR: &str = "ffffff";
}

/// Grid rendering constants.
pub mod grid {
    /// Default grid size (cell spacing).
    pub const DEFAULT_SIZE: usize = 10;

    /// Default grid opacity.
    pub const DEFAULT_OPACITY: f32 = 1.0;

    /// Minimum grid size.
    pub const MIN_SIZE: usize = 1;

    /// Maximum grid size.
    pub const MAX_SIZE: usize = 50;
}

/// Terminal rendering constants.
pub mod terminal {
    /// Default simulation resolution width.
    pub const DEFAULT_WIDTH: usize = 400;

    /// Default simulation resolution height.
    pub const DEFAULT_HEIGHT: usize = 200;

    /// Default FPS target.
    pub const DEFAULT_FPS: u32 = 30;

    /// Minimum FPS.
    pub const MIN_FPS: u32 = 1;

    /// Maximum FPS.
    pub const MAX_FPS: u32 = 144;

    /// Default frame delay in milliseconds.
    pub const DEFAULT_FRAME_DELAY_MS: u64 = 0;

    /// Small terminal width threshold (columns).
    pub const TERMINAL_SMALL: usize = 80;

    /// Medium terminal width threshold (columns).
    pub const TERMINAL_MEDIUM: usize = 120;
}

/// Palette shift speed constants (degrees per second).
pub mod palette_shift {
    /// Palette shift disabled (0 degrees/second).
    pub const OFF: f32 = 0.0;

    /// Slow palette shift speed (5 degrees/second).
    pub const SLOW: f32 = 5.0;

    /// Medium palette shift speed (15 degrees/second).
    pub const MEDIUM: f32 = 15.0;

    /// Fast palette shift speed (45 degrees/second).
    pub const FAST: f32 = 45.0;
}

/// Motion blur constants.
pub mod motion_blur {
    /// Available motion blur frame counts.
    pub const FRAMES: [usize; 4] = [0, 3, 5, 7];
}

/// Dithering constants.
pub mod dither {
    /// Default dithering intensity.
    pub const DEFAULT_INTENSITY: f32 = 0.5;

    /// Minimum dithering intensity.
    pub const MIN_INTENSITY: f32 = 0.0;

    /// Maximum dithering intensity.
    pub const MAX_INTENSITY: f32 = 1.0;

    /// Default hybrid edge threshold.
    pub const DEFAULT_HYBRID_EDGE: f32 = 0.15;
}

/// ASCII contrast constants.
pub mod ascii {
    /// Default ASCII contrast value.
    pub const DEFAULT_CONTRAST: f32 = 1.0;

    /// Minimum contrast value (1.0 = no enhancement).
    pub const MIN_CONTRAST: f32 = 1.0;

    /// Maximum contrast value (2.0+ = strong enhancement).
    pub const MAX_CONTRAST: f32 = 2.0;
}

/// Charset level counts.
pub mod charset_levels {
    /// Half-block character levels.
    pub const HALF_BLOCK: usize = 9;

    /// ASCII character levels.
    pub const ASCII: usize = 10;

    /// Braille character levels.
    pub const BRAILLE: usize = 16;

    /// Quadrant character levels (2^4 combinations).
    pub const QUADRANT: usize = 16;

    /// Shade character levels.
    pub const SHADE: usize = 5;

    /// Point grid levels (on/off).
    pub const POINTS: usize = 2;

    /// Sculpted character levels.
    pub const SCULPTED: usize = 16;
}

/// Brightness normalization constants.
pub mod normalization {
    /// Default adaptive brightness window size.
    pub const DEFAULT_WINDOW: usize = 100;

    /// Small window size.
    pub const WINDOW_SMALL: usize = 50;

    /// Large window size.
    pub const WINDOW_LARGE: usize = 200;
}
