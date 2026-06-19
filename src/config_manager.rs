use crate::cli::Palette;
use crate::config_defaults::trail;
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
    #[serde(default)]
    pub window_frame: String,

    // Window mode chrome / layout
    /// Chrome display style (minimal, expanded, fullscreen).
    #[serde(default = "default_chrome_style")]
    pub chrome_style: String,
    /// Visual aspect ratio (e.g. "3:2").
    #[serde(default = "default_aspect")]
    pub aspect: String,
    /// Outer padding ("auto" or integer).
    #[serde(default = "default_window_padding")]
    pub window_padding: String,
    /// Show legacy status bar in windowed mode.
    #[serde(default)]
    pub show_status_bar: bool,
    /// Minimum sim size before dropping padding (e.g. "20x10").
    #[serde(default = "default_min_sim_size")]
    pub min_sim_size: String,
    /// Minimum sim size before dropping the frame (e.g. "12x6").
    #[serde(default = "default_min_frame_size")]
    pub min_frame_size: String,

    // Temporal color
    /// Temporal-color strength (0.0 = off).
    #[serde(default)]
    pub temporal_color: Option<f32>,
    /// Temporal lag in frames for the EMA comparison.
    #[serde(default)]
    pub temporal_lag: Option<f32>,
    /// Temporal mode: "hue" or "accent".
    #[serde(default)]
    pub temporal_mode: Option<String>,
    // Afterglow
    /// Afterglow strength (0.0 = off).
    #[serde(default)]
    pub afterglow: Option<f32>,
    /// Afterglow EMA rate.
    #[serde(default)]
    pub afterglow_rate: Option<f32>,
    /// Value-dependent decay exponent (1.0 = uniform, <1.0 = faint tails persist longer).
    #[serde(default)]
    pub decay_gamma: Option<f32>,
    /// Lague diffuse-weight blend factor (1.0 = full blur; 0.0 = no diffusion).
    #[serde(default)]
    pub diffuse_weight: Option<f32>,
    /// Nonlinear deposit curve ("linear", "sqrt", "log", "pow").
    #[serde(default)]
    pub deposit_curve: Option<String>,
    /// Deposit scale (post-curve multiplier).
    #[serde(default)]
    pub deposit_scale: Option<f32>,
    /// Deposit gamma (Pow exponent).
    #[serde(default)]
    pub deposit_gamma: Option<f32>,
    /// Deposit cap (0 = off).
    #[serde(default)]
    pub deposit_cap: Option<f32>,

    // Palette cycles
    /// Number of palette repeats across the brightness range (None = identity = 1).
    #[serde(default)]
    pub palette_cycles: Option<u32>,
    /// Palette cycle mode ("wrap" or "mirror"). None = identity.
    #[serde(default)]
    pub palette_cycle_mode: Option<String>,

    // Glyph-by-shape
    /// Glyph-selection mode ("brightness", "shape", "hybrid"). None = identity (native per-charset).
    #[serde(default)]
    pub glyph_selection: Option<String>,
    /// Sobel edge-magnitude threshold for hybrid glyph mode.
    #[serde(default)]
    pub glyph_edge_threshold: Option<f32>,
}

