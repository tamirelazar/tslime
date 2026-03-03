//! Error types for tslime.
//!
//! This module provides structured error types using `thiserror` for consistent
//! error handling across the codebase.

use thiserror::Error;

/// Main error type for tslime operations.
#[derive(Error, Debug)]
pub enum TslimeError {
    /// Validation error - configuration parameter is invalid.
    #[error("validation error: {0}")]
    Validation(#[from] ValidationError),

    /// Configuration error - problem with simulation configuration.
    #[error("configuration error: {0}")]
    Config(#[from] ConfigError),

    /// Rendering error - problem during rendering.
    #[error("rendering error: {0}")]
    Render(String),

    /// Export error - problem during GIF/PNG/WebM export.
    #[error("export error: {0}")]
    Export(String),

    /// IO error - file system or terminal operation failed.
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

/// Validation error - a configuration parameter is out of range or invalid.
#[derive(Error, Debug, Clone)]
pub enum ValidationError {
    /// Value is outside the valid range.
    #[error("{field} must be between {min} and {max}, got {value}")]
    OutOfRange {
        field: String,
        min: String,
        max: String,
        value: String,
    },

    /// Value is below the minimum.
    #[error("{field} must be at least {min}, got {value}")]
    BelowMinimum {
        field: String,
        min: String,
        value: String,
    },

    /// Value is above the maximum.
    #[error("{field} must be at most {max}, got {value}")]
    AboveMaximum {
        field: String,
        max: String,
        value: String,
    },

    /// A required value is empty.
    #[error("{field} cannot be empty")]
    Empty { field: String },

    /// A custom validation error.
    #[error("{0}")]
    Custom(String),
}

impl ValidationError {
    /// Create an out-of-range error for a numeric value.
    pub fn out_of_range<T: std::fmt::Display>(
        field: impl Into<String>,
        min: T,
        max: T,
        value: T,
    ) -> Self {
        ValidationError::OutOfRange {
            field: field.into(),
            min: min.to_string(),
            max: max.to_string(),
            value: value.to_string(),
        }
    }

    /// Create a below-minimum error.
    pub fn below_minimum<T: std::fmt::Display>(field: impl Into<String>, min: T, value: T) -> Self {
        ValidationError::BelowMinimum {
            field: field.into(),
            min: min.to_string(),
            value: value.to_string(),
        }
    }

    /// Create an above-maximum error.
    pub fn above_maximum<T: std::fmt::Display>(field: impl Into<String>, max: T, value: T) -> Self {
        ValidationError::AboveMaximum {
            field: field.into(),
            max: max.to_string(),
            value: value.to_string(),
        }
    }

    /// Create an empty value error.
    pub fn empty(field: impl Into<String>) -> Self {
        ValidationError::Empty {
            field: field.into(),
        }
    }

    /// Create a custom validation error.
    pub fn custom(msg: impl Into<String>) -> Self {
        ValidationError::Custom(msg.into())
    }
}

/// Configuration error - problem with simulation configuration.
#[derive(Error, Debug, Clone, PartialEq)]
pub enum ConfigError {
    /// Invalid resolution dimensions.
    #[error("resolution must be between 10x10 and 2000x2000, got {width}x{height}")]
    InvalidResolution { width: usize, height: usize },

    /// Invalid FPS value.
    #[error("fps must be between 1 and 144, got {fps}")]
    InvalidFps { fps: usize },

    /// Invalid population count.
    #[error("population must be between {min} and {max}, got {pop}")]
    InvalidPopulation { pop: usize, min: usize, max: usize },

    /// Invalid sensor angle.
    #[error("sensor angle must be between {min} and {max}, got {value}")]
    InvalidSensorAngle { value: f32, min: f32, max: f32 },

    /// Invalid sensor distance.
    #[error("sensor distance must be between {min} and {max}, got {value}")]
    InvalidSensorDistance { value: f32, min: f32, max: f32 },

    /// Invalid rotation angle.
    #[error("rotation angle must be between {min} and {max}, got {value}")]
    InvalidRotationAngle { value: f32, min: f32, max: f32 },

    /// Invalid step size.
    #[error("step size must be between {min} and {max}, got {value}")]
    InvalidStepSize { value: f32, min: f32, max: f32 },

    /// Invalid decay factor.
    #[error("decay factor must be between {min} and {max}, got {value}")]
    InvalidDecayFactor { value: f32, min: f32, max: f32 },

    /// Invalid deposit amount.
    #[error("deposit amount must be between {min} and {max}, got {value}")]
    InvalidDepositAmount { value: f32, min: f32, max: f32 },

    /// Invalid max brightness.
    #[error("max brightness must be between {min} and {max}, got {value}")]
    InvalidMaxBrightness { value: f32, min: f32, max: f32 },

    /// Invalid diffusion sigma.
    #[error("diffusion sigma must be between {min} and {max}, got {value}")]
    InvalidDiffusionSigma { value: f32, min: f32, max: f32 },

    /// Invalid time scale.
    #[error("time scale must be between {min} and {max}, got {value}")]
    InvalidTimeScale { value: f32, min: f32, max: f32 },

    /// Invalid attractor strength.
    #[error("attractor strength must be between {min} and {max}, got {value}")]
    InvalidAttractorStrength { value: f32, min: f32, max: f32 },

    /// Invalid terrain strength.
    #[error("terrain strength must be between {min} and {max}, got {value}")]
    InvalidTerrainStrength { value: f32, min: f32, max: f32 },

    /// Failed to parse terrain type.
    #[error("invalid terrain type: {0}")]
    InvalidTerrainType(String),

    /// No species configured.
    #[error("at least one species must be configured")]
    NoSpecies,

    /// Custom configuration error.
    #[error("{0}")]
    Custom(String),
}

/// Result type alias using TslimeError.
pub type Result<T> = std::result::Result<T, TslimeError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_error_out_of_range() {
        let err = ValidationError::out_of_range("test", 0.0, 10.0, 15.0);
        assert!(err.to_string().contains("test"));
        assert!(err.to_string().contains("15"));
    }

    #[test]
    fn test_config_error_population() {
        let err = ConfigError::InvalidPopulation {
            pop: 5,
            min: 1000,
            max: 200_000,
        };
        assert!(err.to_string().contains("5"));
        assert!(err.to_string().contains("1000"));
    }
}
