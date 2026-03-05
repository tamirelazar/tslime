//! Centralized configuration defaults for tslime.
//!
//! This module provides a single source of truth for ALL configuration default values,
//! consolidating defaults from simulation, rendering, and other modules.
//!
//! All defaults use the DEFAULT_* naming convention for consistency.

#![allow(missing_docs)]

/// Agent behavior constants.
pub mod agent {
    /// Default sensor angle in degrees.
    pub const DEFAULT_SENSOR_ANGLE: f32 = 22.5;
    /// Default rotation angle in degrees.
    pub const DEFAULT_ROTATION_ANGLE: f32 = 45.0;
    /// Default sensor distance in pixels.
    pub const DEFAULT_SENSOR_DISTANCE: f32 = 9.0;
    /// Default step size (speed) in pixels per step.
    pub const DEFAULT_STEP_SIZE: f32 = 1.0;
    /// Default deposit amount per step.
    pub const DEFAULT_DEPOSIT_AMOUNT: f32 = 5.0;

    /// Minimum sensor angle in degrees.
    pub const MIN_SENSOR_ANGLE: f32 = 5.0;
    /// Maximum sensor angle in degrees.
    pub const MAX_SENSOR_ANGLE: f32 = 90.0;
    /// Minimum rotation angle in degrees.
    pub const MIN_ROTATION_ANGLE: f32 = 5.0;
    /// Maximum rotation angle in degrees.
    pub const MAX_ROTATION_ANGLE: f32 = 90.0;
    /// Minimum sensor distance.
    pub const MIN_SENSOR_DISTANCE: f32 = 1.0;
    /// Maximum sensor distance.
    pub const MAX_SENSOR_DISTANCE: f32 = 50.0;
    /// Minimum step size.
    pub const MIN_STEP_SIZE: f32 = 0.01;
    /// Maximum step size.
    pub const MAX_STEP_SIZE: f32 = 10.0;
    /// Minimum deposit amount.
    pub const MIN_DEPOSIT_AMOUNT: f32 = 0.1;
    /// Maximum deposit amount.
    pub const MAX_DEPOSIT_AMOUNT: f32 = 20.0;
}

