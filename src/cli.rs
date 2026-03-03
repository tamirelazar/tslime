use clap::Parser;
use std::num::ParseIntError;
use std::str::FromStr;

use crate::config_builder::ConfigBuilder;
use crate::config_defaults::ConfigDefaults;
use crate::render::color_constants::default as color_defaults;
use crate::render::constants::dither as dither_consts;
use crate::render::constants::intensity::{LOG_DEFAULT, PERLIN_SEED};
use crate::render::constants::rendering::{ASCII_CONTRAST_DEFAULT, GRID_OPACITY_DEFAULT};
use crate::render::dither::{DitherMatrix, DitherMode};
use crate::render::palette::RgbColor;
use crate::simulation::config::{
    DiffusionKernel, InitMode, Obstacle, Preset, SimConfig, TerrainType, Wind,
};
use crate::simulation::constants::{
    agent as agent_consts, env as env_consts, food_image as food_img_consts,
    population as pop_consts, time as time_consts, trail as trail_consts,
};

#[derive(Debug, Clone, PartialEq, Eq)]
/// Operational mode of the application.
pub enum Mode {
    /// Standard interactive mode (default).
    Default,
    /// Explicit interactive mode.
    Live,
    /// Screensaver mode (exits on input).
    Screensaver,
    /// Print a single frame and exit.
    Print,
    /// Capture a sequence of frames to files.
    CaptureFrames,
    /// Export animation to GIF.
    GifExport,
    /// Export animation to WebM.
    WebmExport,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Terminal color depth capability.
pub enum ColorMode {
    /// 24-bit True Color (16.7 million colors).
    TrueColor,
    /// Standard 3-bit color (8 colors).
    Bits8,
    /// 4-bit color (16 colors).
    Bits16,
    /// 8-bit color (256 colors).
    Bits256,
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Color palette for rendering trails.
pub enum Palette {
    /// Organic green/brown tones.
    Organic,
    /// Thermal camera style (black-red-yellow-white).
    Heat,
    /// Deep ocean blues and teals.
    Ocean,
    /// Monochrome grayscale.
    Mono,
    /// Deep forest greens.
    Forest,
    /// Cyberpunk neon colors.
    Neon,
    /// Warm earth tones.
    Warm,
    /// High saturation vibrant colors.
    Vibrant,
    /// High contrast monochrome for readability.
    LegibleMono,
    /// Radioactive green slime.
    Slime,
    /// Dark moldy colors.
    Mold,
    /// Fungal growth colors.
    Fungus,
    /// Murky swamp colors.
    Swamp,
    /// Soft mossy greens.
    Moss,
    /// Deep space purples and blues.
    Cosmic,
    /// Ghostly pale colors.
    Ethereal,
    /// Custom user-defined palette.
    Custom(Vec<RgbColor>),
}

impl Palette {
    /// Returns the string representation of the palette name.
    pub fn name(&self) -> &str {
        match self {
            Palette::Organic => "Organic",
            Palette::Heat => "Heat",
            Palette::Ocean => "Ocean",
            Palette::Mono => "Mono",
            Palette::Forest => "Forest",
            Palette::Neon => "Neon",
            Palette::Warm => "Warm",
            Palette::Vibrant => "Vibrant",
            Palette::LegibleMono => "LegibleMono",
            Palette::Slime => "Slime",
            Palette::Mold => "Mold",
            Palette::Fungus => "Fungus",
            Palette::Swamp => "Swamp",
            Palette::Moss => "Moss",
            Palette::Cosmic => "Cosmic",
            Palette::Ethereal => "Ethereal",
            Palette::Custom(_) => "Custom",
        }
    }
}

/// List of all available color palettes for cycling.
/// This is the single source of truth for palette enumeration.
pub const ALL_PALETTES: [Palette; 16] = [
    Palette::Organic,
    Palette::Heat,
    Palette::Ocean,
    Palette::Mono,
    Palette::Forest,
    Palette::Neon,
    Palette::Warm,
    Palette::Vibrant,
    Palette::LegibleMono,
    Palette::Slime,
    Palette::Mold,
    Palette::Fungus,
    Palette::Swamp,
    Palette::Moss,
    Palette::Cosmic,
    Palette::Ethereal,
];

/// The number of palettes in ALL_PALETTES.
pub const NUM_PALETTES: usize = ALL_PALETTES.len();

/// Returns the number of available palettes.
pub fn num_palettes() -> usize {
    NUM_PALETTES
}

#[derive(Debug, Clone)]
/// Simulation grid resolution.
pub struct Resolution {
    /// Width of the simulation grid.
    pub width: usize,
    /// Height of the simulation grid.
    pub height: usize,
}

#[derive(Debug, Clone)]
/// Configuration for a specific agent species.
pub struct SpeciesArg {
    /// Name of the species.
    pub name: String,
    /// Number of agents.
    pub count: usize,
    /// Sensor angle in degrees.
    pub sensor_angle: f32,
    /// Maximum rotation angle in degrees.
    pub rotation_angle: f32,
    /// Movement speed.
    pub step_size: f32,
    /// Amount of pheromone deposited.
    pub deposit_amount: f32,
    /// Hex color code.
    pub color: String,
}

impl std::str::FromStr for SpeciesArg {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let name_and_count_parts: Vec<&str> = s.splitn(2, ':').collect();
        if name_and_count_parts.len() < 2 {
            return Err(format!(
                "Species must be in format 'name:count@params:color' or 'name:count:color', got: {}", s
            ));
        }

        let name = name_and_count_parts[0].to_string();
        let rest = name_and_count_parts[1];

        let mut sensor_angle = agent_consts::DEFAULT_SENSOR_ANGLE;
        let mut rotation_angle = agent_consts::DEFAULT_ROTATION_ANGLE;
        let mut step_size = agent_consts::DEFAULT_STEP_SIZE;
        let mut deposit_amount = agent_consts::DEFAULT_DEPOSIT_AMOUNT;
        let mut color = color_defaults::FOREST_GREEN.to_string();

        if rest.contains('@') {
            let count_and_rest: Vec<&str> = rest.splitn(2, '@').collect();
            let count_str = count_and_rest[0];
            let count = parse_count(count_str)?;

            if count_and_rest.len() >= 2 {
                let params_and_color = count_and_rest[1];
                if params_and_color.contains(':') {
                    let params_parts: Vec<&str> = params_and_color.rsplitn(2, ':').collect();
                    if params_parts.len() == 2 {
                        let params = params_parts[1];
                        let color_part = params_parts[0];

                        if color_part.starts_with('#') || color_part.len() == 6 {
                            color = color_part.trim_start_matches('#').to_string();
                        }

                        let param_values: Vec<&str> = params.split(',').collect();
                        if !param_values.is_empty() {
                            if let Ok(v) = param_values[0].parse::<f32>() {
                                sensor_angle = v;
                            }
                        }
                        if param_values.len() >= 2 {
                            if let Ok(v) = param_values[1].parse::<f32>() {
                                rotation_angle = v;
                            }
                        }
                        if param_values.len() >= 3 {
                            if let Ok(v) = param_values[2].parse::<f32>() {
                                step_size = v;
                            }
                        }
                        if param_values.len() >= 4 {
                            if let Ok(v) = param_values[3].parse::<f32>() {
                                deposit_amount = v;
                            }
                        }
                    }
                } else if params_and_color.starts_with('#') || params_and_color.len() == 6 {
                    color = params_and_color.trim_start_matches('#').to_string();
                }
            }

            Ok(SpeciesArg {
                name,
                count,
                sensor_angle,
                rotation_angle,
                step_size,
                deposit_amount,
                color,
            })
        } else {
            let parts: Vec<&str> = rest.rsplitn(2, ':').collect();
            let count_str;
            if parts.len() == 2 {
                count_str = parts[1];
                let color_part = parts[0];
                if color_part.starts_with('#') || color_part.len() == 6 {
                    color = color_part.trim_start_matches('#').to_string();
                }
            } else {
                count_str = parts[0];
            }
            let count = parse_count(count_str)?;

            Ok(SpeciesArg {
                name,
                count,
                sensor_angle,
                rotation_angle,
                step_size,
                deposit_amount,
                color,
            })
        }
    }
}

fn parse_count(s: &str) -> Result<usize, String> {
    if s.ends_with('k') || s.ends_with('K') {
        let num = &s[..s.len() - 1];
        let val = num
            .parse::<f64>()
            .map_err(|e| format!("Invalid count: {}", e))?;
        Ok((val * 1000.0) as usize)
    } else if s.ends_with('m') || s.ends_with('M') {
        let num = &s[..s.len() - 1];
        let val = num
            .parse::<f64>()
            .map_err(|e| format!("Invalid count: {}", e))?;
        Ok((val * 1000000.0) as usize)
    } else {
        s.parse::<usize>()
            .map_err(|e| format!("Invalid count: {}", e))
    }
}

impl std::str::FromStr for Resolution {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split('x').collect();
        if parts.len() != 2 {
            return Err(format!("Resolution must be in WxH format, got: {}", s));
        }
        let width = parts[0]
            .parse::<usize>()
            .map_err(|e: ParseIntError| format!("Invalid width: {}", e))?;
        let height = parts[1]
            .parse::<usize>()
            .map_err(|e: ParseIntError| format!("Invalid height: {}", e))?;
        Ok(Resolution { width, height })
    }
}

