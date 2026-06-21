//! `Profile` — the shared basis for presets and saved configs.
//!
//! A preset and a saved config both answer "what is the complete tunable lever
//! set?". `Profile` is that answer as one concrete value: the resolved sim
//! config, the resolved render config, the app runtime config, and the (optional)
//! seed. `ProfileSource` records where the active profile came from. See `CONTEXT.md`.

use crate::app_config::AppRuntimeConfig;
use crate::cli::Args;
use crate::render_art_defaults::ResolvedRenderConfig;
use crate::simulation::config::{Preset, SimConfig};

/// The complete resolved lever set (sim ⊕ render ⊕ app ⊕ seed). The shared basis for
/// presets and saved configs.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Profile {
    pub sim: SimConfig,
    pub render: ResolvedRenderConfig,
    /// Restart-only app-level levers (warmup, auto-reset, grid, food-persist).
    pub app: AppRuntimeConfig,
    /// `None` = fresh random each run. `Some` only when a seed was pinned
    /// explicitly (a CLI `--seed`); presets never pin a seed.
    pub seed: Option<u64>,
}

/// Where the active `Profile` came from.
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum ProfileSource {
    /// The profile authored by the launch CLI invocation.
    StartupCli,
    /// A built-in preset selected at runtime.
    Preset(Preset),
    /// A named saved config loaded from disk.
    #[allow(dead_code)]
    SavedConfig(String),
}

impl Profile {
    /// Resolve the launch profile from CLI args: preset base ⊕ CLI overrides for
    /// sim, preset render defaults ⊕ CLI overrides for render, plus the explicit
    /// seed if one was passed.
    pub(crate) fn resolve_from_args(args: &Args) -> Result<Self, String> {
        crate::profile_overrides::ProfileOverrides::from_args(args).and_then(|o| o.resolve())
    }

    /// Resolve a bare preset profile (no CLI overlay). The authoritative definition
    /// of "what this preset looks like" — used for preset-switch and for unit tests.
    #[allow(dead_code)]
    pub(crate) fn from_preset(preset: Preset) -> Result<Self, String> {
        crate::profile_overrides::ProfileOverrides {
            preset: Some(preset),
            ..Default::default()
        }
        .resolve()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// from_preset for the same preset twice is deterministic / equal (dirty basis).
    #[test]
    fn from_preset_is_equal_for_same_preset() {
        let a = Profile::from_preset(Preset::Lumen).expect("a");
        let b = Profile::from_preset(Preset::Lumen).expect("b");
        assert_eq!(a, b);
    }

    /// Different presets resolve to different profiles (sanity: PartialEq discriminates).
    #[test]
    fn from_preset_differs_across_presets() {
        let lumen = Profile::from_preset(Preset::Lumen).expect("lumen");
        let network = Profile::from_preset(Preset::Network).expect("network");
        assert_ne!(lumen, network);
    }

    /// from_preset resolves to the BARE preset defaults (no CLI overlay).
    #[test]
    fn from_preset_is_preset_only() {
        let organic = Profile::from_preset(Preset::Organic).expect("organic");
        let expected = crate::profile_overrides::ProfileOverrides {
            preset: Some(Preset::Organic),
            ..Default::default()
        }
        .resolve()
        .expect("expected");
        assert_eq!(organic, expected);
    }

    /// Seed is None unless --seed was explicitly passed.
    #[test]
    fn seed_is_none_without_cli_flag() {
        let args = Args::default();
        assert_eq!(
            Profile::resolve_from_args(&args).expect("resolve").seed,
            None
        );
    }

    /// Out-of-range CLI args must be rejected by resolve_from_args (validation parity with
    /// the old `SimConfig::try_from(&Args)` path). sensor_angle > MAX (90.0) is a clear
    /// out-of-range value that validate() rejects.
    #[test]
    fn resolve_from_args_rejects_invalid_sensor_angle() {
        // sensor_angle=999.0 is far above MAX_SENSOR_ANGLE (90.0)
        let args = Args {
            sensor_angle: Some(999.0),
            ..Args::default()
        };
        assert!(
            Profile::resolve_from_args(&args).is_err(),
            "expected Err for sensor_angle=999.0 but got Ok"
        );
    }
}