/// Steering force constants for external influences.
pub mod steering {
    /// Strength of attractor/repeller steering.
    pub const DEFAULT_ATTRACTOR_STRENGTH: f32 = 0.1;
    /// Minimum distance for attractor force calculation (prevents division by zero).
    pub const MIN_ATTRACTOR_DISTANCE: f32 = 1.0;
    /// Minimum force threshold for applying steering.
    pub const MIN_FORCE_THRESHOLD: f32 = 0.001;
    // Backward compatibility aliases (deprecated)
    #[deprecated(since = "0.2.0", note = "Use DEFAULT_ATTRACTOR_STRENGTH instead")]
    pub const ATTRACTOR_STRENGTH: f32 = DEFAULT_ATTRACTOR_STRENGTH;
    #[deprecated(since = "0.2.0", note = "Use MIN_ATTRACTOR_DISTANCE instead")]
    pub const ATTRACTOR_MIN_DIST: f32 = MIN_ATTRACTOR_DISTANCE;
    #[deprecated(since = "0.2.0", note = "Use MIN_FORCE_THRESHOLD instead")]
    pub const FORCE_THRESHOLD: f32 = MIN_FORCE_THRESHOLD;
    /// Wind strength multiplier applied to wind vector.
    pub const DEFAULT_WIND_STRENGTH_MULTIPLIER: f32 = 0.05;
    /// Steer strength when applying wind force.
    pub const DEFAULT_WIND_STEER_STRENGTH: f32 = 0.3;
    /// Minimum wind strength to apply (avoids tiny adjustments).
    pub const MIN_WIND_STRENGTH: f32 = 0.0001;
    /// Terrain steering strength for smooth terrain.
    pub const DEFAULT_TERRAIN_STRENGTH_SMOOTH: f32 = 0.1;
    /// Terrain steering strength for turbulent terrain.
    pub const DEFAULT_TERRAIN_STRENGTH_TURBULENT: f32 = 0.2;
    /// Terrain steering strength for mixed terrain.
    pub const DEFAULT_TERRAIN_STRENGTH_MIXED: f32 = 0.15;
    /// Perlin noise scale for smooth terrain.
    pub const DEFAULT_TERRAIN_SCALE_SMOOTH: f32 = 0.005;
    /// Perlin noise scale for turbulent terrain.
    pub const DEFAULT_TERRAIN_SCALE_TURBULENT: f32 = 0.02;
    /// Offset added to noise coordinates for terrain variation.
    pub const DEFAULT_TERRAIN_NOISE_OFFSET: f32 = 100.0;
    // Backward compatibility aliases for terrain constants
    #[deprecated(since = "0.2.0", note = "Use DEFAULT_TERRAIN_STRENGTH_SMOOTH instead")]
    pub const TERRAIN_STRENGTH_SMOOTH: f32 = DEFAULT_TERRAIN_STRENGTH_SMOOTH;
    #[deprecated(
        since = "0.2.0",
        note = "Use DEFAULT_TERRAIN_STRENGTH_TURBULENT instead"
    )]
    pub const TERRAIN_STRENGTH_TURBULENT: f32 = DEFAULT_TERRAIN_STRENGTH_TURBULENT;
    #[deprecated(since = "0.2.0", note = "Use DEFAULT_TERRAIN_STRENGTH_MIXED instead")]
    pub const TERRAIN_STRENGTH_MIXED: f32 = DEFAULT_TERRAIN_STRENGTH_MIXED;
    #[deprecated(since = "0.2.0", note = "Use DEFAULT_TERRAIN_SCALE_SMOOTH instead")]
    pub const TERRAIN_SCALE_SMOOTH: f32 = DEFAULT_TERRAIN_SCALE_SMOOTH;
    #[deprecated(since = "0.2.0", note = "Use DEFAULT_TERRAIN_SCALE_TURBULENT instead")]
    pub const TERRAIN_SCALE_TURBULENT: f32 = DEFAULT_TERRAIN_SCALE_TURBULENT;
    #[deprecated(since = "0.2.0", note = "Use DEFAULT_TERRAIN_NOISE_OFFSET instead")]
    pub const TERRAIN_NOISE_OFFSET: f32 = DEFAULT_TERRAIN_NOISE_OFFSET;
    #[deprecated(since = "0.2.0", note = "Use MIN_WIND_STRENGTH instead")]
    pub const WIND_MIN_STRENGTH: f32 = MIN_WIND_STRENGTH;
    #[deprecated(since = "0.2.0", note = "Use DEFAULT_WIND_STEER_STRENGTH instead")]
    pub const WIND_STEER_STRENGTH: f32 = DEFAULT_WIND_STEER_STRENGTH;
    #[deprecated(since = "0.2.0", note = "Use DEFAULT_WIND_STRENGTH_MULTIPLIER instead")]
    pub const WIND_STRENGTH_MULTIPLIER: f32 = DEFAULT_WIND_STRENGTH_MULTIPLIER;
}

/// Trail map constants.
pub mod trail {
    /// Default decay factor (0.0-1.0).
    pub const DEFAULT_DECAY_FACTOR: f32 = 0.5;
    /// Default diffusion sigma for Gaussian kernel.
    pub const DEFAULT_DIFFUSION_SIGMA: f32 = 1.0;
    /// Default max brightness for normalization.
    pub const DEFAULT_MAX_BRIGHTNESS: f32 = 100.0;

    /// Minimum decay factor.
    pub const MIN_DECAY_FACTOR: f32 = 0.5;
    /// Maximum decay factor.
    pub const MAX_DECAY_FACTOR: f32 = 0.9999;
    /// Minimum diffusion sigma.
    pub const MIN_DIFFUSION_SIGMA: f32 = 0.5;
    /// Maximum diffusion sigma.
    pub const MAX_DIFFUSION_SIGMA: f32 = 2.0;
    /// Minimum max brightness.
    pub const MIN_MAX_BRIGHTNESS: f32 = 1.0;
    /// Maximum max brightness.
    pub const MAX_MAX_BRIGHTNESS: f32 = 1000.0;
}

/// Population constants.
pub mod population {
    /// Default agent count.
    pub const DEFAULT_POPULATION: usize = 50_000;
    /// Default species count.
    pub const DEFAULT_SPECIES_COUNT: usize = 1;

