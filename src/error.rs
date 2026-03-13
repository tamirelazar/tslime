//! Error types for tslime.
//!
//! This module provides structured error types using `thiserror` for consistent
//! error handling across the codebase when the `terminal` feature is enabled.

#![allow(missing_docs)]

#[cfg(feature = "terminal")]
use thiserror::Error;

/// Main error type for tslime operations.
#[cfg_attr(feature = "terminal", derive(Error))]
#[derive(Debug)]
pub enum TslimeError {
    /// Validation error - configuration parameter is invalid.
    #[cfg_attr(feature = "terminal", error("validation error: {0}"))]
    Validation(ValidationError),

    /// Configuration error - problem with simulation configuration.
    #[cfg_attr(feature = "terminal", error("configuration error: {0}"))]
    Config(ConfigError),

    /// Rendering error - problem during rendering.
    #[cfg_attr(feature = "terminal", error("rendering error: {0}"))]
    Render(String),

    /// Export error - problem during GIF/PNG/WebM export.
    #[cfg_attr(feature = "terminal", error("export error: {0}"))]
    Export(String),

    /// IO error - file system or terminal operation failed.
    #[cfg_attr(feature = "terminal", error("io error: {0}"))]
    Io(std::io::Error),
}

#[cfg(not(feature = "terminal"))]
impl std::fmt::Display for TslimeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TslimeError::Validation(e) => write!(f, "validation error: {e}"),
            TslimeError::Config(e) => write!(f, "configuration error: {e}"),
            TslimeError::Render(s) => write!(f, "rendering error: {s}"),
            TslimeError::Export(s) => write!(f, "export error: {s}"),
            TslimeError::Io(e) => write!(f, "io error: {e}"),
        }
    }
}

#[cfg(not(feature = "terminal"))]
impl std::error::Error for TslimeError {}

impl From<ValidationError> for TslimeError {
    fn from(e: ValidationError) -> Self {
        TslimeError::Validation(e)
    }
}

impl From<ConfigError> for TslimeError {
    fn from(e: ConfigError) -> Self {
        TslimeError::Config(e)
    }
}

impl From<std::io::Error> for TslimeError {
    fn from(e: std::io::Error) -> Self {
        TslimeError::Io(e)
    }
}

/// Validation error - a configuration parameter is out of range or invalid.
#[cfg_attr(feature = "terminal", derive(Error))]
#[derive(Debug, Clone)]
pub enum ValidationError {
    /// Value is outside the valid range.
    #[cfg_attr(
        feature = "terminal",
        error("{field} must be between {min} and {max}, got {value}")
    )]
    OutOfRange {
        field: String,
        min: String,
        max: String,
        value: String,
    },

    /// Value is below the minimum.
    #[cfg_attr(
        feature = "terminal",
        error("{field} must be at least {min}, got {value}")
    )]
    BelowMinimum {
        field: String,
        min: String,
        value: String,
    },

    /// Value is above the maximum.
    #[cfg_attr(
        feature = "terminal",
        error("{field} must be at most {max}, got {value}")
    )]
    AboveMaximum {
        field: String,
        max: String,
        value: String,
    },

    /// A required value is empty.
    #[cfg_attr(feature = "terminal", error("{field} cannot be empty"))]
    Empty { field: String },

    /// A custom validation error.
    #[cfg_attr(feature = "terminal", error("{0}"))]
    Custom(String),
}

#[cfg(not(feature = "terminal"))]
impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValidationError::OutOfRange {
                field,
                min,
                max,
                value,
            } => {
                write!(f, "{field} must be between {min} and {max}, got {value}")
            }
            ValidationError::BelowMinimum { field, min, value } => {
                write!(f, "{field} must be at least {min}, got {value}")
            }
            ValidationError::AboveMaximum { field, max, value } => {
                write!(f, "{field} must be at most {max}, got {value}")
            }
            ValidationError::Empty { field } => {
                write!(f, "{field} cannot be empty")
            }
            ValidationError::Custom(s) => write!(f, "{s}"),
        }
    }
}

