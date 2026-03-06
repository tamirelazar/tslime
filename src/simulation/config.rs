//! Simulation configuration types and presets.
//!
//! This module defines all the configuration parameters for the Physarum simulation,
//! including presets, diffusion kernels, initialization modes, and environmental effects.

use image::io::Reader as ImageReader;
use std::borrow::Cow;
use std::path::Path;

use super::agent::normalize_angle;
use crate::config_defaults::{
    agent as agent_consts, environment, environment as env_consts, food as food_img_consts,
    population, population as pop_consts, time as time_consts, trail as trail_consts,
};
use crate::render::palette::RgbColor;

/// Algorithm used for pheromone diffusion (spreading).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiffusionKernel {
    /// Simple 3×3 box blur averaging. Fast with sharp patterns.
    Mean3x3,
    /// 5×5 Gaussian blur. Slower but produces smoother, more organic patterns.
    Gaussian,
}

/// Named parameter presets for different visual styles.
///
/// Each preset combines multiple parameters optimized for a specific aesthetic.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Preset {
    /// Dense, interconnected networks with rapid branching.
    Network,
    /// Wide, searching tentacles with exploratory behavior.
    Exploratory,
    /// Long branching arms stretching across the terminal.
    Tendrils,
    /// Balanced, natural-looking growth (default).
    Organic,
    /// Minimal, sparse patterns with fewer agents.
    Minimal,
    /// Organic moss-like growth patterns.
    Moss,
    /// Space-inspired ethereal patterns.
    Cosmic,
    /// Aggressive, fast-moving flame-like patterns.
    Fire,
    /// Calm, meditative slow-moving patterns.
    Zen,
    /// Dynamic, turbulent patterns.
    Storm,
    /// Flowing, water-like patterns.
    River,
    /// Ethereal, ghost-like patterns.
    Ethereal,
    /// Petri dish simulation: starts center, slow growth, persistent trails.
    PetriDish,
    /// Spinning vortex patterns (rotation_angle > sensor_angle).
    Vortex,
    /// Fast dendritic branching like lightning.
    Lightning,
    /// Slow, stable geometric crystal growth.
    Crystal,
    /// Edge-of-chaos sensitive patterns (sensor_angle ≈ rotation_angle).
    ChaosEdge,
    /// Aggregating blob clusters.
    Blob,
    /// Long snaking worm-like trails.
    Worm,
    /// Rhythmic pulsing waves with trail-based modulation.
    Pulse,
    /// Dense coral-like branching structures.
    Coral,
    /// Cohesive flocking group movement.
    Flocking,
    /// Complex maze-like corridor patterns.
    Maze,
    /// Concentric ripple patterns.
    Ripple,
    /// Enhanced vortex with trail modulation.
    Vortex36,
    /// Dramatic behavior shifts based on trail density.
    Chameleon,
    /// Dynamic tendrils with trail-based sensor modulation.
    DynamicTendrils,
    /// Morphing coral with dramatic parameter shifts.
    MorphingCoral,
    /// Reactive swarm with trail-dependent behavior.
    ReactiveSwarm,
    /// Two-species with opposing modulation patterns.
    DuelingModulators,
}

