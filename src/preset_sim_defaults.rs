//! Per-preset optional sim-layer overrides.
//!
//! `PresetSimDefaults` is the sim-layer counterpart to
//! [`crate::render_art_defaults::RenderArtDefaults`]: a struct of `Option<T>`
//! *simulation* levers resolved from the active [`Preset`] and layered into
//! [`SimConfig`] by [`crate::config_builder::ConfigBuilder::assemble`].
//!
//! It gives sim levers the same "preset suggests, CLI overrides, default fills"
//! precedence that render levers already have, without merging the sim and
//! render config structs. Unlike `Preset::apply()` (imperative mutation that
//! always wins for whatever fields it touches), a `None` here means "the preset
//! does not care — keep the base/default value," and a `Some(_)` is still
//! overridable by an explicit CLI flag because the CLI override is applied
//! afterward.
//!
//! Currently carries only `boundary_mode`; the `apply()`-collapse refactor
//! migrates further sim levers into this struct.

use crate::simulation::config::{BoundaryMode, Preset};

/// Optional sim-layer levers a [`Preset`] may declare. `None` = the preset has
/// no opinion; the base/default value stands (and any CLI flag still wins).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub(crate) struct PresetSimDefaults {
    /// Per-preset boundary handling. `None` = use the global/CLI value (Bounce).
    pub boundary_mode: Option<BoundaryMode>,
}

impl From<Preset> for PresetSimDefaults {
    fn from(_preset: Preset) -> Self {
        // No preset declares a sim override yet; presets opt in here as the
        // capability is adopted (e.g. boundary-mode wrap for River/Ripple).
        Self::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_declares_nothing() {
        assert!(PresetSimDefaults::default().boundary_mode.is_none());
    }

    #[test]
    fn every_preset_keeps_default_boundary_for_now() {
        // Commit A is behaviour-preserving: no preset declares a boundary mode
        // yet, so each resolves to None and inherits the global default.
        for spec in crate::simulation::config::PRESETS {
            assert_eq!(
                PresetSimDefaults::from(spec.preset).boundary_mode,
                None,
                "{} should not declare a boundary mode yet",
                spec.name
            );
        }
    }
}
