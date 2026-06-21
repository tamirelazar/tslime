//! `ProfileOverrides` — the single all-`Option` authored partial.
//!
//! A preset, a CLI invocation, and a saved TOML each produce a
//! `ProfileOverrides`; `resolve()` turns one into a [`Profile`]. It unifies the
//! sim-side partial (formerly `ConfigBuilder`) and the render-side partial
//! (formerly the ad-hoc extraction in `Args::resolve_render_config`). Field
//! shapes are CLI-ish (e.g. `brightness` is a user gain, `terrain` a string)
//! so `resolve()` can be a verbatim port of the old two-call resolution. See
//! `CONTEXT.md`.

use crate::cli::{Args, AttractorArg, ObstacleArg, SpeciesArg, WindArg};
use crate::profile::Profile;
use crate::render::antialiasing::AaStrength;
use crate::render::charset::{Charset, GlyphSelection};
use crate::render::palette::{IntensityMapping, Palette, PaletteCycle, RgbColor, TemporalMode};
use crate::render_art_defaults::ResolvedRenderConfig;
use crate::simulation::config::{
    Aspect, BoundaryMode, ChromeStyle, DepositCurve, DiffusionKernel, Preset, SimConfig,
    TerminalSizeThreshold, WindowFrame, WindowPadding,
};
use serde::{Deserialize, Serialize};

/// The single all-`Option` authored partial (sim ⊕ render ⊕ seed). Sim fields
/// mirror the former `ConfigBuilder`; render fields mirror what
/// `Args::resolve_render_config` reads.
#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct ProfileOverrides {
    // ── provenance / base selector ──
    pub preset: Option<Preset>,
    pub seed: Option<u64>,

    // ── sim levers (the sim-side authored partial) ──
    pub sensor_angle: Option<f32>,
    pub sensor_distance: Option<f32>,
    pub rotation_angle: Option<f32>,
    pub step_size: Option<f32>,
    pub decay_factor: Option<f32>,
    pub deposit_amount: Option<f32>,
    /// User-facing brightness GAIN (converted to white-point in resolve, as today).
    pub brightness: Option<f32>,
    pub diffusion_kernel: Option<DiffusionKernel>,
    pub diffusion_sigma: Option<f32>,
    pub time_scale: Option<f32>,
    pub population: Option<usize>,
    pub fps: Option<usize>,
    pub food_image_path: Option<String>,
    pub food_image_invert: Option<bool>,
    pub food_image_scale: Option<f32>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub attractors: Vec<AttractorArg>,
    pub attractor_strength: Option<f32>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub obstacles: Vec<ObstacleArg>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub species: Vec<SpeciesArg>,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub separate_species_trails: bool,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub species_colors: bool,
    pub use_simd: Option<bool>,
    pub wind: Option<WindArg>,
    pub terrain: Option<String>,
    pub terrain_strength: Option<f32>,
    pub background_color: Option<String>,
    pub boundary_mode: Option<BoundaryMode>,
    pub window_frame: Option<WindowFrame>,
    pub chrome_style: Option<ChromeStyle>,
    pub aspect: Option<Aspect>,
    pub window_padding: Option<WindowPadding>,
    pub show_status_bar: Option<bool>,
    pub min_sim_size: Option<TerminalSizeThreshold>,
    pub min_frame_size: Option<TerminalSizeThreshold>,
    pub respawn_interval: Option<u32>,
    pub decay_gamma: Option<f32>,
    pub diffuse_weight: Option<f32>,
    pub deposit_curve: Option<DepositCurve>,
    pub deposit_scale: Option<f32>,
    pub deposit_gamma: Option<f32>,
    pub deposit_cap: Option<f32>,

    // ── render levers (mirror src/cli.rs resolve_render_config + to_render_art_defaults) ──
    /// `Some` only when the CLI palette was explicitly set (see `palette_explicitly_set`).
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "serde_opt_palette"
    )]
    pub palette: Option<Palette>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "serde_opt_charset"
    )]
    pub charset: Option<Charset>,
    pub color_aa: Option<AaStrength>,
    /// CLI `--palette-shift` (maps to hue_shift). `None` falls through to preset art.
    pub hue_shift: Option<f32>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "serde_opt_intensity_mapping"
    )]
    pub intensity_mapping: Option<IntensityMapping>,
    pub palette_cycle: Option<PaletteCycle>,
    /// `Some` only when `--glyph-selection` was explicitly provided on the CLI.
    pub glyph_selection: Option<GlyphSelection>,
    /// `Some` only when `--glyph-edge-threshold` was explicitly provided on the CLI.
    pub glyph_edge_threshold: Option<f32>,
    pub temporal_color: Option<f32>,
    pub temporal_lag_frames: Option<f32>,
    pub temporal_mode: Option<TemporalMode>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "serde_opt_rgb_hex"
    )]
    pub temporal_accent: Option<RgbColor>,
    pub afterglow: Option<f32>,
    pub afterglow_rate: Option<f32>,

    // ── apply/persistence-only levers (NOT consumed by resolve_sim/resolve_render) ──
    /// Saved palette-reverse flag. Only written by `apply_to_runtime_state`; ignored by
    /// `resolve()`. `from_args` leaves this `None`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reverse_palette: Option<bool>,
    /// Saved palette-invert flag. Only written by `apply_to_runtime_state`; ignored by
    /// `resolve()`. `from_args` leaves this `None`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub invert_palette: Option<bool>,
    /// Saved food-persist flag. Only written by `apply_to_runtime_state`; ignored by
    /// `resolve()`. `from_args` leaves this `None`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub food_persist: Option<bool>,
    /// Apply/persist-only: full per-charset AA array for config save/load. NOT consumed by
    /// `resolve()` (the resolve-side `color_aa` single scalar is the CLI lever). When present,
    /// `apply_to_runtime_state` restores all slots from this Vec; the scalar `color_aa` is used
    /// as a fallback for the active charset only when this field is absent (back-compat).
    /// Phase C may unify these.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub color_aa_all: Option<Vec<AaStrength>>,
}