impl Preset {
    /// Apply this preset to a SimConfig, modifying only the fields that differ from defaults.
    pub fn apply(&self, config: &mut SimConfig) {
        match self {
            Preset::Network => {
                config.sensor_angle = 15.0;
                config.rotation_angle = 30.0;
                config.decay_factor = 0.85;
                config.diffusion_kernel = DiffusionKernel::Mean3x3;
                config.max_brightness = 20.0;
                config.species_configs = vec![SpeciesConfig {
                    name: "default".to_string(),
                    count: 50_000,
                    sensor_angle: 15.0,
                    rotation_angle: 30.0,
                    ..Default::default()
                }];
            }
            Preset::Exploratory => {
                config.sensor_angle = 45.0;
                config.sensor_distance = 15.0;
                config.rotation_angle = 60.0;
                config.decay_factor = 0.96;
                config.deposit_amount = 3.0;
                config.diffusion_kernel = DiffusionKernel::Mean3x3;
                config.max_brightness = 12.0;
                config.species_configs = vec![SpeciesConfig {
                    name: "default".to_string(),
                    count: 30_000,
                    sensor_angle: 45.0,
                    rotation_angle: 60.0,
                    deposit_amount: 3.0,
                    ..Default::default()
                }];
            }
            Preset::Tendrils => {
                config.sensor_angle = 30.0;
                config.sensor_distance = 12.0;
                config.rotation_angle = 45.0;
                config.step_size = 2.0;
                config.decay_factor = 0.90;
                config.deposit_amount = 4.0;
                config.diffusion_kernel = DiffusionKernel::Mean3x3;
                config.max_brightness = 16.0;
                config.species_configs = vec![SpeciesConfig {
                    name: "default".to_string(),
                    count: 40_000,
                    sensor_angle: 30.0,
                    rotation_angle: 45.0,
                    step_size: 2.0,
                    deposit_amount: 4.0,
                    ..Default::default()
                }];
            }
            Preset::Organic => {
                config.sensor_angle = 22.5;
                config.sensor_distance = 9.0;
                config.rotation_angle = 45.0;
                config.step_size = 1.0;
                config.decay_factor = 0.85;
                config.diffusion_kernel = DiffusionKernel::Mean3x3;
                config.max_brightness = 20.0;
            }
            Preset::Minimal => {
                config.sensor_angle = 30.0;
                config.sensor_distance = 9.0;
                config.rotation_angle = 30.0;
                config.step_size = 0.8;
                config.decay_factor = 0.95;
                config.deposit_amount = 3.0;
                config.diffusion_kernel = DiffusionKernel::Mean3x3;
                config.max_brightness = 15.0;
                config.species_configs = vec![SpeciesConfig {
                    name: "default".to_string(),
                    count: 15_000,
                    sensor_angle: 30.0,
                    rotation_angle: 30.0,
                    step_size: 0.8,
                    deposit_amount: 3.0,
                    ..Default::default()
                }];
            }
            Preset::Moss => {
                config.sensor_angle = 22.0;
                config.sensor_distance = 12.0;
                config.rotation_angle = 35.0;
                config.decay_factor = 0.88;
                config.deposit_amount = 4.0;
                config.diffusion_kernel = DiffusionKernel::Mean3x3;
                config.max_brightness = 18.0;
                config.species_configs = vec![SpeciesConfig {
                    name: "default".to_string(),
                    count: 35_000,
                    sensor_angle: 22.0,
                    rotation_angle: 35.0,
                    deposit_amount: 4.0,
                    color: RgbColor::from_hex(0x4a7a4a),
                    ..Default::default()
                }];
            }
            Preset::Cosmic => {
                config.sensor_angle = 55.0;
                config.sensor_distance = 15.0;
                config.rotation_angle = 45.0;
                config.step_size = 0.7;
                config.decay_factor = 0.93;
                config.deposit_amount = 3.0;
                config.max_brightness = 14.0;
                config.species_configs = vec![SpeciesConfig {
                    name: "default".to_string(),
                    count: 25_000,
                    sensor_angle: 55.0,
                    rotation_angle: 45.0,
                    step_size: 0.7,
                    deposit_amount: 3.0,
                    color: RgbColor::from_hex(0x8a2be2),
                    ..Default::default()
                }];
            }
            Preset::Fire => {
                config.sensor_angle = 15.0;
                config.rotation_angle = 30.0;
                config.step_size = 1.5;
                config.decay_factor = 0.85;
                config.diffusion_kernel = DiffusionKernel::Mean3x3;
                config.max_brightness = 20.0;
                config.species_configs = vec![SpeciesConfig {
                    name: "default".to_string(),
                    count: 100_000,
                    sensor_angle: 15.0,
                    rotation_angle: 30.0,
                    step_size: 1.5,
                    color: RgbColor::from_hex(0xff4500),
                    ..Default::default()
                }];
            }
            Preset::Zen => {
                config.sensor_distance = 12.0;
                config.sensor_angle = 25.0;
                config.rotation_angle = 30.0;
                config.step_size = 0.5;
                config.decay_factor = 0.94;
                config.deposit_amount = 2.0;
                config.diffusion_kernel = DiffusionKernel::Mean3x3;
                config.max_brightness = 12.0;
                config.species_configs = vec![SpeciesConfig {
                    name: "default".to_string(),
                    count: 10_000,
                    sensor_angle: 25.0,
                    rotation_angle: 30.0,
                    step_size: 0.5,
                    deposit_amount: 2.0,
                    color: RgbColor::from_hex(0xffffff),
                    ..Default::default()
                }];
            }
            Preset::Storm => {
                config.sensor_angle = 20.0;
                config.rotation_angle = 60.0;
                config.step_size = 2.0;
                config.decay_factor = 0.80;
                config.diffusion_kernel = DiffusionKernel::Mean3x3;
                config.max_brightness = 18.0;
                config.species_configs = vec![SpeciesConfig {
                    name: "default".to_string(),
                    count: 80_000,
                    sensor_angle: 20.0,
                    rotation_angle: 60.0,
                    step_size: 2.0,
                    color: RgbColor::from_hex(0x4682b4),
                    ..Default::default()
                }];
                config.wind = Some(Wind::new(0.1, 0.05));
            }
            Preset::River => {
                config.sensor_angle = 25.0;
                config.step_size = 1.2;
                config.decay_factor = 0.90;
                config.diffusion_kernel = DiffusionKernel::Mean3x3;
                config.max_brightness = 18.0;
                config.species_configs = vec![SpeciesConfig {
                    name: "default".to_string(),
                    count: 45_000,
                    sensor_angle: 25.0,
                    step_size: 1.2,
                    color: RgbColor::from_hex(0x1e90ff),
                    ..Default::default()
                }];
                config.wind = Some(Wind::new(0.3, 0.0));
            }
            Preset::Ethereal => {
                config.sensor_angle = 40.0;
                config.step_size = 0.7;
                config.decay_factor = 0.98;
                config.deposit_amount = 2.0;
                config.max_brightness = 12.0;
                config.species_configs = vec![SpeciesConfig {
                    name: "default".to_string(),
                    count: 25_000,
                    sensor_angle: 40.0,
                    step_size: 0.7,
                    deposit_amount: 2.0,
                    color: RgbColor::from_hex(0xe6e6fa),
                    ..Default::default()
                }];
            }
            Preset::PetriDish => {
                config.sensor_angle = 45.0;
                config.rotation_angle = 20.0;
                config.step_size = 0.05;
                config.decay_factor = 0.999;
                config.deposit_amount = 0.2;
                config.max_brightness = 50.0;
                config.species_configs = vec![SpeciesConfig {
                    name: "mold".to_string(),
                    count: 20_000,
                    sensor_angle: 45.0,
                    rotation_angle: 20.0,
                    step_size: 0.05,
                    deposit_amount: 0.2,
                    color: RgbColor::from_hex(0xd4ff00),
                    ..Default::default()
                }];
                config.obstacles = vec![Obstacle::Circle {
                    x: 200.0,
                    y: 100.0,
                    radius: 90.0,
                }];
                config.background_color = Some("000000".to_string());
                config.preferred_init_mode = Some(InitMode::Petri);
            }
            Preset::Vortex => {
                config.sensor_angle = 25.2;
                config.sensor_distance = 3.9;
                config.rotation_angle = 46.4;
                config.step_size = 1.92;
                config.decay_factor = 0.96;
                config.deposit_amount = 4.3;
                config.diffusion_kernel = DiffusionKernel::Mean3x3;
                config.species_configs = vec![SpeciesConfig {
                    name: "default".to_string(),
                    count: 32_000,
                    sensor_angle: 25.2,
                    rotation_angle: 46.4,
                    step_size: 1.92,
                    deposit_amount: 4.3,
                    color: RgbColor::from_hex(0x9370db),
                    ..Default::default()
                }];
            }
            Preset::Lightning => {
                config.sensor_angle = 31.9;
                config.sensor_distance = 23.2;
                config.rotation_angle = 39.3;
                config.step_size = 2.48;
                config.decay_factor = 0.82;
                config.deposit_amount = 20.0;
                config.diffusion_kernel = DiffusionKernel::Mean3x3;
                config.max_brightness = 40.0;
                config.species_configs = vec![SpeciesConfig {
                    name: "default".to_string(),
                    count: 7_000,
                    sensor_angle: 31.9,
                    rotation_angle: 39.3,
                    step_size: 2.48,
                    deposit_amount: 20.0,
                    color: RgbColor::from_hex(0x00ffff),
                    ..Default::default()
                }];
            }
            Preset::Crystal => {
                config.sensor_angle = 38.9;
                config.sensor_distance = 30.6;
                config.rotation_angle = 21.5;
                config.step_size = 1.47;
                config.decay_factor = 0.50;
                config.deposit_amount = 2.1;
                config.diffusion_sigma = 1.2;
                config.species_configs = vec![SpeciesConfig {
                    name: "default".to_string(),
                    count: 38_000,
                    sensor_angle: 38.9,
                    rotation_angle: 21.5,
                    step_size: 1.47,
                    deposit_amount: 2.1,
                    color: RgbColor::from_hex(0xb0e0e6),
                    ..Default::default()
                }];
            }
            Preset::ChaosEdge => {
                config.sensor_angle = 5.0;
                config.sensor_distance = 26.4;
                config.rotation_angle = 56.2;
                config.step_size = 0.58;
                config.decay_factor = 0.99;
                config.deposit_amount = 15.8;
                config.diffusion_kernel = DiffusionKernel::Mean3x3;
                config.max_brightness = 25.0;
                config.species_configs = vec![SpeciesConfig {
                    name: "default".to_string(),
                    count: 52_000,
                    sensor_angle: 5.0,
                    rotation_angle: 56.2,
                    step_size: 0.58,
                    deposit_amount: 15.8,
                    color: RgbColor::from_hex(0xff6347),
                    ..Default::default()
                }];
            }
            Preset::Blob => {
                config.sensor_angle = 72.1;
                config.sensor_distance = 2.1;
                config.rotation_angle = 90.0;
                config.step_size = 0.92;
                config.decay_factor = 0.50;
                config.deposit_amount = 9.3;
                config.diffusion_kernel = DiffusionKernel::Mean3x3;
                config.max_brightness = 25.0;
                config.species_configs = vec![SpeciesConfig {
                    name: "default".to_string(),
                    count: 21_000,
                    sensor_angle: 72.1,
                    rotation_angle: 90.0,
                    step_size: 0.92,
                    deposit_amount: 9.3,
                    color: RgbColor::from_hex(0x32cd32),
                    ..Default::default()
                }];
            }
            Preset::Worm => {
                config.sensor_angle = 38.8;
                config.sensor_distance = 50.0;
                config.rotation_angle = 13.4;
                config.step_size = 1.96;
                config.decay_factor = 0.65;
                config.deposit_amount = 6.3;
                config.diffusion_kernel = DiffusionKernel::Mean3x3;
                config.species_configs = vec![SpeciesConfig {
                    name: "default".to_string(),
                    count: 6_000,
                    sensor_angle: 38.8,
                    rotation_angle: 13.4,
                    step_size: 1.96,
                    deposit_amount: 6.3,
                    color: RgbColor::from_hex(0xdaa520),
                    ..Default::default()
                }];
            }
            Preset::Pulse => {
                config.sensor_angle = 60.0;
                config.sensor_distance = 30.0;
                config.rotation_angle = 15.0;
                config.step_size = 1.0;
                config.decay_factor = 0.90;
                config.deposit_amount = 4.0;
                config.diffusion_kernel = DiffusionKernel::Mean3x3;
                config.max_brightness = 18.0;
                config.species_configs = vec![SpeciesConfig {
                    name: "default".to_string(),
                    count: 35_000,
                    sensor_angle: 60.0,
                    rotation_angle: 15.0,
                    step_size: 1.0,
                    deposit_amount: 4.0,
                    color: RgbColor::from_hex(0x00ced1),
                    trail_modulation: Some(PointConfig {
                        sensor_distance_base: 30.0,
                        sensor_distance_multiplier: -20.0,
                        sensor_distance_exponent: 1.5,
                        sensor_angle_base: 60.0,
                        sensor_angle_multiplier: -40.0,
                        sensor_angle_exponent: 2.0,
                        rotation_angle_base: 15.0,
                        rotation_angle_multiplier: 30.0,
                        rotation_angle_exponent: 1.0,
                        step_size_base: 1.0,
                        step_size_multiplier: 2.0,
                        step_size_exponent: 1.0,
                        ..Default::default()
                    }),
                }];
            }
            Preset::Coral => {
                config.sensor_angle = 30.0;
                config.sensor_distance = 20.0;
                config.rotation_angle = 45.0;
                config.step_size = 0.3;
                config.decay_factor = 0.92;
                config.deposit_amount = 3.0;
                config.diffusion_kernel = DiffusionKernel::Gaussian;
                config.diffusion_sigma = 1.2;
                config.max_brightness = 15.0;
                config.species_configs = vec![SpeciesConfig {
                    name: "default".to_string(),
                    count: 40_000,
                    sensor_angle: 30.0,
                    rotation_angle: 45.0,
                    step_size: 0.3,
                    deposit_amount: 3.0,
                    color: RgbColor::from_hex(0xff7f50),
                    trail_modulation: Some(PointConfig {
                        sensor_distance_base: 20.0,
                        sensor_distance_multiplier: 10.0,
                        sensor_distance_exponent: 0.5,
                        sensor_angle_base: 30.0,
                        sensor_angle_multiplier: 40.0,
                        sensor_angle_exponent: 1.0,
                        rotation_angle_base: 45.0,
                        rotation_angle_multiplier: -20.0,
                        rotation_angle_exponent: 1.0,
                        step_size_base: 0.3,
                        step_size_multiplier: 0.0,
                        step_size_exponent: 1.0,
                        ..Default::default()
                    }),
                }];
            }
            Preset::Flocking => {
                config.sensor_angle = 45.0;
                config.sensor_distance = 25.0;
                config.rotation_angle = 60.0;
                config.step_size = 1.5;
                config.decay_factor = 0.88;
                config.deposit_amount = 5.0;
                config.diffusion_kernel = DiffusionKernel::Mean3x3;
                config.max_brightness = 20.0;
                config.species_configs = vec![SpeciesConfig {
                    name: "default".to_string(),
                    count: 50_000,
                    sensor_angle: 45.0,
                    rotation_angle: 60.0,
                    step_size: 1.5,
                    deposit_amount: 5.0,
                    color: RgbColor::from_hex(0x4169e1),
                    trail_modulation: Some(PointConfig {
                        sensor_distance_base: 25.0,
                        sensor_distance_multiplier: -20.0,
                        sensor_distance_exponent: 1.0,
                        sensor_angle_base: 45.0,
                        sensor_angle_multiplier: 0.0,
                        sensor_angle_exponent: 1.0,
                        rotation_angle_base: 60.0,
                        rotation_angle_multiplier: -50.0,
                        rotation_angle_exponent: 1.5,
                        step_size_base: 1.5,
                        step_size_multiplier: 1.0,
                        step_size_exponent: 1.0,
                        ..Default::default()
                    }),
                }];
            }
            Preset::Maze => {
                config.sensor_angle = 85.0;
                config.sensor_distance = 15.0;
                config.rotation_angle = 20.0;
                config.step_size = 0.8;
                config.decay_factor = 0.85;
                config.deposit_amount = 6.0;
                config.diffusion_kernel = DiffusionKernel::Mean3x3;
                config.max_brightness = 22.0;
                config.species_configs = vec![SpeciesConfig {
                    name: "default".to_string(),
                    count: 45_000,
                    sensor_angle: 85.0,
                    rotation_angle: 20.0,
                    step_size: 0.8,
                    deposit_amount: 6.0,
                    color: RgbColor::from_hex(0x8b4513),
                    trail_modulation: Some(PointConfig {
                        sensor_distance_base: 15.0,
                        sensor_distance_multiplier: 5.0,
                        sensor_distance_exponent: 1.0,
                        sensor_angle_base: 85.0,
                        sensor_angle_multiplier: 0.0,
                        sensor_angle_exponent: 1.0,
                        rotation_angle_base: 20.0,
                        rotation_angle_multiplier: 40.0,
                        rotation_angle_exponent: 2.0,
                        step_size_base: 0.8,
                        step_size_multiplier: 0.4,
                        step_size_exponent: 1.0,
                        ..Default::default()
                    }),
                }];
            }
            Preset::Ripple => {
                config.sensor_angle = 35.0;
                config.sensor_distance = 12.0;
                config.rotation_angle = 30.0;
                config.step_size = 1.0;
                config.decay_factor = 0.94;
                config.deposit_amount = 3.5;
                config.diffusion_kernel = DiffusionKernel::Mean3x3;
                config.max_brightness = 16.0;
                config.species_configs = vec![SpeciesConfig {
                    name: "default".to_string(),
                    count: 30_000,
                    sensor_angle: 35.0,
                    rotation_angle: 30.0,
                    step_size: 1.0,
                    deposit_amount: 3.5,
                    color: RgbColor::from_hex(0x20b2aa),
                    trail_modulation: Some(PointConfig {
                        sensor_distance_base: 12.0,
                        sensor_distance_multiplier: 8.0,
                        sensor_distance_exponent: 0.8,
                        sensor_angle_base: 35.0,
                        sensor_angle_multiplier: 25.0,
                        sensor_angle_exponent: 1.2,
                        rotation_angle_base: 30.0,
                        rotation_angle_multiplier: -15.0,
                        rotation_angle_exponent: 1.0,
                        step_size_base: 1.0,
                        step_size_multiplier: 1.5,
                        step_size_exponent: 1.0,
                        vertical_offset: 5.0,
                        ..Default::default()
                    }),
                }];
            }
            Preset::Vortex36 => {
                config.sensor_angle = 25.2;
                config.sensor_distance = 7.0;
                config.rotation_angle = 46.4;
                config.step_size = 1.92;
                config.decay_factor = 0.96;
                config.deposit_amount = 4.3;
                config.diffusion_kernel = DiffusionKernel::Mean3x3;
                config.species_configs = vec![SpeciesConfig {
                    name: "default".to_string(),
                    count: 32_000,
                    sensor_angle: 25.2,
                    rotation_angle: 46.4,
                    step_size: 1.92,
                    deposit_amount: 4.3,
                    color: RgbColor::from_hex(0x9370db),
                    trail_modulation: Some(PointConfig {
                        sensor_distance_base: 4.0,
                        sensor_distance_multiplier: 3.0,
                        sensor_distance_exponent: 1.0,
                        sensor_angle_base: 25.0,
                        sensor_angle_multiplier: 10.0,
                        sensor_angle_exponent: 1.0,
                        rotation_angle_base: 46.0,
                        rotation_angle_multiplier: -20.0,
                        rotation_angle_exponent: 1.0,
                        step_size_base: 1.9,
                        step_size_multiplier: 0.5,
                        step_size_exponent: 1.0,
                        heading_offset: 3.0,
                        ..Default::default()
                    }),
                }];
            }
            Preset::Chameleon => {
                config.sensor_angle = 40.0;
                config.sensor_distance = 25.0;
                config.rotation_angle = 45.0;
                config.step_size = 2.5;
                config.decay_factor = 0.90;
                config.deposit_amount = 5.0;
                config.diffusion_kernel = DiffusionKernel::Mean3x3;
                config.max_brightness = 25.0;
                config.species_configs = vec![SpeciesConfig {
                    name: "default".to_string(),
                    count: 35_000,
                    sensor_angle: 40.0,
                    rotation_angle: 45.0,
                    step_size: 2.5,
                    deposit_amount: 5.0,
                    color: RgbColor::from_hex(0x9932cc),
                    trail_modulation: Some(PointConfig {
                        sensor_distance_base: 5.0,
                        sensor_distance_multiplier: 40.0,
                        sensor_distance_exponent: 2.0,
                        sensor_angle_base: 10.0,
                        sensor_angle_multiplier: 70.0,
                        sensor_angle_exponent: 2.0,
                        rotation_angle_base: 5.0,
                        rotation_angle_multiplier: 80.0,
                        rotation_angle_exponent: 2.0,
                        step_size_base: 0.1,
                        step_size_multiplier: 5.0,
                        step_size_exponent: 1.5,
                        ..Default::default()
                    }),
                }];
            }
            Preset::DynamicTendrils => {
                config.decay_factor = 0.92;
                config.species_configs = vec![SpeciesConfig {
                    name: "tendril".to_string(),
                    count: 25_000,
                    trail_modulation: Some(PointConfig {
                        sensor_distance_base: 5.0,
                        sensor_distance_multiplier: 45.0,
                        sensor_distance_exponent: 0.7,
                        sensor_angle_base: 15.0,
                        sensor_angle_multiplier: 60.0,
                        sensor_angle_exponent: 2.0,
                        rotation_angle_base: 10.0,
                        rotation_angle_multiplier: 50.0,
                        rotation_angle_exponent: 1.5,
                        step_size_base: 0.5,
                        step_size_multiplier: 3.0,
                        step_size_exponent: 1.0,
                        ..Default::default()
                    }),
                    color: RgbColor::from_hex(0x00fa9a),
                    ..Default::default()
                }];
            }
            Preset::MorphingCoral => {
                config.decay_factor = 0.88;
                config.diffusion_kernel = DiffusionKernel::Gaussian;
                config.species_configs = vec![SpeciesConfig {
                    name: "coral".to_string(),
                    count: 30_000,
                    trail_modulation: Some(PointConfig {
                        sensor_distance_base: 40.0,
                        sensor_distance_multiplier: -35.0,
                        sensor_distance_exponent: 0.5,
                        sensor_angle_base: 5.0,
                        sensor_angle_multiplier: 80.0,
                        sensor_angle_exponent: 2.5,
                        rotation_angle_base: 5.0,
                        rotation_angle_multiplier: 75.0,
                        rotation_angle_exponent: 2.0,
                        step_size_base: 0.2,
                        step_size_multiplier: 2.0,
                        step_size_exponent: 1.0,
                        trail_rescale: 2.0,
                        ..Default::default()
                    }),
                    color: RgbColor::from_hex(0xff69b4),
                    ..Default::default()
                }];
            }
            Preset::ReactiveSwarm => {
                config.decay_factor = 0.85;
                config.species_configs = vec![SpeciesConfig {
                    name: "swarm".to_string(),
                    count: 60_000,
                    trail_modulation: Some(PointConfig {
                        sensor_distance_base: 25.0,
                        sensor_distance_multiplier: -20.0,
                        sensor_distance_exponent: 1.0,
                        sensor_angle_base: 45.0,
                        sensor_angle_multiplier: -30.0,
                        sensor_angle_exponent: 1.5,
                        rotation_angle_base: 45.0,
                        rotation_angle_multiplier: -40.0,
                        rotation_angle_exponent: 1.0,
                        step_size_base: 1.5,
                        step_size_multiplier: 2.0,
                        step_size_exponent: 1.0,
                        heading_offset: 5.0,
                        ..Default::default()
                    }),
                    color: RgbColor::from_hex(0xffd700),
                    ..Default::default()
                }];
                config.respawn_config = RespawnConfig {
                    interval: 60,
                    base_probability: 0.02,
                    trail_dependent: true,
                    max_probability_multiplier: 5.0,
                };
            }
            Preset::DuelingModulators => {
                config.decay_factor = 0.90;
                config.separate_species_trails = true;
                config.species_configs = vec![
                    SpeciesConfig {
                        name: "expander".to_string(),
                        count: 20_000,
                        trail_modulation: Some(PointConfig {
                            sensor_distance_base: 10.0,
                            sensor_distance_multiplier: 30.0,
                            sensor_distance_exponent: 1.0,
                            sensor_angle_base: 20.0,
                            sensor_angle_multiplier: 40.0,
                            sensor_angle_exponent: 1.0,
                            rotation_angle_base: 30.0,
                            rotation_angle_multiplier: 20.0,
                            rotation_angle_exponent: 1.0,
                            step_size_base: 1.0,
                            step_size_multiplier: 0.5,
                            step_size_exponent: 1.0,
                            ..Default::default()
                        }),
                        color: RgbColor::from_hex(0x00ced1),
                        ..Default::default()
                    },
                    SpeciesConfig {
                        name: "contractor".to_string(),
                        count: 20_000,
                        trail_modulation: Some(PointConfig {
                            sensor_distance_base: 40.0,
                            sensor_distance_multiplier: -30.0,
                            sensor_distance_exponent: 1.0,
                            sensor_angle_base: 60.0,
                            sensor_angle_multiplier: -30.0,
                            sensor_angle_exponent: 1.0,
                            rotation_angle_base: 30.0,
                            rotation_angle_multiplier: -20.0,
                            rotation_angle_exponent: 1.0,
                            step_size_base: 1.0,
                            step_size_multiplier: -0.3,
                            step_size_exponent: 1.0,
                            ..Default::default()
                        }),
                        color: RgbColor::from_hex(0xff6347),
                        ..Default::default()
                    },
                ];
            }
        }
    }
}

