//! Per-preset app-runtime defaults (third declarative seam). Resolved by
//! `ProfileOverrides::resolve_app` as the fallback between explicit override and
//! the global `AppRuntimeConfig::default()`.

use crate::simulation::config::Preset;

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct PresetAppDefaults {
    pub auto_reset: bool,
    /// Entropy-collapse threshold fallback when no CLI `--collapse-threshold` is
    /// given. `0.0` disables the entropy detector (entropy is always `>= 0`, so
    /// `entropy < 0.0` never fires), leaving only the stagnation detector.
    pub entropy_threshold: f32,
}

impl Default for PresetAppDefaults {
    fn default() -> Self {
        Self {
            auto_reset: crate::config_defaults::auto_reset::DEFAULT_AUTO_RESET,
            entropy_threshold: crate::config_defaults::auto_reset::DEFAULT_ENTROPY_THRESHOLD,
        }
    }
}

impl From<Preset> for PresetAppDefaults {
    fn from(preset: Preset) -> Self {
        match preset {
            // Constellation re-rolls a fresh layout on collapse. The entropy
            // detector is too eager for it (low brightness-value diversity in
            // coherent rotating/blob patterns reads as "dead" while they are
            // still alive), so default it off and rely on stagnation alone.
            Preset::Constellation => Self {
                auto_reset: true,
                entropy_threshold: 0.0,
            },
            // ConstellationStatic holds its figure indefinitely via the
            // per-frame template re-stamp, so it must NOT auto-reset — a reset
            // would drop the held figure. Entropy threshold is moot while
            // auto_reset is false; set 0.0 to keep the detector off explicitly.
            Preset::ConstellationStatic => Self {
                auto_reset: false,
                entropy_threshold: 0.0,
            },
            _ => Self::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn default_mirrors_global() {
        assert_eq!(
            PresetAppDefaults::default().auto_reset,
            crate::config_defaults::auto_reset::DEFAULT_AUTO_RESET
        );
    }
    #[test]
    fn constellation_opts_in() {
        assert!(PresetAppDefaults::from(Preset::Constellation).auto_reset);
        assert_eq!(
            PresetAppDefaults::from(Preset::Network).auto_reset,
            crate::config_defaults::auto_reset::DEFAULT_AUTO_RESET
        );
    }

    #[test]
    fn constellation_disables_entropy_detector() {
        // 0.0 turns the entropy detector off (entropy is always >= 0).
        assert_eq!(
            PresetAppDefaults::from(Preset::Constellation).entropy_threshold,
            0.0
        );
        // Other presets keep the global default.
        assert_eq!(
            PresetAppDefaults::from(Preset::Network).entropy_threshold,
            crate::config_defaults::auto_reset::DEFAULT_ENTROPY_THRESHOLD
        );
    }

    #[test]
    fn constellation_static_holds_no_auto_reset() {
        let d = PresetAppDefaults::from(Preset::ConstellationStatic);
        assert!(
            !d.auto_reset,
            "ConstellationStatic must not auto-reset (it holds the figure)"
        );
        assert_eq!(d.entropy_threshold, 0.0);
    }
}
