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
    /// Validates the configuration and returns an error if invalid.
    ///
    /// # Returns
    /// - `Ok(())` if the configuration is valid
    /// - `Err(ValidationError)` with a descriptive error if invalid
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
    /// Validates that a value is within the range.
    /// Specialized implementation for f32 to avoid Clone issues.
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
    /// Validates that a value is within the range.
    /// Specialized implementation for usize to avoid Clone issues.
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

    /// Validation rule for diffusion sigma (0.5 to 2.0).
    pub const DIFFUSION_SIGMA: RangeRule<f32> = RangeRule::new(
        trail::MIN_DIFFUSION_SIGMA,
        trail::MAX_DIFFUSION_SIGMA,
        "diffusion_sigma",
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

/// Validates that a value falls within an inclusive range.
///
/// # Type Parameters
/// * `T` - The type of value to validate (must implement PartialOrd)
///
/// # Arguments
/// * `value` - The value to validate
/// * `min` - The minimum acceptable value (inclusive)
/// * `max` - The maximum acceptable value (inclusive)
/// * `name` - The name of the parameter for error messages
///
/// # Returns
/// - `Ok(())` if the value is within range
/// - `Err(ValidationError)` with a descriptive error message if out of range
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

/// Validates that a value is at least a minimum value.
///
/// # Arguments
/// * `value` - The value to validate
/// * `min` - The minimum acceptable value (inclusive)
/// * `name` - The name of the parameter for error messages
///
/// # Returns
/// - `Ok(())` if the value meets the minimum
/// - `Err(ValidationError)` with a descriptive error message if too small
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

/// Validates that a value is at most a maximum value.
///
/// # Arguments
/// * `value` - The value to validate
/// * `max` - The maximum acceptable value (inclusive)
/// * `name` - The name of the parameter for error messages
///
/// # Returns
/// - `Ok(())` if the value meets the maximum
/// - `Err(ValidationError)` with a descriptive error message if too large
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

/// Validates that a string is not empty.
///
/// # Arguments
/// * `value` - The string to validate
/// * `name` - The name of the parameter for error messages
///
/// # Returns
/// - `Ok(())` if the string is not empty
/// - `Err(ValidationError)` with a descriptive error message if empty
pub fn validate_not_empty(value: &str, name: &str) -> Result<(), ValidationError> {
    if value.trim().is_empty() {
        Err(ValidationError::empty(name))
    } else {
        Ok(())
    }
}

/// Validates that a vector is not empty.
///
/// # Arguments
/// * `value` - The vector to validate
/// * `name` - The name of the parameter for error messages
///
/// # Returns
/// - `Ok(())` if the vector is not empty
/// - `Err(ValidationError)` with a descriptive error message if empty
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
}
