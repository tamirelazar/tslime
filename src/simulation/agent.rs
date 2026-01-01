use rand::Rng as RandRng;
use rand_xoshiro::Xoshiro256PlusPlus as Rng;
use std::f32::consts::PI;

use super::config::Attractor;

#[derive(Clone, Copy)]
pub struct Agent {
    pub x: f32,
    pub y: f32,
    pub heading: f32,
    pub species_id: u8,
}

impl Agent {
    pub fn new(x: f32, y: f32, heading: f32, species_id: u8) -> Self {
        Self {
            x,
            y,
            heading,
            species_id,
        }
    }

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

            let min_dist = 1.0;
            let dist_sq = dist_sq.max(min_dist * min_dist);

            let force = attractor.strength * strength_multiplier / dist_sq.sqrt();

            force_x += dx / dist_sq.sqrt() * force;
            force_y += dy / dist_sq.sqrt() * force;
        }

        if force_x.abs() > 0.001 || force_y.abs() > 0.001 {
            let target_heading = force_y.atan2(force_x);
            let diff = target_heading - self.heading;

            let mut normalized_diff = diff;
            while normalized_diff > PI {
                normalized_diff -= 2.0 * PI;
            }
            while normalized_diff < -PI {
                normalized_diff += 2.0 * PI;
            }

            let steer_strength = 0.1;
            self.heading += normalized_diff * steer_strength;
        }
    }

    pub fn move_forward(&mut self, step_size: f32, width: usize, height: usize) {
        self.x += self.heading.cos() * step_size;
        self.y += self.heading.sin() * step_size;

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

    pub fn deposit(&self, trail: &mut [f32], width: usize, height: usize, deposit_amount: f32) {
        let x = self.x as usize;
        let y = self.y as usize;

        if x < width && y < height {
            let idx = y * width + x;
            trail[idx] += deposit_amount;
        }
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
        agent.move_forward(1.0, 400, 400);
        assert!((agent.x - 101.0).abs() < 0.001);
        assert!((agent.y - 100.0).abs() < 0.001);
    }

    #[test]
    fn test_move_with_heading_90() {
        let mut agent = Agent::new(100.0, 100.0, PI / 2.0, 0);
        agent.move_forward(1.0, 400, 400);
        assert!((agent.x - 100.0).abs() < 0.001);
        assert!((agent.y - 101.0).abs() < 0.001);
    }

    #[test]
    fn test_boundary_handling() {
        let mut agent = Agent::new(0.5, 200.0, PI, 0);
        agent.move_forward(2.0, 400, 400);
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
}