impl FromStr for Preset {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "network" => Ok(Preset::Network),
            "exploratory" => Ok(Preset::Exploratory),
            "tendrils" => Ok(Preset::Tendrils),
            "organic" => Ok(Preset::Organic),
            "minimal" => Ok(Preset::Minimal),
            "moss" => Ok(Preset::Moss),
            "cosmic" => Ok(Preset::Cosmic),
            "fire" => Ok(Preset::Fire),
            "zen" => Ok(Preset::Zen),
            "storm" => Ok(Preset::Storm),
            "river" => Ok(Preset::River),
            "ethereal" => Ok(Preset::Ethereal),
            "petri" | "petridish" => Ok(Preset::PetriDish),
            "vortex" => Ok(Preset::Vortex),
            "lightning" => Ok(Preset::Lightning),
            "crystal" => Ok(Preset::Crystal),
            "chaosedge" | "chaos-edge" | "chaos_edge" => Ok(Preset::ChaosEdge),
            "blob" => Ok(Preset::Blob),
            "worm" => Ok(Preset::Worm),
            _ => Err(format!(
                "Invalid preset: {}. Must be one of: network, exploratory, tendrils, organic, minimal, moss, cosmic, fire, zen, storm, river, ethereal, petri, vortex, lightning, crystal, chaosedge, blob, worm",
                s
            )),
        }
    }
}

impl FromStr for InitMode {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "random" => Ok(InitMode::Random),
            "central" => Ok(InitMode::CentralBurst),
            "circle" => Ok(InitMode::Circle),
            "gradient" => Ok(InitMode::Gradient),
            "wave" => Ok(InitMode::WaveFront),
            "spiral" => Ok(InitMode::Spiral),
            "clusters" => Ok(InitMode::RandomClusters),
            "petri" => Ok(InitMode::Petri),
            "food" => Ok(InitMode::Food),
            _ => Err(format!(
                "Invalid init mode: {}. Must be one of: random, central, circle, gradient, wave, spiral, clusters, petri, food",
                s
            )),
        }
    }
}

#[derive(Debug, Clone)]
/// Configuration for a point attractor/repeller.
pub struct AttractorArg {
    /// X coordinate.
    pub x: f32,
    /// Y coordinate.
    pub y: f32,
    /// Strength (positive = attract, negative = repel).
    pub strength: f32,
}

impl std::str::FromStr for AttractorArg {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split(',').collect();
        if parts.len() != 3 {
            return Err(format!(
                "Attractor must be in x,y,strength format, got: {}",
                s
            ));
        }

        let x = parts[0]
            .parse::<f32>()
            .map_err(|e| format!("Invalid x coordinate: {}", e))?;
        let y = parts[1]
            .parse::<f32>()
            .map_err(|e| format!("Invalid y coordinate: {}", e))?;
        let strength = parts[2]
            .parse::<f32>()
            .map_err(|e| format!("Invalid strength: {}", e))?;

        Ok(AttractorArg { x, y, strength })
    }
}

#[derive(Debug, Clone)]
/// Configuration for wind force.
pub struct WindArg {
    /// Horizontal wind component.
    pub dx: f32,
    /// Vertical wind component.
    pub dy: f32,
}

impl std::str::FromStr for WindArg {
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
        Ok(WindArg { dx, dy })
    }
}

#[derive(Debug, Clone)]
/// Configuration for a physical obstacle.
pub struct ObstacleArg {
    /// The obstacle definition.
    pub obstacle: Obstacle,
}

impl std::str::FromStr for ObstacleArg {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, String> {
        if let Some(circle_part) = s.strip_prefix("circle:") {
            let parts: Vec<&str> = circle_part.split(',').collect();
            if parts.len() != 3 {
                return Err(format!(
                    "Circle obstacle must be in circle:x,y,r format, got: {}",
                    s
                ));
            }
            let x = parts[0]
                .parse::<f32>()
                .map_err(|e| format!("Invalid x coordinate: {}", e))?;
            let y = parts[1]
                .parse::<f32>()
                .map_err(|e| format!("Invalid y coordinate: {}", e))?;
            let radius = parts[2]
                .parse::<f32>()
                .map_err(|e| format!("Invalid radius: {}", e))?;
            if radius <= 0.0 {
                return Err(format!("Radius must be positive, got: {}", radius));
            }
            Ok(ObstacleArg {
                obstacle: Obstacle::Circle { x, y, radius },
            })
        } else if let Some(rect_part) = s.strip_prefix("rect:") {
            let parts: Vec<&str> = rect_part.split(',').collect();
            if parts.len() != 4 {
                return Err(format!(
                    "Rect obstacle must be in rect:x,y,w,h format, got: {}",
                    s
                ));
            }
            let x = parts[0]
                .parse::<f32>()
                .map_err(|e| format!("Invalid x coordinate: {}", e))?;
            let y = parts[1]
                .parse::<f32>()
                .map_err(|e| format!("Invalid y coordinate: {}", e))?;
            let width = parts[2]
                .parse::<f32>()
                .map_err(|e| format!("Invalid width: {}", e))?;
            let height = parts[3]
                .parse::<f32>()
                .map_err(|e| format!("Invalid height: {}", e))?;
            if width <= 0.0 {
                return Err(format!("Width must be positive, got: {}", width));
            }
            if height <= 0.0 {
                return Err(format!("Height must be positive, got: {}", height));
            }
            Ok(ObstacleArg {
                obstacle: Obstacle::Rect {
                    x,
                    y,
                    width,
                    height,
                },
            })
        } else if let Some(img_part) = s.strip_prefix("image:") {
            let parts: Vec<&str> = img_part.split(',').collect();
            if parts.len() != 7 {
                return Err(format!(
                    "Image obstacle must be in image:path,x,y,w,h,invert,threshold format, got: {}",
                    s
                ));
            }
            let path = parts[0].to_string();
            let x = parts[1]
                .parse::<f32>()
                .map_err(|e| format!("Invalid x coordinate: {}", e))?;
            let y = parts[2]
                .parse::<f32>()
                .map_err(|e| format!("Invalid y coordinate: {}", e))?;
            let width = parts[3]
                .parse::<usize>()
                .map_err(|e| format!("Invalid width: {}", e))?;
            let height = parts[4]
                .parse::<usize>()
                .map_err(|e| format!("Invalid height: {}", e))?;
            let invert = parts[5]
                .parse::<bool>()
                .map_err(|e| format!("Invalid invert: {}", e))?;
            let threshold = parts[6]
                .parse::<f32>()
                .map_err(|e| format!("Invalid threshold: {}", e))?;
            if width == 0 || height == 0 {
                return Err(format!(
                    "Width and height must be positive, got: {}x{}",
                    width, height
                ));
            }
            if !(0.0..=1.0).contains(&threshold) {
                return Err(format!(
                    "Threshold must be between 0.0 and 1.0, got: {}",
                    threshold
                ));
            }
            Ok(ObstacleArg {
                obstacle: Obstacle::Image {
                    path,
                    x,
                    y,
                    width,
                    height,
                    invert,
                    threshold,
                },
            })
        } else {
            Err(format!(
                "Obstacle must start with 'circle:', 'rect:', or 'image:', got: {}",
                s
            ))
        }
    }
}

impl FromStr for DiffusionKernel {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "mean3x3" | "mean" | "3x3" => Ok(DiffusionKernel::Mean3x3),
            "gaussian" => Ok(DiffusionKernel::Gaussian),
            _ => Err(format!(
                "Invalid diffusion kernel: {}. Must be one of: mean3x3, gaussian",
                s
            )),
        }
    }
}

#[derive(Parser, Debug, Clone)]
#[command(name = "tslime")]
#[command(about = "Terminal physarum simulation screensaver", long_about = None)]
#[command(version)]
/// Command-line arguments for configuring the simulation.
pub struct Args {
    #[arg(short = 'l', long = "live", help = "Continuous animation mode")]
    /// Continuous animation mode (interactive).
    pub live: bool,

