//! Configuration builder for creating SimConfig instances.
//!
//! This module provides a builder pattern for constructing SimConfig instances
//! from various sources (CLI arguments, saved configs, presets) with consistent
//! validation and default handling.

use crate::cli::{Args, AttractorArg, ObstacleArg, SpeciesArg, WindArg};
use crate::config_defaults::ConfigDefaults;
use crate::simulation::config::{
    Attractor, DiffusionKernel, Preset, SimConfig, SpeciesConfig, TerrainType, Wind,
};
use crate::simulation::constants::{
    agent as agent_consts, population as pop_consts, trail as trail_consts,
};

/// Error type for configuration building failures.
#[derive(Debug, Clone, PartialEq)]
pub enum ConfigBuilderError {
    /// Invalid resolution dimensions.
    InvalidResolution {
        /// The requested width in pixels.
        width: usize,
        /// The requested height in pixels.
        height: usize,
    },
    /// Invalid FPS value.
    InvalidFps {
        /// The requested frames per second.
        fps: usize,
    },
    /// Invalid population count.
    InvalidPopulation {
        /// The requested population count.
        pop: usize,
        /// The minimum allowed population.
        min: usize,
        /// The maximum allowed population.
        max: usize,
    },
    /// Invalid sensor angle.
    InvalidSensorAngle {
        /// The requested sensor angle in degrees.
        value: f32,
        /// The minimum allowed angle in degrees.
        min: f32,
        /// The maximum allowed angle in degrees.
        max: f32,
    },
    /// Invalid sensor distance.
    InvalidSensorDistance {
        /// The requested sensor distance.
        value: f32,
        /// The minimum allowed distance.
        min: f32,
        /// The maximum allowed distance.
        max: f32,
    },
    /// Invalid rotation angle.
    InvalidRotationAngle {
        /// The requested rotation angle in degrees.
        value: f32,
        /// The minimum allowed angle in degrees.
        min: f32,
        /// The maximum allowed angle in degrees.
        max: f32,
    },
    /// Invalid step size.
    InvalidStepSize {
        /// The requested step size.
        value: f32,
        /// The minimum allowed step size.
        min: f32,
        /// The maximum allowed step size.
        max: f32,
    },
    /// Invalid decay factor.
    InvalidDecayFactor {
        /// The requested decay factor.
        value: f32,
        /// The minimum allowed decay factor.
        min: f32,
        /// The maximum allowed decay factor.
        max: f32,
    },
    /// Invalid deposit amount.
    InvalidDepositAmount {
        /// The requested deposit amount.
        value: f32,
        /// The minimum allowed deposit amount.
        min: f32,
        /// The maximum allowed deposit amount.
        max: f32,
    },
    /// Invalid max brightness.
    InvalidMaxBrightness {
        /// The requested max brightness.
        value: f32,
        /// The minimum allowed max brightness.
        min: f32,
        /// The maximum allowed max brightness.
        max: f32,
    },
    /// Invalid diffusion sigma.
    InvalidDiffusionSigma {
        /// The requested diffusion sigma.
        value: f32,
        /// The minimum allowed diffusion sigma.
        min: f32,
        /// The maximum allowed diffusion sigma.
        max: f32,
    },
    /// Invalid time scale.
    InvalidTimeScale {
        /// The requested time scale.
        value: f32,
        /// The minimum allowed time scale.
        min: f32,
        /// The maximum allowed time scale.
        max: f32,
    },
    /// Invalid attractor strength.
    InvalidAttractorStrength {
        /// The requested attractor strength.
        value: f32,
        /// The minimum allowed attractor strength.
        min: f32,
        /// The maximum allowed attractor strength.
        max: f32,
    },
    /// Invalid terrain strength.
    InvalidTerrainStrength {
        /// The requested terrain strength.
        value: f32,
        /// The minimum allowed terrain strength.
        min: f32,
        /// The maximum allowed terrain strength.
        max: f32,
    },
    /// Failed to parse terrain type.
    InvalidTerrainType(String),
}

