//! Simulation engine for Physarum polycephalum behavior.
//!
//! This module contains the core simulation logic including:
//! - [`Simulation`]: The main simulation orchestrator
//! - [`crate::simulation::agent`]: Individual agent behavior (sense, rotate, move)
//! - [`crate::simulation::config`]: Configuration and presets
//! - [`crate::simulation::trail_map`]: Pheromone trail grid and diffusion
//! - [`crate::simulation::food`]: Food source loading from images

pub mod agent;
pub mod config;
pub mod food;
pub mod trail_map;

use crate::simulation::agent::Agent;
use crate::simulation::agent::NoiseWrapper;
use crate::simulation::config::{InitMode, SimConfig};
use crate::simulation::food::{get_brightness_at, load_default_food_image, load_image_grayscale};
use crate::simulation::trail_map::TrailMap;
use rand::Rng as RandRng;
use rand::SeedableRng;
use rand_xoshiro::Xoshiro256PlusPlus as Rng;

/// Circular buffer for storing trail map history, used for motion blur effects.
///
/// Maintains a fixed-size buffer of recent trail maps that can be blended
/// together to create smooth motion blur effects.
pub struct TrailHistory {
    history: Vec<Vec<f32>>,
    capacity: usize,
    current_index: usize,
    count: usize,
    /// Pre-allocated buffer for blended results to avoid per-frame allocations
    blended_buffer: Vec<f32>,
}

impl TrailHistory {
    /// Create a new trail history buffer with the given capacity and frame size.
    ///
    /// All internal buffers are pre-allocated to avoid runtime allocations.
    pub fn new(capacity: usize, frame_size: usize) -> Self {
        let mut history = Vec::with_capacity(capacity);
        // Pre-allocate all history buffers to avoid allocations during push()
        for _ in 0..capacity {
            history.push(vec![0.0f32; frame_size]);
        }

        Self {
            history,
            capacity,
            current_index: 0,
            count: 0,
            blended_buffer: vec![0.0f32; frame_size],
        }
    }

    /// Push a new trail map frame into the history buffer.
    ///
    /// If the buffer is full, overwrites the oldest frame.
    /// This method never allocates - all buffers are pre-allocated.
    pub fn push(&mut self, trail_map: &[f32]) {
        if self.capacity == 0 {
            return;
        }

        // Buffer is already pre-allocated, just copy data
        self.history[self.current_index].copy_from_slice(trail_map);
        self.current_index = (self.current_index + 1) % self.capacity;

        // Update count until we reach capacity
        if self.count < self.capacity {
            self.count += 1;
        }
    }

    /// Calculate the average of all frames in the history buffer.
    ///
    /// Returns `None` if the history is empty.
    /// The returned slice is valid until the next call to blended() or clear().
    pub fn blended(&mut self) -> Option<&[f32]> {
        if self.count == 0 {
            return None;
        }

        // Reset blended buffer
        self.blended_buffer.fill(0.0);

        // Sum all frames
        for frame in &self.history[..self.count] {
            for (i, &val) in frame.iter().enumerate() {
                self.blended_buffer[i] += val;
            }
        }

        // Average
        let weight = 1.0 / self.count as f32;
        for val in &mut self.blended_buffer {
            *val *= weight;
        }

        Some(&self.blended_buffer)
    }

    /// Get the current number of frames stored.
    pub fn count(&self) -> usize {
        self.count
    }

    /// Get the maximum capacity of the history buffer.
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// Clear all history frames.
    ///
    /// Resets the history but keeps all allocations for reuse.
    pub fn clear(&mut self) {
        self.current_index = 0;
        self.count = 0;
    }
}

/// The main simulation engine for Physarum polycephalum behavior.
///
/// Manages a population of agents that sense, move, and deposit pheromones
/// on a trail map. The trail map undergoes diffusion and decay each frame
/// to create organic, network-forming patterns.
///
/// # Example
///
/// ```rust,no_run
/// use tslime::Simulation;
/// use tslime::simulation::config::{SimConfig, InitMode};
///
/// let config = SimConfig::default();
/// let mut sim = Simulation::new(400, 400, config, 42, InitMode::Random, 0);
///
/// // Advance simulation by one frame
/// sim.update(1.0);
///
/// // Get trail data for rendering
/// let mut trail = Vec::new();
/// sim.trail_map_blended(&mut trail);
/// ```
pub struct Simulation {
    config: SimConfig,
    agents: Vec<Agent>,
    trail_maps: Vec<TrailMap>,
    rng: Rng,
    trail_history: Option<TrailHistory>,
    noise: NoiseWrapper,
    trail_age: Option<Vec<f32>>,
    prev_trail: Option<Vec<f32>>,
    trail_delta: Option<Vec<f32>>,
    gradient_magnitude: Option<Vec<f32>>,
    /// Pre-allocated buffer for combining separate species trails.
    /// Only allocated when both `separate_species_trails` and trail history are enabled.
    combined_trail_buffer: Option<Vec<f32>>,
}

