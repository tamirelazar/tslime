use crate::simulation::config::DiffusionKernel;
use crate::simulation::config::TerrainType;
use crate::terminal::control::DefaultValues;
use crate::terminal::control::PaletteShiftSpeed;
use crate::terminal::control::WindDirection;

// Renamed from OptionsOverlay - this is now the Controls overlay
/// Overlay that displays current simulation controls and parameters.
pub struct ControlsOverlay;

// Type alias for backwards compatibility
#[allow(dead_code)]
/// Deprecated alias for `ControlsOverlay`.
pub type OptionsOverlay = ControlsOverlay;

impl ControlsOverlay {
    /// Width of the controls overlay window.
    pub const WIDTH: usize = 48;
    /// Total number of control categories.
    pub const TOTAL_CATEGORIES: usize = 6;

    /// Returns the name of a specific control category.
    pub fn category_name(idx: usize) -> &'static str {
        match idx {
            0 => "SIMULATION CORE",
            1 => "FORCES & ENVIRONMENT",
            2 => "APPEARANCE",
            3 => "POST-PROCESSING",
            4 => "PERFORMANCE",
            5 => "SYSTEM",
            _ => "UNKNOWN",
        }
    }

    #[allow(dead_code)]
    /// Returns the total number of categories.
    pub fn total_categories() -> usize {
        Self::TOTAL_CATEGORIES
    }

    #[allow(clippy::too_many_arguments)]
    /// Builds the list of strings for the controls overlay based on current state.
    pub fn build_overlay(
        category_idx: usize,
        sensor_angle: f32,
        sensor_distance: f32,
        turn_angle: f32,
        step_size: f32,
        decay_factor: f32,
        deposit_amount: f32,
        time_scale: f32,
        diffusion_kernel: DiffusionKernel,
        diffusion_sigma: f32,
        attractor_strength: f32,
        mouse_mode: &str,
        mouse_timeout: f32,
        wind_direction: WindDirection,
        terrain_type: TerrainType,
        terrain_strength: f32,
        auto_normalize: bool,
        motion_blur_frames: usize,
        max_brightness: f32,
        fast_mode_enabled: bool,
        palette_name: &str,
        palette_shift_speed: PaletteShiftSpeed,
        invert_palette: bool,
        reverse_palette: bool,
        dither_mode_name: &str,
        _term_width: usize,
        defaults: DefaultValues,
        population: usize,
    ) -> Vec<String> {
        let builder = crate::render::overlay::WindowBuilder::new(Self::WIDTH, 2);
        let cat_name = Self::category_name(category_idx);
        let cat_num = category_idx + 1;

        let mut content = Vec::new();
        content.push(format!(
            "{:^width$}",
            cat_name,
            width = builder.inner_width()
        ));
        content.push("".to_string());

        let mod_marker = |current: f32, default: f32, eps: f32| {
            if (current - default).abs() > eps {
                "*"
            } else {
                " "
            }
        };
        let mod_marker_int =
            |current: usize, default: usize| if current != default { "*" } else { " " };
        let mod_marker_enum = |current: &dyn std::fmt::Debug, default: &dyn std::fmt::Debug| {
            if format!("{:?}", current) != format!("{:?}", default) {
                "*"
            } else {
                " "
            }
        };

        match category_idx {
            0 => {
                content.push(format!(
                    "{} A/a  Sensor Angle  {:>5.1}° [5-90°]",
                    mod_marker(sensor_angle, defaults.sensor_angle, 0.01),
                    sensor_angle
                ));
                content.push(format!(
                    "{} J/j  Sensor Dist   {:>5.1}px [1-50]",
                    mod_marker(sensor_distance, defaults.sensor_distance, 0.01),
                    sensor_distance
                ));
                content.push(format!(
                    "{} T/t  Turn Angle    {:>5.1}° [5-90°]",
                    mod_marker(turn_angle, defaults.turn_angle, 0.01),
                    turn_angle
                ));
                content.push(format!(
                    "{} S/s  Step Size     {:>5.1}px [0.5-5.0]",
                    mod_marker(step_size, defaults.step_size, 0.01),
                    step_size
                ));
                content.push(format!(
                    "{} E/e  Decay Factor  {:>5.3}x [0.5-0.99]",
                    mod_marker(decay_factor, defaults.decay_factor, 0.001),
                    decay_factor
                ));
                content.push(format!(
                    "{} I/i  Deposit Amt   {:>5.1}x [1-20]",
                    mod_marker(deposit_amount, defaults.deposit_amount, 0.01),
                    deposit_amount
                ));
                content.push(format!(
                    "   +/-  Time Scale    {:>5.1}x [0.5-4x]",
                    time_scale
                ));
            }
            1 => {
                content.push(format!(
                    "{} K    Diffusion         {:>14}",
                    mod_marker_enum(&diffusion_kernel, &defaults.diffusion_kernel),
                    match diffusion_kernel {
                        DiffusionKernel::Mean3x3 => "Mean3x3",
                        DiffusionKernel::Gaussian => "Gaussian",
                    }
                ));
                if matches!(diffusion_kernel, DiffusionKernel::Gaussian) {
                    content.push(format!(
                        "{} ;/:  Diff Sigma  {:>5.2}x [0.5-2.0]",
                        mod_marker(diffusion_sigma, defaults.diffusion_sigma, 0.01),
                        diffusion_sigma
                    ));
                }
                content.push(format!(
                    "{} W    Wind              {:>14}",
                    mod_marker_enum(&wind_direction, &defaults.wind_direction),
                    wind_direction.name()
                ));
                content.push(format!(
                    "{} U    Terrain Type      {:>14}",
                    mod_marker_enum(&terrain_type, &defaults.terrain_type),
                    match terrain_type {
                        TerrainType::None => "None",
                        TerrainType::Smooth => "Smooth",
                        TerrainType::Turbulent => "Turbulent",
                        TerrainType::Mixed => "Mixed",
                    }
                ));
                content.push(format!(
                    "{} Y/y  Terrain Str   {:>5.1}x [0.1-5.0]",
                    mod_marker(terrain_strength, defaults.terrain_strength, 0.01),
                    terrain_strength
                ));
                content.push(format!(
                    "{} L/l  Attractor Str {:>5.1}x [0.1-10]",
                    mod_marker(attractor_strength, defaults.attractor_strength, 0.01),
                    attractor_strength
                ));
                content.push(format!("   ,    Mouse Mode        {:>14}", mouse_mode));
                if mouse_mode != "Disabled" {
                    content.push(format!(
                        "   ─    Mouse Timeout {:>4.1}s (CLI-only)",
                        mouse_timeout
                    ));
                }
            }
            2 => {
                content.push(format!("   c/C  Palette           {:>14}", palette_name));
                content.push(format!(
                    "   O    Palette Shift     {:>14}",
                    match palette_shift_speed {
                        PaletteShiftSpeed::Off => "Off",
                        PaletteShiftSpeed::Slow => "Slow",
                        PaletteShiftSpeed::Medium => "Medium",
                        PaletteShiftSpeed::Fast => "Fast",
                    }
                ));
                content.push(format!(
                    "   X    Invert Palette    {:>14}",
                    if invert_palette { "On" } else { "Off" }
                ));
                content.push(format!(
                    "   Z    Reverse Palette   {:>14}",
                    if reverse_palette { "On" } else { "Off" }
                ));
            }
            3 => {
                content.push(format!(
                    "   d/D  Dither Mode       {:>14}",
                    dither_mode_name
                ));
                content.push(format!(
                    "{} B    Auto Normalize    {:>14}",
                    if auto_normalize != defaults.auto_normalize {
                        "*"
                    } else {
                        " "
                    },
                    if auto_normalize { "On" } else { "Off" }
                ));
                content.push(format!(
                    "{} V    Motion Blur    {:>1} frames [0,3,5,7]",
                    mod_marker_int(motion_blur_frames, defaults.motion_blur_frames),
                    motion_blur_frames
                ));
                content.push(format!(
                    "{} N/n  Max Bright    {:>5.1}x [1-100]",
                    mod_marker(max_brightness, defaults.max_brightness, 0.01),
                    max_brightness
                ));
            }
            4 => {
                content.push(format!(
                    "   F    Fast Mode         {:>14}",
                    if fast_mode_enabled { "On" } else { "Off" }
                ));
                content.push(format!(
                    "   ─    Population      {:>3}k (fixed)",
                    population / 1000
                ));
            }
            5 => {
                content.push("   G    Save Frame             (PNG)".to_string());
                content.push("   0    Reset to Defaults".to_string());
                content.push("   8    Randomize Parameters".to_string());
            }
            _ => {}
        }

        content.push("".to_string());
        content.push("   * Modified from default value".to_string());
        content.push("   ─ Startup-only parameter (CLI)".to_string());
        content.push("   Tab: Next         Esc: Close".to_string());

        let title = format!("CONTROLS [{}/{}]", cat_num, Self::TOTAL_CATEGORIES);
        builder.build(Some(&title), &content).unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::simulation::config::DiffusionKernel;
    use crate::simulation::config::Preset;
    use crate::simulation::config::TerrainType;
    use crate::terminal::control::PaletteShiftSpeed;
    use crate::terminal::control::WindDirection;

    #[test]
    fn test_category_names() {
        assert_eq!(ControlsOverlay::category_name(0), "SIMULATION CORE");
        assert_eq!(ControlsOverlay::category_name(1), "FORCES & ENVIRONMENT");
        assert_eq!(ControlsOverlay::category_name(2), "APPEARANCE");
        assert_eq!(ControlsOverlay::category_name(3), "POST-PROCESSING");
        assert_eq!(ControlsOverlay::category_name(4), "PERFORMANCE");
        assert_eq!(ControlsOverlay::category_name(5), "SYSTEM");
    }

    #[test]
    fn test_total_categories() {
        assert_eq!(ControlsOverlay::total_categories(), 6);
    }

    #[test]
    fn test_overlay_has_correct_border_format() {
        let lines = ControlsOverlay::build_overlay(
            0,
            22.5,
            9.0,
            45.0,
            1.0,
            0.5,
            5.0,
            1.0,
            DiffusionKernel::Mean3x3,
            1.0,
            1.0,
            "Disabled",
            3.0,
            WindDirection::None,
            TerrainType::None,
            1.0,
            false,
            0,
            20.0,
            false,
            "Forest",
            PaletteShiftSpeed::Off,
            false,
            false,
            "None",
            80,
            DefaultValues::from_preset(Preset::Organic),
            50000,
        );

        assert!(lines[0].starts_with('╭'), "First line should start with ╭");
        assert!(lines[0].ends_with('╮'), "First line should end with ╮");
        assert!(
            lines.last().unwrap().starts_with('╰'),
            "Last line should start with ╰"
        );
        assert!(
            lines.last().unwrap().ends_with('╯'),
            "Last line should end with ╯"
        );
    }

    #[test]
    fn test_overlay_all_lines_consistent_width() {
        for category_idx in 0..6 {
            let lines = ControlsOverlay::build_overlay(
                category_idx,
                22.5,
                9.0,
                45.0,
                1.0,
                0.5,
                5.0,
                1.0,
                DiffusionKernel::Mean3x3,
                1.0,
                1.0,
                "Disabled",
                3.0,
                WindDirection::None,
                TerrainType::None,
                1.0,
                false,
                0,
                20.0,
                false,
                "Forest",
                PaletteShiftSpeed::Medium,
                false,
                false,
                "None",
                80,
                DefaultValues::from_preset(Preset::Organic),
                50000,
            );

            // All lines should be exactly WIDTH chars wide
            for (line_num, line) in lines.iter().enumerate() {
                assert!(
                    line.starts_with('╭')
                        || line.starts_with('│')
                        || line.starts_with('╰')
                        || line.starts_with('├'),
                    "Category {}, line {}: All lines should start with border character",
                    category_idx,
                    line_num
                );
                assert!(
                    line.ends_with('╮')
                        || line.ends_with('│')
                        || line.ends_with('╯')
                        || line.ends_with('┤'),
                    "Category {}, line {}: All lines should end with border character",
                    category_idx,
                    line_num
                );
                assert_eq!(
                    line.chars().count(),
                    ControlsOverlay::WIDTH,
                    "Category {}, line {}: '{}' has unexpected length {}",
                    category_idx,
                    line_num,
                    line,
                    line.chars().count()
                );
            }
        }
    }

    #[test]
    fn test_overlay_has_category_indicator() {
        let lines = ControlsOverlay::build_overlay(
            2,
            22.5,
            9.0,
            45.0,
            1.0,
            0.5,
            5.0,
            1.0,
            DiffusionKernel::Mean3x3,
            1.0,
            1.0,
            "Disabled",
            3.0,
            WindDirection::None,
            TerrainType::None,
            1.0,
            false,
            0,
            20.0,
            false,
            "Forest",
            PaletteShiftSpeed::Off,
            false,
            false,
            "None",
            80,
            DefaultValues::from_preset(Preset::Organic),
            50000,
        );

        // First line should contain [3/6] indicator
        assert!(
            lines[0].contains("[3/6]"),
            "Header should contain category indicator [3/6], got: {}",
            lines[0]
        );
    }

    #[test]
    fn test_wind_direction_names() {
        assert_eq!(WindDirection::None.name(), "None");
        assert_eq!(WindDirection::North.name(), "N");
        assert_eq!(WindDirection::Northeast.name(), "NE");
        assert_eq!(WindDirection::East.name(), "E");
        assert_eq!(WindDirection::Southeast.name(), "SE");
        assert_eq!(WindDirection::South.name(), "S");
        assert_eq!(WindDirection::Southwest.name(), "SW");
        assert_eq!(WindDirection::West.name(), "W");
        assert_eq!(WindDirection::Northwest.name(), "NW");
    }

    #[test]
    fn test_palette_shift_speed_degrees() {
        assert_eq!(PaletteShiftSpeed::Off.degrees_per_second(), 0.0);
        assert_eq!(PaletteShiftSpeed::Slow.degrees_per_second(), 5.0);
        assert_eq!(PaletteShiftSpeed::Medium.degrees_per_second(), 15.0);
        assert_eq!(PaletteShiftSpeed::Fast.degrees_per_second(), 45.0);
    }
}

