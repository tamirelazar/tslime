//! Validation utilities for configuration and runtime parameters.
//!
//! This module provides a centralized validation framework with reusable validation rules
//! and the [`Validatable`] trait for types that need validation.

use crate::error::ValidationError;
use std::ops::RangeInclusive;

/// Trait for types that can be validated.
///
/// Implement this trait for any configuration type that needs validation
/// before being used in the simulation.
///
/// # Example
/// ```
/// use tslime::validation::Validatable;
/// use tslime::error::ValidationError;
///
/// struct MyConfig {
///     value: f32,
/// }
///
/// impl Validatable for MyConfig {
///     fn validate(&self) -> Result<(), ValidationError> {
///         if self.value < 0.0 || self.value > 1.0 {
///             Err(ValidationError::out_of_range("value", 0.0, 1.0, self.value))
///         } else {
///             Ok(())
///         }
///     }
/// }
/// ```
pub trait Validatable {
    /// Validates the configuration, returning a descriptive error if invalid.
    fn validate(&self) -> Result<(), ValidationError>;
}

/// A validation rule that checks if a value is within an inclusive range.
#[derive(Debug, Clone, Copy)]
pub struct RangeRule<T: PartialOrd> {
    /// Minimum acceptable value (inclusive).
    pub min: T,
    /// Maximum acceptable value (inclusive).
    pub max: T,
    /// Name of the field being validated (for error messages).
    pub field: &'static str,
}

impl<T: PartialOrd + std::fmt::Display + Clone> RangeRule<T> {
    /// Creates a new range validation rule.
    pub const fn new(min: T, max: T, field: &'static str) -> Self {
        Self { min, max, field }
    }

    /// Validates that a value is within the range.
    pub fn validate(&self, value: &T) -> Result<(), ValidationError> {
        if value < &self.min || value > &self.max {
            Err(ValidationError::out_of_range(
                self.field,
                self.min.clone(),
                self.max.clone(),
                value.clone(),
            ))
        } else {
            Ok(())
        }
    }

    /// Returns the valid range as a RangeInclusive.
    pub fn range(&self) -> RangeInclusive<T> {
        self.min.clone()..=self.max.clone()
    }
}

impl RangeRule<f32> {
    /// By-value convenience over the generic by-reference [`validate`](Self::validate).
    pub fn validate_f32(&self, value: f32) -> Result<(), ValidationError> {
        if value < self.min || value > self.max {
            Err(ValidationError::out_of_range(
                self.field, self.min, self.max, value,
            ))
        } else {
            Ok(())
        }
    }
}

impl RangeRule<usize> {
    /// By-value convenience over the generic by-reference [`validate`](Self::validate).
    pub fn validate_usize(&self, value: usize) -> Result<(), ValidationError> {
        if value < self.min || value > self.max {
            Err(ValidationError::out_of_range(
                self.field, self.min, self.max, value,
            ))
        } else {
            Ok(())
        }
    }
}

/// Predefined validation rules for common simulation parameters.
pub mod rules {
    use super::RangeRule;
    use crate::config_defaults::{agent, environment, population, time, trail};

    /// Validation rule for sensor angle (5.0 to 90.0 degrees).
    pub const SENSOR_ANGLE: RangeRule<f32> = RangeRule::new(
        agent::MIN_SENSOR_ANGLE,
        agent::MAX_SENSOR_ANGLE,
        "sensor_angle",
    );

    /// Validation rule for sensor distance (1.0 to 50.0 pixels).
    pub const SENSOR_DISTANCE: RangeRule<f32> = RangeRule::new(
        agent::MIN_SENSOR_DISTANCE,
        agent::MAX_SENSOR_DISTANCE,
        "sensor_distance",
    );

    /// Validation rule for rotation angle (5.0 to 90.0 degrees).
    pub const ROTATION_ANGLE: RangeRule<f32> = RangeRule::new(
        agent::MIN_ROTATION_ANGLE,
        agent::MAX_ROTATION_ANGLE,
        "rotation_angle",
    );

    /// Validation rule for step size (0.01 to 10.0 pixels).
    pub const STEP_SIZE: RangeRule<f32> =
        RangeRule::new(agent::MIN_STEP_SIZE, agent::MAX_STEP_SIZE, "step_size");

    /// Validation rule for deposit amount (0.1 to 20.0).
    pub const DEPOSIT_AMOUNT: RangeRule<f32> = RangeRule::new(
        agent::MIN_DEPOSIT_AMOUNT,
        agent::MAX_DEPOSIT_AMOUNT,
        "deposit_amount",
    );