    #[arg(
        short = 'S',
        long = "screensaver",
        help = "Screensaver mode (exit on keypress)"
    )]
    /// Screensaver mode - exits on any keypress.
    pub screensaver: bool,

    #[arg(short = 'p', long = "print", help = "Print single frame and exit")]
    /// Print a single frame and exit (non-interactive).
    pub print: bool,

    #[arg(
        long = "explore",
        help = "Run parameter space exploration to find optimal presets"
    )]
    /// Run parameter space exploration mode.
    pub explore: bool,

    #[arg(
        long = "explore-behavior",
        value_name = "BEHAVIOR",
        help = "Target behavior for exploration (vortex, lightning, crystal, blob, worm, chaosedge, all)"
    )]
    /// Target behavior to optimize for during exploration.
    pub explore_behavior: Option<String>,

    #[arg(
        long = "explore-iterations",
        value_name = "INT",
        default_value_t = ConfigDefaults::EXPLORE_ITERATIONS,
        help = "Number of iterations for parameter exploration"
    )]
    /// Number of iterations for parameter exploration.
    pub explore_iterations: usize,

    #[arg(
        long = "capture-frames",
        help = "Capture simulation frames for GIF generation"
    )]
    /// Capture simulation frames to text files.
    pub capture_frames: bool,

    #[arg(
        long = "frame-count",
        value_name = "INT",
        default_value_t = ConfigDefaults::FRAME_COUNT,
        help = "Number of frames to capture"
    )]
    /// Number of frames to capture.
    pub frame_count: usize,

    #[arg(
        long = "frame-skip",
        value_name = "INT",
        default_value_t = ConfigDefaults::FRAME_SKIP,
        help = "Simulation steps between captured frames"
    )]
    /// Number of simulation steps to skip between captured frames.
    pub frame_skip: usize,

    #[arg(
        long = "frame-dir",
        value_name = "PATH",
        default_value = ConfigDefaults::FRAME_DIR,
        help = "Directory to save captured frames"
    )]
    /// Directory to save captured frames.
    pub frame_dir: String,

    #[arg(
        short = 's',
        long = "seed",
        value_name = "INT",
        help = "Random seed for reproducibility"
    )]
    /// Random seed for reproducible results.
    pub seed: Option<u64>,

    #[arg(
        short = 'n',
        long = "population",
        value_name = "INT",
        help = concat!("Number of agents [default: ", stringify!(ConfigDefaults::POPULATION), "]")
    )]
    /// Number of agents in the simulation.
    pub population: Option<usize>,

    #[arg(
        long = "sensor-angle",
        value_name = "DEG",
        help = concat!("Sensor spread angle in degrees [range: ", stringify!(ConfigDefaults::MIN_SENSOR_ANGLE), "-", stringify!(ConfigDefaults::MAX_SENSOR_ANGLE), "]")
    )]
    /// Angle between sensors (in degrees).
    pub sensor_angle: Option<f32>,

    #[arg(
        long = "sensor-distance",
        value_name = "FLOAT",
        help = concat!("Sensor range in pixels [range: ", stringify!(ConfigDefaults::MIN_SENSOR_DISTANCE), "-", stringify!(ConfigDefaults::MAX_SENSOR_DISTANCE), "]")
    )]
    /// Distance to sensors.
    pub sensor_distance: Option<f32>,

    #[arg(
        long = "rotation-angle",
        value_name = "DEG",
        help = concat!("Turn amount per step in degrees [range: ", stringify!(ConfigDefaults::MIN_ROTATION_ANGLE), "-", stringify!(ConfigDefaults::MAX_ROTATION_ANGLE), "]")
    )]
    /// Maximum rotation angle per step (in degrees).
    pub rotation_angle: Option<f32>,

    #[arg(
        long = "step-size",
        value_name = "FLOAT",
        help = concat!("Movement speed in pixels per step [range: ", stringify!(ConfigDefaults::MIN_STEP_SIZE), "-", stringify!(ConfigDefaults::MAX_STEP_SIZE), "]")
    )]
    /// Distance moved per step.
    pub step_size: Option<f32>,

    #[arg(
        long = "decay",
        value_name = "FLOAT",
        help = concat!("Trail decay factor (0.0-1.0) [range: ", stringify!(ConfigDefaults::MIN_DECAY_FACTOR), "-", stringify!(ConfigDefaults::MAX_DECAY_FACTOR), "]")
    )]
    /// Trail decay factor (0.0-1.0).
    pub decay_factor: Option<f32>,

    #[arg(
        long = "deposit",
        value_name = "FLOAT",
        help = concat!("Amount of pheromone deposited by agents per step [range: ", stringify!(ConfigDefaults::MIN_DEPOSIT_AMOUNT), "-", stringify!(ConfigDefaults::MAX_DEPOSIT_AMOUNT), "]")
    )]
    /// Amount of pheromone deposited per step.
    pub deposit_amount: Option<f32>,

    #[arg(
        long = "max-brightness",
        value_name = "FLOAT",
        help = concat!("Fixed maximum brightness for normalization (prevents flickering) [range: ", stringify!(ConfigDefaults::MIN_MAX_BRIGHTNESS), "-", stringify!(ConfigDefaults::MAX_MAX_BRIGHTNESS), "]")
    )]
    /// Maximum brightness for normalization.
    pub max_brightness: Option<f32>,

    #[arg(
        long = "diffusion-kernel",
        value_name = "TYPE",
        help = "Diffusion kernel type (mean3x3, gaussian)"
    )]
    /// Diffusion kernel type (mean3x3 or gaussian).
    pub diffusion_kernel: Option<DiffusionKernel>,

    #[arg(
        long = "diffusion-sigma",
        value_name = "FLOAT",
        help = "Gaussian kernel sigma value (0.5-2.0, only used with gaussian kernel)"
    )]
    /// Sigma value for Gaussian diffusion.
    pub diffusion_sigma: Option<f32>,

    #[arg(
        long = "preset",
        value_name = "NAME",
        help = "Use named preset (network, exploratory, tendrils, organic, minimal, moss, cosmic, fire, zen, storm, river, ethereal, vortex, lightning, crystal, chaosedge, blob, worm)"
    )]
    /// Named parameter preset.
    pub preset: Option<Preset>,

    #[arg(
        long = "init",
        value_name = "MODE",
        help = "Initialization mode (random, central, circle, gradient, wave, spiral, clusters, petri, food)"
    )]
    /// Agent initialization pattern.
    pub init: Option<InitMode>,

    #[arg(
        long = "food",
        value_name = "PATH",
        default_value = ConfigDefaults::FOOD_PATH,
        help = "Load agents from PNG image. High-brightness areas spawn more agents. Use with --init food"
    )]
    /// Path to image for food-based initialization.
    pub food: String,

    #[arg(
        long = "food-invert",
        value_name = "BOOL",
        num_args = 1,
        default_value_t = ConfigDefaults::FOOD_INVERT,
        help = "Invert the food image values (dark areas spawn more agents instead of bright areas)"
    )]
    /// Invert food image brightness.
    pub food_invert: bool,

    #[arg(
        long = "food-scale",
        value_name = "FLOAT",
        default_value_t = ConfigDefaults::FOOD_SCALE,
        help = concat!("Scale factor for food image relative to canvas [default: ", stringify!(ConfigDefaults::FOOD_SCALE), "]")
    )]
    /// Scale factor for food image.
    pub food_scale: f32,

    #[arg(
        short = 't',
        long = "time",
        value_name = "FLOAT",
        default_value_t = ConfigDefaults::FRAME_DELAY,
        help = concat!("Frame delay in seconds [default: ", stringify!(ConfigDefaults::FRAME_DELAY), "]")
    )]
    /// Frame delay in seconds.
    pub frame_delay: f32,

    #[arg(
        long = "fps",
        value_name = "INT",
        default_value_t = ConfigDefaults::FPS,
        help = concat!("Target frames per second [default: ", stringify!(ConfigDefaults::FPS), "]")
    )]
    /// Target FPS.
    pub fps: usize,

    #[arg(
        long = "time-scale",
        value_name = "FLOAT",
        default_value_t = ConfigDefaults::TIME_SCALE,
        help = concat!("Time scaling factor [range: ", stringify!(ConfigDefaults::MIN_TIME_SCALE), "-", stringify!(ConfigDefaults::MAX_TIME_SCALE), "]")
    )]
    /// Simulation time scale.
    pub time_scale: f32,

    #[arg(
        long = "resolution",
        value_name = "WxH",
        default_value = "400x200",
        help = "Simulation resolution"
    )]
    /// Simulation grid resolution.
    pub resolution: Resolution,

    #[arg(
        long = "palette",
        value_name = "NAME",
        default_value = ConfigDefaults::PALETTE,
        help = "Color palette (organic, heat, ocean, mono, forest, neon, warm, vibrant, legiblemono, slime, mold, fungus, swamp, moss, cosmic, ethereal) or custom: \"#rrggbb,#rrggbb,...\" (2-11 colors)"
    )]
    /// Color palette name or definition.
    pub palette: String,

    #[arg(
        long = "colors",
        value_name = "MODE",
        default_value = "true",
        help = "Color mode (8, 16, 256, true)"
    )]
    /// Terminal color depth mode.
    pub colors: String,

    #[arg(long = "ascii", help = "Use ASCII characters only")]
    /// Force ASCII character set.
    pub ascii: bool,

    #[arg(long = "braille", help = "Use braille characters")]
    /// Force Braille character set.
    pub braille: bool,

    #[arg(
        long = "quadrant",
        help = "Use Unicode quadrant characters for 4× vertical resolution"
    )]
    /// Force Quadrant character set.
    pub quadrant: bool,

    #[arg(
        long = "shade",
        help = "Use shade characters (░▒▓█) for smooth density gradients"
    )]
    /// Force Shade character set for smooth density visualization.
    pub shade: bool,

    #[arg(
        long = "points",
        help = "Use point grid (▪) for sparse particle visualization"
    )]
    /// Force Points character set for sparse/stipple visualization.
    pub points: bool,

    #[arg(
        long = "half-block-dual",
        alias = "hbd",
        help = "Use dual-color half-block mode (▀ with independent fg/bg colors for true 2× vertical color resolution)"
    )]
    /// Dual-color half-block mode for maximum color fidelity.
    pub half_block_dual: bool,

    #[arg(
        long = "sculpted",
        help = "Sculpted mode: solid interior blocks with shape-aware outline characters"
    )]
    /// Sculpted charset mode with smooth outline rendering.
    pub sculpted: bool,

    #[arg(
        long = "plain-output",
        help = "Output plain text without ANSI color codes (for frame capture)"
    )]
    /// Output without ANSI color codes.
    pub plain_output: bool,

    #[arg(
        short = 'v',
        long = "verbose",
        help = "Print performance stats to stderr"
    )]
    /// Enable verbose logging.
    pub verbose: bool,

    #[arg(
        long = "reverse-palette",
        help = "Reverse palette order (dark becomes light, light becomes dark)"
    )]
    /// Reverse the color palette.
    pub reverse_palette: bool,

    #[arg(
        long = "invert-palette",
        help = "Invert palette colors (complementary colors)"
    )]
    /// Invert palette colors.
    pub invert_palette: bool,

    #[arg(
        long = "palette-shift",
        value_name = "DEG",
        default_value = "0",
        help = "Rotate palette hue over time (degrees per second, negative for reverse rotation)"
    )]
    /// Palette hue shift speed (degrees/sec).
    pub palette_shift: f32,

    #[arg(
        long = "intensity-mapping",
        value_name = "MODE",
        default_value = ConfigDefaults::INTENSITY_MAPPING,
        help = "Intensity-to-color mapping (linear, log, exp, sqrt, square, sigmoid, smoothstep, quantize, perlin, split)"
    )]
    /// Intensity mapping mode for non-linear color distribution.
    pub intensity_mapping: String,

    #[arg(
        long = "intensity-mapping-base",
        value_name = "FLOAT",
        default_value_t = ConfigDefaults::INTENSITY_MAPPING_BASE,
        help = concat!("Base parameter for log/exp mapping [default: ", stringify!(ConfigDefaults::INTENSITY_MAPPING_BASE), "]")
    )]
    /// Base for logarithmic/exponential intensity mapping.
    pub intensity_mapping_base: f32,

    #[arg(
        long = "intensity-mapping-gamma",
        value_name = "FLOAT",
        default_value_t = ConfigDefaults::INTENSITY_MAPPING_GAMMA,
        help = concat!("Gamma for power mapping [default: ", stringify!(ConfigDefaults::INTENSITY_MAPPING_GAMMA), "]")
    )]
    /// Gamma for power intensity mapping.
    pub intensity_mapping_gamma: f32,

    #[arg(
        long = "intensity-mapping-levels",
        value_name = "INT",
        default_value_t = ConfigDefaults::INTENSITY_MAPPING_LEVELS,
        help = concat!("Levels for quantize mapping [default: ", stringify!(ConfigDefaults::INTENSITY_MAPPING_LEVELS), "]")
    )]
    /// Quantization levels for intensity mapping.
    pub intensity_mapping_levels: u8,

    #[arg(
        long = "perlin-strength",
        value_name = "FLOAT",
        default_value_t = ConfigDefaults::PERLIN_STRENGTH,
        help = concat!("Perlin noise amplitude/strength (0.0-1.0) [default: ", stringify!(ConfigDefaults::PERLIN_STRENGTH), "]")
    )]
    /// Amplitude for perlin intensity mapping (affects both sim and logo).
    pub perlin_strength: f32,

    #[arg(
        long = "logo-mapping",
        value_name = "MODE",
        default_value = ConfigDefaults::INTENSITY_MAPPING,
        help = "Intensity mapping for pause logo (linear, log, exp, sqrt, square, sigmoid, smoothstep, quantize, perlin, split, sim=use sim mapping)"
    )]
    /// Intensity mapping for the pause logo. Defaults to "log".
    pub logo_mapping: String,

    #[arg(
        long = "logo-mapping-base",
        value_name = "FLOAT",
        default_value_t = ConfigDefaults::LOGO_MAPPING_BASE,
        help = concat!("Base for logo log/exp mapping [default: ", stringify!(ConfigDefaults::LOGO_MAPPING_BASE), "]")
    )]
    /// Base for the logo's logarithmic/exponential mapping.
    pub logo_mapping_base: f32,

    #[arg(
        long = "trail-history",
        value_name = "INT",
        default_value = "0",
        help = "Number of historical frames to blend for motion blur (0=disabled, max 10)"
    )]
    /// Number of frames to blend for trails.
    pub trail_history: usize,

    #[arg(
        long = "motion-blur",
        help = "Enable motion blur effect (equivalent to --trail-history 3)"
    )]
    /// Enable simple motion blur.
    pub motion_blur: bool,

    #[arg(
        long = "auto-normalize",
        help = "Enable adaptive brightness normalization to prevent flickering"
    )]
    /// Enable adaptive brightness normalization.
    pub auto_normalize: bool,

    #[arg(
        long = "normalize-window",
        value_name = "INT",
        default_value = "30",
        help = "Number of frames for adaptive brightness normalization window (1-100)"
    )]
    /// Window size for brightness normalization.
    pub normalize_window: usize,

    #[arg(
        long = "attract",
        value_name = "X,Y,STRENGTH",
        help = "Add point attractor (positive=attract, negative=repel). Can be specified multiple times. Example: --attract 200,200,1.0"
    )]
    /// List of point attractors.
    pub attract: Vec<AttractorArg>,

    #[arg(
        long = "obstacle",
        value_name = "TYPE:X,Y,PARAMS",
        help = "Add obstacle (circle:x,y,r or rect:x,y,w,h or image:path,x,y,w,h,invert,threshold). Can be specified multiple times. Example: --obstacle circle:200,200,50"
    )]
    /// List of obstacles.
    pub obstacle: Vec<ObstacleArg>,

    #[arg(
        long = "attractor-strength",
        value_name = "FLOAT",
        default_value_t = ConfigDefaults::ATTRACTOR_STRENGTH,
        help = concat!("Global multiplier for attractor/repeller strength [range: ", stringify!(ConfigDefaults::MIN_ATTRACTOR_STRENGTH), "-", stringify!(ConfigDefaults::MAX_ATTRACTOR_STRENGTH), "]")
    )]
    /// Global strength multiplier for attractors.
    pub attractor_strength: f32,

    #[arg(
        long = "dither-mode",
        value_name = "MODE",
        default_value = ConfigDefaults::DITHER_MODE,
        help = "Dithering mode: none, ordered, error-diffusion, hybrid"
    )]
    /// Dithering algorithm mode.
    pub dither_mode: String,

    #[arg(
        long = "dither-intensity",
        value_name = "FLOAT",
        default_value_t = ConfigDefaults::DITHER_INTENSITY,
        help = concat!("Dithering intensity for ordered/hybrid modes (0.0-1.0) [default: ", stringify!(ConfigDefaults::DITHER_INTENSITY), "]")
    )]
    /// Intensity of dithering effect.
    pub dither_intensity: f32,

    #[arg(
        long = "dither-matrix",
        value_name = "MATRIX",
        default_value = ConfigDefaults::DITHER_MATRIX,
        help = "Dither matrix for ordered mode: 4x4, 8x8"
    )]
    /// Matrix size for ordered dithering.
    pub dither_matrix: String,

    #[arg(
        long = "dither-swap",
        help = "Swap to next dither mode (cycle through none -> ordered -> error-diffusion -> hybrid)"
    )]
    /// Cycle through dither modes.
    pub dither_swap: bool,

    #[arg(long = "error-diffusion-swap", help = "Toggle error diffusion mode")]
    /// Toggle error diffusion dithering.
    pub error_diffusion_swap: bool,

    #[arg(
        long = "species",
        value_name = "SPEC",
        help = "Define agent species with format 'name:count@sensor_angle,rotation_angle,step_size,deposit:color'. Can be specified multiple times or comma-separated. Example: --species 'red:20k@22.5,45,1.0,5.0:ff0000,blue:30k@30,60,1.5,3.0:0000ff'"
    )]
    /// List of agent species.
    pub species: Vec<SpeciesArg>,

    #[arg(
        long = "separate-species-trails",
        help = "Each species maintains its own separate trail map (higher memory, allows species-specific patterns)"
    )]
    /// Use separate trail maps for each species.
    pub separate_species_trails: bool,

    #[arg(
        long = "species-colors",
        help = "Enable species-specific rendering using each species' configured color. Automatically enables --separate-species-trails."
    )]
    /// Render using species-specific colors.
    pub species_colors: bool,

    #[arg(
        long = "simd-off",
        help = "Disable SIMD acceleration for diffusion (use scalar fallback)"
    )]
    /// Disable SIMD acceleration.
    pub simd_off: bool,

    #[arg(
        long = "wind",
        value_name = "DX,DY",
        help = "Apply constant wind force (dx,dy from -1.0 to 1.0). Example: --wind 0.5,0.0 for rightward wind"
    )]
    /// Constant wind force vector.
    pub wind: Option<WindArg>,

    #[arg(
        long = "terrain",
        value_name = "TYPE",
        default_value = "none",
        help = "Terrain type for organic movement patterns: none, smooth, turbulent, mixed"
    )]
    /// Terrain type for organic movement.
    pub terrain: String,

    #[arg(
        long = "terrain-strength",
        value_name = "FLOAT",
        default_value_t = ConfigDefaults::TERRAIN_STRENGTH,
        help = concat!("Strength of terrain influence [range: ", stringify!(ConfigDefaults::MIN_TERRAIN_STRENGTH), "-", stringify!(ConfigDefaults::MAX_TERRAIN_STRENGTH), "]")
    )]
    /// Strength of terrain effect.
    pub terrain_strength: f32,

    #[arg(
        long = "export-gif",
        value_name = "PATH",
        help = "Export simulation to GIF file"
    )]
    /// Path for GIF export.
    pub export_gif: Option<String>,

    #[arg(
        long = "export-webm",
        value_name = "PATH",
        help = "Export simulation to WebM video file (requires FFmpeg)"
    )]
    /// Path for WebM export.
    pub export_webm: Option<String>,

    #[arg(
        long = "export-frames",
        value_name = "INT",
        default_value_t = ConfigDefaults::EXPORT_FRAMES,
        help = concat!("Number of frames to capture for GIF export [default: ", stringify!(ConfigDefaults::EXPORT_FRAMES), "]")
    )]
    /// Number of frames to export.
    pub export_frames: usize,

    #[arg(
        long = "export-fps",
        value_name = "INT",
        default_value_t = ConfigDefaults::EXPORT_FPS,
        help = concat!("GIF playback speed (frames per second) [default: ", stringify!(ConfigDefaults::EXPORT_FPS), "]")
    )]
    /// FPS for exported animation.
    pub export_fps: usize,

    #[arg(
        long = "mouse-attract",
        help = "Enable mouse clicks to create temporary attractors at cursor position"
    )]
    /// Enable mouse attraction.
    pub mouse_attract: bool,

    #[arg(
        long = "mouse-repel",
        help = "Enable mouse clicks to create temporary repellers at cursor position"
    )]
    /// Enable mouse repulsion.
    pub mouse_repel: bool,

    #[arg(
        long = "mouse-timeout",
        value_name = "FLOAT",
        default_value_t = ConfigDefaults::MOUSE_TIMEOUT,
        help = concat!("Time in seconds before mouse-created attractors/repellers expire [default: ", stringify!(ConfigDefaults::MOUSE_TIMEOUT), "]")
    )]
    /// Duration of mouse effects.
    pub mouse_timeout: f32,

    #[arg(long = "stats", help = "Display real-time statistics overlay")]
    /// Show performance statistics.
    pub stats: bool,

    #[arg(
        long = "auto-fps",
        value_name = "BOOL",
        default_value_t = false,
        help = "Enable automatic FPS adjustment when performance drops"
    )]
    /// Enable automatic FPS adjustment.
    pub auto_fps: bool,

    // ===== Warmup frames =====
    #[arg(
        long = "warmup-frames",
        value_name = "INT",
        default_value_t = ConfigDefaults::WARMUP_FRAMES,
        help = concat!("Number of frames to display logo before simulation (0 to disable) [default: ", stringify!(ConfigDefaults::WARMUP_FRAMES), "]")
    )]
    /// Number of warmup frames.
    pub warmup_frames: usize,

    #[arg(
        long = "warmup-brightness",
        value_name = "FLOAT",
        default_value_t = ConfigDefaults::WARMUP_BRIGHTNESS_MULTIPLIER,
        help = concat!("Brightness multiplier during warmup phase [default: ", stringify!(ConfigDefaults::WARMUP_BRIGHTNESS_MULTIPLIER), "]")
    )]
    /// Brightness multiplier during warmup.
    pub warmup_brightness_multiplier: f32,

    #[arg(
        long = "warmup-decay",
        value_name = "FLOAT",
        default_value_t = ConfigDefaults::WARMUP_DECAY,
        help = concat!("Decay factor during warmup (higher = logo persists longer) [default: ", stringify!(ConfigDefaults::WARMUP_DECAY), "]")
    )]
    /// Trail decay during warmup.
    pub warmup_decay: f32,

    #[arg(long = "skip-warmup", help = "Skip warmup phase (useful for exports)")]
    /// Skip the warmup phase.
    pub skip_warmup: bool,

    // ===== Food persistence =====
    #[arg(
        long = "food-persist",
        help = "Keep agents near original food/logo location using attractors"
    )]
    /// Enable food persistence.
    pub food_persist: bool,

    #[arg(
        long = "food-persist-strength",
        value_name = "FLOAT",
        default_value_t = ConfigDefaults::FOOD_PERSIST_STRENGTH,
        help = concat!("Strength of food persistence attractors (0.0-5.0) [default: ", stringify!(ConfigDefaults::FOOD_PERSIST_STRENGTH), "]")
    )]
    /// Strength of food persistence.
    pub food_persist_strength: f32,

    #[arg(
        long = "food-persist-radius",
        value_name = "FLOAT",
        default_value_t = ConfigDefaults::FOOD_PERSIST_RADIUS,
        help = concat!("Radius of influence for food persistence attractors [default: ", stringify!(ConfigDefaults::FOOD_PERSIST_RADIUS), "]")
    )]
    /// Radius of food persistence.
    pub food_persist_radius: f32,

    #[arg(
        long = "food-persist-duration",
        value_name = "INT",
        default_value_t = ConfigDefaults::FOOD_PERSIST_DURATION,
        help = concat!("Number of frames before food attractors fade out (0 = permanent) [default: ", stringify!(ConfigDefaults::FOOD_PERSIST_DURATION), "]")
    )]
    /// Duration of food persistence.
    pub food_persist_duration: usize,

    // ===== Entropy-based auto-reset =====
    #[arg(
        long = "auto-reset",
        help = "Automatically restart simulation when it collapses",
        default_value = "false"
    )]
    /// Enable auto-reset on collapse.
    pub auto_reset: bool,

    #[arg(
        long = "collapse-threshold",
        value_name = "FLOAT",
        default_value_t = ConfigDefaults::COLLAPSE_ENTROPY_THRESHOLD,
        help = concat!("Entropy threshold to detect collapse (0.0-1.0, higher = more sensitive) [default: ", stringify!(ConfigDefaults::COLLAPSE_ENTROPY_THRESHOLD), "]")
    )]
    /// Entropy threshold for collapse detection.
    pub collapse_entropy_threshold: f32,

    #[arg(
        long = "collapse-duration",
        value_name = "INT",
        default_value_t = ConfigDefaults::COLLAPSE_DURATION_FRAMES,
        help = concat!("Number of frames simulation must stay collapsed before auto-reset [default: ", stringify!(ConfigDefaults::COLLAPSE_DURATION_FRAMES), "]")
    )]
    /// Duration to wait before reset.
    pub collapse_duration_frames: usize,

    // ===== Background grid =====
    #[arg(long = "grid", help = "Enable background grid rendering")]
    /// Enable background grid.
    pub grid: bool,

    #[arg(
        long = "grid-size",
        value_name = "INT",
        default_value_t = ConfigDefaults::GRID_SIZE,
        help = concat!("Grid cell size (number of cells per dimension) [default: ", stringify!(ConfigDefaults::GRID_SIZE), "]")
    )]
    /// Grid cell size.
    pub grid_size: usize,

    #[arg(
        long = "grid-style",
        value_name = "TYPE",
        default_value = ConfigDefaults::GRID_STYLE,
        help = "Grid rendering style (cross, dots, gradient)"
    )]
    /// Grid style (cross, dots, gradient).
    pub grid_style: String,

    #[arg(
        long = "grid-color",
        value_name = "HEX",
        default_value = ConfigDefaults::GRID_COLOR,
        help = "Grid color as hex (without #)"
    )]
    /// Grid color (hex).
    pub grid_color: String,

    #[arg(
        long = "grid-opacity",
        value_name = "FLOAT",
        default_value_t = ConfigDefaults::GRID_OPACITY,
        help = concat!("Grid opacity (0.0-1.0) [default: ", stringify!(ConfigDefaults::GRID_OPACITY), "]")
    )]
    /// Grid opacity.
    pub grid_opacity: f32,

    #[arg(
        long = "grid-adaptive",
        help = "Increase grid opacity when trails are sparse"
    )]
    /// Adapt grid opacity to trail density.
    pub grid_adaptive: bool,

    // ===== Custom ASCII charset =====
    #[arg(
        long = "ascii-chars",
        value_name = "STRING",
        help = "Custom ASCII character set for rendering (e.g., \" .:-=+*#@\")"
    )]
    /// Custom ASCII character set.
    pub ascii_chars: Option<String>,

    #[arg(
        long = "ascii-contrast",
        value_name = "FLOAT",
        default_value_t = ConfigDefaults::ASCII_CONTRAST,
        help = concat!("Shape-vector ASCII contrast exponent (1.0 = none, 3.0 = strong edge enhancement) [default: ", stringify!(ConfigDefaults::ASCII_CONTRAST), "]")
    )]
    /// Contrast exponent for shape-vector ASCII rendering.
    pub ascii_contrast: f32,

    #[arg(
        long = "bg-color",
        alias = "bg",
        value_name = "HEX",
        help = "Background color as hex (e.g., '000000' or '#1a1a1a')"
    )]
    /// Background color hex code.
    pub bg_color: Option<String>,

    #[arg(long = "random", help = "Start with randomized parameters")]
    /// Start with randomized parameters.
    pub random: bool,

    #[arg(
        long = "explain",
        help = "Show detailed explanation of all simulation parameters and exit"
    )]
    /// Explain parameters and exit.
    pub explain: bool,

    #[arg(
        long = "completions",
        value_name = "SHELL",
        help = "Generate shell completions for the specified shell (bash, zsh, fish, powershell, elvish)"
    )]
    /// Generate shell completions.
    pub completions: Option<String>,
}

