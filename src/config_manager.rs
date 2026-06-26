use crate::cli::{AttractorArg, ObstacleArg, SpeciesArg};
use crate::profile_overrides::ProfileOverrides;
use crate::render::charset::Charset;
use crate::render::palette::Palette;
use crate::simulation::config::{SimConfig, TerrainType};
use crate::terminal::control::RuntimeState;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

const CONFIG_DIR: &str = ".config/tslime";
const CONFIG_FILE: &str = "presets.toml";

/// A named, persisted profile: human-readable identity + all-optional overrides.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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

/// Convert a `TerrainType` value to the string form used by `ProfileOverrides.terrain`.
///
/// `TerrainType` only implements `FromStr` (no `Display`), so this helper provides
/// the reverse mapping for capture. The `Option` matches the field's type; every
/// variant maps to a name so the field round-trips identically to what `from_args`
/// emits.
fn terrain_name(t: TerrainType) -> Option<String> {
    match t {
        TerrainType::None => Some("none".to_string()),
        TerrainType::Smooth => Some("smooth".to_string()),
        TerrainType::Turbulent => Some("turbulent".to_string()),
        TerrainType::Mixed => Some("mixed".to_string()),
    }
}

/// Build a `ProfileOverrides` from live runtime state for config-save.
///
/// Sources each lever from the live runtime state:
/// - sim levers from `sim_config` (as-resolved)
/// - render/art levers from `runtime_state` (live session values)
/// - reverse/invert/food_persist from `rs.*` (live toggles, not startup args)
/// - app-runtime levers from `rs.app.*`
/// - wind from `rs.wind` (lossless precise vector)
pub fn capture_overrides(
    sim_config: &SimConfig,
    palette: Palette,
    charset: Charset,
    rs: &RuntimeState,
) -> ProfileOverrides {
    // Convert sim.max_brightness (white-point) back to user-facing brightness gain.
    let brightness_gain = crate::config_defaults::trail::brightness_gain(sim_config.max_brightness);

    let temporal_accent = rs.temporal_accent;

    // An identity cycle is the default, so record None rather than reading dirty.
    let palette_cycle = if rs.palette_cycle.is_identity() {
        None
    } else {
        Some(rs.palette_cycle)
    };

    let (glyph_selection, glyph_edge_threshold) = match rs.glyph.selection {
        None => (None, None),
        Some(sel) => (Some(sel), Some(rs.glyph.edge_threshold)),
    };

    // ProfileOverrides.color_aa is a single AaStrength scalar, so save the active
    // charset's value; apply reads it back into all slots. The full per-charset
    // array is preserved separately in color_aa_all below.
    let color_aa = Some(rs.color_aa[rs.charset_index % rs.color_aa.len()]);

    let intensity_mapping = Some(rs.intensity_mapping.clone());

    ProfileOverrides {
        // provenance — not set on save (these would only appear in preset TOML)
        preset: None,
        seed: None,

        // sim levers (sourced from sim_config)
        sensor_angle: Some(sim_config.sensor_angle),
        sensor_distance: Some(sim_config.sensor_distance),
        rotation_angle: Some(sim_config.rotation_angle),
        step_size: Some(sim_config.step_size),
        decay_factor: Some(sim_config.decay_factor),
        deposit_amount: Some(sim_config.deposit_amount),
        brightness: Some(brightness_gain),
        diffusion_kernel: Some(sim_config.diffusion_kernel),
        diffusion_sigma: Some(sim_config.diffusion_sigma),
        // sim levers — time_scale is live-editable via +/- keys so capture the live value.
        time_scale: Some(rs.time_scale),
        // population: capture for display (config-browser "Nk agents") only; a
        // restart-only lever, not live-applied.
        population: Some(sim_config.total_population()),
        fps: None,
        food_image_path: None,
        food_image_invert: None,
        food_image_scale: None,
        attractors: sim_config
            .attractors
            .iter()
            .map(|a| AttractorArg {
                x: a.x,
                y: a.y,
                strength: a.strength,
            })
            .collect(),
        attractor_strength: Some(sim_config.attractor_strength),
        obstacles: sim_config
            .obstacles
            .iter()
            .map(|o| ObstacleArg {
                obstacle: o.clone(),
            })
            .collect(),
        species: sim_config
            .species_configs
            .iter()
            .map(|s| SpeciesArg {
                name: s.name.clone(),
                count: s.count,
                sensor_angle: s.sensor_angle,
                rotation_angle: s.rotation_angle,
                step_size: s.step_size,
                deposit_amount: s.deposit_amount,
                color: s.color,
                trail_modulation: s.trail_modulation,
            })
            .collect(),
        separate_species_trails: sim_config.separate_species_trails,
        species_colors: false,
        use_simd: None,
        wind: rs.wind.map(|w| crate::cli::WindArg { dx: w.dx, dy: w.dy }),
        terrain: terrain_name(sim_config.terrain),
        terrain_strength: Some(sim_config.terrain_strength),
        background_color: sim_config.background_color.clone(),
        // Capture boundary_mode so a Wrap preset (River/Smoke/Mold) round-trips
        // through projection; leaving it None defaulted the live mirror to Bounce
        // and made any Wrap preset read falsely dirty. Symmetric with window_frame.
        boundary_mode: Some(sim_config.boundary_mode),
        window_frame: Some(sim_config.window_frame),
        chrome_style: Some(sim_config.chrome_style),
        transition_style: Some(sim_config.transition_style),
        transition_tagline: Some(sim_config.transition_tagline),
        aspect: Some(sim_config.aspect),
        window_padding: Some(sim_config.window_padding),
        frame_matte_cols: Some(sim_config.frame_matte_cols),
        frame_matte_rows: Some(sim_config.frame_matte_rows),
        show_status_bar: Some(sim_config.show_status_bar),
        min_sim_size: Some(sim_config.min_sim_size),
        min_frame_size: Some(sim_config.min_frame_size),
        respawn_interval: None,
        respawn_config: Some(sim_config.respawn_config),
        decay_gamma: Some(rs.decay_gamma),
        diffuse_weight: Some(rs.diffuse_weight),
        deposit_curve: Some(rs.deposit_curve),
        deposit_scale: Some(rs.deposit_scale),
        deposit_gamma: Some(rs.deposit_gamma),
        deposit_cap: Some(rs.deposit_cap),

        // render levers (sourced from runtime_state)
        palette: Some(palette),
        charset: Some(charset),
        color_aa,
        hue_shift: None,
        // Serialize the explicit live value for dirty parity: a clean
        // auto-normalized preset must project identically on both sides.
        auto_normalize: Some(rs.auto_normalize),
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

        // apply/persistence-only levers (sourced from live rs, not startup args)
        reverse_palette: Some(rs.reverse_palette),
        invert_palette: Some(rs.invert_palette),
        food_persist: Some(rs.food_persist_enabled),
        // Full per-charset AA array (color_aa above is just the active charset's scalar).
        color_aa_all: Some(rs.color_aa.to_vec()),

        // app-runtime levers — sourced from rs.app (live session values).
        init_mode: sim_config.preferred_init_mode,
        warmup_frames: Some(rs.app.warmup_frames),
        skip_warmup: Some(rs.app.skip_warmup),
        warmup_brightness_multiplier: Some(rs.app.warmup_brightness_multiplier),
        auto_reset: Some(rs.app.auto_reset),
        auto_reset_entropy_threshold: Some(rs.app.auto_reset_entropy_threshold),
        auto_reset_duration_frames: Some(rs.app.auto_reset_duration_frames),
        grid: Some(rs.app.grid),
        grid_style: Some(rs.app.grid_style),
        grid_size: Some(rs.app.grid_size),
        grid_color: Some(rs.app.grid_color),
        grid_opacity: Some(rs.app.grid_opacity),
        grid_adaptive: Some(rs.app.grid_adaptive),
        food_persist_strength: Some(rs.app.food_persist_strength),
        food_persist_radius: Some(rs.app.food_persist_radius),
        food_persist_duration: Some(rs.app.food_persist_duration),
    }
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

    parse_config_file(&contents)
}

