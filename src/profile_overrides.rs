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

/// The single all-`Option` authored partial (sim ⊕ render ⊕ seed). Sim fields
/// mirror the former `ConfigBuilder`; render fields mirror what
/// `Args::resolve_render_config` reads.
#[derive(Clone, Debug, PartialEq, Default)]
pub(crate) struct ProfileOverrides {
    // ── provenance / base selector ──
    pub preset: Option<Preset>,
    pub seed: Option<u64>,

    // ── sim levers (mirror src/config_builder.rs:17-60 ConfigBuilder) ──
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
    pub attractors: Vec<AttractorArg>,
    pub attractor_strength: Option<f32>,
    pub obstacles: Vec<ObstacleArg>,
    pub species: Vec<SpeciesArg>,
    pub separate_species_trails: bool,
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
    pub palette: Option<Palette>,
    pub charset: Option<Charset>,
    pub color_aa: Option<AaStrength>,
    /// CLI `--palette-shift` (maps to hue_shift). `None` falls through to preset art.
    pub hue_shift: Option<f32>,
    pub intensity_mapping: Option<IntensityMapping>,
    pub palette_cycle: Option<PaletteCycle>,
    /// `Some` only when `--glyph-selection` was explicitly provided on the CLI.
    pub glyph_selection: Option<GlyphSelection>,
    /// `Some` only when `--glyph-edge-threshold` was explicitly provided on the CLI.
    pub glyph_edge_threshold: Option<f32>,
    pub temporal_color: Option<f32>,
    pub temporal_lag_frames: Option<f32>,
    pub temporal_mode: Option<TemporalMode>,
    pub temporal_accent: Option<RgbColor>,
    pub afterglow: Option<f32>,
    pub afterglow_rate: Option<f32>,
}

impl ProfileOverrides {
    /// Builds a `ProfileOverrides` from CLI args. Sim block is a verbatim port of
    /// `ConfigBuilder::from_args` (`src/config_builder.rs:64-117`). Render block
    /// mirrors the predicates from `Args::resolve_render_config` /
    /// `to_render_art_defaults` (`src/cli.rs:2064-2167`).
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
            // SIM block: verbatim from ConfigBuilder::from_args (src/config_builder.rs:64-117).
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
        })
    }

    /// Resolve to a concrete `Profile`. Byte-identical to the legacy
    /// `ConfigBuilder::assemble()` + `Args::resolve_render_config()`.
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

    /// Verbatim port of `ConfigBuilder::assemble` (`src/config_builder.rs:121-350`).
    /// Same order, same special cases. Only `self.<field>` access changes.
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

    /// GUARD: TUI uses Profile.render (resolve_render); headless uses
    /// Args::to_render_art_defaults. Both run live until Phase C collapses them —
    /// they MUST stay equal.
    ///
    /// This is NOT an oracle/port-gate — it is a permanent divergence guard for the
    /// two-body duplication that exists while both paths are in production.
    #[test]
    fn render_two_live_bodies_stay_equivalent() {
        let cases: &[&[&str]] = &[
            &[],
            &["--preset", "lumen"],
            &["--preset", "etching"],
            &["--palette", "heat", "--braille"],
            &["--temporal-color", "0.6", "--temporal-mode", "accent"],
            &["--afterglow", "0.4", "--palette-shift", "8"],
            // glyph-edge-threshold on a preset whose base differs from the CLI value —
            // regression guard for the "equal-to-GlyphConfig::default()" silent-drop bug.
            &["--preset", "etching", "--glyph-edge-threshold", "0.15"],
        ];
        for c in cases {
            let a = args(c);
            let got = ProfileOverrides::from_args(&a)
                .and_then(|o| o.resolve())
                .expect("resolve");
            let want = a.resolve_render_config().expect("render");
            assert_eq!(got.render, want, "render two-body divergence for {c:?}");
        }
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
}