impl Args {
    /// Determines the operational mode based on flags.
    pub fn mode(&self) -> Mode {
        if self.screensaver {
            Mode::Screensaver
        } else if self.live {
            Mode::Live
        } else if self.print {
            Mode::Print
        } else if self.capture_frames {
            Mode::CaptureFrames
        } else if self.export_gif.is_some() {
            Mode::GifExport
        } else if self.export_webm.is_some() {
            Mode::WebmExport
        } else {
            Mode::Default
        }
    }

    /// Parses the color mode string.
    pub fn color_mode(&self) -> Result<ColorMode, String> {
        match self.colors.as_str() {
            "true" => Ok(ColorMode::TrueColor),
            "8" => Ok(ColorMode::Bits8),
            "16" => Ok(ColorMode::Bits16),
            "256" => Ok(ColorMode::Bits256),
            _ => Err(format!("Invalid color mode: {}", self.colors)),
        }
    }

    /// Parses the palette name or custom definition.
    pub fn palette(&self) -> Result<Palette, String> {
        if self.palette.starts_with('#') || self.palette.contains(',') {
            return parse_custom_palette(&self.palette);
        }
        match self.palette.as_str() {
            "organic" => Ok(Palette::Organic),
            "heat" => Ok(Palette::Heat),
            "ocean" => Ok(Palette::Ocean),
            "mono" => Ok(Palette::Mono),
            "forest" => Ok(Palette::Forest),
            "neon" => Ok(Palette::Neon),
            "warm" => Ok(Palette::Warm),
            "vibrant" => Ok(Palette::Vibrant),
            "legiblemono" => Ok(Palette::LegibleMono),
            "slime" => Ok(Palette::Slime),
            "mold" => Ok(Palette::Mold),
            "fungus" => Ok(Palette::Fungus),
            "swamp" => Ok(Palette::Swamp),
            "moss" => Ok(Palette::Moss),
            "cosmic" => Ok(Palette::Cosmic),
            "ethereal" => Ok(Palette::Ethereal),
            _ => Err(format!("Invalid palette: {}", self.palette)),
        }
    }

