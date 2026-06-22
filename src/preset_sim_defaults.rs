//! Declarative per-preset sim-layer spec.
//!
//! `PresetSimDefaults` replaces the imperative `Preset::apply()`: each preset
//! is expressed as a struct literal using `..Default::default()` for fields it
//! doesn't change. `apply_to` writes every field into a `SimConfig`. `Default`
//! mirrors `SimConfig::default()` so unset fields resolve byte-identically.
//!
//! **afterglow is NOT a field here** — it was vestigial in the assembled config
//! and moves to `RenderArtDefaults` in a later commit. `decay_gamma`,
//! `deposit_curve`, `deposit_scale`, `deposit_gamma` ARE sim levers and are kept.

use crate::config_defaults::{agent, trail};
use crate::simulation::config::{
    Attractor, BoundaryMode, DepositCurve, DiffusionKernel, InitMode, Obstacle, Preset,
    RespawnConfig, SamplingMode, SimConfig, SpeciesConfig, Wind, WindowFrame,
};

/// Declarative per-preset sim-layer spec. Replaces the imperative
/// `Preset::apply()`: each preset is a struct literal using `..Default::default()`
/// for fields it doesn't change. `apply_to` writes every field into a `SimConfig`.
/// `Default` mirrors `SimConfig::default()` so unset fields resolve identically.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct PresetSimDefaults {
    pub sensor_angle: f32,
    pub sensor_distance: f32,
    pub rotation_angle: f32,
    pub step_size: f32,
    pub decay_factor: f32,
    pub deposit_amount: f32,
    pub diffusion_kernel: DiffusionKernel,
    pub diffusion_sigma: f32,
    pub max_brightness: f32,
    pub decay_gamma: f32,
    pub diffuse_weight: f32,
    pub deposit_curve: DepositCurve,
    pub deposit_scale: f32,
    pub deposit_gamma: f32,
    pub boundary_mode: BoundaryMode,
    pub window_frame: WindowFrame,
    pub preferred_init_mode: Option<InitMode>,
    pub wind: Option<Wind>,
    pub background_color: Option<String>,
    pub obstacles: Vec<Obstacle>,
    pub species_configs: Vec<SpeciesConfig>,
    pub deposit_cap: f32,
    pub attractors: Vec<Attractor>,
    pub separate_species_trails: bool,
    pub sampling_mode: SamplingMode,
    pub respawn_config: RespawnConfig,
}

impl Default for PresetSimDefaults {
    fn default() -> Self {
        // Mirror SimConfig::default() (config.rs:2030-2086) for every scalar so a
        // preset that leaves a field unset resolves byte-identically.
        Self {
            sensor_angle: agent::DEFAULT_SENSOR_ANGLE,
            sensor_distance: agent::DEFAULT_SENSOR_DISTANCE,
            rotation_angle: agent::DEFAULT_ROTATION_ANGLE,
            step_size: agent::DEFAULT_STEP_SIZE,
            decay_factor: trail::DEFAULT_DECAY_FACTOR,
            deposit_amount: agent::DEFAULT_DEPOSIT_AMOUNT,
            diffusion_kernel: DiffusionKernel::Gaussian,
            diffusion_sigma: trail::DEFAULT_DIFFUSION_SIGMA,
            max_brightness: trail::DEFAULT_MAX_BRIGHTNESS,
            decay_gamma: trail::DEFAULT_DECAY_GAMMA,
            diffuse_weight: trail::DEFAULT_DIFFUSE_WEIGHT,
            deposit_curve: DepositCurve::default(),
            deposit_scale: trail::DEFAULT_DEPOSIT_SCALE,
            deposit_gamma: trail::DEFAULT_DEPOSIT_GAMMA,
            boundary_mode: BoundaryMode::Bounce,
            window_frame: WindowFrame::Frame,
            preferred_init_mode: Some(InitMode::Food),
            wind: None,
            background_color: None,
            obstacles: Vec::new(),
            species_configs: vec![SpeciesConfig::default()],
            deposit_cap: trail::DEFAULT_DEPOSIT_CAP,
            attractors: Vec::new(),
            separate_species_trails: false,
            sampling_mode: SamplingMode::Nearest,
            respawn_config: RespawnConfig::default(),
        }
    }
}

