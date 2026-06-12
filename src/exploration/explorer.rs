//! Parameter space explorer for discovering optimal presets.
//!
//! Runs headless simulations with various parameter combinations and
//! evaluates them using pattern metrics to find parameters that produce
//! specific emergent behaviors.

use crate::exploration::metrics::PatternMetrics;
use crate::render::palette::RgbColor;
use crate::simulation::config::{
    DiffusionKernel, InitMode, SimConfig, SpeciesConfig, TerrainType, Wind,
};
use crate::simulation::Simulation;
use rand::prelude::*;
use rand_xoshiro::Xoshiro256PlusPlus;

/// Parameter set explored by the optimizer.
#[derive(Debug, Clone, Copy)]
pub struct ExplorationParams {
    // Core agent parameters
    /// Sensor angle in degrees (5-90).
    pub sensor_angle: f32,
    /// Sensor distance in pixels (1-50).
    pub sensor_distance: f32,
    /// Rotation angle in degrees (5-90).
    pub rotation_angle: f32,
    /// Agent step size (0.5-5.0).
    pub step_size: f32,
    /// Trail decay factor (0.5-0.99).
    pub decay_factor: f32,
    /// Amount of pheromone deposited (1-20).
    pub deposit_amount: f32,
    /// Total agent population.
    pub population: usize,

    // Extended parameters
    /// Diffusion algorithm (Mean3x3 or Gaussian).
    pub diffusion_kernel: DiffusionKernel,
    /// Wind horizontal component (-1.0 to 1.0), None if disabled.
    pub wind_dx: Option<f32>,
    /// Wind vertical component (-1.0 to 1.0), None if disabled.
    pub wind_dy: Option<f32>,
    /// Terrain type for steering bias.
    pub terrain: TerrainType,
    /// Terrain effect strength (0.1-5.0).
    pub terrain_strength: f32,
    /// Agent initialization mode.
    pub init_mode: InitMode,
}

impl ExplorationParams {
    /// Create random parameters within valid ranges.
    pub fn random(rng: &mut impl Rng) -> Self {
        // 30% chance of wind
        let (wind_dx, wind_dy) = if rng.gen_bool(0.3) {
            (
                Some(rng.gen_range(-0.5..0.5)),
                Some(rng.gen_range(-0.5..0.5)),
            )
        } else {
            (None, None)
        };

        // Randomly select terrain type (40% none, 20% each other)
        let terrain = match rng.gen_range(0..5) {
            0 | 1 => TerrainType::None,
            2 => TerrainType::Smooth,
            3 => TerrainType::Turbulent,
            _ => TerrainType::Mixed,
        };

        // Randomly select init mode (60% Random, 40% others)
        let init_mode = match rng.gen_range(0..10) {
            0..=5 => InitMode::Random,
            6 => InitMode::CentralBurst,
            7 => InitMode::Circle,
            8 => InitMode::Gradient,
            _ => InitMode::WaveFront,
        };

        Self {
            sensor_angle: rng.gen_range(5.0..90.0),
            sensor_distance: rng.gen_range(1.0..50.0),
            rotation_angle: rng.gen_range(5.0..90.0),
            step_size: rng.gen_range(0.5..5.0),
            decay_factor: rng.gen_range(0.5..0.99),
            deposit_amount: rng.gen_range(1.0..20.0),
            population: rng.gen_range(5000..100000),
            diffusion_kernel: if rng.gen_bool(0.5) {
                DiffusionKernel::Mean3x3
            } else {
                DiffusionKernel::Gaussian
            },
            wind_dx,
            wind_dy,
            terrain,
            terrain_strength: rng.gen_range(0.5..3.0),
            init_mode,
        }
    }