impl ProfileOverrides {
    /// Builds a `ProfileOverrides` from CLI args. Sim block mirrors the sim-field
    /// extraction that the former `ConfigBuilder::from_args` performed. Render block
    /// mirrors the predicates from `Args::resolve_render_config` /
    /// `to_render_art_defaults`.
    ///
    /// Returns `Err` on the same parse failures as the oracle's `resolve_render_config`:
    /// invalid `--glyph-selection`, `--palette`, `--intensity-mapping`,
    /// `--palette-cycle-mode`, or `--temporal-accent`.
    pub(crate) fn from_args(args: &Args) -> Result<Self, String> {
        // Render: glyph — store the raw CLI-flag values separately so resolve_render
        // can apply them onto the PRESET's art.glyph exactly as glyph_config_parsed
        // does (src/cli.rs:2006-2018). Pre-parsing against GlyphConfig::default() would
        // make an explicit --glyph-edge-threshold 0.15 (== the default) indistinguishable
        // from "not set", silently dropping the override on presets with a different
        // default edge_threshold (e.g. Etching). Clamp edge_threshold here to mirror
        // the clamp inside glyph_config_parsed.
        let glyph_selection = args
            .glyph_selection
            .as_ref()
            .map(|s| s.parse::<GlyphSelection>())
            .transpose()?;
        let glyph_edge_threshold = args.glyph_edge_threshold.map(|t| t.clamp(0.0, 2.0));

        // Render: temporal_mode parsed from string, mirroring to_render_art_defaults.
        // Uses a catch-all `_ => Hue` — never errors.
        let temporal_mode =
            args.temporal_mode
                .as_ref()
                .map(|m| match m.to_ascii_lowercase().as_str() {
                    "accent" => TemporalMode::Accent,
                    _ => TemporalMode::Hue,
                });

        // Render: temporal_accent parsed from hex string. Oracle errors via
        // `map_err(|_| format!("invalid --temporal-accent hex: {hex}"))`.
        let temporal_accent = args
            .temporal_accent
            .as_ref()
            .map(|hex| {
                u32::from_str_radix(hex.trim_start_matches('#'), 16)
                    .map(RgbColor::from_hex)
                    .map_err(|_| format!("invalid --temporal-accent hex: {hex}"))
            })
            .transpose()?;

        // Render: palette — Some only when explicitly set on CLI. Oracle errors via
        // `self.palette().map_err(|e| e.to_string())?` in resolve_render_config.
        let palette = if args.palette_explicitly_set() {
            Some(args.palette()?)
        } else {
            None
        };

        // Render: palette_cycle — Some when either palette_cycles or palette_cycle_mode is set.
        // Oracle errors via `palette_cycle_mode_parsed()?` in to_render_art_defaults.
        let palette_cycle = if args.palette_cycles.is_some() || args.palette_cycle_mode.is_some() {
            // Build the PaletteCycle from CLI flags, same as to_render_art_defaults.
            let mut pc = PaletteCycle::default();
            if let Some(n) = args.palette_cycles {
                pc.cycles = n;
            }
            if let Some(mode) = args.palette_cycle_mode_parsed()? {
                pc.mode = mode;
            }
            Some(pc)
        } else {
            None
        };

        // Render: intensity_mapping — Some when explicitly set on CLI. Oracle errors via
        // `self.intensity_mapping()?` in to_render_art_defaults.
        let intensity_mapping = if args.intensity_mapping.is_some() {
            Some(args.intensity_mapping()?)
        } else {
            None
        };

        Ok(Self {
            // SIM block: sim-field extraction (mirrors the former from_args sim block).
            preset: args.preset,
            seed: args.seed,
            sensor_angle: args.sensor_angle,
            sensor_distance: args.sensor_distance,
            rotation_angle: args.rotation_angle,
            step_size: args.step_size,
            decay_factor: args.decay_factor,
            deposit_amount: args.deposit_amount,
            brightness: args.brightness,
            diffusion_kernel: args.diffusion_kernel,
            diffusion_sigma: args.diffusion_sigma,
            time_scale: Some(args.time_scale),
            population: args.population,
            fps: Some(args.fps),
            food_image_path: Some(args.food.clone()),
            food_image_invert: Some(args.food_invert),
            food_image_scale: Some(args.food_scale),
            attractors: args.attract.clone(),
            attractor_strength: Some(args.attractor_strength),
            obstacles: args.obstacle.clone(),
            species: args.species_list().to_vec(),
            separate_species_trails: args.separate_species_trails_enabled(),
            species_colors: args.species_colors_enabled(),
            use_simd: Some(!args.simd_off),
            wind: args.wind.clone(),
            terrain: Some(args.terrain.clone()),
            terrain_strength: Some(args.terrain_strength),
            background_color: args.bg_color.clone(),
            boundary_mode: args.boundary_mode,
            window_frame: args.window_frame,
            chrome_style: if args.fullscreen {
                Some(ChromeStyle::Fullscreen)
            } else {
                args.chrome_style
            },
            aspect: args.aspect,
            window_padding: args.window_padding,
            show_status_bar: if args.show_status_bar {
                Some(true)
            } else {
                None
            },
            min_sim_size: args.min_sim_size,
            min_frame_size: args.min_frame_size,
            respawn_interval: args.respawn_interval,
            decay_gamma: args.decay_gamma,
            diffuse_weight: args.diffuse_weight,
            deposit_curve: args.deposit_curve,
            deposit_scale: args.deposit_scale,
            deposit_gamma: args.deposit_gamma,
            deposit_cap: args.deposit_cap,

            // RENDER block: mirror src/cli.rs resolve_render_config (2129-2167) +
            // to_render_art_defaults (2064-2117).
            palette,
            charset: args.charset_parsed().ok().flatten(),
            color_aa: args.color_aa,
            hue_shift: args.palette_shift,
            intensity_mapping,
            palette_cycle,
            glyph_selection,
            glyph_edge_threshold,
            temporal_color: args.temporal_color,
            temporal_lag_frames: args.temporal_lag,
            temporal_mode,
            temporal_accent,
            afterglow: args.afterglow,
            afterglow_rate: args.afterglow_rate,

            // apply/persistence-only levers — from_args leaves these None (they are
            // not CLI-resolution levers; only capture_overrides / TOML serde sets them).
            reverse_palette: None,
            invert_palette: None,
            food_persist: None,
            color_aa_all: None,
        })
    }

    /// Resolve to a concrete `Profile`. Byte-identical to the legacy startup
    /// two-call path (`assemble` → `resolve_render_config`).
    pub(crate) fn resolve(&self) -> Result<Profile, String> {
        let sim = self.resolve_sim().map_err(|e| e.to_string())?;
        // Validation parity: Profile::resolve_from_args calls validate() after assemble().
        // Keep the exact same call: crate::validation::Validatable::validate(&sim).
        crate::validation::Validatable::validate(&sim).map_err(|e| e.to_string())?;
        let render = self.resolve_render()?;
        Ok(Profile {
            sim,
            render,
            seed: self.seed,
        })
    }

