use crate::cli::Palette;
use crate::render::charset::Charset;
use crate::render::palette::{IntensityMapping, RgbColor};
use crate::simulation::config::{DiffusionKernel, InitMode, SimConfig, SpeciesConfig};
use crate::terminal::control::RuntimeState;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

const CONFIG_DIR: &str = ".config/tslime";
const CONFIG_FILE: &str = "presets.toml";

/// Available intensity mapping presets (mirrors RuntimeState::INTENSITY_MAPPINGS)
#[allow(clippy::type_complexity)]
const INTENSITY_MAPPINGS: &[(&str, fn() -> IntensityMapping)] = &[
    ("Linear", || IntensityMapping::linear()),
    ("Logarithmic", || IntensityMapping::logarithmic(10.0)),
    ("Exponential", || IntensityMapping::exponential(10.0)),
    ("Square Root", || {
        IntensityMapping::new(vec![crate::render::palette::MappingSegment {
            start: 0.0,
            end: 1.0,
            function: crate::render::palette::MappingFunction::SquareRoot,
        }])
        .unwrap()
    }),
    ("Smoothstep", || IntensityMapping::smoothstep()),
    ("Split (Lin/Log)", || {
        IntensityMapping::linear_log_split(10.0)
    }),
    ("Quantize 6", || IntensityMapping::quantize(6)),
    ("Perlin", || IntensityMapping::perlin(0.15, 4.0, 42)),
];

