//! Individual agent behavior for Physarum simulation.
//!
//! Each agent represents a single cell of the slime mold. Agents follow
//! the sense-rotate-move-deposit cycle to create emergent network patterns.

use noise::{NoiseFn, Perlin};
use rand::Rng as RandRng;
use rand_xoshiro::Xoshiro256PlusPlus as Rng;
use std::f32::consts::PI;

use super::config::{Attractor, Obstacle, TerrainType, Wind};
use super::constants::steering::{
    ATTRACTOR_MIN_DIST, ATTRACTOR_STRENGTH, FORCE_THRESHOLD, TERRAIN_NOISE_OFFSET,
    TERRAIN_SCALE_SMOOTH, TERRAIN_SCALE_TURBULENT, TERRAIN_STRENGTH_MIXED, TERRAIN_STRENGTH_SMOOTH,
    TERRAIN_STRENGTH_TURBULENT, WIND_MIN_STRENGTH, WIND_STEER_STRENGTH, WIND_STRENGTH_MULTIPLIER,
};

/// Normalizes an angle to the range [-PI, PI].
#[inline]
pub fn normalize_angle(mut angle: f32) -> f32 {
    while angle > PI {
        angle -= 2.0 * PI;
    }
    while angle < -PI {
        angle += 2.0 * PI;
    }
    angle
}

/// Wrapper for Perlin noise generation used in terrain effects.
pub struct NoiseWrapper {
    perlin: Perlin,
    seed_val: u32,
}

impl NoiseWrapper {
    /// Create a new noise generator with the given seed.
    pub fn new(seed: u32) -> Self {
        Self {
            perlin: Perlin::new(seed),
            seed_val: seed,
        }
    }

    /// Sample noise at the given 2D coordinates.
    pub fn get(&self, x: f64, y: f64) -> f64 {
        self.perlin.get([x, y])
    }

    /// Get the seed value used for this noise generator.
    pub fn seed_value(&self) -> u32 {
        self.seed_val
    }
}

/// A single agent (particle) in the Physarum simulation.
///
/// Each agent has a position (x, y), heading angle, and species ID.
/// The agent struct is kept minimal (16 bytes) for cache efficiency
/// when processing 50,000+ agents per frame.
#[derive(Clone, Copy)]
pub struct Agent {
    /// X position in the simulation grid.
    pub x: f32,
    /// Y position in the simulation grid.
    pub y: f32,
    /// Movement direction in radians.
    pub heading: f32,
    /// Identifier for the agent's species (used for color/config).
    pub species_id: u8,
}

impl Agent {
    /// Create a new agent at (x, y) with the given heading and species.
    pub fn new(x: f32, y: f32, heading: f32, species_id: u8) -> Self {
        Self {
            x,
            y,
            heading,
            species_id,
        }
    }

    /// Sense the pheromone trail at left, center, and right sensors.
    ///
    /// Returns a tuple of (left, center, right) sensed values.
    pub fn sense(
        &self,
        trail: &[f32],
        width: usize,
        height: usize,
        sensor_angle: f32,
        sensor_distance: f32,
    ) -> (f32, f32, f32) {
        let sensor_angle_rad = sensor_angle * PI / 180.0;

        let left_angle = self.heading - sensor_angle_rad;
        let center_angle = self.heading;
        let right_angle = self.heading + sensor_angle_rad;

        let left_x = self.x + left_angle.cos() * sensor_distance;
        let left_y = self.y + left_angle.sin() * sensor_distance;

        let center_x = self.x + center_angle.cos() * sensor_distance;
        let center_y = self.y + center_angle.sin() * sensor_distance;

        let right_x = self.x + right_angle.cos() * sensor_distance;
        let right_y = self.y + right_angle.sin() * sensor_distance;

        let left = sample_trail(trail, width, height, left_x, left_y);
        let center = sample_trail(trail, width, height, center_x, center_y);
        let right = sample_trail(trail, width, height, right_x, right_y);

        (left, center, right)
    }

    /// Update heading based on sensed values.
    ///
    /// Turns towards the strongest signal, or randomly if forward blocked.
    pub fn rotate(
        &mut self,
        left: f32,
        center: f32,
        right: f32,
        rotation_angle: f32,
        rng: &mut Rng,
    ) {
        let rotation_angle_rad = rotation_angle * PI / 180.0;

        if center > left && center > right {
        } else if center < left && center < right {
            if RandRng::gen(rng) {
                self.heading -= rotation_angle_rad;
            } else {
                self.heading += rotation_angle_rad;
            }
        } else if left > right {
            self.heading -= rotation_angle_rad;
        } else if right > left {
            self.heading += rotation_angle_rad;
        }
    }

