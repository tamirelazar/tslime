//! `Profile` — the shared basis for presets and saved configs.
//!
//! A preset and a saved config both answer "what is the complete tunable lever
//! set?". `Profile` is that answer as one concrete value: the resolved sim
//! config, the resolved render config, and the (optional) seed. `ProfileSource`
//! records where the active profile came from. See `CONTEXT.md`.

use crate::cli::Args;
use crate::render_art_defaults::ResolvedRenderConfig;
use crate::simulation::config::{Preset, SimConfig};

/// The complete resolved lever set (sim ⊕ render ⊕ seed). The shared basis for
/// presets and saved configs.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Profile {
    pub sim: SimConfig,
    pub render: ResolvedRenderConfig,
    /// `None` = fresh random each run. `Some` only when a seed was pinned
    /// explicitly (a CLI `--seed`); presets never pin a seed.
    pub seed: Option<u64>,
}

/// Where the active `Profile` came from.
#[derive(Clone, Debug, PartialEq)]
#[allow(dead_code)]
pub(crate) enum ProfileSource {
    /// The profile authored by the launch CLI invocation.
    StartupCli,
    /// A built-in preset selected at runtime.
    Preset(Preset),
    /// A named saved config loaded from disk.
    SavedConfig(String),
}

impl Profile {
    /// Resolve the launch profile from CLI args: preset base ⊕ CLI overrides for
    /// sim, preset render defaults ⊕ CLI overrides for render, plus the explicit
    /// seed if one was passed.
    #[allow(dead_code)]
    pub(crate) fn resolve_from_args(args: &Args) -> Result<Self, String> {
        let sim = crate::config_builder::ConfigBuilder::from_args(args)
            .assemble()
            .map_err(|e| e.to_string())?;
        let render = args.resolve_render_config()?;
        Ok(Self {
            sim,
            render,
            seed: args.seed,
        })
    }

    /// Resolve a profile for switching to `preset` at runtime, preserving the
    /// launch CLI overlay (current startup semantics — Phase A keeps parity;
    /// Phase C changes precedence to launch-only).
    #[allow(dead_code)]
    pub(crate) fn from_preset(preset: Preset, args: &Args) -> Result<Self, String> {
        let mut a = args.clone();
        a.preset = Some(preset);
        Self::resolve_from_args(&a)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Profile.sim equals the standalone ConfigBuilder result for the same args.
    #[test]
    fn resolve_from_args_sim_matches_config_builder() {
        let args = Args::default();
        let profile = Profile::resolve_from_args(&args).expect("resolve");
        let direct = crate::config_builder::ConfigBuilder::from_args(&args)
            .assemble()
            .expect("assemble");
        assert_eq!(profile.sim, direct);
    }

    /// Profile.render equals the standalone resolve_render_config for the same args.
    #[test]
    fn resolve_from_args_render_matches_cli() {
        let args = Args::default();
        let profile = Profile::resolve_from_args(&args).expect("resolve");
        let direct = args.resolve_render_config().expect("render");
        assert_eq!(profile.render, direct);
    }

    /// from_preset for the same preset twice is deterministic / equal (dirty basis).
    #[test]
    fn from_preset_is_equal_for_same_preset() {
        let args = Args::default();
        let a = Profile::from_preset(Preset::Lumen, &args).expect("a");
        let b = Profile::from_preset(Preset::Lumen, &args).expect("b");
        assert_eq!(a, b);
    }

    /// Different presets resolve to different profiles (sanity: PartialEq discriminates).
    #[test]
    fn from_preset_differs_across_presets() {
        let args = Args::default();
        let lumen = Profile::from_preset(Preset::Lumen, &args).expect("lumen");
        let network = Profile::from_preset(Preset::Network, &args).expect("network");
        assert_ne!(lumen, network);
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
}
