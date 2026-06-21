//! `AppRuntimeConfig` — restart-only levers that have no home in `SimConfig` or
//! `ResolvedRenderConfig`.
//!
//! Holds every observable `args.*` setting the runner reads directly that is NOT
//! a sim lever (simulation parameters) or render lever (visual pipeline config).
//! Defaults source from `crate::config_defaults` constants — no magic numbers here.

use crate::render::grid::GridStyle;
use crate::render::palette::RgbColor;
use serde::{Deserialize, Serialize};

/// Restart-only application-level runtime configuration.
///
/// These are the levers the runner reads from `args.*` at startup that don't fit
/// in `SimConfig` (simulation algorithm params) or `ResolvedRenderConfig` (visual
/// pipeline params). `Profile` carries one of these alongside `sim`/`render`/`seed`
/// so saved configs and preset switches can round-trip them.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub(crate) struct AppRuntimeConfig {
    // ── warmup ──
    /// Number of warmup frames at simulation start.
    pub warmup_frames: usize,
    /// Skip the warmup phase entirely.
    pub skip_warmup: bool,
    /// Brightness multiplier applied during warmup.
    pub warmup_brightness_multiplier: f32,

    // ── auto-reset ──
    /// Enable automatic reset when the simulation collapses.
    pub auto_reset: bool,
    /// Entropy threshold above which the simulation is considered collapsed.
    pub auto_reset_entropy_threshold: f32,
    /// Number of frames the simulation must remain collapsed before auto-reset fires.
    pub auto_reset_duration_frames: usize,

    // ── background grid (all five GridRenderer::new params) ──
    /// Enable background grid rendering.
    pub grid: bool,
    /// Visual style of the grid.
    pub grid_style: GridStyle,
    /// Grid cell size (number of cells per dimension).
    pub grid_size: usize,
    /// Grid line color.
    pub grid_color: RgbColor,
    /// Base opacity of the grid (0.0–1.0).
    pub grid_opacity: f32,
    /// Whether grid opacity adapts to trail density.
    pub grid_adaptive: bool,

    // ── food persistence params ──
    /// Strength of persistent food attractors.
    pub food_persist_strength: f32,
    /// Radius of persistent food attractors.
    pub food_persist_radius: f32,
    /// Duration of food persistence in frames.
    pub food_persist_duration: usize,
}

impl Default for AppRuntimeConfig {
    fn default() -> Self {
        use crate::config_defaults::{auto_reset, food_persist, grid, warmup};

        // grid_color: config_defaults::palette::DEFAULT_GRID_COLOR is "ffffff" hex.
        let grid_color = RgbColor::from_hex(0xffffff);

        Self {
            warmup_frames: warmup::DEFAULT_WARMUP_FRAMES,
            skip_warmup: false,
            warmup_brightness_multiplier: warmup::DEFAULT_BRIGHTNESS_MULTIPLIER,

            auto_reset: false,
            auto_reset_entropy_threshold: auto_reset::DEFAULT_ENTROPY_THRESHOLD,
            auto_reset_duration_frames: auto_reset::DEFAULT_DURATION_FRAMES,

            grid: false,
            grid_style: GridStyle::Cross,
            grid_size: grid::DEFAULT_GRID_SIZE,
            grid_color,
            grid_opacity: grid::DEFAULT_GRID_OPACITY,
            grid_adaptive: false,

            food_persist_strength: food_persist::DEFAULT_STRENGTH,
            food_persist_radius: food_persist::DEFAULT_RADIUS,
            food_persist_duration: food_persist::DEFAULT_DURATION,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_sources_from_config_defaults() {
        use crate::config_defaults::{auto_reset, food_persist, grid, warmup};
        let cfg = AppRuntimeConfig::default();
        assert_eq!(cfg.warmup_frames, warmup::DEFAULT_WARMUP_FRAMES);
        assert!(!cfg.skip_warmup);
        assert_eq!(
            cfg.warmup_brightness_multiplier,
            warmup::DEFAULT_BRIGHTNESS_MULTIPLIER
        );
        assert!(!cfg.auto_reset);
        assert_eq!(
            cfg.auto_reset_entropy_threshold,
            auto_reset::DEFAULT_ENTROPY_THRESHOLD
        );
        assert_eq!(
            cfg.auto_reset_duration_frames,
            auto_reset::DEFAULT_DURATION_FRAMES
        );
        assert!(!cfg.grid);
        assert_eq!(cfg.grid_style, GridStyle::Cross);
        assert_eq!(cfg.grid_size, grid::DEFAULT_GRID_SIZE);
        assert_eq!(cfg.grid_opacity, grid::DEFAULT_GRID_OPACITY);
        assert!(!cfg.grid_adaptive);
        assert_eq!(cfg.food_persist_strength, food_persist::DEFAULT_STRENGTH);
        assert_eq!(cfg.food_persist_radius, food_persist::DEFAULT_RADIUS);
        assert_eq!(cfg.food_persist_duration, food_persist::DEFAULT_DURATION);
    }
}
