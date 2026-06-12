//! Parameter space exploration for discovering optimal presets.
//!
//! Searches the Physarum simulation's parameter space, scoring candidates
//! with pattern metrics to find parameters that produce specific emergent
//! behaviors.

pub mod explorer;
pub mod metrics;

pub use explorer::{EvaluationResult, ExplorationParams, Explorer, ExplorerConfig, PresetBehavior};
pub use metrics::PatternMetrics;
