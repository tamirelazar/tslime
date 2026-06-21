use crate::profile_overrides::ProfileOverrides;
use crate::render::charset::{Charset, GlyphConfig, ALL_CHARSETS};
use crate::render::palette::{IntensityMapping, Palette, PaletteCycle, TemporalMode, PALETTES};
use crate::simulation::config::SimConfig;
use crate::terminal::control::RuntimeState;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

const CONFIG_DIR: &str = ".config/tslime";
const CONFIG_FILE: &str = "presets.toml";

/// A named, persisted profile: human-readable identity + all-optional overrides.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NamedProfile {
    /// Name of the saved config.
    pub name: String,
    /// Optional description.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// All overrides, flattened into the TOML table (so fields appear at top level).
    #[serde(flatten)]
    pub overrides: ProfileOverrides,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ConfigFile {
    #[serde(rename = "preset")]
    presets: Vec<NamedProfile>,
}

/// Build a `ProfileOverrides` from live runtime state for config-save.
///
/// Sources each lever from the SAME place the old `SavedConfig::from_runtime` did:
/// - sim levers from `sim_config` (as-resolved)
/// - render/art levers from `runtime_state` (live session values)
/// - reverse/invert/food_persist from startup `args.*` (preserving the
///   known latent bug that saving captures startup args, not live toggles;
///   Phase C fixes this at the apply seam)
///
/// Signature: capture_overrides(sim, palette, charset, rs, reverse, invert, food_persist)
pub fn capture_overrides(
    sim_config: &SimConfig,
    palette: Palette,
    charset: Charset,
    rs: &RuntimeState,
    reverse: bool,
    invert: bool,
    food_persist: bool,
) -> ProfileOverrides {
    // Convert sim.max_brightness (white-point) back to user-facing brightness gain.
    let brightness_gain = crate::config_defaults::trail::brightness_gain(sim_config.max_brightness);

    // Temporal accent: convert Option<RgbColor> to the typed field.
    let temporal_accent = rs.temporal_accent;

    // palette_cycle: convert from the live PaletteCycle struct.
    let palette_cycle = if rs.palette_cycle.is_identity() {
        None
    } else {
        Some(rs.palette_cycle)
    };

    // glyph: convert GlyphConfig to the typed Option<GlyphSelection> + threshold.
    let (glyph_selection, glyph_edge_threshold) = match rs.glyph.selection {
        None => (None, None),
        Some(sel) => (Some(sel), Some(rs.glyph.edge_threshold)),
    };

    // color_aa: convert the fixed array to Option<AaStrength> via the single
    // scalar (ProfileOverrides.color_aa is Option<AaStrength>, not a Vec).
    // The old code saved a Vec<String> per-charset; the new ProfileOverrides has
    // a single AaStrength scalar (the active charset's setting).  We save the
    // active charset's value — apply reads it back into all slots defensively as
    // before.  This is a slight simplification from the old per-slot vec; see
    // Phase C for a full per-slot upgrade.
    let color_aa = Some(rs.color_aa[rs.charset_index % rs.color_aa.len()]);

    // intensity_mapping: save the live session mapping.
    let intensity_mapping = Some(rs.intensity_mapping.clone());

    ProfileOverrides {
        // provenance — not set on save (these would only appear in preset TOML)
        preset: None,
        seed: None,

        // sim levers (sourced from sim_config, same as old from_runtime)
        sensor_angle: Some(sim_config.sensor_angle),
        sensor_distance: Some(sim_config.sensor_distance),
        rotation_angle: Some(sim_config.rotation_angle),
        step_size: Some(sim_config.step_size),
        decay_factor: Some(sim_config.decay_factor),
        deposit_amount: Some(sim_config.deposit_amount),
        brightness: Some(brightness_gain),
        diffusion_kernel: Some(sim_config.diffusion_kernel),
        diffusion_sigma: Some(sim_config.diffusion_sigma),
        // sim levers not round-trippable via apply (restart-only) — drop per Phase B spec
        time_scale: None,
        // population: capture for display (config-browser "Nk agents") only; NOT applied by
        // apply_to_runtime_state (restart-only lever). Mirrors old from_runtime: first_species.count.
        population: Some(sim_config.total_population()),
        fps: None,
        food_image_path: None,
        food_image_invert: None,
        food_image_scale: None,
        attractors: Vec::new(),
        attractor_strength: None,
        obstacles: Vec::new(),
        species: Vec::new(),
        separate_species_trails: false,
        species_colors: false,
        use_simd: None,
        wind: None,
        terrain: None,
        terrain_strength: None,
        background_color: sim_config.background_color.clone(),
        boundary_mode: None,
        window_frame: Some(sim_config.window_frame),
        chrome_style: Some(sim_config.chrome_style),
        aspect: Some(sim_config.aspect),
        window_padding: Some(sim_config.window_padding),
        show_status_bar: Some(sim_config.show_status_bar),
        min_sim_size: Some(sim_config.min_sim_size),
        min_frame_size: Some(sim_config.min_frame_size),
        respawn_interval: None,
        decay_gamma: Some(rs.decay_gamma),
        diffuse_weight: Some(rs.diffuse_weight),
        deposit_curve: Some(rs.deposit_curve),
        deposit_scale: Some(rs.deposit_scale),
        deposit_gamma: Some(rs.deposit_gamma),
        deposit_cap: Some(rs.deposit_cap),

        // render levers (sourced from runtime_state, same as old from_runtime)
        palette: Some(palette),
        charset: Some(charset),
        color_aa,
        hue_shift: None,
        intensity_mapping,
        palette_cycle,
        glyph_selection,
        glyph_edge_threshold,
        temporal_color: Some(rs.temporal_color),
        temporal_lag_frames: Some(rs.temporal_lag_frames),
        temporal_mode: Some(rs.temporal_mode),
        temporal_accent,
        afterglow: Some(rs.afterglow),
        afterglow_rate: Some(rs.afterglow_rate),

        // apply/persistence-only levers (sourced from startup args, per Phase B spec)
        reverse_palette: Some(reverse),
        invert_palette: Some(invert),
        food_persist: Some(food_persist),
        // Full per-charset AA array — mirrors old from_runtime which stored the whole array.
        color_aa_all: Some(rs.color_aa.to_vec()),

        // app-runtime levers — the live source `rs.app` now exists (Task 3 wired the
        // runner to read it). Capturing these from `rs.app` into saved configs is
        // formally Task 5's scope (capture-from-live); left None here so save output
        // and the serde/snapshot round-trip tests stay stable until then.
        // TODO(task-5): capture from rs.app.* (warmup/auto_reset/grid/food_persist).
        init_mode: None,
        warmup_frames: None,
        skip_warmup: None,
        warmup_brightness_multiplier: None,
        auto_reset: None,
        auto_reset_entropy_threshold: None,
        auto_reset_duration_frames: None,
        grid: None,
        grid_style: None,
        grid_size: None,
        grid_color: None,
        grid_opacity: None,
        grid_adaptive: None,
        food_persist_strength: None,
        food_persist_radius: None,
        food_persist_duration: None,
    }
}

