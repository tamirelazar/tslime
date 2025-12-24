use clap::Parser;
use std::num::ParseIntError;
use std::str::FromStr;

use crate::simulation::config::{Preset, SimConfig};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Default,
    Live,
    Screensaver,
    Print,
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
        long = "preset",
        value_name = "NAME",
        help = "Use named preset (network, exploratory, tendrils, organic)"
    )]
    pub preset: Option<Preset>,

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
        long = "resolution",
        value_name = "WxH",
        default_value = "400x400",
        help = "Simulation resolution"
    )]
    pub resolution: Resolution,

    #[arg(
        long = "palette",
        value_name = "NAME",
        default_value = "organic",
        help = "Color palette (organic, heat, ocean, mono)"
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
        short = 'v',
        long = "verbose",
        help = "Print performance stats to stderr"
    )]
    pub verbose: bool,
}

impl Args {
    pub fn mode(&self) -> Mode {
        if self.screensaver {
            Mode::Screensaver
        } else if self.live {
            Mode::Live
        } else if self.print {
            Mode::Print
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

        config
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
            decay_factor: 0.9,
            preset: Option::<Preset>::None,
            frame_delay: 0.033,
            fps: 30,
            resolution: Resolution {
                width: 400,
                height: 400,
            },
            palette: "organic".to_string(),
            colors: "256".to_string(),
            ascii: false,
            braille: false,
            verbose: false,
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
            preset: Option::<Preset>::None,
            frame_delay: 0.033,
            fps: 30,
            resolution: Resolution {
                width: 400,
                height: 400,
            },
            palette: "organic".to_string(),
            colors: "256".to_string(),
            ascii: false,
            braille: false,
            verbose: false,
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
}
