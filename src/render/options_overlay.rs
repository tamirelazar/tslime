use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};

use crate::render::palette::RgbColor;
use crate::render::panel::{Padding, PanelBuilder, RichCell, TextAlignment};
use crate::render::ratatui_adapter::render_styled_rows;
use crate::render::theme::PanelStyle;
use crate::simulation::config::DiffusionKernel;
use crate::simulation::config::TerrainType;
use crate::terminal::control::DefaultValues;
use crate::terminal::control::PaletteShiftSpeed;
use crate::terminal::control::WindDirection;

/// Renders a compact horizontal progress bar of `total` cells.
///
/// `ratio` is clamped to `[0, 1]`. Uses `█` for filled cells and `░` for empty.
///
/// Example: `mini_bar(0.25, 8)` → `"██░░░░░░"`
fn mini_bar(ratio: f32, total: usize) -> String {
    let filled = ((ratio.clamp(0.0, 1.0)) * total as f32).round() as usize;
    let filled = filled.min(total);
    format!("{}{}", "█".repeat(filled), "░".repeat(total - filled))
}

/// Foreground colours used when styling control rows, resolved once per build.
struct LineColors {
    accent: Color,
    muted: Color,
    modified: Color,
    cli_only: Color,
}

/// A single content row of the controls overlay, before `PanelBuilder` chrome is applied.
enum Row {
    /// Blank content line.
    Empty,
    /// Horizontal separator (drawn by `PanelBuilder`, spans the padding).
    Sep,
    /// A styled line whose spans carry their own colours (blitted into the content region).
    Styled(Line<'static>),
}

/// Converts our [`RgbColor`] into a ratatui [`Color`].
fn rt(c: RgbColor) -> Color {
    Color::Rgb(c.r, c.g, c.b)
}

/// Builds a parameter row `{marker} {key:<3}  {label:<18}  {bar}  {value:>9}` as a styled
/// [`Line`]. The key is accent-coloured (or `cli_only` red), the modification marker `✱`
/// is `modified`-coloured, and bar cells are accent (`█`/`▪`) or muted (`░`). Span styling
/// replaces the old column-index colouriser, so the colours track the text exactly.
fn param_line(
    c: &LineColors,
    marker: &str,
    key: &str,
    cli_only: bool,
    label: &str,
    bar: &str,
    value: &str,
) -> Row {
    let key_color = if cli_only { c.cli_only } else { c.accent };
    let mut spans: Vec<Span<'static>> = Vec::new();
    let marker_style = if marker == "✱" {
        Style::default().fg(c.modified)
    } else {
        Style::default()
    };
    spans.push(Span::styled(marker.to_string(), marker_style));
    spans.push(Span::raw(" "));
    spans.push(Span::styled(
        format!("{:<3}", key),
        Style::default().fg(key_color),
    ));
    spans.push(Span::raw("  "));
    spans.push(Span::raw(format!(
        "{:<width$}",
        label,
        width = ControlsOverlay::LABEL_WIDTH
    )));
    spans.push(Span::raw("  "));
    for ch in bar.chars() {
        let style = match ch {
            '█' | '▪' => Style::default().fg(c.accent),
            '░' => Style::default().fg(c.muted),
            _ => Style::default(),
        };
        spans.push(Span::styled(ch.to_string(), style));
    }
    spans.push(Span::raw("  "));
    spans.push(Span::raw(format!(
        "{:>width$}",
        value,
        width = ControlsOverlay::VALUE_WIDTH
    )));
    Row::Styled(Line::from(spans))
}

/// A muted dev-only row (e.g. "Dither (dev)") — every non-space glyph greyed out.
fn dev_line(c: &LineColors, key: &str, label: &str, bar: &str, value: &str) -> Row {
    let text = format!(
        "{} {:<3}  {:<lw$}  {}  {:>vw$}",
        " ",
        key,
        label,
        bar,
        value,
        lw = ControlsOverlay::LABEL_WIDTH,
        vw = ControlsOverlay::VALUE_WIDTH
    );
    let spans = text
        .chars()
        .map(|ch| {
            if ch == ' ' {
                Span::raw(" ".to_string())
            } else {
                Span::styled(ch.to_string(), Style::default().fg(c.muted))
            }
        })
        .collect::<Vec<_>>();
    Row::Styled(Line::from(spans))
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
        rotation_angle: f32,
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
        charset_name: &str,
        color_aa_label: &str,
        palette_shift_speed: PaletteShiftSpeed,
        invert_palette: bool,
        reverse_palette: bool,
        dither_mode_name: &str,
        _term_width: usize,
        defaults: DefaultValues,
        population: usize,
        accent: RgbColor,
        theme_name: &str,
        panel_style: &PanelStyle,
        shift_held: bool,
        trail_age_enabled: bool,
        trail_age_mode: crate::config_defaults::TrailAgeMode,
        trail_age_reverse: bool,
        trail_delta_enabled: bool,
        gradient_magnitude_enabled: bool,
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

        // Helper: returns "✱" when a float param differs from its default, else " ".
        let mod_marker = |current: f32, default: f32, eps: f32| {
            if (current - default).abs() > eps {
                "✱"
            } else {
                " "
            }
        };
        let mod_marker_int =
            |current: usize, default: usize| if current != default { "✱" } else { " " };
        let mod_marker_enum = |current: &dyn std::fmt::Debug, default: &dyn std::fmt::Debug| {
            if format!("{:?}", current) != format!("{:?}", default) {
                "✱"
            } else {
                " "
            }
        };

        let lc = LineColors {
            accent: rt(accent),
            muted: rt(panel_style.muted),
            modified: rt(panel_style.accent_modified),
            cli_only: rt(RgbColor {
                r: 204,
                g: 102,
                b: 102,
            }),
        };

        // Per-category parameter rows, built as styled lines (span colours track the text).
        let category_rows: Vec<Row> = match category_idx {
            // ── Category 0: Simulation Core ──────────────────────────────────
            0 => vec![
                Row::Empty,
                param_line(
                    &lc,
                    mod_marker(sensor_angle, defaults.sensor_angle, 0.01),
                    "A/a",
                    false,
                    "Sensor Angle",
                    &mini_bar((sensor_angle - 5.0) / 85.0, 8),
                    &format!("{:.1}°", sensor_angle),
                ),
                param_line(
                    &lc,
                    mod_marker(sensor_distance, defaults.sensor_distance, 0.01),
                    "J/j",
                    false,
                    "Sensor Dist",
                    &mini_bar((sensor_distance - 1.0) / 49.0, 8),
                    &format!("{:.1}px", sensor_distance),
                ),
                param_line(
                    &lc,
                    mod_marker(rotation_angle, defaults.rotation_angle, 0.01),
                    "T/t",
                    false,
                    "Turn Angle",
                    &mini_bar((rotation_angle - 5.0) / 85.0, 8),
                    &format!("{:.1}°", rotation_angle),
                ),
                param_line(
                    &lc,
                    mod_marker(step_size, defaults.step_size, 0.01),
                    "S/s",
                    false,
                    "Step Size",
                    &mini_bar((step_size - 0.5) / 4.5, 8),
                    &format!("{:.1}px", step_size),
                ),
                param_line(
                    &lc,
                    mod_marker(decay_factor, defaults.decay_factor, 0.001),
                    "E/e",
                    false,
                    "Decay Factor",
                    &mini_bar((decay_factor - 0.5) / 0.49, 8),
                    &format!("{:.3}", decay_factor),
                ),
                param_line(
                    &lc,
                    mod_marker(deposit_amount, defaults.deposit_amount, 0.01),
                    "I/i",
                    false,
                    "Deposit Amt",
                    &mini_bar((deposit_amount - 1.0) / 19.0, 8),
                    &format!("{:.1}×", deposit_amount),
                ),
                param_line(
                    &lc,
                    " ",
                    "+/-",
                    false,
                    "Time Scale",
                    &mini_bar((time_scale - 0.5) / 3.5, 8),
                    &format!("{:.1}×", time_scale),
                ),
                Row::Empty,
            ],
            // ── Category 1: Forces & Environment ─────────────────────────────
            1 => {
                let kernel_name = match diffusion_kernel {
                    DiffusionKernel::Mean3x3 => "Mean3x3",
                    DiffusionKernel::Gaussian => "Gaussian",
                };
                let terrain_name = match terrain_type {
                    TerrainType::None => "None",
                    TerrainType::Smooth => "Smooth",
                    TerrainType::Turbulent => "Turbulent",
                    TerrainType::Mixed => "Mixed",
                };
                let mut v = vec![
                    Row::Empty,
                    param_line(
                        &lc,
                        mod_marker_enum(&diffusion_kernel, &defaults.diffusion_kernel),
                        "K",
                        false,
                        "Diffusion",
                        "────────",
                        kernel_name,
                    ),
                ];
                if matches!(diffusion_kernel, DiffusionKernel::Gaussian) {
                    v.push(param_line(
                        &lc,
                        mod_marker(diffusion_sigma, defaults.diffusion_sigma, 0.01),
                        ";/:",
                        false,
                        "Diff Sigma",
                        &mini_bar((diffusion_sigma - 0.5) / 3.5, 8),
                        &format!("{:.2}", diffusion_sigma),
                    ));
                }
                v.push(param_line(
                    &lc,
                    mod_marker_enum(&wind_direction, &defaults.wind_direction),
                    "W",
                    false,
                    "Wind",
                    "────────",
                    wind_direction.name(),
                ));
                v.push(param_line(
                    &lc,
                    mod_marker_enum(&terrain_type, &defaults.terrain_type),
                    "U",
                    false,
                    "Terrain Type",
                    "────────",
                    terrain_name,
                ));
                v.push(param_line(
                    &lc,
                    mod_marker(terrain_strength, defaults.terrain_strength, 0.01),
                    "Y/y",
                    false,
                    "Terrain Str",
                    &mini_bar((terrain_strength - 0.1) / 4.9, 8),
                    &format!("{:.1}×", terrain_strength),
                ));
                v.push(param_line(
                    &lc,
                    mod_marker(attractor_strength, defaults.attractor_strength, 0.01),
                    "L/l",
                    false,
                    "Attractor",
                    &mini_bar((attractor_strength - 0.1) / 9.9, 8),
                    &format!("{:.1}×", attractor_strength),
                ));
                v.push(param_line(
                    &lc,
                    " ",
                    ",",
                    false,
                    "Mouse Mode",
                    "────────",
                    mouse_mode,
                ));
                if mouse_mode != "Disabled" {
                    v.push(param_line(
                        &lc,
                        " ",
                        "─",
                        false,
                        "Mouse Timeout",
                        "────────",
                        &format!("{:.1}s", mouse_timeout),
                    ));
                }
                v.push(Row::Empty);
                v
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
                vec![
                    Row::Empty,
                    param_line(&lc, " ", "9/*", false, "Theme", "────────", theme_name),
                    param_line(&lc, " ", "c/C", false, "Palette", "────────", palette_name),
                    param_line(&lc, " ", "`/~", false, "Charset", "────────", charset_name),
                    param_line(
                        &lc,
                        " ",
                        "\"",
                        false,
                        "Color AA",
                        "────────",
                        color_aa_label,
                    ),
                    param_line(
                        &lc,
                        " ",
                        "O",
                        false,
                        "Palette Shift",
                        "────────",
                        shift_name,
                    ),
                    param_line(
                        &lc,
                        " ",
                        "X",
                        false,
                        "Invert",
                        inv_bar,
                        if invert_palette { "On" } else { "Off" },
                    ),
                    param_line(
                        &lc,
                        " ",
                        "Z",
                        false,
                        "Reverse",
                        rev_bar,
                        if reverse_palette { "On" } else { "Off" },
                    ),
                    Row::Empty,
                ]
            }
            // ── Category 3: Post-Processing ───────────────────────────────────
            3 => {
                // Normalize toggle bar: filled when auto-normalize is on.
                let norm_bar = if auto_normalize {
                    "▪───────"
                } else {
                    "────────"
                };
                let norm_marker = if auto_normalize != defaults.auto_normalize {
                    "*"
                } else {
                    " "
                };
                let brightness_bar = if auto_normalize {
                    "────────".to_string()
                } else {
                    // Brightness gain; default (1.0×) sits mid-bar so it can read as
                    // brighter or dimmer either way.
                    let gain = crate::config_defaults::trail::brightness_gain(max_brightness);
                    mini_bar((gain / 2.0).clamp(0.0, 1.0), 8)
                };
                let brightness_value = if auto_normalize {
                    "auto".to_string()
                } else {
                    let gain = crate::config_defaults::trail::brightness_gain(max_brightness);
                    format!("{gain:.1}×")
                };
                let trail_age_value = if trail_age_enabled {
                    if trail_age_reverse {
                        format!("On ({} rev)", trail_age_mode.name())
                    } else {
                        format!("On ({})", trail_age_mode.name())
                    }
                } else {
                    "Off".to_string()
                };
                vec![
                    Row::Empty,
                    dev_line(&lc, "d/D", "Dither (dev)", "────────", dither_mode_name),
                    param_line(
                        &lc,
                        norm_marker,
                        "B",
                        false,
                        "Auto Normalize",
                        norm_bar,
                        if auto_normalize { "On" } else { "Off" },
                    ),
                    param_line(
                        &lc,
                        mod_marker_int(motion_blur_frames, defaults.motion_blur_frames),
                        "V",
                        false,
                        "Motion Blur",
                        &mini_bar(motion_blur_frames as f32 / 7.0, 8),
                        &format!("{} fr", motion_blur_frames),
                    ),
                    param_line(
                        &lc,
                        mod_marker(max_brightness, defaults.max_brightness, 0.01),
                        "N/n",
                        false,
                        "Brightness",
                        &brightness_bar,
                        &brightness_value,
                    ),
                    param_line(
                        &lc,
                        " ",
                        "'",
                        false,
                        "Trail Age",
                        if trail_age_enabled {
                            "▪───────"
                        } else {
                            "────────"
                        },
                        &trail_age_value,
                    ),
                    param_line(
                        &lc,
                        " ",
                        ".",
                        false,
                        "Trail Delta",
                        if trail_delta_enabled {
                            "▪───────"
                        } else {
                            "────────"
                        },
                        if trail_delta_enabled { "On" } else { "Off" },
                    ),
                    param_line(
                        &lc,
                        " ",
                        ">",
                        false,
                        "Edge Glow",
                        if gradient_magnitude_enabled {
                            "▪───────"
                        } else {
                            "────────"
                        },
                        if gradient_magnitude_enabled {
                            "On"
                        } else {
                            "Off"
                        },
                    ),
                    Row::Empty,
                ]
            }
            // ── Category 4: Performance ───────────────────────────────────────
            4 => {
                let fast_bar = if fast_mode_enabled {
                    "▪───────"
                } else {
                    "────────"
                };
                vec![
                    Row::Empty,
                    param_line(
                        &lc,
                        " ",
                        "F",
                        false,
                        "Fast Mode",
                        fast_bar,
                        if fast_mode_enabled { "On" } else { "Off" },
                    ),
                    // Population is a CLI-only parameter (key shown in muted red).
                    param_line(
                        &lc,
                        " ",
                        "─",
                        true,
                        "Population",
                        "────────",
                        &format!("{}k", population / 1000),
                    ),
                    Row::Empty,
                ]
            }
            // ── Category 5: System ────────────────────────────────────────────
            5 => vec![
                Row::Empty,
                param_line(&lc, " ", "G", false, "Save Frame", "────────", "(PNG)"),
                param_line(&lc, " ", "0", false, "Reset", "────────", "Defaults"),
                param_line(&lc, " ", "8", false, "Randomize", "────────", "Params"),
                Row::Empty,
            ],
            _ => vec![],
        };

        // ── Assemble all content rows in visual order ──────────────────────────
        let _ = shift_held;
        let accent_c = lc.accent;

        // Tab indicator markers (●/○) accent-coloured.
        let indicator_row = Row::Styled(Line::from(
            indicator_line
                .chars()
                .map(|ch| match ch {
                    '●' | '○' => Span::styled(ch.to_string(), Style::default().fg(accent_c)),
                    _ => Span::raw(ch.to_string()),
                })
                .collect::<Vec<_>>(),
        ));

        // Active tab label accent-coloured.
        let active_label = tab_labels[category_idx.min(tab_labels.len() - 1)];
        let label_chars: Vec<char> = label_line.chars().collect();
        let active: Vec<char> = active_label.chars().collect();
        let active_start = label_chars
            .windows(active.len())
            .position(|w| w == active.as_slice());
        let label_row = Row::Styled(Line::from(
            label_chars
                .iter()
                .enumerate()
                .map(|(i, &ch)| {
                    let hot = active_start.is_some_and(|s| i >= s && i < s + active.len());
                    if hot {
                        Span::styled(ch.to_string(), Style::default().fg(accent_c))
                    } else {
                        Span::raw(ch.to_string())
                    }
                })
                .collect::<Vec<_>>(),
        ));

        // Footer: "✱ modified" in the modified colour, "─ CLI-only" in the CLI-only red.
        let footer_main = format!(
            "{:^width$}",
            "  ✱ modified  ─ CLI-only  ",
            width = Self::CONTENT_WIDTH
        );
        let footer_row = Row::Styled(Line::from(
            footer_main
                .chars()
                .map(|ch| match ch {
                    '✱' => Span::styled("✱".to_string(), Style::default().fg(lc.modified)),
                    '─' => Span::styled("─".to_string(), Style::default().fg(lc.cli_only)),
                    _ => Span::raw(ch.to_string()),
                })
                .collect::<Vec<_>>(),
        ));

        // TAB-cycle hint: the `<`/`>` arrows accent-coloured.
        let tab_hint = format!("{:^width$}", " < TAB > ", width = Self::CONTENT_WIDTH);
        let tab_hint_row = Row::Styled(Line::from(
            tab_hint
                .chars()
                .map(|ch| {
                    if ch == '<' || ch == '>' {
                        Span::styled(ch.to_string(), Style::default().fg(accent_c))
                    } else {
                        Span::raw(ch.to_string())
                    }
                })
                .collect::<Vec<_>>(),
        ));
        let esc_hint = format!("{:^width$}", "ESC", width = Self::CONTENT_WIDTH);
        let esc_row = Row::Styled(Line::from(vec![Span::raw(esc_hint)]));

        let mut rows: Vec<Row> = vec![indicator_row, label_row, Row::Empty, Row::Sep];
        rows.extend(category_rows);
        rows.push(Row::Sep);
        rows.push(footer_row);
        rows.push(Row::Empty);
        rows.push(tab_hint_row);
        rows.push(esc_row);

        // Render the styled content once to obtain per-cell colours, then blit them
        // into the `PanelBuilder` chrome (which keeps the border/title-box/separators
        // pixel-identical to every other overlay).
        let styled_lines: Vec<Line<'static>> = rows
            .iter()
            .filter_map(|r| match r {
                Row::Styled(l) => Some(l.clone()),
                _ => None,
            })
            .collect();
        let rendered = render_styled_rows(&styled_lines, Self::CONTENT_WIDTH);

        let mut builder = PanelBuilder::new(Self::CONTENT_WIDTH, None)
            .with_padding(Padding::new(2, 0, 2, 2))
            .with_title("CONTROLS")
            .with_title_box();
        let mut order: Vec<Option<usize>> = Vec::with_capacity(rows.len());
        let mut styled_idx = 0usize;
        for row in &rows {
            match row {
                Row::Empty => {
                    builder = builder.add_empty();
                    order.push(None);
                }
                Row::Sep => {
                    builder = builder.add_separator();
                    order.push(None);
                }
                Row::Styled(_) => {
                    builder = builder.add_single(rendered[styled_idx].0.clone(), Left);
                    order.push(Some(styled_idx));
                    styled_idx += 1;
                }
            }
        }
        let mut overlay = builder.build_overlay();

        // Blit span colours into the content region. Offset = 1 border + 2 left padding;
        // the first content row sits below the top border + 2 top-padding rows.
        const CONTENT_OFFSET: usize = 3;
        const PREFIX_LINES: usize = 3;
        let rich: Vec<Vec<RichCell>> = overlay
            .lines
            .iter()
            .enumerate()
            .map(|(line_idx, line)| {
                let mut cells: Vec<RichCell> = line.chars().map(|ch| (ch, None, None)).collect();
                if line_idx >= PREFIX_LINES {
                    if let Some(Some(idx)) = order.get(line_idx - PREFIX_LINES) {
                        for (col, &fg) in rendered[*idx].1.iter().enumerate() {
                            if let Some(fg) = fg {
                                if let Some(cell) = cells.get_mut(CONTENT_OFFSET + col) {
                                    cell.1 = Some(fg);
                                }
                            }
                        }
                    }
                }
                cells
            })
            .collect();
        overlay.rich_lines = Some(rich);
        overlay
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::render::theme::GRUVBOX_DARK;
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
            "HalfBlock",
            "Off",
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
            "GruvboxDark",
            &GRUVBOX_DARK,
            false,
            false,
            crate::config_defaults::TrailAgeMode::Bidirectional,
            false,
            false,
            false,
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
                "HalfBlock",
                "Off",
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
                "GruvboxDark",
                &GRUVBOX_DARK,
                false,
                false,
                crate::config_defaults::TrailAgeMode::Bidirectional,
                false,
                false,
                false,
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
            "HalfBlock",
            "Off",
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
            "GruvboxDark",
            &GRUVBOX_DARK,
            false,
            false,
            crate::config_defaults::TrailAgeMode::Bidirectional,
            false,
            false,
            false,
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
    use crate::cli::PauseStyle;
    use crate::render::theme::GRUVBOX_DARK;
    use crate::simulation::config::{InitMode, Preset, SimConfig};
    use crate::terminal::control::RuntimeState;

    let state = RuntimeState::new(
        42,
        InitMode::Random,
        Preset::Organic,
        crate::terminal::control::MouseInteractionMode::Disabled,
        0.0,
        &SimConfig::default(),
        PauseStyle::Vignette,
        false,
        false,
    );

    let total = ControlsOverlay::total_categories();
    assert_eq!(total, 6);

    for idx in 0..total {
        let overlay = ControlsOverlay::build_overlay(
            idx,
            state.sensor_angle,
            state.sensor_distance,
            state.rotation_angle,
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
            "HalfBlock",
            "Off",
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
            "GruvboxDark",
            &GRUVBOX_DARK,
            false,
            false,
            crate::config_defaults::TrailAgeMode::Bidirectional,
            false,
            false,
            false,
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
        state.rotation_angle,
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
        "HalfBlock",
        "Off",
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
        "GruvboxDark",
        &GRUVBOX_DARK,
        false,
        false,
        crate::config_defaults::TrailAgeMode::Bidirectional,
        false,
        false,
        false,
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
        state.rotation_angle,
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
        "HalfBlock",
        "Off",
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
        "GruvboxDark",
        &GRUVBOX_DARK,
        false,
        false,
        crate::config_defaults::TrailAgeMode::Bidirectional,
        false,
        false,
        false,
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
    use crate::cli::PauseStyle;
    use crate::render::theme::GRUVBOX_DARK;
    use crate::simulation::config::{InitMode, Preset, SimConfig};
    use crate::terminal::control::RuntimeState;

    let mut state = RuntimeState::new(
        42,
        InitMode::Random,
        Preset::Organic,
        crate::terminal::control::MouseInteractionMode::Disabled,
        0.0,
        &SimConfig::default(),
        PauseStyle::Vignette,
        false,
        false,
    );

    state.max_brightness = 100.0;
    state.motion_blur_frames = 3;

    let postprocessing_overlay = ControlsOverlay::build_overlay(
        3,
        state.sensor_angle,
        state.sensor_distance,
        state.rotation_angle,
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
        "HalfBlock",
        "Off",
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
        "GruvboxDark",
        &GRUVBOX_DARK,
        false,
        false,
        crate::config_defaults::TrailAgeMode::Bidirectional,
        false,
        false,
        false,
    );

    // Brightness is shown as a gain relative to the default white-point, so the
    // default value (100.0) renders as the neutral 1.0× rather than the raw value.
    assert!(
        postprocessing_overlay
            .lines
            .iter()
            .any(|line| line.contains("Brightness") && line.contains("1.0×")),
        "Should contain brightness gain value. Got: {:?}",
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
    use crate::cli::PauseStyle;
    use crate::render::theme::GRUVBOX_DARK;
    use crate::simulation::config::{InitMode, Preset, SimConfig};
    use crate::terminal::control::RuntimeState;

    let state = RuntimeState::new(
        42,
        InitMode::Random,
        Preset::Organic,
        crate::terminal::control::MouseInteractionMode::Disabled,
        0.0,
        &SimConfig::default(),
        PauseStyle::Vignette,
        false,
        false,
    );

    for idx in 0..ControlsOverlay::total_categories() {
        let overlay = ControlsOverlay::build_overlay(
            idx,
            state.sensor_angle,
            state.sensor_distance,
            state.rotation_angle,
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
            "HalfBlock",
            "Off",
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
            "GruvboxDark",
            &GRUVBOX_DARK,
            false,
            false,
            crate::config_defaults::TrailAgeMode::Bidirectional,
            false,
            false,
            false,
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