fn default_chrome_style() -> String {
    "minimal".to_string()
}
fn default_aspect() -> String {
    "3:2".to_string()
}
fn default_window_padding() -> String {
    "auto".to_string()
}
fn default_min_sim_size() -> String {
    "20x10".to_string()
}
fn default_min_frame_size() -> String {
    "12x6".to_string()
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
        temporal_color: f32,
        temporal_lag_frames: f32,
        temporal_mode: crate::render::palette::TemporalMode,
        afterglow: f32,
        afterglow_rate: f32,
        decay_gamma: f32,
        diffuse_weight: f32,
        deposit_curve: crate::simulation::config::DepositCurve,
        deposit_scale: f32,
        deposit_gamma: f32,
        deposit_cap: f32,
        palette_cycle: crate::render::palette::PaletteCycle,
        glyph: crate::render::charset::GlyphConfig,
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
                    // Complex mappings (Perlin, Split) have no saved-format
                    // representation; record no mapping.
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
            chrome_style: format!("{:?}", sim_config.chrome_style).to_lowercase(),
            aspect: format!("{}:{}", sim_config.aspect.width, sim_config.aspect.height),
            window_padding: match sim_config.window_padding {
                crate::simulation::config::WindowPadding::Auto => "auto".to_string(),
                crate::simulation::config::WindowPadding::Fixed(n) => n.to_string(),
            },
            show_status_bar: sim_config.show_status_bar,
            min_sim_size: format!(
                "{}x{}",
                sim_config.min_sim_size.width, sim_config.min_sim_size.height
            ),
            min_frame_size: format!(
                "{}x{}",
                sim_config.min_frame_size.width, sim_config.min_frame_size.height
            ),
            temporal_color: Some(temporal_color),
            temporal_lag: Some(temporal_lag_frames),
            temporal_mode: Some(match temporal_mode {
                crate::render::palette::TemporalMode::Hue => "hue".to_string(),
                crate::render::palette::TemporalMode::Accent => "accent".to_string(),
            }),
            afterglow: Some(afterglow),
            afterglow_rate: Some(afterglow_rate),
            decay_gamma: Some(decay_gamma),
            diffuse_weight: Some(diffuse_weight),
            deposit_curve: Some(deposit_curve_to_str(deposit_curve).to_string()),
            deposit_scale: Some(deposit_scale),
            deposit_gamma: Some(deposit_gamma),
            deposit_cap: Some(deposit_cap),
            palette_cycles: if palette_cycle.is_identity() {
                None
            } else {
                Some(palette_cycle.cycles)
            },
            palette_cycle_mode: if palette_cycle.is_identity() {
                None
            } else {
                Some(palette_cycle.mode.to_string())
            },
            glyph_selection: match glyph.selection {
                None => None,
                Some(crate::render::charset::GlyphSelection::Brightness) => {
                    Some("brightness".to_string())
                }
                Some(crate::render::charset::GlyphSelection::Shape) => Some("shape".to_string()),
                Some(crate::render::charset::GlyphSelection::Hybrid) => Some("hybrid".to_string()),
            },
            glyph_edge_threshold: if glyph.selection.is_none() {
                None
            } else {
                Some(glyph.edge_threshold)
            },
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
        runtime_state.diffusion_sigma = self.diffusion_sigma;

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

        // Apply window mode chrome / layout fields
        runtime_state.chrome_style = self.chrome_style.parse().unwrap_or_default();
        runtime_state.aspect = self.aspect.parse().unwrap_or_default();
        runtime_state.window_padding = self.window_padding.parse().unwrap_or_default();
        runtime_state.show_status_bar = self.show_status_bar;
        runtime_state.min_sim_size = self.min_sim_size.parse().unwrap_or_default();
        runtime_state.min_frame_size = self.min_frame_size.parse().unwrap_or(
            crate::simulation::config::TerminalSizeThreshold {
                width: 12,
                height: 6,
            },
        );

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
                // Keep the cycle index in sync with the applied mapping.
                runtime_state.intensity_mapping_index = find_intensity_mapping_index(&m);
                runtime_state.intensity_mapping = m;
            }
        } else {
            // No mapping recorded in the saved config — reset to the canonical default
            // (logarithmic) so load fully restores state rather than inheriting the session's.
            runtime_state.intensity_mapping = IntensityMapping::default();
            runtime_state.intensity_mapping_index =
                find_intensity_mapping_index(&runtime_state.intensity_mapping);
        }

        // Apply temporal color fields
        runtime_state.temporal_color = self.temporal_color.unwrap_or(0.0);
        runtime_state.temporal_lag_frames = self.temporal_lag.unwrap_or(8.0);
        runtime_state.temporal_mode = match self.temporal_mode.as_deref() {
            Some(s) if s.eq_ignore_ascii_case("accent") => {
                crate::render::palette::TemporalMode::Accent
            }
            _ => crate::render::palette::TemporalMode::Hue,
        };

        // Apply afterglow fields
        runtime_state.afterglow = self.afterglow.unwrap_or(0.0);
        runtime_state.afterglow_rate = self.afterglow_rate.unwrap_or(0.05);

        // Apply decay gamma
        runtime_state.decay_gamma = self.decay_gamma.unwrap_or(1.0);

        // Apply diffuse weight
        runtime_state.diffuse_weight = self.diffuse_weight.unwrap_or(1.0);

        // Apply deposit knobs
        runtime_state.deposit_curve = self
            .deposit_curve
            .as_deref()
            .map(parse_deposit_curve)
            .unwrap_or_default();
        runtime_state.deposit_scale = self.deposit_scale.unwrap_or(1.0);
        runtime_state.deposit_gamma = self.deposit_gamma.unwrap_or(1.0);
        runtime_state.deposit_cap = self.deposit_cap.unwrap_or(0.0);

        // Apply palette cycles
        {
            use crate::render::palette::{PaletteCycle, PaletteCycleMode};
            let cycles = self.palette_cycles.unwrap_or(1);
            let mode = self
                .palette_cycle_mode
                .as_deref()
                .and_then(|s| s.parse::<PaletteCycleMode>().ok())
                .unwrap_or_default();
            runtime_state.palette_cycle = PaletteCycle { cycles, mode };
        }

        // Apply glyph-by-shape config
        runtime_state.glyph = match &self.glyph_selection {
            Some(s) => {
                let sel = s
                    .parse::<crate::render::charset::GlyphSelection>()
                    .map_err(|e: String| e)?;
                crate::render::charset::GlyphConfig {
                    selection: Some(sel),
                    edge_threshold: self.glyph_edge_threshold.unwrap_or(
                        crate::config_defaults::glyph_consts::DEFAULT_GLYPH_EDGE_THRESHOLD,
                    ),
                }
            }
            None => crate::render::charset::GlyphConfig::default(),
        };

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

    /// Returns true if this config sets parameters that only take effect after
    /// a simulation restart (warmup, auto-reset, grid).
    pub fn requires_restart(&self) -> bool {
        self.warmup_frames > 0 || self.auto_reset || self.grid || self.grid_style.is_some()
    }

    /// Converts this saved config to a `SimConfig` for restarting the simulation.
    ///
    /// Not yet called by the app — live loading goes through
    /// [`apply_to_runtime_state`](Self::apply_to_runtime_state), which cannot
    /// restore restart-only parameters; a full-restore flow would use this.
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
            afterglow: trail::DEFAULT_AFTERGLOW,
            afterglow_rate: trail::DEFAULT_AFTERGLOW_RATE,
            diffuse_weight: trail::DEFAULT_DIFFUSE_WEIGHT,
            decay_gamma: trail::DEFAULT_DECAY_GAMMA,
            deposit_curve: crate::simulation::config::DepositCurve::default(),
            deposit_scale: trail::DEFAULT_DEPOSIT_SCALE,
            deposit_gamma: trail::DEFAULT_DEPOSIT_GAMMA,
            deposit_cap: trail::DEFAULT_DEPOSIT_CAP,
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
            chrome_style: self.chrome_style.parse().unwrap_or_default(),
            aspect: self.aspect.parse().unwrap_or_default(),
            window_padding: self.window_padding.parse().unwrap_or_default(),
            show_status_bar: self.show_status_bar,
            min_sim_size: self.min_sim_size.parse().unwrap_or_default(),
            min_frame_size: self.min_frame_size.parse().unwrap_or(
                crate::simulation::config::TerminalSizeThreshold {
                    width: 12,
                    height: 6,
                },
            ),
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