    /// Resolves the sim-side `ProfileOverrides` fields into a `SimConfig`.
    /// Same order and special cases as the former assemble path; only field
    /// access source changed (`self.<field>` instead of builder fields).
    fn resolve_sim(&self) -> Result<SimConfig, crate::error::ValidationError> {
        use crate::config_defaults::population;
        use crate::preset_sim_defaults::PresetSimDefaults;
        use crate::simulation::config::{Attractor, SpeciesConfig, TerrainType, Wind};

        // No validation here — caller validates the assembled config once.

        // Start with preset or default
        let mut config = if let Some(preset) = self.preset {
            SimConfig::from(preset)
        } else {
            SimConfig::default()
        };

        // Apply overrides
        if let Some(v) = self.sensor_angle {
            config.sensor_angle = v;
        }
        if let Some(v) = self.sensor_distance {
            config.sensor_distance = v;
        }
        if let Some(v) = self.rotation_angle {
            config.rotation_angle = v;
        }
        if let Some(v) = self.step_size {
            config.step_size = v;
        }
        if let Some(v) = self.decay_factor {
            config.decay_factor = v;
        }
        if let Some(gain) = self.brightness {
            // CLI exposes a user-facing brightness gain; the engine stores a
            // normalization white-point that it divides by. Convert here so the
            // internal representation stays a white-point.
            config.max_brightness = crate::config_defaults::trail::white_point_from_gain(gain);
        }
        if let Some(v) = self.deposit_amount {
            config.deposit_amount = v;
        }

        // Food image settings
        if let Some(ref path) = self.food_image_path {
            config.food_image_path = Some(path.clone());
        }
        if let Some(invert) = self.food_image_invert {
            config.food_image_invert = invert;
        }
        if let Some(scale) = self.food_image_scale {
            config.food_image_scale = scale;
        }

        // Diffusion settings
        if let Some(kernel) = self.diffusion_kernel {
            config.diffusion_kernel = kernel;
        }
        if let Some(sigma) = self.diffusion_sigma {
            config.diffusion_sigma = sigma;
        }

        // Decay gamma (override only when explicitly set; else keep preset/default)
        if let Some(g) = self.decay_gamma {
            config.decay_gamma = g;
        }

        // Diffuse weight (Lague blend) — override only when explicitly set
        if let Some(w) = self.diffuse_weight {
            config.diffuse_weight = w;
        }

        // Deposit curve knobs (override only when explicitly set)
        if let Some(c) = self.deposit_curve {
            config.deposit_curve = c;
        }
        if let Some(s) = self.deposit_scale {
            config.deposit_scale = s;
        }
        if let Some(g) = self.deposit_gamma {
            config.deposit_gamma = g;
        }
        if let Some(cap) = self.deposit_cap {
            config.deposit_cap = cap;
        }

        // Time scale
        if let Some(scale) = self.time_scale {
            config.time_scale = scale;
        }

        // High FPS optimization: use Gaussian with lower sigma for smoother diffusion
        if let Some(fps) = self.fps {
            if fps >= 60 && self.diffusion_kernel.is_none() && self.diffusion_sigma.is_none() {
                config.diffusion_kernel = DiffusionKernel::Gaussian;
                config.diffusion_sigma = 0.5;
            }
        }

        // Attractors: CLI overrides; absent --attract, the preset's survive.
        if !self.attractors.is_empty() {
            config.attractors = self
                .attractors
                .iter()
                .map(|a| Attractor::new(a.x, a.y, a.strength))
                .collect();
        }

        if let Some(strength) = self.attractor_strength {
            config.attractor_strength = strength;
        }

        if !self.obstacles.is_empty() {
            config.obstacles = self.obstacles.iter().map(|o| o.obstacle.clone()).collect();
        }
        let _ = config.load_obstacle_masks();

        // Separate trails: CLI/species-colors force-on; absent both, preset survives.
        if self.separate_species_trails || self.species_colors {
            config.separate_species_trails = true;
        }

        if let Some(use_simd) = self.use_simd {
            config.use_simd = use_simd;
        }

        if !self.species.is_empty() {
            // User explicitly provided species
            config.species_configs = self
                .species
                .iter()
                .map(|s| SpeciesConfig {
                    name: s.name.clone(),
                    count: s.count,
                    sensor_angle: s.sensor_angle,
                    rotation_angle: s.rotation_angle,
                    step_size: s.step_size,
                    deposit_amount: s.deposit_amount,
                    color: s.color,
                    trail_modulation: None,
                })
                .collect();
        } else if self.preset.is_none() {
            // Only use default/CLI-overridden single species if NOT using a preset
            use crate::render::palette::RgbColor;
            config.species_configs = vec![SpeciesConfig {
                name: "default".to_string(),
                count: self.population.unwrap_or(population::DEFAULT_POPULATION),
                sensor_angle: self.sensor_angle.unwrap_or(config.sensor_angle),
                rotation_angle: self.rotation_angle.unwrap_or(config.rotation_angle),
                step_size: self.step_size.unwrap_or(config.step_size),
                deposit_amount: self.deposit_amount.unwrap_or(config.deposit_amount),
                color: RgbColor::from_hex(0x228b22),
                trail_modulation: None,
            }];
        } else if let Some(preset_species) = config.species_configs.first_mut() {
            // If using a preset, allow overriding the FIRST species' properties with CLI args if
            // provided
            if let Some(pop) = self.population {
                preset_species.count = pop;
            }
            if let Some(sa) = self.sensor_angle {
                preset_species.sensor_angle = sa;
            }
            if let Some(ra) = self.rotation_angle {
                preset_species.rotation_angle = ra;
            }
            if let Some(ss) = self.step_size {
                preset_species.step_size = ss;
            }
            if let Some(da) = self.deposit_amount {
                preset_species.deposit_amount = da;
            }
        }

        // Wind: CLI overrides; absent the flag, the preset's wind survives.
        if let Some(ref w) = self.wind {
            config.wind = Some(Wind::new(w.dx, w.dy));
        }

        // Terrain
        if let Some(ref terrain_str) = self.terrain {
            config.terrain = terrain_str.parse::<TerrainType>().map_err(|_| {
                crate::error::ValidationError::custom(format!(
                    "invalid terrain type: {terrain_str}"
                ))
            })?;
        }
        if let Some(strength) = self.terrain_strength {
            config.terrain_strength = strength;
        }

        // Background color: CLI overrides; absent the flag, the preset's bg survives.
        if let Some(ref bg) = self.background_color {
            config.background_color = Some(bg.clone());
        }

        // Boundary mode: preset suggests (via PresetSimDefaults), CLI overrides.
        // The preset layer goes first so an explicit --boundary-mode still wins.
        if let Some(preset) = self.preset {
            config.boundary_mode = PresetSimDefaults::from(preset).boundary_mode;
        }
        if let Some(mode) = self.boundary_mode {
            config.boundary_mode = mode;
        }

        // Window frame mode
        if let Some(mode) = self.window_frame {
            config.window_frame = mode;
        }

        // Chrome style, aspect, padding, status bar, size thresholds
        if let Some(cs) = self.chrome_style {
            config.chrome_style = cs;
        }
        if let Some(a) = self.aspect {
            config.aspect = a;
        }
        if let Some(p) = self.window_padding {
            config.window_padding = p;
        }
        if let Some(v) = self.show_status_bar {
            config.show_status_bar = v;
        }
        if let Some(t) = self.min_sim_size {
            config.min_sim_size = t;
        }
        if let Some(t) = self.min_frame_size {
            config.min_frame_size = t;
        }

        // Respawn configuration
        if let Some(interval) = self.respawn_interval {
            config.respawn_config.interval = interval;
        }

        Ok(config)
    }