    /// Create parameters biased toward a specific behavior.
    pub fn random_biased(rng: &mut impl Rng, behavior: PresetBehavior) -> Self {
        match behavior {
            PresetBehavior::Vortex => Self {
                // rotation_angle > sensor_angle for spiraling
                sensor_angle: rng.gen_range(5.0..30.0),
                sensor_distance: rng.gen_range(8.0..20.0),
                rotation_angle: rng.gen_range(40.0..90.0),
                step_size: rng.gen_range(0.8..2.0),
                decay_factor: rng.gen_range(0.8..0.95),
                deposit_amount: rng.gen_range(3.0..8.0),
                population: rng.gen_range(30000..80000),
                // Gaussian diffusion for smoother vortex patterns
                diffusion_kernel: if rng.gen_bool(0.7) {
                    DiffusionKernel::Gaussian
                } else {
                    DiffusionKernel::Mean3x3
                },
                // Light wind can enhance swirling
                wind_dx: if rng.gen_bool(0.4) {
                    Some(rng.gen_range(-0.3..0.3))
                } else {
                    None
                },
                wind_dy: if rng.gen_bool(0.4) {
                    Some(rng.gen_range(-0.3..0.3))
                } else {
                    None
                },
                terrain: TerrainType::None,
                terrain_strength: 1.0,
                init_mode: if rng.gen_bool(0.6) {
                    InitMode::Random
                } else {
                    InitMode::Circle
                },
            },
            PresetBehavior::Lightning => Self {
                // Fast, sparse, high contrast branching
                sensor_angle: rng.gen_range(5.0..20.0),
                sensor_distance: rng.gen_range(10.0..30.0),
                rotation_angle: rng.gen_range(10.0..30.0),
                step_size: rng.gen_range(2.0..5.0),
                decay_factor: rng.gen_range(0.5..0.75),
                deposit_amount: rng.gen_range(10.0..20.0),
                population: rng.gen_range(5000..20000),
                // Mean3x3 for sharper branching
                diffusion_kernel: DiffusionKernel::Mean3x3,
                // No wind for lightning (let it spread naturally)
                wind_dx: None,
                wind_dy: None,
                terrain: TerrainType::None,
                terrain_strength: 1.0,
                // CentralBurst creates explosive lightning effect
                init_mode: if rng.gen_bool(0.5) {
                    InitMode::CentralBurst
                } else {
                    InitMode::Random
                },
            },
            PresetBehavior::Crystal => Self {
                // Slow, stable, persistent structures
                sensor_angle: rng.gen_range(20.0..50.0),
                sensor_distance: rng.gen_range(15.0..40.0),
                rotation_angle: rng.gen_range(10.0..40.0),
                step_size: rng.gen_range(0.5..1.0),
                decay_factor: rng.gen_range(0.95..0.99),
                deposit_amount: rng.gen_range(2.0..6.0),
                population: rng.gen_range(15000..40000),
                // Gaussian for smooth crystal facets
                diffusion_kernel: DiffusionKernel::Gaussian,
                wind_dx: None,
                wind_dy: None,
                // Smooth terrain can create interesting crystal growth patterns
                terrain: if rng.gen_bool(0.3) {
                    TerrainType::Smooth
                } else {
                    TerrainType::None
                },
                terrain_strength: rng.gen_range(0.5..1.5),
                init_mode: InitMode::Random,
            },
            PresetBehavior::Blob => Self {
                // Short-sighted, sharp turns, fast decay for clustering
                sensor_angle: rng.gen_range(30.0..70.0),
                sensor_distance: rng.gen_range(1.0..8.0),
                rotation_angle: rng.gen_range(50.0..90.0),
                step_size: rng.gen_range(0.5..1.5),
                decay_factor: rng.gen_range(0.5..0.7),
                deposit_amount: rng.gen_range(5.0..15.0),
                population: rng.gen_range(20000..60000),
                // Mean3x3 for sharper blob edges
                diffusion_kernel: DiffusionKernel::Mean3x3,
                wind_dx: None,
                wind_dy: None,
                // Turbulent terrain breaks up patterns into blobs
                terrain: if rng.gen_bool(0.5) {
                    TerrainType::Turbulent
                } else {
                    TerrainType::None
                },
                terrain_strength: rng.gen_range(1.0..3.0),
                // RandomClusters naturally creates blobby patterns
                init_mode: if rng.gen_bool(0.4) {
                    InitMode::RandomClusters
                } else {
                    InitMode::Random
                },
            },
            PresetBehavior::Worm => Self {
                // Long-sighted, low population for snaking trails
                sensor_angle: rng.gen_range(10.0..25.0),
                sensor_distance: rng.gen_range(20.0..50.0),
                rotation_angle: rng.gen_range(20.0..45.0),
                step_size: rng.gen_range(1.0..2.5),
                decay_factor: rng.gen_range(0.88..0.96),
                deposit_amount: rng.gen_range(4.0..10.0),
                population: rng.gen_range(3000..15000),
                diffusion_kernel: DiffusionKernel::Mean3x3,
                // Light directional wind creates flowing worms
                wind_dx: if rng.gen_bool(0.5) {
                    Some(rng.gen_range(0.1..0.4))
                } else {
                    None
                },
                wind_dy: if rng.gen_bool(0.3) {
                    Some(rng.gen_range(-0.2..0.2))
                } else {
                    None
                },
                terrain: TerrainType::None,
                terrain_strength: 1.0,
                // Gradient or WaveFront creates aligned worm trails
                init_mode: match rng.gen_range(0..3) {
                    0 => InitMode::Gradient,
                    1 => InitMode::WaveFront,
                    _ => InitMode::Random,
                },
            },
            PresetBehavior::ChaosEdge => Self {
                // sensor_angle ≈ rotation_angle for edge-of-chaos
                sensor_angle: rng.gen_range(15.0..40.0),
                sensor_distance: rng.gen_range(5.0..20.0),
                rotation_angle: rng.gen_range(15.0..40.0),
                step_size: rng.gen_range(0.8..1.5),
                decay_factor: rng.gen_range(0.8..0.92),
                deposit_amount: rng.gen_range(3.0..8.0),
                population: rng.gen_range(30000..70000),
                diffusion_kernel: if rng.gen_bool(0.5) {
                    DiffusionKernel::Mean3x3
                } else {
                    DiffusionKernel::Gaussian
                },
                wind_dx: None,
                wind_dy: None,
                // Mixed terrain adds to chaotic dynamics
                terrain: if rng.gen_bool(0.4) {
                    TerrainType::Mixed
                } else {
                    TerrainType::None
                },
                terrain_strength: rng.gen_range(0.5..2.0),
                init_mode: InitMode::Random,
            },
        }
    }