    /// Validation rule for decay factor (0.5 to 0.9999).
    pub const DECAY_FACTOR: RangeRule<f32> = RangeRule::new(
        trail::MIN_DECAY_FACTOR,
        trail::MAX_DECAY_FACTOR,
        "decay_factor",
    );

    /// Validation rule for max brightness (1.0 to 1000.0).
    pub const MAX_BRIGHTNESS: RangeRule<f32> = RangeRule::new(
        trail::MIN_MAX_BRIGHTNESS,
        trail::MAX_MAX_BRIGHTNESS,
        "max_brightness",
    );

    /// Validation rule for diffusion sigma (0.5 to 4.0).
    pub const DIFFUSION_SIGMA: RangeRule<f32> = RangeRule::new(
        trail::MIN_DIFFUSION_SIGMA,
        trail::MAX_DIFFUSION_SIGMA,
        "diffusion_sigma",
    );

    /// Validation rule for decay gamma (0.25 to 2.0).
    pub const DECAY_GAMMA: RangeRule<f32> = RangeRule::new(
        trail::MIN_DECAY_GAMMA,
        trail::MAX_DECAY_GAMMA,
        "decay_gamma",
    );

    /// Validation rule for afterglow strength (0.0 to 1.0).
    pub const AFTERGLOW: RangeRule<f32> =
        RangeRule::new(trail::MIN_AFTERGLOW, trail::MAX_AFTERGLOW, "afterglow");

    /// Validation rule for afterglow EMA rate (0.001 to 1.0).
    pub const AFTERGLOW_RATE: RangeRule<f32> = RangeRule::new(
        trail::MIN_AFTERGLOW_RATE,
        trail::MAX_AFTERGLOW_RATE,
        "afterglow_rate",
    );

    /// Validation rule for diffuse-weight blend (0.0 to 1.0).
    pub const DIFFUSE_WEIGHT: RangeRule<f32> = RangeRule::new(
        trail::MIN_DIFFUSE_WEIGHT,
        trail::MAX_DIFFUSE_WEIGHT,
        "diffuse_weight",
    );

    /// Validation rule for deposit scale.
    pub const DEPOSIT_SCALE: RangeRule<f32> = RangeRule::new(
        trail::MIN_DEPOSIT_SCALE,
        trail::MAX_DEPOSIT_SCALE,
        "deposit_scale",
    );

    /// Validation rule for deposit gamma (Pow exponent).
    pub const DEPOSIT_GAMMA: RangeRule<f32> = RangeRule::new(
        trail::MIN_DEPOSIT_GAMMA,
        trail::MAX_DEPOSIT_GAMMA,
        "deposit_gamma",
    );

    /// Validation rule for deposit cap (0.0 = off).
    pub const DEPOSIT_CAP: RangeRule<f32> = RangeRule::new(
        trail::MIN_DEPOSIT_CAP,
        trail::MAX_DEPOSIT_CAP,
        "deposit_cap",
    );

    /// Validation rule for time scale (0.1 to 10.0).
    pub const TIME_SCALE: RangeRule<f32> =
        RangeRule::new(time::MIN_TIME_SCALE, time::MAX_TIME_SCALE, "time_scale");

    /// Validation rule for attractor strength (0.1 to 10.0).
    pub const ATTRACTOR_STRENGTH: RangeRule<f32> = RangeRule::new(
        environment::MIN_ATTRACTOR_STRENGTH,
        environment::MAX_ATTRACTOR_STRENGTH,
        "attractor_strength",
    );

    /// Validation rule for terrain strength (0.1 to 5.0).
    pub const TERRAIN_STRENGTH: RangeRule<f32> = RangeRule::new(
        environment::MIN_TERRAIN_STRENGTH,
        environment::MAX_TERRAIN_STRENGTH,
        "terrain_strength",
    );

    /// Validation rule for population count (1000 to 200_000).
    pub const POPULATION: RangeRule<usize> = RangeRule::new(
        population::MIN_POPULATION,
        population::MAX_POPULATION,
        "population",
    );
}

/// Validates that `value` falls within the inclusive range `min..=max`;
/// `name` labels the parameter in the error message.
///
/// # Examples
/// ```
/// use tslime::validation::validate_range;
///
/// assert!(validate_range(5.0, 0.0, 10.0, "test_param").is_ok());
/// assert!(validate_range(15.0, 0.0, 10.0, "test_param").is_err());
/// ```
pub fn validate_range<T: PartialOrd + std::fmt::Display>(
    value: T,
    min: T,
    max: T,
    name: &str,
) -> Result<(), ValidationError> {
    if value < min || value > max {
        Err(ValidationError::out_of_range(name, min, max, value))
    } else {
        Ok(())
    }
}