    /// Minimum total population.
    pub const MIN_POPULATION: usize = 1000;
    /// Maximum total population.
    pub const MAX_POPULATION: usize = 200_000;
    /// Minimum species count per species.
    pub const MIN_SPECIES_COUNT: usize = 100;
    /// Maximum species count per species.
    pub const MAX_SPECIES_COUNT: usize = 200_000;

    // Backward compatibility aliases
    #[deprecated(since = "0.2.0", note = "Use DEFAULT_POPULATION instead")]
    pub const DEFAULT_COUNT: usize = DEFAULT_POPULATION;
    #[deprecated(since = "0.2.0", note = "Use MIN_POPULATION instead")]
    pub const MIN_TOTAL: usize = MIN_POPULATION;
    #[deprecated(since = "0.2.0", note = "Use MAX_POPULATION instead")]
    pub const MAX_TOTAL: usize = MAX_POPULATION;
}

/// Environmental constants.
pub mod environment {
    /// Default mouse attractor timeout in seconds.
    pub const DEFAULT_MOUSE_TIMEOUT: f32 = 3.0;
    /// Default attractor strength multiplier.
    pub const DEFAULT_ATTRACTOR_STRENGTH: f32 = 1.0;
    /// Default terrain strength.
    pub const DEFAULT_TERRAIN_STRENGTH: f32 = 1.0;

    /// Minimum attractor strength.
    pub const MIN_ATTRACTOR_STRENGTH: f32 = 0.1;
    /// Maximum attractor strength.
    pub const MAX_ATTRACTOR_STRENGTH: f32 = 10.0;
    /// Minimum terrain strength.
    pub const MIN_TERRAIN_STRENGTH: f32 = 0.1;
    /// Maximum terrain strength.
    pub const MAX_TERRAIN_STRENGTH: f32 = 5.0;
    /// Wind component range (-1.0 to 1.0).
    pub const MIN_WIND_COMPONENT: f32 = -1.0;
    /// Wind component range (-1.0 to 1.0).
    pub const MAX_WIND_COMPONENT: f32 = 1.0;
    /// Minimum wind vector magnitude (for validation).
    pub const MIN_WIND_MAGNITUDE: f32 = 0.001;

    // Backward compatibility aliases (inconsistent naming in original code)
    #[deprecated(since = "0.2.0", note = "Use MIN_ATTRACTOR_STRENGTH instead")]
    pub const ATTRACTOR_STRENGTH_MIN: f32 = MIN_ATTRACTOR_STRENGTH;
    #[deprecated(since = "0.2.0", note = "Use MAX_ATTRACTOR_STRENGTH instead")]
    pub const ATTRACTOR_STRENGTH_MAX: f32 = MAX_ATTRACTOR_STRENGTH;
}

/// Time scaling constants.
pub mod time {
    /// Default time scale multiplier.
    pub const DEFAULT_TIME_SCALE: f32 = 1.0;
    /// Default FPS target.
    pub const DEFAULT_FPS: u32 = 30;
    /// Default frame delay in seconds (33.3ms for 30 FPS).
    pub const DEFAULT_FRAME_DELAY: f32 = 1.0 / 30.0;
    /// Reference time step for FPS calculations (30 FPS).
    pub const DEFAULT_REFERENCE_TIME_STEP: f32 = 1.0 / 30.0;

    /// Minimum time scale.
    pub const MIN_TIME_SCALE: f32 = 0.1;
    /// Maximum time scale.
    pub const MAX_TIME_SCALE: f32 = 10.0;
    /// Minimum FPS.
    pub const MIN_FPS: u32 = 1;
    /// Maximum FPS.
    pub const MAX_FPS: u32 = 144;
}

/// Wind direction constants.
pub mod wind {
    /// Cardinal direction wind strength.
    pub const DEFAULT_CARDINAL_STRENGTH: f32 = 1.0;
    /// Diagonal direction wind strength.
    pub const DEFAULT_DIAGONAL_STRENGTH: f32 = 0.7;
}

/// Food image initialization constants.
pub mod food {
    /// Default food image scale factor.
    pub const DEFAULT_FOOD_SCALE: f32 = 1.5;
    /// Default food image invert setting.
    pub const DEFAULT_FOOD_INVERT: bool = true;
    /// Default food image path.
    pub const DEFAULT_FOOD_PATH: &str = "assets/tslime_logo.png";

