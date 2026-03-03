//! Validation utilities for configuration and runtime parameters.
//!
//! This module provides a centralized validation framework to ensure
//! consistency across CLI arguments, simulation configuration, and runtime state.

/// Trait for types that can be validated.
///
/// Implement this trait for any configuration type that needs validation
/// before being used in the simulation.
pub trait Validatable {
    /// Validates the configuration and returns an error if invalid.
    ///
    /// # Returns
    /// - `Ok(())` if the configuration is valid
    /// - `Err(String)` with a descriptive error message if invalid
    fn validate(&self) -> Result<(), String>;
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
/// - `Err(String)` with a descriptive error message if out of range
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
) -> Result<(), String> {
    if value < min || value > max {
        Err(format!(
            "{} must be between {} and {}, got {}",
            name, min, max, value
        ))
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
/// - `Err(String)` with a descriptive error message if too small
pub fn validate_min<T: PartialOrd + std::fmt::Display>(
    value: T,
    min: T,
    name: &str,
) -> Result<(), String> {
    if value < min {
        Err(format!("{} must be at least {}, got {}", name, min, value))
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
/// - `Err(String)` with a descriptive error message if too large
pub fn validate_max<T: PartialOrd + std::fmt::Display>(
    value: T,
    max: T,
    name: &str,
) -> Result<(), String> {
    if value > max {
        Err(format!("{} must be at most {}, got {}", name, max, value))
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
/// - `Err(String)` with a descriptive error message if empty
pub fn validate_not_empty(value: &str, name: &str) -> Result<(), String> {
    if value.trim().is_empty() {
        Err(format!("{} cannot be empty", name))
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
/// - `Err(String)` with a descriptive error message if empty
pub fn validate_vec_not_empty<T>(value: &[T], name: &str) -> Result<(), String> {
    if value.is_empty() {
        Err(format!("{} cannot be empty", name))
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
        let err_msg = result.unwrap_err();
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