/// Validates that `value` is at least `min` (inclusive); `name` labels the
/// parameter in the error message.
pub fn validate_min<T: PartialOrd + std::fmt::Display>(
    value: T,
    min: T,
    name: &str,
) -> Result<(), ValidationError> {
    if value < min {
        Err(ValidationError::below_minimum(name, min, value))
    } else {
        Ok(())
    }
}

/// Validates that `value` is at most `max` (inclusive); `name` labels the
/// parameter in the error message.
pub fn validate_max<T: PartialOrd + std::fmt::Display>(
    value: T,
    max: T,
    name: &str,
) -> Result<(), ValidationError> {
    if value > max {
        Err(ValidationError::above_maximum(name, max, value))
    } else {
        Ok(())
    }
}

/// Validates that a string is non-empty after trimming whitespace.
pub fn validate_not_empty(value: &str, name: &str) -> Result<(), ValidationError> {
    if value.trim().is_empty() {
        Err(ValidationError::empty(name))
    } else {
        Ok(())
    }
}

/// Validates that a slice is not empty.
pub fn validate_vec_not_empty<T>(value: &[T], name: &str) -> Result<(), ValidationError> {
    if value.is_empty() {
        Err(ValidationError::custom(format!("{} cannot be empty", name)))
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_range_in_range() {
        assert!(validate_range(5.0, 0.0, 10.0, "test").is_ok());
    }

    #[test]
    fn test_validate_range_below_min() {
        let result = validate_range(-1.0, 0.0, 10.0, "test_param");
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("test_param"));
        assert!(err_msg.contains("0"));
        assert!(err_msg.contains("10"));
    }

    #[test]
    fn test_validate_range_above_max() {
        let result = validate_range(15.0, 0.0, 10.0, "test_param");
        assert!(result.is_err());
    }

    #[test]
    fn test_range_rule_validate() {
        let rule = RangeRule::new(0.0, 10.0, "test");
        assert!(rule.validate_f32(5.0).is_ok());
        assert!(rule.validate_f32(15.0).is_err());
    }

    #[test]
    fn test_validation_rules() {
        // Test that all predefined rules have valid ranges
        assert!(rules::SENSOR_ANGLE.validate_f32(22.5).is_ok());
        assert!(rules::SENSOR_DISTANCE.validate_f32(9.0).is_ok());
        assert!(rules::POPULATION.validate_usize(50_000).is_ok());
    }

    #[test]
    fn test_validate_min() {
        assert!(validate_min(5, 0, "test").is_ok());
        assert!(validate_min(-1, 0, "test").is_err());
    }

    #[test]
    fn test_validate_max() {
        assert!(validate_max(5, 10, "test").is_ok());
        assert!(validate_max(15, 10, "test").is_err());
    }

    #[test]
    fn test_validate_not_empty() {
        assert!(validate_not_empty("hello", "test").is_ok());
        assert!(validate_not_empty("", "test").is_err());
        assert!(validate_not_empty("   ", "test").is_err());
    }

    #[test]
    fn test_validate_vec_not_empty() {
        assert!(validate_vec_not_empty(&[1, 2, 3], "test").is_ok());
        assert!(validate_vec_not_empty(&Vec::<i32>::new(), "test").is_err());
    }

    #[test]
    fn diffusion_sigma_accepts_widened_range() {
        assert!(rules::DIFFUSION_SIGMA.validate(&3.5).is_ok());
        assert!(rules::DIFFUSION_SIGMA.validate(&4.0).is_ok());
        assert!(rules::DIFFUSION_SIGMA.validate(&4.5).is_err());
    }

    #[test]
    fn decay_gamma_rule_bounds() {
        assert!(rules::DECAY_GAMMA.validate(&1.0).is_ok());
        assert!(rules::DECAY_GAMMA.validate(&0.25).is_ok());
        assert!(rules::DECAY_GAMMA.validate(&0.1).is_err());
    }

    #[test]
    fn deposit_rules_accept_defaults_reject_out_of_range() {
        use super::rules;
        assert!(rules::DEPOSIT_SCALE.validate_f32(1.0).is_ok());
        assert!(rules::DEPOSIT_SCALE.validate_f32(-0.1).is_err());
        assert!(rules::DEPOSIT_SCALE.validate_f32(11.0).is_err());
        assert!(rules::DEPOSIT_GAMMA.validate_f32(1.0).is_ok());
        assert!(rules::DEPOSIT_GAMMA.validate_f32(0.05).is_err());
        assert!(rules::DEPOSIT_CAP.validate_f32(0.0).is_ok());
        assert!(rules::DEPOSIT_CAP.validate_f32(-1.0).is_err());
    }
}
