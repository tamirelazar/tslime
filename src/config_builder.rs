//! Internal helper for assembling [`SimConfig`] instances.
//!
//! This module normalizes parsed CLI [`Args`] into a [`SimConfig`], applying
//! preset bases, per-field overrides, and species/terrain/window handling.
//! It does not validate — validation lives in `SimConfig::try_from`.

use crate::cli::{Args, AttractorArg, ObstacleArg, SpeciesArg, WindArg};
use crate::config_defaults::population;
use crate::simulation::config::{
    Aspect, Attractor, BoundaryMode, ChromeStyle, DiffusionKernel, Preset, SimConfig,
    SpeciesConfig, TerminalSizeThreshold, TerrainType, Wind, WindowFrame, WindowPadding,
};

/// Staged CLI overrides, applied to a preset/default base by
/// [`ConfigBuilder::assemble`].
pub(crate) struct ConfigBuilder {
    preset: Option<Preset>,
    sensor_angle: Option<f32>,
    sensor_distance: Option<f32>,
    rotation_angle: Option<f32>,
    step_size: Option<f32>,
    decay_factor: Option<f32>,
    deposit_amount: Option<f32>,
    brightness: Option<f32>,
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
    boundary_mode: Option<BoundaryMode>,
    window_frame: Option<WindowFrame>,
    chrome_style: Option<ChromeStyle>,
    aspect: Option<Aspect>,
    window_padding: Option<WindowPadding>,
    show_status_bar: Option<bool>,
    min_sim_size: Option<TerminalSizeThreshold>,
    min_frame_size: Option<TerminalSizeThreshold>,
    respawn_interval: Option<u32>,
    afterglow: f32,
    afterglow_rate: f32,
    decay_gamma: f32,
    diffuse_weight: f32,
}

impl ConfigBuilder {
    /// Creates a ConfigBuilder from CLI arguments.
    pub(crate) fn from_args(args: &Args) -> Self {
        Self {
            preset: args.preset,
            sensor_angle: args.sensor_angle,
            sensor_distance: args.sensor_distance,
            rotation_angle: args.rotation_angle,
            step_size: args.step_size,
            decay_factor: args.decay_factor,
            deposit_amount: args.deposit_amount,
            brightness: args.brightness,
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
            species: args.species_list().to_vec(),
            separate_species_trails: args.separate_species_trails_enabled(),
            species_colors: args.species_colors_enabled(),
            use_simd: Some(!args.simd_off),
            wind: args.wind.clone(),
            terrain: Some(args.terrain.clone()),
            terrain_strength: Some(args.terrain_strength),
            background_color: args.bg_color.clone(),
            boundary_mode: args.boundary_mode,
            window_frame: args.window_frame,
            chrome_style: if args.fullscreen {
                Some(ChromeStyle::Fullscreen)
            } else {
                args.chrome_style
            },
            aspect: args.aspect,
            window_padding: args.window_padding,
            show_status_bar: if args.show_status_bar {
                Some(true)
            } else {
                None
            },
            min_sim_size: args.min_sim_size,
            min_frame_size: args.min_frame_size,
            respawn_interval: args.respawn_interval,
            afterglow: args.afterglow,
            afterglow_rate: args.afterglow_rate,
            decay_gamma: args.decay_gamma,
            diffuse_weight: args.diffuse_weight,
        }
    }

    /// Applies the staged overrides to a base config (preset or default),
    /// handling species configuration and special cases like high-FPS mode.
    pub(crate) fn assemble(self) -> Result<SimConfig, crate::error::ValidationError> {
        // No validation here — caller validates the assembled config once.

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
        if let Some(gain) = self.brightness {
            // CLI exposes a user-facing brightness gain; the engine stores a
            // normalization white-point that it divides by. Convert here so the
            // internal representation stays a white-point.
            config.max_brightness = crate::config_defaults::trail::white_point_from_gain(gain);
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

        // Afterglow settings
        config.afterglow = self.afterglow;
        config.afterglow_rate = self.afterglow_rate;

        // Decay gamma
        config.decay_gamma = self.decay_gamma;

        // Diffuse weight (Lague blend)
        config.diffuse_weight = self.diffuse_weight;

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
                    color: s.color,
                    trail_modulation: None,
                })
                .collect();
        } else if self.preset.is_none() {
            // Only use default/CLI-overridden single species if NOT using a preset
            use crate::render::palette::RgbColor;
            config.species_configs = vec![SpeciesConfig {
                name: "default".to_string(),
                count: self.population.unwrap_or(population::DEFAULT_POPULATION),
                sensor_angle: self.sensor_angle.unwrap_or(config.sensor_angle),
                rotation_angle: self.rotation_angle.unwrap_or(config.rotation_angle),
                step_size: self.step_size.unwrap_or(config.step_size),
                deposit_amount: self.deposit_amount.unwrap_or(config.deposit_amount),
                color: RgbColor::from_hex(0x228b22),
                trail_modulation: None,
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
            config.terrain = terrain_str.parse::<TerrainType>().map_err(|_| {
                crate::error::ValidationError::custom(format!(
                    "invalid terrain type: {terrain_str}"
                ))
            })?;
        }
        if let Some(strength) = self.terrain_strength {
            config.terrain_strength = strength;
        }

        // Background color
        config.background_color = self.background_color;

        // Boundary mode
        if let Some(mode) = self.boundary_mode {
            config.boundary_mode = mode;
        }

        // Window frame mode
        if let Some(mode) = self.window_frame {
            config.window_frame = mode;
        }

        // Chrome style, aspect, padding, status bar, size thresholds
        if let Some(cs) = self.chrome_style {
            config.chrome_style = cs;
        }
        if let Some(a) = self.aspect {
            config.aspect = a;
        }
        if let Some(p) = self.window_padding {
            config.window_padding = p;
        }
        if let Some(v) = self.show_status_bar {
            config.show_status_bar = v;
        }
        if let Some(t) = self.min_sim_size {
            config.min_sim_size = t;
        }
        if let Some(t) = self.min_frame_size {
            config.min_frame_size = t;
        }

        // Respawn configuration
        if let Some(interval) = self.respawn_interval {
            config.respawn_config.interval = interval;
        }

        Ok(config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::Args;
    use clap::Parser;

    #[test]
    fn test_config_builder_build_default() {
        use crate::config_defaults::agent;
        let args = Args::parse_from(["tslime"]);
        let config = ConfigBuilder::from_args(&args)
            .assemble()
            .expect("assemble should succeed");
        assert_eq!(config.sensor_angle, agent::DEFAULT_SENSOR_ANGLE);
        assert_eq!(config.total_population(), population::DEFAULT_POPULATION);
    }

    #[test]
    fn test_config_builder_build_with_overrides() {
        let args = Args::parse_from(["tslime", "--sensor-angle", "30", "--population", "10000"]);
        let config = ConfigBuilder::from_args(&args)
            .assemble()
            .expect("assemble should succeed");

        assert_eq!(config.sensor_angle, 30.0);
        assert_eq!(config.total_population(), 10000);
    }

    #[test]
    fn test_config_builder_with_preset_override() {
        let args = Args::parse_from(["tslime", "--preset", "organic", "--sensor-angle", "15"]);
        let config = ConfigBuilder::from_args(&args)
            .assemble()
            .expect("assemble should succeed");

        // Preset defines 22.5, but we overrode it with 15.0
        assert_eq!(config.sensor_angle, 15.0);
    }
}
