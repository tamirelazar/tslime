use crate::cli::Palette;
use crate::render::charset::Charset;
use crate::simulation::config::{DiffusionKernel, InitMode, SimConfig, SpeciesConfig};
use crate::terminal::control::RuntimeState;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

const CONFIG_DIR: &str = ".config/tslime";
const CONFIG_FILE: &str = "presets.toml";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedConfig {
    pub name: String,
    pub description: Option<String>,

    // Simulation parameters
    pub population: usize,
    pub sensor_angle: f32,
    pub sensor_distance: f32,
    pub rotation_angle: f32,
    pub step_size: f32,
    pub decay_factor: f32,
    pub deposit_amount: f32,
    pub max_brightness: f32,
    pub diffusion_kernel: String,
    pub diffusion_sigma: f32,

    // Visual parameters
    pub palette: String,
    pub charset: String,
    pub reverse_palette: bool,
    pub invert_palette: bool,

    // Feature flags
    pub warmup_frames: usize,
    pub food_persist: bool,
    pub auto_reset: bool,
    pub grid: bool,
    pub grid_style: Option<String>,

    // Init mode
    pub init_mode: String,
    pub food_path: Option<String>,
}

impl SavedConfig {
    #[allow(clippy::too_many_arguments)]
    pub fn from_runtime(
        name: String,
        sim_config: &SimConfig,
        palette: Palette,
        charset: Charset,
        reverse_palette: bool,
        invert_palette: bool,
        warmup_frames: usize,
        food_persist: bool,
        auto_reset: bool,
        grid: bool,
        grid_style: Option<String>,
        init_mode: InitMode,
        food_path: Option<String>,
    ) -> Self {
        let diffusion_kernel_str = match sim_config.diffusion_kernel {
            DiffusionKernel::Mean3x3 => "mean3x3",
            DiffusionKernel::Gaussian => "gaussian",
        };

        let palette_str = match palette {
            Palette::Organic => "organic",
            Palette::Heat => "heat",
            Palette::Ocean => "ocean",
            Palette::Mono => "mono",
            Palette::Forest => "forest",
            Palette::Neon => "neon",
            Palette::Warm => "warm",
            Palette::Vibrant => "vibrant",
            Palette::LegibleMono => "legiblemono",
            Palette::Slime => "slime",
            Palette::Mold => "mold",
            Palette::Fungus => "fungus",
            Palette::Swamp => "swamp",
            Palette::Moss => "moss",
            Palette::Cosmic => "cosmic",
            Palette::Ethereal => "ethereal",
        };

        let charset_str = match charset {
            Charset::HalfBlock => "halfblock",
            Charset::Ascii => "ascii",
            Charset::Braille => "braille",
            Charset::CustomAscii(_) => "ascii", // Save as "ascii" for now
        };

        let init_mode_str = match init_mode {
            InitMode::Random => "random",
            InitMode::CentralBurst => "central",
            InitMode::Circle => "circle",
            InitMode::Gradient => "gradient",
            InitMode::WaveFront => "wave",
            InitMode::Spiral => "spiral",
            InitMode::RandomClusters => "clusters",
            InitMode::Food => "food",
        };

        // Get first species config for population and parameters
        let first_species = sim_config
            .species_configs
            .first()
            .expect("At least one species config should exist");

        Self {
            name,
            description: None,
            population: first_species.count,
            sensor_angle: first_species.sensor_angle,
            sensor_distance: sim_config.sensor_distance,
            rotation_angle: first_species.rotation_angle,
            step_size: first_species.step_size,
            decay_factor: sim_config.decay_factor,
            deposit_amount: first_species.deposit_amount,
            max_brightness: sim_config.max_brightness,
            diffusion_kernel: diffusion_kernel_str.to_string(),
            diffusion_sigma: sim_config.diffusion_sigma,
            palette: palette_str.to_string(),
            charset: charset_str.to_string(),
            reverse_palette,
            invert_palette,
            warmup_frames,
            food_persist,
            auto_reset,
            grid,
            grid_style,
            init_mode: init_mode_str.to_string(),
            food_path,
        }
    }