#[test]
fn test_options_overlay_renders_all_categories() {
    use crate::simulation::config::InitMode;
    use crate::terminal::control::RuntimeState;

    let state = RuntimeState::new(
        42,
        InitMode::Random,
        crate::simulation::config::Preset::Organic,
        0,
        crate::terminal::control::MouseInteractionMode::Disabled,
        0.0,
    );

    let total = OptionsOverlay::total_categories();
    assert_eq!(total, 6);

    for idx in 0..total {
        let overlay = OptionsOverlay::build_overlay(
            idx,
            state.sensor_angle,
            state.sensor_distance,
            state.turn_angle,
            state.step_size,
            state.decay_factor,
            state.deposit_amount,
            state.time_scale,
            state.diffusion_kernel,
            state.diffusion_sigma,
            state.attractor_strength,
            "Disabled",
            state.mouse_timeout,
            state.wind_direction,
            state.terrain_type,
            state.terrain_strength,
            state.auto_normalize,
            state.motion_blur_frames,
            state.max_brightness,
            state.fast_mode_enabled,
            "Forest",
            state.palette_shift_speed,
            state.invert_palette,
            state.reverse_palette,
            "None",
            80,
            state.default_values,
            50000,
        );

        assert!(!overlay.is_empty(), "Category {} should not be empty", idx);

        let category_name = OptionsOverlay::category_name(idx);
        assert!(
            overlay.iter().any(|line| line.contains(category_name)),
            "Category {} should contain its name '{}'",
            idx,
            category_name
        );

        assert!(!overlay.is_empty(), "Category {} should have lines", idx);
    }

    let sim_overlay = OptionsOverlay::build_overlay(
        0,
        state.sensor_angle,
        state.sensor_distance,
        state.turn_angle,
        state.step_size,
        state.decay_factor,
        state.deposit_amount,
        state.time_scale,
        state.diffusion_kernel,
        state.diffusion_sigma,
        state.attractor_strength,
        "Disabled",
        state.mouse_timeout,
        state.wind_direction,
        state.terrain_type,
        state.terrain_strength,
        state.auto_normalize,
        state.motion_blur_frames,
        state.max_brightness,
        state.fast_mode_enabled,
        "Forest",
        state.palette_shift_speed,
        state.invert_palette,
        state.reverse_palette,
        "None",
        80,
        state.default_values,
        50000,
    );
    assert!(sim_overlay.iter().any(|line| line.contains("Sensor Angle")));
    assert!(sim_overlay.iter().any(|line| line.contains("Sensor Dist")));
    assert!(sim_overlay.iter().any(|line| line.contains("Turn Angle")));
    assert!(sim_overlay.iter().any(|line| line.contains("Step Size")));

    let env_overlay = OptionsOverlay::build_overlay(
        1,
        state.sensor_angle,
        state.sensor_distance,
        state.turn_angle,
        state.step_size,
        state.decay_factor,
        state.deposit_amount,
        state.time_scale,
        state.diffusion_kernel,
        state.diffusion_sigma,
        state.attractor_strength,
        "Disabled",
        state.mouse_timeout,
        state.wind_direction,
        state.terrain_type,
        state.terrain_strength,
        state.auto_normalize,
        state.motion_blur_frames,
        state.max_brightness,
        state.fast_mode_enabled,
        "Forest",
        state.palette_shift_speed,
        state.invert_palette,
        state.reverse_palette,
        "None",
        80,
        state.default_values,
        50000,
    );
    assert!(env_overlay.iter().any(|line| line.contains("Diffusion")));
    assert!(env_overlay.iter().any(|line| line.contains("Wind")));
    assert!(env_overlay.iter().any(|line| line.contains("Terrain")));
}

