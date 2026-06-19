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
    /// Per-preset render defaults. The four showcase presets (#36) carry
    /// art-on payloads; all 30 existing presets fall through to `Self::default()`
    /// for byte-identical output.
    fn from(preset: Preset) -> Self {
        use crate::render::charset::{Charset, GlyphConfig, GlyphSelection};
        use crate::render::palette::{
            IntensityMapping, Palette, PaletteCycle, PaletteCycleMode, TemporalMode,
        };
        // All showcase presets use Accent mode with `temporal_accent: None`, so
        // the growing front blends toward the palette's OWN vivid stop
        // (`palette_accent_color`, brightness 0.85) — keeping every rendered
        // color on the palette gradient (no foreign hand-picked hues, no
        // off-palette hue rotation).
        match preset {
            Preset::Lumen => Self {
                temporal_color: 0.5,
                temporal_lag_frames: 8.0,
                temporal_mode: TemporalMode::Accent,
                palette: Some(Palette::Slime),
                intensity_mapping: IntensityMapping::smoothstep(),
                ..Self::default()
            },
            Preset::Aurora => Self {
                temporal_color: 0.4,
                temporal_lag_frames: 12.0,
                temporal_mode: TemporalMode::Accent,
                palette: Some(Palette::Ocean),
                intensity_mapping: IntensityMapping::smoothstep(),
                ..Self::default()
            },
            Preset::Bloom => Self {
                palette: Some(Palette::Warm),
                palette_cycle: PaletteCycle {
                    cycles: 2,
                    mode: PaletteCycleMode::Mirror,
                },
                temporal_color: 0.2,
                temporal_mode: TemporalMode::Accent,
                ..Self::default()
            },
            Preset::Etching => Self {
                temporal_color: 0.4,
                temporal_mode: TemporalMode::Accent,
                palette: Some(Palette::Neon),
                charset: Some(Charset::Braille),
                glyph: GlyphConfig {
                    selection: Some(GlyphSelection::Hybrid),
                    edge_threshold: 0.08,
                },
                ..Self::default()
            },
            _ => Self::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::render::palette::IntensityMapping;
    use crate::simulation::config::Preset;

    /// The 30 original presets. The 4 showcase presets (Lumen/Aurora/Bloom/Etching)
    /// are intentionally excluded: they carry art-on defaults, not identity.
    const EXISTING_PRESETS: [Preset; 30] = [
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
    fn showcase_presets_have_art_on() {
        let lumen = RenderArtDefaults::from(Preset::Lumen);
        assert!(lumen.temporal_color > 0.0);
        assert_eq!(
            lumen.temporal_mode,
            crate::render::palette::TemporalMode::Accent
        );
        // Showcase presets blend toward their OWN palette's vivid stop, so the
        // accent is intentionally None (palette-derived, not a foreign hue).
        assert_eq!(lumen.temporal_accent, None);
        assert_eq!(lumen.palette, Some(crate::render::palette::Palette::Slime));

        // All showcase presets use palette-coherent Accent mode (no hue-mode
        // drift off the gradient).
        for p in [
            Preset::Lumen,
            Preset::Aurora,
            Preset::Bloom,
            Preset::Etching,
        ] {
            let d = RenderArtDefaults::from(p);
            assert_eq!(
                d.temporal_mode,
                crate::render::palette::TemporalMode::Accent,
                "{p:?} must use Accent mode"
            );
            assert_eq!(
                d.temporal_accent, None,
                "{p:?} accent must be palette-derived"
            );
        }

        let etching = RenderArtDefaults::from(Preset::Etching);
        assert_eq!(
            etching.charset,
            Some(crate::render::charset::Charset::Braille)
        );
        assert!(etching.glyph.selection.is_some());

        let bloom = RenderArtDefaults::from(Preset::Bloom);
        assert!(bloom.palette_cycle.cycles >= 2);

        // Existing presets stay identity.
        for p in [
            Preset::Organic,
            Preset::Network,
            Preset::Coral,
            Preset::Tendrils,
        ] {
            let d = RenderArtDefaults::from(p);
            assert_eq!(d.temporal_color, 0.0);
            assert_eq!(d.palette, None);
            assert_eq!(d.charset, None);
        }
    }

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
        // Back-compat: the 30 original presets are all identity (#33).
        // (Bloom uses cycles=2; it is not in this list.)
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
        // Back-compat: all 30 original presets use identity glyph (#34).
        // (Etching uses Hybrid; it is not in EXISTING_PRESETS.)
        for preset in EXISTING_PRESETS {
            assert_eq!(
                RenderArtDefaults::from(preset).glyph.selection,
                None,
                "preset {preset:?} must default to identity glyph in #34"
            );
        }
    }

    #[test]
    fn every_preset_defaults_to_log10() {
        // Back-compat invariant for #32: the 30 original presets must all
        // resolve to the historical global log10. The showcase presets (#36)
        // may override intensity_mapping and are excluded here.
        let log10 = IntensityMapping::logarithmic(10.0);
        for preset in EXISTING_PRESETS {
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
        // The 30 original presets must all have temporal off (#35 back-compat).
        for preset in EXISTING_PRESETS {
            let d = RenderArtDefaults::from(preset);
            assert_eq!(d.temporal_color, 0.0, "{preset:?} temporal must be off");
            assert_eq!(d.palette, None, "{preset:?} palette must be identity");
            assert_eq!(d.charset, None, "{preset:?} charset must be identity");
        }
    }
}
