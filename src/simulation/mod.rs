pub mod agent;
pub mod config;
pub mod trail_map;

use crate::simulation::agent::Agent;
use crate::simulation::config::{InitMode, SimConfig};
use crate::simulation::trail_map::TrailMap;
use rand::Rng as RandRng;
use rand::SeedableRng;
use rand_xoshiro::Xoshiro256PlusPlus as Rng;

pub struct TrailHistory {
    history: Vec<Vec<f32>>,
    capacity: usize,
    current_index: usize,
    count: usize,
}

impl TrailHistory {
    pub fn new(capacity: usize) -> Self {
        Self {
            history: Vec::with_capacity(capacity),
            capacity,
            current_index: 0,
            count: 0,
        }
    }

    pub fn push(&mut self, trail_map: &[f32]) {
        if self.capacity == 0 {
            return;
        }

        if self.history.len() < self.capacity {
            self.history.push(trail_map.to_vec());
            self.count = self.history.len();
        } else {
            self.history[self.current_index].copy_from_slice(trail_map);
        }

        self.current_index = (self.current_index + 1) % self.capacity;
    }

    pub fn blended(&self) -> Option<Vec<f32>> {
        if self.count == 0 {
            return None;
        }

        let mut result = vec![0.0f32; self.history[0].len()];
        for frame in &self.history[..self.count] {
            for (i, &val) in frame.iter().enumerate() {
                result[i] += val;
            }
        }

        let weight = 1.0 / self.count as f32;
        for val in &mut result {
            *val *= weight;
        }

        Some(result)
    }

    #[allow(dead_code)]
    pub fn count(&self) -> usize {
        self.count
    }

    #[allow(dead_code)]
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    pub fn clear(&mut self) {
        self.history.clear();
        self.current_index = 0;
        self.count = 0;
    }
}

pub struct Simulation {
    config: SimConfig,
    agents: Vec<Agent>,
    trail_map: TrailMap,
    rng: Rng,
    trail_history: Option<TrailHistory>,
}

impl Simulation {
    pub fn new(
        width: usize,
        height: usize,
        config: SimConfig,
        seed: u64,
        init_mode: InitMode,
        trail_history_capacity: usize,
    ) -> Self {
        let mut rng = Rng::seed_from_u64(seed);
        let mut agents = Vec::with_capacity(config.population);

        match init_mode {
            InitMode::Random => {
                Self::init_random(&mut rng, width, height, &mut agents, config.population);
            }
            InitMode::CentralBurst => {
                Self::init_central_burst(&mut rng, width, height, &mut agents, config.population);
            }
            InitMode::Circle => {
                Self::init_circle(&mut rng, width, height, &mut agents, config.population);
            }
            InitMode::Gradient => {
                Self::init_gradient(&mut rng, width, height, &mut agents, config.population);
            }
            InitMode::WaveFront => {
                Self::init_wave_front(&mut rng, width, height, &mut agents, config.population);
            }
            InitMode::Spiral => {
                Self::init_spiral(&mut rng, width, height, &mut agents, config.population);
            }
            InitMode::RandomClusters => {
                Self::init_random_clusters(&mut rng, width, height, &mut agents, config.population);
            }
        }

        let sigma = config.diffusion_sigma;
        let trail_history = if trail_history_capacity > 0 {
            Some(TrailHistory::new(trail_history_capacity))
        } else {
            None
        };

        Self {
            config,
            agents,
            trail_map: TrailMap::new_with_sigma(width, height, sigma),
            rng,
            trail_history,
        }
    }

    fn init_random(
        rng: &mut Rng,
        width: usize,
        height: usize,
        agents: &mut Vec<Agent>,
        population: usize,
    ) {
        for _ in 0..population {
            let x = rng.gen_range(0.0..width as f32);
            let y = rng.gen_range(0.0..height as f32);
            let heading = rng.gen_range(0.0..std::f32::consts::PI * 2.0);
            agents.push(Agent::new(x, y, heading));
        }
    }

