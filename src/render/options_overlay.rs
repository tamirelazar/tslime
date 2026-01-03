use crate::simulation::config::DiffusionKernel;
use crate::simulation::config::TerrainType;
use crate::terminal::control::PaletteShiftSpeed;
use crate::terminal::control::WindDirection;

pub struct OptionsOverlay;

impl OptionsOverlay {
    pub fn category_name(idx: usize) -> &'static str {
        match idx {
            0 => "SIMULATION",
            1 => "ENVIRONMENT",
            2 => "VISUAL EFFECTS",
            3 => "RENDERING",
            4 => "DISPLAY",
            _ => "UNKNOWN",
        }
    }

    #[allow(dead_code)]
    pub fn total_categories() -> usize {
        5
    }

    #[allow(clippy::too_many_arguments)]
    pub fn build_overlay(
        category_idx: usize,
        sensor_angle: f32,
        turn_angle: f32,
        step_size: f32,
        decay_factor: f32,
        deposit_amount: f32,
        diffusion_kernel: DiffusionKernel,
        wind_direction: WindDirection,
        terrain_type: TerrainType,
        terrain_strength: f32,
        auto_normalize: bool,
        motion_blur_frames: usize,
        max_brightness: f32,
        fast_mode_enabled: bool,
        palette_shift_speed: PaletteShiftSpeed,
        invert_palette: bool,
        reverse_palette: bool,
        _term_width: usize,
    ) -> Vec<String> {
        let mut lines = Vec::new();

        lines.push(format!(
            "┌─ OPTIONS: {} ──────────────────┐",
            Self::category_name(category_idx)
        ));
        lines.push("│                                        │".to_string());

        match category_idx {
            0 => {
                lines.push("│  SIMULATION PARAMETERS                 │".to_string());
                lines.push("│                                        │".to_string());
                lines.push(format!(
                    "│  A/a: Sensor Angle     {:>5.1}°       │",
                    sensor_angle
                ));
                lines.push(format!(
                    "│  T/t: Turn Angle       {:>5.1}°       │",
                    turn_angle
                ));
                lines.push(format!(
                    "│  S/s: Step Size        {:>5.1}        │",
                    step_size
                ));
                lines.push(format!(
                    "│  E/e: Decay Factor     {:>5.3}        │",
                    decay_factor
                ));
                lines.push(format!(
                    "│  I/i: Deposit Amount   {:>5.1}        │",
                    deposit_amount
                ));
            }
            1 => {
                lines.push("│  ENVIRONMENT & PHYSICS                 │".to_string());
                lines.push("│                                        │".to_string());
                lines.push(format!(
                    "│  K:   Diffusion Kernel  {:>12}         │",
                    match diffusion_kernel {
                        DiffusionKernel::Mean3x3 => "Mean3x3",
                        DiffusionKernel::Gaussian => "Gaussian",
                    }
                ));
                lines.push(format!(
                    "│  W:   Wind Direction     {:>12}         │",
                    wind_direction.name()
                ));
                lines.push(format!(
                    "│  Y/y: Terrain Strength  {:>5.1}         │",
                    terrain_strength
                ));
                lines.push(format!(
                    "│  U:   Terrain Type       {:>12}         │",
                    match terrain_type {
                        TerrainType::None => "None",
                        TerrainType::Smooth => "Smooth",
                        TerrainType::Turbulent => "Turbulent",
                        TerrainType::Mixed => "Mixed",
                    }
                ));
            }
            2 => {
                lines.push("│  VISUAL EFFECTS                        │".to_string());
                lines.push("│                                        │".to_string());
                lines.push(format!(
                    "│  B:   Auto Normalize     {:>12}        │",
                    if auto_normalize { "On" } else { "Off" }
                ));
                lines.push(format!(
                    "│  V:   Motion Blur        {:>5} frames  │",
                    motion_blur_frames
                ));
                lines.push(format!(
                    "│  N/n: Max Brightness    {:>5.1}         │",
                    max_brightness
                ));
            }
            3 => {
                lines.push("│  RENDERING OPTIONS                     │".to_string());
                lines.push("│                                        │".to_string());
                lines.push("│  G:   Save Frame      (PNG format)    │".to_string());
                lines.push(format!(
                    "│  F:   Fast Mode          {:>12}        │",
                    if fast_mode_enabled { "On" } else { "Off" }
                ));
                lines.push(format!(
                    "│  O:   Palette Shift      {:>12}        │",
                    match palette_shift_speed {
                        PaletteShiftSpeed::Off => "Off",
                        PaletteShiftSpeed::Slow => "Slow (5°/s)",
                        PaletteShiftSpeed::Medium => "Med (15°/s)",
                        PaletteShiftSpeed::Fast => "Fast (45°/s)",
                    }
                ));
            }
            4 => {
                lines.push("│  DISPLAY CONTROLS                      │".to_string());
                lines.push("│                                        │".to_string());
                lines.push(format!(
                    "│  X:   Invert Palette    {:>12}        │",
                    if invert_palette { "On" } else { "Off" }
                ));
                lines.push(format!(
                    "│  Z:   Reverse Palette   {:>12}        │",
                    if reverse_palette { "On" } else { "Off" }
                ));
                lines.push("│  0:   Reset to Defaults                │".to_string());
            }
            _ => {}
        }

        lines.push("│                                        │".to_string());
        lines.push("│  Tab: Next Category    Esc: Close      │".to_string());
        lines.push("└────────────────────────────────────────┘".to_string());

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
        assert_eq!(OptionsOverlay::category_name(0), "SIMULATION");
        assert_eq!(OptionsOverlay::category_name(1), "ENVIRONMENT");
        assert_eq!(OptionsOverlay::category_name(2), "VISUAL EFFECTS");
        assert_eq!(OptionsOverlay::category_name(3), "RENDERING");
        assert_eq!(OptionsOverlay::category_name(4), "DISPLAY");
    }

    #[test]
    fn test_total_categories() {
        assert_eq!(OptionsOverlay::total_categories(), 5);
    }

    #[test]
    fn test_overlay_has_correct_border_format() {
        let lines = OptionsOverlay::build_overlay(
            0,
            22.5,
            45.0,
            1.0,
            0.5,
            5.0,
            DiffusionKernel::Mean3x3,
            WindDirection::None,
            TerrainType::None,
            1.0,
            false,
            0,
            20.0,
            false,
            PaletteShiftSpeed::Off,
            false,
            false,
            80,
        );

        assert!(lines[0].starts_with('┌'), "First line should start with ┌");
        assert!(lines[0].ends_with('┐'), "First line should end with ┐");
        assert!(
            lines.last().unwrap().starts_with('└'),
            "Last line should start with └"
        );
        assert!(
            lines.last().unwrap().ends_with('┘'),
            "Last line should end with ┘"
        );
    }

    #[test]
    fn test_overlay_all_lines_consistent_width() {
        for category_idx in 0..5 {
            let lines = OptionsOverlay::build_overlay(
                category_idx,
                22.5,
                45.0,
                1.0,
                0.5,
                5.0,
                DiffusionKernel::Mean3x3,
                WindDirection::None,
                TerrainType::None,
                1.0,
                false,
                0,
                20.0,
                false,
                PaletteShiftSpeed::Medium,
                false,
                false,
                80,
            );

            for (line_num, line) in lines.iter().enumerate() {
                assert!(
                    line.starts_with('┌') || line.starts_with('│') || line.starts_with('└'),
                    "Category {}, line {}: All lines should start with border character",
                    category_idx,
                    line_num
                );
                assert!(
                    line.ends_with('┐') || line.ends_with('│') || line.ends_with('┘'),
                    "Category {}, line {}: All lines should end with border character",
                    category_idx,
                    line_num
                );
                assert!(
                    line.chars().count() >= 35 && line.chars().count() <= 50,
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
fn test_quick_help_displays_correctly() {
    use crate::simulation::config::InitMode;
    use crate::terminal::control::RuntimeState;

    let _state = RuntimeState::new(
        42,
        InitMode::Random,
        crate::simulation::config::Preset::Network,
        0,
        false,
        crate::terminal::control::MouseInteractionMode::Disabled,
        0.0,
    );

    let quick_help = [
        "┌─ tslime controls ───────────────────────┐",
        "│ p: Pause/Resume                         │",
        "│ r: Restart                              │",
        "│ +/-: Time scale                         │",
        "│ c: Cycle palette                        │",
        "│ h: Toggle help (Tab for options)        │",
        "│ q: Quit                                 │",
        "│                                        │",
        "│ SIMULATION (A,T,S,E,I)                  │",
        "│ ENVIRONMENT (K,W,Y,U)                   │",
        "│ VISUAL (B,V,N)                          │",
        "└─────────────────────────────────────────┘",
    ];

    for line in &quick_help {
        assert!(line.starts_with('│') || line.starts_with('┌') || line.starts_with('└'));
        assert!(line.ends_with('│') || line.ends_with('┐') || line.ends_with('┘'));
    }

    assert_eq!(quick_help.len(), 12);
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
    assert_eq!(total, 5);

    for idx in 0..total {
        let overlay = OptionsOverlay::build_overlay(
            idx,
            state.sensor_angle,
            state.turn_angle,
            state.step_size,
            state.decay_factor,
            state.deposit_amount,
            state.diffusion_kernel,
            state.wind_direction,
            state.terrain_type,
            state.terrain_strength,
            state.auto_normalize,
            state.motion_blur_frames,
            state.max_brightness,
            state.fast_mode_enabled,
            state.palette_shift_speed,
            state.invert_palette,
            state.reverse_palette,
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
        state.turn_angle,
        state.step_size,
        state.decay_factor,
        state.deposit_amount,
        state.diffusion_kernel,
        state.wind_direction,
        state.terrain_type,
        state.terrain_strength,
        state.auto_normalize,
        state.motion_blur_frames,
        state.max_brightness,
        state.fast_mode_enabled,
        state.palette_shift_speed,
        state.invert_palette,
        state.reverse_palette,
        80,
    );
    assert!(sim_overlay.iter().any(|line| line.contains("Sensor Angle")));
    assert!(sim_overlay.iter().any(|line| line.contains("Turn Angle")));
    assert!(sim_overlay.iter().any(|line| line.contains("Step Size")));

    let env_overlay = OptionsOverlay::build_overlay(
        1,
        state.sensor_angle,
        state.turn_angle,
        state.step_size,
        state.decay_factor,
        state.deposit_amount,
        state.diffusion_kernel,
        state.wind_direction,
        state.terrain_type,
        state.terrain_strength,
        state.auto_normalize,
        state.motion_blur_frames,
        state.max_brightness,
        state.fast_mode_enabled,
        state.palette_shift_speed,
        state.invert_palette,
        state.reverse_palette,
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

    let visual_overlay = OptionsOverlay::build_overlay(
        2,
        state.sensor_angle,
        state.turn_angle,
        state.step_size,
        state.decay_factor,
        state.deposit_amount,
        state.diffusion_kernel,
        state.wind_direction,
        state.terrain_type,
        state.terrain_strength,
        state.auto_normalize,
        state.motion_blur_frames,
        state.max_brightness,
        state.fast_mode_enabled,
        state.palette_shift_speed,
        state.invert_palette,
        state.reverse_palette,
        80,
    );

    assert!(
        visual_overlay
            .iter()
            .any(|line| line.contains("100.0") || line.contains("100")),
        "Should contain max brightness value. Got: {:?}",
        visual_overlay
    );

    assert!(
        visual_overlay
            .iter()
            .any(|line| line.contains("3") && line.contains("frames")),
        "Should contain motion blur frames value. Got: {:?}",
        visual_overlay
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
            state.turn_angle,
            state.step_size,
            state.decay_factor,
            state.deposit_amount,
            state.diffusion_kernel,
            state.wind_direction,
            state.terrain_type,
            state.terrain_strength,
            state.auto_normalize,
            state.motion_blur_frames,
            state.max_brightness,
            state.fast_mode_enabled,
            state.palette_shift_speed,
            state.invert_palette,
            state.reverse_palette,
            80,
        );

        assert!(
            overlay.iter().any(|line| line.starts_with("┌─")),
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
                .any(|line| line.ends_with("┘") || line.ends_with("│")),
            "Should have bottom corners"
        );

        for (i, line) in overlay.iter().enumerate() {
            assert!(
                line.len() <= 126,
                "Line {} should not exceed 126 chars, got {}: {}",
                i,
                line.len(),
                line
            );
        }
    }
}