    // Backward compatibility aliases
    #[deprecated(since = "0.2.0", note = "Use DEFAULT_FOOD_SCALE instead")]
    pub const DEFAULT_SCALE: f32 = DEFAULT_FOOD_SCALE;
    #[deprecated(since = "0.2.0", note = "Use DEFAULT_FOOD_INVERT instead")]
    pub const DEFAULT_INVERT: bool = DEFAULT_FOOD_INVERT;
    #[deprecated(since = "0.2.0", note = "Use DEFAULT_FOOD_PATH instead")]
    pub const DEFAULT_PATH: &str = DEFAULT_FOOD_PATH;
}

/// Logo/pause screen image constants.
pub mod logo {
    /// Default logo scale factor (for pause screen display).
    pub const DEFAULT_LOGO_SCALE: f32 = 1.5;
    /// Default logo invert setting.
    pub const DEFAULT_LOGO_INVERT: bool = true;
}

/// Rendering threshold constants.
pub mod threshold {
    /// Edge detection threshold for rendering.
    pub const DEFAULT_EDGE_THRESHOLD: f32 = 0.05;
    /// Minimum brightness threshold for rendering.
    pub const DEFAULT_BRIGHTNESS_MIN: f32 = 0.01;
    /// Noise amount for scanline effects (±percentage of brightness).
    pub const DEFAULT_NOISE_AMOUNT: f32 = 0.08;
    /// Minimum factor for scanline darkening.
    pub const MIN_SCANLINE_FACTOR: f32 = 0.05;
    /// Maximum factor for scanline darkening.
    pub const MAX_SCANLINE_FACTOR: f32 = 0.6;
    /// Default brightness threshold for point grid mode.
    pub const DEFAULT_POINT_THRESHOLD: f32 = 0.15;
    /// Default brightness threshold for braille mode.
    pub const DEFAULT_BRAILLE_THRESHOLD: f32 = 0.05;
    /// Default brightness threshold for quadrant mode.
    pub const DEFAULT_QUADRANT_THRESHOLD: f32 = 0.05;
    /// Default brightness threshold for vertical block mode.
    pub const DEFAULT_VERTICAL_BLOCK_THRESHOLD: f32 = 0.05;
    /// Threshold for sculpted outline mode.
    pub const DEFAULT_SCULPTED_OUTLINE_THRESHOLD: f32 = 0.05;
    /// Threshold for dithering edge detection.
    pub const DEFAULT_DITHER_EDGE_THRESHOLD: f32 = 0.15;
    /// Minimum contrast difference for directional enhancement.
    pub const DEFAULT_CONTRAST_DELTA: f32 = 0.05;

    // Backward compatibility aliases
    #[deprecated(since = "0.2.0", note = "Use DEFAULT_BRAILLE_THRESHOLD instead")]
    pub const BRAILLE_DEFAULT: f32 = DEFAULT_BRAILLE_THRESHOLD;
    #[deprecated(since = "0.2.0", note = "Use DEFAULT_POINT_THRESHOLD instead")]
    pub const POINT_DEFAULT: f32 = DEFAULT_POINT_THRESHOLD;
    #[deprecated(since = "0.2.0", note = "Use DEFAULT_VERTICAL_BLOCK_THRESHOLD instead")]
    pub const VERTICAL_BLOCK_DEFAULT: f32 = DEFAULT_VERTICAL_BLOCK_THRESHOLD;
    #[deprecated(
        since = "0.2.0",
        note = "Use DEFAULT_SCULPTED_OUTLINE_THRESHOLD instead"
    )]
    pub const SCULPTED_OUTLINE: f32 = DEFAULT_SCULPTED_OUTLINE_THRESHOLD;
}

/// Intensity mapping constants.
pub mod intensity {
    /// Number of discrete intensity levels for quantization.
    pub const DEFAULT_QUANTIZE_LEVELS: u32 = 6;
    /// Default log base for logarithmic intensity mapping.
    pub const DEFAULT_LOG_BASE: f32 = 10.0;
    /// Default exponent for exponential intensity mapping.
    pub const DEFAULT_EXP_EXPONENT: f32 = 10.0;
    /// Perlin noise frequency for perlin intensity mapping.
    pub const DEFAULT_PERLIN_FREQUENCY: f32 = 0.15;
    /// Perlin noise octaves for perlin intensity mapping.
    pub const DEFAULT_PERLIN_OCTAVES: u32 = 4;
    /// Perlin noise seed for perlin intensity mapping.
    pub const DEFAULT_PERLIN_SEED: u64 = 42;