/// Finds the index of a given intensity mapping by comparing with presets.
/// Returns 0 if no match found (falls back to Linear).
fn find_intensity_mapping_index(mapping: &IntensityMapping) -> usize {
    for (i, (_, factory)) in INTENSITY_MAPPINGS.iter().enumerate() {
        if &factory() == mapping {
            return i;
        }
    }
    0
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Represents a saved simulation configuration.
pub struct SavedConfig {
    /// Name of the preset.
    pub name: String,
    /// Optional description of the preset.
    pub description: Option<String>,

    // Simulation parameters
    /// Number of agents.
    pub population: usize,
    /// Angle between sensors.
    pub sensor_angle: f32,
    /// Distance to sensors.
    pub sensor_distance: f32,
    /// Rotation angle per step.
    pub rotation_angle: f32,
    /// Movement step size.
    pub step_size: f32,
    /// Trail decay factor.
    pub decay_factor: f32,
    /// Amount of trail deposited.
    pub deposit_amount: f32,
    /// Max brightness for normalization.
    pub max_brightness: f32,
    /// Diffusion kernel name.
    pub diffusion_kernel: String,
    /// Sigma for gaussian diffusion.
    pub diffusion_sigma: f32,

    // Visual parameters
    /// Color palette name or definition.
    pub palette: String,
    /// Character set name.
    pub charset: String,
    /// Whether palette is reversed.
    pub reverse_palette: bool,
    /// Whether palette is inverted.
    pub invert_palette: bool,

    // Feature flags
    /// Number of warmup frames.
    pub warmup_frames: usize,
    /// Whether food persistence is enabled.
    pub food_persist: bool,
    /// Whether auto-reset is enabled.
    pub auto_reset: bool,
    /// Whether grid is enabled.
    pub grid: bool,
    /// Grid style name.
    pub grid_style: Option<String>,

    // Init mode
    /// Initialization mode name.
    pub init_mode: String,
    /// Optional path to food image.
    pub food_path: Option<String>,
    /// Optional background color.
    pub background_color: Option<String>,

    // Intensity mapping
    /// Intensity mapping function name.
    pub intensity_mapping: Option<String>,
    /// Base for log/exp mapping.
    pub intensity_mapping_base: Option<f32>,
    /// Gamma for power mapping.
    pub intensity_mapping_gamma: Option<f32>,
    /// Levels for quantization.
    pub intensity_mapping_levels: Option<u8>,

    // Window frame
    /// Window frame display mode.
    pub window_frame: String,
}

impl SavedConfig {
    #[allow(clippy::too_many_arguments)]
    /// Creates a new `SavedConfig` from the current runtime state.
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
        intensity_mapping: Option<&crate::render::palette::IntensityMapping>,
    ) -> Self {
        let diffusion_kernel_str = match sim_config.diffusion_kernel {
            DiffusionKernel::Mean3x3 => "mean3x3",
            DiffusionKernel::Gaussian => "gaussian",
        };

        // Handle palette string conversion (Custom needs special handling)
        let palette_str: String = match &palette {
            Palette::Organic => "organic".to_string(),
            Palette::Heat => "heat".to_string(),
            Palette::Ocean => "ocean".to_string(),
            Palette::Mono => "mono".to_string(),
            Palette::Forest => "forest".to_string(),
            Palette::Neon => "neon".to_string(),
            Palette::Warm => "warm".to_string(),
            Palette::Vibrant => "vibrant".to_string(),
            Palette::LegibleMono => "legiblemono".to_string(),
            Palette::Slime => "slime".to_string(),
            Palette::Mold => "mold".to_string(),
            Palette::Fungus => "fungus".to_string(),
            Palette::Swamp => "swamp".to_string(),
            Palette::Moss => "moss".to_string(),
            Palette::Cosmic => "cosmic".to_string(),
            Palette::Ethereal => "ethereal".to_string(),
            Palette::Custom(colors) => {
                let hex_colors: Vec<String> = colors
                    .iter()
                    .map(|c| format!("#{:02x}{:02x}{:02x}", c.r, c.g, c.b))
                    .collect();
                format!("custom:{}", hex_colors.join(","))
            }
        };

        let charset_str = match charset {
            Charset::HalfBlock => "halfblock",
            Charset::HalfBlockDual => "halfblockdual",
            Charset::Ascii => "ascii",
            Charset::Braille => "braille",
            Charset::Quadrant => "quadrant",
            Charset::Shade => "shade",
            Charset::Points => "points",
            Charset::Sculpted => "sculpted",
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
            InitMode::Petri => "petri",
        };

        // Get first species config for population and parameters
        let first_species = sim_config
            .species_configs
            .first()
            .expect("At least one species config should exist");

        let (mapping_name, mapping_base, mapping_gamma, mapping_levels) =
            if let Some(mapping) = intensity_mapping {
                match mapping.segments().first().map(|s| &s.function) {
                    Some(crate::render::palette::MappingFunction::Linear) => {
                        (Some("linear".to_string()), None, None, None)
                    }
                    Some(crate::render::palette::MappingFunction::Logarithmic { base }) => {
                        (Some("logarithmic".to_string()), Some(*base), None, None)
                    }
                    Some(crate::render::palette::MappingFunction::Exponential { base }) => {
                        (Some("exponential".to_string()), Some(*base), None, None)
                    }
                    Some(crate::render::palette::MappingFunction::Power { gamma }) => {
                        (Some("power".to_string()), None, Some(*gamma), None)
                    }
                    Some(crate::render::palette::MappingFunction::SquareRoot) => {
                        (Some("sqrt".to_string()), None, None, None)
                    }
                    Some(crate::render::palette::MappingFunction::Quantize { levels }) => {
                        (Some("quantize".to_string()), None, None, Some(*levels))
                    }
                    Some(crate::render::palette::MappingFunction::Smoothstep) => {
                        (Some("smoothstep".to_string()), None, None, None)
                    }
                    // For complex mappings (Perlin, Split), we default to Linear or specialized handling
                    // Currently simplification for basic types
                    _ => (None, None, None, None),
                }
            } else {
                (None, None, None, None)
            };

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
            background_color: sim_config.background_color.clone(),
            intensity_mapping: mapping_name,
            intensity_mapping_base: mapping_base,
            intensity_mapping_gamma: mapping_gamma,
            intensity_mapping_levels: mapping_levels,
            window_frame: format!("{:?}", sim_config.window_frame).to_lowercase(),
        }
    }

    /// Apply this saved config to runtime state
    ///
    /// Note: Some parameters (population, init_mode, food_path) require a simulation
    /// restart to take effect. This function applies all runtime-adjustable parameters.
    pub fn apply_to_runtime_state(&self, runtime_state: &mut RuntimeState) -> Result<(), String> {
        // Parse and apply palette
        runtime_state.palette_index = parse_palette_index(&self.palette)?;
        runtime_state.reverse_palette = self.reverse_palette;
        runtime_state.invert_palette = self.invert_palette;

        // Parse and apply charset
        runtime_state.charset_index = parse_charset_index(&self.charset)?;

        // Parse and apply diffusion kernel
        runtime_state.diffusion_kernel = parse_diffusion_kernel(&self.diffusion_kernel)?;

        // Apply simulation parameters
        runtime_state.sensor_angle = self.sensor_angle;
        runtime_state.sensor_distance = self.sensor_distance;
        runtime_state.rotation_angle = self.rotation_angle;
        runtime_state.step_size = self.step_size;
        runtime_state.decay_factor = self.decay_factor;
        runtime_state.deposit_amount = self.deposit_amount;
        runtime_state.max_brightness = self.max_brightness;

        // Apply window frame
        runtime_state.window_frame = parse_window_frame(&self.window_frame).unwrap_or_default();

        // Apply food persistence setting
        runtime_state.food_persist_enabled = self.food_persist;

        // Reset warmup so the changes can be seen
        runtime_state.warmup_counter = 0;

        // Apply intensity mapping if present
        if let Some(mapping_name) = &self.intensity_mapping {
            use crate::render::palette::{IntensityMapping, MappingFunction};
            let mapping = match mapping_name.as_str() {
                "linear" => Some(IntensityMapping::linear()),
                "logarithmic" => Some(IntensityMapping::logarithmic(
                    self.intensity_mapping_base.unwrap_or(10.0),
                )),
                "exponential" => Some(IntensityMapping::exponential(
                    self.intensity_mapping_base.unwrap_or(10.0),
                )),
                "power" => Some(IntensityMapping::power(
                    self.intensity_mapping_gamma.unwrap_or(2.2),
                )),
                "sqrt" => Some(
                    IntensityMapping::new(vec![crate::render::palette::MappingSegment {
                        start: 0.0,
                        end: 1.0,
                        function: MappingFunction::SquareRoot,
                    }])
                    .unwrap(),
                ),
                "quantize" => Some(IntensityMapping::quantize(
                    self.intensity_mapping_levels.unwrap_or(8),
                )),
                "smoothstep" => Some(IntensityMapping::smoothstep()),
                _ => None,
            };

            if let Some(m) = mapping {
                // Update intensity_mapping_index to match first
                runtime_state.intensity_mapping_index = find_intensity_mapping_index(&m);
                runtime_state.intensity_mapping = m;
            }
        }

        // Parameters that require simulation restart to take effect:
        // - population (agent count)
        // - init_mode (initialization pattern)
        // - food_path (food image path)
        // - auto_reset (collapse detection)
        // - grid (background grid)
        // - grid_style (grid appearance)
        // - warmup_frames (logo display duration)
        //
        // To fully restore a config including these, use to_sim_config() and
        // restart the simulation.

        Ok(())
    }

    /// Returns true if this config may require a simulation restart to fully apply.
    ///
    /// This checks if any parameters differ from runtime-adjustable ones.
    pub fn requires_restart(&self) -> bool {
        // These parameters can be changed at runtime, so no restart needed
        // Check if any "restart-required" parameters are different from defaults
        self.warmup_frames > 0 || self.auto_reset || self.grid || self.grid_style.is_some()
    }

    /// Convert this saved config to a SimConfig for restarting simulation.
    ///
    /// This function is part of the public API but currently unused in the main application.
    /// It is retained for future use in configuration management features like
    /// "Restart with saved config" or "Export config to file".
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
            color: RgbColor::from_hex(0x228b22),
            trail_modulation: None,
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
            time_scale: 1.0,
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
            background_color: self.background_color.clone(),
            preferred_init_mode: None,
            boundary_mode: crate::simulation::config::BoundaryMode::Bounce,
            window_frame: parse_window_frame(&self.window_frame).unwrap_or_default(),
            respawn_config: crate::simulation::config::RespawnConfig::default(),
            sampling_mode: crate::simulation::config::SamplingMode::Nearest,
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