    /// Parses the intensity mapping configuration.
    pub fn intensity_mapping(&self) -> Result<crate::render::palette::IntensityMapping, String> {
        use crate::render::palette::{IntensityMapping, MappingFunction};

        match self.intensity_mapping.to_lowercase().as_str() {
            "linear" => Ok(IntensityMapping::linear()),
            "log" | "logarithmic" => Ok(IntensityMapping::logarithmic(self.intensity_mapping_base)),
            "exp" | "exponential" => Ok(IntensityMapping::exponential(self.intensity_mapping_base)),
            "sqrt" | "squareroot" => {
                Ok(
                    IntensityMapping::new(vec![crate::render::palette::MappingSegment {
                        start: 0.0,
                        end: 1.0,
                        function: MappingFunction::SquareRoot,
                    }])
                    .unwrap(),
                )
            }
            "square" => Ok(
                IntensityMapping::new(vec![crate::render::palette::MappingSegment {
                    start: 0.0,
                    end: 1.0,
                    function: MappingFunction::Square,
                }])
                .unwrap(),
            ),
            "power" | "gamma" => Ok(IntensityMapping::power(self.intensity_mapping_gamma)),
            "sigmoid" => Ok(
                IntensityMapping::new(vec![crate::render::palette::MappingSegment {
                    start: 0.0,
                    end: 1.0,
                    function: MappingFunction::Sigmoid { steepness: 6.0 },
                }])
                .unwrap(),
            ),
            "smoothstep" => Ok(IntensityMapping::smoothstep()),
            "quantize" => Ok(IntensityMapping::quantize(self.intensity_mapping_levels)),
            "perlin" => Ok(IntensityMapping::perlin(
                self.perlin_strength,
                5.0,
                PERLIN_SEED,
            )),
            "split" => Ok(IntensityMapping::linear_log_split(
                self.intensity_mapping_base,
            )),
            _ => Err(format!(
                "Invalid intensity mapping: {}",
                self.intensity_mapping
            )),
        }
    }