    /// Verbatim port of `Args::to_render_art_defaults` (`src/cli.rs:2064-2117`) +
    /// `Args::resolve_render_config` (`src/cli.rs:2129-2167`), reading `self`
    /// render fields instead of `Args` fields.
    ///
    /// Choice: option (a) — duplicate the body here, leaving cli.rs originals
    /// intact as the parity oracle for Task 1 tests. Task 2 deletes the originals.
    fn resolve_render(&self) -> Result<ResolvedRenderConfig, String> {
        use crate::render::charset::ALL_CHARSETS;
        use crate::render_art_defaults::RenderArtDefaults;

        // ── to_render_art_defaults body (src/cli.rs:2064-2117) ──
        let mut art = match self.preset {
            Some(preset) => RenderArtDefaults::from(preset),
            None => RenderArtDefaults::default(),
        };
        if let Some(ref im) = self.intensity_mapping {
            art.intensity_mapping = im.clone();
        }
        if let Some(ref pc) = self.palette_cycle {
            art.palette_cycle = *pc;
        }
        // Mirror glyph_config_parsed(art.glyph) exactly (src/cli.rs:2006-2018):
        // only apply when at least one glyph CLI flag was explicitly provided.
        if self.glyph_selection.is_some() || self.glyph_edge_threshold.is_some() {
            if let Some(sel) = self.glyph_selection {
                art.glyph.selection = Some(sel);
            }
            if let Some(t) = self.glyph_edge_threshold {
                // Already clamped in from_args; applied unconditionally like the oracle.
                art.glyph.edge_threshold = t;
            }
        }
        if let Some(c) = self.temporal_color {
            art.temporal_color = c;
        }
        if let Some(l) = self.temporal_lag_frames {
            art.temporal_lag_frames = l;
        }
        if let Some(m) = self.temporal_mode {
            art.temporal_mode = m;
        }
        if let Some(accent) = self.temporal_accent {
            art.temporal_accent = Some(accent);
        }
        if let Some(a) = self.afterglow {
            art.afterglow = a;
        }
        if let Some(r) = self.afterglow_rate {
            art.afterglow_rate = r;
        }
        // Validation parity: enforce afterglow ranges as in to_render_art_defaults.
        crate::validation::rules::AFTERGLOW
            .validate_f32(art.afterglow)
            .map_err(|e| e.to_string())?;
        crate::validation::rules::AFTERGLOW_RATE
            .validate_f32(art.afterglow_rate)
            .map_err(|e| e.to_string())?;

        // ── resolve_render_config body (src/cli.rs:2129-2167) ──
        let palette = if let Some(ref p) = self.palette {
            // CLI explicitly set
            p.clone()
        } else {
            // Mirror the oracle's fallback: art.palette else the default-palette name.
            // We parse DEFAULT_PALETTE_NAME rather than hard-coding Palette::Moss so
            // this stays correct if the default is changed. (Finding 2.)
            // Invariant: crate::config_defaults::palette::DEFAULT_PALETTE_NAME == "moss"
            // → parse always succeeds; Palette::Moss is a safe fallback.
            art.palette.unwrap_or_else(|| {
                use crate::render::palette::PALETTES;
                PALETTES
                    .iter()
                    .find(|spec| {
                        spec.name.eq_ignore_ascii_case(
                            crate::config_defaults::palette::DEFAULT_PALETTE_NAME,
                        )
                    })
                    .map(|spec| spec.palette.clone())
                    .unwrap_or(crate::cli::Palette::Moss)
            })
        };
        let charset = self
            .charset
            .clone()
            .or(art.charset)
            .unwrap_or_else(|| ALL_CHARSETS[0].clone());
        let charset_index = ALL_CHARSETS.iter().position(|c| c == &charset).unwrap_or(0);
        let color_aa = self
            .color_aa
            .or(art.color_aa)
            .unwrap_or(crate::config_defaults::DEFAULT_COLOR_AA[charset_index]);
        let hue_shift = self.hue_shift.unwrap_or(art.hue_shift);
        Ok(ResolvedRenderConfig {
            palette,
            charset,
            color_aa,
            hue_shift,
            intensity_mapping: art.intensity_mapping,
            palette_cycle: art.palette_cycle,
            glyph: art.glyph,
            temporal_color: art.temporal_color,
            temporal_lag_frames: art.temporal_lag_frames,
            temporal_mode: art.temporal_mode,
            temporal_accent: art.temporal_accent,
            afterglow: art.afterglow,
            afterglow_rate: art.afterglow_rate,
        })
    }
}

/// Deterministic dump of the assembled sim-relevant fields.
/// Used by the preset-config snapshot net (tests/preset_config_snapshot.rs).
pub(crate) fn dump_sim_config(config: &crate::simulation::config::SimConfig) -> String {
    use std::fmt::Write;
    let mut s = String::new();
    let _ = writeln!(s, "sensor_angle={:?}", config.sensor_angle);
    let _ = writeln!(s, "sensor_distance={:?}", config.sensor_distance);
    let _ = writeln!(s, "rotation_angle={:?}", config.rotation_angle);
    let _ = writeln!(s, "step_size={:?}", config.step_size);
    let _ = writeln!(s, "decay_factor={:?}", config.decay_factor);
    let _ = writeln!(s, "deposit_amount={:?}", config.deposit_amount);
    let _ = writeln!(s, "diffusion_kernel={:?}", config.diffusion_kernel);
    let _ = writeln!(s, "diffusion_sigma={:?}", config.diffusion_sigma);
    let _ = writeln!(s, "max_brightness={:?}", config.max_brightness);
    let _ = writeln!(s, "decay_gamma={:?}", config.decay_gamma);
    let _ = writeln!(s, "diffuse_weight={:?}", config.diffuse_weight);
    let _ = writeln!(s, "deposit_curve={:?}", config.deposit_curve);
    let _ = writeln!(s, "deposit_scale={:?}", config.deposit_scale);
    let _ = writeln!(s, "deposit_gamma={:?}", config.deposit_gamma);
    let _ = writeln!(s, "deposit_cap={:?}", config.deposit_cap);
    let _ = writeln!(s, "boundary_mode={:?}", config.boundary_mode);
    let _ = writeln!(s, "preferred_init_mode={:?}", config.preferred_init_mode);
    let _ = writeln!(s, "wind={:?}", config.wind);
    let _ = writeln!(s, "background_color={:?}", config.background_color);
    let _ = writeln!(s, "obstacles={:?}", config.obstacles);
    let _ = writeln!(s, "attractors={:?}", config.attractors);
    let _ = writeln!(
        s,
        "separate_species_trails={:?}",
        config.separate_species_trails
    );
    let _ = writeln!(s, "sampling_mode={:?}", config.sampling_mode);
    let _ = writeln!(s, "respawn_config={:?}", config.respawn_config);
    for (i, sp) in config.species_configs.iter().enumerate() {
        let _ = writeln!(
            s,
            "species[{i}]: name={:?} count={} sa={:?} ra={:?} ss={:?} da={:?} color={:?} mod={}",
            sp.name,
            sp.count,
            sp.sensor_angle,
            sp.rotation_angle,
            sp.step_size,
            sp.deposit_amount,
            sp.color,
            sp.trail_modulation.is_some()
        );
        if let Some(ref m) = sp.trail_modulation {
            let _ = writeln!(s, "  modulation={m:?}");
        }
    }
    s
}