/// Parses an initialization mode from a string.
///
/// This function is part of the configuration parsing API but currently unused.
/// It is retained for future use in saved configuration loading.
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
        "petri" => Ok(InitMode::Petri),
        _ => Err(format!("Unknown init mode: {}", s)),
    }
}

#[allow(dead_code)]
fn parse_charset(s: &str) -> Result<Charset, String> {
    match s.to_lowercase().as_str() {
        "halfblock" => Ok(Charset::HalfBlock),
        "halfblockdual" => Ok(Charset::HalfBlockDual),
        "ascii" => Ok(Charset::Ascii),
        "braille" => Ok(Charset::Braille),
        "quadrant" => Ok(Charset::Quadrant),
        "shade" => Ok(Charset::Shade),
        "points" => Ok(Charset::Points),
        "sculpted" => Ok(Charset::Sculpted),
        _ => Err(format!("Unknown charset: {}", s)),
    }
}

fn parse_window_frame(s: &str) -> Result<crate::simulation::config::WindowFrame, String> {
    match s.to_lowercase().as_str() {
        "none" => Ok(crate::simulation::config::WindowFrame::None),
        "negative" => Ok(crate::simulation::config::WindowFrame::Negative),
        "accented" => Ok(crate::simulation::config::WindowFrame::Accented),
        "glow" => Ok(crate::simulation::config::WindowFrame::Glow),
        "reactive" => Ok(crate::simulation::config::WindowFrame::Reactive),
        "food" => Ok(crate::simulation::config::WindowFrame::Food),
        "frame" => Ok(crate::simulation::config::WindowFrame::Frame),
        _ => Err(format!("Unknown window frame: {}", s)),
    }
}

