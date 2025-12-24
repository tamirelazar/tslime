#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
pub enum DiffusionKernel {
    Mean3x3,
    Gaussian,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct SimConfig {
    pub population: usize,
    pub sensor_angle: f32,
    pub sensor_distance: f32,
    pub rotation_angle: f32,
    pub step_size: f32,
    pub decay_factor: f32,
    pub deposit_amount: f32,
    pub diffusion_kernel: DiffusionKernel,
}

impl Default for SimConfig {
    fn default() -> Self {
        Self {
            population: 50_000,
            sensor_angle: 22.5,
            sensor_distance: 9.0,
            rotation_angle: 45.0,
            step_size: 1.0,
            decay_factor: 0.9,
            deposit_amount: 5.0,
            diffusion_kernel: DiffusionKernel::Mean3x3,
        }
    }
}

#[allow(dead_code)]
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
        Ok(())
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
        assert_eq!(config.decay_factor, 0.9);
        assert_eq!(config.deposit_amount, 5.0);
    }

    #[test]
    fn test_validate_default() {
        let config = SimConfig::default();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_validate_population_too_low() {
        let mut config = SimConfig::default();
        config.population = 500;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_validate_population_too_high() {
        let mut config = SimConfig::default();
        config.population = 300_000;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_validate_sensor_angle() {
        let mut config = SimConfig::default();
        config.sensor_angle = 100.0;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_validate_decay_factor() {
        let mut config = SimConfig::default();
        config.decay_factor = 1.0;
        assert!(config.validate().is_err());
    }
}