    /// Apply steering forces from attractors (or repellers).
    pub fn apply_attractor_forces(
        &mut self,
        attractors: &[Attractor],
        strength_multiplier: f32,
        _width: usize,
        _height: usize,
    ) {
        if attractors.is_empty() {
            return;
        }

        let mut force_x: f32 = 0.0;
        let mut force_y: f32 = 0.0;

        for attractor in attractors {
            let dx = attractor.x - self.x;
            let dy = attractor.y - self.y;
            let dist_sq = dx * dx + dy * dy;

            let dist_sq = dist_sq.max(ATTRACTOR_MIN_DIST * ATTRACTOR_MIN_DIST);

            let force = attractor.strength * strength_multiplier / dist_sq.sqrt();

            force_x += dx / dist_sq.sqrt() * force;
            force_y += dy / dist_sq.sqrt() * force;
        }

        if force_x.abs() > FORCE_THRESHOLD || force_y.abs() > FORCE_THRESHOLD {
            let target_heading = force_y.atan2(force_x);
            self.apply_steering(target_heading, ATTRACTOR_STRENGTH);
        }
    }

    /// Apply constant wind force to heading.
    pub fn apply_wind_force(&mut self, wind: Option<Wind>, strength_multiplier: f32) {
        if let Some(w) = wind {
            let wind_strength = WIND_STRENGTH_MULTIPLIER * w.dx * strength_multiplier;
            let wind_strength_y = WIND_STRENGTH_MULTIPLIER * w.dy * strength_multiplier;

            if wind_strength.abs() > WIND_MIN_STRENGTH || wind_strength_y.abs() > WIND_MIN_STRENGTH
            {
                let target_heading = wind_strength_y.atan2(wind_strength);
                self.apply_steering(target_heading, WIND_STEER_STRENGTH);
            }
        }
    }

    /// Apply terrain-based steering bias using Perlin noise.
    pub fn apply_terrain_bias(
        &mut self,
        terrain: TerrainType,
        terrain_strength: f32,
        noise: &NoiseWrapper,
    ) {
        let seed_val = noise.seed_value() as f64;
        match terrain {
            TerrainType::None => {}
            TerrainType::Smooth => {
                let nx = self.x as f64 * TERRAIN_SCALE_SMOOTH as f64 + seed_val;
                let ny = self.y as f64 * TERRAIN_SCALE_SMOOTH as f64 + seed_val;
                let noise_val = noise.get(nx, ny);

                let angle = (noise_val as f32 - 0.5) * PI * 2.0 * terrain_strength;
                self.apply_steering(angle, TERRAIN_STRENGTH_SMOOTH);
            }
            TerrainType::Turbulent => {
                let nx = self.x as f64 * TERRAIN_SCALE_TURBULENT as f64
                    + seed_val
                    + TERRAIN_NOISE_OFFSET as f64;
                let ny = self.y as f64 * TERRAIN_SCALE_TURBULENT as f64
                    + seed_val
                    + TERRAIN_NOISE_OFFSET as f64;
                let noise_val = noise.get(nx, ny);

                let angle = (noise_val as f32 - 0.5) * PI * 2.0 * terrain_strength;
                self.apply_steering(angle, TERRAIN_STRENGTH_TURBULENT);
            }
            TerrainType::Mixed => {
                let nx = self.x as f64 * TERRAIN_SCALE_SMOOTH as f64 + seed_val;
                let ny = self.y as f64 * TERRAIN_SCALE_SMOOTH as f64 + seed_val;
                let smooth_val = noise.get(nx, ny);

                let nx = self.x as f64 * TERRAIN_SCALE_TURBULENT as f64
                    + seed_val
                    + TERRAIN_NOISE_OFFSET as f64;
                let ny = self.y as f64 * TERRAIN_SCALE_TURBULENT as f64
                    + seed_val
                    + TERRAIN_NOISE_OFFSET as f64;
                let turb_val = noise.get(nx, ny);

                let smooth_angle = (smooth_val as f32 - 0.5) * PI * 2.0 * terrain_strength;
                let turb_angle = (turb_val as f32 - 0.5) * PI * 2.0 * terrain_strength * 0.5;

                let combined_angle = smooth_angle + turb_angle;
                self.apply_steering(combined_angle, TERRAIN_STRENGTH_MIXED);
            }
        }
    }

