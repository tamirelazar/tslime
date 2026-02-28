use crate::render::palette::RgbColor;
use crate::render::panel::{Padding, PanelBuilder, TextAlignment};
use crate::render::theme::GRUVBOX_DARK;
use crate::simulation::config::DiffusionKernel;
use crate::simulation::config::TerrainType;
use crate::terminal::control::DefaultValues;
use crate::terminal::control::PaletteShiftSpeed;
use crate::terminal::control::WindDirection;

/// Renders a compact horizontal progress bar.
///
/// `filled` is clamped to `[0, total]`. Uses `█` for filled cells and `░` for empty.
///
/// Example: `mini_bar(0.25, 8)` → `"██░░░░░░"`
fn mini_bar(ratio: f32, total: usize) -> String {
    let filled = ((ratio.clamp(0.0, 1.0)) * total as f32).round() as usize;
    let filled = filled.min(total);
    format!("{}{}", "█".repeat(filled), "░".repeat(total - filled))
}

/// Overlay that displays current simulation controls and parameters.
pub struct ControlsOverlay;

impl ControlsOverlay {
    /// Total rendered width of the controls overlay window.
    pub const WIDTH: usize = 55;
    /// Content width (inner drawable area).
    const CONTENT_WIDTH: usize = 49; // 55 - 2(border) - 2*2(padding)
    const LABEL_WIDTH: usize = 18;
    const VALUE_WIDTH: usize = 9;
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