    // Backward compatibility aliases
    #[deprecated(since = "0.2.0", note = "Use DEFAULT_LOG_BASE instead")]
    pub const LOG_DEFAULT: f32 = DEFAULT_LOG_BASE;
    #[deprecated(since = "0.2.0", note = "Use DEFAULT_PERLIN_SEED instead")]
    pub const PERLIN_SEED: u64 = DEFAULT_PERLIN_SEED;
}

/// Color palette constants.
pub mod palette {
    /// Default palette name.
    pub const DEFAULT_PALETTE_NAME: &str = "moss";
    /// Number of gradient steps in built-in palettes.
    pub const DEFAULT_PALETTE_STEPS: usize = 11;
    /// Minimum number of colors in custom palette.
    pub const MIN_CUSTOM_COLORS: usize = 2;
    /// Maximum number of colors in custom palette.
    pub const MAX_CUSTOM_COLORS: usize = DEFAULT_PALETTE_STEPS;
    /// Default grid color (white).
    pub const DEFAULT_GRID_COLOR: &str = "ffffff";
    /// Number of palette colors that use linear intensity mapping before switching to logarithmic.
    pub const LINEAR_COLOR_COUNT: f32 = 6.0;

    // Backward compatibility aliases
    #[deprecated(since = "0.2.0", note = "Use DEFAULT_PALETTE_STEPS instead")]
    pub const PALETTE_STEPS: usize = DEFAULT_PALETTE_STEPS;
}

/// Grid rendering constants.
pub mod grid {
    /// Default grid size (cell spacing).
    pub const DEFAULT_GRID_SIZE: usize = 10;
    /// Default grid opacity for rendering.
    pub const DEFAULT_GRID_OPACITY: f32 = 0.15;
    /// Default grid line style.
    pub const DEFAULT_GRID_STYLE: &str = "cross";

    /// Minimum grid size.
    pub const MIN_GRID_SIZE: usize = 1;
    /// Maximum grid size.
    pub const MAX_GRID_SIZE: usize = 50;
}

/// Terminal rendering constants.
pub mod terminal {
    /// Default simulation resolution width.
    pub const DEFAULT_RESOLUTION_WIDTH: usize = 400;
    /// Default simulation resolution height.
    pub const DEFAULT_RESOLUTION_HEIGHT: usize = 200;
    /// Default frame delay in milliseconds.
    pub const DEFAULT_FRAME_DELAY_MS: u64 = 0;
    /// Small terminal width threshold (columns).
    pub const SMALL_TERMINAL_WIDTH: usize = 80;
    /// Medium terminal width threshold (columns).
    pub const MEDIUM_TERMINAL_WIDTH: usize = 120;
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
    pub const DEFAULT_FRAMES: [usize; 4] = [0, 3, 5, 7];
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
    pub const DEFAULT_HYBRID_EDGE_THRESHOLD: f32 = 0.15;

    // Backward compatibility alias
    #[deprecated(since = "0.2.0", note = "Use DEFAULT_HYBRID_EDGE_THRESHOLD instead")]
    pub const DEFAULT_HYBRID_EDGE: f32 = DEFAULT_HYBRID_EDGE_THRESHOLD;
}

/// ASCII contrast constants.
pub mod ascii {
    /// Default ASCII contrast value.
    pub const DEFAULT_CONTRAST: f32 = 1.5;
    /// Minimum contrast value (1.0 = no enhancement).
    pub const MIN_CONTRAST: f32 = 1.0;
    /// Maximum contrast value (2.0+ = strong enhancement).
    pub const MAX_CONTRAST: f32 = 2.0;
}

/// Rendering-specific constants (backward compatibility module).
pub mod rendering {
    use super::{ascii, grid};
    /// Default grid opacity (0.0-1.0).
    pub const GRID_OPACITY_DEFAULT: f32 = grid::DEFAULT_GRID_OPACITY;
    /// Default ASCII contrast for shape-vector rendering.
    pub const ASCII_CONTRAST_DEFAULT: f32 = ascii::DEFAULT_CONTRAST;
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
    pub const DEFAULT_WINDOW_SIZE: usize = 100;
    /// Small window size.
    pub const SMALL_WINDOW: usize = 50;
    /// Large window size.
    pub const LARGE_WINDOW: usize = 200;