    /// Mutate parameters slightly for local search.
    pub fn mutate(&self, rng: &mut impl Rng, mutation_strength: f32) -> Self {
        fn mutate_f32(v: f32, min: f32, max: f32, rng: &mut impl Rng, strength: f32) -> f32 {
            let delta = (max - min) * strength * rng.gen_range(-1.0..1.0);
            (v + delta).clamp(min, max)
        }

        // Occasionally flip diffusion kernel (probability scales with mutation strength, capped at 20%)
        let diffusion_kernel = if rng.gen_bool((mutation_strength * 0.5).min(0.2) as f64) {
            match self.diffusion_kernel {
                DiffusionKernel::Mean3x3 => DiffusionKernel::Gaussian,
                DiffusionKernel::Gaussian => DiffusionKernel::Mean3x3,
            }
        } else {
            self.diffusion_kernel
        };

        // Mutate wind (can enable/disable or adjust)
        let (wind_dx, wind_dy) = if rng.gen_bool((mutation_strength * 0.3).min(0.15) as f64) {
            // Toggle wind on/off or change values
            if self.wind_dx.is_some() && rng.gen_bool(0.3) {
                (None, None) // Disable wind
            } else {
                (
                    Some(mutate_f32(
                        self.wind_dx.unwrap_or(0.0),
                        -0.8,
                        0.8,
                        rng,
                        mutation_strength,
                    )),
                    Some(mutate_f32(
                        self.wind_dy.unwrap_or(0.0),
                        -0.8,
                        0.8,
                        rng,
                        mutation_strength,
                    )),
                )
            }
        } else {
            // Keep wind as is, but mutate values if present
            (
                self.wind_dx
                    .map(|v| mutate_f32(v, -0.8, 0.8, rng, mutation_strength)),
                self.wind_dy
                    .map(|v| mutate_f32(v, -0.8, 0.8, rng, mutation_strength)),
            )
        };

        // Occasionally change terrain type (probability scales with mutation strength, capped at 15%)
        let terrain = if rng.gen_bool((mutation_strength * 0.5).min(0.15) as f64) {
            match rng.gen_range(0..4) {
                0 => TerrainType::None,
                1 => TerrainType::Smooth,
                2 => TerrainType::Turbulent,
                _ => TerrainType::Mixed,
            }
        } else {
            self.terrain
        };

        // Occasionally change init mode (probability scales with mutation strength, capped at 10%)
        let init_mode = if rng.gen_bool((mutation_strength * 0.4).min(0.1) as f64) {
            match rng.gen_range(0..5) {
                0 => InitMode::Random,
                1 => InitMode::CentralBurst,
                2 => InitMode::Circle,
                3 => InitMode::Gradient,
                _ => InitMode::WaveFront,
            }
        } else {
            self.init_mode
        };

        Self {
            sensor_angle: mutate_f32(self.sensor_angle, 5.0, 90.0, rng, mutation_strength),
            sensor_distance: mutate_f32(self.sensor_distance, 1.0, 50.0, rng, mutation_strength),
            rotation_angle: mutate_f32(self.rotation_angle, 5.0, 90.0, rng, mutation_strength),
            step_size: mutate_f32(self.step_size, 0.5, 5.0, rng, mutation_strength),
            decay_factor: mutate_f32(self.decay_factor, 0.5, 0.99, rng, mutation_strength),
            deposit_amount: mutate_f32(self.deposit_amount, 1.0, 20.0, rng, mutation_strength),
            population: (self.population as f32
                * (1.0 + mutation_strength * rng.gen_range(-0.5..0.5)))
            .clamp(5000.0, 100000.0) as usize,
            diffusion_kernel,
            wind_dx,
            wind_dy,
            terrain,
            terrain_strength: mutate_f32(self.terrain_strength, 0.1, 5.0, rng, mutation_strength),
            init_mode,
        }
    }