// ── serde helpers for fields needing custom (de)serialization ──

/// Serde module for `Option<RgbColor>` serialized as a `"rrggbb"` hex string.
mod serde_opt_rgb_hex {
    use crate::render::palette::RgbColor;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    pub fn serialize<S: Serializer>(color: &Option<RgbColor>, s: S) -> Result<S::Ok, S::Error> {
        match color {
            Some(c) => format!("{:02x}{:02x}{:02x}", c.r, c.g, c.b).serialize(s),
            None => s.serialize_none(),
        }
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<Option<RgbColor>, D::Error> {
        let hex = String::deserialize(d)?;
        let hex = hex.trim_start_matches('#');
        let v = u32::from_str_radix(hex, 16).map_err(serde::de::Error::custom)?;
        Ok(Some(RgbColor::from_hex(v)))
    }
}

/// Serde module for `Option<Palette>`.
/// Built-in palettes serialize as their lowercase name (e.g. `"heat"`, `"moss"`).
/// `Custom` serializes as `"custom:#rrggbb,#rrggbb,..."`.
mod serde_opt_palette {
    use crate::render::palette::{Palette, RgbColor, PALETTES};
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    pub fn serialize<S: Serializer>(p: &Option<Palette>, s: S) -> Result<S::Ok, S::Error> {
        match p {
            None => s.serialize_none(),
            Some(Palette::Custom(colors)) => {
                let parts: Vec<String> = colors
                    .iter()
                    .map(|c| format!("#{:02x}{:02x}{:02x}", c.r, c.g, c.b))
                    .collect();
                format!("custom:{}", parts.join(",")).serialize(s)
            }
            Some(p) => {
                let name = PALETTES
                    .iter()
                    .find(|spec| &spec.palette == p)
                    .map(|spec| spec.name)
                    .ok_or_else(|| {
                        serde::ser::Error::custom(format!("palette not in PALETTES: {:?}", p))
                    })?;
                name.to_lowercase().serialize(s)
            }
        }
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<Option<Palette>, D::Error> {
        let s = String::deserialize(d)?;
        if let Some(rest) = s.strip_prefix("custom:") {
            // Parse comma-separated hex colors
            if rest.is_empty() {
                return Ok(Some(Palette::Custom(vec![])));
            }
            let colors: Result<Vec<RgbColor>, _> = rest
                .split(',')
                .map(|hex| {
                    let hex = hex.trim_start_matches('#');
                    u32::from_str_radix(hex, 16)
                        .map(RgbColor::from_hex)
                        .map_err(|e| serde::de::Error::custom(format!("invalid hex color: {e}")))
                })
                .collect();
            return Ok(Some(Palette::Custom(colors?)));
        }
        PALETTES
            .iter()
            .find(|spec| spec.name.eq_ignore_ascii_case(&s))
            .map(|spec| Some(spec.palette.clone()))
            .ok_or_else(|| serde::de::Error::custom(format!("unknown palette: {s}")))
    }
}

/// Serde module for `Option<Charset>`.
/// Named charsets serialize as lowercase tokens (`"halfblock"`, `"braille"`, etc.).
/// `CustomAscii` serializes as `"custom:<chars>"`.
mod serde_opt_charset {
    use crate::render::charset::Charset;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    pub fn serialize<S: Serializer>(c: &Option<Charset>, s: S) -> Result<S::Ok, S::Error> {
        match c {
            None => s.serialize_none(),
            Some(Charset::CustomAscii(chars)) => {
                let encoded: String = chars.iter().collect();
                format!("custom:{encoded}").serialize(s)
            }
            Some(c) => {
                let name = match c {
                    Charset::HalfBlock => "halfblock",
                    Charset::HalfBlockDual => "halfblockdual",
                    Charset::Ascii => "ascii",
                    Charset::Braille => "braille",
                    Charset::Quadrant => "quadrant",
                    Charset::Shade => "shade",
                    Charset::Points => "points",
                    Charset::Sculpted => "sculpted",
                    Charset::CustomAscii(_) => unreachable!(),
                };
                name.serialize(s)
            }
        }
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<Option<Charset>, D::Error> {
        let s = String::deserialize(d)?;
        if let Some(rest) = s.strip_prefix("custom:") {
            return Ok(Some(Charset::from_custom_string(rest)));
        }
        let charset = match s.as_str() {
            "halfblock" => Charset::HalfBlock,
            "halfblockdual" => Charset::HalfBlockDual,
            "ascii" => Charset::Ascii,
            "braille" => Charset::Braille,
            "quadrant" => Charset::Quadrant,
            "shade" => Charset::Shade,
            "points" => Charset::Points,
            "sculpted" => Charset::Sculpted,
            other => {
                return Err(serde::de::Error::custom(format!(
                    "unknown charset: {other}"
                )))
            }
        };
        Ok(Some(charset))
    }
}

/// Serde module for `Option<IntensityMapping>`.
///
/// Serializes as a TOML inline table with `name`, and optional `base`, `gamma`, `levels`
/// fields matching the `config_manager.rs` format. Perlin and split mappings are lossy
/// (serialized as nothing = deserializes to `None`).
mod serde_opt_intensity_mapping {
    use crate::render::palette::{IntensityMapping, MappingFunction};
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    #[derive(Serialize, Deserialize)]
    struct Proxy {
        name: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        base: Option<f32>,
        #[serde(skip_serializing_if = "Option::is_none")]
        gamma: Option<f32>,
        #[serde(skip_serializing_if = "Option::is_none")]
        levels: Option<u8>,
    }