/// How agents are initially distributed in the simulation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InitMode {
    /// Agents randomly distributed across the entire canvas.
    Random,
    /// Agents start from the center and burst outward.
    CentralBurst,
    /// Agents arranged in a circle.
    Circle,
    /// Agents distributed in a gradient pattern.
    Gradient,
    /// Agents start as a wave front.
    WaveFront,
    /// Agents arranged in a spiral pattern.
    Spiral,
    /// Agents in random clusters.
    RandomClusters,
    /// Agents distributed based on a loaded image (food source).
    Food,
    /// Agents distributed in a Gaussian blob at the center (Petri dish style).
    Petri,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
/// Types of terrain-based steering bias.
pub enum TerrainType {
    /// No terrain effect.
    #[default]
    None,
    /// Smooth, flowing patterns based on Perlin noise.
    Smooth,
    /// Chaotic, turbulent patterns.
    Turbulent,
    /// Combination of smooth and turbulent layers.
    Mixed,
}

impl std::str::FromStr for TerrainType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "none" | "off" | "disabled" => Ok(TerrainType::None),
            "smooth" => Ok(TerrainType::Smooth),
            "turbulent" => Ok(TerrainType::Turbulent),
            "mixed" => Ok(TerrainType::Mixed),
            _ => Err(format!(
                "Invalid terrain type: {}. Must be one of: none, smooth, turbulent, mixed",
                s
            )),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
