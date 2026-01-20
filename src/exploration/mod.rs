//! Parameter space exploration for discovering optimal presets.
//!
//! This module provides tools for mathematically analyzing the parameter space
//! of the Physarum simulation to discover parameters that produce specific
//! emergent behaviors.

pub mod explorer;
pub mod metrics;

pub use explorer::{EvaluationResult, ExplorationParams, Explorer, ExplorerConfig, PresetBehavior};
pub use metrics::PatternMetrics;