fn deposit_curve_to_str(c: crate::simulation::config::DepositCurve) -> &'static str {
    use crate::simulation::config::DepositCurve;
    match c {
        DepositCurve::Linear => "linear",
        DepositCurve::Sqrt => "sqrt",
        DepositCurve::Log => "log",
        DepositCurve::Pow => "pow",
    }
}

fn parse_deposit_curve(s: &str) -> crate::simulation::config::DepositCurve {
    use crate::simulation::config::DepositCurve;
    match s.to_lowercase().as_str() {
        "sqrt" => DepositCurve::Sqrt,
        "log" => DepositCurve::Log,
        "pow" => DepositCurve::Pow,
        _ => DepositCurve::Linear,
    }
}

/// Parses an initialization mode name from a saved config (used by
/// [`SavedConfig::to_sim_config`] to reject configs with unknown modes).
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

    config_file.presets.retain(|c| c.name != config.name);
    config_file.presets.push(config);

    save_config_file(&config_file)
}

/// Loads a saved configuration by name.
///
/// Not yet called by the app, which loads via [`list_configs`] and an index;
/// kept as the public load-by-name entry point.
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
            chrome_style: "minimal".to_string(),
            aspect: "3:2".to_string(),
            window_padding: "auto".to_string(),
            show_status_bar: false,
            min_sim_size: "20x10".to_string(),
            min_frame_size: "12x6".to_string(),
            temporal_color: None,
            temporal_lag: None,
            temporal_mode: None,
            afterglow: None,
            afterglow_rate: None,
            decay_gamma: None,
            diffuse_weight: None,
            deposit_curve: None,
            deposit_scale: None,
            deposit_gamma: None,
            deposit_cap: None,
            palette_cycles: None,
            palette_cycle_mode: None,
            glyph_selection: None,
            glyph_edge_threshold: None,
        };

        let toml_str = toml::to_string(&config).unwrap();
        let deserialized: SavedConfig = toml::from_str(&toml_str).unwrap();

        assert_eq!(config.name, deserialized.name);
        assert_eq!(config.population, deserialized.population);
    }

    #[test]
    fn test_deserialize_legacy_config_missing_window_fields() {
        // Regression: presets.toml files written before window_frame and the
        // window-mode chrome fields existed must still parse. Missing optional
        // fields fall back to serde defaults rather than failing the whole load
        // (which would break BOTH save and load, since save_config loads first).
        let legacy = r#"
[[preset]]
name = "Mossy Roots"
population = 50000
sensor_angle = 22.5
sensor_distance = 9.0
rotation_angle = 45.0
step_size = 1.0
decay_factor = 0.5
deposit_amount = 5.0
max_brightness = 100.0
diffusion_kernel = "gaussian"
diffusion_sigma = 1.0
palette = "moss"
charset = "halfblock"
reverse_palette = false
invert_palette = false
warmup_frames = 30
food_persist = false
auto_reset = false
grid = true
grid_style = "cross"
init_mode = "food"
food_path = "assets/tslime_logo.png"
"#;

        let parsed: ConfigFile =
            toml::from_str(legacy).expect("legacy config without window_frame must parse");
        assert_eq!(parsed.presets.len(), 1);
        assert_eq!(parsed.presets[0].name, "Mossy Roots");
        // window_frame defaults to empty; parse_window_frame falls back via unwrap_or_default.
        assert_eq!(parsed.presets[0].window_frame, "");
        assert_eq!(parsed.presets[0].chrome_style, "minimal");
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
            chrome_style: "minimal".to_string(),
            aspect: "3:2".to_string(),
            window_padding: "auto".to_string(),
            show_status_bar: false,
            min_sim_size: "20x10".to_string(),
            min_frame_size: "12x6".to_string(),
            temporal_color: None,
            temporal_lag: None,
            temporal_mode: None,
            afterglow: None,
            afterglow_rate: None,
            decay_gamma: None,
            diffuse_weight: None,
            deposit_curve: None,
            deposit_scale: None,
            deposit_gamma: None,
            deposit_cap: None,
            palette_cycles: None,
            palette_cycle_mode: None,
            glyph_selection: None,
            glyph_edge_threshold: None,
        };
        let sim_config = config.to_sim_config().unwrap();
        assert_eq!(sim_config.species_configs[0].count, 50000);
        assert_eq!(sim_config.diffusion_kernel, DiffusionKernel::Mean3x3);
    }

    #[test]
    fn apply_to_runtime_state_sets_diffusion_sigma() {
        let mut rs = create_test_runtime_state();

        let config = SavedConfig {
            name: "test_sigma".to_string(),
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
            diffusion_sigma: 2.75,
            palette: "heat".to_string(),
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
            chrome_style: "minimal".to_string(),
            aspect: "3:2".to_string(),
            window_padding: "auto".to_string(),
            show_status_bar: false,
            min_sim_size: "20x10".to_string(),
            min_frame_size: "12x6".to_string(),
            temporal_color: None,
            temporal_lag: None,
            temporal_mode: None,
            afterglow: None,
            afterglow_rate: None,
            decay_gamma: None,
            diffuse_weight: None,
            deposit_curve: None,
            deposit_scale: None,
            deposit_gamma: None,
            deposit_cap: None,
            palette_cycles: None,
            palette_cycle_mode: None,
            glyph_selection: None,
            glyph_edge_threshold: None,
        };

        config
            .apply_to_runtime_state(&mut rs)
            .expect("apply_to_runtime_state must succeed");
        assert_eq!(rs.diffusion_sigma, 2.75);
    }

    #[test]
    fn apply_to_runtime_state_resets_intensity_mapping_when_none() {
        use crate::render::palette::IntensityMapping;
        let mut rs = create_test_runtime_state();
        // Put a non-default mapping into the session first (default is logarithmic).
        rs.intensity_mapping = IntensityMapping::linear();
        rs.intensity_mapping_index = find_intensity_mapping_index(&rs.intensity_mapping);

        let config = SavedConfig {
            name: "test_intensity_none".to_string(),
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
            palette: "heat".to_string(),
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
            chrome_style: "minimal".to_string(),
            aspect: "3:2".to_string(),
            window_padding: "auto".to_string(),
            show_status_bar: false,
            min_sim_size: "20x10".to_string(),
            min_frame_size: "12x6".to_string(),
            temporal_color: None,
            temporal_lag: None,
            temporal_mode: None,
            afterglow: None,
            afterglow_rate: None,
            decay_gamma: None,
            diffuse_weight: None,
            deposit_curve: None,
            deposit_scale: None,
            deposit_gamma: None,
            deposit_cap: None,
            palette_cycles: None,
            palette_cycle_mode: None,
            glyph_selection: None,
            glyph_edge_threshold: None,
        };

        config
            .apply_to_runtime_state(&mut rs)
            .expect("apply_to_runtime_state must succeed");

        // Default is logarithmic(10.0); a saved config with no recorded mapping
        // must reset the session to the canonical default, not inherit linear.
        assert_eq!(rs.intensity_mapping, IntensityMapping::default());
        assert_eq!(
            rs.intensity_mapping_index,
            find_intensity_mapping_index(&IntensityMapping::default())
        );
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
            afterglow: trail::DEFAULT_AFTERGLOW,
            afterglow_rate: trail::DEFAULT_AFTERGLOW_RATE,
            diffuse_weight: trail::DEFAULT_DIFFUSE_WEIGHT,
            decay_gamma: trail::DEFAULT_DECAY_GAMMA,
            deposit_curve: crate::simulation::config::DepositCurve::default(),
            deposit_scale: trail::DEFAULT_DEPOSIT_SCALE,
            deposit_gamma: trail::DEFAULT_DEPOSIT_GAMMA,
            deposit_cap: trail::DEFAULT_DEPOSIT_CAP,
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
            chrome_style: crate::simulation::config::ChromeStyle::default(),
            aspect: crate::simulation::config::Aspect::default(),
            window_padding: crate::simulation::config::WindowPadding::default(),
            show_status_bar: false,
            min_sim_size: crate::simulation::config::TerminalSizeThreshold::default(),
            min_frame_size: crate::simulation::config::TerminalSizeThreshold {
                width: 12,
                height: 6,
            },
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
            0.0,
            8.0,
            crate::render::palette::TemporalMode::Hue,
            0.0,
            0.05,
            1.0,
            1.0, // diffuse_weight
            crate::simulation::config::DepositCurve::default(),
            1.0,
            1.0,
            0.0,
            crate::render::palette::PaletteCycle::default(),
            crate::render::charset::GlyphConfig::default(),
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

    #[test]
    fn temporal_fields_round_trip_and_default_off() {
        // An OLD TOML with no temporal fields must still deserialize, with
        // temporal_color defaulting to 0.0 (off).
        let toml = r#"name = "old"
population = 1000
sensor_angle = 22.5
sensor_distance = 9.0
rotation_angle = 45.0
step_size = 1.0
decay_factor = 0.9
deposit_amount = 5.0
max_brightness = 100.0
diffusion_kernel = "Mean3x3"
diffusion_sigma = 1.0
palette = "Organic"
charset = "HalfBlock"
reverse_palette = false
invert_palette = false
warmup_frames = 0
food_persist = false
auto_reset = false
grid = false
init_mode = "Random"
"#;
        let cfg: SavedConfig = toml::from_str(toml).expect("old config must still load");
        assert_eq!(cfg.temporal_color.unwrap_or(0.0), 0.0);
    }

    #[test]
    fn temporal_fields_full_round_trip() {
        use crate::render::palette::TemporalMode;

        // Build a RuntimeState with non-default temporal values and verify they
        // survive from_runtime → serialize → deserialize → apply_to_runtime_state.
        let mut state = create_test_runtime_state();
        state.temporal_color = 0.7;
        state.temporal_lag_frames = 12.0;
        state.temporal_mode = TemporalMode::Accent;

        let sim_config = SimConfig {
            sensor_angle: state.sensor_angle,
            sensor_distance: 9.0,
            rotation_angle: state.rotation_angle,
            step_size: state.step_size,
            decay_factor: state.decay_factor,
            deposit_amount: state.deposit_amount,
            diffusion_kernel: state.diffusion_kernel,
            diffusion_sigma: 1.0,
            afterglow: trail::DEFAULT_AFTERGLOW,
            afterglow_rate: trail::DEFAULT_AFTERGLOW_RATE,
            diffuse_weight: trail::DEFAULT_DIFFUSE_WEIGHT,
            decay_gamma: trail::DEFAULT_DECAY_GAMMA,
            deposit_curve: crate::simulation::config::DepositCurve::default(),
            deposit_scale: trail::DEFAULT_DEPOSIT_SCALE,
            deposit_gamma: trail::DEFAULT_DEPOSIT_GAMMA,
            deposit_cap: trail::DEFAULT_DEPOSIT_CAP,
            max_brightness: state.max_brightness,
            time_scale: 1.0,
            attractors: Vec::new(),
            attractor_strength: 1.0,
            mouse_attractors: Vec::new(),
            mouse_timeout: 3.0,
            species_configs: vec![SpeciesConfig {
                name: "default".to_string(),
                count: 1000,
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
            window_frame: crate::simulation::config::WindowFrame::None,
            chrome_style: crate::simulation::config::ChromeStyle::default(),
            aspect: crate::simulation::config::Aspect::default(),
            window_padding: crate::simulation::config::WindowPadding::default(),
            show_status_bar: false,
            min_sim_size: crate::simulation::config::TerminalSizeThreshold::default(),
            min_frame_size: crate::simulation::config::TerminalSizeThreshold {
                width: 12,
                height: 6,
            },
        };

        let saved = SavedConfig::from_runtime(
            "temporal_rt".to_string(),
            &sim_config,
            crate::cli::Palette::Organic,
            crate::render::charset::Charset::HalfBlock,
            false,
            false,
            0,
            false,
            false,
            false,
            None,
            crate::simulation::config::InitMode::Random,
            None,
            None,
            state.temporal_color,
            state.temporal_lag_frames,
            state.temporal_mode,
            state.afterglow,
            state.afterglow_rate,
            state.decay_gamma,
            state.diffuse_weight,
            crate::simulation::config::DepositCurve::default(),
            1.0,
            1.0,
            0.0,
            crate::render::palette::PaletteCycle::default(),
            crate::render::charset::GlyphConfig::default(),
        );

        // Serialize and deserialize through TOML
        let toml_str = toml::to_string(&saved).expect("serialize must succeed");
        let reloaded: SavedConfig = toml::from_str(&toml_str).expect("deserialize must succeed");

        // Restore into a fresh RuntimeState
        let mut new_state = create_test_runtime_state();
        reloaded
            .apply_to_runtime_state(&mut new_state)
            .expect("apply must succeed");

        assert!((new_state.temporal_color - 0.7).abs() < 1e-6);
        assert!((new_state.temporal_lag_frames - 12.0).abs() < 1e-6);
        assert_eq!(new_state.temporal_mode, TemporalMode::Accent);
    }

    #[test]
    fn diffusion_decay_art_knobs_round_trip() {
        // Verify all five diffusion/decay art knobs survive from_runtime →
        // TOML serialize → TOML deserialize → apply_to_runtime_state.
        let mut state = create_test_runtime_state();
        state.afterglow = 0.4;
        state.afterglow_rate = 0.03;
        state.decay_gamma = 0.6;
        state.diffuse_weight = 0.5;
        state.diffusion_sigma = 3.0;

        let sim_config = SimConfig {
            sensor_angle: state.sensor_angle,
            sensor_distance: 9.0,
            rotation_angle: state.rotation_angle,
            step_size: state.step_size,
            decay_factor: state.decay_factor,
            deposit_amount: state.deposit_amount,
            diffusion_kernel: state.diffusion_kernel,
            diffusion_sigma: state.diffusion_sigma,
            afterglow: state.afterglow,
            afterglow_rate: state.afterglow_rate,
            diffuse_weight: state.diffuse_weight,
            decay_gamma: state.decay_gamma,
            deposit_curve: crate::simulation::config::DepositCurve::default(),
            deposit_scale: trail::DEFAULT_DEPOSIT_SCALE,
            deposit_gamma: trail::DEFAULT_DEPOSIT_GAMMA,
            deposit_cap: trail::DEFAULT_DEPOSIT_CAP,
            max_brightness: state.max_brightness,
            time_scale: 1.0,
            attractors: Vec::new(),
            attractor_strength: 1.0,
            mouse_attractors: Vec::new(),
            mouse_timeout: 3.0,
            species_configs: vec![SpeciesConfig {
                name: "default".to_string(),
                count: 1000,
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
            window_frame: crate::simulation::config::WindowFrame::None,
            chrome_style: crate::simulation::config::ChromeStyle::default(),
            aspect: crate::simulation::config::Aspect::default(),
            window_padding: crate::simulation::config::WindowPadding::default(),
            show_status_bar: false,
            min_sim_size: crate::simulation::config::TerminalSizeThreshold::default(),
            min_frame_size: crate::simulation::config::TerminalSizeThreshold {
                width: 12,
                height: 6,
            },
        };

        let saved = SavedConfig::from_runtime(
            "art_knobs_rt".to_string(),
            &sim_config,
            crate::cli::Palette::Organic,
            crate::render::charset::Charset::HalfBlock,
            false,
            false,
            0,
            false,
            false,
            false,
            None,
            crate::simulation::config::InitMode::Random,
            None,
            None,
            0.0,
            8.0,
            crate::render::palette::TemporalMode::Hue,
            state.afterglow,
            state.afterglow_rate,
            state.decay_gamma,
            state.diffuse_weight,
            crate::simulation::config::DepositCurve::default(),
            1.0,
            1.0,
            0.0,
            crate::render::palette::PaletteCycle::default(),
            crate::render::charset::GlyphConfig::default(),
        );

        // Serialize and deserialize through TOML
        let toml_str = toml::to_string(&saved).expect("serialize must succeed");
        let reloaded: SavedConfig = toml::from_str(&toml_str).expect("deserialize must succeed");

        // Restore into a fresh RuntimeState
        let mut new_state = create_test_runtime_state();
        reloaded
            .apply_to_runtime_state(&mut new_state)
            .expect("apply must succeed");

        assert!(
            (new_state.afterglow - 0.4).abs() < 1e-6,
            "afterglow must survive round-trip (got {})",
            new_state.afterglow
        );
        assert!(
            (new_state.afterglow_rate - 0.03).abs() < 1e-6,
            "afterglow_rate must survive round-trip (got {})",
            new_state.afterglow_rate
        );
        assert!(
            (new_state.decay_gamma - 0.6).abs() < 1e-6,
            "decay_gamma must survive round-trip (got {})",
            new_state.decay_gamma
        );
        assert!(
            (new_state.diffuse_weight - 0.5).abs() < 1e-6,
            "diffuse_weight must survive round-trip (got {})",
            new_state.diffuse_weight
        );
        assert!(
            (new_state.diffusion_sigma - 3.0).abs() < 1e-6,
            "diffusion_sigma must survive round-trip (got {})",
            new_state.diffusion_sigma
        );
    }

    #[test]
    fn old_toml_without_art_knobs_loads_with_defaults() {
        // An OLD TOML without afterglow/decay_gamma/diffuse_weight must still
        // deserialize, and apply_to_runtime_state must produce the canonical
        // defaults (afterglow=0.0, afterglow_rate=0.05, decay_gamma=1.0,
        // diffuse_weight=1.0).
        let toml = r#"name = "old_no_knobs"
population = 1000
sensor_angle = 22.5
sensor_distance = 9.0
rotation_angle = 45.0
step_size = 1.0
decay_factor = 0.9
deposit_amount = 5.0
max_brightness = 100.0
diffusion_kernel = "Mean3x3"
diffusion_sigma = 1.0
palette = "Organic"
charset = "HalfBlock"
reverse_palette = false
invert_palette = false
warmup_frames = 0
food_persist = false
auto_reset = false
grid = false
init_mode = "Random"
"#;
        let cfg: SavedConfig =
            toml::from_str(toml).expect("old config without art knobs must load");
        assert!(
            cfg.afterglow.is_none(),
            "missing key must deserialize as None"
        );
        assert!(cfg.afterglow_rate.is_none());
        assert!(cfg.decay_gamma.is_none());
        assert!(cfg.diffuse_weight.is_none());

        // apply_to_runtime_state must fill defaults from the unwrap_or paths.
        let mut state = create_test_runtime_state();
        cfg.apply_to_runtime_state(&mut state)
            .expect("legacy config must still apply");
        assert_eq!(state.afterglow, 0.0, "default afterglow must be 0.0");
        assert!(
            (state.afterglow_rate - 0.05).abs() < 1e-6,
            "default afterglow_rate must be 0.05"
        );
        assert_eq!(state.decay_gamma, 1.0, "default decay_gamma must be 1.0");
        assert_eq!(
            state.diffuse_weight, 1.0,
            "default diffuse_weight must be 1.0"
        );
        // Deposit fields absent from old TOML → defaults.
        assert!(cfg.deposit_curve.is_none());
        assert!(cfg.deposit_scale.is_none());
        assert!(cfg.deposit_gamma.is_none());
        assert!(cfg.deposit_cap.is_none());
        assert_eq!(
            state.deposit_curve,
            crate::simulation::config::DepositCurve::default(),
            "default deposit_curve must be Linear"
        );
        assert_eq!(
            state.deposit_scale, 1.0,
            "default deposit_scale must be 1.0"
        );
        assert_eq!(
            state.deposit_gamma, 1.0,
            "default deposit_gamma must be 1.0"
        );
        assert_eq!(state.deposit_cap, 0.0, "default deposit_cap must be 0.0");
    }

    #[test]
    fn deposit_fields_round_trip_through_saved_config() {
        use crate::simulation::config::DepositCurve;

        let mut state = create_test_runtime_state();
        state.deposit_curve = DepositCurve::Pow;
        state.deposit_scale = 2.5;
        state.deposit_gamma = 0.5;
        state.deposit_cap = 7.0;

        let sim_config = SimConfig::default();

        let saved = SavedConfig::from_runtime(
            "deposit_rt".to_string(),
            &sim_config,
            crate::cli::Palette::Organic,
            crate::render::charset::Charset::HalfBlock,
            false,
            false,
            0,
            false,
            false,
            false,
            None,
            crate::simulation::config::InitMode::Random,
            None,
            None,
            0.0,
            8.0,
            crate::render::palette::TemporalMode::Hue,
            0.0,
            0.05,
            1.0,
            1.0,
            state.deposit_curve,
            state.deposit_scale,
            state.deposit_gamma,
            state.deposit_cap,
            crate::render::palette::PaletteCycle::default(),
            crate::render::charset::GlyphConfig::default(),
        );

        // Serialize and deserialize through TOML
        let toml_str = toml::to_string(&saved).expect("serialize must succeed");
        let reloaded: SavedConfig = toml::from_str(&toml_str).expect("deserialize must succeed");

        // Restore into a fresh RuntimeState
        let mut restored = create_test_runtime_state();
        reloaded
            .apply_to_runtime_state(&mut restored)
            .expect("apply must succeed");

        assert_eq!(
            restored.deposit_curve,
            DepositCurve::Pow,
            "deposit_curve must survive round-trip"
        );
        assert!(
            (restored.deposit_scale - 2.5).abs() < 1e-6,
            "deposit_scale must survive round-trip (got {})",
            restored.deposit_scale
        );
        assert!(
            (restored.deposit_gamma - 0.5).abs() < 1e-6,
            "deposit_gamma must survive round-trip (got {})",
            restored.deposit_gamma
        );
        assert!(
            (restored.deposit_cap - 7.0).abs() < 1e-6,
            "deposit_cap must survive round-trip (got {})",
            restored.deposit_cap
        );
    }

    #[test]
    fn palette_cycle_round_trips_through_saved_config() {
        use crate::render::palette::{PaletteCycle, PaletteCycleMode};

        let mut state = create_test_runtime_state();
        state.palette_cycle = PaletteCycle {
            cycles: 4,
            mode: PaletteCycleMode::Wrap,
        };

        let saved = SavedConfig::from_runtime(
            "cycle_rt".to_string(),
            &SimConfig::default(),
            crate::cli::Palette::Organic,
            crate::render::charset::Charset::HalfBlock,
            false,
            false,
            0,
            false,
            false,
            false,
            None,
            crate::simulation::config::InitMode::Random,
            None,
            None,
            0.0,
            8.0,
            crate::render::palette::TemporalMode::Hue,
            0.0,
            0.05,
            1.0,
            1.0,
            crate::simulation::config::DepositCurve::default(),
            1.0,
            1.0,
            0.0,
            state.palette_cycle,
            crate::render::charset::GlyphConfig::default(),
        );

        assert_eq!(saved.palette_cycles, Some(4));
        assert_eq!(saved.palette_cycle_mode.as_deref(), Some("wrap"));

        // Serialize and deserialize through TOML
        let toml_str = toml::to_string(&saved).expect("serialize must succeed");
        let reloaded: SavedConfig = toml::from_str(&toml_str).expect("deserialize must succeed");

        // Restore into a fresh RuntimeState
        let mut rs2 = create_test_runtime_state();
        reloaded
            .apply_to_runtime_state(&mut rs2)
            .expect("apply must succeed");

        assert_eq!(
            rs2.palette_cycle,
            PaletteCycle {
                cycles: 4,
                mode: PaletteCycleMode::Wrap
            }
        );
    }

    #[test]
    fn glyph_round_trips_through_saved_config() {
        use crate::render::charset::{GlyphConfig, GlyphSelection};

        let mut rs = create_test_runtime_state();
        rs.glyph = GlyphConfig {
            selection: Some(GlyphSelection::Hybrid),
            edge_threshold: 0.25,
        };

        let saved = SavedConfig::from_runtime(
            "glyph_rt".to_string(),
            &SimConfig::default(),
            crate::cli::Palette::Organic,
            crate::render::charset::Charset::HalfBlock,
            false,
            false,
            0,
            false,
            false,
            false,
            None,
            crate::simulation::config::InitMode::Random,
            None,
            None,
            0.0,
            8.0,
            crate::render::palette::TemporalMode::Hue,
            0.0,
            0.05,
            1.0,
            1.0,
            crate::simulation::config::DepositCurve::default(),
            1.0,
            1.0,
            0.0,
            crate::render::palette::PaletteCycle::default(),
            rs.glyph,
        );

        assert_eq!(saved.glyph_selection.as_deref(), Some("hybrid"));
        assert_eq!(saved.glyph_edge_threshold, Some(0.25));

        let mut rs2 = create_test_runtime_state();
        saved.apply_to_runtime_state(&mut rs2).unwrap();
        assert_eq!(rs2.glyph.selection, Some(GlyphSelection::Hybrid));
        assert_eq!(rs2.glyph.edge_threshold, 0.25);
    }

    #[test]
    fn glyph_identity_serializes_to_none() {
        use crate::render::charset::GlyphConfig;

        let mut rs = create_test_runtime_state();
        rs.glyph = GlyphConfig::default();

        let saved = SavedConfig::from_runtime(
            "glyph_identity".to_string(),
            &SimConfig::default(),
            crate::cli::Palette::Organic,
            crate::render::charset::Charset::HalfBlock,
            false,
            false,
            0,
            false,
            false,
            false,
            None,
            crate::simulation::config::InitMode::Random,
            None,
            None,
            0.0,
            8.0,
            crate::render::palette::TemporalMode::Hue,
            0.0,
            0.05,
            1.0,
            1.0,
            crate::simulation::config::DepositCurve::default(),
            1.0,
            1.0,
            0.0,
            crate::render::palette::PaletteCycle::default(),
            rs.glyph,
        );

        assert_eq!(saved.glyph_selection, None);
        assert_eq!(saved.glyph_edge_threshold, None);
    }

    #[test]
    fn missing_palette_cycle_loads_identity() {
        use crate::render::palette::PaletteCycle;

        // Old TOML without palette_cycle fields must deserialize and produce identity.
        let toml = r#"name = "old_no_cycle"
population = 1000
sensor_angle = 22.5
sensor_distance = 9.0
rotation_angle = 45.0
step_size = 1.0
decay_factor = 0.9
deposit_amount = 5.0
max_brightness = 100.0
diffusion_kernel = "Mean3x3"
diffusion_sigma = 1.0
palette = "Organic"
charset = "HalfBlock"
reverse_palette = false
invert_palette = false
warmup_frames = 0
food_persist = false
auto_reset = false
grid = false
init_mode = "Random"
"#;
        let cfg: SavedConfig =
            toml::from_str(toml).expect("old config without palette_cycle fields must load");
        assert!(
            cfg.palette_cycles.is_none(),
            "missing key must deserialize as None"
        );
        assert!(cfg.palette_cycle_mode.is_none());

        // apply_to_runtime_state must produce identity (cycles=1).
        let mut state = create_test_runtime_state();
        cfg.apply_to_runtime_state(&mut state)
            .expect("legacy config must still apply");
        assert!(
            state.palette_cycle.is_identity(),
            "missing palette_cycles must default to identity"
        );
        assert_eq!(
            state.palette_cycle,
            PaletteCycle::default(),
            "palette_cycle must equal default"
        );
    }

    #[test]
    fn missing_glyph_loads_identity() {
        use crate::render::charset::GlyphConfig;

        // Old TOML without glyph fields must deserialize and produce GlyphConfig::default().
        let toml = r#"name = "old_no_glyph"
population = 1000
sensor_angle = 22.5
sensor_distance = 9.0
rotation_angle = 45.0
step_size = 1.0
decay_factor = 0.9
deposit_amount = 5.0
max_brightness = 100.0
diffusion_kernel = "Mean3x3"
diffusion_sigma = 1.0
palette = "Organic"
charset = "HalfBlock"
reverse_palette = false
invert_palette = false
warmup_frames = 0
food_persist = false
auto_reset = false
grid = false
init_mode = "Random"
"#;
        let cfg: SavedConfig =
            toml::from_str(toml).expect("old config without glyph fields must load");
        assert!(
            cfg.glyph_selection.is_none(),
            "missing glyph_selection must deserialize as None"
        );
        assert!(cfg.glyph_edge_threshold.is_none());

        // apply_to_runtime_state must produce GlyphConfig::default() (selection = None).
        let mut state = create_test_runtime_state();
        cfg.apply_to_runtime_state(&mut state)
            .expect("legacy config must still apply");
        assert_eq!(
            state.glyph,
            GlyphConfig::default(),
            "missing glyph keys must default to GlyphConfig::default()"
        );
        assert!(
            state.glyph.selection.is_none(),
            "missing glyph_selection must default to None"
        );
    }
}