    /// Apply this saved config to runtime state
    pub fn apply_to_runtime_state(&self, runtime_state: &mut RuntimeState) -> Result<(), String> {
        // Parse and apply palette
        runtime_state.palette_index = parse_palette_index(&self.palette)?;
        runtime_state.reverse_palette = self.reverse_palette;
        runtime_state.invert_palette = self.invert_palette;

        // Parse and apply diffusion kernel
        runtime_state.diffusion_kernel = parse_diffusion_kernel(&self.diffusion_kernel)?;

        // Apply simulation parameters
        runtime_state.sensor_angle = self.sensor_angle;
        runtime_state.turn_angle = self.rotation_angle;
        runtime_state.step_size = self.step_size;
        runtime_state.decay_factor = self.decay_factor;
        runtime_state.deposit_amount = self.deposit_amount;
        runtime_state.max_brightness = self.max_brightness;

        // Reset warmup so the changes can be seen
        runtime_state.warmup_counter = 0;

        Ok(())
    }

    /// Convert this saved config to a SimConfig for restarting simulation
    #[allow(dead_code)]
    pub fn to_sim_config(&self) -> Result<SimConfig, String> {
        let diffusion_kernel = parse_diffusion_kernel(&self.diffusion_kernel)?;
        let _init_mode = parse_init_mode(&self.init_mode)?;

        let species_config = SpeciesConfig {
            name: "default".to_string(),
            count: self.population,
            sensor_angle: self.sensor_angle,
            rotation_angle: self.rotation_angle,
            step_size: self.step_size,
            deposit_amount: self.deposit_amount,
            color: "228b22".to_string(),
        };

        Ok(SimConfig {
            sensor_angle: self.sensor_angle,
            sensor_distance: self.sensor_distance,
            rotation_angle: self.rotation_angle,
            step_size: self.step_size,
            decay_factor: self.decay_factor,
            deposit_amount: self.deposit_amount,
            diffusion_kernel,
            diffusion_sigma: self.diffusion_sigma,
            max_brightness: self.max_brightness,
            attractors: Vec::new(),
            attractor_strength: 1.0,
            mouse_attractors: Vec::new(),
            mouse_timeout: 3.0,
            species_configs: vec![species_config],
            separate_species_trails: false,
            use_simd: true,
            food_image_path: self.food_path.clone(),
            food_image_invert: false,
            food_image_scale: 1.0,
            obstacles: Vec::new(),
            obstacle_masks: Vec::new(),
            wind: None,
            terrain: crate::simulation::config::TerrainType::None,
            terrain_strength: 1.0,
        })
    }
}

// Helper functions for parsing saved config strings

fn parse_palette_index(palette_str: &str) -> Result<usize, String> {
    match palette_str.to_lowercase().as_str() {
        "organic" => Ok(0),
        "heat" => Ok(1),
        "ocean" => Ok(2),
        "mono" => Ok(3),
        "forest" => Ok(4),
        "neon" => Ok(5),
        "warm" => Ok(6),
        "vibrant" => Ok(7),
        "legiblemono" => Ok(8),
        "slime" => Ok(9),
        "mold" => Ok(10),
        "fungus" => Ok(11),
        "swamp" => Ok(12),
        "moss" => Ok(13),
        "cosmic" => Ok(14),
        "ethereal" => Ok(15),
        _ => Err(format!("Unknown palette: {}", palette_str)),
    }
}

fn parse_diffusion_kernel(s: &str) -> Result<DiffusionKernel, String> {
    match s.to_lowercase().as_str() {
        "mean3x3" => Ok(DiffusionKernel::Mean3x3),
        "gaussian" => Ok(DiffusionKernel::Gaussian),
        _ => Err(format!("Unknown diffusion kernel: {}", s)),
    }
}

#[allow(dead_code)]
fn parse_init_mode(s: &str) -> Result<InitMode, String> {
    match s.to_lowercase().as_str() {
        "random" => Ok(InitMode::Random),
        "central" => Ok(InitMode::CentralBurst),
        "circle" => Ok(InitMode::Circle),
        "gradient" => Ok(InitMode::Gradient),
        "wave" => Ok(InitMode::WaveFront),
        "spiral" => Ok(InitMode::Spiral),
        "clusters" => Ok(InitMode::RandomClusters),
        "food" => Ok(InitMode::Food),
        _ => Err(format!("Unknown init mode: {}", s)),
    }
}

