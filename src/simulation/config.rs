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

#[derive(Debug, Clone)]
pub struct SimConfig {
    pub population: usize,
    pub sensor_angle: f32,
    pub sensor_distance: f32,
    pub rotation_angle: f32,
    pub step_size: f32,
    pub decay_factor: f32,
    pub deposit_amount: f32,
    pub diffusion_kernel: DiffusionKernel,
    pub diffusion_sigma: f32,
    pub max_brightness: f32,
}

impl Default for SimConfig {
    fn default() -> Self {
        Self {
            population: 50_000,
            sensor_angle: 22.5,
            sensor_distance: 9.0,
            rotation_angle: 45.0,
            step_size: 1.0,
            decay_factor: 0.5,
            deposit_amount: 5.0,
            diffusion_kernel: DiffusionKernel::Mean3x3,
            diffusion_sigma: 1.0,
            max_brightness: 20.0,
        }
    }
}

impl SimConfig {
    pub fn validate(&self) -> Result<(), String> {
        if self.population < 1000 || self.population > 200_000 {
            return Err(format!(
                "population must be between 1000 and 200000, got {}",
                self.population
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
        Ok(())
    }
}

impl From<Preset> for SimConfig {
    fn from(preset: Preset) -> Self {
        match preset {
            Preset::Network => Self {
                population: 50_000,
                sensor_angle: 15.0,
                sensor_distance: 9.0,
                rotation_angle: 30.0,
                step_size: 1.0,
                decay_factor: 0.85,
                deposit_amount: 5.0,
                diffusion_kernel: DiffusionKernel::Mean3x3,
                diffusion_sigma: 1.0,
                max_brightness: 20.0,
            },
            Preset::Exploratory => Self {
                population: 30_000,
                sensor_angle: 45.0,
                sensor_distance: 15.0,
                rotation_angle: 60.0,
                step_size: 1.0,
                decay_factor: 0.96,
                deposit_amount: 3.0,
                diffusion_kernel: DiffusionKernel::Mean3x3,
                diffusion_sigma: 1.0,
                max_brightness: 12.0,
            },
            Preset::Tendrils => Self {
                population: 40_000,
                sensor_angle: 30.0,
                sensor_distance: 12.0,
                rotation_angle: 45.0,
                step_size: 2.0,
                decay_factor: 0.90,
                deposit_amount: 4.0,
                diffusion_kernel: DiffusionKernel::Mean3x3,
                diffusion_sigma: 1.0,
                max_brightness: 16.0,
            },
            Preset::Organic => Self::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = SimConfig::default();
        assert_eq!(config.population, 50_000);
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
            population: 500,
            ..Default::default()
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_validate_population_too_high() {
        let config = SimConfig {
            population: 300_000,
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
}