impl std::fmt::Display for ConfigBuilderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigBuilderError::InvalidResolution { width, height } => {
                write!(
                    f,
                    "resolution must be between 10x10 and 2000x2000, got {}x{}",
                    width, height
                )
            }
            ConfigBuilderError::InvalidFps { fps } => {
                write!(f, "fps must be between 1 and 144, got {}", fps)
            }
            ConfigBuilderError::InvalidPopulation { pop, min, max } => {
                write!(
                    f,
                    "population must be between {} and {}, got {}",
                    min, max, pop
                )
            }
            ConfigBuilderError::InvalidSensorAngle { value, min, max } => {
                write!(
                    f,
                    "sensor_angle must be between {} and {} degrees, got {}",
                    min, max, value
                )
            }
            ConfigBuilderError::InvalidSensorDistance { value, min, max } => {
                write!(
                    f,
                    "sensor_distance must be between {} and {}, got {}",
                    min, max, value
                )
            }
            ConfigBuilderError::InvalidRotationAngle { value, min, max } => {
                write!(
                    f,
                    "rotation_angle must be between {} and {} degrees, got {}",
                    min, max, value
                )
            }
            ConfigBuilderError::InvalidStepSize { value, min, max } => {
                write!(
                    f,
                    "step_size must be between {} and {}, got {}",
                    min, max, value
                )
            }
            ConfigBuilderError::InvalidDecayFactor { value, min, max } => {
                write!(
                    f,
                    "decay must be between {} and {}, got {}",
                    min, max, value
                )
            }
            ConfigBuilderError::InvalidDepositAmount { value, min, max } => {
                write!(
                    f,
                    "deposit must be between {} and {}, got {}",
                    min, max, value
                )
            }
            ConfigBuilderError::InvalidMaxBrightness { value, min, max } => {
                write!(
                    f,
                    "max_brightness must be between {} and {}, got {}",
                    min, max, value
                )
            }
            ConfigBuilderError::InvalidDiffusionSigma { value, min, max } => {
                write!(
                    f,
                    "diffusion_sigma must be between {} and {}, got {}",
                    min, max, value
                )
            }
            ConfigBuilderError::InvalidTimeScale { value, min, max } => {
                write!(
                    f,
                    "time_scale must be between {} and {}, got {}",
                    min, max, value
                )
            }
            ConfigBuilderError::InvalidAttractorStrength { value, min, max } => {
                write!(
                    f,
                    "attractor_strength must be between {} and {}, got {}",
                    min, max, value
                )
            }
            ConfigBuilderError::InvalidTerrainStrength { value, min, max } => {
                write!(
                    f,
                    "terrain_strength must be between {} and {}, got {}",
                    min, max, value
                )
            }
            ConfigBuilderError::InvalidTerrainType(s) => {
                write!(f, "invalid terrain type: {}", s)
            }
        }
    }
}

impl std::error::Error for ConfigBuilderError {}

/// Builder for constructing SimConfig instances with validation.
///
/// Provides a fluent API for setting configuration parameters with
/// automatic validation against acceptable ranges.
#[derive(Default)]
pub struct ConfigBuilder {
    preset: Option<Preset>,
    sensor_angle: Option<f32>,
    sensor_distance: Option<f32>,
    rotation_angle: Option<f32>,
    step_size: Option<f32>,
    decay_factor: Option<f32>,
    deposit_amount: Option<f32>,
    max_brightness: Option<f32>,
    diffusion_kernel: Option<DiffusionKernel>,
    diffusion_sigma: Option<f32>,
    time_scale: Option<f32>,
    population: Option<usize>,
    fps: Option<usize>,
    food_image_path: Option<String>,
    food_image_invert: Option<bool>,
    food_image_scale: Option<f32>,
    attractors: Vec<AttractorArg>,
    attractor_strength: Option<f32>,
    obstacles: Vec<ObstacleArg>,
    species: Vec<SpeciesArg>,
    separate_species_trails: bool,
    species_colors: bool,
    use_simd: Option<bool>,
    wind: Option<WindArg>,
    terrain: Option<String>,
    terrain_strength: Option<f32>,
    background_color: Option<String>,
}

impl ConfigBuilder {
    /// Creates a new ConfigBuilder with default values.
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a ConfigBuilder from CLI arguments.
    pub fn from_args(args: &Args) -> Self {
        Self {
            preset: args.preset,
            sensor_angle: args.sensor_angle,
            sensor_distance: args.sensor_distance,
            rotation_angle: args.rotation_angle,
            step_size: args.step_size,
            decay_factor: args.decay_factor,
            deposit_amount: args.deposit_amount,
            max_brightness: args.max_brightness,
            diffusion_kernel: args.diffusion_kernel,
            diffusion_sigma: args.diffusion_sigma,
            time_scale: Some(args.time_scale),
            population: args.population,
            fps: Some(args.fps),
            food_image_path: Some(args.food.clone()),
            food_image_invert: Some(args.food_invert),
            food_image_scale: Some(args.food_scale),
            attractors: args.attract.clone(),
            attractor_strength: Some(args.attractor_strength),
            obstacles: args.obstacle.clone(),
            species: args.species.clone(),
            separate_species_trails: args.separate_species_trails,
            species_colors: args.species_colors,
            use_simd: Some(!args.simd_off),
            wind: args.wind.clone(),
            terrain: Some(args.terrain.clone()),
            terrain_strength: Some(args.terrain_strength),
            background_color: args.bg_color.clone(),
        }
    }