/// Apply a `ProfileOverrides` to `RuntimeState`.
///
/// Port of the former `SavedConfig::apply_to_runtime_state` logic.
/// Writes the SAME `RuntimeState` fields with the SAME `unwrap_or` defaults.
/// The typed `ProfileOverrides` fields replace old string-parse helpers:
/// `Option<Palette>` → palette_index via PALETTES registry, `Option<Charset>`
/// → charset_index via `ALL_CHARSETS`, `Option<DiffusionKernel>` direct.
pub fn apply_to_runtime_state(
    overrides: &ProfileOverrides,
    runtime_state: &mut RuntimeState,
) -> Result<(), String> {
    // Apply palette via PALETTES registry (typed, no string parse).
    if let Some(ref p) = overrides.palette {
        let idx = PALETTES
            .iter()
            .position(|spec| &spec.palette == p)
            .ok_or_else(|| format!("Unknown palette: {:?}", p))?;
        runtime_state.palette_index = idx;
    }
    // reverse/invert palette
    runtime_state.reverse_palette = overrides.reverse_palette.unwrap_or(false);
    runtime_state.invert_palette = overrides.invert_palette.unwrap_or(false);

    // Apply charset via ALL_CHARSETS (typed, no string parse).
    if let Some(ref c) = overrides.charset {
        let idx = ALL_CHARSETS.iter().position(|cs| cs == c).unwrap_or(0); // default: HalfBlock at index 0
        runtime_state.charset_index = idx;
    }

    // Apply diffusion kernel (typed Option<DiffusionKernel> — no string parse needed).
    if let Some(kernel) = overrides.diffusion_kernel {
        runtime_state.diffusion_kernel = kernel;
    }
    // diffusion_sigma: apply if present (same as old path — it was always present in from_runtime)
    if let Some(sigma) = overrides.diffusion_sigma {
        runtime_state.diffusion_sigma = sigma;
    }

    // Apply sim parameters (all sourced from sim_config in capture_overrides).
    if let Some(v) = overrides.sensor_angle {
        runtime_state.sensor_angle = v;
    }
    if let Some(v) = overrides.sensor_distance {
        runtime_state.sensor_distance = v;
    }
    if let Some(v) = overrides.rotation_angle {
        runtime_state.rotation_angle = v;
    }
    if let Some(v) = overrides.step_size {
        runtime_state.step_size = v;
    }
    if let Some(v) = overrides.decay_factor {
        runtime_state.decay_factor = v;
    }
    if let Some(v) = overrides.deposit_amount {
        runtime_state.deposit_amount = v;
    }
    // brightness is stored as gain; apply via max_brightness (white-point).
    if let Some(gain) = overrides.brightness {
        runtime_state.max_brightness = crate::config_defaults::trail::white_point_from_gain(gain);
    }

    // Apply window frame (typed Option<WindowFrame> — no string parse needed).
    if let Some(wf) = overrides.window_frame {
        runtime_state.window_frame = wf;
    } else {
        // Old default path: parse_window_frame("") → unwrap_or_default()
        runtime_state.window_frame = Default::default();
    }

    // Apply window mode chrome / layout fields.
    if let Some(cs) = overrides.chrome_style {
        runtime_state.chrome_style = cs;
    } else {
        runtime_state.chrome_style = Default::default();
    }
    if let Some(a) = overrides.aspect {
        runtime_state.aspect = a;
    } else {
        runtime_state.aspect = Default::default();
    }
    if let Some(p) = overrides.window_padding {
        runtime_state.window_padding = p;
    } else {
        runtime_state.window_padding = Default::default();
    }
    if let Some(ssb) = overrides.show_status_bar {
        runtime_state.show_status_bar = ssb;
    } else {
        runtime_state.show_status_bar = false;
    }
    if let Some(mss) = overrides.min_sim_size {
        runtime_state.min_sim_size = mss;
    } else {
        runtime_state.min_sim_size = Default::default();
    }
    if let Some(mfs) = overrides.min_frame_size {
        runtime_state.min_frame_size = mfs;
    } else {
        runtime_state.min_frame_size = crate::simulation::config::TerminalSizeThreshold {
            width: 12,
            height: 6,
        };
    }

    // Apply food persistence setting.
    runtime_state.food_persist_enabled = overrides.food_persist.unwrap_or(false);

    // Reset warmup so the changes can be seen.
    runtime_state.warmup_counter = 0;

    // Apply intensity mapping if present.
    if let Some(ref mapping) = overrides.intensity_mapping {
        runtime_state.intensity_mapping_index = RuntimeState::find_intensity_mapping_index(mapping);
        runtime_state.intensity_mapping = mapping.clone();
    } else {
        // No mapping recorded — reset to canonical default (logarithmic) so load
        // fully restores state rather than inheriting the session's.
        runtime_state.intensity_mapping = IntensityMapping::default();
        runtime_state.intensity_mapping_index =
            RuntimeState::find_intensity_mapping_index(&runtime_state.intensity_mapping);
    }

    // Apply temporal color fields.
    runtime_state.temporal_color = overrides.temporal_color.unwrap_or(0.0);
    runtime_state.temporal_lag_frames = overrides.temporal_lag_frames.unwrap_or(8.0);
    runtime_state.temporal_mode = overrides.temporal_mode.unwrap_or(TemporalMode::Hue);

    // Apply afterglow fields.
    runtime_state.afterglow = overrides.afterglow.unwrap_or(0.0);
    runtime_state.afterglow_rate = overrides.afterglow_rate.unwrap_or(0.05);

    // Apply decay gamma.
    runtime_state.decay_gamma = overrides.decay_gamma.unwrap_or(1.0);

    // Apply diffuse weight.
    runtime_state.diffuse_weight = overrides.diffuse_weight.unwrap_or(1.0);

    // Apply deposit knobs.
    runtime_state.deposit_curve = overrides.deposit_curve.unwrap_or_default();
    runtime_state.deposit_scale = overrides.deposit_scale.unwrap_or(1.0);
    runtime_state.deposit_gamma = overrides.deposit_gamma.unwrap_or(1.0);
    runtime_state.deposit_cap = overrides.deposit_cap.unwrap_or(0.0);

    // Apply palette cycles.
    {
        use crate::render::palette::PaletteCycleMode;
        if let Some(pc) = overrides.palette_cycle {
            runtime_state.palette_cycle = pc;
        } else {
            // Old default path: cycles=1, mode=default
            runtime_state.palette_cycle = PaletteCycle {
                cycles: 1,
                mode: PaletteCycleMode::default(),
            };
        }
    }

    // Apply glyph-by-shape config (typed GlyphSelection — no string parse needed).
    runtime_state.glyph = match overrides.glyph_selection {
        Some(sel) => GlyphConfig {
            selection: Some(sel),
            edge_threshold: overrides
                .glyph_edge_threshold
                .unwrap_or(crate::config_defaults::glyph_consts::DEFAULT_GLYPH_EDGE_THRESHOLD),
        },
        None => GlyphConfig::default(),
    };

    // Apply temporal accent color (typed Option<RgbColor> — no hex parse needed).
    runtime_state.temporal_accent = overrides.temporal_accent;

    // Apply per-charset color-AA. When color_aa_all is present (new format), restore
    // all slots from the Vec (matches old from_runtime/apply_to_runtime_state behavior).
    // Fall back to the scalar color_aa for the active charset only (back-compat with
    // configs saved before color_aa_all was introduced).
    if let Some(ref arr) = overrides.color_aa_all {
        for (i, aa) in arr.iter().enumerate() {
            if i < runtime_state.color_aa.len() {
                runtime_state.color_aa[i] = *aa;
            }
        }
    } else if let Some(aa) = overrides.color_aa {
        let i = runtime_state.charset_index % runtime_state.color_aa.len();
        runtime_state.color_aa[i] = aa;
    }

    // Phase C: apply seam extends this (restart-only levers: population, init_mode,
    // food_path, auto_reset, grid, grid_style, warmup_frames).

    Ok(())
}

