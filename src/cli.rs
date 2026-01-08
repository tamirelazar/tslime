use clap::Parser;
use std::num::ParseIntError;
use std::str::FromStr;

use crate::render::dither::{DitherMatrix, DitherMode};
use crate::render::palette::RgbColor;
use crate::simulation::config::{
    Attractor, DiffusionKernel, InitMode, Obstacle, Preset, SimConfig, SpeciesConfig, TerrainType,
    Wind,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Mode {
    Default,
    Live,
    Screensaver,
    Print,
    CaptureFrames,
    GifExport,
    WebmExport,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorMode {
    TrueColor,
    Bits8,
    Bits16,
    Bits256,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Palette {
    Organic,
    Heat,
    Ocean,
    Mono,
    Forest,
    Neon,
    Warm,
    Vibrant,
    LegibleMono,
    Slime,
    Mold,
    Fungus,
    Swamp,
    Moss,
    Cosmic,
    Ethereal,
    Custom(Vec<RgbColor>),
}

impl Palette {
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

#[derive(Debug, Clone)]
pub struct Resolution {
    pub width: usize,
    pub height: usize,
}

#[derive(Debug, Clone)]
pub struct SpeciesArg {
    pub name: String,
    pub count: usize,
    pub sensor_angle: f32,
    pub rotation_angle: f32,
    pub step_size: f32,
    pub deposit_amount: f32,
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

        let mut sensor_angle = 22.5;
        let mut rotation_angle = 45.0;
        let mut step_size = 1.0;
        let mut deposit_amount = 5.0;
        let mut color = "228b22".to_string();

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
            _ => Err(format!(
                "Invalid preset: {}. Must be one of: network, exploratory, tendrils, organic, minimal, moss, cosmic, fire, zen, storm, river, ethereal",
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
            "food" => Ok(InitMode::Food),
            _ => Err(format!(
                "Invalid init mode: {}. Must be one of: random, central, circle, gradient, wave, spiral, clusters, food",
                s
            )),
        }
    }
}

#[derive(Debug, Clone)]
pub struct AttractorArg {
    pub x: f32,
    pub y: f32,
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
pub struct WindArg {
    pub dx: f32,
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
pub struct ObstacleArg {
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
pub struct Args {
    #[arg(short = 'l', long = "live", help = "Continuous animation mode")]
    pub live: bool,

    #[arg(
        short = 'S',
        long = "screensaver",
        help = "Screensaver mode (exit on keypress)"
    )]
    pub screensaver: bool,

    #[arg(short = 'p', long = "print", help = "Print single frame and exit")]
    pub print: bool,

    #[arg(
        long = "capture-frames",
        help = "Capture simulation frames for GIF generation"
    )]
    pub capture_frames: bool,

    #[arg(
        long = "frame-count",
        value_name = "INT",
        default_value = "50",
        help = "Number of frames to capture"
    )]
    pub frame_count: usize,

    #[arg(
        long = "frame-skip",
        value_name = "INT",
        default_value = "50",
        help = "Simulation steps between captured frames"
    )]
    pub frame_skip: usize,

    #[arg(
        long = "frame-dir",
        value_name = "PATH",
        default_value = "frames",
        help = "Directory to save captured frames"
    )]
    pub frame_dir: String,

    #[arg(
        short = 's',
        long = "seed",
        value_name = "INT",
        help = "Random seed for reproducibility"
    )]
    pub seed: Option<u64>,

    #[arg(
        short = 'n',
        long = "population",
        value_name = "INT",
        default_value = "50000",
        help = "Number of agents"
    )]
    pub population: usize,

    #[arg(
        long = "sensor-angle",
        value_name = "DEG",
        default_value = "22.5",
        help = "Sensor spread angle"
    )]
    pub sensor_angle: f32,

    #[arg(
        long = "sensor-distance",
        value_name = "FLOAT",
        default_value = "9.0",
        help = "Sensor range"
    )]
    pub sensor_distance: f32,

    #[arg(
        long = "rotation-angle",
        value_name = "DEG",
        default_value = "45.0",
        help = "Turn amount per step"
    )]
    pub rotation_angle: f32,

    #[arg(
        long = "step-size",
        value_name = "FLOAT",
        default_value = "1.0",
        help = "Movement speed"
    )]
    pub step_size: f32,

    #[arg(
        long = "decay",
        value_name = "FLOAT",
        default_value = "0.5",
        help = "Trail decay factor"
    )]
    pub decay_factor: f32,

    #[arg(
        long = "deposit",
        value_name = "FLOAT",
        default_value = "5.0",
        help = "Amount of pheromone deposited by agents per step"
    )]
    pub deposit_amount: f32,

    #[arg(
        long = "max-brightness",
        value_name = "FLOAT",
        default_value = "100.0",
        help = "Fixed maximum brightness for normalization (prevents flickering)"
    )]
    pub max_brightness: f32,

    #[arg(
        long = "diffusion-kernel",
        value_name = "TYPE",
        help = "Diffusion kernel type (mean3x3, gaussian)"
    )]
    pub diffusion_kernel: Option<DiffusionKernel>,

    #[arg(
        long = "diffusion-sigma",
        value_name = "FLOAT",
        help = "Gaussian kernel sigma value (0.5-2.0, only used with gaussian kernel)"
    )]
    pub diffusion_sigma: Option<f32>,

    #[arg(
        long = "preset",
        value_name = "NAME",
        help = "Use named preset (network, exploratory, tendrils, organic, minimal, moss, cosmic, fire, zen, storm, river)"
    )]
    pub preset: Option<Preset>,

    #[arg(
        long = "init",
        value_name = "MODE",
        default_value = "food",
        help = "Initialization mode (random, central, circle, gradient, wave, spiral, clusters, food)"
    )]
    pub init: InitMode,

    #[arg(
        long = "food",
        value_name = "PATH",
        default_value = "assets/tslime_logo.png",
        help = "Load agents from PNG image. High-brightness areas spawn more agents. Use with --init food"
    )]
    pub food: String,

    #[arg(
        long = "food-invert",
        value_name = "BOOL",
        num_args = 1,
        default_value_t = true,
        help = "Invert the food image values (dark areas spawn more agents instead of bright areas)"
    )]
    pub food_invert: bool,

    #[arg(
        long = "food-scale",
        value_name = "FLOAT",
        default_value = "1.5",
        help = "Scale factor for food image relative to canvas (0.1-5.0, e.g., 0.5 = half size, 2.0 = double size)"
    )]
    pub food_scale: f32,

    #[arg(
        short = 't',
        long = "time",
        value_name = "FLOAT",
        default_value = "0.033",
        help = "Frame delay in seconds"
    )]
    pub frame_delay: f32,

    #[arg(
        long = "fps",
        value_name = "INT",
        default_value = "30",
        help = "Target frames per second"
    )]
    pub fps: usize,

    #[arg(
        long = "time-scale",
        value_name = "FLOAT",
        default_value = "1.0",
        help = "Time scaling factor (1.0 = normal, 0.5 = half speed, 2.0 = double speed)"
    )]
    pub time_scale: f32,

    #[arg(
        long = "resolution",
        value_name = "WxH",
        default_value = "400x200",
        help = "Simulation resolution"
    )]
    pub resolution: Resolution,

    #[arg(
        long = "palette",
        value_name = "NAME",
        default_value = "forest",
        help = "Color palette (organic, heat, ocean, mono, forest, neon, warm, vibrant, legiblemono, slime, mold, fungus, swamp, moss, cosmic, ethereal) or custom: \"#rrggbb,#rrggbb,...\" (2-11 colors)"
    )]
    pub palette: String,

    #[arg(
        long = "colors",
        value_name = "MODE",
        default_value = "true",
        help = "Color mode (8, 16, 256, true)"
    )]
    pub colors: String,

    #[arg(long = "ascii", help = "Use ASCII characters only")]
    pub ascii: bool,

    #[arg(long = "braille", help = "Use braille characters")]
    pub braille: bool,

    #[arg(
        long = "quadrant",
        help = "Use Unicode quadrant characters for 4× vertical resolution"
    )]
    pub quadrant: bool,

    #[arg(
        long = "plain-output",
        help = "Output plain text without ANSI color codes (for frame capture)"
    )]
    pub plain_output: bool,

    #[arg(
        short = 'v',
        long = "verbose",
        help = "Print performance stats to stderr"
    )]
    pub verbose: bool,

    #[arg(
        long = "reverse-palette",
        help = "Reverse palette order (dark becomes light, light becomes dark)"
    )]
    pub reverse_palette: bool,

    #[arg(
        long = "invert-palette",
        help = "Invert palette colors (complementary colors)"
    )]
    pub invert_palette: bool,

    #[arg(
        long = "palette-shift",
        value_name = "DEG",
        default_value = "0",
        help = "Rotate palette hue over time (degrees per second, negative for reverse rotation)"
    )]
    pub palette_shift: f32,

    #[arg(
        long = "trail-history",
        value_name = "INT",
        default_value = "0",
        help = "Number of historical frames to blend for motion blur (0=disabled, max 10)"
    )]
    pub trail_history: usize,

    #[arg(
        long = "motion-blur",
        help = "Enable motion blur effect (equivalent to --trail-history 3)"
    )]
    pub motion_blur: bool,

    #[arg(
        long = "auto-normalize",
        help = "Enable adaptive brightness normalization to prevent flickering"
    )]
    pub auto_normalize: bool,

    #[arg(
        long = "normalize-window",
        value_name = "INT",
        default_value = "30",
        help = "Number of frames for adaptive brightness normalization window (1-100)"
    )]
    pub normalize_window: usize,

    #[arg(
        long = "attract",
        value_name = "X,Y,STRENGTH",
        help = "Add point attractor (positive=attract, negative=repel). Can be specified multiple times. Example: --attract 200,200,1.0"
    )]
    pub attract: Vec<AttractorArg>,

    #[arg(
        long = "obstacle",
        value_name = "TYPE:X,Y,PARAMS",
        help = "Add obstacle (circle:x,y,r or rect:x,y,w,h or image:path,x,y,w,h,invert,threshold). Can be specified multiple times. Example: --obstacle circle:200,200,50"
    )]
    pub obstacle: Vec<ObstacleArg>,

    #[arg(
        long = "attractor-strength",
        value_name = "FLOAT",
        default_value = "1.0",
        help = "Global multiplier for attractor/repeller strength (0.1-10.0)"
    )]
    pub attractor_strength: f32,

    #[arg(
        long = "dither-mode",
        value_name = "MODE",
        default_value = "none",
        help = "Dithering mode: none, ordered, error-diffusion, hybrid"
    )]
    pub dither_mode: String,

    #[arg(
        long = "dither-intensity",
        value_name = "FLOAT",
        default_value = "0.5",
        help = "Dithering intensity for ordered/hybrid modes (0.0-1.0, higher = more dithering effect)"
    )]
    pub dither_intensity: f32,

    #[arg(
        long = "dither-matrix",
        value_name = "MATRIX",
        default_value = "4x4",
        help = "Dither matrix for ordered mode: 4x4, 8x8"
    )]
    pub dither_matrix: String,

    #[arg(
        long = "dither-swap",
        help = "Swap to next dither mode (cycle through none -> ordered -> error-diffusion -> hybrid)"
    )]
    pub dither_swap: bool,

    #[arg(long = "error-diffusion-swap", help = "Toggle error diffusion mode")]
    pub error_diffusion_swap: bool,

    #[arg(
        long = "species",
        value_name = "SPEC",
        help = "Define agent species with format 'name:count@sensor_angle,rotation_angle,step_size,deposit:color'. Can be specified multiple times or comma-separated. Example: --species 'red:20k@22.5,45,1.0,5.0:ff0000,blue:30k@30,60,1.5,3.0:0000ff'"
    )]
    pub species: Vec<SpeciesArg>,

    #[arg(
        long = "separate-species-trails",
        help = "Each species maintains its own separate trail map (higher memory, allows species-specific patterns)"
    )]
    pub separate_species_trails: bool,

    #[arg(
        long = "species-colors",
        help = "Enable species-specific rendering using each species' configured color. Automatically enables --separate-species-trails."
    )]
    pub species_colors: bool,

    #[arg(
        long = "simd-off",
        help = "Disable SIMD acceleration for diffusion (use scalar fallback)"
    )]
    pub simd_off: bool,

    #[arg(
        long = "wind",
        value_name = "DX,DY",
        help = "Apply constant wind force (dx,dy from -1.0 to 1.0). Example: --wind 0.5,0.0 for rightward wind"
    )]
    pub wind: Option<WindArg>,

    #[arg(
        long = "terrain",
        value_name = "TYPE",
        default_value = "none",
        help = "Terrain type for organic movement patterns: none, smooth, turbulent, mixed"
    )]
    pub terrain: String,

    #[arg(
        long = "terrain-strength",
        value_name = "FLOAT",
        default_value = "1.0",
        help = "Strength of terrain influence (0.1-5.0)"
    )]
    pub terrain_strength: f32,

    #[arg(
        long = "export-gif",
        value_name = "PATH",
        help = "Export simulation to GIF file"
    )]
    pub export_gif: Option<String>,

    #[arg(
        long = "export-webm",
        value_name = "PATH",
        help = "Export simulation to WebM video file (requires FFmpeg)"
    )]
    pub export_webm: Option<String>,

    #[arg(
        long = "export-frames",
        value_name = "INT",
        default_value = "50",
        help = "Number of frames to capture for GIF export"
    )]
    pub export_frames: usize,

    #[arg(
        long = "export-fps",
        value_name = "INT",
        default_value = "30",
        help = "GIF playback speed (frames per second)"
    )]
    pub export_fps: usize,

    #[arg(
        long = "mouse-attract",
        help = "Enable mouse clicks to create temporary attractors at cursor position"
    )]
    pub mouse_attract: bool,

    #[arg(
        long = "mouse-repel",
        help = "Enable mouse clicks to create temporary repellers at cursor position"
    )]
    pub mouse_repel: bool,

    #[arg(
        long = "mouse-timeout",
        value_name = "FLOAT",
        default_value = "3.0",
        help = "Time in seconds before mouse-created attractors/repellers expire (0.1-30.0)"
    )]
    pub mouse_timeout: f32,

    #[arg(long = "stats", help = "Display real-time statistics overlay")]
    pub stats: bool,

    #[arg(
        long = "no-auto-fps",
        help = "Disable automatic FPS adjustment when performance drops"
    )]
    pub no_auto_fps: bool,

    // ===== Warmup frames =====
    #[arg(
        long = "warmup-frames",
        value_name = "INT",
        default_value = "60",
        help = "Number of frames to display logo before simulation (0 to disable)"
    )]
    pub warmup_frames: usize,

    #[arg(
        long = "warmup-brightness",
        value_name = "FLOAT",
        default_value = "2.5",
        help = "Brightness multiplier during warmup phase"
    )]
    pub warmup_brightness_multiplier: f32,

    #[arg(
        long = "warmup-decay",
        value_name = "FLOAT",
        default_value = "0.99",
        help = "Decay factor during warmup (higher = logo persists longer)"
    )]
    pub warmup_decay: f32,

    #[arg(long = "skip-warmup", help = "Skip warmup phase (useful for exports)")]
    pub skip_warmup: bool,

    // ===== Food persistence =====
    #[arg(
        long = "food-persist",
        help = "Keep agents near original food/logo location using attractors"
    )]
    pub food_persist: bool,

    #[arg(
        long = "food-persist-strength",
        value_name = "FLOAT",
        default_value = "0.3",
        help = "Strength of food persistence attractors (0.0-5.0)"
    )]
    pub food_persist_strength: f32,

    #[arg(
        long = "food-persist-radius",
        value_name = "FLOAT",
        default_value = "50.0",
        help = "Radius of influence for food persistence attractors"
    )]
    pub food_persist_radius: f32,

    #[arg(
        long = "food-persist-duration",
        value_name = "INT",
        default_value = "300",
        help = "Number of frames before food attractors fade out (0 = permanent)"
    )]
    pub food_persist_duration: usize,

    // ===== Entropy-based auto-reset =====
    #[arg(
        long = "auto-reset",
        help = "Automatically restart simulation when it collapses",
        default_value = "false"
    )]
    pub auto_reset: bool,

    #[arg(
        long = "collapse-threshold",
        value_name = "FLOAT",
        default_value = "0.95",
        help = "Entropy threshold to detect collapse (0.0-1.0, higher = more sensitive)"
    )]
    pub collapse_entropy_threshold: f32,

    #[arg(
        long = "collapse-duration",
        value_name = "INT",
        default_value = "90",
        help = "Number of frames simulation must stay collapsed before auto-reset"
    )]
    pub collapse_duration_frames: usize,

    // ===== Background grid =====
    #[arg(long = "grid", help = "Enable background grid rendering")]
    pub grid: bool,

    #[arg(
        long = "grid-size",
        value_name = "INT",
        default_value = "10",
        help = "Grid cell size (number of cells per dimension)"
    )]
    pub grid_size: usize,

    #[arg(
        long = "grid-style",
        value_name = "TYPE",
        default_value = "cross",
        help = "Grid rendering style (cross, dots, gradient)"
    )]
    pub grid_style: String,

    #[arg(
        long = "grid-color",
        value_name = "HEX",
        default_value = "ffffff",
        help = "Grid color as hex (without #)"
    )]
    pub grid_color: String,

    #[arg(
        long = "grid-opacity",
        value_name = "FLOAT",
        default_value = "0.15",
        help = "Grid opacity (0.0-1.0)"
    )]
    pub grid_opacity: f32,

    #[arg(
        long = "grid-adaptive",
        help = "Increase grid opacity when trails are sparse"
    )]
    pub grid_adaptive: bool,

    // ===== Custom ASCII charset =====
    #[arg(
        long = "ascii-chars",
        value_name = "STRING",
        help = "Custom ASCII character set for rendering (e.g., \" .:-=+*#@\")"
    )]
    pub ascii_chars: Option<String>,

    #[arg(long = "random", help = "Start with randomized parameters")]
    pub random: bool,

    #[arg(
        long = "explain",
        help = "Show detailed explanation of all simulation parameters and exit"
    )]
    pub explain: bool,
}