    // Backward compatibility aliases
    #[deprecated(since = "0.2.0", note = "Use LARGE_WINDOW instead")]
    pub const WINDOW_LARGE: usize = LARGE_WINDOW;
    #[deprecated(since = "0.2.0", note = "Use DEFAULT_WINDOW_SIZE instead")]
    pub const DEFAULT_WINDOW: usize = DEFAULT_WINDOW_SIZE;
}

/// Mathematical and precision constants.
pub mod math {
    /// Epsilon value for floating point comparisons.
    pub const EPSILON: f32 = 1e-6;
    /// Degrees in a full circle (360.0).
    pub const DEGREES_IN_CIRCLE: f32 = 360.0;
    /// Half degrees in a circle (180.0), used for hue rotation.
    pub const DEGREES_HALF_CIRCLE: f32 = 180.0;
    /// Center point for sigmoid curve (maps to 0.5 output).
    pub const SIGMOID_CENTER: f32 = 0.5;
    /// Minimum steepness to prevent numerical issues in sigmoid.
    pub const SIGMOID_MIN_STEEPNESS: f32 = 0.1;
}

/// HSV color space constants.
pub mod hsv {
    /// Minimum saturation for species colors (prevents completely desaturated colors).
    pub const MIN_SATURATION: f32 = 0.05;
    /// Base maximum saturation floor for species colors.
    pub const MAX_SATURATION_FLOOR: f32 = 0.10;
    /// Minimum value/brightness for species colors (prevents pure black).
    pub const MIN_VALUE: f32 = 0.08;
    /// Scaling factor for maximum value computation.
    pub const MAX_VALUE_SCALE: f32 = 0.9;
    /// Offset added when computing maximum value.
    pub const MAX_VALUE_OFFSET: f32 = 0.1;
    /// Hard cap on maximum value to prevent oversaturation.
    pub const MAX_VALUE_CAP: f32 = 0.95;
}

/// Color defaults.
pub mod color {
    /// Default forest green color (used for food and agents).
    pub const DEFAULT_FOOD_COLOR: &str = "228b22";
    /// Default agent color.
    pub const DEFAULT_AGENT_COLOR: &str = "228b22";
    /// White color hex code.
    pub const WHITE: &str = "ffffff";
    /// Black color hex code.
    pub const BLACK: &str = "000000";
}

/// Warmup defaults.
pub mod warmup {
    /// Default number of warmup frames before full speed.
    pub const DEFAULT_WARMUP_FRAMES: usize = 60;
    /// Default brightness multiplier during warmup.
    pub const DEFAULT_BRIGHTNESS_MULTIPLIER: f32 = 2.5;
    /// Default decay factor during warmup.
    pub const DEFAULT_DECAY_FACTOR: f32 = 0.99;
}

/// Food persistence defaults.
pub mod food_persist {
    /// Default strength of persistent food attractors.
    pub const DEFAULT_STRENGTH: f32 = 0.3;
    /// Default radius of persistent food attractors.
    pub const DEFAULT_RADIUS: f32 = 50.0;
    /// Default duration of food persistence in frames.
    pub const DEFAULT_DURATION: usize = 300;
}

/// Auto-reset defaults.
pub mod auto_reset {
    /// Default entropy threshold for collapse detection.
    pub const DEFAULT_ENTROPY_THRESHOLD: f32 = 0.95;
    /// Default number of frames below threshold before reset.
    pub const DEFAULT_DURATION_FRAMES: usize = 90;
}

/// Export defaults.
pub mod export {
    /// Default number of frames to export.
    pub const DEFAULT_FRAMES: usize = 50;
    /// Default FPS for exported video.
    pub const DEFAULT_FPS: usize = 30;
    /// Default simulation steps to skip between frames.
    pub const DEFAULT_FRAME_SKIP: usize = 50;
    /// Default number of frames to capture.
    pub const DEFAULT_FRAME_COUNT: usize = 50;
    /// Default directory for captured frames.
    pub const DEFAULT_FRAME_DIR: &str = "frames";
}

/// Trail age hue shift mode for visual effects.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TrailAgeMode {
    /// Bidirectional shift: age=0 → -range/2, age=0.5 → 0, age=1 → +range/2
    /// Preserves color balance by shifting on both sides of base hue
    #[default]
    Bidirectional,
    /// Alternating spatial pattern: shifts vary by cell position
    /// Creates organic variation with complementary shifts
    Alternating,
}

