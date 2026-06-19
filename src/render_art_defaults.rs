//! Per-preset render/art-layer defaults.
//!
//! `RenderArtDefaults` is the render-layer counterpart to [`SimConfig`]: it is
//! resolved from the active [`Preset`] alongside the sim config, carrying render
//! parameters (currently `intensity_mapping`) that must not pollute the sim layer.
//! For now every preset uses the historical global default (log10); tasteful
//! per-preset values land in the showcase-presets issue (#36).

use crate::render::charset::Charset;
use crate::render::palette::{IntensityMapping, Palette, PaletteCycle, RgbColor, TemporalMode};
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
    /// Temporal-color strength (lever 3). Identity = 0.0 (off).
    pub temporal_color: f32,
    /// Temporal lag in frames (lever 3). Identity = 8.0.
    pub temporal_lag_frames: f32,
    /// Temporal color mode (lever 3). Identity = Hue.
    pub temporal_mode: TemporalMode,
    /// Hand-picked front accent (Accent mode). None = derive from palette hot-end.
    pub temporal_accent: Option<RgbColor>,
    /// Per-preset default palette. None = use the global/CLI palette.
    pub palette: Option<Palette>,
    /// Per-preset default charset. None = use the global/CLI charset.
    pub charset: Option<Charset>,
}

impl Default for RenderArtDefaults {
    fn default() -> Self {
        Self {
            intensity_mapping: IntensityMapping::logarithmic(10.0),
            palette_cycle: PaletteCycle::default(),
            glyph: crate::render::charset::GlyphConfig::default(),
            temporal_color: 0.0,
            temporal_lag_frames: 8.0,
            temporal_mode: TemporalMode::Hue,
            temporal_accent: None,
            palette: None,
            charset: None,
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

    const ALL_PRESETS_FOR_TEST: [Preset; 30] = [
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
    ];

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

    #[test]
    fn default_temporal_palette_charset_are_identity() {
        let d = RenderArtDefaults::default();
        assert_eq!(d.temporal_color, 0.0);
        assert_eq!(d.temporal_lag_frames, 8.0);
        assert_eq!(d.temporal_mode, crate::render::palette::TemporalMode::Hue);
        assert_eq!(d.temporal_accent, None);
        assert_eq!(d.palette, None);
        assert_eq!(d.charset, None);
    }

    #[test]
    fn every_preset_temporal_is_off_identity() {
        for preset in ALL_PRESETS_FOR_TEST {
            let d = RenderArtDefaults::from(preset);
            assert_eq!(d.temporal_color, 0.0, "{preset:?} temporal must be off");
            assert_eq!(d.palette, None, "{preset:?} palette must be identity");
            assert_eq!(d.charset, None, "{preset:?} charset must be identity");
        }
    }
}