    fn init_central_burst(
        rng: &mut Rng,
        width: usize,
        height: usize,
        agents: &mut Vec<Agent>,
        population: usize,
    ) {
        let center_x = width as f32 / 2.0;
        let center_y = height as f32 / 2.0;

        for _ in 0..population {
            let x = center_x + rng.gen_range(-2.0..2.0);
            let y = center_y + rng.gen_range(-2.0..2.0);
            let heading = rng.gen_range(0.0..std::f32::consts::PI * 2.0);
            agents.push(Agent::new(x, y, heading));
        }
    }

    fn init_circle(
        rng: &mut Rng,
        width: usize,
        height: usize,
        agents: &mut Vec<Agent>,
        population: usize,
    ) {
        let center_x = width as f32 / 2.0;
        let center_y = height as f32 / 2.0;
        let radius = (width.min(height) as f32) * 0.35;

        for _ in 0..population {
            let angle = rng.gen_range(0.0..std::f32::consts::PI * 2.0);
            let x = center_x + angle.cos() * radius;
            let y = center_y + angle.sin() * radius;
            let heading = (angle + std::f32::consts::PI).atan2(0.0);
            agents.push(Agent::new(x, y, heading));
        }
    }

    fn init_gradient(
        rng: &mut Rng,
        width: usize,
        height: usize,
        agents: &mut Vec<Agent>,
        population: usize,
    ) {
        let center_x = width as f32 / 2.0;
        let center_y = height as f32 / 2.0;
        let max_radius = (width.min(height) as f32) * 0.45;

        for _ in 0..population {
            let r = rng.gen_range(0.0..1.0);
            let radius = r * r * max_radius;
            let angle = rng.gen_range(0.0..std::f32::consts::PI * 2.0);
            let x = center_x + angle.cos() * radius;
            let y = center_y + angle.sin() * radius;
            let heading = rng.gen_range(0.0..std::f32::consts::PI * 2.0);
            agents.push(Agent::new(x, y, heading));
        }
    }

    fn init_wave_front(
        rng: &mut Rng,
        width: usize,
        height: usize,
        agents: &mut Vec<Agent>,
        population: usize,
    ) {
        for _ in 0..population {
            let side = rng.gen_range(0..4);
            let (x, y, heading) = match side {
                0 => (
                    rng.gen_range(0.0..width as f32),
                    0.0,
                    std::f32::consts::PI / 2.0,
                ),
                1 => (
                    width as f32 - 1.0,
                    rng.gen_range(0.0..height as f32),
                    std::f32::consts::PI,
                ),
                2 => (
                    rng.gen_range(0.0..width as f32),
                    height as f32 - 1.0,
                    -std::f32::consts::PI / 2.0,
                ),
                _ => (0.0, rng.gen_range(0.0..height as f32), 0.0),
            };
            let heading = heading + rng.gen_range(-0.1..0.1);
            agents.push(Agent::new(x, y, heading));
        }
    }

    fn init_spiral(
        rng: &mut Rng,
        width: usize,
        height: usize,
        agents: &mut Vec<Agent>,
        population: usize,
    ) {
        let center_x = width as f32 / 2.0;
        let center_y = height as f32 / 2.0;
        let max_radius = (width.min(height) as f32) * 0.4;
        let golden_angle = std::f32::consts::PI * (3.0 - 5.0f32.sqrt());

        for i in 0..population {
            let radius = (i as f32 / population as f32).sqrt() * max_radius;
            let angle = i as f32 * golden_angle;
            let x = center_x + angle.cos() * radius;
            let y = center_y + angle.sin() * radius;
            let heading = angle + std::f32::consts::PI / 2.0;
            let heading = heading + rng.gen_range(-0.05..0.05);
            agents.push(Agent::new(x, y, heading));
        }
    }

