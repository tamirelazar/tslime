//! Centralized constants for simulation parameters.
//!
//! This module consolidates all magic numbers related to agent behavior,
//! steering forces, terrain effects, and other simulation parameters.

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

    /// Default deposit amount per step.
    pub const DEFAULT_DEPOSIT_AMOUNT: f32 = 5.0;
}

/// Steering force constants for various external influences.
pub mod steering {
    /// Strength of attractor/repeller steering.
    pub const ATTRACTOR_STRENGTH: f32 = 0.1;

    /// Minimum distance for attractor force calculation (prevents division by zero).
    pub const ATTRACTOR_MIN_DIST: f32 = 1.0;

    /// Minimum force threshold for applying steering.
    pub const FORCE_THRESHOLD: f32 = 0.001;

    /// Wind strength multiplier applied to wind vector.
    pub const WIND_STRENGTH_MULTIPLIER: f32 = 0.05;

    /// Steer strength when applying wind force.
    pub const WIND_STEER_STRENGTH: f32 = 0.3;

    /// Minimum wind strength to apply (avoids tiny adjustments).
    pub const WIND_MIN_STRENGTH: f32 = 0.0001;

    /// Terrain steering strength for smooth terrain.
    pub const TERRAIN_STRENGTH_SMOOTH: f32 = 0.1;

    /// Terrain steering strength for turbulent terrain.
    pub const TERRAIN_STRENGTH_TURBULENT: f32 = 0.2;

    /// Terrain steering strength for mixed terrain.
    pub const TERRAIN_STRENGTH_MIXED: f32 = 0.15;

    /// Perlin noise scale for smooth terrain.
    pub const TERRAIN_SCALE_SMOOTH: f32 = 0.005;

    /// Perlin noise scale for turbulent terrain.
    pub const TERRAIN_SCALE_TURBULENT: f32 = 0.02;

    /// Offset added to noise coordinates for terrain variation.
    pub const TERRAIN_NOISE_OFFSET: f32 = 100.0;
}

/// Trail map constants.
pub mod trail {
    /// Default decay factor (0.0-1.0).
    pub const DEFAULT_DECAY_FACTOR: f32 = 0.5;

    /// Minimum decay factor.
    pub const MIN_DECAY_FACTOR: f32 = 0.5;

    /// Maximum decay factor.
    pub const MAX_DECAY_FACTOR: f32 = 0.9999;

    /// Default diffusion sigma for Gaussian kernel.
    pub const DEFAULT_DIFFUSION_SIGMA: f32 = 1.0;

    /// Minimum diffusion sigma.
    pub const MIN_DIFFUSION_SIGMA: f32 = 0.5;

    /// Maximum diffusion sigma.
    pub const MAX_DIFFUSION_SIGMA: f32 = 2.0;

    /// Default max brightness for normalization.
    pub const DEFAULT_MAX_BRIGHTNESS: f32 = 100.0;

    /// Minimum max brightness.
    pub const MIN_MAX_BRIGHTNESS: f32 = 1.0;

    /// Maximum max brightness.
    pub const MAX_MAX_BRIGHTNESS: f32 = 1000.0;
}

/// Population constants.
pub mod population {
    /// Default agent count.
    pub const DEFAULT_COUNT: usize = 50_000;

    /// Minimum total population.
    pub const MIN_TOTAL: usize = 1000;

    /// Maximum total population.
    pub const MAX_TOTAL: usize = 200_000;

    /// Minimum species count.
    pub const MIN_SPECIES_COUNT: usize = 100;

    /// Maximum species count per species.
    pub const MAX_SPECIES_COUNT: usize = 200_000;
}

/// Environmental constants.
pub mod env {
    /// Default mouse attractor timeout in seconds.
    pub const DEFAULT_MOUSE_TIMEOUT: f32 = 3.0;

    /// Default attractor strength multiplier.
    pub const DEFAULT_ATTRACTOR_STRENGTH: f32 = 1.0;

    /// Minimum attractor strength.
    pub const MIN_ATTRACTOR_STRENGTH: f32 = 0.1;

    /// Maximum attractor strength.
    pub const MAX_ATTRACTOR_STRENGTH: f32 = 10.0;

    /// Attractor strength range (for validation).
    pub const ATTRACTOR_STRENGTH_MIN: f32 = -10.0;

    /// Attractor strength range (for validation).
    pub const ATTRACTOR_STRENGTH_MAX: f32 = 10.0;

    /// Default terrain strength.
    pub const DEFAULT_TERRAIN_STRENGTH: f32 = 1.0;

    /// Minimum terrain strength.
    pub const MIN_TERRAIN_STRENGTH: f32 = 0.1;

    /// Maximum terrain strength.
    pub const MAX_TERRAIN_STRENGTH: f32 = 5.0;

    /// Wind component range (-1.0 to 1.0).
    pub const WIND_COMPONENT_MIN: f32 = -1.0;

    /// Wind component range (-1.0 to 1.0).
    pub const WIND_COMPONENT_MAX: f32 = 1.0;

    /// Minimum wind vector magnitude (for validation).
    pub const WIND_MIN_MAGNITUDE: f32 = 0.001;
}

/// Time scaling constants.
pub mod time {
    /// Default time scale multiplier.
    pub const DEFAULT_TIME_SCALE: f32 = 1.0;

    /// Minimum time scale.
    pub const MIN_TIME_SCALE: f32 = 0.1;

    /// Maximum time scale.
    pub const MAX_TIME_SCALE: f32 = 10.0;

    /// Reference time step for FPS calculations (30 FPS).
    pub const REFERENCE_TIME_STEP: f32 = 1.0 / 30.0;
}

/// Wind direction constants.
pub mod wind {
    /// Cardinal direction wind strength.
    pub const CARDINAL_STRENGTH: f32 = 1.0;

    /// Diagonal direction wind strength.
    pub const DIAGONAL_STRENGTH: f32 = 0.7;
}
