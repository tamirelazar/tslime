use clap::Parser;
use std::num::ParseIntError;
use std::str::FromStr;

use crate::simulation::config::{DiffusionKernel, InitMode, Preset, SimConfig};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Default,
    Live,
    Screensaver,
    Print,
    CaptureFrames,
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
}

#[derive(Debug, Clone)]
pub struct Resolution {
    pub width: usize,
    pub height: usize,
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
            _ => Err(format!(
                "Invalid preset: {}. Must be one of: network, exploratory, tendrils, organic",
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
            _ => Err(format!(
                "Invalid init mode: {}. Must be one of: random, central, circle, gradient, wave, spiral, clusters",
                s
            )),
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
        default_value = "0.9",
        help = "Trail decay factor"
    )]
    pub decay_factor: f32,

    #[arg(
        long = "max-brightness",
        value_name = "FLOAT",
        default_value = "20.0",
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
        help = "Use named preset (network, exploratory, tendrils, organic)"
    )]
    pub preset: Option<Preset>,

    #[arg(
        long = "init",
        value_name = "MODE",
        default_value = "random",
        help = "Initialization mode (random, central, circle, gradient, wave, spiral, clusters)"
    )]
    pub init: InitMode,

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
        default_value = "400x400",
        help = "Simulation resolution"
    )]
    pub resolution: Resolution,

    #[arg(
        long = "palette",
        value_name = "NAME",
        default_value = "forest",
        help = "Color palette (organic, heat, ocean, mono, forest, neon, warm, vibrant, legiblemono, slime, mold, fungus, swamp)"
    )]
    pub palette: String,

    #[arg(
        long = "colors",
        value_name = "MODE",
        default_value = "256",
        help = "Color mode (8, 16, 256, true)"
    )]
    pub colors: String,

    #[arg(long = "ascii", help = "Use ASCII characters only")]
    pub ascii: bool,

    #[arg(long = "braille", help = "Use braille characters")]
    pub braille: bool,

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
            _ => Err(format!("Invalid palette: {}", self.palette)),
        }
    }

    pub fn to_sim_config(&self) -> SimConfig {
        let mut config = if let Some(preset) = self.preset {
            SimConfig::from(preset)
        } else {
            SimConfig::default()
        };

        config.population = self.population;
        config.sensor_angle = self.sensor_angle;
        config.sensor_distance = self.sensor_distance;
        config.rotation_angle = self.rotation_angle;
        config.step_size = self.step_size;
        config.decay_factor = self.decay_factor;
        config.max_brightness = self.max_brightness;

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

        config
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.time_scale < 0.1 || self.time_scale > 10.0 {
            return Err(format!(
                "time_scale must be between 0.1 and 10.0, got {}",
                self.time_scale
            ));
        }
        Ok(())
    }
}

impl Default for Args {
    fn default() -> Self {
        Self {
            live: false,
            screensaver: false,
            print: false,
            capture_frames: false,
            frame_count: 50,
            frame_skip: 50,
            frame_dir: "frames".to_string(),
            seed: None,
            population: 50000,
            sensor_angle: 22.5,
            sensor_distance: 9.0,
            rotation_angle: 45.0,
            step_size: 1.0,
            decay_factor: 0.9,
            max_brightness: 20.0,
            diffusion_kernel: None,
            diffusion_sigma: None,
            preset: Option::<Preset>::None,
            init: InitMode::Random,
            frame_delay: 0.033,
            fps: 30,
            time_scale: 1.0,
            resolution: Resolution {
                width: 400,
                height: 400,
            },
            palette: "forest".to_string(),
            colors: "256".to_string(),
            ascii: false,
            braille: false,
            plain_output: false,
            verbose: false,
            reverse_palette: false,
            invert_palette: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mode_default() {
        let args = Args {
            live: false,
            screensaver: false,
            print: false,
            seed: None,
            population: 50000,
            sensor_angle: 22.5,
            sensor_distance: 9.0,
            rotation_angle: 45.0,
            step_size: 1.0,
            decay_factor: 0.9,
            max_brightness: 20.0,
            diffusion_kernel: None,
            diffusion_sigma: None,
            preset: Option::<Preset>::None,
            init: InitMode::Random,
            frame_delay: 0.033,
            fps: 30,
            time_scale: 1.0,
            resolution: Resolution {
                width: 400,
                height: 400,
            },
            palette: "forest".to_string(),
            colors: "256".to_string(),
            ascii: false,
            braille: false,
            plain_output: false,
            verbose: false,
            capture_frames: false,
            frame_count: 50,
            frame_skip: 50,
            frame_dir: "frames".to_string(),
            reverse_palette: false,
            invert_palette: false,
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
}