impl PresetSimDefaults {
    /// Materialize this spec into `config`. The single source of truth used by
    /// both `From<Preset> for SimConfig` (the shim) and `ProfileOverrides::resolve_sim`.
    pub(crate) fn apply_to(&self, config: &mut SimConfig) {
        config.sensor_angle = self.sensor_angle;
        config.sensor_distance = self.sensor_distance;
        config.rotation_angle = self.rotation_angle;
        config.step_size = self.step_size;
        config.decay_factor = self.decay_factor;
        config.deposit_amount = self.deposit_amount;
        config.diffusion_kernel = self.diffusion_kernel;
        config.diffusion_sigma = self.diffusion_sigma;
        config.max_brightness = self.max_brightness;
        config.decay_gamma = self.decay_gamma;
        config.diffuse_weight = self.diffuse_weight;
        config.deposit_curve = self.deposit_curve;
        config.deposit_scale = self.deposit_scale;
        config.deposit_gamma = self.deposit_gamma;
        config.boundary_mode = self.boundary_mode;
        config.window_frame = self.window_frame;
        config.preferred_init_mode = self.preferred_init_mode;
        config.wind = self.wind;
        config.background_color = self.background_color.clone();
        config.obstacles = self.obstacles.clone();
        config.species_configs = self.species_configs.clone();
        config.deposit_cap = self.deposit_cap;
        config.attractors = self.attractors.clone();
        config.separate_species_trails = self.separate_species_trails;
        config.sampling_mode = self.sampling_mode;
        config.respawn_config = self.respawn_config;
    }
}

