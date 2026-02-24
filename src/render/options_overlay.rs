use crate::render::panel::{Padding, PanelBuilder, TextAlignment};
use crate::simulation::config::DiffusionKernel;
use crate::simulation::config::TerrainType;
use crate::terminal::control::DefaultValues;
use crate::terminal::control::PaletteShiftSpeed;
use crate::terminal::control::WindDirection;

/// Overlay that displays current simulation controls and parameters.
pub struct ControlsOverlay;

impl ControlsOverlay {
    /// Total rendered width of the controls overlay window.
    pub const WIDTH: usize = 50;
    /// Content width (inner drawable area).
    const CONTENT_WIDTH: usize = 44; // 50 - 2(border) - 2*2(padding)
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
    ) -> crate::render::panel::RenderedOverlay {
        use TextAlignment::Left;

        let cat_name = Self::category_name(category_idx);
        let cat_num = category_idx + 1;
        let title = format!("CONTROLS [{}/{}]", cat_num, Self::TOTAL_CATEGORIES);

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

        let mut builder = PanelBuilder::new(Self::CONTENT_WIDTH, None)
            .with_padding(Padding::new(1, 1, 2, 2))
            .with_title(title)
            .with_title_box()
            .add_single(
                format!("{:^width$}", cat_name, width = Self::CONTENT_WIDTH),
                Left,
            )
            .add_empty();

        builder = match category_idx {
            0 => builder
                .add_single(
                    format!(
                        "{} A/a  Sensor Angle  {:>5.1}° [5-90°]",
                        mod_marker(sensor_angle, defaults.sensor_angle, 0.01),
                        sensor_angle
                    ),
                    Left,
                )
                .add_single(
                    format!(
                        "{} J/j  Sensor Dist   {:>5.1}px [1-50]",
                        mod_marker(sensor_distance, defaults.sensor_distance, 0.01),
                        sensor_distance
                    ),
                    Left,
                )
                .add_single(
                    format!(
                        "{} T/t  Turn Angle    {:>5.1}° [5-90°]",
                        mod_marker(turn_angle, defaults.turn_angle, 0.01),
                        turn_angle
                    ),
                    Left,
                )
                .add_single(
                    format!(
                        "{} S/s  Step Size     {:>5.1}px [0.5-5.0]",
                        mod_marker(step_size, defaults.step_size, 0.01),
                        step_size
                    ),
                    Left,
                )
                .add_single(
                    format!(
                        "{} E/e  Decay Factor  {:>5.3}x [0.5-0.99]",
                        mod_marker(decay_factor, defaults.decay_factor, 0.001),
                        decay_factor
                    ),
                    Left,
                )
                .add_single(
                    format!(
                        "{} I/i  Deposit Amt   {:>5.1}x [1-20]",
                        mod_marker(deposit_amount, defaults.deposit_amount, 0.01),
                        deposit_amount
                    ),
                    Left,
                )
                .add_single(
                    format!("   +/-  Time Scale    {:>5.1}x [0.5-4x]", time_scale),
                    Left,
                ),
            1 => {
                let kernel_name = match diffusion_kernel {
                    DiffusionKernel::Mean3x3 => "Mean3x3",
                    DiffusionKernel::Gaussian => "Gaussian",
                };
                let mut b = builder.add_single(
                    format!(
                        "{} K    Diffusion         {:>14}",
                        mod_marker_enum(&diffusion_kernel, &defaults.diffusion_kernel),
                        kernel_name
                    ),
                    Left,
                );
                if matches!(diffusion_kernel, DiffusionKernel::Gaussian) {
                    b = b.add_single(
                        format!(
                            "{} ;/:  Diff Sigma  {:>5.2}x [0.5-2.0]",
                            mod_marker(diffusion_sigma, defaults.diffusion_sigma, 0.01),
                            diffusion_sigma
                        ),
                        Left,
                    );
                }
                let terrain_name = match terrain_type {
                    TerrainType::None => "None",
                    TerrainType::Smooth => "Smooth",
                    TerrainType::Turbulent => "Turbulent",
                    TerrainType::Mixed => "Mixed",
                };
                b = b
                    .add_single(
                        format!(
                            "{} W    Wind              {:>14}",
                            mod_marker_enum(&wind_direction, &defaults.wind_direction),
                            wind_direction.name()
                        ),
                        Left,
                    )
                    .add_single(
                        format!(
                            "{} U    Terrain Type      {:>14}",
                            mod_marker_enum(&terrain_type, &defaults.terrain_type),
                            terrain_name
                        ),
                        Left,
                    )
                    .add_single(
                        format!(
                            "{} Y/y  Terrain Str   {:>5.1}x [0.1-5.0]",
                            mod_marker(terrain_strength, defaults.terrain_strength, 0.01),
                            terrain_strength
                        ),
                        Left,
                    )
                    .add_single(
                        format!(
                            "{} L/l  Attractor Str {:>5.1}x [0.1-10]",
                            mod_marker(attractor_strength, defaults.attractor_strength, 0.01),
                            attractor_strength
                        ),
                        Left,
                    )
                    .add_single(
                        format!("   ,    Mouse Mode        {:>14}", mouse_mode),
                        Left,
                    );
                if mouse_mode != "Disabled" {
                    b = b.add_single(
                        format!("   ─    Mouse Timeout {:>4.1}s (CLI-only)", mouse_timeout),
                        Left,
                    );
                }
                b
            }
            2 => {
                let shift_name = match palette_shift_speed {
                    PaletteShiftSpeed::Off => "Off",
                    PaletteShiftSpeed::Slow => "Slow",
                    PaletteShiftSpeed::Medium => "Medium",
                    PaletteShiftSpeed::Fast => "Fast",
                };
                builder
                    .add_single(
                        format!("   c/C  Palette           {:>14}", palette_name),
                        Left,
                    )
                    .add_single(
                        format!("   O    Palette Shift     {:>14}", shift_name),
                        Left,
                    )
                    .add_single(
                        format!(
                            "   X    Invert Palette    {:>14}",
                            if invert_palette { "On" } else { "Off" }
                        ),
                        Left,
                    )
                    .add_single(
                        format!(
                            "   Z    Reverse Palette   {:>14}",
                            if reverse_palette { "On" } else { "Off" }
                        ),
                        Left,
                    )
            }
            3 => builder
                .add_single(
                    format!("   d/D  Dither Mode       {:>14}", dither_mode_name),
                    Left,
                )
                .add_single(
                    format!(
                        "{} B    Auto Normalize    {:>14}",
                        if auto_normalize != defaults.auto_normalize {
                            "*"
                        } else {
                            " "
                        },
                        if auto_normalize { "On" } else { "Off" }
                    ),
                    Left,
                )
                .add_single(
                    format!(
                        "{} V    Motion Blur    {:>1} frames [0,3,5,7]",
                        mod_marker_int(motion_blur_frames, defaults.motion_blur_frames),
                        motion_blur_frames
                    ),
                    Left,
                )
                .add_single(
                    format!(
                        "{} N/n  Max Bright    {:>5.1}x [1-100]",
                        mod_marker(max_brightness, defaults.max_brightness, 0.01),
                        max_brightness
                    ),
                    Left,
                ),
            4 => builder
                .add_single(
                    format!(
                        "   F    Fast Mode         {:>14}",
                        if fast_mode_enabled { "On" } else { "Off" }
                    ),
                    Left,
                )
                .add_single(
                    format!("   ─    Population      {:>3}k (fixed)", population / 1000),
                    Left,
                ),
            5 => builder
                .add_single("   G    Save Frame             (PNG)".to_string(), Left)
                .add_single("   0    Reset to Defaults".to_string(), Left)
                .add_single("   8    Randomize Parameters".to_string(), Left),
            _ => builder,
        };

        builder
            .add_empty()
            .add_single("   * Modified from default value".to_string(), Left)
            .add_single("   ─ Startup-only parameter (CLI)".to_string(), Left)
            .add_single("   Tab: Next         Esc: Close".to_string(), Left)
            .build_overlay()
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

        assert!(lines.lines[0].starts_with('▀'), "First line should start with ▀");
        assert!(lines.lines[0].ends_with('▀'), "First line should end with ▀");
        assert!(
            lines.lines.last().unwrap().starts_with('▄'),
            "Last line should start with ▄"
        );
        assert!(
            lines.lines.last().unwrap().ends_with('▄'),
            "Last line should end with ▄"
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
            for (line_num, line) in lines.lines.iter().enumerate() {
                assert!(
                    line.starts_with('▀')
                        || line.starts_with('█')
                        || line.starts_with('▄')
                        || line.starts_with('▌'),
                    "Category {}, line {}: All lines should start with solid block char",
                    category_idx,
                    line_num
                );
                assert!(
                    line.ends_with('▀')
                        || line.ends_with('█')
                        || line.ends_with('▄')
                        || line.ends_with('▐'),
                    "Category {}, line {}: All lines should end with solid block char",
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

        // First line (top border) should contain [3/6] indicator
        assert!(
            lines.lines[0].contains("[3/6]"),
            "Header should contain category indicator [3/6], got: {}",
            lines.lines[0]
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
        crate::render::palette::IntensityMapping::linear(),
    );

    let total = ControlsOverlay::total_categories();
    assert_eq!(total, 6);

    for idx in 0..total {
        let overlay = ControlsOverlay::build_overlay(
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

        assert!(!overlay.lines.is_empty(), "Category {} should not be empty", idx);

        let category_name = ControlsOverlay::category_name(idx);
        assert!(
            overlay.lines.iter().any(|line| line.contains(category_name)),
            "Category {} should contain its name '{}'",
            idx,
            category_name
        );

        assert!(!overlay.lines.is_empty(), "Category {} should have lines", idx);
    }

    let sim_overlay = ControlsOverlay::build_overlay(
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
    assert!(sim_overlay.lines.iter().any(|line| line.contains("Sensor Angle")));
    assert!(sim_overlay.lines.iter().any(|line| line.contains("Sensor Dist")));
    assert!(sim_overlay.lines.iter().any(|line| line.contains("Turn Angle")));
    assert!(sim_overlay.lines.iter().any(|line| line.contains("Step Size")));

    let env_overlay = ControlsOverlay::build_overlay(
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
    assert!(env_overlay.lines.iter().any(|line| line.contains("Diffusion")));
    assert!(env_overlay.lines.iter().any(|line| line.contains("Wind")));
    assert!(env_overlay.lines.iter().any(|line| line.contains("Terrain")));
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
        crate::render::palette::IntensityMapping::linear(),
    );

    state.max_brightness = 100.0;
    state.motion_blur_frames = 3;

    let postprocessing_overlay = ControlsOverlay::build_overlay(
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
            .lines
            .iter()
            .any(|line| line.contains("100.0") || line.contains("100")),
        "Should contain max brightness value. Got: {:?}",
        postprocessing_overlay.lines
    );

    assert!(
        postprocessing_overlay
            .lines
            .iter()
            .any(|line| line.contains("3") && line.contains("[0,3,5,7]")),
        "Should contain motion blur frames value. Got: {:?}",
        postprocessing_overlay.lines
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
        crate::render::palette::IntensityMapping::linear(),
    );

    for idx in 0..ControlsOverlay::total_categories() {
        let overlay = ControlsOverlay::build_overlay(
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

        // Solid block borders
        assert!(
            overlay.lines.iter().any(|line| line.starts_with('▀')),
            "Should have solid block top border"
        );
        assert!(
            overlay.lines.iter().any(|line| line.contains('█')),
            "Should have solid block vertical border"
        );
        assert!(
            overlay
                .lines
                .iter()
                .any(|line| line.ends_with('▄') || line.ends_with('█')),
            "Should have bottom border or vertical chars"
        );

        for (i, line) in overlay.lines.iter().enumerate() {
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
