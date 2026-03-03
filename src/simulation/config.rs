//! Simulation configuration types and presets.
//!
//! This module defines all the configuration parameters for the Physarum simulation,
//! including presets, diffusion kernels, initialization modes, and environmental effects.

use image::io::Reader as ImageReader;
use std::path::Path;

use super::agent::normalize_angle;
use super::constants::agent as agent_consts;
use super::constants::env as env_consts;
use super::constants::food_image as food_img_consts;
use super::constants::population as pop_consts;
use super::constants::time as time_consts;
use super::constants::trail as trail_consts;
use crate::render::color_constants::{default, presets, ui};

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

    /// Validates wind parameters.
    pub fn validate(&self) -> Result<(), String> {
        if self.dx < -1.0 || self.dx > 1.0 {
            return Err(format!(
                "wind.dx must be between -1.0 and 1.0, got {}",
                self.dx
            ));
        }
        if self.dy < -1.0 || self.dy > 1.0 {
            return Err(format!(
                "wind.dy must be between -1.0 and 1.0, got {}",
                self.dy
            ));
        }
        if self.dx.abs() < 0.001 && self.dy.abs() < 0.001 {
            return Err("wind cannot be zero vector".to_string());
        }
        Ok(())
    }
}

impl Default for Wind {
    fn default() -> Self {
        Self { dx: 0.0, dy: 0.0 }
    }
}

impl std::str::FromStr for Wind {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
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
        wind.validate()?;
        Ok(wind)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
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
    /// Color hex code.
    pub color: String,
}

impl Default for SpeciesConfig {
    fn default() -> Self {
        Self {
            name: "default".to_string(),
            count: pop_consts::DEFAULT_COUNT,
            sensor_angle: agent_consts::DEFAULT_SENSOR_ANGLE,
            rotation_angle: agent_consts::DEFAULT_ROTATION_ANGLE,
            step_size: agent_consts::DEFAULT_STEP_SIZE,
            deposit_amount: agent_consts::DEFAULT_DEPOSIT_AMOUNT,
            color: default::FOREST_GREEN.to_string(),
        }
    }
}

