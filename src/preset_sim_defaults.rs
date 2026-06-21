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
    fn from(preset: Preset) -> Self {
        match preset {
            // Wrap keeps flow continuous across edges — River would otherwise
            // pile up on wall contact, and Ripple's rings need to re-enter.
            Preset::River | Preset::Ripple => Self {
                boundary_mode: Some(BoundaryMode::Wrap),
            },
            // Every other preset has no opinion; the base/default (Bounce) stands.
            _ => Self::default(),
        }
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
    fn only_river_and_ripple_declare_boundary_wrap() {
        use crate::simulation::config::Preset;
        for spec in crate::simulation::config::PRESETS {
            let expected = match spec.preset {
                Preset::River | Preset::Ripple => Some(BoundaryMode::Wrap),
                _ => None,
            };
            assert_eq!(
                PresetSimDefaults::from(spec.preset).boundary_mode,
                expected,
                "{} declared an unexpected boundary mode",
                spec.name
            );
        }
    }
}
