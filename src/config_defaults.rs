//! Centralized configuration defaults for tslime.
//!
//! This module provides a single source of truth for all configuration default values,
//! ensuring consistency between CLI help text, SimConfig defaults, and documentation.

use crate::simulation::constants::{
    agent as agent_consts, env as env_consts, food_image as food_consts, population as pop_consts,
    time as time_consts, trail as trail_consts,
};

/// Configuration defaults for the simulation.
///
/// All fields are constant and derived from the centralized constants in
/// `simulation/constants.rs`. This ensures consistency across the codebase.
pub struct ConfigDefaults;

impl ConfigDefaults {
    // Agent behavior defaults

    /// Default angle between left/right sensors in degrees.
    pub const SENSOR_ANGLE: f32 = agent_consts::DEFAULT_SENSOR_ANGLE;
    /// Default maximum turn amount per step in degrees.
    pub const ROTATION_ANGLE: f32 = agent_consts::DEFAULT_ROTATION_ANGLE;
    /// Default distance ahead agents can sense pheromones.
    pub const SENSOR_DISTANCE: f32 = agent_consts::DEFAULT_SENSOR_DISTANCE;
    /// Default distance agents move per simulation step.
    pub const STEP_SIZE: f32 = agent_consts::DEFAULT_STEP_SIZE;
    /// Default amount of pheromone deposited by agents per step.
    pub const DEPOSIT_AMOUNT: f32 = agent_consts::DEFAULT_DEPOSIT_AMOUNT;
    /// Default number of agents in the simulation.
    pub const POPULATION: usize = pop_consts::DEFAULT_COUNT;

    // Trail map defaults

    /// Default trail persistence multiplier applied each frame.
    pub const DECAY_FACTOR: f32 = trail_consts::DEFAULT_DECAY_FACTOR;
    /// Default maximum brightness for normalization.
    pub const MAX_BRIGHTNESS: f32 = trail_consts::DEFAULT_MAX_BRIGHTNESS;
    /// Default Gaussian blur sigma for diffusion.
    pub const DIFFUSION_SIGMA: f32 = trail_consts::DEFAULT_DIFFUSION_SIGMA;

    // Time and performance defaults

    /// Default speed multiplier for simulation time.
    pub const TIME_SCALE: f32 = time_consts::DEFAULT_TIME_SCALE;
    /// Default target frames per second.
    pub const FPS: usize = 30;
    /// Default frame delay in seconds (33.3ms for 30 FPS).
    pub const FRAME_DELAY: f32 = 1.0 / 30.0;

    // Environmental defaults

    /// Default strength of point attractors.
    pub const ATTRACTOR_STRENGTH: f32 = env_consts::DEFAULT_ATTRACTOR_STRENGTH;
    /// Default intensity of terrain influence on movement.
    pub const TERRAIN_STRENGTH: f32 = env_consts::DEFAULT_TERRAIN_STRENGTH;
    /// Default timeout for mouse effects in seconds.
    pub const MOUSE_TIMEOUT: f32 = env_consts::DEFAULT_MOUSE_TIMEOUT;

    // Rendering defaults

    /// Default simulation grid width in pixels.
    pub const RESOLUTION_WIDTH: usize = 400;
    /// Default simulation grid height in pixels.
    pub const RESOLUTION_HEIGHT: usize = 200;
    /// Default color palette name.
    pub const PALETTE: &str = "moss";
    /// Default terminal color mode.
    pub const COLOR_MODE: &str = "true";

    // Food/Logo defaults

    /// Default scale factor for food image.
    pub const FOOD_SCALE: f32 = food_consts::DEFAULT_SCALE;
    /// Default invert flag for food image.
    pub const FOOD_INVERT: bool = food_consts::DEFAULT_INVERT;
    /// Default path to food image.
    pub const FOOD_PATH: &str = food_consts::DEFAULT_PATH;

    // Intensity mapping defaults

    /// Default intensity mapping type.
    pub const INTENSITY_MAPPING: &str = "log";
    /// Default base for logarithmic intensity mapping.
    pub const INTENSITY_MAPPING_BASE: f32 = 10.0;
    /// Default gamma correction value.
    pub const INTENSITY_MAPPING_GAMMA: f32 = 2.2;
    /// Default number of quantization levels.
    pub const INTENSITY_MAPPING_LEVELS: u8 = 8;
    /// Default strength of Perlin noise effects.
    pub const PERLIN_STRENGTH: f32 = 0.2;
    /// Default base for logo brightness mapping.
    pub const LOGO_MAPPING_BASE: f32 = 4.0;

