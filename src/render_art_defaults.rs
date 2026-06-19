//! Per-preset render/art-layer defaults.
//!
//! `RenderArtDefaults` is the render-layer counterpart to [`SimConfig`]: it is
//! resolved from the active [`Preset`] alongside the sim config, carrying render
//! parameters (currently `intensity_mapping`) that must not pollute the sim layer.
//! For now every preset uses the historical global default (log10); tasteful
//! per-preset values land in the showcase-presets issue (#36).

use crate::render::palette::{IntensityMapping, PaletteCycle};
use crate::simulation::config::Preset;

/// Render-layer art defaults resolved per [`Preset`], emitted alongside
/// [`crate::simulation::config::SimConfig`]. Keeps render concerns out of the
/// sim layer (spec §5). Currently carries only `intensity_mapping`; later
/// render-layer levers extend this struct.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct RenderArtDefaults {
    /// Brightness→color tone curve. Default = global log10 (historical default).
    pub intensity_mapping: IntensityMapping,
    /// Spatial palette-repeat config (lever 6). Default = identity (cycles 1).
    pub palette_cycle: PaletteCycle,
    /// Glyph-selection strategy (lever 10). Default = identity (selection: None).
    pub glyph: crate::render::charset::GlyphConfig,
}

impl Default for RenderArtDefaults {
    fn default() -> Self {
        Self {
            intensity_mapping: IntensityMapping::logarithmic(10.0),
            palette_cycle: PaletteCycle::default(),
            glyph: crate::render::charset::GlyphConfig::default(),
        }
    }
}

impl From<Preset> for RenderArtDefaults {
    /// Per-preset render defaults. For #32 every preset uses the historical
    /// global log10 so output is byte-identical; tasteful per-preset values
    /// are deferred to the showcase-presets issue (#36).
    fn from(_preset: Preset) -> Self {
        Self::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::render::palette::IntensityMapping;
    use crate::simulation::config::Preset;

    #[test]
    fn default_is_log10() {
        assert_eq!(
            RenderArtDefaults::default().intensity_mapping,
            IntensityMapping::logarithmic(10.0)
        );
    }

    #[test]
    fn default_palette_cycle_is_identity() {
        let d = RenderArtDefaults::default();
        assert_eq!(
            d.palette_cycle,
            crate::render::palette::PaletteCycle::default()
        );
        assert_eq!(d.palette_cycle.cycles, 1);
    }

    #[test]
    fn every_preset_palette_cycle_is_identity() {
        // Back-compat: mechanism-only ship — every preset is identity (#33).
        for preset in [
            Preset::Network,
            Preset::Organic,
            Preset::Coral,
            Preset::Maze,
        ] {
            assert!(RenderArtDefaults::from(preset).palette_cycle.is_identity());
        }
    }

    #[test]
    fn default_glyph_is_identity() {
        assert_eq!(
            RenderArtDefaults::default().glyph,
            crate::render::charset::GlyphConfig::default()
        );
        assert_eq!(RenderArtDefaults::default().glyph.selection, None);
    }

    #[test]
    fn every_preset_glyph_is_identity() {
        // Back-compat: mechanism-only ship — every preset is identity (#34).
        for preset in [
            Preset::Network,
            Preset::Exploratory,
            Preset::Tendrils,
            Preset::Organic,
            Preset::Minimal,
            Preset::Moss,
            Preset::Cosmic,
            Preset::Fire,
            Preset::Zen,
            Preset::Storm,
            Preset::River,
            Preset::Ethereal,
            Preset::PetriDish,
            Preset::Vortex,
            Preset::Lightning,
            Preset::Crystal,
            Preset::ChaosEdge,
            Preset::Blob,
            Preset::Worm,
            Preset::Pulse,
            Preset::Coral,
            Preset::Flocking,
            Preset::Maze,
            Preset::Ripple,
            Preset::Vortex36,
            Preset::Chameleon,
            Preset::DynamicTendrils,
            Preset::MorphingCoral,
            Preset::ReactiveSwarm,
            Preset::DuelingModulators,
        ] {
            assert_eq!(
                RenderArtDefaults::from(preset).glyph.selection,
                None,
                "preset {preset:?} must default to identity glyph in #34"
            );
        }
    }

    #[test]
    fn every_preset_defaults_to_log10() {
        // Back-compat invariant for #32: payload is pure plumbing, so every
        // preset must resolve to today's global default. Per-preset tuning is #36.
        // NOTE: `Preset::ALL` does not exist in this codebase — list explicitly.
        let log10 = IntensityMapping::logarithmic(10.0);
        for preset in [
            Preset::Network,
            Preset::Exploratory,
            Preset::Tendrils,
            Preset::Organic,
            Preset::Minimal,
            Preset::Moss,
            Preset::Cosmic,
            Preset::Fire,
            Preset::Zen,
            Preset::Storm,
            Preset::River,
            Preset::Ethereal,
            Preset::PetriDish,
            Preset::Vortex,
            Preset::Lightning,
            Preset::Crystal,
            Preset::ChaosEdge,
            Preset::Blob,
            Preset::Worm,
            Preset::Pulse,
            Preset::Coral,
            Preset::Flocking,
            Preset::Maze,
            Preset::Ripple,
            Preset::Vortex36,
            Preset::Chameleon,
            Preset::DynamicTendrils,
            Preset::MorphingCoral,
            Preset::ReactiveSwarm,
            Preset::DuelingModulators,
        ] {
            assert_eq!(
                RenderArtDefaults::from(preset).intensity_mapping,
                log10,
                "preset {preset:?} must default to log10 in #32"
            );
        }
    }
}