/// Global wind force configuration.
pub struct Wind {
    /// Horizontal wind strength (-1.0 to 1.0).
    pub dx: f32,
    /// Vertical wind strength (-1.0 to 1.0).
    pub dy: f32,
}

impl Wind {
    /// Creates a new wind vector.
    pub fn new(dx: f32, dy: f32) -> Self {
        Self { dx, dy }
    }
}

impl Default for Wind {
    fn default() -> Self {
        Self { dx: 0.0, dy: 0.0 }
    }
}

impl Validatable for Wind {
    fn validate(&self) -> Result<(), ValidationError> {
        if self.dx < -1.0 || self.dx > 1.0 {
            return Err(ValidationError::out_of_range("wind.dx", -1.0, 1.0, self.dx));
        }
        if self.dy < -1.0 || self.dy > 1.0 {
            return Err(ValidationError::out_of_range("wind.dy", -1.0, 1.0, self.dy));
        }
        if self.dx.abs() < 0.001 && self.dy.abs() < 0.001 {
            return Err(ValidationError::custom("wind cannot be zero vector"));
        }
        Ok(())
    }
}

impl std::str::FromStr for Wind {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use crate::validation::Validatable;

        let parts: Vec<&str> = s.split(',').collect();
        if parts.len() != 2 {
            return Err(format!("Wind must be in dx,dy format, got: {}", s));
        }

        let dx = parts[0]
            .parse::<f32>()
            .map_err(|e| format!("Invalid dx: {}", e))?;
        let dy = parts[1]
            .parse::<f32>()
            .map_err(|e| format!("Invalid dy: {}", e))?;