    /// Parses the logo intensity mapping. Returns `None` for "sim" (use sim's mapping).
    pub fn logo_mapping(&self) -> Result<Option<crate::render::palette::IntensityMapping>, String> {
        use crate::render::palette::{IntensityMapping, MappingFunction};

        match self.logo_mapping.to_lowercase().as_str() {
            "sim" => Ok(None),
            "linear" => Ok(Some(IntensityMapping::linear())),
            "log" | "logarithmic" => {
                Ok(Some(IntensityMapping::logarithmic(self.logo_mapping_base)))
            }
            "exp" | "exponential" => {
                Ok(Some(IntensityMapping::exponential(self.logo_mapping_base)))
            }
            "sqrt" | "squareroot" => Ok(Some(
                IntensityMapping::new(vec![crate::render::palette::MappingSegment {
                    start: 0.0,
                    end: 1.0,
                    function: MappingFunction::SquareRoot,
                }])
                .unwrap(),
            )),
            "square" => Ok(Some(
                IntensityMapping::new(vec![crate::render::palette::MappingSegment {
                    start: 0.0,
                    end: 1.0,
                    function: MappingFunction::Square,
                }])
                .unwrap(),
            )),
            "power" | "gamma" => Ok(Some(IntensityMapping::power(self.intensity_mapping_gamma))),
            "sigmoid" => Ok(Some(
                IntensityMapping::new(vec![crate::render::palette::MappingSegment {
                    start: 0.0,
                    end: 1.0,
                    function: MappingFunction::Sigmoid { steepness: 6.0 },
                }])
                .unwrap(),
            )),
            "smoothstep" => Ok(Some(IntensityMapping::smoothstep())),
            "quantize" => Ok(Some(IntensityMapping::quantize(
                self.intensity_mapping_levels,
            ))),
            "perlin" => Ok(Some(IntensityMapping::perlin(
                self.perlin_strength,
                5.0,
                PERLIN_SEED,
            ))),
            "split" => Ok(Some(IntensityMapping::linear_log_split(
                self.logo_mapping_base,
            ))),
            _ => Err(format!("Invalid logo mapping: {}", self.logo_mapping)),
        }
    }

    /// Parses the dither mode string.
    pub fn dither_mode(&self) -> Result<DitherMode, String> {
        match self.dither_mode.as_str() {
            "none" => Ok(DitherMode::None),
            "ordered" => Ok(DitherMode::Ordered {
                intensity: self
                    .dither_intensity
                    .clamp(dither_consts::MIN_INTENSITY, dither_consts::MAX_INTENSITY),
                matrix: self.parse_dither_matrix()?,
            }),
            "error-diffusion" | "error_diffusion" => {
                Ok(DitherMode::ErrorDiffusion { serpentine: true })
            }
            "hybrid" => Ok(DitherMode::Hybrid {
                edge_threshold: dither_consts::DEFAULT_HYBRID_EDGE,
                intensity: self
                    .dither_intensity
                    .clamp(dither_consts::MIN_INTENSITY, dither_consts::MAX_INTENSITY),
                matrix: self.parse_dither_matrix()?,
            }),
            _ => Err(format!("Invalid dither mode: {}", self.dither_mode)),
        }
    }

    fn parse_dither_matrix(&self) -> Result<DitherMatrix, String> {
        match self.dither_matrix.as_str() {
            "4x4" | "4" => Ok(DitherMatrix::Bayer4x4),
            "8x8" | "8" => Ok(DitherMatrix::Bayer8x8),
            _ => Err(format!("Invalid dither matrix: {}", self.dither_matrix)),
        }
    }

    /// Converts CLI arguments to simulation configuration.
    ///
    /// Uses ConfigBuilder for consistent construction and validation.
    /// Panics if validation fails - use `validate()` before calling this method.
    pub fn to_sim_config(&self) -> SimConfig {
        ConfigBuilder::from_args(self)
            .build()
            .expect("ConfigBuilder validation should have been called before to_sim_config")
    }

    /// Validates CLI arguments using ConfigBuilder.
    ///
    /// Returns an error message if any argument is invalid.
    pub fn validate_with_builder(&self) -> Result<(), String> {
        ConfigBuilder::from_args(self)
            .validate()
            .map_err(|e| e.to_string())
    }

    /// Validates arguments for correctness and safety bounds.
    ///
    /// Uses ConfigBuilder for simulation parameter validation, while keeping
    /// CLI-specific validations (resolution, trail_history, etc.) inline.
    #[allow(clippy::manual_range_contains)]
    pub fn validate(&self) -> Result<(), String> {
        // Resolution bounds - prevent excessive memory allocation
        if self.resolution.width < 10 || self.resolution.width > 2000 {
            return Err(format!(
                "resolution width must be between 10 and 2000, got {}",
                self.resolution.width
            ));
        }
        if self.resolution.height < 10 || self.resolution.height > 2000 {
            return Err(format!(
                "resolution height must be between 10 and 2000, got {}",
                self.resolution.height
            ));
        }
        // FPS bounds - prevent unreasonable values
        if self.fps < 1 || self.fps > 144 {
            return Err(format!("fps must be between 1 and 144, got {}", self.fps));
        }

        // Use ConfigBuilder for simulation parameter validation
        self.validate_with_builder()?;

        // Validate terrain type (not validated by ConfigBuilder)
        if self.terrain.parse::<TerrainType>().is_err() {
            return Err(format!(
                "Invalid terrain type: {}. Must be one of: none, smooth, turbulent, mixed",
                self.terrain
            ));
        }

        // CLI-specific validations
        if self.trail_history > 10 {
            return Err(format!(
                "trail_history must be between 0 and 10, got {}",
                self.trail_history
            ));
        }
        if self.normalize_window < 1 || self.normalize_window > 100 {
            return Err(format!(
                "normalize_window must be between 1 and 100, got {}",
                self.normalize_window
            ));
        }
        if self.dither_intensity < 0.0 || self.dither_intensity > 1.0 {
            return Err(format!(
                "dither_intensity must be between 0.0 and 1.0, got {}",
                self.dither_intensity
            ));
        }
        if self.food_scale < 0.1 || self.food_scale > 5.0 {
            return Err(format!(
                "food_scale must be between 0.1 and 5.0, got {}",
                self.food_scale
            ));
        }
        if self.mouse_attract && self.mouse_repel {
            return Err(
                "Cannot specify both --mouse-attract and --mouse-repel. Choose one mode."
                    .to_string(),
            );
        }
        if self.grid && self.grid_size == 0 {
            return Err("grid_size must be greater than 0".to_string());
        }
        if self.grid_opacity < 0.0 || self.grid_opacity > 1.0 {
            return Err(format!(
                "grid_opacity must be between 0.0 and 1.0, got {}",
                self.grid_opacity
            ));
        }
        Ok(())
    }

    /// Calculates the effective number of history frames to blend.
    pub fn effective_trail_history(&self) -> usize {
        if self.motion_blur {
            3
        } else {
            self.trail_history
        }
    }
}