    // Dithering defaults

    /// Default dithering algorithm mode.
    pub const DITHER_MODE: &str = "none";
    /// Default intensity of dithering effect.
    pub const DITHER_INTENSITY: f32 = 0.5;
    /// Default dithering matrix size.
    pub const DITHER_MATRIX: &str = "4x4";

    // Grid defaults

    /// Default grid cell size in characters.
    pub const GRID_SIZE: usize = 10;
    /// Default grid line style.
    pub const GRID_STYLE: &str = "cross";
    /// Default grid color in hex.
    pub const GRID_COLOR: &str = "ffffff";
    /// Default grid line opacity (0.0-1.0).
    pub const GRID_OPACITY: f32 = 0.15;

    // Warmup defaults

    /// Default number of warmup frames before full speed.
    pub const WARMUP_FRAMES: usize = 60;
    /// Default brightness multiplier during warmup.
    pub const WARMUP_BRIGHTNESS_MULTIPLIER: f32 = 2.5;
    /// Default decay factor during warmup.
    pub const WARMUP_DECAY: f32 = 0.99;

    // Food persistence defaults

    /// Default strength of persistent food attractors.
    pub const FOOD_PERSIST_STRENGTH: f32 = 0.3;
    /// Default radius of persistent food attractors.
    pub const FOOD_PERSIST_RADIUS: f32 = 50.0;
    /// Default duration of food persistence in frames.
    pub const FOOD_PERSIST_DURATION: usize = 300;

    // Auto-reset defaults

    /// Default entropy threshold for collapse detection.
    pub const COLLAPSE_ENTROPY_THRESHOLD: f32 = 0.95;
    /// Default number of frames below threshold before reset.
    pub const COLLAPSE_DURATION_FRAMES: usize = 90;

    // Export defaults

    /// Default number of frames to export.
    pub const EXPORT_FRAMES: usize = 50;
    /// Default FPS for exported video.
    pub const EXPORT_FPS: usize = 30;
    /// Default simulation steps to skip between frames.
    pub const FRAME_SKIP: usize = 50;
    /// Default number of frames to capture.
    pub const FRAME_COUNT: usize = 50;
    /// Default directory for captured frames.
    pub const FRAME_DIR: &str = "frames";

    // ASCII rendering defaults

    /// Default contrast multiplier for ASCII rendering.
    pub const ASCII_CONTRAST: f32 = 1.5;

    // Simulation defaults

    /// Default number of exploration iterations.
    pub const EXPLORE_ITERATIONS: usize = 100;

    // Range limits (for validation)

