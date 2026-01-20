//! Parameter space explorer for discovering optimal presets.
//!
//! Runs headless simulations with various parameter combinations and
//! evaluates them using pattern metrics to find parameters that produce
//! specific emergent behaviors.

use crate::exploration::metrics::PatternMetrics;
use crate::simulation::config::{DiffusionKernel, InitMode, SimConfig, SpeciesConfig};
use crate::simulation::Simulation;
use rand::prelude::*;
use rand_xoshiro::Xoshiro256PlusPlus;

/// Parameters being explored (subset of SimConfig for optimization).
#[derive(Debug, Clone, Copy)]
pub struct ExplorationParams {
    pub sensor_angle: f32,
    pub sensor_distance: f32,
    pub rotation_angle: f32,
    pub step_size: f32,
    pub decay_factor: f32,
    pub deposit_amount: f32,
    pub population: usize,
}

impl ExplorationParams {
    /// Create random parameters within valid ranges.
    pub fn random(rng: &mut impl Rng) -> Self {
        Self {
            sensor_angle: rng.gen_range(5.0..90.0),
            sensor_distance: rng.gen_range(1.0..50.0),
            rotation_angle: rng.gen_range(5.0..90.0),
            step_size: rng.gen_range(0.5..5.0),
            decay_factor: rng.gen_range(0.5..0.99),
            deposit_amount: rng.gen_range(1.0..20.0),
            population: rng.gen_range(5000..100000),
        }
    }

    /// Create parameters biased toward a specific behavior.
    pub fn random_biased(rng: &mut impl Rng, behavior: PresetBehavior) -> Self {
        match behavior {
            PresetBehavior::Vortex => Self {
                // rotation_angle > sensor_angle
                sensor_angle: rng.gen_range(5.0..30.0),
                sensor_distance: rng.gen_range(8.0..20.0),
                rotation_angle: rng.gen_range(40.0..90.0),
                step_size: rng.gen_range(0.8..2.0),
                decay_factor: rng.gen_range(0.8..0.95),
                deposit_amount: rng.gen_range(3.0..8.0),
                population: rng.gen_range(30000..80000),
            },
            PresetBehavior::Lightning => Self {
                // Fast, sparse, high contrast
                sensor_angle: rng.gen_range(5.0..20.0),
                sensor_distance: rng.gen_range(10.0..30.0),
                rotation_angle: rng.gen_range(10.0..30.0),
                step_size: rng.gen_range(2.0..5.0),
                decay_factor: rng.gen_range(0.5..0.75),
                deposit_amount: rng.gen_range(10.0..20.0),
                population: rng.gen_range(5000..20000),
            },
            PresetBehavior::Crystal => Self {
                // Slow, stable, persistent
                sensor_angle: rng.gen_range(20.0..50.0),
                sensor_distance: rng.gen_range(15.0..40.0),
                rotation_angle: rng.gen_range(10.0..40.0),
                step_size: rng.gen_range(0.5..1.0),
                decay_factor: rng.gen_range(0.95..0.99),
                deposit_amount: rng.gen_range(2.0..6.0),
                population: rng.gen_range(15000..40000),
            },
            PresetBehavior::Blob => Self {
                // Short-sighted, sharp turns, fast decay
                sensor_angle: rng.gen_range(30.0..70.0),
                sensor_distance: rng.gen_range(1.0..8.0),
                rotation_angle: rng.gen_range(50.0..90.0),
                step_size: rng.gen_range(0.5..1.5),
                decay_factor: rng.gen_range(0.5..0.7),
                deposit_amount: rng.gen_range(5.0..15.0),
                population: rng.gen_range(20000..60000),
            },
            PresetBehavior::Worm => Self {
                // Long-sighted, low population
                sensor_angle: rng.gen_range(10.0..25.0),
                sensor_distance: rng.gen_range(20.0..50.0),
                rotation_angle: rng.gen_range(20.0..45.0),
                step_size: rng.gen_range(1.0..2.5),
                decay_factor: rng.gen_range(0.88..0.96),
                deposit_amount: rng.gen_range(4.0..10.0),
                population: rng.gen_range(3000..15000),
            },
            PresetBehavior::ChaosEdge => Self {
                // sensor_angle ≈ rotation_angle
                sensor_angle: rng.gen_range(15.0..40.0),
                sensor_distance: rng.gen_range(5.0..20.0),
                rotation_angle: rng.gen_range(15.0..40.0),
                step_size: rng.gen_range(0.8..1.5),
                decay_factor: rng.gen_range(0.8..0.92),
                deposit_amount: rng.gen_range(3.0..8.0),
                population: rng.gen_range(30000..70000),
            },
        }
    }

