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
    /// Per-preset color anti-aliasing strength. None = use the global/CLI/auto value.
    pub color_aa: Option<crate::render::antialiasing::AaStrength>,
    /// Per-preset animated hue-shift in degrees/second. Identity = 0.0 (off).
    pub hue_shift: f32,
    /// Afterglow strength (lever 7). Identity = 0.0 (off).
    pub afterglow: f32,
    /// Afterglow EMA rate. Identity = 0.05.
    pub afterglow_rate: f32,
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
            color_aa: None,
            hue_shift: 0.0,
            afterglow: 0.0,
            afterglow_rate: 0.05,
        }
    }
}

/// Every render lever fully resolved to a concrete value (CLI ⊕ preset ⊕ default).
/// `RenderArtDefaults` is the per-preset Option spec; this is the merged result
/// used by startup, live preset-switch, and reset identically.
#[allow(dead_code)] // wired in Task 13
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct ResolvedRenderConfig {
    pub palette: crate::cli::Palette,
    pub charset: crate::render::charset::Charset,
    pub color_aa: crate::render::antialiasing::AaStrength,
    pub hue_shift: f32,
    pub intensity_mapping: crate::render::palette::IntensityMapping,
    pub palette_cycle: crate::render::palette::PaletteCycle,
    pub glyph: crate::render::charset::GlyphConfig,
    pub temporal_color: f32,
    pub temporal_lag_frames: f32,
    pub temporal_mode: crate::render::palette::TemporalMode,
    pub temporal_accent: Option<crate::render::palette::RgbColor>,
    pub afterglow: f32,
    pub afterglow_rate: f32,
}

impl From<Preset> for RenderArtDefaults {
    /// Per-preset render defaults. The showcase presets (the original four plus
    /// the completion-pass set) carry art-on payloads; the 30 original presets
    /// fall through to `Self::default()` for byte-identical output.
    fn from(preset: Preset) -> Self {
        use crate::render::antialiasing::AaStrength;
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
                afterglow: 0.3,
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
                afterglow: 0.2,
                ..Self::default()
            },
            // Temporal Hue mode: front recolors by motion direction (not Accent).
            Preset::Drift => Self {
                temporal_color: 0.45,
                temporal_lag_frames: 10.0,
                temporal_mode: TemporalMode::Hue,
                palette: Some(Palette::Vibrant),
                afterglow: 0.1,
                ..Self::default()
            },
            // Points charset: sparse particle star-map.
            Preset::Constellation => Self {
                charset: Some(Charset::Points),
                palette: Some(Palette::Cosmic),
                ..Self::default()
            },
            // Quantize mapping + Wrap palette cycling: posterized bands.
            Preset::Mosaic => Self {
                intensity_mapping: IntensityMapping::quantize(6),
                palette_cycle: PaletteCycle {
                    cycles: 3,
                    mode: PaletteCycleMode::Wrap,
                },
                palette: Some(Palette::Amber),
                ..Self::default()
            },
            // Perlin mapping: organic noise-veined stone.
            Preset::Marble => Self {
                intensity_mapping: IntensityMapping::perlin(0.5, 3.0, 1),
                palette: Some(Palette::Slate),
                ..Self::default()
            },
            // HalfBlockDual charset + sqrt curve + Strong color-AA: max color resolution.
            Preset::Prism => Self {
                charset: Some(Charset::HalfBlockDual),
                intensity_mapping: IntensityMapping::power(0.5),
                palette: Some(Palette::Pastel),
                color_aa: Some(AaStrength::Strong),
                ..Self::default()
            },
            // Shade charset: smooth parchment density.
            Preset::Vellum => Self {
                charset: Some(Charset::Shade),
                palette: Some(Palette::Ink),
                ..Self::default()
            },
            // Exponential mapping: lifts darks for molten body.
            Preset::Forge => Self {
                intensity_mapping: IntensityMapping::exponential(4.0),
                palette: Some(Palette::Heat),
                afterglow: 0.3,
                ..Self::default()
            },
            // Sim-driven (decay-gamma + Pow deposit); Copper palette for oxidized fade.
            Preset::Wane => Self {
                palette: Some(Palette::Copper),
                ..Self::default()
            },
            // Braille + brightness glyphs + Power mapping + Subtle color-AA: delicate threads.
            Preset::Gossamer => Self {
                charset: Some(Charset::Braille),
                glyph: GlyphConfig {
                    selection: Some(GlyphSelection::Brightness),
                    ..GlyphConfig::default()
                },
                intensity_mapping: IntensityMapping::power(1.6),
                palette: Some(Palette::Ethereal),
                color_aa: Some(AaStrength::Subtle),
                afterglow: 0.2,
                ..Self::default()
            },
            // Custom ASCII charset + Sigmoid contrast: typographic engraving.
            Preset::Codex => Self {
                charset: Some(Charset::CustomAscii(vec![
                    '.', ':', '-', '=', '+', '*', '#', '%', '@',
                ])),
                intensity_mapping: IntensityMapping::sigmoid(8.0),
                palette: Some(Palette::Ink),
                ..Self::default()
            },
            // Animated hue-shift over time: living water.
            Preset::Tide => Self {
                palette: Some(Palette::Ocean),
                hue_shift: 8.0,
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

    /// The surviving "plain" presets that carry no art-on defaults (identity).
    /// Showcase presets (Lumen/Etching + the completion-pass set) are excluded:
    /// they carry art-on payloads, not identity.
    const EXISTING_PRESETS: [Preset; 16] = [
        Preset::Network,
        Preset::Exploratory,
        Preset::Tendrils,
        Preset::Organic,
        Preset::Fire,
        Preset::River,
        Preset::PetriDish,
        Preset::Vortex,
        Preset::Lightning,
        Preset::ChaosEdge,
        Preset::Blob,
        Preset::Pulse,
        Preset::Flocking,
        Preset::Ripple,
        Preset::Vortex36,
        Preset::DynamicTendrils,
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
        for p in [Preset::Lumen, Preset::Etching] {
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

        // Mosaic showcases palette cycling.
        let mosaic = RenderArtDefaults::from(Preset::Mosaic);
        assert!(mosaic.palette_cycle.cycles >= 2);

        // Existing presets stay identity.
        for p in [
            Preset::Organic,
            Preset::Network,
            Preset::Fire,
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
        // Back-compat: plain presets are all identity (#33).
        // (Mosaic uses cycles=3; it is not in this list.)
        for preset in [
            Preset::Network,
            Preset::Organic,
            Preset::Fire,
            Preset::Ripple,
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

    #[test]
    fn afterglow_default_is_off() {
        let d = RenderArtDefaults::default();
        assert_eq!(d.afterglow, 0.0);
        assert_eq!(d.afterglow_rate, 0.05);
    }

    #[test]
    fn afterglow_presets_glow() {
        assert_eq!(RenderArtDefaults::from(Preset::Lumen).afterglow, 0.3);
        assert_eq!(RenderArtDefaults::from(Preset::Etching).afterglow, 0.2);
        assert_eq!(RenderArtDefaults::from(Preset::Drift).afterglow, 0.1);
        assert_eq!(RenderArtDefaults::from(Preset::Forge).afterglow, 0.3);
        assert_eq!(RenderArtDefaults::from(Preset::Gossamer).afterglow, 0.2);
    }
}