    pub fn serialize<S: Serializer>(m: &Option<IntensityMapping>, s: S) -> Result<S::Ok, S::Error> {
        let mapping = match m {
            None => return s.serialize_none(),
            Some(m) => m,
        };
        // Extract name + params from the single-segment case (multi-segment is lossy).
        let proxy = if mapping.segments().len() == 1 {
            match &mapping.segments()[0].function {
                MappingFunction::Linear => Some(Proxy {
                    name: "linear".to_string(),
                    base: None,
                    gamma: None,
                    levels: None,
                }),
                MappingFunction::Logarithmic { base } => Some(Proxy {
                    name: "logarithmic".to_string(),
                    base: Some(*base),
                    gamma: None,
                    levels: None,
                }),
                MappingFunction::Exponential { base } => Some(Proxy {
                    name: "exponential".to_string(),
                    base: Some(*base),
                    gamma: None,
                    levels: None,
                }),
                MappingFunction::Power { gamma } => Some(Proxy {
                    name: "power".to_string(),
                    base: None,
                    gamma: Some(*gamma),
                    levels: None,
                }),
                MappingFunction::SquareRoot => Some(Proxy {
                    name: "sqrt".to_string(),
                    base: None,
                    gamma: None,
                    levels: None,
                }),
                MappingFunction::Square => Some(Proxy {
                    name: "square".to_string(),
                    base: None,
                    gamma: None,
                    levels: None,
                }),
                MappingFunction::Sigmoid { steepness } => Some(Proxy {
                    name: "sigmoid".to_string(),
                    base: Some(*steepness),
                    gamma: None,
                    levels: None,
                }),
                MappingFunction::Smoothstep => Some(Proxy {
                    name: "smoothstep".to_string(),
                    base: None,
                    gamma: None,
                    levels: None,
                }),
                MappingFunction::Quantize { levels } => Some(Proxy {
                    name: "quantize".to_string(),
                    base: None,
                    gamma: None,
                    levels: Some(*levels),
                }),
                // Perlin is lossy — cannot faithfully round-trip via this format.
                MappingFunction::Perlin { .. } => None,
            }
        } else {
            // Multi-segment (e.g. linear_log_split) is lossy.
            None
        };
        match proxy {
            Some(p) => p.serialize(s),
            None => s.serialize_none(),
        }
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(
        d: D,
    ) -> Result<Option<IntensityMapping>, D::Error> {
        use crate::render::palette::{MappingFunction, MappingSegment};
        let proxy = Proxy::deserialize(d)?;
        let unit_seg = |function: MappingFunction| -> Option<IntensityMapping> {
            IntensityMapping::new(vec![MappingSegment {
                start: 0.0,
                end: 1.0,
                function,
            }])
            .ok()
        };
        let mapping = match proxy.name.as_str() {
            "linear" => Some(IntensityMapping::linear()),
            "logarithmic" | "log" => {
                Some(IntensityMapping::logarithmic(proxy.base.unwrap_or(10.0)))
            }
            "exponential" | "exp" => {
                Some(IntensityMapping::exponential(proxy.base.unwrap_or(10.0)))
            }
            "power" | "pow" => Some(IntensityMapping::power(proxy.gamma.unwrap_or(2.2))),
            "sqrt" | "squareroot" => unit_seg(MappingFunction::SquareRoot),
            "square" => unit_seg(MappingFunction::Square),
            "sigmoid" => Some(IntensityMapping::sigmoid(proxy.base.unwrap_or(10.0))),
            "smoothstep" => Some(IntensityMapping::smoothstep()),
            "quantize" => Some(IntensityMapping::quantize(proxy.levels.unwrap_or(8))),
            _ => None,
        };
        Ok(mapping)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::simulation::config::PRESETS;
    use clap::Parser;

    fn args(extra: &[&str]) -> Args {
        let mut v = vec!["tslime"];
        v.extend_from_slice(extra);
        Args::parse_from(v)
    }

    fn resolve(extra: &[&str]) -> crate::profile::Profile {
        ProfileOverrides::from_args(&args(extra))
            .and_then(|o| o.resolve())
            .expect("resolve")
    }

    #[test]
    fn test_build_default() {
        use crate::config_defaults::agent;
        use crate::config_defaults::population;
        let c = resolve(&[]).sim;
        assert_eq!(c.sensor_angle, agent::DEFAULT_SENSOR_ANGLE);
        assert_eq!(c.total_population(), population::DEFAULT_POPULATION);
    }

    #[test]
    fn empty_cli_attractors_keep_preset_base_then_override_lands() {
        let c = resolve(&["--preset", "organic"]).sim;
        assert!(c.attractors.is_empty());
        let c = resolve(&["--preset", "organic", "--attract", "5,5,1.0"]).sim;
        assert_eq!(c.attractors.len(), 1);
    }

    #[test]
    fn no_trail_flags_keep_preset_separate_trails() {
        let c = resolve(&["--preset", "organic"]).sim;
        assert!(!c.separate_species_trails);
    }

    #[cfg(feature = "multi-species")]
    #[test]
    fn species_colors_forces_separate_trails_on() {
        let c = resolve(&["--preset", "organic", "--species-colors"]).sim;
        assert!(c.separate_species_trails);
    }

    #[test]
    fn test_build_with_overrides() {
        let c = resolve(&["--sensor-angle", "30", "--population", "10000"]).sim;
        assert_eq!(c.sensor_angle, 30.0);
        assert_eq!(c.total_population(), 10000);
    }

    #[test]
    fn test_with_preset_override() {
        let c = resolve(&["--preset", "organic", "--sensor-angle", "15"]).sim;
        assert_eq!(c.sensor_angle, 15.0);
    }

    #[test]
    fn test_preset_decay_gamma_survives_assemble() {
        let c = resolve(&["--preset", "wane"]).sim;
        assert_eq!(c.decay_gamma, 0.6);
    }

    #[test]
    fn test_preset_diffuse_weight_survives_assemble() {
        let c = resolve(&["--preset", "marble"]).sim;
        assert_eq!(c.diffuse_weight, 0.8);
    }

    #[test]
    fn test_cli_overrides_preset_decay_gamma_and_diffuse_weight() {
        let c = resolve(&[
            "--preset",
            "wane",
            "--decay-gamma",
            "0.3",
            "--diffuse-weight",
            "0.5",
        ])
        .sim;
        assert_eq!(c.decay_gamma, 0.3);
        assert_eq!(c.diffuse_weight, 0.5);
    }

    #[test]
    fn test_boundary_mode_defaults_to_bounce_without_preset_or_flag() {
        use crate::simulation::config::BoundaryMode;
        let c = resolve(&[]).sim;
        assert_eq!(c.boundary_mode, BoundaryMode::Bounce);
    }

    #[test]
    fn test_cli_boundary_mode_flag_wins() {
        use crate::simulation::config::BoundaryMode;
        let c = resolve(&["--boundary-mode", "wrap"]).sim;
        assert_eq!(c.boundary_mode, BoundaryMode::Wrap);
    }

    #[test]
    fn test_only_river_and_ripple_resolve_to_wrap() {
        use crate::simulation::config::{BoundaryMode, Preset};
        for spec in PRESETS {
            let a = args(&["--preset", spec.name]);
            let c = ProfileOverrides::from_args(&a)
                .and_then(|o| o.resolve())
                .expect("resolve")
                .sim;
            let expected = match spec.preset {
                Preset::River | Preset::Ripple => BoundaryMode::Wrap,
                _ => BoundaryMode::Bounce,
            };
            assert_eq!(
                c.boundary_mode, expected,
                "{} boundary_mode resolved unexpectedly",
                spec.name
            );
        }
    }

    #[test]
    fn test_ripple_and_river_declare_boundary_wrap() {
        use crate::simulation::config::BoundaryMode;
        for name in ["ripple", "river"] {
            let c = resolve(&["--preset", name]).sim;
            assert_eq!(
                c.boundary_mode,
                BoundaryMode::Wrap,
                "{name} should declare boundary-mode wrap"
            );
        }
    }

    #[test]
    fn test_cli_boundary_mode_overrides_preset_wrap() {
        use crate::simulation::config::BoundaryMode;
        let c = resolve(&["--preset", "ripple", "--boundary-mode", "bounce"]).sim;
        assert_eq!(c.boundary_mode, BoundaryMode::Bounce);
    }

    #[test]
    fn test_no_preset_no_flag_uses_default_decay_gamma_and_diffuse_weight() {
        use crate::config_defaults::trail;
        let c = resolve(&[]).sim;
        assert_eq!(c.decay_gamma, trail::DEFAULT_DECAY_GAMMA);
        assert_eq!(c.diffuse_weight, trail::DEFAULT_DIFFUSE_WEIGHT);
    }

    #[test]
    fn river_preset_keeps_wind() {
        use crate::simulation::config::Wind;
        let c = resolve(&["--preset", "river"]).sim;
        assert_eq!(c.wind, Some(Wind::new(0.3, 0.0)));
    }

    #[test]
    fn cli_wind_overrides_preset_wind() {
        use crate::simulation::config::Wind;
        let c = resolve(&["--preset", "river", "--wind", "1,0"]).sim;
        assert_eq!(c.wind, Some(Wind::new(1.0, 0.0)));
    }

    #[test]
    fn petridish_preset_keeps_obstacle_and_bg() {
        let c = resolve(&["--preset", "petridish"]).sim;
        assert_eq!(c.obstacles.len(), 1);
        assert_eq!(c.background_color.as_deref(), Some("000000"));
    }

    #[test]
    fn empty_cli_obstacles_do_not_clear_preset_obstacles() {
        let c = resolve(&["--preset", "petridish"]).sim;
        assert!(!c.obstacles.is_empty());
    }

    /// Validation parity: invalid sensor_angle must be rejected (Phase A CRITICAL).
    #[test]
    fn resolve_rejects_invalid_sensor_angle() {
        let a = args(&["--sensor-angle", "999"]);
        assert!(ProfileOverrides::from_args(&a)
            .and_then(|o| o.resolve())
            .is_err());
    }

    /// Seed passthrough.
    #[test]
    fn seed_passthrough() {
        assert_eq!(
            ProfileOverrides::from_args(&args(&[]))
                .and_then(|o| o.resolve())
                .unwrap()
                .seed,
            None
        );
        assert_eq!(
            ProfileOverrides::from_args(&args(&["--seed", "7"]))
                .and_then(|o| o.resolve())
                .unwrap()
                .seed,
            Some(7)
        );
    }

    /// Brightness gain→white-point: --brightness 2.0 must produce the white-point
    /// that `white_point_from_gain(2.0)` returns. Pins the conversion so it cannot
    /// silently break.
    #[test]
    fn brightness_gain_to_white_point_conversion() {
        use crate::config_defaults::trail;
        let gain = 2.0_f32;
        let expected = trail::white_point_from_gain(gain);
        let c = resolve(&["--brightness", "2.0"]).sim;
        assert_eq!(
            c.max_brightness, expected,
            "--brightness 2.0 should map to white_point_from_gain(2.0) = {expected}"
        );
    }

    /// High-FPS branch: --fps 60 (with no explicit kernel/sigma flags) must
    /// activate the Gaussian kernel with sigma 0.5. This branch previously had zero
    /// direct assertion.
    #[test]
    fn high_fps_activates_gaussian_kernel() {
        use crate::simulation::config::DiffusionKernel;
        let c = resolve(&["--fps", "60"]).sim;
        assert_eq!(
            c.diffusion_kernel,
            DiffusionKernel::Gaussian,
            "--fps 60 should switch diffusion kernel to Gaussian"
        );
        assert_eq!(
            c.diffusion_sigma, 0.5,
            "--fps 60 Gaussian branch should set sigma to 0.5"
        );
    }

    // ── serde TOML round-trip tests ──

    #[test]
    fn overrides_toml_round_trip_scalars() {
        let o = ProfileOverrides {
            sensor_angle: Some(33.0),
            decay_factor: Some(0.91),
            diffusion_kernel: Some(DiffusionKernel::Gaussian),
            palette: Some(Palette::Heat),
            temporal_color: Some(0.6),
            ..ProfileOverrides::default()
        };
        let s = toml::to_string(&o).expect("ser");
        let back: ProfileOverrides = toml::from_str(&s).expect("de");
        assert_eq!(o, back);
    }

    #[test]
    fn overrides_toml_omitted_fields_default_to_none() {
        let back: ProfileOverrides = toml::from_str("sensor_angle = 12.0\n").expect("de");
        assert_eq!(back.sensor_angle, Some(12.0));
        assert_eq!(back.decay_factor, None);
    }

    #[test]
    fn overrides_toml_intensity_mapping_round_trip() {
        let o = ProfileOverrides {
            intensity_mapping: Some(IntensityMapping::quantize(6)),
            ..ProfileOverrides::default()
        };
        let s = toml::to_string(&o).expect("ser");
        let back: ProfileOverrides = toml::from_str(&s).expect("de");
        assert_eq!(o.intensity_mapping, back.intensity_mapping);
    }

    #[test]
    fn overrides_toml_attractor_vec_round_trip() {
        use crate::cli::AttractorArg;
        let o = ProfileOverrides {
            attractors: vec![AttractorArg {
                x: 100.0,
                y: 200.0,
                strength: 1.5,
            }],
            ..ProfileOverrides::default()
        };
        let s = toml::to_string(&o).expect("ser");
        let back: ProfileOverrides = toml::from_str(&s).expect("de");
        assert_eq!(o, back);
    }

    #[test]
    fn overrides_toml_wind_round_trip() {
        use crate::cli::WindArg;
        let o = ProfileOverrides {
            wind: Some(WindArg { dx: 0.5, dy: -0.3 }),
            ..ProfileOverrides::default()
        };
        let s = toml::to_string(&o).expect("ser");
        let back: ProfileOverrides = toml::from_str(&s).expect("de");
        assert_eq!(o, back);
    }

    #[test]
    fn overrides_toml_temporal_accent_hex_round_trip() {
        let o = ProfileOverrides {
            temporal_accent: Some(RgbColor::new(0xff, 0xb3, 0x47)),
            ..ProfileOverrides::default()
        };
        let s = toml::to_string(&o).expect("ser");
        let back: ProfileOverrides = toml::from_str(&s).expect("de");
        assert_eq!(o.temporal_accent, back.temporal_accent);
        // verify it's stored as hex string
        assert!(s.contains("ffb347"), "expected hex 'ffb347' in: {s}");
    }

    #[test]
    fn overrides_toml_glyph_selection_round_trip() {
        use crate::render::charset::GlyphSelection;
        let o = ProfileOverrides {
            glyph_selection: Some(GlyphSelection::Hybrid),
            glyph_edge_threshold: Some(0.25),
            ..ProfileOverrides::default()
        };
        let s = toml::to_string(&o).expect("ser");
        let back: ProfileOverrides = toml::from_str(&s).expect("de");
        assert_eq!(o, back);
    }

    #[test]
    fn overrides_toml_palette_cycle_round_trip() {
        use crate::render::palette::{PaletteCycle, PaletteCycleMode};
        let o = ProfileOverrides {
            palette_cycle: Some(PaletteCycle {
                cycles: 3,
                mode: PaletteCycleMode::Wrap,
            }),
            ..ProfileOverrides::default()
        };
        let s = toml::to_string(&o).expect("ser");
        let back: ProfileOverrides = toml::from_str(&s).expect("de");
        assert_eq!(o, back);
    }

    #[test]
    fn overrides_toml_lossless_full_struct_round_trip() {
        // Round-trip with many losslessly-representable fields set.
        // Excludes IntensityMapping Perlin/Split (lossy).
        use crate::render::palette::TemporalMode;
        use crate::simulation::config::DepositCurve;
        let o = ProfileOverrides {
            preset: Some(Preset::Organic),
            seed: Some(42),
            sensor_angle: Some(22.5),
            decay_factor: Some(0.85),
            diffusion_kernel: Some(DiffusionKernel::Gaussian),
            palette: Some(Palette::Ocean),
            temporal_color: Some(0.5),
            temporal_mode: Some(TemporalMode::Accent),
            afterglow: Some(0.3),
            deposit_curve: Some(DepositCurve::Sqrt),
            intensity_mapping: Some(IntensityMapping::quantize(4)),
            ..ProfileOverrides::default()
        };
        let s = toml::to_string(&o).expect("ser");
        let back: ProfileOverrides = toml::from_str(&s).expect("de");
        assert_eq!(o, back);
    }

    #[test]
    fn overrides_toml_intensity_mapping_perlin_is_lossy() {
        let o = ProfileOverrides {
            intensity_mapping: Some(IntensityMapping::perlin(0.15, 4.0, 42)),
            ..ProfileOverrides::default()
        };
        let s = toml::to_string(&o).expect("ser");
        let back: ProfileOverrides = toml::from_str(&s).expect("de");
        // Perlin is lossy — serializes as None (no recognized name)
        assert_eq!(back.intensity_mapping, None);
    }

    // ── Important 1: Vec fields must not emit empty arrays in minimal TOML ──

    #[test]
    fn overrides_toml_empty_vecs_not_emitted() {
        // A near-empty ProfileOverrides should NOT contain "attractors", "obstacles",
        // or "species" keys in the serialized TOML.
        let o = ProfileOverrides {
            sensor_angle: Some(30.0),
            ..ProfileOverrides::default()
        };
        let s = toml::to_string(&o).expect("ser");
        assert!(
            !s.contains("attractors"),
            "empty attractors must not be emitted; got: {s}"
        );
        assert!(
            !s.contains("obstacles"),
            "empty obstacles must not be emitted; got: {s}"
        );
        assert!(
            !s.contains("species"),
            "empty species must not be emitted; got: {s}"
        );
    }

    // ── Important 2: serde_opt_charset named + CustomAscii round-trips ──

    #[test]
    fn overrides_toml_charset_braille_round_trip() {
        use crate::render::charset::Charset;
        let o = ProfileOverrides {
            charset: Some(Charset::Braille),
            ..ProfileOverrides::default()
        };
        let s = toml::to_string(&o).expect("ser");
        // Must serialize as lowercase "braille"
        assert!(s.contains("braille"), "expected 'braille' token in: {s}");
        let back: ProfileOverrides = toml::from_str(&s).expect("de");
        assert_eq!(back.charset, Some(Charset::Braille));
    }

    #[test]
    fn overrides_toml_charset_custom_ascii_round_trip() {
        use crate::render::charset::Charset;
        // Build a CustomAscii via from_custom_string so ordering is canonical.
        let original = Charset::from_custom_string("@#.!");
        let o = ProfileOverrides {
            charset: Some(original.clone()),
            ..ProfileOverrides::default()
        };
        let s = toml::to_string(&o).expect("ser");
        // Token must start with "custom:"
        assert!(s.contains("custom:"), "expected 'custom:' prefix in: {s}");
        let back: ProfileOverrides = toml::from_str(&s).expect("de");
        assert_eq!(back.charset, Some(original));
    }

    // ── Important 2: serde_opt_palette Custom round-trip ──

    #[test]
    fn overrides_toml_palette_custom_round_trip() {
        use crate::render::palette::{Palette, RgbColor};
        let colors = vec![
            RgbColor::new(0xff, 0x00, 0x00),
            RgbColor::new(0x00, 0xff, 0x00),
            RgbColor::new(0x00, 0x00, 0xff),
        ];
        let o = ProfileOverrides {
            palette: Some(Palette::Custom(colors.clone())),
            ..ProfileOverrides::default()
        };
        let s = toml::to_string(&o).expect("ser");
        // Token must start with "custom:"
        assert!(s.contains("custom:"), "expected 'custom:' prefix in: {s}");
        let back: ProfileOverrides = toml::from_str(&s).expect("de");
        assert_eq!(back.palette, Some(Palette::Custom(colors)));
    }

    // ── Important 3: linear_log_split lossy test ──

    #[test]
    fn overrides_toml_intensity_mapping_linear_log_split_is_lossy() {
        // linear_log_split is multi-segment — serializes as None (lossy, same as Perlin).
        let o = ProfileOverrides {
            intensity_mapping: Some(IntensityMapping::linear_log_split(10.0)),
            ..ProfileOverrides::default()
        };
        let s = toml::to_string(&o).expect("ser");
        let back: ProfileOverrides = toml::from_str(&s).expect("de");
        // Multi-segment is lossy — must come back as None
        assert_eq!(
            back.intensity_mapping, None,
            "linear_log_split must be lossy; serialized TOML: {s}"
        );
    }

    // ── Minor 5: AaStrength uses lowercase serde tokens ──

    #[test]
    fn overrides_toml_color_aa_lowercase_token_and_round_trip() {
        use crate::render::antialiasing::AaStrength;
        for (variant, expected_token) in [
            (AaStrength::Off, "off"),
            (AaStrength::Subtle, "subtle"),
            (AaStrength::Strong, "strong"),
        ] {
            let o = ProfileOverrides {
                color_aa: Some(variant),
                ..ProfileOverrides::default()
            };
            let s = toml::to_string(&o).expect("ser");
            assert!(
                s.contains(expected_token),
                "AaStrength::{variant:?} should serialize as '{expected_token}'; got: {s}"
            );
            let back: ProfileOverrides = toml::from_str(&s).expect("de");
            assert_eq!(
                back.color_aa,
                Some(variant),
                "AaStrength::{variant:?} must round-trip"
            );
        }
    }
}