    /// Convert to SimConfig for simulation.
    pub fn to_sim_config(&self) -> SimConfig {
        // Wind requires both components present and a non-negligible magnitude
        let wind = match (self.wind_dx, self.wind_dy) {
            (Some(dx), Some(dy)) if dx.abs() > 0.001 || dy.abs() > 0.001 => Some(Wind::new(dx, dy)),
            _ => None,
        };

        SimConfig {
            sensor_angle: self.sensor_angle,
            sensor_distance: self.sensor_distance,
            rotation_angle: self.rotation_angle,
            step_size: self.step_size,
            decay_factor: self.decay_factor,
            deposit_amount: self.deposit_amount,
            species_configs: vec![SpeciesConfig {
                name: "explorer".to_string(),
                count: self.population,
                sensor_angle: self.sensor_angle,
                rotation_angle: self.rotation_angle,
                step_size: self.step_size,
                deposit_amount: self.deposit_amount,
                color: RgbColor::from_hex(0xffffff),
                trail_modulation: None,
            }],
            diffusion_kernel: self.diffusion_kernel,
            wind,
            terrain: self.terrain,
            terrain_strength: self.terrain_strength,
            ..Default::default()
        }
    }

    /// Format as Rust code for preset definition.
    pub fn to_rust_code(&self, name: &str) -> String {
        let diffusion_str = match self.diffusion_kernel {
            DiffusionKernel::Mean3x3 => "DiffusionKernel::Mean3x3",
            DiffusionKernel::Gaussian => "DiffusionKernel::Gaussian",
        };

        let wind_str = match (self.wind_dx, self.wind_dy) {
            (Some(dx), Some(dy)) if dx.abs() > 0.001 || dy.abs() > 0.001 => {
                format!("Some(Wind::new({:.2}, {:.2}))", dx, dy)
            }
            _ => "None".to_string(),
        };

        let terrain_str = match self.terrain {
            TerrainType::None => "TerrainType::None",
            TerrainType::Smooth => "TerrainType::Smooth",
            TerrainType::Turbulent => "TerrainType::Turbulent",
            TerrainType::Mixed => "TerrainType::Mixed",
        };

        let init_str = match self.init_mode {
            InitMode::Random => "InitMode::Random",
            InitMode::CentralBurst => "InitMode::CentralBurst",
            InitMode::Circle => "InitMode::Circle",
            InitMode::Gradient => "InitMode::Gradient",
            InitMode::WaveFront => "InitMode::WaveFront",
            InitMode::Spiral => "InitMode::Spiral",
            InitMode::RandomClusters => "InitMode::RandomClusters",
            InitMode::Food => "InitMode::Food",
            InitMode::Petri => "InitMode::Petri",
        };

        format!(
            r#"Preset::{name} => Self {{
    sensor_angle: {:.1},
    sensor_distance: {:.1},
    rotation_angle: {:.1},
    step_size: {:.2},
    decay_factor: {:.2},
    deposit_amount: {:.1},
    diffusion_kernel: {},
    wind: {},
    terrain: {},
    terrain_strength: {:.2},
    // population: {}, init_mode: {}
    ...
}}"#,
            self.sensor_angle,
            self.sensor_distance,
            self.rotation_angle,
            self.step_size,
            self.decay_factor,
            self.deposit_amount,
            diffusion_str,
            wind_str,
            terrain_str,
            self.terrain_strength,
            self.population,
            init_str,
        )
    }
}