impl TrailAgeMode {
    /// Returns the display name of the trail age mode.
    pub fn name(&self) -> &str {
        match self {
            TrailAgeMode::Bidirectional => "Bidirectional",
            TrailAgeMode::Alternating => "Alternating",
        }
    }
}

/// Visual effects constants for trail age, temporal delta, and Laplacian sharpening.
pub mod visual_fx {
    /// Maximum trail age in seconds before clamping.
    pub const AGE_MAX_SECONDS: f32 = 10.0;
    /// Hue shift range in degrees for aged trails.
    pub const AGE_HUE_RANGE: f32 = 60.0;
    /// Brightness boost strength for temporal delta effect.
    pub const DELTA_STRENGTH: f32 = 0.5;
    /// Laplacian sharpening strength.
    pub const SHARPEN_STRENGTH: f32 = 0.3;
}

/// Simulation defaults.
pub mod simulation {
    /// Default number of exploration iterations.
    pub const DEFAULT_EXPLORE_ITERATIONS: usize = 100;
}

/// Intensity mapping configuration defaults.
pub mod intensity_mapping {
    /// Default intensity mapping type.
    pub const DEFAULT_TYPE: &str = "log";
    /// Default base for logarithmic intensity mapping.
    pub const DEFAULT_LOG_BASE: f32 = 10.0;
    /// Default gamma correction value.
    pub const DEFAULT_GAMMA: f32 = 2.2;
    /// Default number of quantization levels.
    pub const DEFAULT_LEVELS: u8 = 8;
    /// Default strength of Perlin noise effects.
    pub const DEFAULT_PERLIN_STRENGTH: f32 = 0.2;
    /// Default base for logo brightness mapping.
    pub const DEFAULT_LOGO_BASE: f32 = 4.0;
}

/// Dithering configuration defaults.
pub mod dithering {
    /// Default dithering algorithm mode.
    pub const DEFAULT_MODE: &str = "none";
    /// Default intensity of dithering effect.
    pub const DEFAULT_INTENSITY: f32 = 0.5;
    /// Default dithering matrix size.
    pub const DEFAULT_MATRIX: &str = "4x4";
}

/// Terminal color mode defaults.
pub mod color_mode {
    /// Default terminal color mode (true color).
    pub const DEFAULT_MODE: &str = "true";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_defaults() {
        assert_eq!(agent::DEFAULT_SENSOR_ANGLE, 22.5);
        assert_eq!(agent::DEFAULT_ROTATION_ANGLE, 45.0);
        assert_eq!(agent::DEFAULT_SENSOR_DISTANCE, 9.0);
        assert_eq!(agent::DEFAULT_STEP_SIZE, 1.0);
        assert_eq!(agent::DEFAULT_DEPOSIT_AMOUNT, 5.0);
    }

    #[test]
    fn test_agent_ranges() {
        assert_eq!(agent::MIN_SENSOR_ANGLE, 5.0);
        assert_eq!(agent::MAX_SENSOR_ANGLE, 90.0);
        assert_eq!(agent::MIN_STEP_SIZE, 0.01);
        assert_eq!(agent::MAX_STEP_SIZE, 10.0);
    }

    #[test]
    fn test_population_defaults() {
        assert_eq!(population::DEFAULT_POPULATION, 50_000);
        assert_eq!(population::MIN_POPULATION, 1000);
        assert_eq!(population::MAX_POPULATION, 200_000);
    }

    #[test]
    fn test_time_defaults() {
        assert_eq!(time::DEFAULT_FPS, 30);
        assert_eq!(time::MIN_FPS, 1);
        assert_eq!(time::MAX_FPS, 144);
    }

    #[test]
    fn test_palette_constants() {
        assert_eq!(palette::DEFAULT_PALETTE_NAME, "moss");
        assert_eq!(palette::DEFAULT_PALETTE_STEPS, 11);
        assert_eq!(palette::LINEAR_COLOR_COUNT, 6.0);
    }

    #[test]
    fn test_color_defaults() {
        assert_eq!(color::DEFAULT_FOOD_COLOR, "228b22");
        assert_eq!(palette::DEFAULT_GRID_COLOR, "ffffff");
    }
}