    /// Minimum valid sensor angle in degrees.
    pub const MIN_SENSOR_ANGLE: f32 = agent_consts::MIN_SENSOR_ANGLE;
    /// Maximum valid sensor angle in degrees.
    pub const MAX_SENSOR_ANGLE: f32 = agent_consts::MAX_SENSOR_ANGLE;
    /// Minimum valid sensor distance.
    pub const MIN_SENSOR_DISTANCE: f32 = agent_consts::MIN_SENSOR_DISTANCE;
    /// Maximum valid sensor distance.
    pub const MAX_SENSOR_DISTANCE: f32 = agent_consts::MAX_SENSOR_DISTANCE;
    /// Minimum valid rotation angle in degrees.
    pub const MIN_ROTATION_ANGLE: f32 = agent_consts::MIN_ROTATION_ANGLE;
    /// Maximum valid rotation angle in degrees.
    pub const MAX_ROTATION_ANGLE: f32 = agent_consts::MAX_ROTATION_ANGLE;
    /// Minimum valid step size.
    pub const MIN_STEP_SIZE: f32 = agent_consts::MIN_STEP_SIZE;
    /// Maximum valid step size.
    pub const MAX_STEP_SIZE: f32 = agent_consts::MAX_STEP_SIZE;
    /// Minimum valid decay factor.
    pub const MIN_DECAY_FACTOR: f32 = trail_consts::MIN_DECAY_FACTOR;
    /// Maximum valid decay factor.
    pub const MAX_DECAY_FACTOR: f32 = trail_consts::MAX_DECAY_FACTOR;
    /// Minimum valid deposit amount.
    pub const MIN_DEPOSIT_AMOUNT: f32 = agent_consts::MIN_DEPOSIT_AMOUNT;
    /// Maximum valid deposit amount.
    pub const MAX_DEPOSIT_AMOUNT: f32 = agent_consts::MAX_DEPOSIT_AMOUNT;
    /// Minimum valid max brightness.
    pub const MIN_MAX_BRIGHTNESS: f32 = trail_consts::MIN_MAX_BRIGHTNESS;
    /// Maximum valid max brightness.
    pub const MAX_MAX_BRIGHTNESS: f32 = trail_consts::MAX_MAX_BRIGHTNESS;
    /// Minimum valid time scale.
    pub const MIN_TIME_SCALE: f32 = time_consts::MIN_TIME_SCALE;
    /// Maximum valid time scale.
    pub const MAX_TIME_SCALE: f32 = time_consts::MAX_TIME_SCALE;
    /// Minimum valid population count.
    pub const MIN_POPULATION: usize = pop_consts::MIN_TOTAL;
    /// Maximum valid population count.
    pub const MAX_POPULATION: usize = pop_consts::MAX_TOTAL;
    /// Minimum valid attractor strength.
    pub const MIN_ATTRACTOR_STRENGTH: f32 = env_consts::MIN_ATTRACTOR_STRENGTH;
    /// Maximum valid attractor strength.
    pub const MAX_ATTRACTOR_STRENGTH: f32 = env_consts::MAX_ATTRACTOR_STRENGTH;
    /// Minimum valid terrain strength.
    pub const MIN_TERRAIN_STRENGTH: f32 = env_consts::MIN_TERRAIN_STRENGTH;
    /// Maximum valid terrain strength.
    pub const MAX_TERRAIN_STRENGTH: f32 = env_consts::MAX_TERRAIN_STRENGTH;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_defaults_match_constants() {
        // Ensure ConfigDefaults matches the underlying constants
        assert_eq!(ConfigDefaults::SENSOR_ANGLE, 22.5);
        assert_eq!(ConfigDefaults::SENSOR_DISTANCE, 9.0);
        assert_eq!(ConfigDefaults::ROTATION_ANGLE, 45.0);
        assert_eq!(ConfigDefaults::STEP_SIZE, 1.0);
        assert_eq!(ConfigDefaults::DECAY_FACTOR, 0.5);
        assert_eq!(ConfigDefaults::DEPOSIT_AMOUNT, 5.0);
        assert_eq!(ConfigDefaults::MAX_BRIGHTNESS, 100.0);
        assert_eq!(ConfigDefaults::POPULATION, 50_000);
        assert_eq!(ConfigDefaults::TIME_SCALE, 1.0);
        assert_eq!(ConfigDefaults::ATTRACTOR_STRENGTH, 1.0);
        assert_eq!(ConfigDefaults::TERRAIN_STRENGTH, 1.0);
        assert_eq!(ConfigDefaults::MOUSE_TIMEOUT, 3.0);
    }

    #[test]
    fn test_range_limits() {
        // Ensure range limits are correctly exposed
        assert_eq!(ConfigDefaults::MIN_SENSOR_ANGLE, 5.0);
        assert_eq!(ConfigDefaults::MAX_SENSOR_ANGLE, 90.0);
        assert_eq!(ConfigDefaults::MIN_SENSOR_DISTANCE, 1.0);
        assert_eq!(ConfigDefaults::MAX_SENSOR_DISTANCE, 50.0);
        assert_eq!(ConfigDefaults::MIN_STEP_SIZE, 0.01);
        assert_eq!(ConfigDefaults::MAX_STEP_SIZE, 10.0);
        assert_eq!(ConfigDefaults::MIN_DECAY_FACTOR, 0.5);
        assert_eq!(ConfigDefaults::MAX_DECAY_FACTOR, 0.9999);
        assert_eq!(ConfigDefaults::MIN_DEPOSIT_AMOUNT, 0.1);
        assert_eq!(ConfigDefaults::MAX_DEPOSIT_AMOUNT, 20.0);
        assert_eq!(ConfigDefaults::MIN_MAX_BRIGHTNESS, 1.0);
        assert_eq!(ConfigDefaults::MAX_MAX_BRIGHTNESS, 1000.0);
        assert_eq!(ConfigDefaults::MIN_TIME_SCALE, 0.1);
        assert_eq!(ConfigDefaults::MAX_TIME_SCALE, 10.0);
        assert_eq!(ConfigDefaults::MIN_POPULATION, 1000);
        assert_eq!(ConfigDefaults::MAX_POPULATION, 200_000);
    }
}