    /// Move agent forward and handle collisions with boundaries and obstacles.
    pub fn move_forward(
        &mut self,
        step_size: f32,
        width: usize,
        height: usize,
        obstacles: &[Obstacle],
        obstacle_masks: &[Option<super::config::ObstacleMask>],
    ) {
        self.x += self.heading.cos() * step_size;
        self.y += self.heading.sin() * step_size;

        for (i, obstacle) in obstacles.iter().enumerate() {
            let mask = obstacle_masks.get(i).cloned().flatten();
            if obstacle.contains(self.x, self.y, mask.as_ref()) {
                self.heading = obstacle.bounce(self.x, self.y, self.heading, mask.as_ref());
                self.x += self.heading.cos() * step_size;
                self.y += self.heading.sin() * step_size;
            }
        }

        if self.x < 0.0 {
            self.x = 0.0;
            self.heading = PI - self.heading;
        } else if self.x >= width as f32 {
            self.x = (width - 1) as f32;
            self.heading = PI - self.heading;
        }

        if self.y < 0.0 {
            self.y = 0.0;
            self.heading = -self.heading;
        } else if self.y >= height as f32 {
            self.y = (height - 1) as f32;
            self.heading = -self.heading;
        }
    }

    /// Deposit pheromone at the current position.
    pub fn deposit(&self, trail: &mut [f32], width: usize, height: usize, deposit_amount: f32) {
        let x = self.x as usize;
        let y = self.y as usize;

        if x < width && y < height {
            let idx = y * width + x;
            trail[idx] += deposit_amount;
        }
    }

    #[inline]
    fn apply_steering(&mut self, target_angle: f32, steer_strength: f32) {
        let diff = normalize_angle(target_angle - self.heading);
        self.heading += diff * steer_strength;
    }
}