impl Default for Args {
    fn default() -> Self {
        Self {
            live: false,
            screensaver: false,
            print: false,
            explore: false,
            explore_behavior: None,
            explore_iterations: 100,
            seed: None,
            population: Some(pop_consts::DEFAULT_COUNT),
            sensor_angle: Some(agent_consts::DEFAULT_SENSOR_ANGLE),
            sensor_distance: Some(agent_consts::DEFAULT_SENSOR_DISTANCE),
            rotation_angle: Some(agent_consts::DEFAULT_ROTATION_ANGLE),
            step_size: Some(agent_consts::DEFAULT_STEP_SIZE),
            decay_factor: Some(trail_consts::DEFAULT_DECAY_FACTOR),
            deposit_amount: Some(agent_consts::DEFAULT_DEPOSIT_AMOUNT),
            max_brightness: Some(trail_consts::DEFAULT_MAX_BRIGHTNESS),
            diffusion_kernel: None,
            diffusion_sigma: None,
            preset: Option::<Preset>::None,
            init: Some(InitMode::Food),
            food: food_img_consts::DEFAULT_PATH.to_string(),
            food_invert: food_img_consts::DEFAULT_INVERT,
            food_scale: ConfigDefaults::FOOD_SCALE, // Food image scale
            frame_delay: ConfigDefaults::FRAME_DELAY,
            fps: ConfigDefaults::FPS,
            time_scale: time_consts::DEFAULT_TIME_SCALE,
            resolution: Resolution {
                width: ConfigDefaults::RESOLUTION_WIDTH,
                height: ConfigDefaults::RESOLUTION_HEIGHT,
            },
            palette: ConfigDefaults::PALETTE.to_string(),
            colors: "true".to_string(),
            ascii: false,
            braille: false,
            quadrant: false,
            shade: false,
            points: false,
            half_block_dual: false,
            sculpted: false,
            plain_output: false,
            verbose: false,
            reverse_palette: false,
            invert_palette: false,
            palette_shift: 0.0,
            intensity_mapping: ConfigDefaults::INTENSITY_MAPPING.to_string(),
            intensity_mapping_base: LOG_DEFAULT,
            intensity_mapping_gamma: 2.2,
            intensity_mapping_levels: 8,
            perlin_strength: 0.2,
            logo_mapping: "log".to_string(),
            logo_mapping_base: 4.0,
            trail_history: 0,
            motion_blur: false,
            auto_normalize: false,
            normalize_window: 30,
            attract: Vec::new(),
            attractor_strength: env_consts::DEFAULT_ATTRACTOR_STRENGTH,
            capture_frames: false,
            frame_count: 50,
            frame_skip: 50,
            frame_dir: "frames".to_string(),
            dither_mode: "none".to_string(),
            dither_intensity: dither_consts::DEFAULT_INTENSITY,
            dither_matrix: "4x4".to_string(),
            dither_swap: false,
            error_diffusion_swap: false,
            species: Vec::new(),
            separate_species_trails: false,
            species_colors: false,
            simd_off: false,
            wind: None,
            terrain: "none".to_string(),
            terrain_strength: env_consts::DEFAULT_TERRAIN_STRENGTH,
            export_gif: None,
            export_webm: None,
            export_frames: 50,
            export_fps: 30,
            obstacle: Vec::new(),
            mouse_attract: false,
            mouse_repel: false,
            mouse_timeout: env_consts::DEFAULT_MOUSE_TIMEOUT,
            stats: false,
            auto_fps: false,
            warmup_frames: 60,
            warmup_brightness_multiplier: 2.5,
            warmup_decay: 0.99,
            skip_warmup: false,
            food_persist: false,
            food_persist_strength: 0.3,
            food_persist_radius: 50.0,
            food_persist_duration: 300,
            auto_reset: false,
            collapse_entropy_threshold: 0.95,
            collapse_duration_frames: 90,
            grid: false,
            grid_size: 10,
            grid_style: "cross".to_string(),
            grid_color: "ffffff".to_string(),
            grid_opacity: GRID_OPACITY_DEFAULT,
            grid_adaptive: false,
            ascii_chars: None,
            ascii_contrast: ASCII_CONTRAST_DEFAULT,
            random: false,
            explain: false,
            completions: None,
            bg_color: None,
        }
    }
}