/// Returns the path to the configuration file.
///
/// Creates the config directory if it doesn't exist.
pub fn get_config_path() -> Result<PathBuf, String> {
    let home = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .map_err(|_| "Could not determine home directory".to_string())?;

    let config_dir = PathBuf::from(home).join(CONFIG_DIR);

    if !config_dir.exists() {
        fs::create_dir_all(&config_dir)
            .map_err(|e| format!("Failed to create config directory: {}", e))?;
    }

    Ok(config_dir.join(CONFIG_FILE))
}

fn load_config_file() -> Result<ConfigFile, String> {
    let path = get_config_path()?;

    if !path.exists() {
        return Ok(ConfigFile {
            presets: Vec::new(),
        });
    }

    let contents =
        fs::read_to_string(&path).map_err(|e| format!("Failed to read config file: {}", e))?;

    toml::from_str(&contents).map_err(|e| format!("Failed to parse config file: {}", e))
}

fn save_config_file(config_file: &ConfigFile) -> Result<(), String> {
    let path = get_config_path()?;

    let toml_string = toml::to_string_pretty(config_file)
        .map_err(|e| format!("Failed to serialize config: {}", e))?;

    fs::write(&path, toml_string).map_err(|e| format!("Failed to write config file: {}", e))?;

    Ok(())
}

/// Saves a `NamedProfile` to the config file.
///
/// Overwrites any existing configuration with the same name.
pub fn save_config(profile: NamedProfile) -> Result<(), String> {
    let mut config_file = load_config_file()?;

    config_file.presets.retain(|c| c.name != profile.name);
    config_file.presets.push(profile);

    save_config_file(&config_file)
}

/// Loads a saved configuration by name.
pub fn load_config(name: &str) -> Result<NamedProfile, String> {
    let config_file = load_config_file()?;

    config_file
        .presets
        .iter()
        .find(|c| c.name == name)
        .cloned()
        .ok_or_else(|| format!("Config '{}' not found", name))
}

/// Lists all saved configurations.
pub fn list_configs() -> Result<Vec<NamedProfile>, String> {
    let config_file = load_config_file()?;
    Ok(config_file.presets)
}

