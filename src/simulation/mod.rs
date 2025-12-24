pub mod agent;
pub mod config;
pub mod trail_map;

use crate::simulation::agent::Agent;
use crate::simulation::config::SimConfig;
use crate::simulation::trail_map::TrailMap;
use rand::Rng as RandRng;
use rand::SeedableRng;
use rand_xoshiro::Xoshiro256PlusPlus as Rng;

pub struct Simulation {
    config: SimConfig,
    agents: Vec<Agent>,
    trail_map: TrailMap,
    rng: Rng,
}

impl Simulation {
    pub fn new(width: usize, height: usize, config: SimConfig, seed: u64) -> Self {
        let mut rng = Rng::seed_from_u64(seed);
        let mut agents = Vec::with_capacity(config.population);

        for _ in 0..config.population {
            let x = rng.gen_range(0.0..width as f32);
            let y = rng.gen_range(0.0..height as f32);
            let heading = rng.gen_range(0.0..std::f32::consts::PI * 2.0);
            agents.push(Agent::new(x, y, heading));
        }

        Self {
            config,
            agents,
            trail_map: TrailMap::new(width, height),
            rng,
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

    pub fn trail_map(&self) -> &TrailMap {
        &self.trail_map
    }

    #[allow(dead_code)]
    pub fn trail_map_mut(&mut self) -> &mut TrailMap {
        &mut self.trail_map
    }

    pub fn update(&mut self) {
        let width = self.trail_map.width();
        let height = self.trail_map.height();

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

            agent.move_forward(self.config.step_size, width, height);

            agent.deposit(
                self.trail_map.current_mut(),
                width,
                height,
                self.config.deposit_amount,
            );
        }

        self.trail_map.diffuse();
        self.trail_map.decay(self.config.decay_factor);
    }

    pub fn agents(&self) -> &[Agent] {
        &self.agents
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simulation_creation() {
        let config = SimConfig::default();
        let sim = Simulation::new(400, 400, config, 42);
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
        let mut sim = Simulation::new(400, 400, config, 42);

        let initial_max = *sim
            .trail_map()
            .current()
            .iter()
            .max_by(|a, b| a.total_cmp(b))
            .unwrap();
        assert_eq!(initial_max, 0.0);

        sim.update();

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
        let mut sim = Simulation::new(400, 400, config, 42);

        sim.update();
        let max_after_1 = *sim
            .trail_map()
            .current()
            .iter()
            .max_by(|a, b| a.total_cmp(b))
            .unwrap();

        sim.update();
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
        let mut sim1 = Simulation::new(400, 400, config.clone(), 42);
        let mut sim2 = Simulation::new(400, 400, config, 42);

        sim1.update();
        sim2.update();

        assert_eq!(sim1.trail_map().current(), sim2.trail_map().current());

        for (a1, a2) in sim1.agents().iter().zip(sim2.agents().iter()) {
            assert!((a1.x - a2.x).abs() < 0.001);
            assert!((a1.y - a2.y).abs() < 0.001);
            assert!((a1.heading - a2.heading).abs() < 0.001);
        }
    }
}