fn parse_custom_palette(s: &str) -> Result<Palette, String> {
    let hex_colors: Vec<&str> = s.split(',').collect();
    let mut colors = Vec::new();

    for hex in hex_colors {
        let hex = hex.trim();
        if hex.is_empty() {
            continue;
        }
        let color = crate::render::palette::hex_to_rgb(hex)
            .ok_or_else(|| format!("Invalid hex color: {}", hex))?;
        colors.push(color);
    }

    if colors.len() < 2 {
        return Err(format!(
            "Custom palette requires at least 2 colors, got {}",
            colors.len()
        ));
    }
    if colors.len() > 11 {
        return Err(format!(
            "Custom palette supports maximum 11 colors, got {}",
            colors.len()
        ));
    }

    Ok(Palette::Custom(colors))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mode_default() {
        let args = Args {
            ..Default::default()
        };
        assert_eq!(args.mode(), Mode::Default);
    }

    #[test]
    fn test_mode_screensaver() {
        let args = Args {
            screensaver: true,
            live: false,
            print: false,
            ..Default::default()
        };
        assert_eq!(args.mode(), Mode::Screensaver);
    }

    #[test]
    fn test_resolution_parsing() {
        let res: Resolution = "400x400".parse().unwrap();
        assert_eq!(res.width, 400);
        assert_eq!(res.height, 400);
    }

    #[test]
    fn test_color_mode_parsing() {
        let args = Args {
            colors: "256".to_string(),
            ..Default::default()
        };
        assert_eq!(args.color_mode().unwrap(), ColorMode::Bits256);
    }

    #[test]
    fn test_palette_parsing() {
        let args = Args {
            palette: "heat".to_string(),
            ..Default::default()
        };
        assert_eq!(args.palette().unwrap(), Palette::Heat);
    }

    #[test]
    fn test_invalid_color_mode() {
        let args = Args {
            colors: "invalid".to_string(),
            ..Default::default()
        };
        assert!(args.color_mode().is_err());
    }

    #[test]
    fn test_diffusion_kernel_parsing() {
        assert_eq!(
            "mean3x3".parse::<DiffusionKernel>().unwrap(),
            DiffusionKernel::Mean3x3
        );
        assert_eq!(
            "gaussian".parse::<DiffusionKernel>().unwrap(),
            DiffusionKernel::Gaussian
        );
    }

    #[test]
    fn test_invalid_diffusion_kernel() {
        assert!("invalid".parse::<DiffusionKernel>().is_err());
    }

    #[test]
    fn test_effective_trail_history_default() {
        let args = Args {
            trail_history: 0,
            motion_blur: false,
            ..Default::default()
        };
        assert_eq!(args.effective_trail_history(), 0);
    }

    #[test]
    fn test_effective_trail_history_motion_blur() {
        let args = Args {
            trail_history: 0,
            motion_blur: true,
            ..Default::default()
        };
        assert_eq!(args.effective_trail_history(), 3);
    }

    #[test]
    fn test_effective_trail_history_explicit() {
        let args = Args {
            trail_history: 5,
            motion_blur: true,
            ..Default::default()
        };
        assert_eq!(args.effective_trail_history(), 3);
    }

    #[test]
    fn test_validate_trail_history_too_high() {
        let args = Args {
            trail_history: 15,
            ..Default::default()
        };
        assert!(args.validate().is_err());
    }

    #[test]
    fn test_validate_trail_history_valid() {
        let args = Args {
            trail_history: 5,
            ..Default::default()
        };
        assert!(args.validate().is_ok());
    }

    #[test]
    fn test_attractor_arg_parsing() {
        let arg: AttractorArg = "200,300,1.5".parse().unwrap();
        assert_eq!(arg.x, 200.0);
        assert_eq!(arg.y, 300.0);
        assert_eq!(arg.strength, 1.5);
    }

    #[test]
    fn test_attractor_arg_negative_strength() {
        let arg: AttractorArg = "100,100,-1.0".parse().unwrap();
        assert_eq!(arg.strength, -1.0);
    }

    #[test]
    fn test_attractor_arg_invalid_format() {
        assert!("200,300".parse::<AttractorArg>().is_err());
        assert!("200,300,1.0,extra".parse::<AttractorArg>().is_err());
        assert!("abc,def,ghi".parse::<AttractorArg>().is_err());
    }

    #[test]
    fn test_validate_attractor_strength_too_low() {
        let args = Args {
            attractor_strength: 0.05,
            ..Default::default()
        };
        assert!(args.validate().is_err());
    }

    #[test]
    fn test_validate_attractor_strength_valid() {
        let args = Args {
            attractor_strength: 5.0,
            ..Default::default()
        };
        assert!(args.validate().is_ok());
    }

    #[test]
    fn test_species_arg_basic() {
        let species: SpeciesArg = "red:20k@22.5,45,1.0,5.0:ff0000".parse().unwrap();
        assert_eq!(species.name, "red");
        assert_eq!(species.count, 20000);
        assert!((species.sensor_angle - 22.5).abs() < 0.01);
        assert!((species.rotation_angle - 45.0).abs() < 0.01);
        assert!((species.step_size - 1.0).abs() < 0.01);
        assert!((species.deposit_amount - 5.0).abs() < 0.01);
        assert_eq!(species.color, "ff0000");
    }

    #[test]
    fn test_species_arg_count_formats() {
        let s1: SpeciesArg = "red:1000".parse().unwrap();
        assert_eq!(s1.count, 1000);

        let s2: SpeciesArg = "red:10k".parse().unwrap();
        assert_eq!(s2.count, 10000);

        let s3: SpeciesArg = "red:1m".parse().unwrap();
        assert_eq!(s3.count, 1000000);
    }

    #[test]
    fn test_species_arg_color_formats() {
        let s1: SpeciesArg = "red:1000:ff0000".parse().unwrap();
        assert_eq!(s1.color, "ff0000");

        let s2: SpeciesArg = "red:1000:#00ff00".parse().unwrap();
        assert_eq!(s2.color, "00ff00");
    }

    #[test]
    fn test_species_arg_defaults() {
        let species: SpeciesArg = "red:1000".parse().unwrap();
        assert_eq!(species.sensor_angle, 22.5);
        assert_eq!(species.rotation_angle, 45.0);
        assert_eq!(species.step_size, 1.0);
        assert_eq!(species.deposit_amount, 5.0);
        assert_eq!(species.color, "228b22");
    }

    #[test]
    fn test_species_arg_invalid_format() {
        assert!("red".parse::<SpeciesArg>().is_err());
        assert!("red:invalid".parse::<SpeciesArg>().is_err());
    }

    #[test]
    fn test_obstacle_arg_circle_parsing() {
        let arg: ObstacleArg = "circle:200,300,50".parse().unwrap();
        match arg.obstacle {
            Obstacle::Circle { x, y, radius } => {
                assert_eq!(x, 200.0);
                assert_eq!(y, 300.0);
                assert_eq!(radius, 50.0);
            }
            _ => panic!("Expected Circle obstacle"),
        }
    }

    #[test]
    fn test_obstacle_arg_rect_parsing() {
        let arg: ObstacleArg = "rect:100,150,80,60".parse().unwrap();
        match arg.obstacle {
            Obstacle::Rect {
                x,
                y,
                width,
                height,
            } => {
                assert_eq!(x, 100.0);
                assert_eq!(y, 150.0);
                assert_eq!(width, 80.0);
                assert_eq!(height, 60.0);
            }
            _ => panic!("Expected Rect obstacle"),
        }
    }

    #[test]
    fn test_obstacle_arg_invalid_format() {
        assert!("circle:100,200".parse::<ObstacleArg>().is_err());
        assert!("circle:100,200,abc".parse::<ObstacleArg>().is_err());
        assert!("rect:100,200,50".parse::<ObstacleArg>().is_err());
        assert!("invalid:100,200,50".parse::<ObstacleArg>().is_err());
    }

    #[test]
    fn test_obstacle_arg_negative_radius() {
        assert!("circle:100,200,-50".parse::<ObstacleArg>().is_err());
    }

    #[test]
    fn test_obstacle_arg_negative_dimensions() {
        assert!("rect:100,200,-50,30".parse::<ObstacleArg>().is_err());
        assert!("rect:100,200,50,-30".parse::<ObstacleArg>().is_err());
    }

    #[test]
    fn test_obstacle_image_parsing() {
        let arg: ObstacleArg = "image:test.png,100,200,50,50,false,0.5".parse().unwrap();
        match &arg.obstacle {
            Obstacle::Image {
                path,
                x,
                y,
                width,
                height,
                invert,
                threshold,
            } => {
                assert_eq!(path, "test.png");
                assert_eq!(*x, 100.0);
                assert_eq!(*y, 200.0);
                assert_eq!(*width, 50);
                assert_eq!(*height, 50);
                assert!(!*invert);
                assert!((*threshold - 0.5).abs() < 0.001);
            }
            _ => panic!("Expected Image obstacle"),
        }

        let arg: ObstacleArg = "image:logo.png,0,0,100,100,true,0.8".parse().unwrap();
        match &arg.obstacle {
            Obstacle::Image {
                path,
                x,
                y,
                width,
                height,
                invert,
                threshold,
            } => {
                assert_eq!(path, "logo.png");
                assert_eq!(*x, 0.0);
                assert_eq!(*y, 0.0);
                assert_eq!(*width, 100);
                assert_eq!(*height, 100);
                assert!(*invert);
                assert!((*threshold - 0.8).abs() < 0.001);
            }
            _ => panic!("Expected Image obstacle"),
        }
    }

    #[test]
    fn test_obstacle_image_invalid() {
        assert!("image:test.png".parse::<ObstacleArg>().is_err());
        assert!("image:test.png,100".parse::<ObstacleArg>().is_err());
        assert!("image:test.png,100,200,50".parse::<ObstacleArg>().is_err());
        assert!("image:test.png,100,200,50,abc,0.5"
            .parse::<ObstacleArg>()
            .is_err());
        assert!("image:test.png,100,200,0,50,false,0.5"
            .parse::<ObstacleArg>()
            .is_err());
        assert!("image:test.png,100,200,50,0,false,0.5"
            .parse::<ObstacleArg>()
            .is_err());
        assert!("image:test.png,100,200,50,50,false,1.5"
            .parse::<ObstacleArg>()
            .is_err());
    }

    #[test]
    fn test_wind_arg_parsing() {
        let arg: WindArg = "0.5,0.5".parse().unwrap();
        assert_eq!(arg.dx, 0.5);
        assert_eq!(arg.dy, 0.5);

        let arg: WindArg = "-0.3,0.7".parse().unwrap();
        assert_eq!(arg.dx, -0.3);
        assert_eq!(arg.dy, 0.7);
    }

    #[test]
    fn test_wind_arg_invalid() {
        assert!("0.5".parse::<WindArg>().is_err());
        assert!("0.5,0.5,extra".parse::<WindArg>().is_err());
        assert!("abc,def".parse::<WindArg>().is_err());
    }

    #[test]
    fn test_terrain_type_in_args() {
        let args = Args {
            terrain: "smooth".to_string(),
            ..Default::default()
        };
        let config = args.to_sim_config();
        assert_eq!(config.terrain, TerrainType::Smooth);

        let args = Args {
            terrain: "turbulent".to_string(),
            ..Default::default()
        };
        let config = args.to_sim_config();
        assert_eq!(config.terrain, TerrainType::Turbulent);

        let args = Args {
            terrain: "mixed".to_string(),
            ..Default::default()
        };
        let config = args.to_sim_config();
        assert_eq!(config.terrain, TerrainType::Mixed);

        let args = Args {
            terrain: "none".to_string(),
            ..Default::default()
        };
        let config = args.to_sim_config();
        assert_eq!(config.terrain, TerrainType::None);
    }

    #[test]
    fn test_terrain_strength_in_args() {
        let args = Args {
            terrain_strength: 2.0,
            ..Default::default()
        };
        let config = args.to_sim_config();
        assert_eq!(config.terrain_strength, 2.0);
    }

    #[test]
    fn test_wind_in_args() {
        let args = Args {
            wind: Some(WindArg { dx: 0.5, dy: 0.0 }),
            ..Default::default()
        };
        let config = args.to_sim_config();
        assert!(config.wind.is_some());
        assert_eq!(config.wind.unwrap().dx, 0.5);
        assert_eq!(config.wind.unwrap().dy, 0.0);
    }

    #[test]
    fn test_validate_terrain_strength() {
        let args = Args {
            terrain_strength: 0.05,
            ..Default::default()
        };
        assert!(args.validate().is_err());

        let args = Args {
            terrain_strength: 10.0,
            ..Default::default()
        };
        assert!(args.validate().is_err());
    }

    #[test]
    fn test_validate_terrain_type() {
        let args = Args {
            terrain: "invalid".to_string(),
            ..Default::default()
        };
        assert!(args.validate().is_err());
    }

    #[test]
    fn test_custom_palette_parsing() {
        let args = Args {
            palette: "#ff0000,#00ff00,#0000ff".to_string(),
            ..Default::default()
        };
        let palette = args.palette().unwrap();
        match palette {
            Palette::Custom(colors) => {
                assert_eq!(colors.len(), 3);
                assert_eq!(colors[0].r, 255);
                assert_eq!(colors[0].g, 0);
                assert_eq!(colors[0].b, 0);
                assert_eq!(colors[1].r, 0);
                assert_eq!(colors[1].g, 255);
                assert_eq!(colors[1].b, 0);
                assert_eq!(colors[2].r, 0);
                assert_eq!(colors[2].g, 0);
                assert_eq!(colors[2].b, 255);
            }
            _ => panic!("Expected Custom palette"),
        }
    }

    #[test]
    fn test_custom_palette_with_hash() {
        let args = Args {
            palette: "#ff0000,#00ff00".to_string(),
            ..Default::default()
        };
        let palette = args.palette().unwrap();
        match palette {
            Palette::Custom(colors) => {
                assert_eq!(colors.len(), 2);
                assert_eq!(colors[0].r, 255);
                assert_eq!(colors[0].g, 0);
                assert_eq!(colors[0].b, 0);
            }
            _ => panic!("Expected Custom palette"),
        }
    }

    #[test]
    fn test_custom_palette_too_few_colors() {
        let args = Args {
            palette: "#ff0000".to_string(),
            ..Default::default()
        };
        assert!(args.palette().is_err());
    }

    #[test]
    fn test_custom_palette_too_many_colors() {
        let args = Args {
            palette: "#ff0000,#00ff00,#0000ff,#ffff00,#00ffff,#ff00ff,#ffffff,#000000,#880000,#008800,#004400,#002200"
                .to_string(),
            ..Default::default()
        };
        assert!(args.palette().is_err());
    }

    #[test]
    fn test_custom_palette_invalid_hex() {
        let args = Args {
            palette: "#gg0000,#00ff00".to_string(),
            ..Default::default()
        };
        assert!(args.palette().is_err());
    }
}