#[test]
fn test_options_overlay_shows_live_parameter_values() {
    use crate::simulation::config::InitMode;
    use crate::terminal::control::RuntimeState;

    let mut state = RuntimeState::new(
        42,
        InitMode::Random,
        crate::simulation::config::Preset::Organic,
        0,
        crate::terminal::control::MouseInteractionMode::Disabled,
        0.0,
    );

    state.max_brightness = 100.0;
    state.motion_blur_frames = 3;

    let postprocessing_overlay = OptionsOverlay::build_overlay(
        3,
        state.sensor_angle,
        state.sensor_distance,
        state.turn_angle,
        state.step_size,
        state.decay_factor,
        state.deposit_amount,
        state.time_scale,
        state.diffusion_kernel,
        state.diffusion_sigma,
        state.attractor_strength,
        "Disabled",
        state.mouse_timeout,
        state.wind_direction,
        state.terrain_type,
        state.terrain_strength,
        state.auto_normalize,
        state.motion_blur_frames,
        state.max_brightness,
        state.fast_mode_enabled,
        "Forest",
        state.palette_shift_speed,
        state.invert_palette,
        state.reverse_palette,
        "None",
        80,
        state.default_values,
        50000,
    );

    assert!(
        postprocessing_overlay
            .iter()
            .any(|line| line.contains("100.0") || line.contains("100")),
        "Should contain max brightness value. Got: {:?}",
        postprocessing_overlay
    );

    assert!(
        postprocessing_overlay
            .iter()
            .any(|line| line.contains("3") && line.contains("[0,3,5,7]")),
        "Should contain motion blur frames value. Got: {:?}",
        postprocessing_overlay
    );
}