/// Parse config-file TOML into a `ConfigFile`. Pure (no IO) so the stale-schema
/// tolerance can be regression-tested directly.
fn parse_config_file(contents: &str) -> Result<ConfigFile, String> {
    toml::from_str(contents).map_err(|e| format!("Failed to parse config file: {}", e))
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
/// Overwrites any existing configuration with the same name. If the existing
/// on-disk file is unparseable (e.g. a stale schema the lenient serde still
/// can't recover), it is moved aside to `presets.toml.bak` and a fresh file is
/// started, so one bad file never blocks saving. Returns `Some(warning)` when a
/// backup happened, so the caller can surface it.
pub fn save_config(profile: NamedProfile) -> Result<Option<String>, String> {
    let (mut config_file, warning) = match load_config_file() {
        Ok(cf) => (cf, None),
        Err(_) => {
            let bak = back_up_config_file()?;
            (
                ConfigFile {
                    presets: Vec::new(),
                },
                Some(format!(
                    "Existing config was unreadable; backed up to {bak}"
                )),
            )
        }
    };

    config_file.presets.retain(|c| c.name != profile.name);
    config_file.presets.push(profile);

    save_config_file(&config_file)?;
    Ok(warning)
}

/// Move the current (unparseable) config file aside to `presets.toml.bak`,
/// returning the backup file name for surfacing to the user.
fn back_up_config_file() -> Result<String, String> {
    let path = get_config_path()?;
    let bak = path.with_extension("toml.bak");
    fs::rename(&path, &bak).map_err(|e| format!("Failed to back up unreadable config: {}", e))?;
    Ok(bak
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("presets.toml.bak")
        .to_string())
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

    /// Resolve overrides through the SHIPPING path (`resolve`), the same seam the
    /// runner's `apply_overrides` uses. Tests assert on the resolved
    /// `Profile { sim, render, app }` rather than poking a `RuntimeState` via a
    /// parallel apply path.
    fn resolved(ov: &ProfileOverrides) -> crate::profile::Profile {
        ov.resolve().expect("resolve must succeed")
    }

    /// Build a minimal ProfileOverrides with required sim fields populated —
    /// mirrors a config written by capture_overrides from default sim state.
    fn minimal_overrides(name: &str) -> NamedProfile {
        let sim = SimConfig::default();
        let rs = create_test_runtime_state();
        NamedProfile {
            name: name.to_string(),
            description: None,
            overrides: capture_overrides(&sim, Palette::Organic, Charset::HalfBlock, &rs),
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
            overrides: capture_overrides(&sim, Palette::Heat, Charset::HalfBlock, &rs),
        };

        let p = resolved(&profile.overrides);
        assert_eq!(p.render.palette, Palette::Heat);
    }

    #[test]
    fn diffusion_sigma_resolves_through_saved_config() {
        let sim = SimConfig {
            diffusion_sigma: 2.75,
            ..SimConfig::default()
        };
        let rs = create_test_runtime_state();

        let profile = NamedProfile {
            name: "test_sigma".to_string(),
            description: None,
            overrides: capture_overrides(&sim, Palette::Heat, Charset::HalfBlock, &rs),
        };

        let p = resolved(&profile.overrides);
        assert!((p.sim.diffusion_sigma - 2.75).abs() < 1e-6);
    }

    #[test]
    fn intensity_mapping_resolves_to_default_when_none() {
        use crate::render::palette::IntensityMapping;
        let rs = create_test_runtime_state();

        // Build overrides without intensity_mapping (simulate old-format config).
        let mut overrides = capture_overrides(
            &SimConfig::default(),
            Palette::Heat,
            Charset::HalfBlock,
            &rs,
        );
        overrides.intensity_mapping = None;

        // Default is logarithmic(10.0); a saved config with no recorded mapping
        // must resolve to the canonical default, not inherit a session value.
        let p = resolved(&overrides);
        assert_eq!(p.render.intensity_mapping, IntensityMapping::default());
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

        let overrides = capture_overrides(&sim_config, Palette::Neon, Charset::HalfBlock, &state);

        // Resolve through the shipping path and verify all values survived.
        let p = resolved(&overrides);
        assert_eq!(p.render.palette, Palette::Neon);
        // reverse/invert are persist-only (not in resolve); assert on the capture.
        assert_eq!(overrides.reverse_palette, Some(true));
        assert_eq!(overrides.invert_palette, Some(true));
        assert_eq!(p.sim.sensor_angle, state.sensor_angle);
        assert_eq!(p.sim.rotation_angle, state.rotation_angle);
        assert_eq!(p.sim.step_size, state.step_size);
        assert_eq!(p.sim.decay_factor, state.decay_factor);
        assert_eq!(p.sim.deposit_amount, state.deposit_amount);
        assert!((p.sim.max_brightness - state.max_brightness).abs() < 1e-3);
        assert_eq!(p.sim.diffusion_kernel, state.diffusion_kernel);
    }

    #[test]
    fn temporal_fields_round_trip_and_default_off() {
        // Minimal overrides with no temporal fields set → temporal_color defaults to 0.0.
        let mut overrides = capture_overrides(
            &SimConfig::default(),
            Palette::Organic,
            Charset::HalfBlock,
            &create_test_runtime_state(),
        );
        // Simulate an old-format config that had no temporal field.
        overrides.temporal_color = None;

        let p = resolved(&overrides);
        assert_eq!(p.render.temporal_color, 0.0);
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
        );

        // Serialize and deserialize through TOML
        let toml_str = toml::to_string(&overrides).expect("serialize must succeed");
        let reloaded: ProfileOverrides =
            toml::from_str(&toml_str).expect("deserialize must succeed");

        // Resolve through the shipping path.
        let p = resolved(&reloaded);
        assert!((p.render.temporal_color - 0.7).abs() < 1e-6);
        assert!((p.render.temporal_lag_frames - 12.0).abs() < 1e-6);
        assert_eq!(p.render.temporal_mode, TemporalMode::Accent);
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

        let overrides = capture_overrides(&sim, Palette::Organic, Charset::HalfBlock, &state);

        // Serialize and deserialize through TOML
        let toml_str = toml::to_string(&overrides).expect("serialize must succeed");
        let reloaded: ProfileOverrides =
            toml::from_str(&toml_str).expect("deserialize must succeed");

        // Resolve through the shipping path.
        let p = resolved(&reloaded);
        assert!(
            (p.render.afterglow - 0.4).abs() < 1e-6,
            "afterglow must survive round-trip (got {})",
            p.render.afterglow
        );
        assert!(
            (p.render.afterglow_rate - 0.03).abs() < 1e-6,
            "afterglow_rate must survive round-trip (got {})",
            p.render.afterglow_rate
        );
        assert!(
            (p.sim.decay_gamma - 0.6).abs() < 1e-6,
            "decay_gamma must survive round-trip (got {})",
            p.sim.decay_gamma
        );
        assert!(
            (p.sim.diffuse_weight - 0.5).abs() < 1e-6,
            "diffuse_weight must survive round-trip (got {})",
            p.sim.diffuse_weight
        );
        assert!(
            (p.sim.diffusion_sigma - 3.0).abs() < 1e-6,
            "diffusion_sigma must survive round-trip (got {})",
            p.sim.diffusion_sigma
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

        let p = resolved(&cfg);
        assert_eq!(p.render.afterglow, 0.0, "default afterglow must be 0.0");
        assert!(
            (p.render.afterglow_rate - 0.05).abs() < 1e-6,
            "default afterglow_rate must be 0.05"
        );
        assert_eq!(p.sim.decay_gamma, 1.0, "default decay_gamma must be 1.0");
        assert_eq!(
            p.sim.diffuse_weight, 1.0,
            "default diffuse_weight must be 1.0"
        );
        assert!(cfg.deposit_curve.is_none());
        assert!(cfg.deposit_scale.is_none());
        assert!(cfg.deposit_gamma.is_none());
        assert!(cfg.deposit_cap.is_none());
        assert_eq!(
            p.sim.deposit_curve,
            DepositCurve::default(),
            "default deposit_curve must be Linear"
        );
        assert_eq!(
            p.sim.deposit_scale, 1.0,
            "default deposit_scale must be 1.0"
        );
        assert_eq!(
            p.sim.deposit_gamma, 1.0,
            "default deposit_gamma must be 1.0"
        );
        assert_eq!(p.sim.deposit_cap, 0.0, "default deposit_cap must be 0.0");
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
        );

        // Serialize and deserialize through TOML
        let toml_str = toml::to_string(&overrides).expect("serialize must succeed");
        let reloaded: ProfileOverrides =
            toml::from_str(&toml_str).expect("deserialize must succeed");

        // Resolve through the shipping path.
        let p = resolved(&reloaded);
        assert_eq!(
            p.sim.deposit_curve,
            DepositCurve::Pow,
            "deposit_curve must survive round-trip"
        );
        assert!(
            (p.sim.deposit_scale - 2.5).abs() < 1e-6,
            "deposit_scale must survive round-trip (got {})",
            p.sim.deposit_scale
        );
        assert!(
            (p.sim.deposit_gamma - 0.5).abs() < 1e-6,
            "deposit_gamma must survive round-trip (got {})",
            p.sim.deposit_gamma
        );
        assert!(
            (p.sim.deposit_cap - 7.0).abs() < 1e-6,
            "deposit_cap must survive round-trip (got {})",
            p.sim.deposit_cap
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

        // Resolve through the shipping path.
        let p = resolved(&reloaded);
        assert_eq!(
            p.render.palette_cycle,
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
        );

        assert_eq!(overrides.glyph_selection, Some(GlyphSelection::Hybrid));
        assert_eq!(overrides.glyph_edge_threshold, Some(0.25));

        let p = resolved(&overrides);
        assert_eq!(p.render.glyph.selection, Some(GlyphSelection::Hybrid));
        assert_eq!(p.render.glyph.edge_threshold, 0.25);
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

        let p = resolved(&cfg);
        assert!(
            p.render.palette_cycle.is_identity(),
            "missing palette_cycles must default to identity"
        );
        assert_eq!(
            p.render.palette_cycle,
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

        let p = resolved(&cfg);
        assert_eq!(
            p.render.glyph,
            GlyphConfig::default(),
            "missing glyph keys must default to GlyphConfig::default()"
        );
        assert!(
            p.render.glyph.selection.is_none(),
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
        );

        // Verify it's set in the overrides.
        assert_eq!(overrides.temporal_accent, Some(accent));

        // Round-trip through TOML.
        let toml_str = toml::to_string(&overrides).expect("serialize must succeed");
        let reloaded: ProfileOverrides =
            toml::from_str(&toml_str).expect("deserialize must succeed");
        let p = resolved(&reloaded);

        assert_eq!(
            p.render.temporal_accent,
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
        );
        // Should capture the active slot value.
        assert_eq!(overrides.color_aa, Some(AaStrength::Subtle));

        // Per-charset AA is applied via the shipping seam rs.apply_color_aa_all.
        let mut rs2 = create_test_runtime_state();
        rs2.apply_color_aa_all(&overrides);
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
        );
        // Simulate a config saved before this feature existed.
        overrides.color_aa = None;

        let mut rs2 = create_test_runtime_state();
        rs2.apply_color_aa_all(&overrides);
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
        rs2.apply_color_aa_all(&reloaded);
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
        );
        // Simulate a config saved before color_aa_all existed.
        overrides.color_aa_all = None;
        // color_aa scalar should still be Some(Subtle) from capture_overrides.
        assert_eq!(overrides.color_aa, Some(AaStrength::Subtle));

        let mut rs2 = create_test_runtime_state();
        rs2.apply_color_aa_all(&overrides);
        // Active charset slot (0) must be restored via the scalar fallback.
        assert_eq!(rs2.color_aa[0], AaStrength::Subtle);
    }

    #[test]
    fn reverse_invert_food_persist_round_trip() {
        // Verify the 3 new apply/persistence-only levers round-trip.
        let mut rs = create_test_runtime_state();
        rs.reverse_palette = true;
        rs.invert_palette = true;
        rs.food_persist_enabled = true;
        let overrides = capture_overrides(
            &SimConfig::default(),
            Palette::Organic,
            Charset::HalfBlock,
            &rs,
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

        // These three are persist-only (not in the resolved Profile); the runner's
        // apply seam reads them as `ov.<field>.unwrap_or(false)`.
        assert!(reloaded.reverse_palette.unwrap_or(false));
        assert!(reloaded.invert_palette.unwrap_or(false));
        assert!(reloaded.food_persist.unwrap_or(false));
    }

    #[test]
    fn reverse_invert_default_false_when_absent() {
        // Absent reverse/invert/food_persist fields must apply as false.
        let toml = r#"
palette = "organic"
charset = "halfblock"
"#;
        let cfg: ProfileOverrides = toml::from_str(toml).unwrap();
        // Persist-only levers; the apply seam reads `ov.<field>.unwrap_or(false)`.
        assert!(!cfg.reverse_palette.unwrap_or(false));
        assert!(!cfg.invert_palette.unwrap_or(false));
        assert!(!cfg.food_persist.unwrap_or(false));
    }

    #[test]
    fn capture_overrides_brightness_gain_roundtrip() {
        // Verify brightness gain ↔ white-point conversion round-trips.
        let sim = SimConfig {
            max_brightness: 50.0,
            ..SimConfig::default()
        };
        let rs = create_test_runtime_state();

        let overrides = capture_overrides(&sim, Palette::Organic, Charset::HalfBlock, &rs);

        // The overrides should store the brightness as gain.
        let expected_gain = crate::config_defaults::trail::brightness_gain(50.0);
        assert!(
            (overrides.brightness.unwrap() - expected_gain).abs() < 1e-5,
            "brightness gain should round-trip"
        );

        let p = resolved(&overrides);
        assert!(
            (p.sim.max_brightness - 50.0).abs() < 1e-3,
            "max_brightness must round-trip through gain (got {})",
            p.sim.max_brightness
        );
    }
    #[test]
    fn capture_reads_live_reverse_flag() {
        // Proves that capture_overrides reads rs.reverse_palette (live), not a frozen arg.
        let mut rs = create_test_runtime_state();
        rs.reverse_palette = true;
        let overrides = capture_overrides(
            &SimConfig::default(),
            Palette::Organic,
            Charset::HalfBlock,
            &rs,
        );
        assert_eq!(
            overrides.reverse_palette,
            Some(true),
            "capture must read live rs.reverse_palette, not a frozen startup arg"
        );
    }

    #[test]
    fn capture_reads_live_auto_normalize_flag() {
        // Proves capture_overrides serializes the live rs.auto_normalize (explicit
        // Some), required for dirty parity against a clean auto-normalized session.
        let mut rs = create_test_runtime_state();
        rs.auto_normalize = true;
        let overrides = capture_overrides(
            &SimConfig::default(),
            Palette::Organic,
            Charset::HalfBlock,
            &rs,
        );
        assert_eq!(
            overrides.auto_normalize,
            Some(true),
            "capture must read live rs.auto_normalize, not a frozen startup arg"
        );
    }

    #[test]
    fn capture_reads_live_app_lever_warmup_frames() {
        // Proves app lever round-trips: warmup_frames is captured from rs.app.
        use crate::app_config::AppRuntimeConfig;
        let mut rs = create_test_runtime_state();
        rs.app = AppRuntimeConfig {
            warmup_frames: 120,
            ..AppRuntimeConfig::default()
        };
        let overrides = capture_overrides(
            &SimConfig::default(),
            Palette::Organic,
            Charset::HalfBlock,
            &rs,
        );
        assert_eq!(overrides.warmup_frames, Some(120));
        // Resolve and verify it round-trips.
        let profile = overrides.resolve().expect("resolve must succeed");
        assert_eq!(profile.app.warmup_frames, 120);
    }

    #[test]
    fn apply_color_aa_all_restores_all_slots() {
        use crate::render::antialiasing::AaStrength;
        use crate::render::charset::NUM_CHARSETS;

        let mut rs = create_test_runtime_state();
        // Build overrides with color_aa_all carrying distinct per-slot values.
        let values: Vec<AaStrength> = (0..NUM_CHARSETS)
            .map(|i| [AaStrength::Off, AaStrength::Subtle, AaStrength::Strong][i % 3])
            .collect();
        let ov = crate::profile_overrides::ProfileOverrides {
            color_aa_all: Some(values.clone()),
            ..crate::profile_overrides::ProfileOverrides::default()
        };

        rs.apply_color_aa_all(&ov);

        for (i, expected) in values.iter().enumerate() {
            assert_eq!(
                rs.color_aa[i], *expected,
                "slot {i} must be restored from color_aa_all"
            );
        }
    }

    #[test]
    fn apply_color_aa_all_falls_back_to_scalar_active_slot() {
        use crate::render::antialiasing::AaStrength;

        let mut rs = create_test_runtime_state();
        rs.charset_index = 0;
        let ov = crate::profile_overrides::ProfileOverrides {
            color_aa_all: None,
            color_aa: Some(AaStrength::Subtle),
            ..crate::profile_overrides::ProfileOverrides::default()
        };

        rs.apply_color_aa_all(&ov);

        assert_eq!(
            rs.color_aa[0],
            AaStrength::Subtle,
            "scalar fallback must set the active charset slot"
        );
    }

    /// capture_overrides must round-trip obstacles, attractors, attractor_strength,
    /// terrain, and terrain_strength through resolve() so a clean session stays clean.
    /// This is the regression test for I-1: before the fix, these fields were hardcoded
    /// to empty / None, causing resolved SimConfigs to diverge on re-capture.
    #[test]
    fn capture_roundtrip_obstacle_attractor_and_terrain() {
        use crate::simulation::config::{Attractor, TerrainType};

        // Build a SimConfig carrying all the fields the fix addresses.
        let obstacle = crate::simulation::config::Obstacle::Circle {
            x: 200.0,
            y: 100.0,
            radius: 90.0,
        };
        let attractor = Attractor::new(10.0, 20.0, 0.8);
        let sim = SimConfig {
            obstacles: vec![obstacle.clone()],
            attractors: vec![attractor],
            attractor_strength: 1.5,
            terrain: TerrainType::Smooth,
            terrain_strength: 2.0,
            ..SimConfig::default()
        };

        let rs = create_test_runtime_state();
        let overrides = capture_overrides(&sim, Palette::Organic, Charset::HalfBlock, &rs);

        // obstacles must be captured, not empty.
        assert_eq!(
            overrides.obstacles.len(),
            1,
            "obstacles must be captured from sim_config"
        );
        assert_eq!(overrides.obstacles[0].obstacle, obstacle);

        // attractors must be captured.
        assert_eq!(
            overrides.attractors.len(),
            1,
            "attractors must be captured from sim_config"
        );
        assert!((overrides.attractors[0].x - 10.0).abs() < 1e-6);
        assert!((overrides.attractors[0].y - 20.0).abs() < 1e-6);
        assert!((overrides.attractors[0].strength - 0.8).abs() < 1e-6);

        // attractor_strength must be captured.
        assert_eq!(overrides.attractor_strength, Some(1.5));

        // terrain must be captured as "smooth".
        assert_eq!(overrides.terrain, Some("smooth".to_string()));

        // terrain_strength must be captured.
        assert_eq!(overrides.terrain_strength, Some(2.0));

        // Round-trip through resolve: the resolved SimConfig must carry the same values.
        let profile = overrides.resolve().expect("overrides must resolve");
        assert_eq!(
            profile.sim.obstacles,
            vec![obstacle],
            "resolved obstacles must match capture source"
        );
        assert_eq!(
            profile.sim.attractors.len(),
            1,
            "resolved attractors must match capture source"
        );
        assert!((profile.sim.attractors[0].x - 10.0).abs() < 1e-6);
        assert!((profile.sim.attractor_strength - 1.5).abs() < 1e-6);
        assert_eq!(profile.sim.terrain, TerrainType::Smooth);
        assert!((profile.sim.terrain_strength - 2.0).abs() < 1e-6);
    }

    /// terrain_name helper: all four variants map to the expected strings.
    #[test]
    fn terrain_name_covers_all_variants() {
        use crate::simulation::config::TerrainType;
        assert_eq!(terrain_name(TerrainType::None), Some("none".to_string()));
        assert_eq!(
            terrain_name(TerrainType::Smooth),
            Some("smooth".to_string())
        );
        assert_eq!(
            terrain_name(TerrainType::Turbulent),
            Some("turbulent".to_string())
        );
        assert_eq!(terrain_name(TerrainType::Mixed), Some("mixed".to_string()));
        // Each string must round-trip through FromStr.
        for (t, s) in [
            (TerrainType::None, "none"),
            (TerrainType::Smooth, "smooth"),
            (TerrainType::Turbulent, "turbulent"),
            (TerrainType::Mixed, "mixed"),
        ] {
            let parsed: TerrainType = s.parse().expect("must parse");
            assert_eq!(parsed, t, "terrain_name/FromStr must be inverse for {s}");
        }
    }

    /// A pre-Phase-B `presets.toml` carried `window_frame = ""` plus fields that
    /// were later dropped (`food_path`, `show_status_bar`, `min_sim_size`, …).
    /// The new serde must tolerate it: unknown fields ignored, empty `window_frame`
    /// treated as `None`, so one stale file never poisons all config save/load.
    const LEGACY_PRESETS_TOML: &str = r#"
[[preset]]
name = "Mossy Roots"
population = 50000
decay_factor = 0.5
palette = "moss"
charset = "halfblock"
init_mode = "food"
food_path = "assets/tslime_logo.png"
window_frame = ""
chrome_style = "minimal"
aspect = "3:2"
window_padding = "auto"
show_status_bar = false
min_sim_size = "20x10"
min_frame_size = "12x6"

[[preset]]
name = "warm-1"
population = 50000
palette = "warm"
charset = "ascii"
window_frame = "glow"
chrome_style = "minimal"
food_path = "assets/tslime_logo.png"
"#;

    #[test]
    fn trail_modulation_round_trips_through_saved_config() {
        use crate::simulation::config::{PointConfig, SpeciesConfig};
        let rs = create_test_runtime_state();
        // Preset-derived species carry trail_modulation; capture must preserve it.
        // Use exactly-representable f32 values so the TOML round-trip is bit-stable.
        let modulation = PointConfig {
            sensor_angle_multiplier: 0.5,
            step_size_base: 2.5,
            trail_rescale: 0.25,
            ..PointConfig::default()
        };
        let sim = SimConfig {
            species_configs: vec![SpeciesConfig {
                trail_modulation: Some(modulation),
                ..SpeciesConfig::default()
            }],
            ..SimConfig::default()
        };
        let overrides = capture_overrides(&sim, Palette::Organic, Charset::HalfBlock, &rs);
        assert_eq!(
            overrides.species[0].trail_modulation,
            Some(modulation),
            "capture must carry trail_modulation into SpeciesArg"
        );
        // Round-trip through the actual save format (TOML), then resolve.
        let named = NamedProfile {
            name: "t".to_string(),
            description: None,
            overrides,
        };
        let toml_str = toml::to_string(&named).expect("serialize");
        let reparsed: NamedProfile = toml::from_str(&toml_str).expect("deserialize");
        let profile = reparsed.overrides.resolve().expect("resolve");
        assert_eq!(
            profile.sim.species_configs[0].trail_modulation,
            Some(modulation),
            "trail_modulation must survive save -> reload -> resolve"
        );
    }

    #[test]
    fn legacy_presets_file_parses_without_poisoning() {
        let cf = parse_config_file(LEGACY_PRESETS_TOML)
            .expect("stale-schema presets.toml must not hard-fail parsing");
        assert_eq!(cf.presets.len(), 2, "both entries survive");
        // Empty-string window_frame degrades to None (falls through to preset art).
        assert_eq!(cf.presets[0].overrides.window_frame, None);
        // A valid window_frame still deserializes.
        assert_eq!(
            cf.presets[1].overrides.window_frame,
            Some(crate::simulation::config::WindowFrame::Glow)
        );
        // Dropped fields are ignored, real fields still land.
        assert_eq!(cf.presets[0].name, "Mossy Roots");
        assert_eq!(cf.presets[0].overrides.population, Some(50000));
    }

    #[test]
    fn time_scale_captured_from_live_runtime_state() {
        // capture_overrides must emit Some(time_scale), not None, so that a non-default
        // time_scale is preserved through save → load → resolve.
        let mut rs = create_test_runtime_state();
        rs.time_scale = 2.5;

        let overrides = capture_overrides(
            &SimConfig::default(),
            Palette::Organic,
            Charset::HalfBlock,
            &rs,
        );

        assert_eq!(
            overrides.time_scale,
            Some(2.5),
            "capture_overrides must capture the live time_scale, not hardcode None"
        );

        // Verify it round-trips through resolve().
        let p = resolved(&overrides);
        assert!(
            (p.sim.time_scale - 2.5).abs() < 1e-6,
            "time_scale must survive round-trip through resolve (got {})",
            p.sim.time_scale
        );
    }
}