    fn init_random_clusters(
        rng: &mut Rng,
        width: usize,
        height: usize,
        agents: &mut Vec<Agent>,
        population: usize,
    ) {
        let num_clusters = rng.gen_range(3..7);
        let agents_per_cluster = population / num_clusters;

        for cluster in 0..num_clusters {
            let cluster_x = rng.gen_range(width as f32 * 0.2..width as f32 * 0.8);
            let cluster_y = rng.gen_range(height as f32 * 0.2..height as f32 * 0.8);

            let count = if cluster == num_clusters - 1 {
                population - (num_clusters - 1) * agents_per_cluster
            } else {
                agents_per_cluster
            };

            for _ in 0..count {
                let x = cluster_x + rng.gen_range(-20.0..20.0);
                let y = cluster_y + rng.gen_range(-20.0..20.0);
                let heading = rng.gen_range(0.0..std::f32::consts::PI * 2.0);
                agents.push(Agent::new(x, y, heading));
            }
        }
    }

    pub fn width(&self) -> usize {
        self.trail_map.width()
    }

    pub fn height(&self) -> usize {
        self.trail_map.height()
    }

    #[allow(dead_code)]
    pub fn config(&self) -> &SimConfig {
        &self.config
    }

    #[allow(dead_code)]
    pub fn trail_map(&self) -> &TrailMap {
        &self.trail_map
    }

    pub fn trail_map_blended(&self) -> Vec<f32> {
        if let Some(ref history) = self.trail_history {
            if let Some(blended) = history.blended() {
                return blended;
            }
        }
        self.trail_map.current().to_vec()
    }

    #[allow(dead_code)]
    pub fn trail_map_mut(&mut self) -> &mut TrailMap {
        &mut self.trail_map
    }

    pub fn update(&mut self, dt: f32) {
        let width = self.trail_map.width();
        let height = self.trail_map.height();

        let effective_step_size = self.config.step_size * dt;
        let effective_deposit = self.config.deposit_amount * dt;
        let effective_decay = self.config.decay_factor.powf(dt);

        for agent in &mut self.agents {
            let trail = self.trail_map.current();

            let (left, center, right) = agent.sense(
                trail,
                width,
                height,
                self.config.sensor_angle,
                self.config.sensor_distance,
            );

            agent.rotate(
                left,
                center,
                right,
                self.config.rotation_angle,
                &mut self.rng,
            );

            agent.move_forward(effective_step_size, width, height);

            agent.deposit(
                self.trail_map.current_mut(),
                width,
                height,
                effective_deposit,
            );
        }

        self.trail_map.diffuse_with_kernel(matches!(
            self.config.diffusion_kernel,
            crate::simulation::config::DiffusionKernel::Gaussian
        ));
        self.trail_map.decay(effective_decay);

        if let Some(ref mut history) = self.trail_history {
            history.push(self.trail_map.current());
        }
    }

    #[allow(dead_code)]
    pub fn agents(&self) -> &[Agent] {
        &self.agents
    }

    pub fn reset(&mut self, seed: u64, init_mode: InitMode) {
        self.rng = Rng::seed_from_u64(seed);
        self.agents.clear();
        self.agents = Vec::with_capacity(self.config.population);

        match init_mode {
            InitMode::Random => {
                Self::init_random(&mut self.rng, self.trail_map.width(), self.trail_map.height(), &mut self.agents, self.config.population);
            }
            InitMode::CentralBurst => {
                Self::init_central_burst(&mut self.rng, self.trail_map.width(), self.trail_map.height(), &mut self.agents, self.config.population);
            }
            InitMode::Circle => {
                Self::init_circle(&mut self.rng, self.trail_map.width(), self.trail_map.height(), &mut self.agents, self.config.population);
            }
            InitMode::Gradient => {
                Self::init_gradient(&mut self.rng, self.trail_map.width(), self.trail_map.height(), &mut self.agents, self.config.population);
            }
            InitMode::WaveFront => {
                Self::init_wave_front(&mut self.rng, self.trail_map.width(), self.trail_map.height(), &mut self.agents, self.config.population);
            }
            InitMode::Spiral => {
                Self::init_spiral(&mut self.rng, self.trail_map.width(), self.trail_map.height(), &mut self.agents, self.config.population);
            }
            InitMode::RandomClusters => {
                Self::init_random_clusters(&mut self.rng, self.trail_map.width(), self.trail_map.height(), &mut self.agents, self.config.population);
            }
        }

        self.trail_map.clear();
        if let Some(ref mut history) = self.trail_history {
            history.clear();
        }
    }