    /// Calculates centered position for the controls overlay.
    pub fn calculate_position(term_width: usize, term_height: usize) -> (usize, usize) {
        let x = (term_width.saturating_sub(Self::WIDTH)) / 2;
        let y = (term_height.saturating_sub(24)) / 2;
        (x, y)
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
        accent: RgbColor,
    ) -> crate::render::panel::RenderedOverlay {
        use TextAlignment::Left;

        // Visual tab strip showing all 6 categories; active one is marked with ●.
        // Short names kept to 3 chars so all 6 fit in the 49-char content area.
        // Indicators are CENTERED ABOVE their labels (stacked vertically).
        let tab_labels = ["SIM", "ENV", "APP", "PST", "PRF", "SYS"];
        let mut indicator_parts: Vec<String> = Vec::new();
        let mut label_parts: Vec<String> = Vec::new();
        for (i, lbl) in tab_labels.iter().enumerate() {
            let marker = if i == category_idx { '●' } else { '○' };
            let indicator = format!("{:^width$}", marker, width = lbl.len());
            indicator_parts.push(indicator);
            label_parts.push(lbl.to_string());
        }
        let indicator_line = indicator_parts.join("  ");
        let label_line = label_parts.join("  ");
        // Center both lines within CONTENT_WIDTH (49 chars)
        let indicator_line = format!("{:^width$}", indicator_line, width = Self::CONTENT_WIDTH);
        let label_line = format!("{:^width$}", label_line, width = Self::CONTENT_WIDTH);

        // Helper: returns "*" when a float param differs from its default, else " ".
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

        // Helper: builds a row with an 8-char mini progress bar.
        // Format: "{marker} {key:<3}  {label:<LABEL_WIDTH>}  {bar:8}  {value:>VALUE_WIDTH}"
        let param_row = |marker: &str, key: &str, label: &str, bar: String, value: String| {
            format!(
                "{} {:<3}  {:<label_w$}  {}  {:>value_w$}",
                marker,
                key,
                label,
                bar,
                value,
                label_w = Self::LABEL_WIDTH,
                value_w = Self::VALUE_WIDTH
            )
        };

        // Title no longer includes the "[N/6]" counter — the tab bar makes it redundant.
        let mut builder = PanelBuilder::new(Self::CONTENT_WIDTH, None)
            .with_padding(Padding::new(2, 0, 2, 2))
            .with_title("CONTROLS")
            .with_title_box()
            .add_single(indicator_line, Left)
            .add_single(label_line, Left)
            .add_empty()
            .add_separator();

        builder = match category_idx {
            // ── Category 0: Simulation Core ──────────────────────────────────
            0 => builder
                .add_empty()
                .add_single(
                    param_row(
                        mod_marker(sensor_angle, defaults.sensor_angle, 0.01),
                        "A/a",
                        "Sensor Angle",
                        mini_bar((sensor_angle - 5.0) / 85.0, 8),
                        format!("{:.1}°", sensor_angle),
                    ),
                    Left,
                )
                .add_single(
                    param_row(
                        mod_marker(sensor_distance, defaults.sensor_distance, 0.01),
                        "J/j",
                        "Sensor Dist",
                        mini_bar((sensor_distance - 1.0) / 49.0, 8),
                        format!("{:.1}px", sensor_distance),
                    ),
                    Left,
                )
                .add_single(
                    param_row(
                        mod_marker(turn_angle, defaults.turn_angle, 0.01),
                        "T/t",
                        "Turn Angle",
                        mini_bar((turn_angle - 5.0) / 85.0, 8),
                        format!("{:.1}°", turn_angle),
                    ),
                    Left,
                )
                .add_single(
                    param_row(
                        mod_marker(step_size, defaults.step_size, 0.01),
                        "S/s",
                        "Step Size",
                        mini_bar((step_size - 0.5) / 4.5, 8),
                        format!("{:.1}px", step_size),
                    ),
                    Left,
                )
                .add_single(
                    param_row(
                        mod_marker(decay_factor, defaults.decay_factor, 0.001),
                        "E/e",
                        "Decay Factor",
                        mini_bar((decay_factor - 0.5) / 0.49, 8),
                        format!("{:.3}", decay_factor),
                    ),
                    Left,
                )
                .add_single(
                    param_row(
                        mod_marker(deposit_amount, defaults.deposit_amount, 0.01),
                        "I/i",
                        "Deposit Amt",
                        mini_bar((deposit_amount - 1.0) / 19.0, 8),
                        format!("{:.1}×", deposit_amount),
                    ),
                    Left,
                )
                .add_single(
                    param_row(
                        " ",
                        "+/-",
                        "Time Scale",
                        mini_bar((time_scale - 0.5) / 3.5, 8),
                        format!("{:.1}×", time_scale),
                    ),
                    Left,
                )
                .add_empty(),
            // ── Category 1: Forces & Environment ─────────────────────────────
            1 => {
                let kernel_name = match diffusion_kernel {
                    DiffusionKernel::Mean3x3 => "Mean3x3",
                    DiffusionKernel::Gaussian => "Gaussian",
                };
                let mut b = builder.add_empty().add_single(
                    param_row(
                        mod_marker_enum(&diffusion_kernel, &defaults.diffusion_kernel),
                        "K",
                        "Diffusion",
                        "────────".to_string(),
                        kernel_name.to_string(),
                    ),
                    Left,
                );
                if matches!(diffusion_kernel, DiffusionKernel::Gaussian) {
                    b = b.add_single(
                        param_row(
                            mod_marker(diffusion_sigma, defaults.diffusion_sigma, 0.01),
                            ";/:",
                            "Diff Sigma",
                            mini_bar((diffusion_sigma - 0.5) / 1.5, 8),
                            format!("{:.2}", diffusion_sigma),
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
                        param_row(
                            mod_marker_enum(&wind_direction, &defaults.wind_direction),
                            "W",
                            "Wind",
                            "────────".to_string(),
                            wind_direction.name().to_string(),
                        ),
                        Left,
                    )
                    .add_single(
                        param_row(
                            mod_marker_enum(&terrain_type, &defaults.terrain_type),
                            "U",
                            "Terrain Type",
                            "────────".to_string(),
                            terrain_name.to_string(),
                        ),
                        Left,
                    )
                    .add_single(
                        param_row(
                            mod_marker(terrain_strength, defaults.terrain_strength, 0.01),
                            "Y/y",
                            "Terrain Str",
                            mini_bar((terrain_strength - 0.1) / 4.9, 8),
                            format!("{:.1}×", terrain_strength),
                        ),
                        Left,
                    )
                    .add_single(
                        param_row(
                            mod_marker(attractor_strength, defaults.attractor_strength, 0.01),
                            "L/l",
                            "Attractor",
                            mini_bar((attractor_strength - 0.1) / 9.9, 8),
                            format!("{:.1}×", attractor_strength),
                        ),
                        Left,
                    )
                    .add_single(
                        param_row(
                            " ",
                            ",",
                            "Mouse Mode",
                            "────────".to_string(),
                            mouse_mode.to_string(),
                        ),
                        Left,
                    );
                if mouse_mode != "Disabled" {
                    b = b.add_single(
                        param_row(
                            " ",
                            "─",
                            "Mouse Timeout",
                            "────────".to_string(),
                            format!("{:.1}s", mouse_timeout),
                        ),
                        Left,
                    );
                }
                b.add_empty()
            }
            // ── Category 2: Appearance ────────────────────────────────────────
            2 => {
                let shift_name = match palette_shift_speed {
                    PaletteShiftSpeed::Off => "Off",
                    PaletteShiftSpeed::Slow => "Slow",
                    PaletteShiftSpeed::Medium => "Medium",
                    PaletteShiftSpeed::Fast => "Fast",
                };
                let inv_bar = if invert_palette {
                    "▪───────"
                } else {
                    "────────"
                };
                let rev_bar = if reverse_palette {
                    "▪───────"
                } else {
                    "────────"
                };
                builder
                    .add_empty()
                    .add_single(
                        param_row(
                            " ",
                            "c/C",
                            "Palette",
                            "────────".to_string(),
                            palette_name.to_string(),
                        ),
                        Left,
                    )
                    .add_single(
                        param_row(
                            " ",
                            "O",
                            "Palette Shift",
                            "────────".to_string(),
                            shift_name.to_string(),
                        ),
                        Left,
                    )
                    .add_single(
                        param_row(
                            " ",
                            "X",
                            "Invert",
                            inv_bar.to_string(),
                            if invert_palette { "On" } else { "Off" }.to_string(),
                        ),
                        Left,
                    )
                    .add_single(
                        param_row(
                            " ",
                            "Z",
                            "Reverse",
                            rev_bar.to_string(),
                            if reverse_palette { "On" } else { "Off" }.to_string(),
                        ),
                        Left,
                    )
                    .add_empty()
            }
            // ── Category 3: Post-Processing ───────────────────────────────────
            3 => {
                let norm_bar = if auto_normalize != defaults.auto_normalize {
                    if auto_normalize {
                        "▪───────"
                    } else {
                        "────────"
                    }
                } else if auto_normalize {
                    "▪───────"
                } else {
                    "────────"
                };
                builder
                    .add_empty()
                    .add_single(
                        param_row(
                            " ",
                            "d/D",
                            "Dither Mode",
                            "────────".to_string(),
                            dither_mode_name.to_string(),
                        ),
                        Left,
                    )
                    .add_single(
                        param_row(
                            if auto_normalize != defaults.auto_normalize {
                                "*"
                            } else {
                                " "
                            },
                            "B",
                            "Auto Normalize",
                            norm_bar.to_string(),
                            if auto_normalize { "On" } else { "Off" }.to_string(),
                        ),
                        Left,
                    )
                    .add_single(
                        param_row(
                            mod_marker_int(motion_blur_frames, defaults.motion_blur_frames),
                            "V",
                            "Motion Blur",
                            mini_bar(motion_blur_frames as f32 / 7.0, 8),
                            format!("{} fr", motion_blur_frames),
                        ),
                        Left,
                    )
                    .add_single(
                        param_row(
                            mod_marker(max_brightness, defaults.max_brightness, 0.01),
                            "N/n",
                            "Max Bright",
                            mini_bar((max_brightness - 1.0) / 99.0, 8),
                            format!("{:.1}×", max_brightness),
                        ),
                        Left,
                    )
                    .add_empty()
            }
            // ── Category 4: Performance ───────────────────────────────────────
            4 => {
                let fast_bar = if fast_mode_enabled {
                    "▪───────"
                } else {
                    "────────"
                };
                builder
                    .add_empty()
                    .add_single(
                        param_row(
                            " ",
                            "F",
                            "Fast Mode",
                            fast_bar.to_string(),
                            if fast_mode_enabled { "On" } else { "Off" }.to_string(),
                        ),
                        Left,
                    )
                    .add_single(
                        param_row(
                            " ",
                            "─",
                            "Population",
                            "────────".to_string(),
                            format!("{}k", population / 1000),
                        ),
                        Left,
                    )
                    .add_empty()
            }
            // ── Category 5: System ────────────────────────────────────────────
            5 => builder
                .add_empty()
                .add_single(
                    param_row(
                        " ",
                        "G",
                        "Save Frame",
                        "────────".to_string(),
                        "(PNG)".to_string(),
                    ),
                    Left,
                )
                .add_single(
                    param_row(
                        " ",
                        "0",
                        "Reset",
                        "────────".to_string(),
                        "Defaults".to_string(),
                    ),
                    Left,
                )
                .add_single(
                    param_row(
                        " ",
                        "8",
                        "Randomize",
                        "────────".to_string(),
                        "Params".to_string(),
                    ),
                    Left,
                )
                .add_empty(),
            _ => builder,
        };

        let mut overlay = builder
            .add_separator()
            .add_single(
                format!(
                    "{:<width$}",
                    "  * modified  ─ CLI-only  Tab: next  Esc: close",
                    width = Self::CONTENT_WIDTH
                ),
                Left,
            )
            .build_overlay();
        overlay.rich_lines = Some(generate_controls_rich_lines(&overlay.lines, accent));
        overlay
    }
}

/// Applies per-cell color overrides to the controls overlay lines.
///
/// Identifies parameter rows (key binding + mini bar) and applies:
/// - Accent colour to the key chars (cols 5–7).
/// - `accent_modified` colour to the `*` modification marker at col 3.
/// - Accent colour to filled bar chars (`█`/`▪`) at cols 25–32.
/// - Muted colour to empty bar chars (`░`) at the same positions.
///
/// Also colors tab indicators (●/○) in the tab strip with accent color.
fn generate_controls_rich_lines(
    lines: &[String],
    accent: RgbColor,
) -> Vec<Vec<(char, Option<RgbColor>, Option<RgbColor>)>> {
    let modified_color = GRUVBOX_DARK.accent_modified;
    let muted_color = GRUVBOX_DARK.muted;

    lines
        .iter()
        .map(|line| {
            let chars: Vec<char> = line.chars().collect();
            let n = chars.len();

            // Check if this is the indicator line (contains ● or ○)
            let is_indicator_line = chars.iter().any(|&c| c == '●' || c == '○');

            if is_indicator_line {
                return chars
                    .iter()
                    .map(|&c| {
                        let fg = match c {
                            '●' | '○' => Some(accent),
                            _ => None,
                        };
                        (c, fg, None)
                    })
                    .collect();
            }

            // A param row: marker at col 3 (' ' or '*'), space at col 4,
            // col 5 is not another '*' (distinguishes from the "  * modified…" footer),
            // and the region cols 3..47 is not all-spaces (padding rows).
            let is_param_row = n == ControlsOverlay::WIDTH
                && matches!(chars.get(3), Some(' ') | Some('*'))
                && matches!(chars.get(4), Some(' '))
                && !matches!(chars.get(5), Some('*'))
                && chars
                    .get(3..47.min(n))
                    .map_or(false, |s| s.iter().any(|&c| c != ' '));

            if !is_param_row {
                return chars.iter().map(|&c| (c, None, None)).collect();
            }

            chars
                .iter()
                .enumerate()
                .map(|(i, &c)| {
                    let fg = match i {
                        3 if c == '*' => Some(modified_color),
                        5..=7 => Some(accent),
                        27..=38 => match c {
                            '█' | '▪' => Some(accent),
                            '░' => Some(muted_color),
                            _ => None,
                        },
                        _ => None,
                    };
                    (c, fg, None)
                })
                .collect()
        })
        .collect()
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
            RgbColor {
                r: 57,
                g: 211,
                b: 83,
            },
        );

        assert!(
            lines.lines[0].starts_with('█'),
            "First line should start with solid block █"
        );
        assert!(
            lines.lines[0].ends_with('█'),
            "First line should end with solid block █"
        );
        assert!(
            lines.lines.last().unwrap().starts_with('█'),
            "Last line should start with solid block █"
        );
        assert!(
            lines.lines.last().unwrap().ends_with('█'),
            "Last line should end with solid block █"
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
                RgbColor {
                    r: 57,
                    g: 211,
                    b: 83,
                },
            );

            // All lines should be exactly WIDTH chars wide
            for (line_num, line) in lines.lines.iter().enumerate() {
                assert!(
                    line.starts_with('█') || line.starts_with('▀') || line.starts_with('▄'),
                    "Category {}, line {}: All lines should start with solid block char",
                    category_idx,
                    line_num
                );
                assert!(
                    line.ends_with('█') || line.ends_with('▀') || line.ends_with('▄'),
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
            RgbColor {
                r: 57,
                g: 211,
                b: 83,
            },
        );

        // The title box shows "CONTROLS" (no longer includes [N/6]; the tab bar does that).
        let title_box = lines
            .title_box
            .as_ref()
            .expect("Expected title_box to be present");
        assert!(
            title_box.lines.iter().any(|l| l.contains("CONTROLS")),
            "Title box should contain CONTROLS"
        );
        // The tab bar (in the main body) should mark the active category with ●
        assert!(
            lines.lines.iter().any(|l| l.contains('●')),
            "Overlay body should contain the active-tab marker ●"
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
            RgbColor {
                r: 57,
                g: 211,
                b: 83,
            },
        );

        assert!(
            !overlay.lines.is_empty(),
            "Category {} should not be empty",
            idx
        );

        // The active tab marker ● is present in every category's overlay.
        assert!(
            overlay.lines.iter().any(|line| line.contains('●')),
            "Category {} should contain the active tab marker ●",
            idx
        );

        assert!(
            !overlay.lines.is_empty(),
            "Category {} should have lines",
            idx
        );
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
        RgbColor {
            r: 57,
            g: 211,
            b: 83,
        },
    );
    assert!(sim_overlay
        .lines
        .iter()
        .any(|line| line.contains("Sensor Angle")));
    assert!(sim_overlay
        .lines
        .iter()
        .any(|line| line.contains("Sensor Dist")));
    assert!(sim_overlay
        .lines
        .iter()
        .any(|line| line.contains("Turn Angle")));
    assert!(sim_overlay
        .lines
        .iter()
        .any(|line| line.contains("Step Size")));

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
        RgbColor {
            r: 57,
            g: 211,
            b: 83,
        },
    );
    assert!(env_overlay
        .lines
        .iter()
        .any(|line| line.contains("Diffusion")));
    assert!(env_overlay.lines.iter().any(|line| line.contains("Wind")));
    assert!(env_overlay
        .lines
        .iter()
        .any(|line| line.contains("Terrain")));
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
        RgbColor {
            r: 57,
            g: 211,
            b: 83,
        },
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
            .any(|line| line.contains("Motion Blur") && line.contains("fr")),
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
            RgbColor {
                r: 57,
                g: 211,
                b: 83,
            },
        );

        // Solid-block borders
        assert!(
            overlay.lines.iter().any(|line| line.starts_with('█')),
            "Should have solid block border char █"
        );
        assert!(
            overlay.lines.iter().any(|line| line.contains('█')),
            "Should have solid block vertical border"
        );
        assert!(
            overlay
                .lines
                .iter()
                .any(|line| line.ends_with('█') || line.ends_with('▄')),
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