#[allow(dead_code)]
fn parse_charset(s: &str) -> Result<Charset, String> {
    match s.to_lowercase().as_str() {
        "halfblock" => Ok(Charset::HalfBlock),
        "ascii" => Ok(Charset::Ascii),
        "braille" => Ok(Charset::Braille),
        _ => Err(format!("Unknown charset: {}", s)),
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ConfigFile {
    #[serde(rename = "preset")]
    presets: Vec<SavedConfig>,
}

pub fn get_config_path() -> Result<PathBuf, String> {
    let home = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .map_err(|_| "Could not determine home directory".to_string())?;

    let config_dir = PathBuf::from(home).join(CONFIG_DIR);

    // Create directory if it doesn't exist
    if !config_dir.exists() {
        fs::create_dir_all(&config_dir)
            .map_err(|e| format!("Failed to create config directory: {}", e))?;
    }

    Ok(config_dir.join(CONFIG_FILE))
}

fn load_config_file() -> Result<ConfigFile, String> {
    let path = get_config_path()?;

    if !path.exists() {
        return Ok(ConfigFile {
            presets: Vec::new(),
        });
    }

    let contents =
        fs::read_to_string(&path).map_err(|e| format!("Failed to read config file: {}", e))?;

    toml::from_str(&contents).map_err(|e| format!("Failed to parse config file: {}", e))
}

fn save_config_file(config_file: &ConfigFile) -> Result<(), String> {
    let path = get_config_path()?;

    let toml_string = toml::to_string_pretty(config_file)
        .map_err(|e| format!("Failed to serialize config: {}", e))?;

    fs::write(&path, toml_string).map_err(|e| format!("Failed to write config file: {}", e))?;

    Ok(())
}

pub fn save_config(config: SavedConfig) -> Result<(), String> {
    let mut config_file = load_config_file()?;

    // Remove existing config with same name
    config_file.presets.retain(|c| c.name != config.name);

    // Add new config
    config_file.presets.push(config);

    save_config_file(&config_file)
}

#[allow(dead_code)]
pub fn load_config(name: &str) -> Result<SavedConfig, String> {
    let config_file = load_config_file()?;

    config_file
        .presets
        .iter()
        .find(|c| c.name == name)
        .cloned()
        .ok_or_else(|| format!("Config '{}' not found", name))
}

pub fn list_configs() -> Result<Vec<SavedConfig>, String> {
    let config_file = load_config_file()?;
    Ok(config_file.presets)
}

pub fn delete_config(name: &str) -> Result<(), String> {
    let mut config_file = load_config_file()?;

    let original_len = config_file.presets.len();
    config_file.presets.retain(|c| c.name != name);

    if config_file.presets.len() == original_len {
        return Err(format!("Config '{}' not found", name));
    }

    save_config_file(&config_file)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_serialization() {
        let config = SavedConfig {
            name: "test".to_string(),
            description: Some("Test config".to_string()),
            population: 50000,
            sensor_angle: 22.5,
            sensor_distance: 9.0,
            rotation_angle: 45.0,
            step_size: 1.0,
            decay_factor: 0.85,
            deposit_amount: 5.0,
            max_brightness: 20.0,
            diffusion_kernel: "mean3x3".to_string(),
            diffusion_sigma: 1.0,
            palette: "forest".to_string(),
            charset: "halfblock".to_string(),
            reverse_palette: false,
            invert_palette: false,
            warmup_frames: 60,
            food_persist: false,
            auto_reset: true,
            grid: false,
            grid_style: None,
            init_mode: "food".to_string(),
            food_path: Some("assets/tslime_logo.png".to_string()),
        };

        let toml_str = toml::to_string(&config).unwrap();
        let deserialized: SavedConfig = toml::from_str(&toml_str).unwrap();

        assert_eq!(config.name, deserialized.name);
        assert_eq!(config.population, deserialized.population);
    }
}