impl SpeciesConfig {
    /// Validates species configuration.
    pub fn validate(&self) -> Result<(), String> {
        if self.count < pop_consts::MIN_SPECIES_COUNT || self.count > pop_consts::MAX_SPECIES_COUNT
        {
            return Err(format!(
                "species '{}' count must be between {} and {}, got {}",
                self.name,
                pop_consts::MIN_SPECIES_COUNT,
                pop_consts::MAX_SPECIES_COUNT,
                self.count
            ));
        }
        if self.sensor_angle < 5.0 || self.sensor_angle > 90.0 {
            return Err(format!(
                "species '{}' sensor_angle must be between 5.0 and 90.0, got {}",
                self.name, self.sensor_angle
            ));
        }
        if self.rotation_angle < 5.0 || self.rotation_angle > 90.0 {
            return Err(format!(
                "species '{}' rotation_angle must be between 5.0 and 90.0, got {}",
                self.name, self.rotation_angle
            ));
        }
        if self.step_size < 0.01 || self.step_size > 5.0 {
            return Err(format!(
                "species '{}' step_size must be between 0.01 and 5.0, got {}",
                self.name, self.step_size
            ));
        }
        if self.deposit_amount < 0.1 || self.deposit_amount > 20.0 {
            return Err(format!(
                "species '{}' deposit_amount must be between 0.1 and 20.0, got {}",
                self.name, self.deposit_amount
            ));
        }
        Ok(())
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
}

impl SimConfig {
    /// Validates the simulation configuration.
    pub fn validate(&self) -> Result<(), String> {
        if self.species_configs.is_empty() {
            return Err("at least one species must be configured".to_string());
        }
        let total_pop: usize = self.species_configs.iter().map(|s| s.count).sum();
        if !(pop_consts::MIN_TOTAL..=pop_consts::MAX_TOTAL).contains(&total_pop) {
            return Err(format!(
                "total population must be between {} and {}, got {}",
                pop_consts::MIN_TOTAL,
                pop_consts::MAX_TOTAL,
                total_pop
            ));
        }
        if self.sensor_angle < agent_consts::MIN_SENSOR_ANGLE
            || self.sensor_angle > agent_consts::MAX_SENSOR_ANGLE
        {
            return Err(format!(
                "sensor_angle must be between {} and {}, got {}",
                agent_consts::MIN_SENSOR_ANGLE,
                agent_consts::MAX_SENSOR_ANGLE,
                self.sensor_angle
            ));
        }
        if self.sensor_distance < agent_consts::MIN_SENSOR_DISTANCE
            || self.sensor_distance > agent_consts::MAX_SENSOR_DISTANCE
        {
            return Err(format!(
                "sensor_distance must be between {} and {}, got {}",
                agent_consts::MIN_SENSOR_DISTANCE,
                agent_consts::MAX_SENSOR_DISTANCE,
                self.sensor_distance
            ));
        }
        if self.rotation_angle < agent_consts::MIN_ROTATION_ANGLE
            || self.rotation_angle > agent_consts::MAX_ROTATION_ANGLE
        {
            return Err(format!(
                "rotation_angle must be between {} and {}, got {}",
                agent_consts::MIN_ROTATION_ANGLE,
                agent_consts::MAX_ROTATION_ANGLE,
                self.rotation_angle
            ));
        }
        if self.step_size < agent_consts::MIN_STEP_SIZE
            || self.step_size > agent_consts::MAX_STEP_SIZE
        {
            return Err(format!(
                "step_size must be between {} and {}, got {}",
                agent_consts::MIN_STEP_SIZE,
                agent_consts::MAX_STEP_SIZE,
                self.step_size
            ));
        }
        if self.decay_factor < trail_consts::MIN_DECAY_FACTOR
            || self.decay_factor > trail_consts::MAX_DECAY_FACTOR
        {
            return Err(format!(
                "decay_factor must be between {} and {}, got {}",
                trail_consts::MIN_DECAY_FACTOR,
                trail_consts::MAX_DECAY_FACTOR,
                self.decay_factor
            ));
        }
        if self.deposit_amount < agent_consts::MIN_DEPOSIT_AMOUNT
            || self.deposit_amount > agent_consts::MAX_DEPOSIT_AMOUNT
        {
            return Err(format!(
                "deposit_amount must be between {} and {}, got {}",
                agent_consts::MIN_DEPOSIT_AMOUNT,
                agent_consts::MAX_DEPOSIT_AMOUNT,
                self.deposit_amount
            ));
        }
        if self.max_brightness < trail_consts::MIN_MAX_BRIGHTNESS
            || self.max_brightness > trail_consts::MAX_MAX_BRIGHTNESS
        {
            return Err(format!(
                "max_brightness must be between {} and {}, got {}",
                trail_consts::MIN_MAX_BRIGHTNESS,
                trail_consts::MAX_MAX_BRIGHTNESS,
                self.max_brightness
            ));
        }
        if self.diffusion_sigma < trail_consts::MIN_DIFFUSION_SIGMA
            || self.diffusion_sigma > trail_consts::MAX_DIFFUSION_SIGMA
        {
            return Err(format!(
                "diffusion_sigma must be between {} and {}, got {}",
                trail_consts::MIN_DIFFUSION_SIGMA,
                trail_consts::MAX_DIFFUSION_SIGMA,
                self.diffusion_sigma
            ));
        }
        if self.time_scale < time_consts::MIN_TIME_SCALE
            || self.time_scale > time_consts::MAX_TIME_SCALE
        {
            return Err(format!(
                "time_scale must be between {} and {}, got {}",
                time_consts::MIN_TIME_SCALE,
                time_consts::MAX_TIME_SCALE,
                self.time_scale
            ));
        }
        if self.attractor_strength < env_consts::MIN_ATTRACTOR_STRENGTH
            || self.attractor_strength > env_consts::MAX_ATTRACTOR_STRENGTH
        {
            return Err(format!(
                "attractor_strength must be between {} and {}, got {}",
                env_consts::MIN_ATTRACTOR_STRENGTH,
                env_consts::MAX_ATTRACTOR_STRENGTH,
                self.attractor_strength
            ));
        }
        for (i, attractor) in self.attractors.iter().enumerate() {
            if attractor.strength < env_consts::ATTRACTOR_STRENGTH_MIN
                || attractor.strength > env_consts::ATTRACTOR_STRENGTH_MAX
            {
                return Err(format!(
                    "attractor[{}].strength must be between {} and {}, got {}",
                    i,
                    env_consts::ATTRACTOR_STRENGTH_MIN,
                    env_consts::ATTRACTOR_STRENGTH_MAX,
                    attractor.strength
                ));
            }
        }
        for species in &self.species_configs {
            species.validate()?;
        }
        if self.terrain_strength < env_consts::MIN_TERRAIN_STRENGTH
            || self.terrain_strength > env_consts::MAX_TERRAIN_STRENGTH
        {
            return Err(format!(
                "terrain_strength must be between {} and {}, got {}",
                env_consts::MIN_TERRAIN_STRENGTH,
                env_consts::MAX_TERRAIN_STRENGTH,
                self.terrain_strength
            ));
        }
        if let Some(ref wind) = self.wind {
            wind.validate()?;
        }
        Ok(())
    }

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
    pub fn effective_attractors(&self) -> Vec<Attractor> {
        let mut result = self.attractors.clone();
        for ma in &self.mouse_attractors {
            result.push(Attractor::new(ma.x, ma.y, ma.strength));
        }
        result
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
            food_image_path: Some(food_img_consts::DEFAULT_PATH.to_string()),
            food_image_invert: food_img_consts::DEFAULT_INVERT,
            food_image_scale: food_img_consts::DEFAULT_SCALE,
            obstacles: Vec::new(),
            obstacle_masks: Vec::new(),
            wind: None,
            terrain: TerrainType::None,
            terrain_strength: env_consts::DEFAULT_TERRAIN_STRENGTH,
            background_color: None,
            preferred_init_mode: Some(InitMode::Food),
        }
    }
}