impl From<Preset> for PresetSimDefaults {
    fn from(preset: Preset) -> Self {
        use crate::render::palette::RgbColor;
        use crate::simulation::config::{InitMode, Obstacle, PointConfig, Wind};
        match preset {
            // Dense network of branching paths (config.rs:387-400)
            Preset::Network => Self {
                sensor_angle: 15.0,
                rotation_angle: 30.0,
                decay_factor: 0.85,
                diffusion_kernel: DiffusionKernel::Mean3x3,
                max_brightness: 20.0,
                species_configs: vec![SpeciesConfig {
                    name: "default".to_string(),
                    count: 50_000,
                    sensor_angle: 15.0,
                    rotation_angle: 30.0,
                    ..Default::default()
                }],
                ..Self::default()
            },
            // Wide searching tentacles (config.rs:401-417)
            Preset::Exploratory => Self {
                sensor_angle: 45.0,
                sensor_distance: 15.0,
                rotation_angle: 60.0,
                decay_factor: 0.96,
                deposit_amount: 3.0,
                diffusion_kernel: DiffusionKernel::Mean3x3,
                max_brightness: 12.0,
                species_configs: vec![SpeciesConfig {
                    name: "default".to_string(),
                    count: 30_000,
                    sensor_angle: 45.0,
                    rotation_angle: 60.0,
                    deposit_amount: 3.0,
                    ..Default::default()
                }],
                ..Self::default()
            },
            // Long branching arms (config.rs:418-436)
            Preset::Tendrils => Self {
                sensor_angle: 30.0,
                sensor_distance: 12.0,
                rotation_angle: 45.0,
                step_size: 2.0,
                decay_factor: 0.90,
                deposit_amount: 4.0,
                diffusion_kernel: DiffusionKernel::Mean3x3,
                max_brightness: 16.0,
                species_configs: vec![SpeciesConfig {
                    name: "default".to_string(),
                    count: 40_000,
                    sensor_angle: 30.0,
                    rotation_angle: 45.0,
                    step_size: 2.0,
                    deposit_amount: 4.0,
                    ..Default::default()
                }],
                ..Self::default()
            },
            // Balanced natural-looking growth (config.rs:437-445) — no species override
            Preset::Organic => Self {
                sensor_angle: 22.5,
                sensor_distance: 9.0,
                rotation_angle: 45.0,
                step_size: 1.0,
                decay_factor: 0.85,
                diffusion_kernel: DiffusionKernel::Gaussian,
                max_brightness: 20.0,
                window_frame: WindowFrame::Accented,
                ..Self::default()
            },
            // Fast flame-like patterns (config.rs:446-462)
            Preset::Fire => Self {
                sensor_angle: 15.0,
                rotation_angle: 30.0,
                step_size: 1.5,
                decay_factor: 0.85,
                diffusion_kernel: DiffusionKernel::Mean3x3,
                max_brightness: 5.0,
                window_frame: WindowFrame::Accented,
                species_configs: vec![SpeciesConfig {
                    name: "default".to_string(),
                    count: 100_000,
                    sensor_angle: 15.0,
                    rotation_angle: 30.0,
                    step_size: 1.5,
                    color: RgbColor::from_hex(0xff4500),
                    ..Default::default()
                }],
                ..Self::default()
            },
            // Flowing water patterns with wind; boundary wraps to keep flow continuous (config.rs:463-478)
            Preset::River => Self {
                sensor_angle: 25.0,
                step_size: 1.2,
                decay_factor: 0.90,
                diffusion_kernel: DiffusionKernel::Mean3x3,
                max_brightness: 18.0,
                boundary_mode: BoundaryMode::Wrap,
                wind: Some(Wind::new(0.3, 0.0)),
                species_configs: vec![SpeciesConfig {
                    name: "default".to_string(),
                    count: 45_000,
                    sensor_angle: 25.0,
                    step_size: 1.2,
                    color: RgbColor::from_hex(0x1e90ff),
                    ..Default::default()
                }],
                ..Self::default()
            },
            // Petri dish: starts center, slow growth, persistent trails (config.rs:479-503)
            Preset::PetriDish => Self {
                sensor_angle: 45.0,
                rotation_angle: 20.0,
                step_size: 0.05,
                decay_factor: 0.999,
                deposit_amount: 0.2,
                max_brightness: 50.0,
                preferred_init_mode: Some(InitMode::Petri),
                background_color: Some("000000".to_string()),
                obstacles: vec![Obstacle::Circle {
                    x: 200.0,
                    y: 100.0,
                    radius: 90.0,
                }],
                species_configs: vec![SpeciesConfig {
                    name: "mold".to_string(),
                    count: 20_000,
                    sensor_angle: 45.0,
                    rotation_angle: 20.0,
                    step_size: 0.05,
                    deposit_amount: 0.2,
                    color: RgbColor::from_hex(0xd4ff00),
                    ..Default::default()
                }],
                ..Self::default()
            },
            // Spinning vortex (config.rs:504-522)
            Preset::Vortex => Self {
                sensor_angle: 25.2,
                sensor_distance: 3.9,
                rotation_angle: 46.4,
                step_size: 1.92,
                decay_factor: 0.96,
                deposit_amount: 4.3,
                diffusion_kernel: DiffusionKernel::Mean3x3,
                species_configs: vec![SpeciesConfig {
                    name: "default".to_string(),
                    count: 32_000,
                    sensor_angle: 25.2,
                    rotation_angle: 46.4,
                    step_size: 1.92,
                    deposit_amount: 4.3,
                    color: RgbColor::from_hex(0x9370db),
                    ..Default::default()
                }],
                ..Self::default()
            },
            // Fast dendritic branching (config.rs:523-542)
            Preset::Lightning => Self {
                sensor_angle: 31.9,
                sensor_distance: 23.2,
                rotation_angle: 39.3,
                step_size: 2.48,
                decay_factor: 0.82,
                deposit_amount: 20.0,
                diffusion_kernel: DiffusionKernel::Mean3x3,
                max_brightness: 40.0,
                species_configs: vec![SpeciesConfig {
                    name: "default".to_string(),
                    count: 7_000,
                    sensor_angle: 31.9,
                    rotation_angle: 39.3,
                    step_size: 2.48,
                    deposit_amount: 20.0,
                    color: RgbColor::from_hex(0x00ffff),
                    ..Default::default()
                }],
                ..Self::default()
            },
            // Edge-of-chaos sensitive patterns (config.rs:543-562)
            Preset::ChaosEdge => Self {
                sensor_angle: 5.0,
                sensor_distance: 26.4,
                rotation_angle: 56.2,
                step_size: 0.58,
                decay_factor: 0.99,
                deposit_amount: 15.8,
                diffusion_kernel: DiffusionKernel::Mean3x3,
                max_brightness: 25.0,
                species_configs: vec![SpeciesConfig {
                    name: "default".to_string(),
                    count: 52_000,
                    sensor_angle: 5.0,
                    rotation_angle: 56.2,
                    step_size: 0.58,
                    deposit_amount: 15.8,
                    color: RgbColor::from_hex(0xff6347),
                    ..Default::default()
                }],
                ..Self::default()
            },
            // Aggregating blob clusters (config.rs:563-582)
            Preset::Blob => Self {
                sensor_angle: 72.1,
                sensor_distance: 2.1,
                rotation_angle: 90.0,
                step_size: 0.92,
                decay_factor: 0.50,
                deposit_amount: 9.3,
                diffusion_kernel: DiffusionKernel::Mean3x3,
                max_brightness: 25.0,
                species_configs: vec![SpeciesConfig {
                    name: "default".to_string(),
                    count: 21_000,
                    sensor_angle: 72.1,
                    rotation_angle: 90.0,
                    step_size: 0.92,
                    deposit_amount: 9.3,
                    color: RgbColor::from_hex(0x32cd32),
                    ..Default::default()
                }],
                ..Self::default()
            },
            // Slime-mold surface tension with trail-based flow modulation (config.rs:583-616)
            Preset::Slime => Self {
                sensor_angle: 60.0,
                sensor_distance: 30.0,
                rotation_angle: 15.0,
                step_size: 1.0,
                decay_factor: 0.90,
                deposit_amount: 4.0,
                diffusion_kernel: DiffusionKernel::Mean3x3,
                max_brightness: 18.0,
                window_frame: WindowFrame::Accented,
                species_configs: vec![SpeciesConfig {
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
                }],
                ..Self::default()
            },
            // Creeping vine tendrils with trail-modulated cohesion (config.rs:617-650)
            Preset::Vines => Self {
                sensor_angle: 45.0,
                sensor_distance: 25.0,
                rotation_angle: 60.0,
                step_size: 1.5,
                decay_factor: 0.88,
                deposit_amount: 5.0,
                diffusion_kernel: DiffusionKernel::Mean3x3,
                max_brightness: 20.0,
                window_frame: WindowFrame::None,
                species_configs: vec![SpeciesConfig {
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
                }],
                ..Self::default()
            },
            // Drifting smoke columns; boundary wraps so plumes re-enter (config.rs:651-685)
            Preset::Smoke => Self {
                sensor_angle: 35.0,
                sensor_distance: 12.0,
                rotation_angle: 30.0,
                step_size: 1.0,
                decay_factor: 0.94,
                deposit_amount: 3.5,
                diffusion_kernel: DiffusionKernel::Mean3x3,
                max_brightness: 16.0,
                window_frame: WindowFrame::Accented,
                boundary_mode: BoundaryMode::Wrap,
                species_configs: vec![SpeciesConfig {
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
                }],
                ..Self::default()
            },
            // Enhanced vortex with trail modulation (config.rs:686-719)
            Preset::Vortex36 => Self {
                sensor_angle: 25.2,
                sensor_distance: 7.0,
                rotation_angle: 46.4,
                step_size: 1.92,
                decay_factor: 0.96,
                deposit_amount: 4.3,
                diffusion_kernel: DiffusionKernel::Mean3x3,
                species_configs: vec![SpeciesConfig {
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
                }],
                ..Self::default()
            },
            // Dynamic tendrils with trail-based sensor modulation (config.rs:720-743)
            Preset::DynamicTendrils => Self {
                decay_factor: 0.92,
                species_configs: vec![SpeciesConfig {
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
                }],
                ..Self::default()
            },
            // Bleuje-style front-lit veins; NOTE: afterglow=0.3 intentionally dropped
            // (moves to RenderArtDefaults). Sim levers: deposit_curve, deposit_scale,
            // decay_gamma (config.rs:744-762)
            Preset::Mold => Self {
                sensor_angle: 15.0,
                rotation_angle: 30.0,
                decay_factor: 0.85,
                diffusion_kernel: DiffusionKernel::Mean3x3,
                max_brightness: 20.0,
                window_frame: WindowFrame::Accented,
                deposit_curve: DepositCurve::Sqrt,
                deposit_scale: 1.5,
                decay_gamma: 0.8,
                species_configs: vec![SpeciesConfig {
                    name: "default".to_string(),
                    count: 50_000,
                    sensor_angle: 15.0,
                    rotation_angle: 30.0,
                    ..Default::default()
                }],
                ..Self::default()
            },
            // Directional filament linework; NOTE: afterglow=0.2 intentionally dropped
            // (moves to RenderArtDefaults) (config.rs:763-784)
            Preset::Etching => Self {
                sensor_angle: 30.0,
                sensor_distance: 12.0,
                rotation_angle: 45.0,
                step_size: 2.0,
                decay_factor: 0.90,
                deposit_amount: 4.0,
                diffusion_kernel: DiffusionKernel::Mean3x3,
                deposit_curve: DepositCurve::Sqrt,
                max_brightness: 16.0,
                species_configs: vec![SpeciesConfig {
                    name: "default".to_string(),
                    count: 40_000,
                    sensor_angle: 30.0,
                    rotation_angle: 45.0,
                    step_size: 2.0,
                    deposit_amount: 4.0,
                    ..Default::default()
                }],
                ..Self::default()
            },
            // Smooth flowing; NOTE: afterglow=0.1 intentionally dropped (config.rs:785-805)
            Preset::Drift => Self {
                sensor_angle: 30.0,
                sensor_distance: 14.0,
                rotation_angle: 35.0,
                step_size: 1.2,
                decay_factor: 0.93,
                deposit_amount: 3.0,
                diffusion_kernel: DiffusionKernel::Mean3x3,
                max_brightness: 16.0,
                species_configs: vec![SpeciesConfig {
                    name: "default".to_string(),
                    count: 35_000,
                    sensor_angle: 30.0,
                    rotation_angle: 35.0,
                    step_size: 1.2,
                    deposit_amount: 3.0,
                    ..Default::default()
                }],
                ..Self::default()
            },
            // Sparse star-map scatter (config.rs:806-826)
            Preset::Constellation => Self {
                sensor_angle: 45.0,
                sensor_distance: 12.0,
                rotation_angle: 25.0,
                step_size: 0.8,
                decay_factor: 0.96,
                deposit_amount: 4.0,
                diffusion_kernel: DiffusionKernel::Mean3x3,
                max_brightness: 30.0,
                preferred_init_mode: Some(InitMode::Random),
                species_configs: vec![SpeciesConfig {
                    name: "default".to_string(),
                    count: 12_000,
                    sensor_angle: 45.0,
                    rotation_angle: 25.0,
                    step_size: 0.8,
                    deposit_amount: 4.0,
                    ..Default::default()
                }],
                ..Self::default()
            },
            // Dense smooth body for posterized bands (config.rs:827-847)
            Preset::Mosaic => Self {
                sensor_angle: 30.0,
                sensor_distance: 15.0,
                rotation_angle: 40.0,
                step_size: 1.0,
                decay_factor: 0.94,
                deposit_amount: 3.5,
                diffusion_kernel: DiffusionKernel::Gaussian,
                diffusion_sigma: 1.5,
                max_brightness: 16.0,
                species_configs: vec![SpeciesConfig {
                    name: "default".to_string(),
                    count: 40_000,
                    sensor_angle: 30.0,
                    rotation_angle: 40.0,
                    step_size: 1.0,
                    deposit_amount: 3.5,
                    ..Default::default()
                }],
                ..Self::default()
            },
            // Heavy Gaussian smear for veined stone (config.rs:848-869)
            Preset::Marble => Self {
                sensor_angle: 50.0,
                sensor_distance: 18.0,
                rotation_angle: 45.0,
                step_size: 0.7,
                decay_factor: 0.95,
                deposit_amount: 3.0,
                diffusion_kernel: DiffusionKernel::Gaussian,
                diffusion_sigma: 2.5,
                diffuse_weight: 0.8,
                max_brightness: 18.0,
                species_configs: vec![SpeciesConfig {
                    name: "default".to_string(),
                    count: 30_000,
                    sensor_angle: 50.0,
                    rotation_angle: 45.0,
                    step_size: 0.7,
                    deposit_amount: 3.0,
                    ..Default::default()
                }],
                ..Self::default()
            },
            // Balanced colorful base (config.rs:870-889)
            Preset::Prism => Self {
                sensor_angle: 25.0,
                sensor_distance: 12.0,
                rotation_angle: 40.0,
                step_size: 1.0,
                decay_factor: 0.90,
                deposit_amount: 3.0,
                diffusion_kernel: DiffusionKernel::Mean3x3,
                max_brightness: 16.0,
                species_configs: vec![SpeciesConfig {
                    name: "default".to_string(),
                    count: 45_000,
                    sensor_angle: 25.0,
                    rotation_angle: 40.0,
                    step_size: 1.0,
                    deposit_amount: 3.0,
                    ..Default::default()
                }],
                ..Self::default()
            },
            // Soft parchment density (config.rs:890-911)
            Preset::Vellum => Self {
                sensor_angle: 28.0,
                sensor_distance: 12.0,
                rotation_angle: 40.0,
                step_size: 1.0,
                decay_factor: 0.91,
                deposit_amount: 4.0,
                diffusion_kernel: DiffusionKernel::Gaussian,
                diffusion_sigma: 1.2,
                deposit_curve: DepositCurve::Log,
                max_brightness: 18.0,
                species_configs: vec![SpeciesConfig {
                    name: "default".to_string(),
                    count: 35_000,
                    sensor_angle: 28.0,
                    rotation_angle: 40.0,
                    step_size: 1.0,
                    deposit_amount: 4.0,
                    ..Default::default()
                }],
                ..Self::default()
            },
            // Aggressive molten; NOTE: afterglow=0.3 intentionally dropped (config.rs:912-932)
            Preset::Forge => Self {
                sensor_angle: 15.0,
                sensor_distance: 9.0,
                rotation_angle: 30.0,
                step_size: 1.5,
                decay_factor: 0.85,
                deposit_amount: 5.0,
                diffusion_kernel: DiffusionKernel::Mean3x3,
                max_brightness: 20.0,
                species_configs: vec![SpeciesConfig {
                    name: "default".to_string(),
                    count: 80_000,
                    sensor_angle: 15.0,
                    rotation_angle: 30.0,
                    step_size: 1.5,
                    deposit_amount: 5.0,
                    ..Default::default()
                }],
                ..Self::default()
            },
            // Long faint tails: low decay_gamma + Pow deposit + high decay_factor (config.rs:933-955)
            Preset::Wane => Self {
                sensor_angle: 25.0,
                sensor_distance: 12.0,
                rotation_angle: 30.0,
                step_size: 0.9,
                decay_factor: 0.97,
                deposit_amount: 3.0,
                diffusion_kernel: DiffusionKernel::Mean3x3,
                decay_gamma: 0.6,
                deposit_curve: DepositCurve::Pow,
                deposit_gamma: 0.6,
                max_brightness: 16.0,
                species_configs: vec![SpeciesConfig {
                    name: "default".to_string(),
                    count: 30_000,
                    sensor_angle: 25.0,
                    rotation_angle: 30.0,
                    step_size: 0.9,
                    deposit_amount: 3.0,
                    ..Default::default()
                }],
                ..Self::default()
            },
            // Fine sparse threads; NOTE: afterglow=0.2 intentionally dropped (config.rs:956-976)
            Preset::Gossamer => Self {
                sensor_angle: 35.0,
                sensor_distance: 10.0,
                rotation_angle: 40.0,
                step_size: 0.7,
                decay_factor: 0.96,
                deposit_amount: 2.5,
                diffusion_kernel: DiffusionKernel::Mean3x3,
                max_brightness: 14.0,
                species_configs: vec![SpeciesConfig {
                    name: "default".to_string(),
                    count: 25_000,
                    sensor_angle: 35.0,
                    rotation_angle: 40.0,
                    step_size: 0.7,
                    deposit_amount: 2.5,
                    ..Default::default()
                }],
                ..Self::default()
            },
            // Typographic linework base (config.rs:977-996)
            Preset::Codex => Self {
                sensor_angle: 30.0,
                sensor_distance: 14.0,
                rotation_angle: 45.0,
                step_size: 1.5,
                decay_factor: 0.90,
                deposit_amount: 4.0,
                diffusion_kernel: DiffusionKernel::Mean3x3,
                max_brightness: 16.0,
                species_configs: vec![SpeciesConfig {
                    name: "default".to_string(),
                    count: 40_000,
                    sensor_angle: 30.0,
                    rotation_angle: 45.0,
                    step_size: 1.5,
                    deposit_amount: 4.0,
                    ..Default::default()
                }],
                ..Self::default()
            },
            // Flowing water body (config.rs:997-1016)
            Preset::Tide => Self {
                sensor_angle: 25.0,
                sensor_distance: 12.0,
                rotation_angle: 30.0,
                step_size: 1.2,
                decay_factor: 0.92,
                deposit_amount: 3.0,
                diffusion_kernel: DiffusionKernel::Mean3x3,
                max_brightness: 16.0,
                species_configs: vec![SpeciesConfig {
                    name: "default".to_string(),
                    count: 40_000,
                    sensor_angle: 25.0,
                    rotation_angle: 30.0,
                    step_size: 1.2,
                    deposit_amount: 3.0,
                    ..Default::default()
                }],
                ..Self::default()
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_declares_bounce() {
        assert_eq!(
            PresetSimDefaults::default().boundary_mode,
            BoundaryMode::Bounce
        );
    }

    #[test]
    fn default_matches_simconfig_default() {
        use crate::simulation::config::SimConfig;
        let mut from_default = SimConfig::default();
        PresetSimDefaults::default().apply_to(&mut from_default);
        let plain = SimConfig::default();
        // apply_to of the default spec must be a no-op on a default config.
        assert_eq!(from_default.sensor_angle, plain.sensor_angle);
        assert_eq!(from_default.diffusion_kernel, plain.diffusion_kernel);
        assert_eq!(from_default.diffusion_sigma, plain.diffusion_sigma);
        assert_eq!(from_default.boundary_mode, plain.boundary_mode);
        assert_eq!(from_default.species_configs, plain.species_configs);
        assert_eq!(from_default.decay_gamma, plain.decay_gamma);
    }

    #[test]
    fn network_arm_ported() {
        let s = PresetSimDefaults::from(Preset::Network);
        assert_eq!(s.sensor_angle, 15.0);
        assert_eq!(s.rotation_angle, 30.0);
        assert_eq!(s.decay_factor, 0.85);
        assert_eq!(s.diffusion_kernel, DiffusionKernel::Mean3x3);
        assert_eq!(s.max_brightness, 20.0);
        assert_eq!(s.species_configs.len(), 1);
        assert_eq!(s.species_configs[0].count, 50_000);
    }

    #[test]
    fn river_arm_carries_wind() {
        use crate::simulation::config::Wind;
        let s = PresetSimDefaults::from(Preset::River);
        assert_eq!(s.wind, Some(Wind::new(0.3, 0.0)));
    }

    #[test]
    fn petridish_arm_carries_obstacle_and_bg() {
        use crate::simulation::config::InitMode;
        let s = PresetSimDefaults::from(Preset::PetriDish);
        assert_eq!(s.obstacles.len(), 1);
        assert_eq!(s.background_color.as_deref(), Some("000000"));
        assert_eq!(s.preferred_init_mode, Some(InitMode::Petri));
    }

    #[test]
    fn mold_arm_keeps_sim_levers_drops_afterglow() {
        let s = PresetSimDefaults::from(Preset::Mold);
        assert_eq!(s.decay_gamma, 0.8);
        assert_eq!(s.deposit_curve, DepositCurve::Sqrt);
        assert_eq!(s.deposit_scale, 1.5);
        // afterglow is NOT a field of PresetSimDefaults (moved to render layer).
    }

    #[test]
    fn apply_to_lands_the_five_new_levers() {
        use crate::simulation::config::{Attractor, RespawnConfig, SamplingMode, SimConfig};
        let spec = PresetSimDefaults {
            deposit_cap: 5.0,
            attractors: vec![Attractor {
                x: 10.0,
                y: 20.0,
                strength: 1.5,
            }],
            separate_species_trails: true,
            sampling_mode: SamplingMode::Bilinear,
            respawn_config: RespawnConfig {
                interval: 42,
                ..RespawnConfig::default()
            },
            ..PresetSimDefaults::default()
        };
        let mut cfg = SimConfig::default();
        spec.apply_to(&mut cfg);
        assert_eq!(cfg.deposit_cap, 5.0);
        assert_eq!(cfg.attractors.len(), 1);
        assert_eq!(cfg.attractors[0].strength, 1.5);
        assert!(cfg.separate_species_trails);
        assert_eq!(cfg.sampling_mode, SamplingMode::Bilinear);
        assert_eq!(cfg.respawn_config.interval, 42);
    }

    #[test]
    fn default_new_levers_match_simconfig_default() {
        use crate::simulation::config::SimConfig;
        let plain = SimConfig::default();
        let d = PresetSimDefaults::default();
        assert_eq!(d.deposit_cap, plain.deposit_cap);
        assert_eq!(d.attractors, plain.attractors);
        assert_eq!(d.separate_species_trails, plain.separate_species_trails);
        assert_eq!(d.sampling_mode, plain.sampling_mode);
        assert_eq!(d.respawn_config, plain.respawn_config);
    }

    #[test]
    fn window_frame_defaults_to_frame_and_is_settable() {
        use crate::simulation::config::{SimConfig, WindowFrame};
        assert_eq!(
            PresetSimDefaults::default().window_frame,
            WindowFrame::Frame
        );
        let spec = PresetSimDefaults {
            window_frame: WindowFrame::Accented,
            ..PresetSimDefaults::default()
        };
        let mut cfg = SimConfig::default();
        spec.apply_to(&mut cfg);
        assert_eq!(cfg.window_frame, WindowFrame::Accented);
    }

    #[test]
    fn plain_preset_keeps_default_frame() {
        use crate::simulation::config::WindowFrame;
        assert_eq!(
            PresetSimDefaults::from(crate::simulation::config::Preset::Network).window_frame,
            WindowFrame::Frame
        );
    }

    #[test]
    fn only_river_and_smoke_declare_boundary_wrap() {
        for spec in crate::simulation::config::PRESETS {
            let expected = match spec.preset {
                Preset::River | Preset::Smoke => BoundaryMode::Wrap,
                _ => BoundaryMode::Bounce,
            };
            assert_eq!(
                PresetSimDefaults::from(spec.preset).boundary_mode,
                expected,
                "{} boundary",
                spec.name
            );
        }
    }
}
