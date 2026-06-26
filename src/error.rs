//! Error types for tslime.
//!
//! The `thiserror` derives are gated on the `terminal` feature; without it,
//! `Display` and `Error` are implemented by hand so the types work everywhere.

#![allow(missing_docs)]

#[cfg(feature = "terminal")]
use thiserror::Error;

/// Main error type for tslime operations.
#[cfg_attr(feature = "terminal", derive(Error))]
#[derive(Debug)]
pub enum TslimeError {
    /// A configuration parameter is invalid.
    #[cfg_attr(feature = "terminal", error("validation error: {0}"))]
    Validation(ValidationError),

    /// Rendering failed.
    #[cfg_attr(feature = "terminal", error("rendering error: {0}"))]
    Render(String),

    /// GIF/PNG/WebM export failed.
    #[cfg_attr(feature = "terminal", error("export error: {0}"))]
    Export(String),

    /// A file system or terminal operation failed.
    #[cfg_attr(feature = "terminal", error("io error: {0}"))]
    Io(std::io::Error),
}

#[cfg(not(feature = "terminal"))]
impl std::fmt::Display for TslimeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TslimeError::Validation(e) => write!(f, "validation error: {e}"),
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
}