/// Target behaviors for optimization.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PresetBehavior {
    /// Swirling vortex patterns with high angular momentum.
    Vortex,
    /// Fast dendritic branching patterns like lightning.
    Lightning,
    /// Slow, stable geometric crystal growth patterns.
    Crystal,
    /// Aggregating blob clusters with high fragmentation.
    Blob,
    /// Long snaking worm-like trails.
    Worm,
    /// Edge-of-chaos sensitive patterns with high variance.
    ChaosEdge,
}

impl PresetBehavior {
    /// Score the given metrics for this behavior.
    pub fn score(&self, metrics: &PatternMetrics) -> f32 {
        match self {
            PresetBehavior::Vortex => metrics.vortex_score(),
            PresetBehavior::Lightning => metrics.lightning_score(),
            PresetBehavior::Crystal => metrics.crystal_score(),
            PresetBehavior::Blob => metrics.blob_score(),
            PresetBehavior::Worm => metrics.worm_score(),
            PresetBehavior::ChaosEdge => metrics.chaos_score(),
        }
    }

    /// All behavior types.
    pub fn all() -> &'static [PresetBehavior] {
        &[
            PresetBehavior::Vortex,
            PresetBehavior::Lightning,
            PresetBehavior::Crystal,
            PresetBehavior::Blob,
            PresetBehavior::Worm,
            PresetBehavior::ChaosEdge,
        ]
    }
}

/// Result of evaluating a parameter set.
#[derive(Debug, Clone)]
pub struct EvaluationResult {
    /// The parameters that were evaluated.
    pub params: ExplorationParams,
    /// Computed pattern metrics.
    pub metrics: PatternMetrics,
    /// Scores for each behavior type.
    pub scores: Vec<(PresetBehavior, f32)>,
}

/// Explorer configuration.
#[derive(Debug, Clone)]
pub struct ExplorerConfig {
    /// Grid width for simulation.
    pub width: usize,
    /// Grid height for simulation.
    pub height: usize,
    /// Number of frames to simulate before measuring.
    pub warmup_frames: usize,
    /// Number of frames to measure over.
    pub measurement_frames: usize,
    /// Random seed.
    pub seed: u64,
}

impl Default for ExplorerConfig {
    fn default() -> Self {
        Self {
            width: 200,
            height: 200,
            warmup_frames: 100,
            measurement_frames: 50,
            seed: 42,
        }
    }
}

/// Parameter space explorer.
pub struct Explorer {
    config: ExplorerConfig,
    rng: Xoshiro256PlusPlus,
}

impl Explorer {
    /// Create a new explorer.
    pub fn new(config: ExplorerConfig) -> Self {
        let rng = Xoshiro256PlusPlus::seed_from_u64(config.seed);
        Self { config, rng }
    }

