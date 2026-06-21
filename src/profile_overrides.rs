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
use crate::render::charset::{Charset, GlyphConfig};
use crate::render::palette::{IntensityMapping, Palette, PaletteCycle, RgbColor, TemporalMode};
use crate::render_art_defaults::ResolvedRenderConfig;
use crate::simulation::config::{
    Aspect, BoundaryMode, ChromeStyle, DepositCurve, DiffusionKernel, Preset, SimConfig,
    TerminalSizeThreshold, WindowFrame, WindowPadding,
};

/// The single all-`Option` authored partial (sim ⊕ render ⊕ seed). Sim fields
/// mirror the former `ConfigBuilder`; render fields mirror what
/// `Args::resolve_render_config` reads.
// Task 1: intentionally unused until Task 2 wires it into the live path.
#[allow(dead_code)]
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
    pub glyph: Option<GlyphConfig>,
    pub temporal_color: Option<f32>,
    pub temporal_lag_frames: Option<f32>,
    pub temporal_mode: Option<TemporalMode>,
    pub temporal_accent: Option<RgbColor>,
    pub afterglow: Option<f32>,
    pub afterglow_rate: Option<f32>,
}

// Task 1: impl intentionally unused until Task 2 wires it into the live path.
#[allow(dead_code)]
impl ProfileOverrides {
    /// Builds a `ProfileOverrides` from CLI args. Sim block is a verbatim port of
    /// `ConfigBuilder::from_args` (`src/config_builder.rs:64-117`). Render block
    /// mirrors the predicates from `Args::resolve_render_config` /
    /// `to_render_art_defaults` (`src/cli.rs:2064-2167`).
    pub(crate) fn from_args(args: &Args) -> Self {
        // Render: resolve glyph from CLI if either glyph flag is set, as in
        // to_render_art_defaults (src/cli.rs:2082-2084).
        let glyph = if args.glyph_selection.is_some() || args.glyph_edge_threshold.is_some() {
            // Parse the glyph config using the same function as the original.
            // Use the default glyph as the base (same as to_render_art_defaults:
            // art.glyph is the preset's glyph, but at from_args time we don't
            // know the preset art yet — the merge happens in resolve_render).
            // We store the CLI-authored glyph overrides here so resolve_render
            // can apply them as the original does.
            args.glyph_config_parsed(GlyphConfig::default()).ok()
        } else {
            None
        };

        // Render: temporal_mode parsed from string, mirroring to_render_art_defaults.
        let temporal_mode =
            args.temporal_mode
                .as_ref()
                .map(|m| match m.to_ascii_lowercase().as_str() {
                    "accent" => TemporalMode::Accent,
                    _ => TemporalMode::Hue,
                });

        // Render: temporal_accent parsed from hex string.
        let temporal_accent = args.temporal_accent.as_ref().and_then(|hex| {
            u32::from_str_radix(hex.trim_start_matches('#'), 16)
                .ok()
                .map(RgbColor::from_hex)
        });

        // Render: palette — Some only when explicitly set on CLI.
        let palette = if args.palette_explicitly_set() {
            args.palette().ok()
        } else {
            None
        };

        // Render: palette_cycle — Some when either palette_cycles or palette_cycle_mode is set.
        let palette_cycle = if args.palette_cycles.is_some() || args.palette_cycle_mode.is_some() {
            // Build the PaletteCycle from CLI flags, same as to_render_art_defaults.
            let mut pc = PaletteCycle::default();
            if let Some(n) = args.palette_cycles {
                pc.cycles = n;
            }
            if let Some(mode) = args.palette_cycle_mode_parsed().ok().flatten() {
                pc.mode = mode;
            }
            Some(pc)
        } else {
            None
        };

        // Render: intensity_mapping — Some when explicitly set on CLI.
        let intensity_mapping = if args.intensity_mapping.is_some() {
            args.intensity_mapping().ok()
        } else {
            None
        };

        Self {
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
            glyph,
            temporal_color: args.temporal_color,
            temporal_lag_frames: args.temporal_lag,
            temporal_mode,
            temporal_accent,
            afterglow: args.afterglow,
            afterglow_rate: args.afterglow_rate,
        }
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
        if let Some(ref glyph) = self.glyph {
            // Merge with preset glyph as base (same as glyph_config_parsed does).
            if glyph.selection.is_some() {
                art.glyph.selection = glyph.selection;
            }
            if glyph.edge_threshold != GlyphConfig::default().edge_threshold
                || self.glyph.as_ref().map(|g| g.edge_threshold)
                    != Some(GlyphConfig::default().edge_threshold)
            {
                // Always propagate the parsed edge_threshold from CLI.
                art.glyph.edge_threshold = glyph.edge_threshold;
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
            art.palette.unwrap_or(crate::cli::Palette::Moss)
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config_builder::ConfigBuilder;
    use crate::simulation::config::PRESETS;
    use clap::Parser;

    fn args(extra: &[&str]) -> Args {
        let mut v = vec!["tslime"];
        v.extend_from_slice(extra);
        Args::parse_from(v)
    }

    /// resolve_sim == legacy assemble for default args.
    #[test]
    fn sim_parity_default() {
        let a = args(&[]);
        let got = ProfileOverrides::from_args(&a).resolve().expect("resolve");
        let want = ConfigBuilder::from_args(&a).assemble().expect("assemble");
        assert_eq!(got.sim, want);
    }

    /// resolve_sim == legacy assemble for every preset.
    #[test]
    fn sim_parity_every_preset() {
        for spec in PRESETS {
            let a = args(&["--preset", spec.name]);
            let got = ProfileOverrides::from_args(&a).resolve().expect("resolve");
            let want = ConfigBuilder::from_args(&a).assemble().expect("assemble");
            assert_eq!(got.sim, want, "sim parity broke for {}", spec.name);
        }
    }

    /// resolve_sim == legacy assemble across a CLI-override matrix (hits the special cases).
    #[test]
    fn sim_parity_override_matrix() {
        let cases: &[&[&str]] = &[
            &["--sensor-angle", "30", "--population", "10000"],
            &["--preset", "organic", "--sensor-angle", "15"],
            &["--fps", "60"], // high-FPS Gaussian opt
            &["--preset", "ripple", "--boundary-mode", "bounce"],
            &["--attract", "5,5,1.0"],
            &["--brightness", "2.0"], // gain→white_point
            &["--terrain", "smooth", "--terrain-strength", "0.5"],
            &["--preset", "river", "--wind", "1,0"],
            &["--respawn-interval", "120"],
            &["--decay-gamma", "0.3", "--diffuse-weight", "0.5"],
        ];
        for c in cases {
            let a = args(c);
            let got = ProfileOverrides::from_args(&a).resolve().expect("resolve");
            let want = ConfigBuilder::from_args(&a).assemble().expect("assemble");
            assert_eq!(got.sim, want, "sim parity broke for {c:?}");
        }
    }

    /// resolve_render == legacy resolve_render_config (default + preset + CLI overrides).
    #[test]
    fn render_parity_matrix() {
        let cases: &[&[&str]] = &[
            &[],
            &["--preset", "lumen"],   // art-on showcase preset
            &["--preset", "etching"], // charset + glyph
            &["--palette", "heat", "--braille"],
            &["--temporal-color", "0.6", "--temporal-mode", "accent"],
            &["--afterglow", "0.4", "--palette-shift", "8"],
        ];
        for c in cases {
            let a = args(c);
            let got = ProfileOverrides::from_args(&a).resolve().expect("resolve");
            let want = a.resolve_render_config().expect("render");
            assert_eq!(got.render, want, "render parity broke for {c:?}");
        }
    }

    /// Validation parity: invalid sensor_angle must be rejected (Phase A CRITICAL).
    #[test]
    fn resolve_rejects_invalid_sensor_angle() {
        let a = args(&["--sensor-angle", "999"]);
        assert!(ProfileOverrides::from_args(&a).resolve().is_err());
    }

    /// Seed passthrough.
    #[test]
    fn seed_passthrough() {
        assert_eq!(
            ProfileOverrides::from_args(&args(&[]))
                .resolve()
                .unwrap()
                .seed,
            None
        );
        assert_eq!(
            ProfileOverrides::from_args(&args(&["--seed", "7"]))
                .resolve()
                .unwrap()
                .seed,
            Some(7)
        );
    }
}