    pub fn update_config(&mut self, config: SimConfig) {
        self.config = config;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simulation_creation() {
        let config = SimConfig::default();
        let sim = Simulation::new(400, 400, config, 42, InitMode::Random, 0);
        assert_eq!(sim.width(), 400);
        assert_eq!(sim.height(), 400);
        assert_eq!(sim.agents().len(), 50000);
    }

    #[test]
    fn test_update_changes_trail() {
        let config = SimConfig {
            population: 100,
            ..Default::default()
        };
        let mut sim = Simulation::new(400, 400, config, 42, InitMode::Random, 0);

        let initial_max = *sim
            .trail_map()
            .current()
            .iter()
            .max_by(|a, b| a.total_cmp(b))
            .unwrap();
        assert_eq!(initial_max, 0.0);

        sim.update(1.0);

        let max_after = *sim
            .trail_map()
            .current()
            .iter()
            .max_by(|a, b| a.total_cmp(b))
            .unwrap();
        assert!(max_after > 0.0);
    }

    #[test]
    fn test_multiple_updates() {
        let config = SimConfig {
            population: 100,
            decay_factor: 0.99,
            ..Default::default()
        };
        let mut sim = Simulation::new(400, 400, config, 42, InitMode::Random, 0);

        sim.update(1.0);
        let max_after_1 = *sim
            .trail_map()
            .current()
            .iter()
            .max_by(|a, b| a.total_cmp(b))
            .unwrap();

        sim.update(1.0);
        let max_after_2 = *sim
            .trail_map()
            .current()
            .iter()
            .max_by(|a, b| a.total_cmp(b))
            .unwrap();

        assert!(max_after_2 < max_after_1 * 1.5);
    }

    #[test]
    fn test_reproducibility() {
        let config = SimConfig::default();
        let mut sim1 = Simulation::new(400, 400, config.clone(), 42, InitMode::Random, 0);
        let mut sim2 = Simulation::new(400, 400, config, 42, InitMode::Random, 0);

        sim1.update(1.0);
        sim2.update(1.0);

        assert_eq!(sim1.trail_map().current(), sim2.trail_map().current());

        for (a1, a2) in sim1.agents().iter().zip(sim2.agents().iter()) {
            assert!((a1.x - a2.x).abs() < 0.001);
            assert!((a1.y - a2.y).abs() < 0.001);
            assert!((a1.heading - a2.heading).abs() < 0.001);
        }
    }

    #[test]
    fn test_fps_invariance() {
        let config = SimConfig::default();
        let mut sim_low_fps = Simulation::new(100, 100, config.clone(), 42, InitMode::Random, 0);
        let mut sim_high_fps = Simulation::new(100, 100, config, 42, InitMode::Random, 0);

        for _ in 0..15 {
            sim_low_fps.update(2.0);
        }

        for _ in 0..30 {
            sim_high_fps.update(1.0);
        }

        let sum_low: f32 = sim_low_fps.trail_map().current().iter().sum();
        let sum_high: f32 = sim_high_fps.trail_map().current().iter().sum();

        let diff_ratio = (sum_low - sum_high).abs() / (sum_low + sum_high).max(0.001);
        assert!(diff_ratio < 0.25, "Low FPS ({}) and high FPS ({}) simulations should produce similar results, diff ratio: {}", sum_low, sum_high, diff_ratio);
    }

    #[test]
    fn test_time_scaling() {
        let config = SimConfig::default();
        let mut sim_half_speed = Simulation::new(100, 100, config.clone(), 42, InitMode::Random, 0);
        let mut sim_normal = Simulation::new(100, 100, config.clone(), 42, InitMode::Random, 0);
        let mut sim_double_speed = Simulation::new(100, 100, config, 42, InitMode::Random, 0);

        for _ in 0..20 {
            sim_half_speed.update(0.5);
        }

        for _ in 0..10 {
            sim_normal.update(1.0);
        }

        for _ in 0..5 {
            sim_double_speed.update(2.0);
        }

        let sum_half: f32 = sim_half_speed.trail_map().current().iter().sum();
        let sum_normal: f32 = sim_normal.trail_map().current().iter().sum();
        let sum_double: f32 = sim_double_speed.trail_map().current().iter().sum();

        let diff_half = (sum_half - sum_normal).abs() / (sum_half + sum_normal).max(0.001);
        let diff_double = (sum_double - sum_normal).abs() / (sum_double + sum_normal).max(0.001);
        assert!(
            diff_half < 0.15,
            "Half speed ({}) should match normal speed ({}) over 2x time, diff: {}",
            sum_half,
            sum_normal,
            diff_half
        );
        assert!(
            diff_double < 0.25,
            "Double speed ({}) should match normal speed ({}) over 0.5x time, diff: {}",
            sum_double,
            sum_normal,
            diff_double
        );
    }

    #[test]
    fn test_trail_history_creation() {
        let history = TrailHistory::new(5);
        assert_eq!(history.capacity(), 5);
        assert_eq!(history.count(), 0);
    }

    #[test]
    fn test_trail_history_push_and_blend() {
        let mut history = TrailHistory::new(3);

        let frame1 = vec![1.0, 2.0, 3.0, 4.0];
        let frame2 = vec![2.0, 4.0, 6.0, 8.0];
        let frame3 = vec![3.0, 6.0, 9.0, 12.0];

        history.push(&frame1);
        history.push(&frame2);
        history.push(&frame3);

        assert_eq!(history.count(), 3);

        let blended = history.blended().unwrap();
        assert_eq!(blended, vec![2.0, 4.0, 6.0, 8.0]);
    }

    #[test]
    fn test_trail_history_circular_buffer() {
        let mut history = TrailHistory::new(2);

        let frame1 = vec![1.0, 1.0];
        let frame2 = vec![2.0, 2.0];
        let frame3 = vec![3.0, 3.0];

        history.push(&frame1);
        history.push(&frame2);
        history.push(&frame3);

        assert_eq!(history.count(), 2);

        let blended = history.blended().unwrap();
        assert_eq!(blended, vec![2.5, 2.5]);
    }

    #[test]
    fn test_trail_history_no_frames() {
        let history = TrailHistory::new(3);
        assert!(history.blended().is_none());
    }

    #[test]
    fn test_trail_history_clear() {
        let mut history = TrailHistory::new(3);

        let frame = vec![1.0, 2.0];
        history.push(&frame);
        history.push(&frame);

        assert_eq!(history.count(), 2);

        history.clear();

        assert_eq!(history.count(), 0);
        assert!(history.blended().is_none());
    }

    #[test]
    fn test_trail_history_disabled() {
        let config = SimConfig::default();
        let sim = Simulation::new(100, 100, config, 42, InitMode::Random, 0);
        assert!(sim.trail_history.is_none());
    }

    #[test]
    fn test_trail_history_enabled() {
        let config = SimConfig::default();
        let sim = Simulation::new(100, 100, config, 42, InitMode::Random, 5);
        assert!(sim.trail_history.is_some());
    }

    #[test]
    fn test_trail_map_blended_without_history() {
        let config = SimConfig::default();
        let sim = Simulation::new(100, 100, config, 42, InitMode::Random, 0);

        let blended = sim.trail_map_blended();
        let current = sim.trail_map().current();
        assert_eq!(blended, current);
    }
}