    /// Sets the preset to use as a base configuration.
    pub fn preset(mut self, preset: Preset) -> Self {
        self.preset = Some(preset);
        self
    }

    /// Sets the sensor angle.
    pub fn sensor_angle(mut self, angle: f32) -> Self {
        self.sensor_angle = Some(angle);
        self
    }

    /// Sets the sensor distance.
    pub fn sensor_distance(mut self, distance: f32) -> Self {
        self.sensor_distance = Some(distance);
        self
    }

    /// Sets the rotation angle.
    pub fn rotation_angle(mut self, angle: f32) -> Self {
        self.rotation_angle = Some(angle);
        self
    }

    /// Sets the step size.
    pub fn step_size(mut self, size: f32) -> Self {
        self.step_size = Some(size);
        self
    }

    /// Sets the decay factor.
    pub fn decay_factor(mut self, factor: f32) -> Self {
        self.decay_factor = Some(factor);
        self
    }

    /// Sets the deposit amount.
    pub fn deposit_amount(mut self, amount: f32) -> Self {
        self.deposit_amount = Some(amount);
        self
    }

    /// Sets the max brightness.
    pub fn max_brightness(mut self, brightness: f32) -> Self {
        self.max_brightness = Some(brightness);
        self
    }

    /// Sets the diffusion kernel.
    pub fn diffusion_kernel(mut self, kernel: DiffusionKernel) -> Self {
        self.diffusion_kernel = Some(kernel);
        self
    }

    /// Sets the diffusion sigma.
    pub fn diffusion_sigma(mut self, sigma: f32) -> Self {
        self.diffusion_sigma = Some(sigma);
        self
    }

    /// Sets the time scale.
    pub fn time_scale(mut self, scale: f32) -> Self {
        self.time_scale = Some(scale);
        self
    }

    /// Sets the population.
    pub fn population(mut self, pop: usize) -> Self {
        self.population = Some(pop);
        self
    }

    /// Sets the food image path.
    pub fn food_image_path(mut self, path: String) -> Self {
        self.food_image_path = Some(path);
        self
    }

    /// Sets whether to invert the food image.
    pub fn food_image_invert(mut self, invert: bool) -> Self {
        self.food_image_invert = Some(invert);
        self
    }

    /// Sets the food image scale.
    pub fn food_image_scale(mut self, scale: f32) -> Self {
        self.food_image_scale = Some(scale);
        self
    }

    /// Adds an attractor.
    pub fn add_attractor(mut self, attractor: AttractorArg) -> Self {
        self.attractors.push(attractor);
        self
    }

    /// Sets the attractor strength.
    pub fn attractor_strength(mut self, strength: f32) -> Self {
        self.attractor_strength = Some(strength);
        self
    }

    /// Adds an obstacle.
    pub fn add_obstacle(mut self, obstacle: ObstacleArg) -> Self {
        self.obstacles.push(obstacle);
        self
    }

    /// Adds a species.
    pub fn add_species(mut self, species: SpeciesArg) -> Self {
        self.species.push(species);
        self
    }

    /// Sets whether to use separate species trails.
    pub fn separate_species_trails(mut self, separate: bool) -> Self {
        self.separate_species_trails = separate;
        self
    }

    /// Sets whether to use species colors.
    pub fn species_colors(mut self, colors: bool) -> Self {
        self.species_colors = colors;
        self
    }

    /// Sets whether to use SIMD.
    pub fn use_simd(mut self, use_simd: bool) -> Self {
        self.use_simd = Some(use_simd);
        self
    }

    /// Sets the wind.
    pub fn wind(mut self, wind: WindArg) -> Self {
        self.wind = Some(wind);
        self
    }

    /// Sets the terrain type.
    pub fn terrain(mut self, terrain: String) -> Self {
        self.terrain = Some(terrain);
        self
    }