#[cfg(not(feature = "terminal"))]
impl std::error::Error for ValidationError {}

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
#[cfg_attr(feature = "terminal", derive(Error))]
#[derive(Debug, Clone, PartialEq)]
pub enum ConfigError {
    /// Invalid resolution dimensions.
    #[cfg_attr(
        feature = "terminal",
        error("resolution must be between 10x10 and 2000x2000, got {width}x{height}")
    )]
    InvalidResolution { width: usize, height: usize },

    /// Invalid FPS value.
    #[cfg_attr(
        feature = "terminal",
        error("fps must be between 1 and 144, got {fps}")
    )]
    InvalidFps { fps: usize },

    /// Invalid population count.
    #[cfg_attr(
        feature = "terminal",
        error("population must be between {min} and {max}, got {pop}")
    )]
    InvalidPopulation { pop: usize, min: usize, max: usize },

    /// Invalid sensor angle.
    #[cfg_attr(
        feature = "terminal",
        error("sensor angle must be between {min} and {max}, got {value}")
    )]
    InvalidSensorAngle { value: f32, min: f32, max: f32 },

    /// Invalid sensor distance.
    #[cfg_attr(
        feature = "terminal",
        error("sensor distance must be between {min} and {max}, got {value}")
    )]
    InvalidSensorDistance { value: f32, min: f32, max: f32 },

    /// Invalid rotation angle.
    #[cfg_attr(
        feature = "terminal",
        error("rotation angle must be between {min} and {max}, got {value}")
    )]
    InvalidRotationAngle { value: f32, min: f32, max: f32 },

    /// Invalid step size.
    #[cfg_attr(
        feature = "terminal",
        error("step size must be between {min} and {max}, got {value}")
    )]
    InvalidStepSize { value: f32, min: f32, max: f32 },

    /// Invalid decay factor.
    #[cfg_attr(
        feature = "terminal",
        error("decay factor must be between {min} and {max}, got {value}")
    )]
    InvalidDecayFactor { value: f32, min: f32, max: f32 },

    /// Invalid deposit amount.
    #[cfg_attr(
        feature = "terminal",
        error("deposit amount must be between {min} and {max}, got {value}")
    )]
    InvalidDepositAmount { value: f32, min: f32, max: f32 },

    /// Invalid max brightness.
    #[cfg_attr(
        feature = "terminal",
        error("max brightness must be between {min} and {max}, got {value}")
    )]
    InvalidMaxBrightness { value: f32, min: f32, max: f32 },

    /// Invalid diffusion sigma.
    #[cfg_attr(
        feature = "terminal",
        error("diffusion sigma must be between {min} and {max}, got {value}")
    )]
    InvalidDiffusionSigma { value: f32, min: f32, max: f32 },

    /// Invalid time scale.
    #[cfg_attr(
        feature = "terminal",
        error("time scale must be between {min} and {max}, got {value}")
    )]
    InvalidTimeScale { value: f32, min: f32, max: f32 },

    /// Invalid attractor strength.
    #[cfg_attr(
        feature = "terminal",
        error("attractor strength must be between {min} and {max}, got {value}")
    )]
    InvalidAttractorStrength {
        /// The invalid value.
        value: f32,
        /// Minimum allowed value.
        min: f32,
        /// Maximum allowed value.
        max: f32,
    },

    /// Invalid terrain strength.
    #[cfg_attr(
        feature = "terminal",
        error("terrain strength must be between {min} and {max}, got {value}")
    )]
    InvalidTerrainStrength {
        /// The invalid value.
        value: f32,
        /// Minimum allowed value.
        min: f32,
        /// Maximum allowed value.
        max: f32,
    },

    /// Failed to parse terrain type.
    #[cfg_attr(feature = "terminal", error("invalid terrain type: {0}"))]
    InvalidTerrainType(String),

    /// No species configured.
    #[cfg_attr(feature = "terminal", error("at least one species must be configured"))]
    NoSpecies,

    /// Custom configuration error.
    #[cfg_attr(feature = "terminal", error("{0}"))]
    Custom(String),
}

#[cfg(not(feature = "terminal"))]
impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigError::InvalidResolution { width, height } => {
                write!(
                    f,
                    "resolution must be between 10x10 and 2000x2000, got {width}x{height}"
                )
            }
            ConfigError::InvalidFps { fps } => {
                write!(f, "fps must be between 1 and 144, got {fps}")
            }
            ConfigError::InvalidPopulation { pop, min, max } => {
                write!(f, "population must be between {min} and {max}, got {pop}")
            }
            ConfigError::InvalidSensorAngle { value, min, max } => {
                write!(
                    f,
                    "sensor angle must be between {min} and {max}, got {value}"
                )
            }
            ConfigError::InvalidSensorDistance { value, min, max } => {
                write!(
                    f,
                    "sensor distance must be between {min} and {max}, got {value}"
                )
            }
            ConfigError::InvalidRotationAngle { value, min, max } => {
                write!(
                    f,
                    "rotation angle must be between {min} and {max}, got {value}"
                )
            }
            ConfigError::InvalidStepSize { value, min, max } => {
                write!(f, "step size must be between {min} and {max}, got {value}")
            }
            ConfigError::InvalidDecayFactor { value, min, max } => {
                write!(
                    f,
                    "decay factor must be between {min} and {max}, got {value}"
                )
            }
            ConfigError::InvalidDepositAmount { value, min, max } => {
                write!(
                    f,
                    "deposit amount must be between {min} and {max}, got {value}"
                )
            }
            ConfigError::InvalidMaxBrightness { value, min, max } => {
                write!(
                    f,
                    "max brightness must be between {min} and {max}, got {value}"
                )
            }
            ConfigError::InvalidDiffusionSigma { value, min, max } => {
                write!(
                    f,
                    "diffusion sigma must be between {min} and {max}, got {value}"
                )
            }
            ConfigError::InvalidTimeScale { value, min, max } => {
                write!(f, "time scale must be between {min} and {max}, got {value}")
            }
            ConfigError::InvalidAttractorStrength { value, min, max } => {
                write!(
                    f,
                    "attractor strength must be between {min} and {max}, got {value}"
                )
            }
            ConfigError::InvalidTerrainStrength { value, min, max } => {
                write!(
                    f,
                    "terrain strength must be between {min} and {max}, got {value}"
                )
            }
            ConfigError::InvalidTerrainType(s) => write!(f, "invalid terrain type: {s}"),
            ConfigError::NoSpecies => write!(f, "at least one species must be configured"),
            ConfigError::Custom(s) => write!(f, "{s}"),
        }
    }
}

#[cfg(not(feature = "terminal"))]
impl std::error::Error for ConfigError {}

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