#[test]
fn test_options_overlay_format() {
    use crate::simulation::config::InitMode;
    use crate::terminal::control::RuntimeState;

    let state = RuntimeState::new(
        42,
        InitMode::Random,
        crate::simulation::config::Preset::Organic,
        0,
        crate::terminal::control::MouseInteractionMode::Disabled,
        0.0,
    );

    for idx in 0..OptionsOverlay::total_categories() {
        let overlay = OptionsOverlay::build_overlay(
            idx,
            state.sensor_angle,
            state.sensor_distance,
            state.turn_angle,
            state.step_size,
            state.decay_factor,
            state.deposit_amount,
            state.time_scale,
            state.diffusion_kernel,
            state.diffusion_sigma,
            state.attractor_strength,
            "Disabled",
            state.mouse_timeout,
            state.wind_direction,
            state.terrain_type,
            state.terrain_strength,
            state.auto_normalize,
            state.motion_blur_frames,
            state.max_brightness,
            state.fast_mode_enabled,
            "Forest",
            state.palette_shift_speed,
            state.invert_palette,
            state.reverse_palette,
            "None",
            80,
            state.default_values,
            50000,
        );

        assert!(
            overlay.iter().any(|line| line.starts_with("╭─")),
            "Should have top border"
        );
        assert!(
            overlay.iter().any(|line| line.contains("─")),
            "Should have horizontal border"
        );
        assert!(
            overlay.iter().any(|line| line.contains("│")),
            "Should have vertical border"
        );
        assert!(
            overlay
                .iter()
                .any(|line| line.ends_with("╯") || line.ends_with("│")),
            "Should have bottom corners"
        );

        for (i, line) in overlay.iter().enumerate() {
            assert_eq!(
                line.chars().count(),
                ControlsOverlay::WIDTH,
                "Line {} should be {} chars, got {}: {}",
                i,
                ControlsOverlay::WIDTH,
                line.chars().count(),
                line
            );
        }
    }
}