    /// Sets the terrain strength.
    pub fn terrain_strength(mut self, strength: f32) -> Self {
        self.terrain_strength = Some(strength);
        self
    }

    /// Sets the background color.
    pub fn background_color(mut self, color: String) -> Self {
        self.background_color = Some(color);
        self
    }

    /// Validates the current configuration state.
    ///
    /// Returns Ok(()) if all parameters are within valid ranges.
    pub fn validate(&self) -> Result<(), ConfigBuilderError> {
        // Validate population
        if let Some(pop) = self.population {
            if !(pop_consts::MIN_TOTAL..=pop_consts::MAX_TOTAL).contains(&pop) {
                return Err(ConfigBuilderError::InvalidPopulation {
                    pop,
                    min: pop_consts::MIN_TOTAL,
                    max: pop_consts::MAX_TOTAL,
                });
            }
        }

        // Validate FPS
        if let Some(fps) = self.fps {
            if !(1..=144).contains(&fps) {
                return Err(ConfigBuilderError::InvalidFps { fps });
            }
        }

        // Validate sensor angle
        if let Some(sa) = self.sensor_angle {
            if !(agent_consts::MIN_SENSOR_ANGLE..=agent_consts::MAX_SENSOR_ANGLE).contains(&sa) {
                return Err(ConfigBuilderError::InvalidSensorAngle {
                    value: sa,
                    min: agent_consts::MIN_SENSOR_ANGLE,
                    max: agent_consts::MAX_SENSOR_ANGLE,
                });
            }
        }

        // Validate sensor distance
        if let Some(sd) = self.sensor_distance {
            if !(agent_consts::MIN_SENSOR_DISTANCE..=agent_consts::MAX_SENSOR_DISTANCE)
                .contains(&sd)
            {
                return Err(ConfigBuilderError::InvalidSensorDistance {
                    value: sd,
                    min: agent_consts::MIN_SENSOR_DISTANCE,
                    max: agent_consts::MAX_SENSOR_DISTANCE,
                });
            }
        }

        // Validate rotation angle
        if let Some(ra) = self.rotation_angle {
            if !(agent_consts::MIN_ROTATION_ANGLE..=agent_consts::MAX_ROTATION_ANGLE).contains(&ra)
            {
                return Err(ConfigBuilderError::InvalidRotationAngle {
                    value: ra,
                    min: agent_consts::MIN_ROTATION_ANGLE,
                    max: agent_consts::MAX_ROTATION_ANGLE,
                });
            }
        }

        // Validate step size
        if let Some(ss) = self.step_size {
            if !(agent_consts::MIN_STEP_SIZE..=agent_consts::MAX_STEP_SIZE).contains(&ss) {
                return Err(ConfigBuilderError::InvalidStepSize {
                    value: ss,
                    min: agent_consts::MIN_STEP_SIZE,
                    max: agent_consts::MAX_STEP_SIZE,
                });
            }
        }

        // Validate decay factor
        if let Some(df) = self.decay_factor {
            if !(trail_consts::MIN_DECAY_FACTOR..=trail_consts::MAX_DECAY_FACTOR).contains(&df) {
                return Err(ConfigBuilderError::InvalidDecayFactor {
                    value: df,
                    min: trail_consts::MIN_DECAY_FACTOR,
                    max: trail_consts::MAX_DECAY_FACTOR,
                });
            }
        }

        // Validate deposit amount
        if let Some(da) = self.deposit_amount {
            if !(agent_consts::MIN_DEPOSIT_AMOUNT..=agent_consts::MAX_DEPOSIT_AMOUNT).contains(&da)
            {
                return Err(ConfigBuilderError::InvalidDepositAmount {
                    value: da,
                    min: agent_consts::MIN_DEPOSIT_AMOUNT,
                    max: agent_consts::MAX_DEPOSIT_AMOUNT,
                });
            }
        }

        // Validate max brightness
        if let Some(mb) = self.max_brightness {
            if !(trail_consts::MIN_MAX_BRIGHTNESS..=trail_consts::MAX_MAX_BRIGHTNESS).contains(&mb)
            {
                return Err(ConfigBuilderError::InvalidMaxBrightness {
                    value: mb,
                    min: trail_consts::MIN_MAX_BRIGHTNESS,
                    max: trail_consts::MAX_MAX_BRIGHTNESS,
                });
            }
        }

        // Validate diffusion sigma
        if let Some(ds) = self.diffusion_sigma {
            if !(trail_consts::MIN_DIFFUSION_SIGMA..=trail_consts::MAX_DIFFUSION_SIGMA)
                .contains(&ds)
            {
                return Err(ConfigBuilderError::InvalidDiffusionSigma {
                    value: ds,
                    min: trail_consts::MIN_DIFFUSION_SIGMA,
                    max: trail_consts::MAX_DIFFUSION_SIGMA,
                });
            }
        }

        // Validate time scale
        if let Some(ts) = self.time_scale {
            if !(ConfigDefaults::MIN_TIME_SCALE..=ConfigDefaults::MAX_TIME_SCALE).contains(&ts) {
                return Err(ConfigBuilderError::InvalidTimeScale {
                    value: ts,
                    min: ConfigDefaults::MIN_TIME_SCALE,
                    max: ConfigDefaults::MAX_TIME_SCALE,
                });
            }
        }

        // Validate attractor strength
        if let Some(strength) = self.attractor_strength {
            if !(ConfigDefaults::MIN_ATTRACTOR_STRENGTH..=ConfigDefaults::MAX_ATTRACTOR_STRENGTH)
                .contains(&strength)
            {
                return Err(ConfigBuilderError::InvalidAttractorStrength {
                    value: strength,
                    min: ConfigDefaults::MIN_ATTRACTOR_STRENGTH,
                    max: ConfigDefaults::MAX_ATTRACTOR_STRENGTH,
                });
            }
        }

        // Validate terrain strength
        if let Some(ts) = self.terrain_strength {
            if !(ConfigDefaults::MIN_TERRAIN_STRENGTH..=ConfigDefaults::MAX_TERRAIN_STRENGTH)
                .contains(&ts)
            {
                return Err(ConfigBuilderError::InvalidTerrainStrength {
                    value: ts,
                    min: ConfigDefaults::MIN_TERRAIN_STRENGTH,
                    max: ConfigDefaults::MAX_TERRAIN_STRENGTH,
                });
            }
        }

        Ok(())
    }

