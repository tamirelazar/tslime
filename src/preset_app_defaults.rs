//! Per-preset app-runtime defaults (third declarative seam). Resolved by
//! `ProfileOverrides::resolve_app` as the fallback between explicit override and
//! the global `AppRuntimeConfig::default()`.

use crate::simulation::config::Preset;

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct PresetAppDefaults {
    pub auto_reset: bool,
}

impl Default for PresetAppDefaults {
    fn default() -> Self {
        Self {
            auto_reset: crate::config_defaults::auto_reset::DEFAULT_AUTO_RESET,
        }
    }
}

impl From<Preset> for PresetAppDefaults {
    fn from(preset: Preset) -> Self {
        match preset {
            Preset::Constellation => Self { auto_reset: true },
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
}