    /// Mutate parameters slightly for local search.
    pub fn mutate(&self, rng: &mut impl Rng, mutation_strength: f32) -> Self {
        let mut mutate_f32 = |v: f32, min: f32, max: f32| -> f32 {
            let delta = (max - min) * mutation_strength * rng.gen_range(-1.0..1.0);
            (v + delta).clamp(min, max)
        };

        Self {
            sensor_angle: mutate_f32(self.sensor_angle, 5.0, 90.0),
            sensor_distance: mutate_f32(self.sensor_distance, 1.0, 50.0),
            rotation_angle: mutate_f32(self.rotation_angle, 5.0, 90.0),
            step_size: mutate_f32(self.step_size, 0.5, 5.0),
            decay_factor: mutate_f32(self.decay_factor, 0.5, 0.99),
            deposit_amount: mutate_f32(self.deposit_amount, 1.0, 20.0),
            population: (self.population as f32
                * (1.0 + mutation_strength * rng.gen_range(-0.5..0.5)))
                .clamp(5000.0, 100000.0) as usize,
        }
    }

    /// Convert to SimConfig for simulation.
    pub fn to_sim_config(&self) -> SimConfig {
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
                color: "ffffff".to_string(),
            }],
            diffusion_kernel: DiffusionKernel::Mean3x3,
            ..Default::default()
        }
    }

    /// Format as Rust code for preset definition.
    pub fn to_rust_code(&self, name: &str) -> String {
        format!(
            r#"Preset::{name} => Self {{
    sensor_angle: {:.1},
    sensor_distance: {:.1},
    rotation_angle: {:.1},
    step_size: {:.2},
    decay_factor: {:.2},
    deposit_amount: {:.1},
    // ... population: {},
}}"#,
            self.sensor_angle,
            self.sensor_distance,
            self.rotation_angle,
            self.step_size,
            self.decay_factor,
            self.deposit_amount,
            self.population,
        )
    }
}

/// Target behaviors for optimization.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PresetBehavior {
    Vortex,
    Lightning,
    Crystal,
    Blob,
    Worm,
    ChaosEdge,
}

impl PresetBehavior {
    /// Get score function for this behavior.
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
    pub params: ExplorationParams,
    pub metrics: PatternMetrics,
    pub scores: Vec<(PresetBehavior, f32)>,
}

/// Explorer configuration.
#[derive(Debug, Clone)]
pub struct ExplorerConfig {
    /// Grid dimensions for simulation.
    pub width: usize,
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
            InitMode::Random,
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
                eprintln!(
                    "Iteration {}/{}: score = {:.4}",
                    i + 1,
                    iterations,
                    score
                );
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
            let mut current = initial.unwrap_or_else(|| {
                ExplorationParams::random_biased(&mut self.rng, behavior)
            });
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

    /// Find optimal parameters for all behaviors.
    pub fn optimize_all(&mut self, iterations_per_behavior: usize) -> Vec<(PresetBehavior, EvaluationResult)> {
        let mut results = Vec::new();

        for &behavior in PresetBehavior::all() {
            eprintln!("\n=== Optimizing {:?} ===", behavior);
            let result = self.hill_climb(behavior, None, iterations_per_behavior, 3);
            eprintln!(
                "Best {:?} score: {:.4}",
                behavior,
                behavior.score(&result.metrics)
            );
            eprintln!("Parameters:\n{}", result.params.to_rust_code(&format!("{:?}", behavior)));
            results.push((behavior, result));
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
        };

        let config = params.to_sim_config();
        assert_eq!(config.sensor_angle, 30.0);
        assert_eq!(config.rotation_angle, 45.0);
        assert_eq!(config.total_population(), 25000);
    }
}