    /// Builds the SimConfig from the current configuration state.
    ///
    /// This method applies all configured parameters to a base configuration
    /// (either from a preset or default), handling species configuration
    /// and special cases like high-FPS mode.
    pub fn build(self) -> Result<SimConfig, ConfigBuilderError> {
        // Validate first
        self.validate()?;

        // Start with preset or default
        let mut config = if let Some(preset) = self.preset {
            SimConfig::from(preset)
        } else {
            SimConfig::default()
        };

        // Apply overrides
        if let Some(v) = self.sensor_angle {
            config.sensor_angle = v;
        }
        if let Some(v) = self.sensor_distance {
            config.sensor_distance = v;
        }
        if let Some(v) = self.rotation_angle {
            config.rotation_angle = v;
        }
        if let Some(v) = self.step_size {
            config.step_size = v;
        }
        if let Some(v) = self.decay_factor {
            config.decay_factor = v;
        }
        if let Some(v) = self.max_brightness {
            config.max_brightness = v;
        }
        if let Some(v) = self.deposit_amount {
            config.deposit_amount = v;
        }

        // Food image settings
        if let Some(path) = self.food_image_path {
            config.food_image_path = Some(path);
        }
        if let Some(invert) = self.food_image_invert {
            config.food_image_invert = invert;
        }
        if let Some(scale) = self.food_image_scale {
            config.food_image_scale = scale;
        }

        // Diffusion settings
        if let Some(kernel) = self.diffusion_kernel {
            config.diffusion_kernel = kernel;
        }
        if let Some(sigma) = self.diffusion_sigma {
            config.diffusion_sigma = sigma;
        }

        // Time scale
        if let Some(scale) = self.time_scale {
            config.time_scale = scale;
        }

        // High FPS optimization: use Gaussian with lower sigma for smoother diffusion
        if let Some(fps) = self.fps {
            if fps >= 60 && self.diffusion_kernel.is_none() && self.diffusion_sigma.is_none() {
                config.diffusion_kernel = DiffusionKernel::Gaussian;
                config.diffusion_sigma = 0.5;
            }
        }

        // Attractors and obstacles
        config.attractors = self
            .attractors
            .iter()
            .map(|a| Attractor::new(a.x, a.y, a.strength))
            .collect();

        if let Some(strength) = self.attractor_strength {
            config.attractor_strength = strength;
        }

        config.obstacles = self.obstacles.iter().map(|o| o.obstacle.clone()).collect();
        let _ = config.load_obstacle_masks();

        // Species configuration
        config.separate_species_trails = self.separate_species_trails || self.species_colors;

        if let Some(use_simd) = self.use_simd {
            config.use_simd = use_simd;
        }

        if !self.species.is_empty() {
            // User explicitly provided species
            config.species_configs = self
                .species
                .iter()
                .map(|s| SpeciesConfig {
                    name: s.name.clone(),
                    count: s.count,
                    sensor_angle: s.sensor_angle,
                    rotation_angle: s.rotation_angle,
                    step_size: s.step_size,
                    deposit_amount: s.deposit_amount,
                    color: s.color.clone(),
                })
                .collect();
        } else if self.preset.is_none() {
            // Only use default/CLI-overridden single species if NOT using a preset
            config.species_configs = vec![SpeciesConfig {
                name: "default".to_string(),
                count: self.population.unwrap_or(ConfigDefaults::POPULATION),
                sensor_angle: self.sensor_angle.unwrap_or(config.sensor_angle),
                rotation_angle: self.rotation_angle.unwrap_or(config.rotation_angle),
                step_size: self.step_size.unwrap_or(config.step_size),
                deposit_amount: self.deposit_amount.unwrap_or(config.deposit_amount),
                color: "228b22".to_string(),
            }];
        } else if let Some(preset_species) = config.species_configs.first_mut() {
            // If using a preset, allow overriding the FIRST species' properties with CLI args if provided
            if let Some(pop) = self.population {
                preset_species.count = pop;
            }
            if let Some(sa) = self.sensor_angle {
                preset_species.sensor_angle = sa;
            }
            if let Some(ra) = self.rotation_angle {
                preset_species.rotation_angle = ra;
            }
            if let Some(ss) = self.step_size {
                preset_species.step_size = ss;
            }
            if let Some(da) = self.deposit_amount {
                preset_species.deposit_amount = da;
            }
        }

        // Wind
        config.wind = self.wind.map(|w| Wind::new(w.dx, w.dy));

        // Terrain
        if let Some(terrain_str) = self.terrain {
            config.terrain = terrain_str
                .parse::<TerrainType>()
                .map_err(|_| ConfigBuilderError::InvalidTerrainType(terrain_str))?;
        }
        if let Some(strength) = self.terrain_strength {
            config.terrain_strength = strength;
        }

        // Background color
        config.background_color = self.background_color;

        Ok(config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_builder_default() {
        let builder = ConfigBuilder::new();
        assert!(builder.preset.is_none());
        assert!(builder.sensor_angle.is_none());
    }

    #[test]
    fn test_config_builder_with_preset() {
        let builder = ConfigBuilder::new().preset(Preset::Organic);
        assert_eq!(builder.preset, Some(Preset::Organic));
    }

    #[test]
    fn test_config_builder_validation_population() {
        let builder = ConfigBuilder::new().population(500);
        let result = builder.validate();
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ConfigBuilderError::InvalidPopulation { .. }
        ));
    }

