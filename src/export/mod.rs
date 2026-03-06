//! Export functionality for saving simulation output.
//!
//! This module provides various export formats for saving the simulation output:
//!
//! - **GIF**: Animated GIF export with configurable frame rate and duration
//! - **PNG**: Single frame capture as PNG image
//! - **WebM**: Video export using FFmpeg (requires FFmpeg to be installed)
//!
//! # Example
//!
//! ```rust,no_run
//! use tslime::export::GifExporter;
//!
//! let mut exporter = GifExporter::new(400, 400, "output.gif", 30).unwrap();
//! // ... capture frames ...
//! exporter.finish("output.gif").unwrap();
//! ```

/// GIF export functionality.
pub mod gif;
/// PNG frame saving.
pub mod png;
/// WebM export functionality.
pub mod webm;

pub use gif::GifExporter;
pub use webm::WebmExporter;
