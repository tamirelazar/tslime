use crate::simulation::config::DiffusionKernel;
use crate::simulation::config::TerrainType;
use crate::terminal::control::PaletteShiftSpeed;
use crate::terminal::control::WindDirection;

// Renamed from OptionsOverlay - this is now the Controls overlay
pub struct ControlsOverlay;

// Type alias for backwards compatibility
#[allow(dead_code)]
pub type OptionsOverlay = ControlsOverlay;

impl ControlsOverlay {
    #[allow(dead_code)]
    pub const WIDTH: usize = 42;
    pub const TOTAL_CATEGORIES: usize = 6;

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
    pub fn total_categories() -> usize {
        Self::TOTAL_CATEGORIES
    }

    #[allow(clippy::too_many_arguments)]
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
    ) -> Vec<String> {
        let mut lines = Vec::new();
        let cat_name = Self::category_name(category_idx);
        let cat_num = category_idx + 1;

        // All lines are exactly 42 characters wide
        // Header with category indicator [1/5]
        lines.push(format!(
            "╭─ CONTROLS [{}/{}] ───────────────────────╮",
            cat_num,
            Self::TOTAL_CATEGORIES
        ));

        match category_idx {
            // Category 0: SIMULATION CORE
            0 => {
                lines.push(format!("│ {:^38} │", cat_name));
                lines.push("│                                        │".to_string());
                lines.push(format!(
                    "│  A/a  Sensor Angle         {:>6.1}°     │",
                    sensor_angle
                ));
                lines.push(format!(
                    "│  J/j  Sensor Distance      {:>6.1}      │",
                    sensor_distance
                ));
                lines.push(format!(
                    "│  T/t  Turn Angle           {:>6.1}°     │",
                    turn_angle
                ));
                lines.push(format!(
                    "│  S/s  Step Size            {:>6.1}      │",
                    step_size
                ));
                lines.push(format!(
                    "│  E/e  Decay Factor         {:>6.3}      │",
                    decay_factor
                ));
                lines.push(format!(
                    "│  I/i  Deposit Amount       {:>6.1}      │",
                    deposit_amount
                ));
                lines.push(format!(
                    "│  +/-  Time Scale           {:>6.1}x     │",
                    time_scale
                ));
            }
            // Category 1: FORCES & ENVIRONMENT
            1 => {
                lines.push(format!("│ {:^38} │", cat_name));
                lines.push("│                                        │".to_string());
                lines.push(format!(
                    "│  K    Diffusion         {:>14} │",
                    match diffusion_kernel {
                        DiffusionKernel::Mean3x3 => "Mean3x3",
                        DiffusionKernel::Gaussian => "Gaussian",
                    }
                ));
                // Only show diffusion_sigma when Gaussian kernel is selected
                if matches!(diffusion_kernel, DiffusionKernel::Gaussian) {
                    lines.push(format!(
                        "│  ;/:  Diffusion Sigma   {:>14.2} │",
                        diffusion_sigma
                    ));
                }
                lines.push(format!(
                    "│  W    Wind              {:>14} │",
                    wind_direction.name()
                ));
                lines.push(format!(
                    "│  U    Terrain Type      {:>14} │",
                    match terrain_type {
                        TerrainType::None => "None",
                        TerrainType::Smooth => "Smooth",
                        TerrainType::Turbulent => "Turbulent",
                        TerrainType::Mixed => "Mixed",
                    }
                ));
                lines.push(format!(
                    "│  Y/y  Terrain Str       {:>14.1} │",
                    terrain_strength
                ));
                lines.push(format!(
                    "│  L/l  Attractor Str     {:>14.1} │",
                    attractor_strength
                ));
                lines.push(format!(
                    "│  ,    Mouse Mode        {:>14} │",
                    mouse_mode
                ));
                if mouse_mode != "Disabled" {
                    lines.push(format!(
                        "│       Mouse Timeout     {:>12.1}s │",
                        mouse_timeout
                    ));
                }
            }
            // Category 2: APPEARANCE
            2 => {
                lines.push(format!("│ {:^38} │", cat_name));
                lines.push("│                                        │".to_string());
                lines.push(format!(
                    "│  c/C  Palette           {:>14} │",
                    palette_name
                ));
                lines.push(format!(
                    "│  O    Palette Shift     {:>14} │",
                    match palette_shift_speed {
                        PaletteShiftSpeed::Off => "Off",
                        PaletteShiftSpeed::Slow => "Slow",
                        PaletteShiftSpeed::Medium => "Medium",
                        PaletteShiftSpeed::Fast => "Fast",
                    }
                ));
                lines.push(format!(
                    "│  X    Invert Palette    {:>14} │",
                    if invert_palette { "On" } else { "Off" }
                ));
                lines.push(format!(
                    "│  Z    Reverse Palette   {:>14} │",
                    if reverse_palette { "On" } else { "Off" }
                ));
            }
            // Category 3: POST-PROCESSING
            3 => {
                lines.push(format!("│ {:^38} │", cat_name));
                lines.push("│                                        │".to_string());
                lines.push(format!(
                    "│  d/D  Dither Mode       {:>14} │",
                    dither_mode_name
                ));
                lines.push(format!(
                    "│  B    Auto Normalize    {:>14} │",
                    if auto_normalize { "On" } else { "Off" }
                ));
                lines.push(format!(
                    "│  V    Motion Blur     {:>10} frames│",
                    motion_blur_frames
                ));
                lines.push(format!(
                    "│  N/n  Max Brightness    {:>14.1} │",
                    max_brightness
                ));
            }
            // Category 4: PERFORMANCE
            4 => {
                lines.push(format!("│ {:^38} │", cat_name));
                lines.push("│                                        │".to_string());
                lines.push(format!(
                    "│  F    Fast Mode         {:>14} │",
                    if fast_mode_enabled { "On" } else { "Off" }
                ));
            }
            // Category 5: SYSTEM
            5 => {
                lines.push(format!("│ {:^38} │", cat_name));
                lines.push("│                                        │".to_string());
                lines.push("│  G    Save Frame             (PNG)     │".to_string());
                lines.push("│  0    Reset to Defaults                │".to_string());
                lines.push("│  8    Randomize Parameters             │".to_string());
            }
            _ => {}
        }

        lines.push("│                                        │".to_string());
        lines.push("│  Tab: Next         Esc: Close          │".to_string());
        lines.push("╰────────────────────────────────────────╯".to_string());

        lines
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::simulation::config::DiffusionKernel;
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
            );

            // All lines should be exactly 42 chars wide
            for (line_num, line) in lines.iter().enumerate() {
                assert!(
                    line.starts_with('╭') || line.starts_with('│') || line.starts_with('╰'),
                    "Category {}, line {}: All lines should start with border character",
                    category_idx,
                    line_num
                );
                assert!(
                    line.ends_with('╮') || line.ends_with('│') || line.ends_with('╯'),
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
fn test_help_overlay_format() {
    use crate::render::overlay::HelpOverlay;

    let help_lines = HelpOverlay::build_overlay();

    for line in &help_lines {
        assert!(line.starts_with('│') || line.starts_with('╭') || line.starts_with('╰'));
        assert!(line.ends_with('│') || line.ends_with('╮') || line.ends_with('╯'));
    }

    // All lines should be 42 chars wide
    for line in &help_lines {
        assert_eq!(
            line.chars().count(),
            HelpOverlay::width(),
            "Help line has unexpected width: {}",
            line
        );
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
        false,
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
    );
    assert!(sim_overlay.iter().any(|line| line.contains("Sensor Angle")));
    assert!(sim_overlay.iter().any(|line| line.contains("Sensor Distance")));
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
        false,
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
            .any(|line| line.contains("3") && line.contains("frames")),
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
        false,
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
