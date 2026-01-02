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
    pub species_configs: Vec<SpeciesConfig>,
    pub separate_species_trails: bool,
    pub use_simd: bool,
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
            species_configs: vec![SpeciesConfig::default()],
            separate_species_trails: false,
            use_simd: true,
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
        Ok(())
    }

    pub fn total_population(&self) -> usize {
        self.species_configs.iter().map(|s| s.count).sum()
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
            },
            Preset::Organic => Self::default(),
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
}