        let wind = Wind::new(dx, dy);
        Validatable::validate(&wind).map_err(|e| e.to_string())?;
        Ok(wind)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
/// A point attractor or repeller.
pub struct Attractor {
    /// X coordinate.
    pub x: f32,
    /// Y coordinate.
    pub y: f32,
    /// Strength of attraction (negative for repulsion).
    pub strength: f32,
}

impl Attractor {
    /// Creates a new attractor.
    pub fn new(x: f32, y: f32, strength: f32) -> Self {
        Self { x, y, strength }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
/// A temporary attractor created by mouse interaction.
pub struct MouseAttractor {
    /// X coordinate.
    pub x: f32,
    /// Y coordinate.
    pub y: f32,
    /// Strength of attraction/repulsion.
    pub strength: f32,
    /// Time of creation.
    pub created_at: std::time::Instant,
    /// Duration in seconds before expiration.
    pub timeout_seconds: f32,
}

impl MouseAttractor {
    /// Creates a new mouse attractor.
    pub fn new(x: f32, y: f32, strength: f32, timeout_seconds: f32) -> Self {
        Self {
            x,
            y,
            strength,
            created_at: std::time::Instant::now(),
            timeout_seconds,
        }
    }

    /// Checks if the attractor has expired.
    pub fn is_expired(&self) -> bool {
        self.created_at.elapsed().as_secs_f32() >= self.timeout_seconds
    }
}

#[derive(Debug, Clone, PartialEq)]
/// Mask data for image-based obstacles.
pub struct ObstacleMask {
    /// Flattened pixel data (normalized brightness).
    pub pixels: Vec<f32>,
    /// Width of the mask.
    pub width: usize,
    /// Height of the mask.
    pub height: usize,
}

impl ObstacleMask {
    /// Creates a mask from an image file.
    ///
    /// Resizes the image to target dimensions.
    pub fn from_image(
        image_path: &str,
        target_width: usize,
        target_height: usize,
        invert: bool,
    ) -> Result<Self, String> {
        let path = Path::new(image_path);

        if !path.exists() {
            return Err(format!("Image file not found: {}", image_path));
        }

        let img = ImageReader::open(path)
            .map_err(|e| format!("Failed to open image: {}", e))?
            .decode()
            .map_err(|e| format!("Failed to decode image: {}", e))?;

        let resized = img.resize_exact(
            target_width as u32,
            target_height as u32,
            image::imageops::FilterType::Nearest,
        );

        let pixels: Vec<f32> = resized
            .to_luma8()
            .pixels()
            .map(|p| {
                let brightness = p[0] as f32 / 255.0;
                if invert {
                    1.0 - brightness
                } else {
                    brightness
                }
            })
            .collect();

        Ok(Self {
            pixels,
            width: target_width,
            height: target_height,
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
/// Geometric shape or image obstacle definition.
pub enum Obstacle {
    /// Circular obstacle.
    Circle {
        /// Center X.
        x: f32,
        /// Center Y.
        y: f32,
        /// Radius.
        radius: f32,
    },
    /// Rectangular obstacle.
    Rect {
        /// Top-left X.
        x: f32,
        /// Top-left Y.
        y: f32,
        /// Width.
        width: f32,
        /// Height.
        height: f32,
    },
    /// Image-based obstacle mask.
    Image {
        /// Path to image file.
        path: String,
        /// Top-left X.
        x: f32,
        /// Top-left Y.
        y: f32,
        /// Width.
        width: usize,
        /// Height.
        height: usize,
        /// Whether to invert the image mask.
        invert: bool,
        /// Brightness threshold for collision.
        threshold: f32,
    },
}

impl Obstacle {
    /// Checks if a point is contained within the obstacle.
    pub fn contains(&self, px: f32, py: f32, mask: Option<&ObstacleMask>) -> bool {
        match self {
            Obstacle::Circle { x, y, radius } => {
                let dx = px - x;
                let dy = py - y;
                dx * dx + dy * dy <= radius * radius
            }
            Obstacle::Rect {
                x,
                y,
                width,
                height,
            } => px >= *x && px <= *x + *width && py >= *y && py <= *y + *height,
            Obstacle::Image {
                path: _,
                x,
                y,
                width,
                height,
                invert: _,
                threshold,
            } => {
                let lx = px - x;
                let ly = py - y;
                if lx < 0.0 || lx >= *width as f32 || ly < 0.0 || ly >= *height as f32 {
                    return false;
                }
                if let Some(m) = mask {
                    let ix = lx as usize;
                    let iy = ly as usize;
                    let idx = iy * m.width + ix;
                    if idx >= m.pixels.len() {
                        return false;
                    }
                    m.pixels[idx] >= *threshold
                } else {
                    false
                }
            }
        }
    }

    /// Calculates new heading after bouncing off the obstacle.
    pub fn bounce(&self, px: f32, py: f32, heading: f32, _mask: Option<&ObstacleMask>) -> f32 {
        match self {
            Obstacle::Circle { x, y, radius: _ } => {
                let dx = px - x;
                let dy = py - y;
                let normal_angle = dy.atan2(dx);
                let new_heading = 2.0 * normal_angle - heading + std::f32::consts::PI;
                normalize_angle(new_heading)
            }
            Obstacle::Rect {
                x,
                y,
                width,
                height,
            } => {
                let nearest_x = px.clamp(*x, *x + *width);
                let nearest_y = py.clamp(*y, *y + *height);
                let dx = px - nearest_x;
                let dy = py - nearest_y;
                if dx.abs() > dy.abs() {
                    -heading + std::f32::consts::PI
                } else {
                    -heading
                }
            }
            Obstacle::Image {
                path: _,
                x: _,
                y: _,
                width: _,
                height: _,
                invert: _,
                threshold: _,
            } => -heading,
        }
    }
}

/// Boundary handling mode for agent movement.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BoundaryMode {
    /// Agents bounce/reflect at boundaries (default).
    #[default]
    Bounce,
    /// Agents wrap around to opposite side (toroidal).
    Wrap,
}

/// Trail sampling method for agent sensing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SamplingMode {
    /// Fast nearest-pixel sampling (default).
    #[default]
    Nearest,
    /// Smooth bilinear interpolation.
    Bilinear,
}

impl std::str::FromStr for BoundaryMode {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "bounce" => Ok(BoundaryMode::Bounce),
            "wrap" | "toroidal" => Ok(BoundaryMode::Wrap),
            _ => Err(format!(
                "Invalid boundary mode: {}. Must be one of: bounce, wrap",
                s
            )),
        }
    }
}

/// 36 Points trail-based parameter modulation configuration.
///
/// This enables dynamic parameter adjustment based on the trail value at each agent's position,
/// creating diverse emergent behaviors as described in Sage Jenson's "36 Points" work.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PointConfig {
    /// Sensor distance base value (p1).
    pub sensor_distance_base: f32,
    /// Sensor distance multiplier (p2).
    pub sensor_distance_multiplier: f32,
    /// Sensor distance exponent (p3).
    pub sensor_distance_exponent: f32,

    /// Sensor angle base value in degrees (p4).
    pub sensor_angle_base: f32,
    /// Sensor angle multiplier (p5).
    pub sensor_angle_multiplier: f32,
    /// Sensor angle exponent (p6).
    pub sensor_angle_exponent: f32,

    /// Rotation angle base value in degrees (p7).
    pub rotation_angle_base: f32,
    /// Rotation angle multiplier (p8).
    pub rotation_angle_multiplier: f32,
    /// Rotation angle exponent (p9).
    pub rotation_angle_exponent: f32,

    /// Step size base value (p10).
    pub step_size_base: f32,
    /// Step size multiplier (p11).
    pub step_size_multiplier: f32,
    /// Step size exponent (p12).
    pub step_size_exponent: f32,

    /// Absolute vertical offset in pixels (p13).
    pub vertical_offset: f32,
    /// Heading-relative offset in pixels (p14).
    pub heading_offset: f32,
    /// Trail value rescaling factor (p15).
    pub trail_rescale: f32,
}

impl Default for PointConfig {
    fn default() -> Self {
        Self {
            // Default: no modulation (multipliers = 0, exponents = 1)
            sensor_distance_base: agent_consts::DEFAULT_SENSOR_DISTANCE,
            sensor_distance_multiplier: 0.0,
            sensor_distance_exponent: 1.0,
            sensor_angle_base: agent_consts::DEFAULT_SENSOR_ANGLE,
            sensor_angle_multiplier: 0.0,
            sensor_angle_exponent: 1.0,
            rotation_angle_base: agent_consts::DEFAULT_ROTATION_ANGLE,
            rotation_angle_multiplier: 0.0,
            rotation_angle_exponent: 1.0,
            step_size_base: agent_consts::DEFAULT_STEP_SIZE,
            step_size_multiplier: 0.0,
            step_size_exponent: 1.0,
            vertical_offset: 0.0,
            heading_offset: 0.0,
            trail_rescale: 1.0,
        }
    }
}

/// Computed modulated parameters for an agent.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ModulatedParams {
    /// Modulated sensor distance.
    pub sensor_distance: f32,
    /// Modulated sensor angle in degrees.
    pub sensor_angle: f32,
    /// Modulated rotation angle in degrees.
    pub rotation_angle: f32,
    /// Modulated step size.
    pub step_size: f32,
}

impl PointConfig {
    /// Compute modulated parameters based on trail value x.
    ///
    /// Formulas:
    /// - sensor_distance = p1 + p2 * x^p3
    /// - sensor_angle    = p4 + p5 * x^p6
    /// - rotation_angle  = p7 + p8 * x^p9
    /// - step_size       = p10 + p11 * x^p12
    ///
    /// # Arguments
    /// * `x` - Trail value at agent position (should be in [0, 1])
    ///
    /// # Returns
    /// A `ModulatedParams` struct containing:
    /// - `sensor_distance`: Modulated sensor distance in pixels
    /// - `sensor_angle`: Modulated sensor angle in degrees
    /// - `rotation_angle`: Modulated rotation angle in degrees
    /// - `step_size`: Modulated step size in pixels
    pub fn compute_params(&self, x: f32) -> ModulatedParams {
        // Apply rescale factor and clamp to [0, 1]
        let x = (x * self.trail_rescale).clamp(0.0, 1.0);

        // Helper to compute modulated value with formula: base + multiplier * x^exponent
        let compute = |base: f32, multiplier: f32, exponent: f32| -> f32 {
            if multiplier == 0.0 || x == 0.0 {
                base
            } else if exponent == 1.0 {
                base + multiplier * x
            } else {
                base + multiplier * x.powf(exponent)
            }
        };

        ModulatedParams {
            sensor_distance: compute(
                self.sensor_distance_base,
                self.sensor_distance_multiplier,
                self.sensor_distance_exponent,
            )
            .clamp(
                agent_consts::MIN_SENSOR_DISTANCE,
                agent_consts::MAX_SENSOR_DISTANCE,
            ),
            sensor_angle: compute(
                self.sensor_angle_base,
                self.sensor_angle_multiplier,
                self.sensor_angle_exponent,
            )
            .clamp(
                agent_consts::MIN_SENSOR_ANGLE,
                agent_consts::MAX_SENSOR_ANGLE,
            ),
            rotation_angle: compute(
                self.rotation_angle_base,
                self.rotation_angle_multiplier,
                self.rotation_angle_exponent,
            )
            .clamp(
                agent_consts::MIN_ROTATION_ANGLE,
                agent_consts::MAX_ROTATION_ANGLE,
            ),
            step_size: compute(
                self.step_size_base,
                self.step_size_multiplier,
                self.step_size_exponent,
            )
            .clamp(agent_consts::MIN_STEP_SIZE, agent_consts::MAX_STEP_SIZE),
        }
    }

    /// Returns true if this config has any modulation enabled.
    pub fn has_modulation(&self) -> bool {
        self.sensor_distance_multiplier != 0.0
            || self.sensor_angle_multiplier != 0.0
            || self.rotation_angle_multiplier != 0.0
            || self.step_size_multiplier != 0.0
    }
}

/// Particle respawn configuration.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RespawnConfig {
    /// Interval in frames between respawn checks (0 = disabled).
    pub interval: u32,
    /// Base probability of respawn when interval is reached (0.0-1.0).
    pub base_probability: f32,
    /// Whether respawn probability depends on trail value.
    pub trail_dependent: bool,
    /// Maximum respawn probability multiplier when trail is high.
    pub max_probability_multiplier: f32,
}