impl SimConfig {
    /// Creates a base preset configuration with common fields set to their defaults.
    fn base_preset() -> Self {
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
            food_image_path: None,
            food_image_invert: false,
            food_image_scale: 1.0,
            obstacles: Vec::new(),
            obstacle_masks: Vec::new(),
            wind: None,
            terrain: TerrainType::None,
            terrain_strength: env_consts::DEFAULT_TERRAIN_STRENGTH,
            background_color: None,
            preferred_init_mode: None,
        }
    }
}

impl From<Preset> for SimConfig {
    fn from(preset: Preset) -> Self {
        match preset {
            Preset::Network => Self {
                sensor_angle: 15.0,
                sensor_distance: 9.0,
                rotation_angle: 30.0,
                decay_factor: 0.85,
                diffusion_kernel: DiffusionKernel::Mean3x3,
                max_brightness: 20.0,
                species_configs: vec![SpeciesConfig {
                    name: "default".to_string(),
                    count: 50_000,
                    sensor_angle: 15.0,
                    rotation_angle: 30.0,
                    step_size: 1.0,
                    deposit_amount: 5.0,
                    color: "228b22".to_string(),
                }],
                ..Self::base_preset()
            },
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
                    step_size: 1.0,
                    deposit_amount: 3.0,
                    color: "228b22".to_string(),
                }],
                ..Self::base_preset()
            },
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
                    color: "228b22".to_string(),
                }],
                ..Self::base_preset()
            },
            Preset::Organic => Self {
                sensor_angle: 22.5,
                sensor_distance: 9.0,
                rotation_angle: 45.0,
                step_size: 1.0,
                decay_factor: 0.85,
                diffusion_kernel: DiffusionKernel::Mean3x3,
                max_brightness: 20.0,
                ..Self::base_preset()
            },
            Preset::Minimal => Self {
                sensor_angle: 30.0,
                sensor_distance: 9.0,
                rotation_angle: 30.0,
                step_size: 0.8,
                decay_factor: 0.95,
                deposit_amount: 3.0,
                diffusion_kernel: DiffusionKernel::Mean3x3,
                max_brightness: 15.0,
                species_configs: vec![SpeciesConfig {
                    name: "default".to_string(),
                    count: 15_000,
                    sensor_angle: 30.0,
                    rotation_angle: 30.0,
                    step_size: 0.8,
                    deposit_amount: 3.0,
                    color: "228b22".to_string(),
                }],
                ..Self::base_preset()
            },
            Preset::Moss => Self {
                sensor_angle: 22.0,
                sensor_distance: 12.0,
                rotation_angle: 35.0,
                decay_factor: 0.88,
                deposit_amount: 4.0,
                diffusion_kernel: DiffusionKernel::Mean3x3,
                max_brightness: 18.0,
                species_configs: vec![SpeciesConfig {
                    name: "default".to_string(),
                    count: 35_000,
                    sensor_angle: 22.0,
                    rotation_angle: 35.0,
                    step_size: 1.0,
                    deposit_amount: 4.0,
                    color: "4a7a4a".to_string(),
                }],
                ..Self::base_preset()
            },
            Preset::Cosmic => Self {
                sensor_angle: 55.0,
                sensor_distance: 15.0,
                rotation_angle: 45.0,
                step_size: 0.7,
                decay_factor: 0.93,
                deposit_amount: 3.0,
                max_brightness: 14.0,
                species_configs: vec![SpeciesConfig {
                    name: "default".to_string(),
                    count: 25_000,
                    sensor_angle: 55.0,
                    rotation_angle: 45.0,
                    step_size: 0.7,
                    deposit_amount: 3.0,
                    color: "8a2be2".to_string(),
                }],
                ..Self::base_preset()
            },
            Preset::Fire => Self {
                sensor_angle: 15.0,
                rotation_angle: 30.0,
                step_size: 1.5,
                decay_factor: 0.85,
                diffusion_kernel: DiffusionKernel::Mean3x3,
                max_brightness: 20.0,
                species_configs: vec![SpeciesConfig {
                    name: "default".to_string(),
                    count: 100_000,
                    sensor_angle: 15.0,
                    rotation_angle: 30.0,
                    step_size: 1.5,
                    deposit_amount: 5.0,
                    color: presets::FIRE_ORANGE.to_string(),
                }],
                ..Self::base_preset()
            },
            Preset::Zen => Self {
                sensor_distance: 12.0,
                sensor_angle: 25.0,
                rotation_angle: 30.0,
                step_size: 0.5,
                decay_factor: 0.94,
                deposit_amount: 2.0,
                diffusion_kernel: DiffusionKernel::Mean3x3,
                max_brightness: 12.0,
                species_configs: vec![SpeciesConfig {
                    name: "default".to_string(),
                    count: 10_000,
                    sensor_angle: 25.0,
                    rotation_angle: 30.0,
                    step_size: 0.5,
                    deposit_amount: 2.0,
                    color: presets::ZEN_WHITE.to_string(),
                }],
                ..Self::base_preset()
            },
            Preset::Storm => Self {
                sensor_angle: 20.0,
                rotation_angle: 60.0,
                step_size: 2.0,
                decay_factor: 0.80,
                diffusion_kernel: DiffusionKernel::Mean3x3,
                max_brightness: 18.0,
                species_configs: vec![SpeciesConfig {
                    name: "default".to_string(),
                    count: 80_000,
                    sensor_angle: 20.0,
                    rotation_angle: 60.0,
                    step_size: 2.0,
                    deposit_amount: 5.0,
                    color: presets::STORM_BLUE.to_string(),
                }],
                wind: Some(Wind::new(0.1, 0.05)),
                ..Self::base_preset()
            },
            Preset::River => Self {
                sensor_angle: 25.0,
                step_size: 1.2,
                decay_factor: 0.90,
                diffusion_kernel: DiffusionKernel::Mean3x3,
                max_brightness: 18.0,
                species_configs: vec![SpeciesConfig {
                    name: "default".to_string(),
                    count: 45_000,
                    sensor_angle: 25.0,
                    rotation_angle: 45.0,
                    step_size: 1.2,
                    deposit_amount: 5.0,
                    color: presets::RIVER_BLUE.to_string(),
                }],
                wind: Some(Wind::new(0.3, 0.0)),
                ..Self::base_preset()
            },
            Preset::Ethereal => Self {
                sensor_angle: 40.0,
                step_size: 0.7,
                decay_factor: 0.98,
                deposit_amount: 2.0,
                max_brightness: 12.0,
                species_configs: vec![SpeciesConfig {
                    name: "default".to_string(),
                    count: 25_000,
                    sensor_angle: 40.0,
                    rotation_angle: 45.0,
                    step_size: 0.7,
                    deposit_amount: 2.0,
                    color: presets::ETHEREAL_LAVENDER.to_string(),
                }],
                ..Self::base_preset()
            },
            Preset::PetriDish => Self {
                sensor_angle: 45.0,
                rotation_angle: 20.0,
                step_size: 0.05,
                decay_factor: 0.999,
                deposit_amount: 0.2,
                max_brightness: 50.0,
                species_configs: vec![SpeciesConfig {
                    name: "mold".to_string(),
                    count: 20_000,
                    sensor_angle: 45.0,
                    rotation_angle: 20.0,
                    step_size: 0.05,
                    deposit_amount: 0.2,
                    color: presets::MOLD_YELLOW.to_string(),
                }],
                obstacles: vec![Obstacle::Circle {
                    x: 200.0,
                    y: 100.0,
                    radius: 90.0,
                }],
                background_color: Some(ui::BLACK.to_string()),
                preferred_init_mode: Some(InitMode::Petri),
                ..Self::base_preset()
            },
            // Empirically-derived presets from parameter space optimization
            Preset::Vortex => Self {
                // Optimized for high angular momentum (swirling patterns)
                // rotation_angle > sensor_angle causes oscillation/spiraling
                sensor_angle: 25.2,
                sensor_distance: 3.9, // Short-sighted creates local vortices
                rotation_angle: 46.4, // ~2x sensor_angle for strong spiraling
                step_size: 1.92,
                decay_factor: 0.96, // High persistence keeps vortices visible
                deposit_amount: 4.3,
                diffusion_kernel: DiffusionKernel::Mean3x3,
                species_configs: vec![SpeciesConfig {
                    name: "default".to_string(),
                    count: 32_000,
                    sensor_angle: 25.2,
                    rotation_angle: 46.4,
                    step_size: 1.92,
                    deposit_amount: 4.3,
                    color: presets::VORTEX_PURPLE.to_string(),
                }],
                ..Self::base_preset()
            },
            Preset::Lightning => Self {
                // Optimized for high branching factor with sparse coverage
                // Fast movement + high deposit + medium decay = branching dendrites
                sensor_angle: 31.9,
                sensor_distance: 23.2, // Long-range sensing for coherent branches
                rotation_angle: 39.3,
                step_size: 2.48,      // Fast movement
                decay_factor: 0.82,   // Medium decay for visible branches
                deposit_amount: 20.0, // Max intensity trails
                diffusion_kernel: DiffusionKernel::Mean3x3,
                max_brightness: 40.0,
                species_configs: vec![SpeciesConfig {
                    name: "default".to_string(),
                    count: 7_000, // Very low population for distinct branches
                    sensor_angle: 31.9,
                    rotation_angle: 39.3,
                    step_size: 2.48,
                    deposit_amount: 20.0,
                    color: presets::LIGHTNING_CYAN.to_string(),
                }],
                ..Self::base_preset()
            },
            Preset::Crystal => Self {
                // Optimized for high temporal stability (persistent structures)
                // rotation_angle < sensor_angle for smooth, stable turns
                sensor_angle: 38.9,
                sensor_distance: 30.6, // Long-range sensing for coherent growth
                rotation_angle: 21.5,  // Smaller than sensor for smooth turns
                step_size: 1.47,
                decay_factor: 0.50, // Fast decay creates sharp edges
                deposit_amount: 2.1,
                diffusion_sigma: 1.2,
                species_configs: vec![SpeciesConfig {
                    name: "default".to_string(),
                    count: 38_000,
                    sensor_angle: 38.9,
                    rotation_angle: 21.5,
                    step_size: 1.47,
                    deposit_amount: 2.1,
                    color: presets::CRYSTAL_ICE.to_string(),
                }],
                ..Self::base_preset()
            },
            Preset::ChaosEdge => Self {
                // Optimized for high heading variance × density variance (chaotic dynamics)
                // Very narrow sensor + large rotation creates unpredictable behavior
                sensor_angle: 5.0, // Minimum - very narrow field of view
                sensor_distance: 26.4,
                rotation_angle: 56.2, // Large turns amplify chaos
                step_size: 0.58,      // Slow movement for intricate patterns
                decay_factor: 0.99,   // Max persistence preserves chaotic trails
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
                    color: presets::CHAOS_RED.to_string(),
                }],
                ..Self::base_preset()
            },
            Preset::Blob => Self {
                // Optimized for high fragmentation (isolated clusters)
                // Very short sensor + max rotation + fast decay = clumping
                sensor_angle: 72.1,
                sensor_distance: 2.1, // Extremely short-sighted
                rotation_angle: 90.0, // Maximum sharp turns
                step_size: 0.92,
                decay_factor: 0.50, // Fast decay isolates clusters
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
                    color: presets::BLOB_LIME.to_string(),
                }],
                ..Self::base_preset()
            },
            Preset::Worm => Self {
                // Optimized for high elongation (long snaking trails)
                // Max sensor distance + low rotation + low population
                sensor_angle: 38.8,
                sensor_distance: 50.0, // Maximum long-range sensing
                rotation_angle: 13.4,  // Very gradual turns
                step_size: 1.96,
                decay_factor: 0.65, // Medium decay for visible trails
                deposit_amount: 6.3,
                diffusion_kernel: DiffusionKernel::Mean3x3,
                species_configs: vec![SpeciesConfig {
                    name: "default".to_string(),
                    count: 6_000, // Very low population for distinct worms
                    sensor_angle: 38.8,
                    rotation_angle: 13.4,
                    step_size: 1.96,
                    deposit_amount: 6.3,
                    color: presets::WORM_GOLD.to_string(),
                }],
                ..Self::base_preset()
            },
        }
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
                    color: "ff0000".to_string(),
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
}