impl Args {
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

    pub fn color_mode(&self) -> Result<ColorMode, String> {
        match self.colors.as_str() {
            "true" => Ok(ColorMode::TrueColor),
            "8" => Ok(ColorMode::Bits8),
            "16" => Ok(ColorMode::Bits16),
            "256" => Ok(ColorMode::Bits256),
            _ => Err(format!("Invalid color mode: {}", self.colors)),
        }
    }

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

    pub fn dither_mode(&self) -> Result<DitherMode, String> {
        match self.dither_mode.as_str() {
            "none" => Ok(DitherMode::None),
            "ordered" => Ok(DitherMode::Ordered {
                intensity: self.dither_intensity.clamp(0.0, 1.0),
                matrix: self.parse_dither_matrix()?,
            }),
            "error-diffusion" | "error_diffusion" => {
                Ok(DitherMode::ErrorDiffusion { serpentine: true })
            }
            "hybrid" => Ok(DitherMode::Hybrid {
                edge_threshold: 0.15,
                intensity: self.dither_intensity.clamp(0.0, 1.0),
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

    pub fn to_sim_config(&self) -> SimConfig {
        let mut config = if let Some(preset) = self.preset {
            SimConfig::from(preset)
        } else {
            SimConfig::default()
        };

        config.sensor_angle = self.sensor_angle;
        config.sensor_distance = self.sensor_distance;
        config.rotation_angle = self.rotation_angle;
        config.step_size = self.step_size;
        config.decay_factor = self.decay_factor;
        config.max_brightness = self.max_brightness;
        config.food_image_path = Some(self.food.clone());
        config.food_image_invert = self.food_invert;
        config.food_image_scale = self.food_scale;

        if let Some(kernel) = self.diffusion_kernel {
            config.diffusion_kernel = kernel;
        }

        if let Some(sigma) = self.diffusion_sigma {
            config.diffusion_sigma = sigma;
        }

        if self.fps >= 60 && self.diffusion_kernel.is_none() && self.diffusion_sigma.is_none() {
            config.diffusion_kernel = crate::simulation::config::DiffusionKernel::Gaussian;
            config.diffusion_sigma = 0.5;
        }

        config.attractors = self
            .attract
            .iter()
            .map(|a| Attractor::new(a.x, a.y, a.strength))
            .collect();
        config.attractor_strength = self.attractor_strength;

        config.obstacles = self.obstacle.iter().map(|o| o.obstacle.clone()).collect();
        let _ = config.load_obstacle_masks();

        config.separate_species_trails = self.separate_species_trails || self.species_colors;

        config.use_simd = !self.simd_off;

        if !self.species.is_empty() {
            config.species_configs = self
                .species
                .iter()
                .map(|s| SpeciesConfig {
                    name: s.name.clone(),
                    count: s.count,
                    sensor_angle: s.sensor_angle,
                    rotation_angle: s.rotation_angle,
                    step_size: s.step_size,
                    deposit_amount: s.deposit_amount,
                    color: s.color.clone(),
                })
                .collect();
        } else {
            config.species_configs = vec![SpeciesConfig {
                name: "default".to_string(),
                count: self.population,
                sensor_angle: self.sensor_angle,
                rotation_angle: self.rotation_angle,
                step_size: self.step_size,
                deposit_amount: self.deposit_amount,
                color: "228b22".to_string(),
            }];
        }

        config.wind = self.wind.as_ref().map(|w| Wind::new(w.dx, w.dy));
        config.terrain = self
            .terrain
            .parse::<TerrainType>()
            .unwrap_or(TerrainType::None);
        config.terrain_strength = self.terrain_strength;

        config
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.time_scale < 0.1 || self.time_scale > 10.0 {
            return Err(format!(
                "time_scale must be between 0.1 and 10.0, got {}",
                self.time_scale
            ));
        }
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
        if self.attractor_strength < 0.1 || self.attractor_strength > 10.0 {
            return Err(format!(
                "attractor_strength must be between 0.1 and 10.0, got {}",
                self.attractor_strength
            ));
        }
        if self.dither_intensity < 0.0 || self.dither_intensity > 1.0 {
            return Err(format!(
                "dither_intensity must be between 0.0 and 1.0, got {}",
                self.dither_intensity
            ));
        }
        if self.terrain_strength < 0.1 || self.terrain_strength > 5.0 {
            return Err(format!(
                "terrain_strength must be between 0.1 and 5.0, got {}",
                self.terrain_strength
            ));
        }
        if self.terrain.parse::<TerrainType>().is_err() {
            return Err(format!(
                "Invalid terrain type: {}. Must be one of: none, smooth, turbulent, mixed",
                self.terrain
            ));
        }
        if let Some(ref wind) = self.wind {
            let w = Wind::new(wind.dx, wind.dy);
            if let Err(e) = w.validate() {
                return Err(format!("Invalid wind: {}", e));
            }
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
        if self.mouse_timeout < 0.1 || self.mouse_timeout > 30.0 {
            return Err(format!(
                "mouse_timeout must be between 0.1 and 30.0, got {}",
                self.mouse_timeout
            ));
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
            seed: None,
            population: 50000,
            sensor_angle: 22.5,
            sensor_distance: 9.0,
            rotation_angle: 45.0,
            step_size: 1.0,
            decay_factor: 0.5,
            deposit_amount: 5.0,
            max_brightness: 100.0,
            diffusion_kernel: None,
            diffusion_sigma: None,
            preset: Option::<Preset>::None,
            init: InitMode::Food,
            food: "assets/tslime_logo.png".to_string(),
            food_invert: true,
            food_scale: 1.5,
            frame_delay: 0.033,
            fps: 30,
            time_scale: 1.0,
            resolution: Resolution {
                width: 400,
                height: 200,
            },
            palette: "forest".to_string(),
            colors: "true".to_string(),
            ascii: false,
            braille: false,
            quadrant: false,
            plain_output: false,
            verbose: false,
            reverse_palette: false,
            invert_palette: false,
            palette_shift: 0.0,
            trail_history: 0,
            motion_blur: false,
            auto_normalize: false,
            normalize_window: 30,
            attract: Vec::new(),
            attractor_strength: 1.0,
            capture_frames: false,
            frame_count: 50,
            frame_skip: 50,
            frame_dir: "frames".to_string(),
            dither_mode: "none".to_string(),
            dither_intensity: 0.5,
            dither_matrix: "4x4".to_string(),
            dither_swap: false,
            error_diffusion_swap: false,
            species: Vec::new(),
            separate_species_trails: false,
            species_colors: false,
            simd_off: false,
            wind: None,
            terrain: "none".to_string(),
            terrain_strength: 1.0,
            export_gif: None,
            export_webm: None,
            export_frames: 50,
            export_fps: 30,
            obstacle: Vec::new(),
            mouse_attract: false,
            mouse_repel: false,
            mouse_timeout: 3.0,
            stats: false,
            no_auto_fps: false,
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
            grid_opacity: 0.15,
            grid_adaptive: false,
            ascii_chars: None,
            random: false,
            explain: false,
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
