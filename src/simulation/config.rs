use image::io::Reader as ImageReader;
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiffusionKernel {
    Mean3x3,
    Gaussian,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Preset {
    Network,
    Exploratory,
    Tendrils,
    Organic,
    Minimal,
    Moss,
    Cosmic,
    Fire,
    Zen,
    Storm,
    River,
    Ethereal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InitMode {
    Random,
    CentralBurst,
    Circle,
    Gradient,
    WaveFront,
    Spiral,
    RandomClusters,
    Food,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TerrainType {
    #[default]
    None,
    Smooth,
    Turbulent,
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
pub struct Wind {
    pub dx: f32,
    pub dy: f32,
}

impl Wind {
    pub fn new(dx: f32, dy: f32) -> Self {
        Self { dx, dy }
    }

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
pub struct Attractor {
    pub x: f32,
    pub y: f32,
    pub strength: f32,
}

impl Attractor {
    pub fn new(x: f32, y: f32, strength: f32) -> Self {
        Self { x, y, strength }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MouseAttractor {
    pub x: f32,
    pub y: f32,
    pub strength: f32,
    pub created_at: std::time::Instant,
    pub timeout_seconds: f32,
}

impl MouseAttractor {
    pub fn new(x: f32, y: f32, strength: f32, timeout_seconds: f32) -> Self {
        Self {
            x,
            y,
            strength,
            created_at: std::time::Instant::now(),
            timeout_seconds,
        }
    }

    pub fn is_expired(&self) -> bool {
        self.created_at.elapsed().as_secs_f32() >= self.timeout_seconds
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ObstacleMask {
    pub pixels: Vec<f32>,
    pub width: usize,
    pub height: usize,
}

impl ObstacleMask {
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
pub enum Obstacle {
    Circle {
        x: f32,
        y: f32,
        radius: f32,
    },
    Rect {
        x: f32,
        y: f32,
        width: f32,
        height: f32,
    },
    Image {
        path: String,
        x: f32,
        y: f32,
        width: usize,
        height: usize,
        invert: bool,
        threshold: f32,
    },
}

impl Obstacle {
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

    pub fn bounce(&self, px: f32, py: f32, heading: f32, _mask: Option<&ObstacleMask>) -> f32 {
        match self {
            Obstacle::Circle { x, y, radius: _ } => {
                let dx = px - x;
                let dy = py - y;
                let normal_angle = dy.atan2(dx);
                let mut new_heading = 2.0 * normal_angle - heading + std::f32::consts::PI;
                while new_heading > std::f32::consts::PI {
                    new_heading -= 2.0 * std::f32::consts::PI;
                }
                while new_heading < -std::f32::consts::PI {
                    new_heading += 2.0 * std::f32::consts::PI;
                }
                new_heading
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
pub struct SpeciesConfig {
    pub name: String,
    pub count: usize,
    pub sensor_angle: f32,
    pub rotation_angle: f32,
    pub step_size: f32,
    pub deposit_amount: f32,
    pub color: String,
}

impl Default for SpeciesConfig {
    fn default() -> Self {
        Self {
            name: "default".to_string(),
            count: 50_000,
            sensor_angle: 22.5,
            rotation_angle: 45.0,
            step_size: 1.0,
            deposit_amount: 5.0,
            color: "228b22".to_string(),
        }
    }
}

impl SpeciesConfig {
    pub fn validate(&self) -> Result<(), String> {
        if self.count < 100 || self.count > 200_000 {
            return Err(format!(
                "species '{}' count must be between 100 and 200000, got {}",
                self.name, self.count
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
        if self.step_size < 0.5 || self.step_size > 5.0 {
            return Err(format!(
                "species '{}' step_size must be between 0.5 and 5.0, got {}",
                self.name, self.step_size
            ));
        }
        if self.deposit_amount < 1.0 || self.deposit_amount > 20.0 {
            return Err(format!(
                "species '{}' deposit_amount must be between 1.0 and 20.0, got {}",
                self.name, self.deposit_amount
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct SimConfig {
    pub sensor_angle: f32,
    pub sensor_distance: f32,
    pub rotation_angle: f32,
    pub step_size: f32,
    pub decay_factor: f32,
    pub deposit_amount: f32,
    pub diffusion_kernel: DiffusionKernel,
    pub diffusion_sigma: f32,
    pub max_brightness: f32,
    pub attractors: Vec<Attractor>,
    pub attractor_strength: f32,
    pub mouse_attractors: Vec<MouseAttractor>,
    pub mouse_timeout: f32,
    pub species_configs: Vec<SpeciesConfig>,
    pub separate_species_trails: bool,
    pub use_simd: bool,
    pub food_image_path: Option<String>,
    pub food_image_invert: bool,
    pub food_image_scale: f32,
    pub obstacles: Vec<Obstacle>,
    pub obstacle_masks: Vec<Option<ObstacleMask>>,
    pub wind: Option<Wind>,
    pub terrain: TerrainType,
    pub terrain_strength: f32,
}

impl Default for SimConfig {
    fn default() -> Self {
        Self {
            sensor_angle: 22.5,
            sensor_distance: 9.0,
            rotation_angle: 45.0,
            step_size: 1.0,
            decay_factor: 0.5,
            deposit_amount: 5.0,
            diffusion_kernel: DiffusionKernel::Mean3x3,
            diffusion_sigma: 1.0,
            max_brightness: 20.0,
            attractors: Vec::new(),
            attractor_strength: 1.0,
            mouse_attractors: Vec::new(),
            mouse_timeout: 3.0,
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
            terrain_strength: 1.0,
        }
    }
}

impl SimConfig {
    pub fn validate(&self) -> Result<(), String> {
        if self.species_configs.is_empty() {
            return Err("at least one species must be configured".to_string());
        }
        if self.species_configs.iter().map(|s| s.count).sum::<usize>() < 1000
            || self.species_configs.iter().map(|s| s.count).sum::<usize>() > 200_000
        {
            return Err(format!(
                "total population must be between 1000 and 200000, got {}",
                self.species_configs.iter().map(|s| s.count).sum::<usize>()
            ));
        }
        if self.sensor_angle < 5.0 || self.sensor_angle > 90.0 {
            return Err(format!(
                "sensor_angle must be between 5.0 and 90.0, got {}",
                self.sensor_angle
            ));
        }
        if self.sensor_distance < 1.0 || self.sensor_distance > 50.0 {
            return Err(format!(
                "sensor_distance must be between 1.0 and 50.0, got {}",
                self.sensor_distance
            ));
        }
        if self.rotation_angle < 5.0 || self.rotation_angle > 90.0 {
            return Err(format!(
                "rotation_angle must be between 5.0 and 90.0, got {}",
                self.rotation_angle
            ));
        }
        if self.step_size < 0.5 || self.step_size > 5.0 {
            return Err(format!(
                "step_size must be between 0.5 and 5.0, got {}",
                self.step_size
            ));
        }
        if self.decay_factor < 0.5 || self.decay_factor > 0.99 {
            return Err(format!(
                "decay_factor must be between 0.5 and 0.99, got {}",
                self.decay_factor
            ));
        }
        if self.deposit_amount < 1.0 || self.deposit_amount > 20.0 {
            return Err(format!(
                "deposit_amount must be between 1.0 and 20.0, got {}",
                self.deposit_amount
            ));
        }
        if self.max_brightness < 1.0 || self.max_brightness > 100.0 {
            return Err(format!(
                "max_brightness must be between 1.0 and 100.0, got {}",
                self.max_brightness
            ));
        }
        if self.diffusion_sigma < 0.5 || self.diffusion_sigma > 2.0 {
            return Err(format!(
                "diffusion_sigma must be between 0.5 and 2.0, got {}",
                self.diffusion_sigma
            ));
        }
        if self.attractor_strength < 0.1 || self.attractor_strength > 10.0 {
            return Err(format!(
                "attractor_strength must be between 0.1 and 10.0, got {}",
                self.attractor_strength
            ));
        }
        for (i, attractor) in self.attractors.iter().enumerate() {
            if attractor.strength < -10.0 || attractor.strength > 10.0 {
                return Err(format!(
                    "attractor[{}].strength must be between -10.0 and 10.0, got {}",
                    i, attractor.strength
                ));
            }
        }
        for species in &self.species_configs {
            species.validate()?;
        }
        if self.terrain_strength < 0.1 || self.terrain_strength > 5.0 {
            return Err(format!(
                "terrain_strength must be between 0.1 and 5.0, got {}",
                self.terrain_strength
            ));
        }
        if let Some(ref wind) = self.wind {
            wind.validate()?;
        }
        Ok(())
    }

    pub fn total_population(&self) -> usize {
        self.species_configs.iter().map(|s| s.count).sum()
    }

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

    pub fn add_mouse_attractor(&mut self, x: f32, y: f32, strength: f32) {
        self.mouse_attractors
            .push(MouseAttractor::new(x, y, strength, self.mouse_timeout));
    }

    pub fn remove_expired_mouse_attractors(&mut self) {
        self.mouse_attractors.retain(|ma| !ma.is_expired());
    }

    pub fn effective_attractors(&self) -> Vec<Attractor> {
        let mut result = self.attractors.clone();
        for ma in &self.mouse_attractors {
            result.push(Attractor::new(ma.x, ma.y, ma.strength));
        }
        result
    }
}

impl From<Preset> for SimConfig {
    fn from(preset: Preset) -> Self {
        match preset {
            Preset::Network => Self {
                sensor_angle: 15.0,
                sensor_distance: 9.0,
                rotation_angle: 30.0,
                step_size: 1.0,
                decay_factor: 0.85,
                deposit_amount: 5.0,
                diffusion_kernel: DiffusionKernel::Mean3x3,
                diffusion_sigma: 1.0,
                max_brightness: 20.0,
                attractors: Vec::new(),
                attractor_strength: 1.0,
                mouse_attractors: Vec::new(),
                mouse_timeout: 3.0,
                species_configs: vec![SpeciesConfig {
                    name: "default".to_string(),
                    count: 50_000,
                    sensor_angle: 15.0,
                    rotation_angle: 30.0,
                    step_size: 1.0,
                    deposit_amount: 5.0,
                    color: "228b22".to_string(),
                }],
                separate_species_trails: false,
                use_simd: true,
                food_image_path: None,
                food_image_invert: false,
                food_image_scale: 1.0,
                obstacles: Vec::new(),
                obstacle_masks: Vec::new(),
                wind: None,
                terrain: TerrainType::None,
                terrain_strength: 1.0,
            },
            Preset::Exploratory => Self {
                sensor_angle: 45.0,
                sensor_distance: 15.0,
                rotation_angle: 60.0,
                step_size: 1.0,
                decay_factor: 0.96,
                deposit_amount: 3.0,
                diffusion_kernel: DiffusionKernel::Mean3x3,
                diffusion_sigma: 1.0,
                max_brightness: 12.0,
                attractors: Vec::new(),
                attractor_strength: 1.0,
                mouse_attractors: Vec::new(),
                mouse_timeout: 3.0,
                species_configs: vec![SpeciesConfig {
                    name: "default".to_string(),
                    count: 30_000,
                    sensor_angle: 45.0,
                    rotation_angle: 60.0,
                    step_size: 1.0,
                    deposit_amount: 3.0,
                    color: "228b22".to_string(),
                }],
                separate_species_trails: false,
                use_simd: true,
                food_image_path: None,
                food_image_invert: false,
                food_image_scale: 1.0,
                obstacles: Vec::new(),
                obstacle_masks: Vec::new(),
                wind: None,
                terrain: TerrainType::None,
                terrain_strength: 1.0,
            },
            Preset::Tendrils => Self {
                sensor_angle: 30.0,
                sensor_distance: 12.0,
                rotation_angle: 45.0,
                step_size: 2.0,
                decay_factor: 0.90,
                deposit_amount: 4.0,
                diffusion_kernel: DiffusionKernel::Mean3x3,
                diffusion_sigma: 1.0,
                max_brightness: 16.0,
                attractors: Vec::new(),
                attractor_strength: 1.0,
                mouse_attractors: Vec::new(),
                mouse_timeout: 3.0,
                species_configs: vec![SpeciesConfig {
                    name: "default".to_string(),
                    count: 40_000,
                    sensor_angle: 30.0,
                    rotation_angle: 45.0,
                    step_size: 2.0,
                    deposit_amount: 4.0,
                    color: "228b22".to_string(),
                }],
                separate_species_trails: false,
                use_simd: true,
                food_image_path: None,
                food_image_invert: false,
                food_image_scale: 1.0,
                obstacles: Vec::new(),
                obstacle_masks: Vec::new(),
                wind: None,
                terrain: TerrainType::None,
                terrain_strength: 1.0,
            },
            Preset::Organic => Self {
                sensor_angle: 22.5,
                sensor_distance: 9.0,
                rotation_angle: 45.0,
                step_size: 1.0,
                decay_factor: 0.5,
                deposit_amount: 5.0,
                diffusion_kernel: DiffusionKernel::Mean3x3,
                diffusion_sigma: 1.0,
                max_brightness: 20.0,
                attractors: Vec::new(),
                attractor_strength: 1.0,
                mouse_attractors: Vec::new(),
                mouse_timeout: 3.0,
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
                terrain_strength: 1.0,
            },
            Preset::Minimal => Self {
                sensor_angle: 30.0,
                sensor_distance: 9.0,
                rotation_angle: 30.0,
                step_size: 0.8,
                decay_factor: 0.95,
                deposit_amount: 3.0,
                diffusion_kernel: DiffusionKernel::Mean3x3,
                diffusion_sigma: 1.0,
                max_brightness: 15.0,
                attractors: Vec::new(),
                attractor_strength: 1.0,
                mouse_attractors: Vec::new(),
                mouse_timeout: 3.0,
                species_configs: vec![SpeciesConfig {
                    name: "default".to_string(),
                    count: 15_000,
                    sensor_angle: 30.0,
                    rotation_angle: 30.0,
                    step_size: 0.8,
                    deposit_amount: 3.0,
                    color: "228b22".to_string(),
                }],
                separate_species_trails: false,
                use_simd: true,
                food_image_path: None,
                food_image_invert: false,
                food_image_scale: 1.0,
                obstacles: Vec::new(),
                obstacle_masks: Vec::new(),
                wind: None,
                terrain: TerrainType::None,
                terrain_strength: 1.0,
            },
            Preset::Moss => Self {
                sensor_angle: 22.0,
                sensor_distance: 12.0,
                rotation_angle: 35.0,
                step_size: 1.0,
                decay_factor: 0.88,
                deposit_amount: 4.0,
                diffusion_kernel: DiffusionKernel::Mean3x3,
                diffusion_sigma: 1.0,
                max_brightness: 18.0,
                attractors: Vec::new(),
                attractor_strength: 1.0,
                mouse_attractors: Vec::new(),
                mouse_timeout: 3.0,
                species_configs: vec![SpeciesConfig {
                    name: "default".to_string(),
                    count: 35_000,
                    sensor_angle: 22.0,
                    rotation_angle: 35.0,
                    step_size: 1.0,
                    deposit_amount: 4.0,
                    color: "4a7a4a".to_string(),
                }],
                separate_species_trails: false,
                use_simd: true,
                food_image_path: None,
                food_image_invert: false,
                food_image_scale: 1.0,
                obstacles: Vec::new(),
                obstacle_masks: Vec::new(),
                wind: None,
                terrain: TerrainType::None,
                terrain_strength: 1.0,
            },
            Preset::Cosmic => Self {
                sensor_angle: 55.0,
                sensor_distance: 15.0,
                rotation_angle: 45.0,
                step_size: 0.7,
                decay_factor: 0.93,
                deposit_amount: 3.0,
                diffusion_kernel: DiffusionKernel::Gaussian,
                diffusion_sigma: 1.0,
                max_brightness: 14.0,
                attractors: Vec::new(),
                attractor_strength: 1.0,
                mouse_attractors: Vec::new(),
                mouse_timeout: 3.0,
                species_configs: vec![SpeciesConfig {
                    name: "default".to_string(),
                    count: 25_000,
                    sensor_angle: 55.0,
                    rotation_angle: 45.0,
                    step_size: 0.7,
                    deposit_amount: 3.0,
                    color: "8a2be2".to_string(),
                }],
                separate_species_trails: false,
                use_simd: true,
                food_image_path: None,
                food_image_invert: false,
                food_image_scale: 1.0,
                obstacles: Vec::new(),
                obstacle_masks: Vec::new(),
                wind: None,
                terrain: TerrainType::None,
                terrain_strength: 1.0,
            },
            Preset::Fire => Self {
                sensor_angle: 15.0,
                sensor_distance: 9.0,
                rotation_angle: 30.0,
                step_size: 1.5,
                decay_factor: 0.85,
                deposit_amount: 5.0,
                diffusion_kernel: DiffusionKernel::Mean3x3,
                diffusion_sigma: 1.0,
                max_brightness: 20.0,
                attractors: Vec::new(),
                attractor_strength: 1.0,
                mouse_attractors: Vec::new(),
                mouse_timeout: 3.0,
                species_configs: vec![SpeciesConfig {
                    name: "default".to_string(),
                    count: 100_000,
                    sensor_angle: 15.0,
                    rotation_angle: 30.0,
                    step_size: 1.5,
                    deposit_amount: 5.0,
                    color: "ff4500".to_string(),
                }],
                separate_species_trails: false,
                use_simd: true,
                food_image_path: None,
                food_image_invert: false,
                food_image_scale: 1.0,
                obstacles: Vec::new(),
                obstacle_masks: Vec::new(),
                wind: None,
                terrain: TerrainType::None,
                terrain_strength: 1.0,
            },
            Preset::Zen => Self {
                sensor_angle: 25.0,
                sensor_distance: 12.0,
                rotation_angle: 30.0,
                step_size: 0.5,
                decay_factor: 0.94,
                deposit_amount: 2.0,
                diffusion_kernel: DiffusionKernel::Mean3x3,
                diffusion_sigma: 1.0,
                max_brightness: 12.0,
                attractors: Vec::new(),
                attractor_strength: 1.0,
                mouse_attractors: Vec::new(),
                mouse_timeout: 3.0,
                species_configs: vec![SpeciesConfig {
                    name: "default".to_string(),
                    count: 10_000,
                    sensor_angle: 25.0,
                    rotation_angle: 30.0,
                    step_size: 0.5,
                    deposit_amount: 2.0,
                    color: "ffffff".to_string(),
                }],
                separate_species_trails: false,
                use_simd: true,
                food_image_path: None,
                food_image_invert: false,
                food_image_scale: 1.0,
                obstacles: Vec::new(),
                obstacle_masks: Vec::new(),
                wind: None,
                terrain: TerrainType::None,
                terrain_strength: 1.0,
            },
            Preset::Storm => Self {
                sensor_angle: 20.0,
                sensor_distance: 9.0,
                rotation_angle: 60.0,
                step_size: 2.0,
                decay_factor: 0.80,
                deposit_amount: 5.0,
                diffusion_kernel: DiffusionKernel::Mean3x3,
                diffusion_sigma: 1.0,
                max_brightness: 18.0,
                attractors: Vec::new(),
                attractor_strength: 1.0,
                mouse_attractors: Vec::new(),
                mouse_timeout: 3.0,
                species_configs: vec![SpeciesConfig {
                    name: "default".to_string(),
                    count: 80_000,
                    sensor_angle: 20.0,
                    rotation_angle: 60.0,
                    step_size: 2.0,
                    deposit_amount: 5.0,
                    color: "4682b4".to_string(),
                }],
                separate_species_trails: false,
                use_simd: true,
                food_image_path: None,
                food_image_invert: false,
                food_image_scale: 1.0,
                obstacles: Vec::new(),
                obstacle_masks: Vec::new(),
                wind: Some(Wind::new(0.1, 0.05)),
                terrain: TerrainType::None,
                terrain_strength: 1.0,
            },
            Preset::River => Self {
                sensor_angle: 25.0,
                sensor_distance: 9.0,
                rotation_angle: 45.0,
                step_size: 1.2,
                decay_factor: 0.90,
                deposit_amount: 5.0,
                diffusion_kernel: DiffusionKernel::Mean3x3,
                diffusion_sigma: 1.0,
                max_brightness: 18.0,
                attractors: Vec::new(),
                attractor_strength: 1.0,
                mouse_attractors: Vec::new(),
                mouse_timeout: 3.0,
                species_configs: vec![SpeciesConfig {
                    name: "default".to_string(),
                    count: 45_000,
                    sensor_angle: 25.0,
                    rotation_angle: 45.0,
                    step_size: 1.2,
                    deposit_amount: 5.0,
                    color: "1e90ff".to_string(),
                }],
                separate_species_trails: false,
                use_simd: true,
                food_image_path: None,
                food_image_invert: false,
                food_image_scale: 1.0,
                obstacles: Vec::new(),
                obstacle_masks: Vec::new(),
                wind: Some(Wind::new(0.3, 0.0)),
                terrain: TerrainType::None,
                terrain_strength: 1.0,
            },
            Preset::Ethereal => Self {
                sensor_angle: 40.0,
                sensor_distance: 9.0,
                rotation_angle: 45.0,
                step_size: 0.7,
                decay_factor: 0.98,
                deposit_amount: 2.0,
                diffusion_kernel: DiffusionKernel::Gaussian,
                diffusion_sigma: 1.0,
                max_brightness: 12.0,
                attractors: Vec::new(),
                attractor_strength: 1.0,
                mouse_attractors: Vec::new(),
                mouse_timeout: 3.0,
                species_configs: vec![SpeciesConfig {
                    name: "default".to_string(),
                    count: 25_000,
                    sensor_angle: 40.0,
                    rotation_angle: 45.0,
                    step_size: 0.7,
                    deposit_amount: 2.0,
                    color: "e6e6fa".to_string(),
                }],
                separate_species_trails: false,
                use_simd: true,
                food_image_path: None,
                food_image_invert: false,
                food_image_scale: 1.0,
                obstacles: Vec::new(),
                obstacle_masks: Vec::new(),
                wind: None,
                terrain: TerrainType::None,
                terrain_strength: 1.0,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
        assert_eq!(config.max_brightness, 20.0);
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
            max_brightness: 150.0,
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
}