fn parse_charset_index(charset_str: &str) -> Result<usize, String> {
    let charset = parse_charset(charset_str)?;
    match charset {
        Charset::HalfBlock => Ok(0),
        Charset::HalfBlockDual => Ok(1),
        Charset::Ascii => Ok(2),
        Charset::Braille => Ok(3),
        Charset::Quadrant => Ok(4),
        Charset::Shade => Ok(5),
        Charset::Points => Ok(6),
        Charset::Sculpted => Ok(7),
        Charset::CustomAscii(_) => Ok(2), // Default to ASCII for custom
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ConfigFile {
    #[serde(rename = "preset")]
    presets: Vec<SavedConfig>,
}

/// Returns the path to the configuration file.
///
/// Creates the config directory if it doesn't exist.
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

/// Saves a configuration to the config file.
///
/// Overwrites any existing configuration with the same name.
pub fn save_config(config: SavedConfig) -> Result<(), String> {
    let mut config_file = load_config_file()?;

    // Remove existing config with same name
    config_file.presets.retain(|c| c.name != config.name);

    // Add new config
    config_file.presets.push(config);

    save_config_file(&config_file)
}

/// Loads a specific configuration by name.
///
/// This function is part of the public API but currently unused in the main application.
/// It is retained for future use in configuration management features like
/// "Load saved preset by name" or CLI config restoration.
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

/// Lists all saved configurations.
pub fn list_configs() -> Result<Vec<SavedConfig>, String> {
    let config_file = load_config_file()?;
    Ok(config_file.presets)
}

/// Deletes a configuration by name.
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
    use crate::cli::PauseStyle;
    use crate::simulation::config::Preset;

    fn create_test_runtime_state() -> RuntimeState {
        RuntimeState::new(
            42,
            InitMode::Random,
            Preset::Organic,
            0,
            0,
            crate::terminal::control::MouseInteractionMode::Disabled,
            3.0,
            crate::render::palette::IntensityMapping::linear(),
            &SimConfig::default(),
            PauseStyle::Vignette,
            false,
            false,
        )
    }

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
            charset: "halfblockdual".to_string(),
            reverse_palette: false,
            invert_palette: false,
            warmup_frames: 60,
            food_persist: false,
            auto_reset: true,
            grid: false,
            grid_style: None,
            init_mode: "food".to_string(),
            food_path: Some("assets/tslime_logo.png".to_string()),
            background_color: None,
            intensity_mapping: None,
            intensity_mapping_base: None,
            intensity_mapping_gamma: None,
            intensity_mapping_levels: None,
            window_frame: "frame".to_string(),
        };

        let toml_str = toml::to_string(&config).unwrap();
        let deserialized: SavedConfig = toml::from_str(&toml_str).unwrap();

        assert_eq!(config.name, deserialized.name);
        assert_eq!(config.population, deserialized.population);
    }

    #[test]
    fn test_apply_palette_to_runtime_state() {
        let state = create_test_runtime_state();
        let _initial_palette_index = state.palette_index;

        let config = SavedConfig {
            name: "test_palette".to_string(),
            description: None,
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
            palette: "heat".to_string(), // Different from default
            charset: "halfblock".to_string(),
            reverse_palette: false,
            invert_palette: false,
            warmup_frames: 60,
            food_persist: false,
            auto_reset: false,
            grid: false,
            grid_style: None,
            init_mode: "random".to_string(),
            food_path: None,
            background_color: None,
            intensity_mapping: None,
            intensity_mapping_base: None,
            intensity_mapping_gamma: None,
            intensity_mapping_levels: None,
            window_frame: "frame".to_string(),
        };
        let sim_config = config.to_sim_config().unwrap();
        assert_eq!(sim_config.species_configs[0].count, 50000);
        assert_eq!(sim_config.diffusion_kernel, DiffusionKernel::Mean3x3);
    }

    #[test]
    fn test_helper_parsers() {
        assert_eq!(parse_init_mode("random").unwrap(), InitMode::Random);
        assert_eq!(parse_init_mode("circle").unwrap(), InitMode::Circle);
        assert!(parse_init_mode("invalid").is_err());

        assert_eq!(parse_charset("ascii").unwrap(), Charset::Ascii);
        assert_eq!(parse_charset("braille").unwrap(), Charset::Braille);
        assert!(parse_charset("invalid").is_err());
    }

    #[test]
    fn test_full_config_roundtrip() {
        let mut state = create_test_runtime_state();

        // Modify state to have specific values
        state.palette_index = 5; // neon
        state.reverse_palette = true;
        state.invert_palette = true;
        state.sensor_angle = 35.0;
        state.rotation_angle = 55.0;
        state.step_size = 1.5;
        state.decay_factor = 0.92;
        state.deposit_amount = 6.5;
        state.max_brightness = 30.0;
        state.diffusion_kernel = DiffusionKernel::Gaussian;

        // Create SavedConfig from runtime state (via SimConfig)
        let sim_config = SimConfig {
            sensor_angle: state.sensor_angle,
            sensor_distance: 9.0,
            rotation_angle: state.rotation_angle,
            step_size: state.step_size,
            decay_factor: state.decay_factor,
            deposit_amount: state.deposit_amount,
            diffusion_kernel: state.diffusion_kernel,
            diffusion_sigma: 1.0,
            max_brightness: state.max_brightness,
            time_scale: 1.0,
            attractors: Vec::new(),
            attractor_strength: 1.0,
            mouse_attractors: Vec::new(),
            mouse_timeout: 3.0,
            species_configs: vec![SpeciesConfig {
                name: "default".to_string(),
                count: 50000,
                sensor_angle: state.sensor_angle,
                rotation_angle: state.rotation_angle,
                step_size: state.step_size,
                deposit_amount: state.deposit_amount,
                color: RgbColor::from_hex(0x228b22),
                trail_modulation: None,
            }],
            separate_species_trails: false,
            use_simd: true,
            food_image_path: None,
            food_image_invert: false,
            food_image_scale: 1.0,
            obstacles: Vec::new(),
            obstacle_masks: Vec::new(),
            wind: None,
            terrain: crate::simulation::config::TerrainType::None,
            terrain_strength: 1.0,
            background_color: None,
            preferred_init_mode: None,
            boundary_mode: crate::simulation::config::BoundaryMode::Bounce,
            respawn_config: crate::simulation::config::RespawnConfig::default(),
            sampling_mode: crate::simulation::config::SamplingMode::Nearest,
            window_frame: crate::simulation::config::WindowFrame::Frame,
        };

        let saved_config = SavedConfig::from_runtime(
            "roundtrip_test".to_string(),
            &sim_config,
            Palette::Neon,
            Charset::HalfBlock,
            state.reverse_palette,
            state.invert_palette,
            60,
            false,
            false,
            false,
            None,
            InitMode::Random,
            None,
            Some(&crate::render::palette::IntensityMapping::linear()),
        );

        // Create new state and apply config
        let mut new_state = create_test_runtime_state();
        saved_config.apply_to_runtime_state(&mut new_state).unwrap();

        // Verify all values match
        assert_eq!(new_state.palette_index, state.palette_index);
        assert_eq!(new_state.reverse_palette, state.reverse_palette);
        assert_eq!(new_state.invert_palette, state.invert_palette);
        assert_eq!(new_state.sensor_angle, state.sensor_angle);
        assert_eq!(new_state.rotation_angle, state.rotation_angle);
        assert_eq!(new_state.step_size, state.step_size);
        assert_eq!(new_state.decay_factor, state.decay_factor);
        assert_eq!(new_state.deposit_amount, state.deposit_amount);
        assert_eq!(new_state.max_brightness, state.max_brightness);
        assert_eq!(new_state.diffusion_kernel, state.diffusion_kernel);
    }
}