impl Simulation {
    /// Create a new simulation with the given dimensions and configuration.
    pub fn new(
        width: usize,
        height: usize,
        config: SimConfig,
        seed: u64,
        init_mode: InitMode,
        trail_history_capacity: usize,
    ) -> Self {
        let mut rng = Rng::seed_from_u64(seed);
        let total_population = config.total_population();
        let mut agents = Vec::with_capacity(total_population);

        let food_path = config.food_image_path.as_deref();
        let food_invert = config.food_image_invert;
        let food_scale = config.food_image_scale;

        for (species_id, species_config) in config.species_configs.iter().enumerate() {
            Self::init_species(
                &mut rng,
                width,
                height,
                &mut agents,
                species_config.count,
                init_mode,
                species_id as u8,
                food_path,
                food_invert,
                food_scale,
            );
        }

        let sigma = config.diffusion_sigma;
        let frame_size = width * height;
        let trail_history = if trail_history_capacity > 0 {
            Some(TrailHistory::new(trail_history_capacity, frame_size))
        } else {
            None
        };

        let num_trails = if config.separate_species_trails {
            config.species_configs.len()
        } else {
            1
        };

        let mut trail_maps = Vec::with_capacity(num_trails);
        for _ in 0..num_trails {
            trail_maps.push(TrailMap::new_with_sigma(width, height, sigma));
        }

        let noise_seed = (seed % u64::MAX) as u32;
        let noise = NoiseWrapper::new(noise_seed);

        // Pre-allocate combined buffer only when needed for separate species + history
        let combined_trail_buffer = if trail_history_capacity > 0 && config.separate_species_trails
        {
            Some(vec![0.0f32; frame_size])
        } else {
            None
        };

        Self {
            config,
            agents,
            trail_maps,
            rng,
            trail_history,
            noise,
            trail_age: None,
            prev_trail: None,
            trail_delta: None,
            gradient_magnitude: None,
            combined_trail_buffer,
        }
    }

    /// Compute gradient magnitude for edge glow effect.
    ///
    /// Computes the magnitude of the gradient using central differences on the
    /// simulation-resolution trail map. This is used for the edge glow visual effect.
    ///
    /// # Arguments
    /// * `trail_data` - The trail map data to compute gradient from
    /// * `width` - Width of the trail map
    /// * `height` - Height of the trail map
    /// * `gradient` - Mutable slice to store the computed gradient magnitudes
    fn compute_gradient_magnitude(
        trail_data: &[f32],
        width: usize,
        height: usize,
        gradient: &mut [f32],
    ) {
        // Reset gradient buffer
        gradient.fill(0.0);

        // Compute gradient using central differences
        for y in 1..height - 1 {
            for x in 1..width - 1 {
                let idx = y * width + x;
                let up = (y - 1) * width + x;
                let dn = (y + 1) * width + x;
                let lt = y * width + (x - 1);
                let rt = y * width + (x + 1);

                // Central differences for gradient
                let gx = (trail_data[rt] - trail_data[lt]) * 0.5;
                let gy = (trail_data[dn] - trail_data[up]) * 0.5;

                // Gradient magnitude
                gradient[idx] = (gx * gx + gy * gy).sqrt();
            }
        }

        // Normalize to [0, 1]
        let max_val = gradient.iter().copied().fold(0.0f32, f32::max);
        if max_val > 0.0 {
            for g in gradient.iter_mut() {
                *g = (*g / max_val).min(1.0);
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn init_species(
        rng: &mut Rng,
        width: usize,
        height: usize,
        agents: &mut Vec<Agent>,
        population: usize,
        init_mode: InitMode,
        species_id: u8,
        food_image_path: Option<&str>,
        food_image_invert: bool,
        food_image_scale: f32,
    ) {
        match init_mode {
            InitMode::Random => {
                Self::init_random(rng, width, height, agents, population, species_id);
            }
            InitMode::CentralBurst => {
                Self::init_central_burst(rng, width, height, agents, population, species_id);
            }
            InitMode::Circle => {
                Self::init_circle(rng, width, height, agents, population, species_id);
            }
            InitMode::Gradient => {
                Self::init_gradient(rng, width, height, agents, population, species_id);
            }
            InitMode::WaveFront => {
                Self::init_wave_front(rng, width, height, agents, population, species_id);
            }
            InitMode::Spiral => {
                Self::init_spiral(rng, width, height, agents, population, species_id);
            }
            InitMode::RandomClusters => {
                Self::init_random_clusters(rng, width, height, agents, population, species_id);
            }
            InitMode::Petri => {
                Self::init_petri(rng, width, height, agents, population, species_id);
            }
            InitMode::Food => {
                if let Some(path) = food_image_path {
                    Self::init_from_food(
                        rng,
                        width,
                        height,
                        agents,
                        population,
                        species_id,
                        path,
                        food_image_invert,
                        food_image_scale,
                    );
                } else {
                    eprintln!("Warning: Food mode selected but no image path provided, falling back to random");
                    Self::init_random(rng, width, height, agents, population, species_id);
                }
            }
        }
    }

    fn init_random(
        rng: &mut Rng,
        width: usize,
        height: usize,
        agents: &mut Vec<Agent>,
        population: usize,
        species_id: u8,
    ) {
        for _ in 0..population {
            let x = rng.gen_range(0.0..width as f32);
            let y = rng.gen_range(0.0..height as f32);
            let heading = rng.gen_range(0.0..std::f32::consts::PI * 2.0);
            agents.push(Agent::new(x, y, heading, species_id));
        }
    }

    fn init_central_burst(
        rng: &mut Rng,
        width: usize,
        height: usize,
        agents: &mut Vec<Agent>,
        population: usize,
        species_id: u8,
    ) {
        let center_x = width as f32 / 2.0;
        let center_y = height as f32 / 2.0;

        for _ in 0..population {
            let x = center_x + rng.gen_range(-2.0..2.0);
            let y = center_y + rng.gen_range(-2.0..2.0);
            let heading = rng.gen_range(0.0..std::f32::consts::PI * 2.0);
            agents.push(Agent::new(x, y, heading, species_id));
        }
    }

    fn init_circle(
        rng: &mut Rng,
        width: usize,
        height: usize,
        agents: &mut Vec<Agent>,
        population: usize,
        species_id: u8,
    ) {
        let center_x = width as f32 / 2.0;
        let center_y = height as f32 / 2.0;
        let radius = (width.min(height) as f32) * 0.35;

        for _ in 0..population {
            let angle = rng.gen_range(0.0..std::f32::consts::PI * 2.0);
            let x = center_x + angle.cos() * radius;
            let y = center_y + angle.sin() * radius;
            let heading = (angle + std::f32::consts::PI).atan2(0.0);
            agents.push(Agent::new(x, y, heading, species_id));
        }
    }

    fn init_gradient(
        rng: &mut Rng,
        width: usize,
        height: usize,
        agents: &mut Vec<Agent>,
        population: usize,
        species_id: u8,
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
            agents.push(Agent::new(x, y, heading, species_id));
        }
    }

    fn init_wave_front(
        rng: &mut Rng,
        width: usize,
        height: usize,
        agents: &mut Vec<Agent>,
        population: usize,
        species_id: u8,
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
            agents.push(Agent::new(x, y, heading, species_id));
        }
    }

    fn init_spiral(
        rng: &mut Rng,
        width: usize,
        height: usize,
        agents: &mut Vec<Agent>,
        population: usize,
        species_id: u8,
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
            agents.push(Agent::new(x, y, heading, species_id));
        }
    }