fn sample_trail(trail: &[f32], width: usize, height: usize, x: f32, y: f32) -> f32 {
    let ix = x.floor() as i32;
    let iy = y.floor() as i32;

    if ix >= 0 && ix < width as i32 && iy >= 0 && iy < height as i32 {
        let idx = iy as usize * width + ix as usize;
        trail[idx]
    } else {
        0.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;

    #[test]
    fn test_agent_creation() {
        let agent = Agent::new(200.0, 200.0, 0.0, 0);
        assert_eq!(agent.x, 200.0);
        assert_eq!(agent.y, 200.0);
        assert_eq!(agent.heading, 0.0);
        assert_eq!(agent.species_id, 0);
    }

    #[test]
    fn test_agent_size() {
        assert_eq!(std::mem::size_of::<Agent>(), 16);
    }

    #[test]
    fn test_move_forward() {
        let mut agent = Agent::new(100.0, 100.0, 0.0, 0);
        agent.move_forward(1.0, 400, 400, &[], &[]);
        assert!((agent.x - 101.0).abs() < 0.001);
        assert!((agent.y - 100.0).abs() < 0.001);
    }

    #[test]
    fn test_move_with_heading_90() {
        let mut agent = Agent::new(100.0, 100.0, PI / 2.0, 0);
        agent.move_forward(1.0, 400, 400, &[], &[]);
        assert!((agent.x - 100.0).abs() < 0.001);
        assert!((agent.y - 101.0).abs() < 0.001);
    }

    #[test]
    fn test_boundary_handling() {
        let mut agent = Agent::new(0.5, 200.0, PI, 0);
        agent.move_forward(2.0, 400, 400, &[], &[]);
        assert!(agent.x >= 0.0);
    }

    #[test]
    fn test_deposit() {
        let mut trail = vec![0.0; 400 * 400];
        let agent = Agent::new(100.0, 100.0, 0.0, 0);
        agent.deposit(&mut trail, 400, 400, 5.0);
        assert_eq!(trail[100 * 400 + 100], 5.0);
    }

    #[test]
    fn test_sense() {
        let mut trail = vec![0.0; 400 * 400];
        let agent = Agent::new(100.0, 100.0, 0.0, 0);
        let sensor_x = (100.0 + 0.0_f32.cos() * 9.0) as usize;
        let sensor_y = 100_usize;
        trail[sensor_y * 400 + sensor_x] = 10.0;
        let (_left, center, _right) = agent.sense(&trail, 400, 400, 22.5, 9.0);
        assert_eq!(center, 10.0);
    }

    #[test]
    fn test_rotate_center_strongest() {
        let mut agent = Agent::new(100.0, 100.0, 0.0, 0);
        let mut rng = rand_xoshiro::Xoshiro256PlusPlus::seed_from_u64(42);
        agent.rotate(1.0, 10.0, 1.0, 45.0, &mut rng);
        assert_eq!(agent.heading, 0.0);
    }

    #[test]
    fn test_rotate_left_strongest() {
        let mut agent = Agent::new(100.0, 100.0, 0.0, 0);
        let mut rng = rand_xoshiro::Xoshiro256PlusPlus::seed_from_u64(42);
        agent.rotate(10.0, 1.0, 1.0, 45.0, &mut rng);
        assert!((agent.heading - (-45.0 * PI / 180.0)).abs() < 0.001);
    }

    #[test]
    fn test_rotate_right_strongest() {
        let mut agent = Agent::new(100.0, 100.0, 0.0, 0);
        let mut rng = rand_xoshiro::Xoshiro256PlusPlus::seed_from_u64(42);
        agent.rotate(1.0, 1.0, 10.0, 45.0, &mut rng);
        assert!((agent.heading - (45.0 * PI / 180.0)).abs() < 0.001);
    }

    #[test]
    fn test_apply_attractor_forces_attract() {
        let mut agent = Agent::new(100.0, 100.0, PI / 2.0, 0);
        let attractors = vec![Attractor::new(150.0, 200.0, 1.0)];
        agent.apply_attractor_forces(&attractors, 1.0, 400, 400);
        assert!(
            (agent.heading - PI / 2.0).abs() < 0.15,
            "heading should adjust toward attractor, got {}",
            agent.heading
        );
    }

    #[test]
    fn test_apply_attractor_forces_repel() {
        let mut agent = Agent::new(100.0, 100.0, PI / 2.0, 0);
        let attractors = vec![Attractor::new(150.0, 200.0, -1.0)];
        agent.apply_attractor_forces(&attractors, 1.0, 400, 400);
        assert!(
            (agent.heading - PI / 2.0).abs() > 0.1,
            "heading should turn away from repeller, got {}",
            agent.heading
        );
    }

    #[test]
    fn test_apply_attractor_forces_no_attractors() {
        let mut agent = Agent::new(100.0, 100.0, 0.0, 0);
        let original_heading = agent.heading;
        agent.apply_attractor_forces(&[], 1.0, 400, 400);
        assert_eq!(agent.heading, original_heading);
    }

    #[test]
    fn test_apply_attractor_forces_multiple() {
        let mut agent = Agent::new(200.0, 200.0, 0.0, 0);
        let attractors = vec![
            Attractor::new(100.0, 200.0, 1.0),
            Attractor::new(300.0, 200.0, 1.0),
        ];
        agent.apply_attractor_forces(&attractors, 1.0, 400, 400);
    }

    #[test]
    fn test_apply_wind_force_no_wind() {
        let mut agent = Agent::new(100.0, 100.0, 0.0, 0);
        let original_heading = agent.heading;
        agent.apply_wind_force(None, 1.0);
        assert_eq!(agent.heading, original_heading);
    }

    #[test]
    fn test_apply_wind_force_with_wind() {
        let mut agent = Agent::new(100.0, 100.0, PI / 2.0, 0);
        let original_heading = agent.heading;
        agent.apply_wind_force(Some(Wind::new(1.0, 0.0)), 1.0);
        assert!(
            (agent.heading - original_heading).abs() > 0.1,
            "heading should change with wind force, got {} (original was {})",
            agent.heading,
            original_heading
        );
    }

    #[test]
    fn test_apply_terrain_bias_none() {
        let noise = NoiseWrapper::new(42);
        let mut agent = Agent::new(100.0, 100.0, 0.0, 0);
        let original_heading = agent.heading;
        agent.apply_terrain_bias(TerrainType::None, 1.0, &noise);
        assert_eq!(agent.heading, original_heading);
    }

    #[test]
    fn test_apply_terrain_bias_smooth() {
        let noise = NoiseWrapper::new(42);
        let mut agent = Agent::new(100.0, 100.0, 0.0, 0);
        agent.apply_terrain_bias(TerrainType::Smooth, 1.0, &noise);
        assert!(
            agent.heading.is_finite(),
            "heading should be finite after terrain bias"
        );
    }

    #[test]
    fn test_apply_terrain_bias_turbulent() {
        let noise = NoiseWrapper::new(42);
        let mut agent = Agent::new(100.0, 100.0, 0.0, 0);
        agent.apply_terrain_bias(TerrainType::Turbulent, 1.0, &noise);
        assert!(
            agent.heading.is_finite(),
            "heading should be finite after terrain bias"
        );
    }

    #[test]
    fn test_apply_terrain_bias_mixed() {
        let noise = NoiseWrapper::new(42);
        let mut agent = Agent::new(100.0, 100.0, 0.0, 0);
        agent.apply_terrain_bias(TerrainType::Mixed, 1.0, &noise);
        assert!(
            agent.heading.is_finite(),
            "heading should be finite after terrain bias"
        );
    }

    #[test]
    fn test_noise_wrapper_seed() {
        let noise = NoiseWrapper::new(123);
        assert_eq!(noise.seed_value(), 123);
    }

    #[test]
    fn test_move_forward_with_obstacles() {
        let mut agent = Agent::new(95.0, 100.0, 0.0, 0);
        let obstacles = vec![Obstacle::Circle {
            x: 100.0,
            y: 100.0,
            radius: 10.0,
        }];
        let obstacle_masks = vec![None];
        // Move into circle
        agent.move_forward(10.0, 400, 400, &obstacles, &obstacle_masks);
        assert!(agent.heading != 0.0);
    }

    #[test]
    fn test_deposit_out_of_bounds() {
        let mut trail = vec![0.0; 100];
        let agent = Agent::new(20.0, 20.0, 0.0, 0);
        agent.deposit(&mut trail, 10, 10, 1.0);
        assert!(trail.iter().all(|&x| x == 0.0));
    }

    #[test]
    fn test_rotate_random_choice() {
        let mut agent = Agent::new(100.0, 100.0, 0.0, 0);
        let mut rng = rand_xoshiro::Xoshiro256PlusPlus::seed_from_u64(42);
        // left == right, center is smaller
        agent.rotate(10.0, 1.0, 10.0, 45.0, &mut rng);
        let h1 = agent.heading;

        let mut agent = Agent::new(100.0, 100.0, 0.0, 0);
        let mut rng = rand_xoshiro::Xoshiro256PlusPlus::seed_from_u64(43); // Different seed
        agent.rotate(10.0, 1.0, 10.0, 45.0, &mut rng);
        let h2 = agent.heading;

        assert!(h1 != h2 || h1 != 0.0); // At least one of them rotated
    }

    #[test]
    fn test_sample_trail_out_of_bounds() {
        let trail = vec![1.0; 100];
        assert_eq!(sample_trail(&trail, 10, 10, -1.0, 5.0), 0.0);
        assert_eq!(sample_trail(&trail, 10, 10, 11.0, 5.0), 0.0);
        assert_eq!(sample_trail(&trail, 10, 10, 5.0, -1.0), 0.0);
        assert_eq!(sample_trail(&trail, 10, 10, 5.0, 11.0), 0.0);
    }
}

#[cfg(test)]
mod prop_tests {
    use super::*;
    use proptest::prelude::*;
    use rand::SeedableRng;

    proptest! {
        #[test]
        fn test_agent_move_always_finite(
            x in 0.0..1000.0f32,
            y in 0.0..1000.0f32,
            heading in -10.0..10.0f32,
            step_size in 0.0..10.0f32,
        ) {
            let mut agent = Agent::new(x, y, heading, 0);
            agent.move_forward(step_size, 1000, 1000, &[], &[]);
            prop_assert!(agent.x.is_finite());
            prop_assert!(agent.y.is_finite());
            prop_assert!(agent.heading.is_finite());
        }

        #[test]
        fn test_rotate_always_finite(
            left in 0.0..1000.0f32,
            center in 0.0..1000.0f32,
            right in 0.0..1000.0f32,
            rotation_angle in 0.0..180.0f32,
        ) {
            let mut agent = Agent::new(0.0, 0.0, 0.0, 0);
            let mut rng = rand_xoshiro::Xoshiro256PlusPlus::seed_from_u64(42);
            agent.rotate(left, center, right, rotation_angle, &mut rng);
            prop_assert!(agent.heading.is_finite());
        }
    }
}