    /// Evaluate a single parameter set.
    pub fn evaluate(&self, params: &ExplorationParams) -> EvaluationResult {
        let sim_config = params.to_sim_config();
        let mut sim = Simulation::new(
            self.config.width,
            self.config.height,
            sim_config,
            self.config.seed,
            params.init_mode,
            0, // No trail history needed for metrics
        );

        // Warmup
        for _ in 0..self.config.warmup_frames {
            sim.update(1.0);
        }

        // Measure over multiple frames
        let mut prev_trail: Option<Vec<f32>> = None;
        let mut accumulated_metrics = PatternMetrics::default();
        let mut sample_count = 0;

        for _ in 0..self.config.measurement_frames {
            sim.update(1.0);

            let trail = sim.trail_map().current().to_vec();
            let agents = sim.agents();

            let metrics = PatternMetrics::compute(
                &trail,
                self.config.width,
                self.config.height,
                agents,
                prev_trail.as_deref(),
            );

            // Accumulate metrics
            accumulated_metrics.angular_momentum += metrics.angular_momentum;
            accumulated_metrics.heading_variance += metrics.heading_variance;
            accumulated_metrics.trail_fragmentation += metrics.trail_fragmentation;
            accumulated_metrics.trail_elongation += metrics.trail_elongation;
            accumulated_metrics.spatial_entropy += metrics.spatial_entropy;
            accumulated_metrics.temporal_stability += metrics.temporal_stability;
            accumulated_metrics.density_variance += metrics.density_variance;
            accumulated_metrics.mean_intensity += metrics.mean_intensity;
            accumulated_metrics.coverage += metrics.coverage;
            accumulated_metrics.branching_factor += metrics.branching_factor;
            accumulated_metrics.flow_coherence += metrics.flow_coherence;
            accumulated_metrics.spatial_concentration += metrics.spatial_concentration;
            accumulated_metrics.path_continuity += metrics.path_continuity;

            sample_count += 1;
            prev_trail = Some(trail);
        }

        // Average metrics
        let n = sample_count as f32;
        let avg_metrics = PatternMetrics {
            angular_momentum: accumulated_metrics.angular_momentum / n,
            heading_variance: accumulated_metrics.heading_variance / n,
            trail_fragmentation: accumulated_metrics.trail_fragmentation / sample_count as u32,
            trail_elongation: accumulated_metrics.trail_elongation / n,
            spatial_entropy: accumulated_metrics.spatial_entropy / n,
            temporal_stability: accumulated_metrics.temporal_stability / n,
            density_variance: accumulated_metrics.density_variance / n,
            mean_intensity: accumulated_metrics.mean_intensity / n,
            coverage: accumulated_metrics.coverage / n,
            branching_factor: accumulated_metrics.branching_factor / n,
            flow_coherence: accumulated_metrics.flow_coherence / n,
            spatial_concentration: accumulated_metrics.spatial_concentration / n,
            path_continuity: accumulated_metrics.path_continuity / n,
        };

        // Compute scores for all behaviors
        let scores: Vec<(PresetBehavior, f32)> = PresetBehavior::all()
            .iter()
            .map(|&b| (b, b.score(&avg_metrics)))
            .collect();

        EvaluationResult {
            params: *params,
            metrics: avg_metrics,
            scores,
        }
    }

    /// Run random search for a specific behavior.
    pub fn random_search(
        &mut self,
        behavior: PresetBehavior,
        iterations: usize,
        use_bias: bool,
    ) -> Vec<EvaluationResult> {
        let mut results = Vec::with_capacity(iterations);

        for i in 0..iterations {
            let params = if use_bias {
                ExplorationParams::random_biased(&mut self.rng, behavior)
            } else {
                ExplorationParams::random(&mut self.rng)
            };

            let result = self.evaluate(&params);

            if i % 10 == 0 {
                let score = behavior.score(&result.metrics);
                eprintln!("Iteration {}/{}: score = {:.4}", i + 1, iterations, score);
            }

            results.push(result);
        }

        // Sort by target behavior score (descending)
        results.sort_by(|a, b| {
            let sa = behavior.score(&a.metrics);
            let sb = behavior.score(&b.metrics);
            sb.partial_cmp(&sa).unwrap_or(std::cmp::Ordering::Equal)
        });

        results
    }

    /// Run hill-climbing optimization for a specific behavior.
    pub fn hill_climb(
        &mut self,
        behavior: PresetBehavior,
        initial: Option<ExplorationParams>,
        iterations: usize,
        restarts: usize,
    ) -> EvaluationResult {
        let mut best_result: Option<EvaluationResult> = None;

        for restart in 0..restarts {
            // Initialize
            let mut current = initial
                .unwrap_or_else(|| ExplorationParams::random_biased(&mut self.rng, behavior));
            let mut current_result = self.evaluate(&current);
            let mut current_score = behavior.score(&current_result.metrics);

            let mut no_improvement = 0;
            let mut mutation_strength = 0.2f32;

            for iter in 0..iterations {
                // Generate neighbor
                let neighbor = current.mutate(&mut self.rng, mutation_strength);
                let neighbor_result = self.evaluate(&neighbor);
                let neighbor_score = behavior.score(&neighbor_result.metrics);

                if neighbor_score > current_score {
                    current = neighbor;
                    current_result = neighbor_result;
                    current_score = neighbor_score;
                    no_improvement = 0;
                    mutation_strength = (mutation_strength * 0.95).max(0.05);
                } else {
                    no_improvement += 1;
                    if no_improvement > 10 {
                        mutation_strength = (mutation_strength * 1.1).min(0.5);
                        no_improvement = 0;
                    }
                }

                if iter % 20 == 0 {
                    eprintln!(
                        "Restart {}/{}, Iter {}/{}: score = {:.4}, mutation = {:.3}",
                        restart + 1,
                        restarts,
                        iter + 1,
                        iterations,
                        current_score,
                        mutation_strength
                    );
                }
            }

            // Update best
            if best_result.is_none()
                || behavior.score(&current_result.metrics)
                    > behavior.score(&best_result.as_ref().unwrap().metrics)
            {
                best_result = Some(current_result);
            }
        }

        best_result.expect("At least one restart should complete")
    }