    fn init_random_clusters(
        rng: &mut Rng,
        width: usize,
        height: usize,
        agents: &mut Vec<Agent>,
        population: usize,
        species_id: u8,
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
                agents.push(Agent::new(x, y, heading, species_id));
            }
        }
    }

    fn init_petri(
        rng: &mut Rng,
        width: usize,
        height: usize,
        agents: &mut Vec<Agent>,
        population: usize,
        species_id: u8,
    ) {
        let center_x = width as f32 / 2.0;
        let center_y = height as f32 / 2.0;
        // Standard deviation for Gaussian distribution (pixels)
        let sigma = 5.0;

        for _ in 0..population {
            // Use Box-Muller transform for Gaussian distribution
            let u1: f32 = rng.gen();
            let u2: f32 = rng.gen();
            let r = (-2.0 * u1.ln()).sqrt();
            let theta = 2.0 * std::f32::consts::PI * u2;

            let dx = r * theta.cos() * sigma;
            let dy = r * theta.sin() * sigma;

            let x = center_x + dx;
            let y = center_y + dy;
            let heading = rng.gen_range(0.0..std::f32::consts::PI * 2.0);

            agents.push(Agent::new(x, y, heading, species_id));
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn init_from_food(
        rng: &mut Rng,
        width: usize,
        height: usize,
        agents: &mut Vec<Agent>,
        population: usize,
        species_id: u8,
        food_path: &str,
        food_image_invert: bool,
        food_image_scale: f32,
    ) {
        // Use embedded default food image for standard initialization,
        // or load from custom path if user specified one.
        let brightness_map = if food_path == "assets/tslime_logo.png" {
            match load_default_food_image(width, height, food_image_invert, food_image_scale) {
                Ok(map) => map,
                Err(e) => {
                    eprintln!("Warning: Failed to load embedded food image: {}", e);
                    eprintln!("Falling back to random initialization");
                    return Self::init_random(rng, width, height, agents, population, species_id);
                }
            }
        } else {
            match load_image_grayscale(
                food_path,
                width,
                height,
                food_image_invert,
                food_image_scale,
            ) {
                Ok(map) => map,
                Err(e) => {
                    eprintln!("Warning: Failed to load food image '{}': {}", food_path, e);
                    eprintln!("Falling back to random initialization");
                    return Self::init_random(rng, width, height, agents, population, species_id);
                }
            }
        };

        let total_brightness: f32 = brightness_map.iter().sum();
        if total_brightness == 0.0 {
            eprintln!(
                "Warning: Food image is completely dark, falling back to random initialization"
            );
            return Self::init_random(rng, width, height, agents, population, species_id);
        }

        let agents_per_brightness_unit = population as f32 / total_brightness;

        for y in 0..height {
            for x in 0..width {
                let brightness = get_brightness_at(&brightness_map, width, x, y);
                let expected_agents = brightness * agents_per_brightness_unit;

                let base_x = x as f32;
                let base_y = y as f32;

                let mut agents_to_spawn = expected_agents as usize;

                if rng.gen::<f32>() < expected_agents - agents_to_spawn as f32 {
                    agents_to_spawn += 1;
                }

                for _ in 0..agents_to_spawn {
                    let offset_x = rng.gen_range(-0.5..0.5);
                    let offset_y = rng.gen_range(-0.5..0.5);
                    let heading = rng.gen_range(0.0..std::f32::consts::PI * 2.0);
                    agents.push(Agent::new(
                        base_x + offset_x,
                        base_y + offset_y,
                        heading,
                        species_id,
                    ));
                }
            }
        }

        if agents.len() > population {
            agents.truncate(population);
        }
    }

    /// Get the width of the simulation grid.
    pub fn width(&self) -> usize {
        self.trail_maps.first().map(|tm| tm.width()).unwrap_or(0)
    }

    /// Get the height of the simulation grid.
    pub fn height(&self) -> usize {
        self.trail_maps.first().map(|tm| tm.height()).unwrap_or(0)
    }

    /// Get the total number of agents.
    pub fn agent_count(&self) -> usize {
        self.agents.len()
    }

    /// Get the total number of active attractors.
    pub fn attractor_count(&self) -> usize {
        self.config.attractors.len() + self.config.mouse_attractors.len()
    }

    /// Get the number of obstacles.
    pub fn obstacle_count(&self) -> usize {
        self.config.obstacles.len()
    }

    /// Get the number of agent species.
    pub fn species_count(&self) -> usize {
        self.config.species_configs.len()
    }

    #[allow(dead_code)]
    /// Get references to all trail maps.
    pub fn trail_maps(&self) -> &[TrailMap] {
        &self.trail_maps
    }

    #[allow(dead_code)]
    /// Get a reference to the primary trail map.
    pub fn trail_map(&self) -> &TrailMap {
        &self.trail_maps[0]
    }

    /// Get the combined trail map for rendering.
    ///
    /// Applies motion blur if enabled (via history blending) or combines
    /// multiple species trails if separate trails are enabled.
    ///
    /// # Performance
    /// This method writes to a pre-allocated buffer to avoid allocations.
    /// The buffer is cleared and reused on each call.
    pub fn trail_map_blended(&mut self, output: &mut Vec<f32>) {
        // Check if we have history with blended data
        if let Some(blended) = self.trail_history.as_mut().and_then(|h| h.blended()) {
            let size = blended.len();
            if output.len() != size {
                output.resize(size, 0.0);
            }
            output.copy_from_slice(blended);
            return;
        }

        if self.config.separate_species_trails {
            let width = self.width();
            let height = self.height();
            let size = width * height;
            if output.len() != size {
                output.resize(size, 0.0);
            } else {
                output.fill(0.0);
            }
            for trail_map in &self.trail_maps {
                for (i, &val) in trail_map.current().iter().enumerate() {
                    output[i] += val;
                }
            }
        } else {
            let source = self.trail_maps[0].current();
            let size = source.len();
            if output.len() != size {
                output.resize(size, 0.0);
            }
            output.copy_from_slice(source);
        }
    }

    /// Get trail maps for each species as separate slices.
    pub fn trail_maps_for_species_colors(&self) -> Vec<&[f32]> {
        self.trail_maps.iter().map(|tm| tm.current()).collect()
    }

    #[allow(dead_code)]
    /// Get a mutable reference to the primary trail map.
    pub fn trail_map_mut(&mut self) -> &mut TrailMap {
        &mut self.trail_maps[0]
    }

    #[allow(dead_code)]
    /// Get the current simulation configuration.
    pub fn config(&self) -> &SimConfig {
        &self.config
    }

    /// Advance the simulation by one time step `dt`.
    ///
    /// This performs the sense-rotate-move-deposit cycle for all agents,
    /// and then diffuses and decays the trail map.
    pub fn update(&mut self, dt: f32) {
        let width = self.width();
        let height = self.height();

        let effective_step_size = self.config.step_size * dt;
        let effective_decay = self.config.decay_factor.powf(dt);

        let attractors = self.config.effective_attractors();
        let attractor_strength = self.config.attractor_strength * dt;

        let obstacles = &self.config.obstacles;
        let obstacle_masks = &self.config.obstacle_masks;

        let wind = self.config.wind;
        let terrain = self.config.terrain;
        let terrain_strength = self.config.terrain_strength * dt;

        let separate_trails = self.config.separate_species_trails;
        let boundary_mode = self.config.boundary_mode;
        let respawn_config = self.config.respawn_config;

        if separate_trails {
            for species_idx in 0..self.config.species_configs.len() {
                let species_config = &self.config.species_configs[species_idx];
                let trail_idx = species_idx;
                let has_modulation = species_config.trail_modulation.is_some();
                let modulation = species_config.trail_modulation.unwrap_or_default();

                let trail = self.trail_maps[trail_idx].current();
                for agent in self
                    .agents
                    .iter_mut()
                    .filter(|a| a.species_id as usize == species_idx)
                {
                    // Compute modulated parameters if enabled
                    let (sensor_angle, sensor_distance, rotation_angle, step_size) =
                        if has_modulation {
                            let x = agent.sample_trail_at_position(trail, width, height);
                            let params = modulation.compute_params(x);
                            (
                                params.sensor_angle,
                                params.sensor_distance,
                                params.rotation_angle,
                                params.step_size,
                            )
                        } else {
                            (
                                species_config.sensor_angle,
                                self.config.sensor_distance,
                                species_config.rotation_angle,
                                effective_step_size,
                            )
                        };

                    let (left, center, right) = if has_modulation {
                        agent.sense_with_offsets(
                            trail,
                            width,
                            height,
                            sensor_angle,
                            sensor_distance,
                            modulation.vertical_offset,
                            modulation.heading_offset,
                        )
                    } else {
                        agent.sense(trail, width, height, sensor_angle, sensor_distance)
                    };

                    agent.rotate(left, center, right, rotation_angle, &mut self.rng);

                    agent.apply_attractor_forces(&attractors, attractor_strength);

                    agent.apply_wind_force(wind, dt);

                    agent.apply_terrain_bias(terrain, terrain_strength, &self.noise);

                    agent.move_forward(
                        if has_modulation {
                            step_size * dt
                        } else {
                            effective_step_size
                        },
                        width,
                        height,
                        obstacles,
                        obstacle_masks,
                        boundary_mode,
                    );
                }

                let trail_mut = self.trail_maps[trail_idx].current_mut();
                for agent in self
                    .agents
                    .iter_mut()
                    .filter(|a| a.species_id as usize == species_idx)
                {
                    agent.deposit(trail_mut, width, height, species_config.deposit_amount * dt);
                }
            }
        } else {
            let species_config = self
                .config
                .species_configs
                .first()
                .cloned()
                .unwrap_or_default();
            let has_modulation = species_config.trail_modulation.is_some();
            let modulation = species_config.trail_modulation.unwrap_or_default();

            let trail = self.trail_maps[0].current();
            for agent in self.agents.iter_mut() {
                // Compute modulated parameters if enabled
                let (sensor_angle, sensor_distance, rotation_angle, step_size) = if has_modulation {
                    let x = agent.sample_trail_at_position(trail, width, height);
                    let params = modulation.compute_params(x);
                    (
                        params.sensor_angle,
                        params.sensor_distance,
                        params.rotation_angle,
                        params.step_size,
                    )
                } else {
                    (
                        species_config.sensor_angle,
                        self.config.sensor_distance,
                        species_config.rotation_angle,
                        effective_step_size,
                    )
                };

                let (left, center, right) = if has_modulation {
                    agent.sense_with_offsets(
                        trail,
                        width,
                        height,
                        sensor_angle,
                        sensor_distance,
                        modulation.vertical_offset,
                        modulation.heading_offset,
                    )
                } else {
                    agent.sense(trail, width, height, sensor_angle, sensor_distance)
                };

                agent.rotate(left, center, right, rotation_angle, &mut self.rng);

                agent.apply_attractor_forces(&attractors, attractor_strength);

                agent.apply_wind_force(wind, dt);

                agent.apply_terrain_bias(terrain, terrain_strength, &self.noise);

                agent.move_forward(
                    if has_modulation {
                        step_size * dt
                    } else {
                        effective_step_size
                    },
                    width,
                    height,
                    obstacles,
                    obstacle_masks,
                    boundary_mode,
                );
            }

            let trail_mut = self.trail_maps[0].current_mut();
            for agent in self.agents.iter_mut() {
                agent.deposit(trail_mut, width, height, species_config.deposit_amount * dt);
            }
        }

        // Handle particle respawn
        if respawn_config.interval > 0 {
            use rand::Rng;
            let should_check_respawn = self
                .agents
                .first()
                .is_some_and(|a| a.progress % respawn_config.interval as u8 == 0);
            if should_check_respawn {
                let trail = self.trail_maps[0].current();
                for agent in &mut self.agents {
                    agent.progress = agent.progress.wrapping_add(1);
                    let mut probability = respawn_config.base_probability;
                    if respawn_config.trail_dependent {
                        let x = agent.sample_trail_at_position(trail, width, height);
                        probability *= 1.0 + x * (respawn_config.max_probability_multiplier - 1.0);
                    }
                    if self.rng.gen::<f32>() < probability * dt {
                        // Respawn at random position
                        agent.x = self.rng.gen_range(0.0..width as f32);
                        agent.y = self.rng.gen_range(0.0..height as f32);
                        agent.heading = self.rng.gen_range(0.0..std::f32::consts::PI * 2.0);
                    }
                }
            } else {
                for agent in &mut self.agents {
                    agent.progress = agent.progress.wrapping_add(1);
                }
            }
        }

        for trail_map in &mut self.trail_maps {
            trail_map.diffuse_with_kernel(
                self.config.use_simd,
                matches!(
                    self.config.diffusion_kernel,
                    crate::simulation::config::DiffusionKernel::Gaussian
                ),
            );
            trail_map.decay(effective_decay);
        }

        // Compute trail age: increment where pheromone present, reset where absent
        // Clamp dt to prevent accumulation errors from large time steps
        let safe_dt = dt.min(1.0);
        if let Some(ref mut age) = self.trail_age {
            let current = self.trail_maps[0].current();
            let max_val = current.iter().copied().fold(0.0f32, f32::max);
            let threshold = max_val * 0.01;
            for (a, &v) in age.iter_mut().zip(current.iter()) {
                if v > threshold {
                    *a = (*a + safe_dt).min(crate::config_defaults::visual_fx::AGE_MAX_SECONDS);
                } else {
                    *a = 0.0;
                }
            }
        }

        // Compute trail delta: absolute difference from previous frame, normalized
        if let (Some(ref mut delta), Some(ref mut prev)) =
            (&mut self.trail_delta, &mut self.prev_trail)
        {
            let current = self.trail_maps[0].current();
            let max_val = current.iter().copied().fold(0.0f32, f32::max).max(0.001);
            for ((d, p), &c) in delta.iter_mut().zip(prev.iter_mut()).zip(current.iter()) {
                *d = (c - *p).abs() / max_val;
                *p = c;
            }
        }

        // Compute gradient magnitude for edge glow effect
        if let Some(ref mut gradient) = self.gradient_magnitude {
            // Use primary trail map directly to avoid allocation from trail_map_blended()
            Self::compute_gradient_magnitude(self.trail_maps[0].current(), width, height, gradient);
        }

        self.config.remove_expired_mouse_attractors();

        if let Some(ref mut history) = self.trail_history {
            if self.config.separate_species_trails {
                // Use pre-allocated buffer to avoid allocation in hot path
                if let Some(ref mut combined) = self.combined_trail_buffer {
                    combined.fill(0.0);
                    for trail_map in &self.trail_maps {
                        for (i, &val) in trail_map.current().iter().enumerate() {
                            combined[i] += val;
                        }
                    }
                    history.push(combined);
                }
            } else {
                history.push(self.trail_maps[0].current());
            }
        }
    }

    #[allow(dead_code)]
    /// Get a reference to the agent list.
    pub fn agents(&self) -> &[Agent] {
        &self.agents
    }

    /// Reset the simulation with a new seed and initialization mode.
    ///
    /// Clears trails, re-initializes agents, and resets random state.
    pub fn reset(&mut self, seed: u64, init_mode: InitMode) {
        self.rng = Rng::seed_from_u64(seed);
        self.agents.clear();

        let total_population = self.config.total_population();
        self.agents = Vec::with_capacity(total_population);

        let width = self.width();
        let height = self.height();

        let food_path = self.config.food_image_path.as_deref();
        let food_invert = self.config.food_image_invert;
        let food_scale = self.config.food_image_scale;

        for (species_id, species_config) in self.config.species_configs.iter().enumerate() {
            Self::init_species(
                &mut self.rng,
                width,
                height,
                &mut self.agents,
                species_config.count,
                init_mode,
                species_id as u8,
                food_path,
                food_invert,
                food_scale,
            );
        }

        for trail_map in &mut self.trail_maps {
            trail_map.clear();
        }
        if let Some(ref mut history) = self.trail_history {
            history.clear();
        }
        if let Some(ref mut buf) = self.trail_age {
            buf.fill(0.0);
        }
        if let Some(ref mut buf) = self.trail_delta {
            buf.fill(0.0);
        }
        if let Some(ref mut buf) = self.prev_trail {
            buf.fill(0.0);
        }
        if let Some(ref mut buf) = self.gradient_magnitude {
            buf.fill(0.0);
        }

        // Re-seed noise
        let noise_seed = (seed % u64::MAX) as u32;
        self.noise = NoiseWrapper::new(noise_seed);
    }

    /// Update the simulation configuration at runtime.
    ///
    /// Adjusts trail map buffers if the number of species trails changes.
    /// Also manages the combined trail buffer for separate species with history.
    pub fn update_config(&mut self, config: SimConfig) {
        let old_separate_trails = self.config.separate_species_trails;
        self.config = config;
        let num_trails = if self.config.separate_species_trails {
            self.config.species_configs.len()
        } else {
            1
        };
        while self.trail_maps.len() < num_trails {
            self.trail_maps.push(TrailMap::new_with_sigma(
                self.width(),
                self.height(),
                self.config.diffusion_sigma,
            ));
        }

        // Manage combined trail buffer based on configuration changes
        let needs_combined_buffer =
            self.config.separate_species_trails && self.trail_history.is_some();

        if needs_combined_buffer && self.combined_trail_buffer.is_none() {
            // Need to create the buffer
            self.combined_trail_buffer = Some(vec![0.0f32; self.width() * self.height()]);
        } else if !needs_combined_buffer && self.combined_trail_buffer.is_some() {
            // No longer need the buffer
            self.combined_trail_buffer = None;
        } else if needs_combined_buffer
            && old_separate_trails != self.config.separate_species_trails
        {
            // Configuration changed, ensure buffer is cleared
            if let Some(ref mut buf) = self.combined_trail_buffer {
                buf.fill(0.0);
            }
        }
    }

    /// Enable or disable trail age computation.
    pub fn set_compute_trail_age(&mut self, enabled: bool) {
        if enabled && self.trail_age.is_none() {
            self.trail_age = Some(vec![0.0; self.width() * self.height()]);
        }
    }

    /// Enable or disable trail delta computation.
    pub fn set_compute_trail_delta(&mut self, enabled: bool) {
        if enabled && self.trail_delta.is_none() {
            let size = self.width() * self.height();
            self.trail_delta = Some(vec![0.0; size]);
            self.prev_trail = Some(vec![0.0; size]);
        }
    }

    /// Get the trail age buffer (normalized cumulative seconds above threshold).
    pub fn trail_age(&self) -> Option<&[f32]> {
        self.trail_age.as_deref()
    }

    /// Get the trail delta buffer (absolute change normalized by peak).
    pub fn trail_delta(&self) -> Option<&[f32]> {
        self.trail_delta.as_deref()
    }

    /// Enable or disable gradient magnitude computation.
    pub fn set_compute_gradient_magnitude(&mut self, enabled: bool) {
        if enabled && self.gradient_magnitude.is_none() {
            self.gradient_magnitude = Some(vec![0.0; self.width() * self.height()]);
        }
    }

    /// Get the gradient magnitude buffer (normalized edge detection values).
    pub fn gradient_magnitude(&self) -> Option<&[f32]> {
        self.gradient_magnitude.as_deref()
    }

    /// Add a temporary attractor at the given coordinates.
    pub fn add_mouse_attractor(&mut self, x: f32, y: f32, strength: f32) {
        self.config.add_mouse_attractor(x, y, strength);
    }

    /// Generate attractors based on a food image.
    ///
    /// Creates point attractors at bright locations in the image.
    pub fn create_food_attractors(
        width: usize,
        height: usize,
        food_path: &str,
        food_invert: bool,
        food_scale: f32,
        strength: f32,
        brightness_threshold: f32,
    ) -> Vec<crate::simulation::config::Attractor> {
        // Use embedded default food image for standard initialization,
        // or load from custom path if user specified one.
        let brightness_map = if food_path == "assets/tslime_logo.png" {
            match load_default_food_image(width, height, food_invert, food_scale) {
                Ok(map) => map,
                Err(_) => return Vec::new(),
            }
        } else {
            match load_image_grayscale(food_path, width, height, food_invert, food_scale) {
                Ok(map) => map,
                Err(_) => return Vec::new(),
            }
        };

        let mut attractors = Vec::new();

        // Sample at lower resolution to avoid too many attractors
        let step_size = 5; // Sample every 5 pixels

        for y in (0..height).step_by(step_size) {
            for x in (0..width).step_by(step_size) {
                let brightness = get_brightness_at(&brightness_map, width, x, y);
                if brightness > brightness_threshold {
                    attractors.push(crate::simulation::config::Attractor::new(
                        x as f32,
                        y as f32,
                        strength * brightness, // Scale strength by brightness
                    ));
                }
            }
        }

        attractors
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::render::palette::RgbColor;
    use crate::simulation::config::SpeciesConfig;

    #[test]
    fn test_simulation_creation() {
        let config = SimConfig::default();
        let sim = Simulation::new(400, 400, config, 42, InitMode::Random, 0);
        assert_eq!(sim.width(), 400);
        assert_eq!(sim.height(), 400);
        assert_eq!(sim.agents().len(), 50000);
    }

    #[test]
    fn test_multi_species_creation() {
        let config = SimConfig {
            species_configs: vec![
                SpeciesConfig {
                    name: "red".to_string(),
                    count: 10000,
                    color: RgbColor::from_hex(0xff0000),
                    ..Default::default()
                },
                SpeciesConfig {
                    name: "blue".to_string(),
                    count: 20000,
                    color: RgbColor::from_hex(0x0000ff),
                    ..Default::default()
                },
            ],
            ..Default::default()
        };
        let sim = Simulation::new(400, 400, config, 42, InitMode::Random, 0);
        assert_eq!(sim.agents().len(), 30000);
    }

    #[test]
    fn test_update_changes_trail() {
        let config = SimConfig {
            species_configs: vec![SpeciesConfig {
                count: 100,
                ..Default::default()
            }],
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
            species_configs: vec![SpeciesConfig {
                count: 100,
                ..Default::default()
            }],
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

        assert!(
            max_after_2 < max_after_1 * 1.5,
            "max_after_2 ({}) should be less than max_after_1 * 1.5 ({})",
            max_after_2,
            max_after_1 * 1.5
        );
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
        let history = TrailHistory::new(5, 100);
        assert_eq!(history.capacity(), 5);
        assert_eq!(history.count(), 0);
    }

    #[test]
    fn test_trail_history_push_and_blend() {
        let mut history = TrailHistory::new(3, 4);

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
        let mut history = TrailHistory::new(2, 2);

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
        let mut history = TrailHistory::new(3, 4);
        assert!(history.blended().is_none());
    }

    #[test]
    fn test_trail_history_clear() {
        let mut history = TrailHistory::new(3, 2);

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
        let mut sim = Simulation::new(100, 100, config, 42, InitMode::Random, 0);

        let mut blended = Vec::new();
        sim.trail_map_blended(&mut blended);
        let current = sim.trail_map().current().to_vec();
        assert_eq!(blended, current);
    }

    #[test]
    fn test_separate_trails_single_species() {
        let config = SimConfig {
            species_configs: vec![SpeciesConfig {
                count: 100,
                ..Default::default()
            }],
            separate_species_trails: true,
            ..Default::default()
        };
        let sim = Simulation::new(100, 100, config, 42, InitMode::Random, 0);
        assert_eq!(sim.trail_maps().len(), 1);
    }

    #[test]
    fn test_separate_trails_multiple_species() {
        let config = SimConfig {
            species_configs: vec![
                SpeciesConfig {
                    count: 100,
                    ..Default::default()
                },
                SpeciesConfig {
                    count: 100,
                    name: "blue".to_string(),
                    color: RgbColor::from_hex(0x0000ff),
                    ..Default::default()
                },
            ],
            separate_species_trails: true,
            ..Default::default()
        };
        let sim = Simulation::new(100, 100, config, 42, InitMode::Random, 0);
        assert_eq!(sim.trail_maps().len(), 2);
    }

    #[test]
    fn test_separate_trails_all_species_visible() {
        let config = SimConfig {
            species_configs: vec![
                SpeciesConfig {
                    name: "red".to_string(),
                    count: 500,
                    color: RgbColor::from_hex(0xff0000),
                    sensor_angle: 22.5,
                    rotation_angle: 45.0,
                    step_size: 1.0,
                    deposit_amount: 5.0,
                    trail_modulation: None,
                },
                SpeciesConfig {
                    name: "blue".to_string(),
                    count: 500,
                    color: RgbColor::from_hex(0x0000ff),
                    sensor_angle: 22.5,
                    rotation_angle: 45.0,
                    step_size: 1.0,
                    deposit_amount: 5.0,
                    trail_modulation: None,
                },
            ],
            separate_species_trails: true,
            decay_factor: 0.99,
            ..Default::default()
        };

        let mut sim = Simulation::new(100, 100, config, 42, InitMode::CentralBurst, 0);

        for _ in 0..50 {
            sim.update(1.0);
        }

        let mut blended = Vec::new();
        sim.trail_map_blended(&mut blended);
        let trail_map_sum: f32 = blended.iter().sum();
        assert!(
            trail_map_sum > 100.0,
            "Combined trail map should have significant values when all species have agents, got sum: {}",
            trail_map_sum
        );

        assert_eq!(sim.trail_maps().len(), 2);
        let red_trail_sum: f32 = sim.trail_maps()[0].current().iter().sum();
        let blue_trail_sum: f32 = sim.trail_maps()[1].current().iter().sum();
        assert!(
            red_trail_sum > 50.0,
            "Red species trail (index 0) should have significant values, got sum: {}",
            red_trail_sum
        );
        assert!(
            blue_trail_sum > 50.0,
            "Blue species trail (index 1) should have significant values, got sum: {}",
            blue_trail_sum
        );
    }

    #[test]
    #[allow(clippy::needless_update)]
    fn test_separate_trails_second_species_only() {
        let config = SimConfig {
            species_configs: vec![
                SpeciesConfig {
                    name: "red".to_string(),
                    count: 0,
                    color: RgbColor::from_hex(0xff0000),
                    sensor_angle: 22.5,
                    rotation_angle: 45.0,
                    step_size: 1.0,
                    deposit_amount: 5.0,
                    trail_modulation: None,
                },
                SpeciesConfig {
                    name: "blue".to_string(),
                    count: 500,
                    color: RgbColor::from_hex(0x0000ff),
                    sensor_angle: 22.5,
                    rotation_angle: 45.0,
                    step_size: 1.0,
                    deposit_amount: 5.0,
                    trail_modulation: None,
                },
            ],
            separate_species_trails: true,
            decay_factor: 0.99,
            ..Default::default()
        };

        let mut sim = Simulation::new(100, 100, config, 42, InitMode::CentralBurst, 0);

        for _ in 0..50 {
            sim.update(1.0);
        }

        let mut blended = Vec::new();
        sim.trail_map_blended(&mut blended);
        let blended_sum: f32 = blended.iter().sum();
        assert!(
            blended_sum > 50.0,
            "trail_map_blended() should include second species' trail when first species has 0 agents. Got sum: {}",
            blended_sum
        );

        let red_trail_sum: f32 = sim.trail_maps()[0].current().iter().sum();
        let blue_trail_sum: f32 = sim.trail_maps()[1].current().iter().sum();
        assert!(
            red_trail_sum < 1.0,
            "Red species trail (index 0, 0 agents) should have minimal values, got sum: {}",
            red_trail_sum
        );
        assert!(
            blue_trail_sum > 50.0,
            "Blue species trail (index 1, 500 agents) should have significant values, got sum: {}",
            blue_trail_sum
        );
    }

    #[test]
    fn test_simulation_reset() {
        let config = SimConfig::default();
        let mut sim = Simulation::new(100, 100, config, 42, InitMode::Random, 0);
        sim.update(1.0);
        let sum_before = sim.trail_map().current().iter().sum::<f32>();
        assert!(sum_before > 0.0);

        sim.reset(123, InitMode::CentralBurst);
        assert_eq!(sim.trail_map().current().iter().sum::<f32>(), 0.0);
        assert_eq!(sim.agent_count(), 50000);
    }

    #[test]
    fn test_update_config() {
        let config = SimConfig::default();
        let mut sim = Simulation::new(100, 100, config.clone(), 42, InitMode::Random, 0);

        let mut new_config = config;
        new_config.separate_species_trails = true;
        new_config.species_configs.push(SpeciesConfig {
            name: "blue".to_string(),
            count: 100,
            color: RgbColor::from_hex(0x0000ff),
            ..Default::default()
        });

        sim.update_config(new_config);
        assert_eq!(sim.trail_maps().len(), 2);
    }

    #[test]
    fn test_add_mouse_attractor() {
        let config = SimConfig::default();
        let mut sim = Simulation::new(100, 100, config, 42, InitMode::Random, 0);
        sim.add_mouse_attractor(50.0, 50.0, 5.0);
        assert_eq!(sim.attractor_count(), 1);
    }

    #[test]
    fn test_create_food_attractors_empty() {
        let attractors =
            Simulation::create_food_attractors(100, 100, "nonexistent.png", false, 1.0, 1.0, 0.5);
        assert!(attractors.is_empty());
    }

    #[test]
    fn test_init_modes() {
        let modes = [
            InitMode::Random,
            InitMode::CentralBurst,
            InitMode::Circle,
            InitMode::Gradient,
            InitMode::WaveFront,
            InitMode::Spiral,
            InitMode::RandomClusters,
            InitMode::Food, // Falls back to random if no path
        ];

        for mode in modes {
            let config = SimConfig {
                species_configs: vec![SpeciesConfig {
                    count: 100,
                    ..Default::default()
                }],
                // Override food path to None so Food mode falls back to random,
                // ensuring exactly 100 agents regardless of embedded image coverage.
                food_image_path: None,
                ..Default::default()
            };
            let sim = Simulation::new(100, 100, config, 42, mode, 0);
            assert_eq!(sim.agent_count(), 100, "Failed for mode {:?}", mode);
        }
    }

    #[test]
    fn test_counts() {
        let config = SimConfig {
            attractors: vec![crate::simulation::config::Attractor::new(1.0, 1.0, 1.0)],
            obstacles: vec![crate::simulation::config::Obstacle::Circle {
                x: 1.0,
                y: 1.0,
                radius: 1.0,
            }],
            ..Default::default()
        };
        let sim = Simulation::new(100, 100, config, 42, InitMode::Random, 0);
        assert_eq!(sim.attractor_count(), 1);
        assert_eq!(sim.obstacle_count(), 1);
        assert_eq!(sim.species_count(), 1);
    }

    #[test]
    fn test_init_from_food_fallback() {
        // Test fallback when image path is missing
        let mut rng = rand_xoshiro::Xoshiro256PlusPlus::seed_from_u64(42);
        let mut agents = Vec::new();
        let population = 100;
        let species_id = 0;
        let width = 100;
        let height = 100;

        Simulation::init_from_food(
            &mut rng,
            width,
            height,
            &mut agents,
            population,
            species_id,
            "nonexistent.png",
            false,
            1.0,
        );

        assert_eq!(agents.len(), population);
        // Verify agents are distributed randomly (not all at 0,0 or something)
        let unique_positions = agents
            .iter()
            .map(|a| ((a.x as i32), (a.y as i32)))
            .collect::<std::collections::HashSet<_>>();
        assert!(
            unique_positions.len() > 50,
            "Agents should be randomly distributed"
        );
    }

    #[test]
    fn test_gradient_magnitude_computation() {
        let config = SimConfig {
            species_configs: vec![SpeciesConfig {
                count: 100,
                ..Default::default()
            }],
            ..Default::default()
        };
        let mut sim = Simulation::new(100, 100, config, 42, InitMode::Random, 0);

        // Gradient should be None initially
        assert!(sim.gradient_magnitude().is_none());

        // Enable gradient computation
        sim.set_compute_gradient_magnitude(true);
        assert!(sim.gradient_magnitude().is_some());

        // Run a few updates to generate trails
        for _ in 0..10 {
            sim.update(1.0);
        }

        // Gradient should now have values
        let gradient = sim.gradient_magnitude().unwrap();
        assert_eq!(gradient.len(), 100 * 100);

        // Gradient should be normalized to [0, 1]
        let max_val = gradient.iter().copied().fold(0.0f32, f32::max);
        assert!(
            max_val <= 1.0,
            "Gradient should be normalized, max was {}",
            max_val
        );

        // With trails present, there should be some non-zero gradient values
        let non_zero_count = gradient.iter().filter(|&&g| g > 0.001).count();
        assert!(
            non_zero_count > 0,
            "There should be some non-zero gradient values with active trails"
        );
    }

    #[test]
    fn test_gradient_magnitude_disable() {
        let config = SimConfig::default();
        let mut sim = Simulation::new(100, 100, config, 42, InitMode::Random, 0);

        // Enable then disable
        sim.set_compute_gradient_magnitude(true);
        assert!(sim.gradient_magnitude().is_some());

        // After reset, gradient should be cleared
        sim.reset(123, InitMode::Random);
        if let Some(gradient) = sim.gradient_magnitude() {
            let sum: f32 = gradient.iter().sum();
            assert_eq!(sum, 0.0, "Gradient should be cleared after reset");
        }
    }
}