/// Deletes a configuration by name.
pub fn delete_config(name: &str) -> Result<(), String> {
    let mut config_file = load_config_file()?;

    let original_len = config_file.presets.len();
    config_file.presets.retain(|c| c.name != name);

    if config_file.presets.len() == original_len {
        return Err(format!("Config '{}' not found", name));
    }

    save_config_file(&config_file)
}

// ── Helper types referenced in tests (kept for type coherence) ──────────────
// These are NOT used in production code; they exist only so tests can reference
// the typed structs imported above without additional use-statements.
#[allow(unused_imports)]
use crate::render::charset::GlyphSelection;
#[allow(unused_imports)]
use crate::render::palette::PaletteCycleMode;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::PauseStyle;
    use crate::config_defaults::trail;
    use crate::render::charset::Charset;
    use crate::render::palette::{Palette, PaletteCycle, PaletteCycleMode};
    use crate::simulation::config::{DepositCurve, DiffusionKernel, Preset, SimConfig};
    use crate::terminal::control::RuntimeState;

    fn create_test_runtime_state() -> RuntimeState {
        RuntimeState::new(
            42,
            crate::simulation::config::InitMode::Random,
            Preset::Organic,
            crate::terminal::control::MouseInteractionMode::Disabled,
            3.0,
            &SimConfig::default(),
            PauseStyle::Vignette,
            false,
            false,
        )
    }

    /// Build a minimal ProfileOverrides with required sim fields populated —
    /// mirrors a config written by capture_overrides from default sim state.
    fn minimal_overrides(name: &str) -> NamedProfile {
        let sim = SimConfig::default();
        let rs = create_test_runtime_state();
        NamedProfile {
            name: name.to_string(),
            description: None,
            overrides: capture_overrides(
                &sim,
                Palette::Organic,
                Charset::HalfBlock,
                &rs,
                false,
                false,
                false,
            ),
        }
    }

    #[test]
    fn test_named_profile_serialization() {
        let profile = minimal_overrides("test");
        let toml_str = toml::to_string(&profile).unwrap();
        let deserialized: NamedProfile = toml::from_str(&toml_str).unwrap();

        assert_eq!(profile.name, deserialized.name);
        assert_eq!(profile.overrides.palette, deserialized.overrides.palette);
    }

    #[test]
    fn test_config_file_serialization_roundtrip() {
        let profile = minimal_overrides("roundtrip");
        let config_file = ConfigFile {
            presets: vec![profile.clone()],
        };
        let toml_str = toml::to_string_pretty(&config_file).unwrap();
        let deserialized: ConfigFile = toml::from_str(&toml_str).unwrap();
        assert_eq!(deserialized.presets.len(), 1);
        assert_eq!(deserialized.presets[0].name, "roundtrip");
    }

    #[test]
    fn test_apply_palette_to_runtime_state() {
        let sim = SimConfig::default();
        let rs = create_test_runtime_state();
        let profile = NamedProfile {
            name: "test_palette".to_string(),
            description: None,
            overrides: capture_overrides(
                &sim,
                Palette::Heat,
                Charset::HalfBlock,
                &rs,
                false,
                false,
                false,
            ),
        };

        let mut new_state = create_test_runtime_state();
        apply_to_runtime_state(&profile.overrides, &mut new_state).unwrap();

        // Heat should be index 1 in PALETTES
        let heat_idx = PALETTES
            .iter()
            .position(|s| s.palette == Palette::Heat)
            .unwrap();
        assert_eq!(new_state.palette_index, heat_idx);
    }

    #[test]
    fn apply_to_runtime_state_sets_diffusion_sigma() {
        let sim = SimConfig {
            diffusion_sigma: 2.75,
            ..SimConfig::default()
        };
        let rs = create_test_runtime_state();

        let profile = NamedProfile {
            name: "test_sigma".to_string(),
            description: None,
            overrides: capture_overrides(
                &sim,
                Palette::Heat,
                Charset::HalfBlock,
                &rs,
                false,
                false,
                false,
            ),
        };

        let mut new_state = create_test_runtime_state();
        apply_to_runtime_state(&profile.overrides, &mut new_state)
            .expect("apply_to_runtime_state must succeed");
        assert!((new_state.diffusion_sigma - 2.75).abs() < 1e-6);
    }

    #[test]
    fn apply_to_runtime_state_resets_intensity_mapping_when_none() {
        use crate::render::palette::IntensityMapping;
        let mut rs = create_test_runtime_state();
        // Put a non-default mapping into the session first.
        rs.intensity_mapping = IntensityMapping::linear();
        rs.intensity_mapping_index =
            RuntimeState::find_intensity_mapping_index(&rs.intensity_mapping);

        // Build overrides without intensity_mapping (simulate old-format config).
        let mut overrides = capture_overrides(
            &SimConfig::default(),
            Palette::Heat,
            Charset::HalfBlock,
            &rs,
            false,
            false,
            false,
        );
        overrides.intensity_mapping = None;

        let mut new_state = create_test_runtime_state();
        apply_to_runtime_state(&overrides, &mut new_state)
            .expect("apply_to_runtime_state must succeed");

        // Default is logarithmic(10.0); a saved config with no recorded mapping
        // must reset the session to the canonical default, not inherit linear.
        assert_eq!(new_state.intensity_mapping, IntensityMapping::default());
        assert_eq!(
            new_state.intensity_mapping_index,
            RuntimeState::find_intensity_mapping_index(&IntensityMapping::default())
        );
    }

    #[test]
    fn test_full_config_roundtrip() {
        let mut state = create_test_runtime_state();

        // Modify state to have specific values
        state.palette_index = 5; // neon
        state.reverse_palette = true;
        state.invert_palette = true;
        state.sensor_angle = 35.0;
        state.rotation_angle = 55.0;
        state.step_size = 1.5;
        state.decay_factor = 0.92;
        state.deposit_amount = 6.5;
        state.max_brightness = 30.0;
        state.diffusion_kernel = DiffusionKernel::Gaussian;

        let sim_config = SimConfig {
            sensor_angle: state.sensor_angle,
            sensor_distance: 9.0,
            rotation_angle: state.rotation_angle,
            step_size: state.step_size,
            decay_factor: state.decay_factor,
            deposit_amount: state.deposit_amount,
            diffusion_kernel: state.diffusion_kernel,
            diffusion_sigma: 1.0,
            diffuse_weight: trail::DEFAULT_DIFFUSE_WEIGHT,
            decay_gamma: trail::DEFAULT_DECAY_GAMMA,
            deposit_curve: DepositCurve::default(),
            deposit_scale: trail::DEFAULT_DEPOSIT_SCALE,
            deposit_gamma: trail::DEFAULT_DEPOSIT_GAMMA,
            deposit_cap: trail::DEFAULT_DEPOSIT_CAP,
            max_brightness: state.max_brightness,
            ..SimConfig::default()
        };

        let overrides = capture_overrides(
            &sim_config,
            Palette::Neon,
            Charset::HalfBlock,
            &state,
            state.reverse_palette,
            state.invert_palette,
            false,
        );

        // Create new state and apply config
        let mut new_state = create_test_runtime_state();
        apply_to_runtime_state(&overrides, &mut new_state).unwrap();

        // Verify all values match
        let neon_idx = PALETTES
            .iter()
            .position(|s| s.palette == Palette::Neon)
            .unwrap();
        assert_eq!(new_state.palette_index, neon_idx);
        assert_eq!(new_state.reverse_palette, state.reverse_palette);
        assert_eq!(new_state.invert_palette, state.invert_palette);
        assert_eq!(new_state.sensor_angle, state.sensor_angle);
        assert_eq!(new_state.rotation_angle, state.rotation_angle);
        assert_eq!(new_state.step_size, state.step_size);
        assert_eq!(new_state.decay_factor, state.decay_factor);
        assert_eq!(new_state.deposit_amount, state.deposit_amount);
        assert_eq!(new_state.max_brightness, state.max_brightness);
        assert_eq!(new_state.diffusion_kernel, state.diffusion_kernel);
    }

    #[test]
    fn temporal_fields_round_trip_and_default_off() {
        // Minimal overrides with no temporal fields set → temporal_color defaults to 0.0.
        let mut overrides = capture_overrides(
            &SimConfig::default(),
            Palette::Organic,
            Charset::HalfBlock,
            &create_test_runtime_state(),
            false,
            false,
            false,
        );
        // Simulate an old-format config that had no temporal field.
        overrides.temporal_color = None;

        let mut rs = create_test_runtime_state();
        apply_to_runtime_state(&overrides, &mut rs).expect("apply must succeed");
        assert_eq!(rs.temporal_color, 0.0);
    }

    #[test]
    fn temporal_fields_full_round_trip() {
        use crate::render::palette::TemporalMode;

        let mut state = create_test_runtime_state();
        state.temporal_color = 0.7;
        state.temporal_lag_frames = 12.0;
        state.temporal_mode = TemporalMode::Accent;

        let overrides = capture_overrides(
            &SimConfig::default(),
            Palette::Organic,
            Charset::HalfBlock,
            &state,
            false,
            false,
            false,
        );

        // Serialize and deserialize through TOML
        let toml_str = toml::to_string(&overrides).expect("serialize must succeed");
        let reloaded: ProfileOverrides =
            toml::from_str(&toml_str).expect("deserialize must succeed");

        // Restore into a fresh RuntimeState
        let mut new_state = create_test_runtime_state();
        apply_to_runtime_state(&reloaded, &mut new_state).expect("apply must succeed");

        assert!((new_state.temporal_color - 0.7).abs() < 1e-6);
        assert!((new_state.temporal_lag_frames - 12.0).abs() < 1e-6);
        assert_eq!(new_state.temporal_mode, TemporalMode::Accent);
    }

    #[test]
    fn diffusion_decay_art_knobs_round_trip() {
        let mut state = create_test_runtime_state();
        state.afterglow = 0.4;
        state.afterglow_rate = 0.03;
        state.decay_gamma = 0.6;
        state.diffuse_weight = 0.5;
        state.diffusion_sigma = 3.0;

        let sim = SimConfig {
            diffusion_sigma: state.diffusion_sigma,
            diffuse_weight: state.diffuse_weight,
            decay_gamma: state.decay_gamma,
            ..SimConfig::default()
        };

        let overrides = capture_overrides(
            &sim,
            Palette::Organic,
            Charset::HalfBlock,
            &state,
            false,
            false,
            false,
        );

        // Serialize and deserialize through TOML
        let toml_str = toml::to_string(&overrides).expect("serialize must succeed");
        let reloaded: ProfileOverrides =
            toml::from_str(&toml_str).expect("deserialize must succeed");

        // Restore into a fresh RuntimeState
        let mut new_state = create_test_runtime_state();
        apply_to_runtime_state(&reloaded, &mut new_state).expect("apply must succeed");

        assert!(
            (new_state.afterglow - 0.4).abs() < 1e-6,
            "afterglow must survive round-trip (got {})",
            new_state.afterglow
        );
        assert!(
            (new_state.afterglow_rate - 0.03).abs() < 1e-6,
            "afterglow_rate must survive round-trip (got {})",
            new_state.afterglow_rate
        );
        assert!(
            (new_state.decay_gamma - 0.6).abs() < 1e-6,
            "decay_gamma must survive round-trip (got {})",
            new_state.decay_gamma
        );
        assert!(
            (new_state.diffuse_weight - 0.5).abs() < 1e-6,
            "diffuse_weight must survive round-trip (got {})",
            new_state.diffuse_weight
        );
        assert!(
            (new_state.diffusion_sigma - 3.0).abs() < 1e-6,
            "diffusion_sigma must survive round-trip (got {})",
            new_state.diffusion_sigma
        );
    }

    #[test]
    fn old_toml_without_art_knobs_loads_with_defaults() {
        // A minimal TOML (palette + charset typed via serde) must parse and
        // produce canonical defaults when applied.
        let toml = r#"
palette = "organic"
charset = "halfblock"
"#;
        let cfg: ProfileOverrides = toml::from_str(toml).expect("minimal overrides must load");
        assert!(cfg.afterglow.is_none());
        assert!(cfg.afterglow_rate.is_none());
        assert!(cfg.decay_gamma.is_none());
        assert!(cfg.diffuse_weight.is_none());

        let mut state = create_test_runtime_state();
        apply_to_runtime_state(&cfg, &mut state).expect("legacy config must still apply");
        assert_eq!(state.afterglow, 0.0, "default afterglow must be 0.0");
        assert!(
            (state.afterglow_rate - 0.05).abs() < 1e-6,
            "default afterglow_rate must be 0.05"
        );
        assert_eq!(state.decay_gamma, 1.0, "default decay_gamma must be 1.0");
        assert_eq!(
            state.diffuse_weight, 1.0,
            "default diffuse_weight must be 1.0"
        );
        assert!(cfg.deposit_curve.is_none());
        assert!(cfg.deposit_scale.is_none());
        assert!(cfg.deposit_gamma.is_none());
        assert!(cfg.deposit_cap.is_none());
        assert_eq!(
            state.deposit_curve,
            DepositCurve::default(),
            "default deposit_curve must be Linear"
        );
        assert_eq!(
            state.deposit_scale, 1.0,
            "default deposit_scale must be 1.0"
        );
        assert_eq!(
            state.deposit_gamma, 1.0,
            "default deposit_gamma must be 1.0"
        );
        assert_eq!(state.deposit_cap, 0.0, "default deposit_cap must be 0.0");
    }

    #[test]
    fn deposit_fields_round_trip_through_saved_config() {
        let mut state = create_test_runtime_state();
        state.deposit_curve = DepositCurve::Pow;
        state.deposit_scale = 2.5;
        state.deposit_gamma = 0.5;
        state.deposit_cap = 7.0;

        let overrides = capture_overrides(
            &SimConfig::default(),
            Palette::Organic,
            Charset::HalfBlock,
            &state,
            false,
            false,
            false,
        );

        // Serialize and deserialize through TOML
        let toml_str = toml::to_string(&overrides).expect("serialize must succeed");
        let reloaded: ProfileOverrides =
            toml::from_str(&toml_str).expect("deserialize must succeed");

        // Restore into a fresh RuntimeState
        let mut restored = create_test_runtime_state();
        apply_to_runtime_state(&reloaded, &mut restored).expect("apply must succeed");

        assert_eq!(
            restored.deposit_curve,
            DepositCurve::Pow,
            "deposit_curve must survive round-trip"
        );
        assert!(
            (restored.deposit_scale - 2.5).abs() < 1e-6,
            "deposit_scale must survive round-trip (got {})",
            restored.deposit_scale
        );
        assert!(
            (restored.deposit_gamma - 0.5).abs() < 1e-6,
            "deposit_gamma must survive round-trip (got {})",
            restored.deposit_gamma
        );
        assert!(
            (restored.deposit_cap - 7.0).abs() < 1e-6,
            "deposit_cap must survive round-trip (got {})",
            restored.deposit_cap
        );
    }

    #[test]
    fn palette_cycle_round_trips_through_saved_config() {
        let mut state = create_test_runtime_state();
        state.palette_cycle = PaletteCycle {
            cycles: 4,
            mode: PaletteCycleMode::Wrap,
        };

        let overrides = capture_overrides(
            &SimConfig::default(),
            Palette::Organic,
            Charset::HalfBlock,
            &state,
            false,
            false,
            false,
        );

        assert_eq!(
            overrides.palette_cycle,
            Some(PaletteCycle {
                cycles: 4,
                mode: PaletteCycleMode::Wrap
            })
        );

        // Serialize and deserialize through TOML
        let toml_str = toml::to_string(&overrides).expect("serialize must succeed");
        let reloaded: ProfileOverrides =
            toml::from_str(&toml_str).expect("deserialize must succeed");

        // Restore into a fresh RuntimeState
        let mut rs2 = create_test_runtime_state();
        apply_to_runtime_state(&reloaded, &mut rs2).expect("apply must succeed");

        assert_eq!(
            rs2.palette_cycle,
            PaletteCycle {
                cycles: 4,
                mode: PaletteCycleMode::Wrap
            }
        );
    }

    #[test]
    fn glyph_round_trips_through_saved_config() {
        use crate::render::charset::{GlyphConfig, GlyphSelection};

        let mut rs = create_test_runtime_state();
        rs.glyph = GlyphConfig {
            selection: Some(GlyphSelection::Hybrid),
            edge_threshold: 0.25,
        };

        let overrides = capture_overrides(
            &SimConfig::default(),
            Palette::Organic,
            Charset::HalfBlock,
            &rs,
            false,
            false,
            false,
        );

        assert_eq!(overrides.glyph_selection, Some(GlyphSelection::Hybrid));
        assert_eq!(overrides.glyph_edge_threshold, Some(0.25));

        let mut rs2 = create_test_runtime_state();
        apply_to_runtime_state(&overrides, &mut rs2).unwrap();
        assert_eq!(rs2.glyph.selection, Some(GlyphSelection::Hybrid));
        assert_eq!(rs2.glyph.edge_threshold, 0.25);
    }

    #[test]
    fn glyph_identity_serializes_to_none() {
        use crate::render::charset::GlyphConfig;

        let mut rs = create_test_runtime_state();
        rs.glyph = GlyphConfig::default();

        let overrides = capture_overrides(
            &SimConfig::default(),
            Palette::Organic,
            Charset::HalfBlock,
            &rs,
            false,
            false,
            false,
        );

        assert_eq!(overrides.glyph_selection, None);
        assert_eq!(overrides.glyph_edge_threshold, None);
    }

    #[test]
    fn missing_palette_cycle_loads_identity() {
        // Minimal overrides with no palette_cycle → identity.
        let toml = r#"
palette = "organic"
charset = "halfblock"
"#;
        let cfg: ProfileOverrides =
            toml::from_str(toml).expect("config without palette_cycle must load");
        assert!(
            cfg.palette_cycle.is_none(),
            "missing key must deserialize as None"
        );

        let mut state = create_test_runtime_state();
        apply_to_runtime_state(&cfg, &mut state).expect("legacy config must still apply");
        assert!(
            state.palette_cycle.is_identity(),
            "missing palette_cycles must default to identity"
        );
        assert_eq!(
            state.palette_cycle,
            PaletteCycle::default(),
            "palette_cycle must equal default"
        );
    }

    #[test]
    fn missing_glyph_loads_identity() {
        use crate::render::charset::GlyphConfig;

        // Minimal overrides with no glyph fields → GlyphConfig::default().
        let toml = r#"
palette = "organic"
charset = "halfblock"
"#;
        let cfg: ProfileOverrides =
            toml::from_str(toml).expect("config without glyph fields must load");
        assert!(
            cfg.glyph_selection.is_none(),
            "missing glyph_selection must deserialize as None"
        );
        assert!(cfg.glyph_edge_threshold.is_none());

        let mut state = create_test_runtime_state();
        apply_to_runtime_state(&cfg, &mut state).expect("legacy config must still apply");
        assert_eq!(
            state.glyph,
            GlyphConfig::default(),
            "missing glyph keys must default to GlyphConfig::default()"
        );
        assert!(
            state.glyph.selection.is_none(),
            "missing glyph_selection must default to None"
        );
    }

    #[test]
    fn temporal_accent_round_trips_and_back_compat() {
        use crate::render::palette::RgbColor;

        // Missing field deserializes to None (old TOML — back-compat).
        let old_toml = r#"
palette = "organic"
charset = "halfblock"
"#;
        let old: ProfileOverrides = toml::from_str(old_toml).unwrap();
        assert_eq!(old.temporal_accent, None);

        // Some(color) → typed field → serialize → deserialize → apply → same color.
        let accent = RgbColor::new(0xff, 0xb3, 0x47);
        let mut rs = create_test_runtime_state();
        rs.temporal_accent = Some(accent);

        let overrides = capture_overrides(
            &SimConfig::default(),
            Palette::Organic,
            Charset::HalfBlock,
            &rs,
            false,
            false,
            false,
        );

        // Verify it's set in the overrides.
        assert_eq!(overrides.temporal_accent, Some(accent));

        // Round-trip through TOML.
        let toml_str = toml::to_string(&overrides).expect("serialize must succeed");
        let reloaded: ProfileOverrides =
            toml::from_str(&toml_str).expect("deserialize must succeed");
        let mut rs2 = create_test_runtime_state();
        apply_to_runtime_state(&reloaded, &mut rs2).expect("apply must succeed");

        assert_eq!(
            rs2.temporal_accent,
            Some(accent),
            "temporal_accent must survive round-trip"
        );
    }

    #[test]
    fn color_aa_round_trips_through_saved_config() {
        // color_aa in ProfileOverrides is a single AaStrength scalar (active charset).
        // Verify it round-trips for the active charset slot.
        use crate::render::antialiasing::AaStrength;
        let mut rs = create_test_runtime_state();
        // Set a non-default value for the active charset (index 0 = HalfBlock).
        rs.color_aa[0] = AaStrength::Subtle;

        let overrides = capture_overrides(
            &SimConfig::default(),
            Palette::Organic,
            Charset::HalfBlock,
            &rs,
            false,
            false,
            false,
        );
        // Should capture the active slot value.
        assert_eq!(overrides.color_aa, Some(AaStrength::Subtle));

        let mut rs2 = create_test_runtime_state();
        apply_to_runtime_state(&overrides, &mut rs2).expect("apply must succeed");
        assert_eq!(rs2.color_aa[0], AaStrength::Subtle);
    }

    #[test]
    fn color_aa_absent_keeps_defaults() {
        use crate::render::antialiasing::AaStrength;
        // Braille (index 3) defaults to Strong.
        let rs = create_test_runtime_state();
        let mut overrides = capture_overrides(
            &SimConfig::default(),
            Palette::Organic,
            Charset::HalfBlock,
            &rs,
            false,
            false,
            false,
        );
        // Simulate a config saved before this feature existed.
        overrides.color_aa = None;

        let mut rs2 = create_test_runtime_state();
        apply_to_runtime_state(&overrides, &mut rs2).expect("apply must succeed");
        // Default Strong for Braille must survive (not overwritten by absent field).
        assert_eq!(rs2.color_aa[3], AaStrength::Strong);
    }

    #[test]
    fn color_aa_all_full_per_charset_array_round_trips() {
        // Verify that ALL per-charset AA slots are saved and restored faithfully.
        // This guards the regression where the old per-slot Vec was collapsed to a
        // single scalar, silently dropping non-active charsets' AA settings.
        use crate::render::antialiasing::AaStrength;
        use crate::render::charset::NUM_CHARSETS;

        let mut rs = create_test_runtime_state();
        // Set DISTINCT values for every charset slot so any slot-drop is detectable.
        // Cycle through Off/Subtle/Strong/Off/Subtle/Strong/Off... for NUM_CHARSETS slots.
        let values = [AaStrength::Off, AaStrength::Subtle, AaStrength::Strong];
        for i in 0..NUM_CHARSETS {
            rs.color_aa[i] = values[i % values.len()];
        }

        let overrides = capture_overrides(
            &SimConfig::default(),
            Palette::Organic,
            Charset::HalfBlock,
            &rs,
            false,
            false,
            false,
        );

        // color_aa_all must capture the full array.
        let captured = overrides
            .color_aa_all
            .as_ref()
            .expect("color_aa_all must be Some");
        assert_eq!(
            captured.len(),
            NUM_CHARSETS,
            "must capture all charset slots"
        );
        for i in 0..NUM_CHARSETS {
            assert_eq!(
                captured[i],
                values[i % values.len()],
                "slot {i} must match the source value"
            );
        }

        // Serialize to TOML and deserialize back.
        let toml_str = toml::to_string(&overrides).expect("serialize must succeed");
        let reloaded: ProfileOverrides =
            toml::from_str(&toml_str).expect("deserialize must succeed");

        // Apply into a fresh RuntimeState and verify all slots.
        let mut rs2 = create_test_runtime_state();
        apply_to_runtime_state(&reloaded, &mut rs2).expect("apply must succeed");
        for i in 0..NUM_CHARSETS {
            assert_eq!(
                rs2.color_aa[i],
                values[i % values.len()],
                "slot {i} must be restored after round-trip"
            );
        }
    }

    #[test]
    fn color_aa_all_fallback_to_scalar_when_absent() {
        // When color_aa_all is None (old saved-config format), the scalar color_aa
        // must still be applied to the active charset slot (back-compat).
        use crate::render::antialiasing::AaStrength;

        let mut rs = create_test_runtime_state();
        rs.color_aa[0] = AaStrength::Subtle;

        let mut overrides = capture_overrides(
            &SimConfig::default(),
            Palette::Organic,
            Charset::HalfBlock,
            &rs,
            false,
            false,
            false,
        );
        // Simulate a config saved before color_aa_all existed.
        overrides.color_aa_all = None;
        // color_aa scalar should still be Some(Subtle) from capture_overrides.
        assert_eq!(overrides.color_aa, Some(AaStrength::Subtle));

        let mut rs2 = create_test_runtime_state();
        apply_to_runtime_state(&overrides, &mut rs2).expect("apply must succeed");
        // Active charset slot (0) must be restored via the scalar fallback.
        assert_eq!(rs2.color_aa[0], AaStrength::Subtle);
    }

    #[test]
    fn reverse_invert_food_persist_round_trip() {
        // Verify the 3 new apply/persistence-only levers round-trip.
        let rs = create_test_runtime_state();
        let overrides = capture_overrides(
            &SimConfig::default(),
            Palette::Organic,
            Charset::HalfBlock,
            &rs,
            true, // reverse
            true, // invert
            true, // food_persist
        );
        assert_eq!(overrides.reverse_palette, Some(true));
        assert_eq!(overrides.invert_palette, Some(true));
        assert_eq!(overrides.food_persist, Some(true));

        // Serialize and deserialize.
        let toml_str = toml::to_string(&overrides).expect("serialize must succeed");
        let reloaded: ProfileOverrides =
            toml::from_str(&toml_str).expect("deserialize must succeed");
        assert_eq!(reloaded.reverse_palette, Some(true));
        assert_eq!(reloaded.invert_palette, Some(true));
        assert_eq!(reloaded.food_persist, Some(true));

        // Apply and verify.
        let mut new_state = create_test_runtime_state();
        apply_to_runtime_state(&reloaded, &mut new_state).expect("apply must succeed");
        assert!(new_state.reverse_palette);
        assert!(new_state.invert_palette);
        assert!(new_state.food_persist_enabled);
    }

    #[test]
    fn reverse_invert_default_false_when_absent() {
        // Absent reverse/invert/food_persist fields must apply as false.
        let toml = r#"
palette = "organic"
charset = "halfblock"
"#;
        let cfg: ProfileOverrides = toml::from_str(toml).unwrap();
        let mut state = create_test_runtime_state();
        apply_to_runtime_state(&cfg, &mut state).expect("apply must succeed");
        assert!(!state.reverse_palette);
        assert!(!state.invert_palette);
        assert!(!state.food_persist_enabled);
    }

    #[test]
    fn capture_overrides_brightness_gain_roundtrip() {
        // Verify brightness gain ↔ white-point conversion round-trips.
        let sim = SimConfig {
            max_brightness: 50.0,
            ..SimConfig::default()
        };
        let rs = create_test_runtime_state();

        let overrides = capture_overrides(
            &sim,
            Palette::Organic,
            Charset::HalfBlock,
            &rs,
            false,
            false,
            false,
        );

        // The overrides should store the brightness as gain.
        let expected_gain = crate::config_defaults::trail::brightness_gain(50.0);
        assert!(
            (overrides.brightness.unwrap() - expected_gain).abs() < 1e-5,
            "brightness gain should round-trip"
        );

        let mut new_state = create_test_runtime_state();
        apply_to_runtime_state(&overrides, &mut new_state).expect("apply must succeed");
        assert!(
            (new_state.max_brightness - 50.0).abs() < 1e-3,
            "max_brightness must round-trip through gain (got {})",
            new_state.max_brightness
        );
    }
}