impl Default for RespawnConfig {
    fn default() -> Self {
        Self {
            interval: 0, // Disabled by default
            base_probability: 0.01,
            trail_dependent: false,
            max_probability_multiplier: 1.0,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
/// Configuration for a single agent species.
pub struct SpeciesConfig {
    /// Species name.
    pub name: String,
    /// Population count.
    pub count: usize,
    /// Sensor angle (degrees).
    pub sensor_angle: f32,
    /// Rotation angle (degrees).
    pub rotation_angle: f32,
    /// Step size (speed).
    pub step_size: f32,
    /// Amount of pheromone deposited.
    pub deposit_amount: f32,
    /// Color as RGB.
    pub color: RgbColor,
    /// Trail-based parameter modulation (36 Points).
    pub trail_modulation: Option<PointConfig>,
}

impl Default for SpeciesConfig {
    fn default() -> Self {
        Self {
            name: "default".to_string(),
            count: population::DEFAULT_POPULATION,
            sensor_angle: agent_consts::DEFAULT_SENSOR_ANGLE,
            rotation_angle: agent_consts::DEFAULT_ROTATION_ANGLE,
            step_size: agent_consts::DEFAULT_STEP_SIZE,
            deposit_amount: agent_consts::DEFAULT_DEPOSIT_AMOUNT,
            color: RgbColor::from_hex(0x228b22), // Forest green
            trail_modulation: None,
        }
    }
}

#[derive(Debug, Clone)]
/// Global simulation configuration.
pub struct SimConfig {
    /// Sensor angle (degrees).
    pub sensor_angle: f32,
    /// Sensor offset distance (pixels).
    pub sensor_distance: f32,
    /// Rotation angle (degrees).
    pub rotation_angle: f32,
    /// Agent speed (pixels/step).
    pub step_size: f32,
    /// Trail decay factor (0.0-1.0).
    pub decay_factor: f32,
    /// Amount of trail deposited per step.
    pub deposit_amount: f32,
    /// Diffusion algorithm.
    pub diffusion_kernel: DiffusionKernel,
    /// Sigma for Gaussian diffusion.
    pub diffusion_sigma: f32,
    /// Max brightness for normalization.
    pub max_brightness: f32,
    /// Time scale multiplier (0.1-10.0).
    pub time_scale: f32,
    /// List of active attractors.
    pub attractors: Vec<Attractor>,
    /// Global attractor strength multiplier.
    pub attractor_strength: f32,
    /// Temporary mouse attractors.
    pub mouse_attractors: Vec<MouseAttractor>,
    /// Timeout for mouse attractors (seconds).
    pub mouse_timeout: f32,
    /// Configuration for each species.
    pub species_configs: Vec<SpeciesConfig>,
    /// Whether to use separate trail maps per species.
    pub separate_species_trails: bool,
    /// Whether to use SIMD acceleration.
    pub use_simd: bool,
    /// Path to food image for initialization.
    pub food_image_path: Option<String>,
    /// Whether to invert food image brightness.
    pub food_image_invert: bool,
    /// Scaling factor for food image.
    pub food_image_scale: f32,
    /// List of obstacles.
    pub obstacles: Vec<Obstacle>,
    /// Loaded masks for image obstacles.
    pub obstacle_masks: Vec<Option<ObstacleMask>>,
    /// Global wind force.
    pub wind: Option<Wind>,
    /// Active terrain effect.
    pub terrain: TerrainType,
    /// Strength of terrain effect.
    pub terrain_strength: f32,
    /// Background color hex code.
    pub background_color: Option<String>,
    /// Preferred initialization mode for this config (if any).
    pub preferred_init_mode: Option<InitMode>,
    /// Boundary handling mode (bounce or wrap).
    pub boundary_mode: BoundaryMode,
    /// Particle respawn configuration.
    pub respawn_config: RespawnConfig,
    /// Trail sampling method (nearest or bilinear).
    pub sampling_mode: SamplingMode,
}

impl SimConfig {
    /// Returns the total population across all species.
    pub fn total_population(&self) -> usize {
        self.species_configs.iter().map(|s| s.count).sum()
    }

    /// Loads mask data for all image-based obstacles.
    pub fn load_obstacle_masks(&mut self) -> Result<(), String> {
        self.obstacle_masks.clear();
        for obstacle in &self.obstacles {
            match obstacle {
                Obstacle::Image {
                    path,
                    width,
                    height,
                    invert,
                    ..
                } => {
                    let mask = ObstacleMask::from_image(path, *width, *height, *invert)?;
                    self.obstacle_masks.push(Some(mask));
                }
                _ => {
                    self.obstacle_masks.push(None);
                }
            }
        }
        Ok(())
    }

    /// Adds a new mouse-controlled attractor.
    pub fn add_mouse_attractor(&mut self, x: f32, y: f32, strength: f32) {
        self.mouse_attractors
            .push(MouseAttractor::new(x, y, strength, self.mouse_timeout));
    }

    /// Removes mouse attractors that have timed out.
    pub fn remove_expired_mouse_attractors(&mut self) {
        self.mouse_attractors.retain(|ma| !ma.is_expired());
    }

    /// Returns a combined list of all active attractors (static + mouse).
    ///
    /// # Performance
    /// If there are no mouse attractors, returns a reference to the static attractors
    /// without cloning. Otherwise, returns an owned vector with both static and mouse attractors.
    pub fn effective_attractors(&self) -> Cow<'_, [Attractor]> {
        if self.mouse_attractors.is_empty() {
            // Fast path: no mouse attractors, return borrowed reference
            Cow::Borrowed(&self.attractors)
        } else {
            // Slow path: need to combine static and mouse attractors
            let mut result = self.attractors.clone();
            for ma in &self.mouse_attractors {
                result.push(Attractor::new(ma.x, ma.y, ma.strength));
            }
            Cow::Owned(result)
        }
    }
}

impl Default for SimConfig {
    fn default() -> Self {
        Self {
            sensor_angle: agent_consts::DEFAULT_SENSOR_ANGLE,
            sensor_distance: agent_consts::DEFAULT_SENSOR_DISTANCE,
            rotation_angle: agent_consts::DEFAULT_ROTATION_ANGLE,
            step_size: agent_consts::DEFAULT_STEP_SIZE,
            decay_factor: trail_consts::DEFAULT_DECAY_FACTOR,
            deposit_amount: agent_consts::DEFAULT_DEPOSIT_AMOUNT,
            diffusion_kernel: DiffusionKernel::Gaussian,
            diffusion_sigma: trail_consts::DEFAULT_DIFFUSION_SIGMA,
            max_brightness: trail_consts::DEFAULT_MAX_BRIGHTNESS,
            time_scale: time_consts::DEFAULT_TIME_SCALE,
            attractors: Vec::new(),
            attractor_strength: env_consts::DEFAULT_ATTRACTOR_STRENGTH,
            mouse_attractors: Vec::new(),
            mouse_timeout: env_consts::DEFAULT_MOUSE_TIMEOUT,
            species_configs: vec![SpeciesConfig::default()],
            separate_species_trails: false,
            use_simd: true,
            food_image_path: Some(food_img_consts::DEFAULT_FOOD_PATH.to_string()),
            food_image_invert: food_img_consts::DEFAULT_FOOD_INVERT,
            food_image_scale: food_img_consts::DEFAULT_FOOD_SCALE,
            obstacles: Vec::new(),
            obstacle_masks: Vec::new(),
            wind: None,
            terrain: TerrainType::None,
            terrain_strength: env_consts::DEFAULT_TERRAIN_STRENGTH,
            background_color: None,
            preferred_init_mode: Some(InitMode::Food),
            boundary_mode: BoundaryMode::Bounce,
            respawn_config: RespawnConfig::default(),
            sampling_mode: SamplingMode::Nearest,
        }
    }
}

// Validation implementations using the Validatable trait
use crate::error::ValidationError;
use crate::validation::{rules, Validatable};

impl Validatable for SimConfig {
    fn validate(&self) -> Result<(), ValidationError> {
        // Check that at least one species is configured
        if self.species_configs.is_empty() {
            return Err(ValidationError::custom(
                "at least one species must be configured",
            ));
        }

        // Validate total population
        let total_pop: usize = self.species_configs.iter().map(|s| s.count).sum();
        if !(population::MIN_POPULATION..=population::MAX_POPULATION).contains(&total_pop) {
            return Err(ValidationError::custom(format!(
                "total population must be between {} and {}, got {}",
                population::MIN_POPULATION,
                population::MAX_POPULATION,
                total_pop
            )));
        }

        // Validate agent parameters
        rules::SENSOR_ANGLE.validate_f32(self.sensor_angle)?;
        rules::SENSOR_DISTANCE.validate_f32(self.sensor_distance)?;
        rules::ROTATION_ANGLE.validate_f32(self.rotation_angle)?;
        rules::STEP_SIZE.validate_f32(self.step_size)?;
        rules::DEPOSIT_AMOUNT.validate_f32(self.deposit_amount)?;

        // Validate trail parameters
        rules::DECAY_FACTOR.validate_f32(self.decay_factor)?;
        rules::MAX_BRIGHTNESS.validate_f32(self.max_brightness)?;
        rules::DIFFUSION_SIGMA.validate_f32(self.diffusion_sigma)?;

        // Validate time and environment parameters
        rules::TIME_SCALE.validate_f32(self.time_scale)?;
        rules::ATTRACTOR_STRENGTH.validate_f32(self.attractor_strength)?;
        rules::TERRAIN_STRENGTH.validate_f32(self.terrain_strength)?;

        // Validate individual attractors
        for (i, attractor) in self.attractors.iter().enumerate() {
            if attractor.strength < environment::MIN_ATTRACTOR_STRENGTH
                || attractor.strength > environment::MAX_ATTRACTOR_STRENGTH
            {
                return Err(ValidationError::out_of_range(
                    format!("attractor[{}].strength", i),
                    environment::MIN_ATTRACTOR_STRENGTH,
                    environment::MAX_ATTRACTOR_STRENGTH,
                    attractor.strength,
                ));
            }
        }

        // Validate species configs
        for species in &self.species_configs {
            Validatable::validate(species)?;
        }

        // Validate wind if present
        if let Some(ref wind) = self.wind {
            Validatable::validate(wind)?;
        }

        Ok(())
    }
}

impl Validatable for SpeciesConfig {
    fn validate(&self) -> Result<(), ValidationError> {
        // Validate count
        if self.count < pop_consts::MIN_SPECIES_COUNT || self.count > pop_consts::MAX_SPECIES_COUNT
        {
            return Err(ValidationError::out_of_range(
                format!("species '{}' count", self.name),
                pop_consts::MIN_SPECIES_COUNT,
                pop_consts::MAX_SPECIES_COUNT,
                self.count,
            ));
        }

        // Validate sensor angle
        if self.sensor_angle < agent_consts::MIN_SENSOR_ANGLE
            || self.sensor_angle > agent_consts::MAX_SENSOR_ANGLE
        {
            return Err(ValidationError::out_of_range(
                format!("species '{}' sensor_angle", self.name),
                agent_consts::MIN_SENSOR_ANGLE,
                agent_consts::MAX_SENSOR_ANGLE,
                self.sensor_angle,
            ));
        }

        // Validate rotation angle
        if self.rotation_angle < agent_consts::MIN_ROTATION_ANGLE
            || self.rotation_angle > agent_consts::MAX_ROTATION_ANGLE
        {
            return Err(ValidationError::out_of_range(
                format!("species '{}' rotation_angle", self.name),
                agent_consts::MIN_ROTATION_ANGLE,
                agent_consts::MAX_ROTATION_ANGLE,
                self.rotation_angle,
            ));
        }

        // Validate step size
        if self.step_size < agent_consts::MIN_STEP_SIZE
            || self.step_size > agent_consts::MAX_STEP_SIZE
        {
            return Err(ValidationError::out_of_range(
                format!("species '{}' step_size", self.name),
                agent_consts::MIN_STEP_SIZE,
                agent_consts::MAX_STEP_SIZE,
                self.step_size,
            ));
        }

        // Validate deposit amount
        if self.deposit_amount < agent_consts::MIN_DEPOSIT_AMOUNT
            || self.deposit_amount > agent_consts::MAX_DEPOSIT_AMOUNT
        {
            return Err(ValidationError::out_of_range(
                format!("species '{}' deposit_amount", self.name),
                agent_consts::MIN_DEPOSIT_AMOUNT,
                agent_consts::MAX_DEPOSIT_AMOUNT,
                self.deposit_amount,
            ));
        }

        Ok(())
    }
}

impl From<Preset> for SimConfig {
    fn from(preset: Preset) -> Self {
        let mut config = Self::default();
        preset.apply(&mut config);
        config
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::PI;

    #[test]
    fn test_default_config() {
        let config = SimConfig::default();
        assert_eq!(config.total_population(), 50_000);
        assert_eq!(config.sensor_angle, 22.5);
        assert_eq!(config.sensor_distance, 9.0);
        assert_eq!(config.rotation_angle, 45.0);
        assert_eq!(config.step_size, 1.0);
        assert_eq!(config.decay_factor, 0.5);
        assert_eq!(config.deposit_amount, 5.0);
        assert_eq!(config.max_brightness, 100.0);
    }

    #[test]
    fn test_validate_default() {
        let config = SimConfig::default();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_validate_population_too_low() {
        let config = SimConfig {
            species_configs: vec![SpeciesConfig {
                count: 500,
                ..Default::default()
            }],
            ..Default::default()
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_validate_population_too_high() {
        let config = SimConfig {
            species_configs: vec![SpeciesConfig {
                count: 300_000,
                ..Default::default()
            }],
            ..Default::default()
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_validate_sensor_angle() {
        let config = SimConfig {
            sensor_angle: 100.0,
            ..Default::default()
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_validate_decay_factor() {
        let config = SimConfig {
            decay_factor: 1.0,
            ..Default::default()
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_validate_max_brightness_too_low() {
        let config = SimConfig {
            max_brightness: 0.5,
            ..Default::default()
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_validate_max_brightness_too_high() {
        let config = SimConfig {
            max_brightness: 1500.0,
            ..Default::default()
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_validate_attractor_strength_too_low() {
        let config = SimConfig {
            attractor_strength: 0.05,
            ..Default::default()
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_validate_attractor_strength_too_high() {
        let config = SimConfig {
            attractor_strength: 15.0,
            ..Default::default()
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_validate_attractor_strength_valid() {
        let config = SimConfig {
            attractor_strength: 5.0,
            ..Default::default()
        };
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_attractor_creation() {
        let attractor = Attractor::new(200.0, 200.0, 1.0);
        assert_eq!(attractor.x, 200.0);
        assert_eq!(attractor.y, 200.0);
        assert_eq!(attractor.strength, 1.0);
    }

    #[test]
    fn test_negative_attractor_strength() {
        let attractor = Attractor::new(200.0, 200.0, -1.0);
        assert_eq!(attractor.strength, -1.0);
    }

    #[test]
    fn test_species_config_default() {
        let species = SpeciesConfig::default();
        assert_eq!(species.count, 50_000);
        assert_eq!(species.sensor_angle, 22.5);
        assert_eq!(species.rotation_angle, 45.0);
        assert_eq!(species.step_size, 1.0);
        assert_eq!(species.deposit_amount, 5.0);
    }

    #[test]
    fn test_species_config_validate_count_too_low() {
        let species = SpeciesConfig {
            count: 50,
            ..Default::default()
        };
        assert!(species.validate().is_err());
    }

    #[test]
    fn test_species_config_validate_count_too_high() {
        let species = SpeciesConfig {
            count: 300_000,
            ..Default::default()
        };
        assert!(species.validate().is_err());
    }

    #[test]
    fn test_total_population_single_species() {
        let config = SimConfig {
            species_configs: vec![SpeciesConfig {
                count: 10000,
                ..Default::default()
            }],
            ..Default::default()
        };
        assert_eq!(config.total_population(), 10000);
    }

    #[test]
    fn test_total_population_multiple_species() {
        let config = SimConfig {
            species_configs: vec![
                SpeciesConfig {
                    count: 10000,
                    ..Default::default()
                },
                SpeciesConfig {
                    count: 20000,
                    name: "second".to_string(),
                    color: RgbColor::from_hex(0xff0000),
                    ..Default::default()
                },
            ],
            ..Default::default()
        };
        assert_eq!(config.total_population(), 30000);
    }

    #[test]
    fn test_obstacle_circle_contains() {
        let circle = Obstacle::Circle {
            x: 100.0,
            y: 100.0,
            radius: 50.0,
        };
        assert!(circle.contains(100.0, 100.0, None));
        assert!(circle.contains(100.0, 150.0, None));
        assert!(circle.contains(150.0, 100.0, None));
        assert!(!circle.contains(200.0, 100.0, None));
        assert!(!circle.contains(100.0, 200.0, None));
    }

    #[test]
    fn test_obstacle_rect_contains() {
        let rect = Obstacle::Rect {
            x: 100.0,
            y: 100.0,
            width: 50.0,
            height: 50.0,
        };
        assert!(rect.contains(100.0, 100.0, None));
        assert!(rect.contains(150.0, 150.0, None));
        assert!(!rect.contains(99.0, 100.0, None));
        assert!(!rect.contains(100.0, 99.0, None));
        assert!(!rect.contains(151.0, 100.0, None));
        assert!(!rect.contains(100.0, 151.0, None));
    }

    #[test]
    fn test_obstacle_circle_bounce() {
        let circle = Obstacle::Circle {
            x: 100.0,
            y: 100.0,
            radius: 50.0,
        };
        let heading = circle.bounce(100.0, 60.0, 0.0, None);
        assert!(
            heading.is_finite(),
            "Bounce should return a valid heading, got {}",
            heading
        );
    }

    #[test]
    fn test_obstacle_rect_bounce() {
        let rect = Obstacle::Rect {
            x: 100.0,
            y: 100.0,
            width: 50.0,
            height: 50.0,
        };
        let heading = rect.bounce(120.0, 100.0, 0.0, None);
        assert!(
            heading.is_finite(),
            "Bounce should return a valid heading, got {}",
            heading
        );
    }

    #[test]
    fn test_obstacle_mask_from_image_nonexistent() {
        let result = ObstacleMask::from_image("nonexistent.png", 100, 100, false);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not found"));
    }

    #[test]
    fn test_sim_config_load_obstacle_masks() {
        let mut config = SimConfig {
            obstacles: vec![Obstacle::Circle {
                x: 100.0,
                y: 100.0,
                radius: 50.0,
            }],
            ..Default::default()
        };
        let result = config.load_obstacle_masks();
        assert!(result.is_ok());
        assert_eq!(config.obstacle_masks.len(), 1);
        assert!(config.obstacle_masks[0].is_none());
    }

    #[test]
    fn test_wind_creation() {
        let wind = Wind::new(0.5, 0.5);
        assert_eq!(wind.dx, 0.5);
        assert_eq!(wind.dy, 0.5);
    }

    #[test]
    fn test_wind_validate_valid() {
        let wind = Wind::new(1.0, 1.0);
        assert!(wind.validate().is_ok());

        let wind = Wind::new(-1.0, 0.0);
        assert!(wind.validate().is_ok());

        let wind = Wind::new(0.0, -1.0);
        assert!(wind.validate().is_ok());
    }

    #[test]
    fn test_wind_validate_invalid_dx() {
        let wind = Wind::new(1.5, 0.0);
        assert!(wind.validate().is_err());
    }

    #[test]
    fn test_wind_validate_invalid_dy() {
        let wind = Wind::new(0.0, 1.5);
        assert!(wind.validate().is_err());
    }

    #[test]
    fn test_wind_validate_zero() {
        let wind = Wind::new(0.0, 0.0);
        assert!(wind.validate().is_err());
    }

    #[test]
    fn test_wind_parse() {
        let wind: Wind = "0.5,0.5".parse().unwrap();
        assert_eq!(wind.dx, 0.5);
        assert_eq!(wind.dy, 0.5);

        let wind: Wind = "-0.3,0.7".parse().unwrap();
        assert_eq!(wind.dx, -0.3);
        assert_eq!(wind.dy, 0.7);
    }

    #[test]
    fn test_wind_parse_invalid() {
        assert!("0.5".parse::<Wind>().is_err());
        assert!("0.5,0.5,extra".parse::<Wind>().is_err());
        assert!("abc,def".parse::<Wind>().is_err());
    }

    #[test]
    fn test_terrain_type_parse() {
        assert_eq!("none".parse::<TerrainType>().unwrap(), TerrainType::None);
        assert_eq!("off".parse::<TerrainType>().unwrap(), TerrainType::None);
        assert_eq!(
            "smooth".parse::<TerrainType>().unwrap(),
            TerrainType::Smooth
        );
        assert_eq!(
            "turbulent".parse::<TerrainType>().unwrap(),
            TerrainType::Turbulent
        );
        assert_eq!("mixed".parse::<TerrainType>().unwrap(), TerrainType::Mixed);

        assert_eq!("NONE".parse::<TerrainType>().unwrap(), TerrainType::None);
        assert_eq!(
            "Smooth".parse::<TerrainType>().unwrap(),
            TerrainType::Smooth
        );
    }

    #[test]
    fn test_terrain_type_parse_invalid() {
        assert!("invalid".parse::<TerrainType>().is_err());
        assert!("chaos".parse::<TerrainType>().is_err());
    }

    #[test]
    fn test_sim_config_wind_field() {
        let config = SimConfig {
            wind: Some(Wind::new(0.5, 0.0)),
            ..Default::default()
        };
        assert!(config.wind.is_some());
        assert_eq!(config.wind.unwrap().dx, 0.5);
    }

    #[test]
    fn test_sim_config_terrain_field() {
        let config = SimConfig {
            terrain: TerrainType::Turbulent,
            terrain_strength: 2.0,
            ..Default::default()
        };
        assert_eq!(config.terrain, TerrainType::Turbulent);
        assert_eq!(config.terrain_strength, 2.0);
    }

    #[test]
    fn test_validate_terrain_strength_too_low() {
        let config = SimConfig {
            terrain_strength: 0.05,
            ..Default::default()
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_validate_terrain_strength_too_high() {
        let config = SimConfig {
            terrain_strength: 10.0,
            ..Default::default()
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_validate_wind_invalid() {
        let config = SimConfig {
            wind: Some(Wind::new(1.5, 0.0)),
            ..Default::default()
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_effective_attractors() {
        let mut config = SimConfig {
            attractors: vec![Attractor::new(10.0, 10.0, 1.0)],
            ..Default::default()
        };
        config.add_mouse_attractor(20.0, 20.0, 2.0);
        let effective = config.effective_attractors();
        assert_eq!(effective.len(), 2);
        assert_eq!(effective[0].strength, 1.0);
        assert_eq!(effective[1].strength, 2.0);
    }

    #[test]
    fn test_mouse_attractor_expiry() {
        let ma = MouseAttractor::new(10.0, 10.0, 1.0, 0.01);
        assert!(!ma.is_expired());
        std::thread::sleep(std::time::Duration::from_millis(20));
        assert!(ma.is_expired());
    }

    #[test]
    fn test_remove_expired_mouse_attractors() {
        let mut config = SimConfig {
            mouse_timeout: 0.01,
            ..Default::default()
        };
        config.add_mouse_attractor(10.0, 10.0, 1.0);
        assert_eq!(config.mouse_attractors.len(), 1);
        std::thread::sleep(std::time::Duration::from_millis(20));
        config.remove_expired_mouse_attractors();
        assert_eq!(config.mouse_attractors.len(), 0);
    }

    #[test]
    fn test_presets_valid() {
        let presets = [
            Preset::Network,
            Preset::Exploratory,
            Preset::Tendrils,
            Preset::Organic,
            Preset::Minimal,
            Preset::Moss,
            Preset::Cosmic,
            Preset::Fire,
            Preset::Zen,
            Preset::Storm,
            Preset::River,
            Preset::Ethereal,
            Preset::PetriDish,
            Preset::Vortex,
            Preset::Lightning,
            Preset::Crystal,
            Preset::ChaosEdge,
            Preset::Blob,
            Preset::Worm,
            Preset::Pulse,
            Preset::Coral,
            Preset::Flocking,
            Preset::Maze,
            Preset::Ripple,
            Preset::Vortex36,
            Preset::Chameleon,
            Preset::DynamicTendrils,
            Preset::MorphingCoral,
            Preset::ReactiveSwarm,
            Preset::DuelingModulators,
        ];
        for preset in presets {
            let config: SimConfig = preset.into();
            assert!(
                config.validate().is_ok(),
                "Preset {:?} failed validation: {:?}",
                preset,
                config.validate()
            );
        }
    }

    #[test]
    fn test_obstacle_rect_bounce_sides() {
        let rect = Obstacle::Rect {
            x: 100.0,
            y: 100.0,
            width: 50.0,
            height: 50.0,
        };
        // Bounce off top/bottom (dy > dx)
        let h1 = rect.bounce(125.0, 99.9, 0.1, None);
        assert!((h1 - (-0.1)).abs() < 0.001);
        // Bounce off left/right (dx > dy)
        let h2 = rect.bounce(99.9, 125.0, 0.1, None);
        assert!((h2 - (PI - 0.1)).abs() < 0.001);
    }

    #[test]
    fn test_species_config_validate_all() {
        let s = SpeciesConfig {
            sensor_angle: 1.0,
            ..Default::default()
        };
        assert!(s.validate().is_err());
        let s = SpeciesConfig {
            rotation_angle: 1.0,
            ..Default::default()
        };
        assert!(s.validate().is_err());
        let s = SpeciesConfig {
            step_size: 0.005,
            ..Default::default()
        };
        assert!(s.validate().is_err());
        let s = SpeciesConfig {
            deposit_amount: 0.05,
            ..Default::default()
        };
        assert!(s.validate().is_err());
    }

    #[test]
    fn test_validatable_trait() {
        use crate::validation::Validatable;

        let valid_config = SimConfig::default();
        assert!(valid_config.validate().is_ok());

        let invalid_config = SimConfig {
            sensor_angle: 200.0, // Invalid
            ..Default::default()
        };
        assert!(invalid_config.validate().is_err());
    }

    #[test]
    fn test_species_validatable_trait() {
        use crate::validation::Validatable;

        let valid_species = SpeciesConfig::default();
        assert!(valid_species.validate().is_ok());

        let invalid_species = SpeciesConfig {
            count: 50, // Below minimum
            ..Default::default()
        };
        assert!(invalid_species.validate().is_err());
    }
}