    #[test]
    fn test_config_builder_validation_sensor_angle() {
        let builder = ConfigBuilder::new().sensor_angle(100.0);
        let result = builder.validate();
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ConfigBuilderError::InvalidSensorAngle { .. }
        ));
    }

    #[test]
    fn test_config_builder_build_default() {
        use crate::config_defaults::ConfigDefaults;
        let builder = ConfigBuilder::new();
        let config = builder.build().expect("build should succeed");
        assert_eq!(config.sensor_angle, ConfigDefaults::SENSOR_ANGLE);
        assert_eq!(config.total_population(), ConfigDefaults::POPULATION);
    }

    #[test]
    fn test_config_builder_build_with_overrides() {
        let config = ConfigBuilder::new()
            .sensor_angle(30.0)
            .population(10000)
            .build()
            .expect("build should succeed");

        assert_eq!(config.sensor_angle, 30.0);
        assert_eq!(config.total_population(), 10000);
    }

    #[test]
    fn test_config_builder_with_preset_override() {
        let config = ConfigBuilder::new()
            .preset(Preset::Organic)
            .sensor_angle(15.0)
            .build()
            .expect("build should succeed");

        // Preset defines 22.5, but we overrode it with 15.0
        assert_eq!(config.sensor_angle, 15.0);
    }
}