    /// Hybrid search: combines random search with hill-climbing refinement.
    /// First runs random search to find promising regions, then refines the best with hill-climbing.
    pub fn hybrid_search(
        &mut self,
        behavior: PresetBehavior,
        random_iterations: usize,
        hill_climb_iterations: usize,
        top_k: usize,
    ) -> EvaluationResult {
        eprintln!(
            "Phase 1: Random search ({} iterations)...",
            random_iterations
        );

        // Phase 1: Random search to find promising starting points
        let random_results = self.random_search(behavior, random_iterations, true);

        // Select top-k candidates
        let mut sorted = random_results;
        sorted.sort_by(|a, b| {
            behavior
                .score(&b.metrics)
                .partial_cmp(&behavior.score(&a.metrics))
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        let candidates: Vec<_> = sorted.into_iter().take(top_k).collect();

        eprintln!(
            "Phase 1 complete. Top {} scores: {:?}",
            top_k,
            candidates
                .iter()
                .map(|r| behavior.score(&r.metrics))
                .collect::<Vec<_>>()
        );

        // Phase 2: Hill-climb from each candidate
        eprintln!(
            "Phase 2: Hill-climbing refinement ({} iterations x {} candidates)...",
            hill_climb_iterations, top_k
        );

        let mut best_overall: Option<EvaluationResult> = None;

        for (i, candidate) in candidates.iter().enumerate() {
            eprintln!("  Refining candidate {}/{}...", i + 1, top_k);
            let refined =
                self.hill_climb(behavior, Some(candidate.params), hill_climb_iterations, 1);
            let refined_score = behavior.score(&refined.metrics);

            eprintln!(
                "    Candidate {}: initial={:.4}, refined={:.4}",
                i + 1,
                behavior.score(&candidate.metrics),
                refined_score
            );

            if best_overall.is_none()
                || refined_score > behavior.score(&best_overall.as_ref().unwrap().metrics)
            {
                best_overall = Some(refined);
            }
        }

        best_overall.expect("At least one candidate should be refined")
    }

    /// Find optimal parameters for all behaviors using the basic hill-climb method.
    pub fn optimize_all(
        &mut self,
        iterations_per_behavior: usize,
    ) -> Vec<(PresetBehavior, EvaluationResult)> {
        let mut results = Vec::new();

        for &behavior in PresetBehavior::all() {
            eprintln!("\n=== Optimizing {:?} ===", behavior);
            let result = self.hill_climb(behavior, None, iterations_per_behavior, 3);
            eprintln!(
                "Best {:?} score: {:.4}",
                behavior,
                behavior.score(&result.metrics)
            );
            eprintln!(
                "Parameters:\n{}",
                result.params.to_rust_code(&format!("{:?}", behavior))
            );
            results.push((behavior, result));
        }

        results
    }

    /// Find optimal parameters for all behaviors using the hybrid search method.
    /// This is more thorough than optimize_all, combining random exploration with refinement.
    pub fn optimize_all_hybrid(
        &mut self,
        random_iterations: usize,
        hill_climb_iterations: usize,
        top_k: usize,
    ) -> Vec<(PresetBehavior, EvaluationResult)> {
        let mut results = Vec::new();

        for &behavior in PresetBehavior::all() {
            eprintln!("\n=== Hybrid optimization for {:?} ===", behavior);
            let result =
                self.hybrid_search(behavior, random_iterations, hill_climb_iterations, top_k);
            let final_score = behavior.score(&result.metrics);

            eprintln!("Best {:?} score: {:.4}", behavior, final_score);
            eprintln!(
                "Parameters:\n{}",
                result.params.to_rust_code(&format!("{:?}", behavior))
            );

            // Print key metrics for analysis
            eprintln!("Key metrics:");
            eprintln!("  angular_momentum: {:.4}", result.metrics.angular_momentum);
            eprintln!("  heading_variance: {:.4}", result.metrics.heading_variance);
            eprintln!(
                "  trail_fragmentation: {}",
                result.metrics.trail_fragmentation
            );
            eprintln!("  trail_elongation: {:.4}", result.metrics.trail_elongation);
            eprintln!("  flow_coherence: {:.4}", result.metrics.flow_coherence);
            eprintln!(
                "  spatial_concentration: {:.4}",
                result.metrics.spatial_concentration
            );
            eprintln!("  path_continuity: {:.4}", result.metrics.path_continuity);
            eprintln!("  coverage: {:.4}", result.metrics.coverage);

            results.push((behavior, result));
        }

        // Summary
        eprintln!("\n=== SUMMARY ===");
        for (behavior, result) in &results {
            let score = behavior.score(&result.metrics);
            eprintln!(
                "{:?}: score={:.4}, coverage={:.2}, frag={}, elongation={:.2}",
                behavior,
                score,
                result.metrics.coverage,
                result.metrics.trail_fragmentation,
                result.metrics.trail_elongation
            );
        }

        results
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exploration_params_random() {
        let mut rng = Xoshiro256PlusPlus::seed_from_u64(42);
        let params = ExplorationParams::random(&mut rng);

        assert!(params.sensor_angle >= 5.0 && params.sensor_angle <= 90.0);
        assert!(params.decay_factor >= 0.5 && params.decay_factor <= 0.99);
        assert!(params.population >= 5000 && params.population <= 100000);
    }

    #[test]
    fn test_exploration_params_mutate() {
        let mut rng = Xoshiro256PlusPlus::seed_from_u64(42);
        let params = ExplorationParams::random(&mut rng);
        let mutated = params.mutate(&mut rng, 0.1);

        // Should be different but still valid
        assert!(mutated.sensor_angle >= 5.0 && mutated.sensor_angle <= 90.0);
        assert!(mutated.decay_factor >= 0.5 && mutated.decay_factor <= 0.99);
    }

    #[test]
    fn test_explorer_evaluate() {
        let config = ExplorerConfig {
            width: 100,
            height: 100,
            warmup_frames: 10,
            measurement_frames: 5,
            seed: 42,
        };
        let explorer = Explorer::new(config);
        let params = ExplorationParams {
            sensor_angle: 22.5,
            sensor_distance: 9.0,
            rotation_angle: 45.0,
            step_size: 1.0,
            decay_factor: 0.9,
            deposit_amount: 5.0,
            population: 10000,
            diffusion_kernel: DiffusionKernel::Mean3x3,
            wind_dx: None,
            wind_dy: None,
            terrain: TerrainType::None,
            terrain_strength: 1.0,
            init_mode: InitMode::Random,
        };

        let result = explorer.evaluate(&params);

        assert!(!result.scores.is_empty());
        assert!(result.metrics.coverage >= 0.0);
    }

    #[test]
    fn test_to_sim_config() {
        let params = ExplorationParams {
            sensor_angle: 30.0,
            sensor_distance: 15.0,
            rotation_angle: 45.0,
            step_size: 1.5,
            decay_factor: 0.85,
            deposit_amount: 5.0,
            population: 25000,
            diffusion_kernel: DiffusionKernel::Gaussian,
            wind_dx: Some(0.2),
            wind_dy: Some(0.1),
            terrain: TerrainType::Smooth,
            terrain_strength: 1.5,
            init_mode: InitMode::CentralBurst,
        };

        let config = params.to_sim_config();
        assert_eq!(config.sensor_angle, 30.0);
        assert_eq!(config.rotation_angle, 45.0);
        assert_eq!(config.total_population(), 25000);
        assert_eq!(config.diffusion_kernel, DiffusionKernel::Gaussian);
        assert!(config.wind.is_some());
        assert_eq!(config.terrain, TerrainType::Smooth);
    }

    #[test]
    fn test_extended_params_random() {
        let mut rng = Xoshiro256PlusPlus::seed_from_u64(42);
        let params = ExplorationParams::random(&mut rng);

        // Check extended params have valid values
        assert!(params.terrain_strength >= 0.1 && params.terrain_strength <= 5.0);
        // init_mode and terrain can be any valid variant
    }

    #[test]
    fn test_extended_params_biased() {
        let mut rng = Xoshiro256PlusPlus::seed_from_u64(42);

        // Test each behavior has valid extended params
        for behavior in PresetBehavior::all() {
            let params = ExplorationParams::random_biased(&mut rng, *behavior);
            assert!(params.terrain_strength >= 0.1 && params.terrain_strength <= 5.0);
        }
    }
}
