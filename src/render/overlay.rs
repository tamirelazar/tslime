use crate::cli::Palette;
use crate::render::dither::DitherMode;
use crate::render::palette::RgbColor;
use crate::render::theme::PanelStyle;
use crate::simulation::config::Preset;
use crate::terminal::control::{palette_name, preset_name};

pub use crate::render::panel::{
    BorderConfig, ColumnLayout, Padding, PanelBuilder, PanelRow, PanelSize, RenderedOverlay,
    RenderedTitleBox, RichCell, TextAlignment, TitleAlignment,
};

// --- OverlayConfig ---

/// Configuration for different overlay types.
///
/// Used primarily for color/style configuration by the renderer; panel
/// construction now uses `PanelBuilder` directly.
#[derive(Clone, Debug)]
pub struct OverlayConfig {
    /// Width of the overlay in characters (total, including border and padding)
    pub width: usize,
    /// Vertical padding (empty lines at top/bottom)
    pub height_padding: usize,
    /// Horizontal padding (spaces on left/right inside border)
    pub width_padding: usize,
    /// Text color (ANSI 256 index)
    pub text_color_256: u8,
    /// Background color (ANSI 256 index)
    pub bg_color_256: u8,
    /// Whether this overlay has a border
    pub has_border: bool,
}

impl OverlayConfig {
    /// Help overlay configuration
    pub const HELP: OverlayConfig = OverlayConfig {
        width: 62,
        height_padding: 1,
        width_padding: 2,
        text_color_256: 15,
        bg_color_256: 236,
        has_border: true,
    };

    /// Controls overlay configuration
    pub const CONTROLS: OverlayConfig = OverlayConfig {
        width: 50,
        height_padding: 1,
        width_padding: 2,
        text_color_256: 245,
        bg_color_256: 236,
        has_border: true,
    };

    /// Dashboard overlay configuration (merged stats + info, landscape layout).
    pub const DASHBOARD: OverlayConfig = OverlayConfig {
        width: 84,
        height_padding: 1,
        width_padding: 2,
        text_color_256: 245,
        bg_color_256: 236,
        has_border: true,
    };

    /// Preset comparison overlay configuration
    pub const PRESET_COMPARISON: OverlayConfig = OverlayConfig {
        width: 62,
        height_padding: 1,
        width_padding: 2,
        text_color_256: 15,
        bg_color_256: 236,
        has_border: true,
    };

    /// Config browser overlay configuration
    pub const CONFIG_BROWSER: OverlayConfig = OverlayConfig {
        width: 56,
        height_padding: 1,
        width_padding: 2,
        text_color_256: 15,
        bg_color_256: 236,
        has_border: true,
    };

    /// Config save overlay configuration
    pub const CONFIG_SAVE: OverlayConfig = OverlayConfig {
        width: 38,
        height_padding: 1,
        width_padding: 1,
        text_color_256: 15,
        bg_color_256: 236,
        has_border: true,
    };

    /// Keyboard hints overlay configuration
    pub const KEYBOARD_HINTS: OverlayConfig = OverlayConfig {
        width: 88,
        height_padding: 1,
        width_padding: 2,
        text_color_256: 15,
        bg_color_256: 236,
        has_border: true,
    };

    /// Attractor help overlay configuration
    pub const ATTRACTOR: OverlayConfig = OverlayConfig {
        width: 42,
        height_padding: 1,
        width_padding: 1,
        text_color_256: 15,
        bg_color_256: 236,
        has_border: true,
    };

    /// Obstacle help overlay configuration
    pub const OBSTACLE: OverlayConfig = OverlayConfig {
        width: 42,
        height_padding: 1,
        width_padding: 1,
        text_color_256: 15,
        bg_color_256: 236,
        has_border: true,
    };

    /// Mouse attractor help overlay configuration
    pub const MOUSE_ATTRACTOR: OverlayConfig = OverlayConfig {
        width: 46,
        height_padding: 1,
        width_padding: 1,
        text_color_256: 15,
        bg_color_256: 236,
        has_border: true,
    };

    /// Status bar overlay configuration
    pub const STATUS: OverlayConfig = OverlayConfig {
        width: 0,
        height_padding: 1,
        width_padding: 0,
        text_color_256: 250,
        bg_color_256: 234,
        has_border: false,
    };

    /// Notification overlay configuration
    pub const NOTIFICATION: OverlayConfig = OverlayConfig {
        width: 0,
        height_padding: 1,
        width_padding: 0,
        text_color_256: 15,
        bg_color_256: 235,
        has_border: false,
    };

    /// Palette editor overlay configuration
    pub const PALETTE_EDITOR: OverlayConfig = OverlayConfig {
        width: 56,
        height_padding: 0,
        width_padding: 1,
        text_color_256: 15,
        bg_color_256: 236,
        has_border: true,
    };
}

// --- END OverlayConfig ---

/// Overlay showing keyboard shortcuts.
pub struct KeyboardHintsOverlay;

impl KeyboardHintsOverlay {
    /// Total rendered width of the keyboard hints window.
    pub const WIDTH: usize = 88;
    /// Content width (inner drawable area).
    const CONTENT_WIDTH: usize = 74; // 88 - 2(border) - 6(pad-L) - 6(pad-R)

    /// Builds the keyboard hints overlay content.
    pub fn build_overlay(accent: RgbColor) -> RenderedOverlay {
        use TextAlignment::Left;

        let thin_sep = "─".repeat(Self::CONTENT_WIDTH);

        let mut overlay = PanelBuilder::new(Self::CONTENT_WIDTH, None)
            .with_columns(ColumnLayout::TwoEqual)
            .with_padding(Padding::new(2, 2, 6, 6))
            .with_title("KEYBOARD REFERENCE")
            .with_title_box()
            .add_two_col("SIMULATION", "SYSTEM", Left, Left)
            .add_two_col(
                "Space      Pause / Resume",
                "Ctrl+S     Save config",
                Left,
                Left,
            )
            .add_two_col("r          Restart", "Ctrl+L     Load config", Left, Left)
            .add_two_col(
                "1 \u{2013} 7      Select preset",
                "Ctrl+Z     Undo",
                Left,
                Left,
            )
            .add_two_col(
                "Shift+1\u{2013}7  Compare preset",
                "Ctrl+Y     Redo",
                Left,
                Left,
            )
            .add_two_col(
                "p          Palette editor",
                "q / Esc    Quit / close",
                Left,
                Left,
            )
            .add_empty()
            .add_single(thin_sep, Left)
            .add_empty()
            .add_two_col("OVERLAYS", "POST-PROCESSING", Left, Left)
            .add_two_col(
                "h          Controls panel",
                "m / M      Intensity map",
                Left,
                Left,
            )
            .add_two_col(
                "?          Keyboard hints",
                "[ / ]      Dither strength",
                Left,
                Left,
            )
            .add_two_col(
                "\\ / |      Dashboard",
                "9 / *      Cycle theme",
                Left,
                Left,
            )
            .add_two_col(
                "Tab        Cycle category",
                "F2         Choir on / off",
                Left,
                Left,
            )
            .build_overlay();
        overlay.rich_lines = Some(Self::generate_rich_lines(&overlay.lines, accent));
        overlay
    }

    /// Generates per-cell colour data: keybind tokens coloured with the accent colour.
    fn generate_rich_lines(lines: &[String], accent: RgbColor) -> Vec<Vec<RichCell>> {
        // Layout constants: 1 border + 6 padding = content starts at char index 7.
        const CONTENT_START: usize = 7;
        let col_width = Self::CONTENT_WIDTH / 2; // 37

        lines
            .iter()
            .map(|line| {
                let chars: Vec<char> = line.chars().collect();
                let n = chars.len();

                // Top border (█▀…), block separator (█▀…), and bottom border (█▄…):
                // all start with █ and have ▀ or ▄ as the second character.
                if chars.first() == Some(&'█')
                    && chars.get(1).map(|&c| c == '▀' || c == '▄').unwrap_or(false)
                {
                    return chars.iter().map(|&c| (c, None, None)).collect();
                }

                // Empty content rows: everything between the border █ chars is spaces.
                let inner_all_spaces = chars
                    .get(1..n.saturating_sub(1))
                    .map(|s| s.iter().all(|&c| c == ' '))
                    .unwrap_or(true);
                if inner_all_spaces {
                    return chars.iter().map(|&c| (c, None, None)).collect();
                }

                // Thin separator row: first content char (after border + padding) is ─.
                if chars.get(CONTENT_START) == Some(&'─') {
                    return chars.iter().map(|&c| (c, None, None)).collect();
                }

                // Regular content row: colour the key token in each column.
                let mut rich: Vec<RichCell> = chars.iter().map(|&c| (c, None, None)).collect();

                for col_idx in 0..2 {
                    let col_start = CONTENT_START + col_idx * col_width;
                    let col_end = (col_start + col_width).min(n);
                    if col_start >= n {
                        break;
                    }

                    let col_chars = &chars[col_start..col_end];
                    let col_text: String = col_chars.iter().collect();
                    let trimmed = col_text.trim_start();

                    if trimmed.is_empty() {
                        continue;
                    }

                    // Section headers are all-uppercase (e.g. "SIMULATION", "POST-PROCESSING").
                    let first_word = trimmed.split_whitespace().next().unwrap_or("");
                    let is_header = !first_word.is_empty()
                        && first_word.chars().all(|c| c.is_uppercase() || c == '-');
                    if is_header {
                        continue;
                    }

                    // The key is the text up to the first run of two or more spaces.
                    let indent = col_text.chars().count() - trimmed.chars().count();
                    let key_len = Self::key_char_len(trimmed);
                    let abs_start = col_start + indent;
                    let abs_end = (abs_start + key_len).min(n);
                    for cell in rich.iter_mut().take(abs_end).skip(abs_start) {
                        cell.1 = Some(accent);
                    }
                }

                rich
            })
            .collect()
    }

    /// Returns the char length of the key token: everything before the first double-space run.
    fn key_char_len(text: &str) -> usize {
        let chars: Vec<char> = text.chars().collect();
        for (i, window) in chars.windows(2).enumerate() {
            if window[0] == ' ' && window[1] == ' ' {
                return i;
            }
        }
        chars.len()
    }

    /// Calculates center position for the overlay.
    pub fn calculate_position(term_width: usize, term_height: usize) -> (usize, usize) {
        let x = (term_width.saturating_sub(Self::WIDTH)) / 2;
        let y = (term_height.saturating_sub(20)) / 2;
        (x, y)
    }
}

/// Overlay for comparing current settings against a preset.
pub struct PresetComparisonOverlay;

impl PresetComparisonOverlay {
    /// Total rendered width of the comparison window.
    pub const WIDTH: usize = 62;
    /// Content width (inner drawable area).
    const CONTENT_WIDTH: usize = 56; // 62 - 2(border) - 2*2(padding)

    /// Builds the comparison overlay showing modified parameters.
    pub fn build_overlay(
        current: &crate::terminal::control::RuntimeState,
        preset: Preset,
    ) -> RenderedOverlay {
        use TextAlignment::Left;

        let defaults = crate::terminal::control::DefaultValues::from_preset(preset);
        let pname = preset_name(preset);

        let mut builder = PanelBuilder::new(Self::CONTENT_WIDTH, None)
            .with_padding(Padding::new(1, 1, 2, 2))
            .with_title(format!("PRESET COMPARISON: {}", pname))
            .with_title_box()
            .add_empty()
            .add_single("Parameter        │ Current      │ Preset Default", Left)
            .add_single(
                "──────────────────┼──────────────┼──────────────────────",
                Left,
            );

        let add_row =
            |b: PanelBuilder, name: &str, cur: String, def: String, modif: bool| -> PanelBuilder {
                let marker = if modif { "⚙" } else { " " };
                b.add_single(
                    format!("{} {:<16} │ {:<12} │ {:<18}", marker, name, cur, def),
                    Left,
                )
            };

        builder = add_row(
            builder,
            "Sensor Angle",
            format!("{:.1}°", current.sensor_angle),
            format!("{:.1}°", defaults.sensor_angle),
            (current.sensor_angle - defaults.sensor_angle).abs() > 0.01,
        );
        builder = add_row(
            builder,
            "Sensor Dist",
            format!("{:.1}px", current.sensor_distance),
            format!("{:.1}px", defaults.sensor_distance),
            (current.sensor_distance - defaults.sensor_distance).abs() > 0.01,
        );
        builder = add_row(
            builder,
            "Turn Angle",
            format!("{:.1}°", current.rotation_angle),
            format!("{:.1}°", defaults.rotation_angle),
            (current.rotation_angle - defaults.rotation_angle).abs() > 0.01,
        );
        builder = add_row(
            builder,
            "Step Size",
            format!("{:.1}px", current.step_size),
            format!("{:.1}px", defaults.step_size),
            (current.step_size - defaults.step_size).abs() > 0.01,
        );
        builder = add_row(
            builder,
            "Decay Factor",
            format!("{:.3}x", current.decay_factor),
            format!("{:.3}x", defaults.decay_factor),
            (current.decay_factor - defaults.decay_factor).abs() > 0.001,
        );
        builder = add_row(
            builder,
            "Deposit Amt",
            format!("{:.1}x", current.deposit_amount),
            format!("{:.1}x", defaults.deposit_amount),
            (current.deposit_amount - defaults.deposit_amount).abs() > 0.01,
        );
        builder = add_row(
            builder,
            "Diff Sigma",
            format!("{:.2}x", current.diffusion_sigma),
            format!("{:.2}x", defaults.diffusion_sigma),
            (current.diffusion_sigma - defaults.diffusion_sigma).abs() > 0.01,
        );
        builder = add_row(
            builder,
            "Brightness",
            format!(
                "{:.1}x",
                crate::config_defaults::trail::brightness_gain(current.max_brightness)
            ),
            format!(
                "{:.1}x",
                crate::config_defaults::trail::brightness_gain(defaults.max_brightness)
            ),
            (current.max_brightness - defaults.max_brightness).abs() > 0.01,
        );

        builder
            .add_empty()
            .add_single("Press Enter to Apply Preset     Esc to Close", Left)
            .build_overlay()
    }

    /// Calculates center position for the comparison overlay.
    pub fn calculate_position(term_width: usize, term_height: usize) -> (usize, usize) {
        let x = (term_width.saturating_sub(Self::WIDTH)) / 2;
        let y = (term_height.saturating_sub(15)) / 2;
        (x, y)
    }
}

/// Overlay for browsing saved configurations.
pub struct ConfigBrowserOverlay;

impl ConfigBrowserOverlay {
    /// Total rendered width of the browser window.
    pub const WIDTH: usize = 56;
    /// Content width (inner drawable area).
    const CONTENT_WIDTH: usize = 50; // 56 - 2(border) - 2*2(padding)
    /// Maximum number of config rows shown at once (preserves panel height).
    const MAX_VISIBLE_CONFIGS: usize = 9;

    /// Computes the index of the first visible config so the selection stays in view.
    ///
    /// Returns the window start that keeps `selected` visible, anchored at the
    /// bottom of the `[start, start + max_visible)` window. Because the selection
    /// is always pinned to the bottom of the window, navigating upward jumps the
    /// window so `selected` sits on the last visible row rather than the nearest
    /// edge. The result is clamped so the window never runs past the end of the list.
    ///
    /// # Parameters
    /// - `selected`: currently highlighted index (clamped to `total - 1`)
    /// - `total`: number of configs available
    /// - `max_visible`: number of rows the panel can display at once
    fn config_browser_window(selected: usize, total: usize, max_visible: usize) -> usize {
        if total <= max_visible || max_visible == 0 {
            return 0;
        }
        selected
            .min(total - 1)
            .saturating_sub(max_visible - 1)
            .min(total - max_visible)
    }

    /// Builds the configuration list overlay.
    pub fn build_overlay(
        configs: &[crate::config_manager::SavedConfig],
        selected_index: usize,
    ) -> RenderedOverlay {
        use TextAlignment::Left;

        let mut builder = PanelBuilder::new(Self::CONTENT_WIDTH, None)
            .with_padding(Padding::new(1, 1, 2, 2))
            .with_title("SAVED CONFIGURATIONS")
            .with_title_box();

        if configs.is_empty() {
            builder = builder
                .add_empty()
                .add_single("No saved configurations", Left)
                .add_empty()
                .add_single("Press Ctrl+S to save current settings", Left)
                .add_empty();
        } else {
            let total = configs.len();
            let start =
                Self::config_browser_window(selected_index, total, Self::MAX_VISIBLE_CONFIGS);
            let end = (start + Self::MAX_VISIBLE_CONFIGS).min(total);

            // "Above" scroll indicator replaces the leading empty line when scrolled.
            if start > 0 {
                builder = builder.add_single(format!("▲ {} above", start), Left);
            } else {
                builder = builder.add_empty();
            }

            // Enumerate before skip so `i` stays the absolute index for marker logic.
            for (i, config) in configs
                .iter()
                .enumerate()
                .skip(start)
                .take(Self::MAX_VISIBLE_CONFIGS)
            {
                let num = i + 1;
                let selected_marker = if i == selected_index { "›" } else { " " };
                let name = &config.name;
                let palette = &config.palette;
                let pop = config.population / 1000;
                let line = format!(
                    "{}{} {} - {} - {}k agents",
                    selected_marker, num, name, palette, pop,
                );
                builder = builder.add_single(line, Left);
            }

            // "Below" scroll indicator when more entries remain past the window.
            // Always emit this row (blank when at the bottom) so the panel body
            // keeps a constant row count and never changes height while scrolling.
            if end < total {
                builder = builder.add_single(format!("▼ {} below", total - end), Left);
            } else {
                builder = builder.add_empty();
            }

            builder = builder
                .add_empty()
                .add_single("↑/↓: Navigate  Enter: Load  Del: Delete", Left);
        }

        builder.add_single("Esc: Cancel", Left).build_overlay()
    }

    /// Calculates center position for the browser overlay.
    pub fn calculate_position(term_width: usize, term_height: usize) -> (usize, usize) {
        let x = (term_width.saturating_sub(Self::WIDTH)) / 2;
        let y = (term_height.saturating_sub(15)) / 2;
        (x, y)
    }
}

/// Overlay for saving a new configuration.
pub struct ConfigSaveOverlay;

impl ConfigSaveOverlay {
    /// Total rendered width.
    const TOTAL_WIDTH: usize = 38;
    /// Content width (inner drawable area).
    const CONTENT_WIDTH: usize = 34; // 38 - 2(border) - 2*1(padding)

    /// Builds the save dialog overlay.
    pub fn build_overlay(name_input: &str) -> RenderedOverlay {
        use TextAlignment::Left;

        PanelBuilder::new(Self::CONTENT_WIDTH, None)
            .with_padding(Padding::new(0, 0, 1, 1))
            .with_title("SAVE CONFIGURATION")
            .with_title_box()
            .add_empty()
            .add_single(format!("Name: {:<25}", name_input), Left)
            .add_empty()
            .add_single("Enter: Save    Esc: Cancel", Left)
            .build_overlay()
    }

    /// Calculates center position for the save dialog.
    pub fn calculate_position(term_width: usize, term_height: usize) -> (usize, usize) {
        let x = (term_width.saturating_sub(Self::TOTAL_WIDTH)) / 2;
        let y = (term_height.saturating_sub(5)) / 2;
        (x, y)
    }
}

pub use crate::terminal::control::NotificationLevel;

/// Formats a notification string with an icon prefix for the given level.
///
/// Example output: `"✓  Config saved"` for `NotificationLevel::Success`.
pub fn format_notification(text: &str, level: NotificationLevel) -> String {
    format!("{}  {}", level.icon(), text)
}

/// Builds a two-line notification toast panel with an accent-colored border.
///
/// The panel shows:
/// - Line 1: `"{icon}  {message}"` with icon in the level's accent color
/// - Border characters colored with the theme's accent color for the notification level
pub fn build_notification_panel(
    msg: &str,
    level: NotificationLevel,
    panel_style: &PanelStyle,
) -> RenderedOverlay {
    let content = format!(" {}  {} ", level.icon(), msg);
    let cw = content.chars().count();
    let accent = match level {
        NotificationLevel::Info => panel_style.accent_info,
        NotificationLevel::Success => panel_style.accent_success,
        NotificationLevel::Warning => panel_style.accent_warning,
        NotificationLevel::Error => panel_style.accent_error,
    };
    let mut overlay = PanelBuilder::new(cw, None)
        .with_padding(Padding::COMPACT)
        .with_border_color(accent)
        .add_single(content, TextAlignment::Left)
        .build_overlay();
    overlay.rich_lines = Some(build_notification_rich_lines(&overlay.lines, accent));
    overlay
}

/// Applies per-character color overrides to a notification panel.
///
/// Colors:
/// - All border characters (`█`, `▀`, `▄`) → accent foreground
/// - Icon prefix on the content line → accent foreground
fn build_notification_rich_lines(lines: &[String], accent: RgbColor) -> Vec<Vec<RichCell>> {
    lines
        .iter()
        .enumerate()
        .map(|(line_idx, line)| {
            let chars: Vec<char> = line.chars().collect();
            chars
                .iter()
                .enumerate()
                .map(|(i, &c)| {
                    let fg = if matches!(c, '█' | '▀' | '▄') {
                        // Border characters: color with accent
                        Some(accent)
                    } else if line_idx == 1 {
                        // Content line: positions 0-1 are border+padding, position 2 onwards is content.
                        // Content format: " {icon}  {message}"
                        // icon = 1 char at position 3 (border(1) + pad(1) + space(1) = 3)
                        if i == 3 {
                            Some(accent)
                        } else {
                            None
                        }
                    } else {
                        None
                    };
                    (c, fg, None)
                })
                .collect()
        })
        .collect()
}

/// Utilities for rendering overlay elements (status line, help lists).
pub struct OverlayRenderer;

impl OverlayRenderer {
    #[allow(clippy::too_many_arguments)]
    /// Builds the status bar string displayed at the bottom of the screen.
    ///
    /// Uses `◦` as a segment separator for a clean powerline-inspired look.
    /// Segments are added in priority order; lower-priority segments are omitted
    /// when the terminal is too narrow to fit them.
    ///
    /// Layout (left to right):
    /// ```text
    ///   PRESET  ◦  1.0×  ▸  PALETTE  ▸  50k  [Z Y]  ⏸ PAUSED  ·  ? help
    /// ```
    pub fn build_status_line(
        _is_paused: bool,
        preset: Preset,
        time_scale: f32,
        palette: Palette,
        dither_mode: DitherMode,
        width: usize,
        population: Option<usize>,
        diffusion_kernel: Option<&str>,
        can_undo: bool,
        can_redo: bool,
        accent: Option<RgbColor>,
    ) -> (String, Vec<(usize, RgbColor)>) {
        const SEP: &str = "  ◦  ";
        let mut color_overrides: Vec<(usize, RgbColor)> = Vec::new();

        // Theme colors for status bar chrome (GRUVBOX-matching hardcoded values)
        let muted = RgbColor {
            r: 102,
            g: 92,
            b: 84,
        };
        let accent_success = RgbColor {
            r: 184,
            g: 187,
            b: 38,
        }; // green for undo ↺
        let accent_info = RgbColor {
            r: 131,
            g: 165,
            b: 152,
        }; // teal for redo ↻ and ?

        let preset_text = preset_name(preset);
        let palette_text = palette_name(palette);
        let time_text = format!("{:.1}×", time_scale);

        // Core left-side segments (always visible)
        let mut left = format!("  {}{}{}  ", preset_text, SEP, time_text);

        // Add palette swatch + name if space permits
        if width >= 52 {
            left.push_str("◦  ");
            // Color swatch: two block chars tinted with the palette accent
            if let Some(accent_color) = accent {
                let swatch_start = left.chars().count();
                left.push_str("■  ");
                color_overrides.push((swatch_start, accent_color));
                color_overrides.push((swatch_start + 1, accent_color));
            }
            left.push_str(&format!("{}  ", palette_text));
        }

        // Add population if space permits
        if let Some(pop) = population {
            if width >= 68 {
                left.push_str(&format!("◦  {}k  ", pop / 1000));
            }
        }

        // Add diffusion kernel if space permits
        if let Some(kernel) = diffusion_kernel {
            if width >= 88 {
                left.push_str(&format!("◦  {}  ", kernel));
            }
        }

        // Add dither mode if active and space permits
        let dither_segment = match dither_mode {
            DitherMode::None => None,
            DitherMode::Ordered { intensity, .. } => Some(format!("D {:.1}×", intensity)),
            DitherMode::ErrorDiffusion { .. } => Some("ED".to_string()),
            DitherMode::Hybrid { intensity, .. } => Some(format!("H {:.1}×", intensity)),
        };
        if let Some(ref d) = dither_segment {
            if width >= 60 {
                left.push_str(&format!("◦  {}  ", d));
            }
        }

        // Color all ◦ separator characters in the left segment with muted color
        let separator_positions: Vec<usize> = left
            .chars()
            .enumerate()
            .filter_map(|(i, c)| if c == '◦' { Some(i) } else { None })
            .collect();
        for pos in &separator_positions {
            color_overrides.push((*pos, muted));
        }

        // Right-side status indicators
        let mut right = String::new();
        let mut paused_offset_in_right: Option<usize> = None;
        let mut undo_offset_in_right: Option<usize> = None;
        let mut redo_offset_in_right: Option<usize> = None;
        let mut help_q_offset_in_right: Option<usize> = None;

        if can_undo || can_redo {
            if can_undo {
                undo_offset_in_right = Some(right.chars().count());
            }
            right.push_str(if can_undo { "↺" } else { "·" });
            right.push(' ');
            if can_redo {
                redo_offset_in_right = Some(right.chars().count());
            }
            right.push_str(if can_redo { "↻" } else { "·" });
            right.push_str("  ");
        }

        if _is_paused {
            paused_offset_in_right = Some(right.chars().count());
            right.push_str("⏸ PAUSED  ");
        }

        if width >= 100 {
            help_q_offset_in_right = Some(right.chars().count());
            right.push_str("? help  ");
        }

        // Combine: left segments + right-aligned indicators
        let combined_len = left.chars().count() + right.chars().count();
        let result = if combined_len <= width {
            // Pad between left and right
            let gap = width.saturating_sub(combined_len);
            let right_start = left.chars().count() + gap;

            // Color ↺ (undo) in accent_success
            if let Some(off) = undo_offset_in_right {
                color_overrides.push((right_start + off, accent_success));
            }
            // Color ↻ (redo) in accent_info
            if let Some(off) = redo_offset_in_right {
                color_overrides.push((right_start + off, accent_info));
            }
            // Color ? in accent_info
            if let Some(off) = help_q_offset_in_right {
                color_overrides.push((right_start + off, accent_info));
            }
            // Color ⏸ PAUSED with amber
            if let Some(paused_off) = paused_offset_in_right {
                let global_start = right_start + paused_off;
                let amber = RgbColor {
                    r: 215,
                    g: 153,
                    b: 33,
                };
                for i in 0.."⏸ PAUSED".chars().count() {
                    color_overrides.push((global_start + i, amber));
                }
            }
            format!("{}{}{}", left, " ".repeat(gap), right)
        } else {
            // No room to right-align; just return the left part
            left
        };

        (result, color_overrides)
    }

    /// Calculates the X position for the status line (left-aligned or centered).
    pub fn status_line_x(status_line: &str, width: usize) -> usize {
        if status_line.len() < width {
            2
        } else {
            width.saturating_sub(status_line.len() + 2)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_panel_builder_separator() {
        let panel = PanelBuilder::new(8, None).with_padding(Padding::new(0, 0, 1, 1));
        let sep = panel.render_separator_line();
        assert!(sep.starts_with('█'));
        assert!(sep.ends_with('█'));
        // total_width = 1 + 1 + 8 + 1 + 1 = 12
        assert_eq!(sep.chars().count(), 12);
    }

    #[test]
    fn test_keyboard_hints_position() {
        let (x, y) = KeyboardHintsOverlay::calculate_position(100, 100);
        assert_eq!(x, (100 - KeyboardHintsOverlay::WIDTH) / 2);
        assert_eq!(y, (100usize.saturating_sub(20)) / 2);
    }

    #[test]
    fn test_config_browser_overlay_empty() {
        let lines = ConfigBrowserOverlay::build_overlay(&[], 0);
        assert!(lines
            .lines
            .iter()
            .any(|l| l.contains("No saved configurations")));
        let (x, _y) = ConfigBrowserOverlay::calculate_position(100, 100);
        assert_eq!(x, 22);
    }

    #[test]
    fn test_config_save_overlay() {
        let lines = ConfigSaveOverlay::build_overlay("test");
        assert!(lines.lines.iter().any(|l| l.contains("test")));
        let (x, _y) = ConfigSaveOverlay::calculate_position(100, 100);
        assert_eq!(x, 31);
    }

    #[test]
    fn test_overlay_renderer_helper_positions() {
        assert_eq!(OverlayRenderer::status_line_x("abc", 10), 2);
        assert_eq!(OverlayRenderer::status_line_x("abcdefghij", 10), 0);
    }
}

/// Unified dashboard overlay combining stats and environment info in a landscape three-zone layout.
pub struct DashboardOverlay;

impl DashboardOverlay {
    /// Total rendered width of the dashboard window.
    pub const WIDTH: usize = 84;
    /// Content width (inner drawable area): 84 - 2(border) - 2*2(padding) = 78
    const CONTENT_WIDTH: usize = 78;
    /// Left column width within the two-column middle zone.
    const LEFT_COL: usize = 37;
    /// Right column width within the two-column middle zone.
    const RIGHT_COL: usize = 38;

    /// Calculates entropy of the trail map for complexity analysis.
    pub fn calculate_entropy(trail_map: &[f32], sample_rate: usize) -> f32 {
        if trail_map.is_empty() {
            return 0.0;
        }

        const NUM_BINS: usize = 16;
        let mut bins = [0usize; NUM_BINS];
        let mut total_samples = 0usize;

        for (i, &value) in trail_map.iter().enumerate() {
            if i % sample_rate == 0 && value > 0.01 {
                let normalized = (value / 10.0).clamp(0.0, 0.9999);
                let bin_idx = (normalized * NUM_BINS as f32) as usize;
                bins[bin_idx] += 1;
                total_samples += 1;
            }
        }

        if total_samples < 2 {
            return 0.0;
        }

        let mut entropy = 0.0f32;
        for &count in bins.iter() {
            if count > 0 {
                let p = count as f32 / total_samples as f32;
                entropy -= p * p.log2();
            }
        }

        let max_entropy = (NUM_BINS as f32).log2();
        if max_entropy > 0.0 {
            (entropy / max_entropy * 8.0).clamp(0.0, 8.0)
        } else {
            0.0
        }
    }

    /// Builds a proportional progress bar string.
    fn build_progress_bar(value: f32, max: f32, width: usize) -> String {
        let filled = if max > 0.0 {
            ((value / max).clamp(0.0, 1.0) * width as f32) as usize
        } else {
            0
        };
        let empty = width.saturating_sub(filled);
        format!("{}{}", "█".repeat(filled), "░".repeat(empty))
    }

    /// Builds a section header spanning the full content width.
    fn build_section_header(label: &str, width: usize) -> String {
        let prefix = format!(" ── {} ", label);
        let dashes = width.saturating_sub(prefix.chars().count());
        format!("{}{}", prefix, "─".repeat(dashes))
    }

    /// Builds a two-column row with a │ divider.
    fn build_two_col_row(left: &str, right: &str) -> String {
        let left_chars: usize = left.chars().count();
        let right_chars: usize = right.chars().count();
        let left_padded = if left_chars < Self::LEFT_COL {
            format!("{}{}", left, " ".repeat(Self::LEFT_COL - left_chars))
        } else {
            left.chars().take(Self::LEFT_COL).collect()
        };
        let right_padded = if right_chars < Self::RIGHT_COL {
            format!("{}{}", right, " ".repeat(Self::RIGHT_COL - right_chars))
        } else {
            right.chars().take(Self::RIGHT_COL).collect()
        };
        format!("{} │ {}", left_padded, right_padded)
    }

    /// Returns a status pill string for a boolean.
    fn build_status(on: bool) -> &'static str {
        if on {
            "◦ On"
        } else {
            "○ Off"
        }
    }

    #[allow(clippy::too_many_arguments)]
    /// Builds the dashboard overlay content.
    pub fn build_overlay(
        agent_count: usize,
        trail_sum: f32,
        trail_capacity: f32,
        trail_max: f32,
        entropy: f32,
        fps: f32,
        avg_fps: f32,
        frame_count: u64,
        elapsed_seconds: f32,
        grid_width: usize,
        grid_height: usize,
        attractor_count: usize,
        obstacle_count: usize,
        species_count: usize,
        memory_mb: f32,
        cpu_percent: f32,
        is_paused: bool,
        preset_name: &str,
        palette_name: &str,
        palette_colors: &[RgbColor],
        term_width: usize,
        term_height: usize,
        init_mode: &str,
        color_mode: &str,
        charset: &str,
        simd_enabled: bool,
        _decay_factor: f32,
        _sensor_angle: f32,
        seed: u64,
        food_source: &Option<String>,
        _warmup_frames: usize,
        auto_reset: bool,
        accent: RgbColor,
        panel_style: &PanelStyle,
    ) -> RenderedOverlay {
        use TextAlignment::Left;

        let cw = Self::CONTENT_WIDTH;

        let trail_percent = if trail_capacity > 0.0 {
            (trail_sum / trail_capacity * 100.0).min(99.9)
        } else {
            0.0
        };
        let elapsed_str = format_elapsed_time(elapsed_seconds);

        // Progress bars
        let fps_bar = Self::build_progress_bar(fps, 60.0, 32);
        let trail_bar = Self::build_progress_bar(trail_percent, 100.0, 15);
        let entropy_bar = Self::build_progress_bar(entropy, 8.0, 15);
        let cpu_bar = Self::build_progress_bar(cpu_percent, 100.0, 15);
        let mem_bar = Self::build_progress_bar(memory_mb, 100.0, 15);

        let grid_str = format!("{}×{}", grid_width, grid_height);
        let term_str = format!("{}×{}", term_width, term_height);
        let seed_str = seed.to_string();
        let food_str = food_source.as_ref().map(|f| {
            std::path::Path::new(f)
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or(f)
                .to_string()
        });

        // Build line list
        let mut lines: Vec<String> = Vec::new();

        // ── PERFORMANCE ──
        lines.push(format!(
            "{:<cw$}",
            Self::build_section_header("PERFORMANCE", cw)
        ));
        lines.push(format!("{:<cw$}", ""));
        // FPS label + bar + values (paused-aware)
        let fps_row = if is_paused {
            format!("  FPS   {:<32}  PAUSED", "░".repeat(32))
        } else {
            format!("  FPS   {}  {:.0}  avg {:.0}", fps_bar, fps, avg_fps)
        };
        lines.push(format!("{:<cw$}", fps_row));
        lines.push(format!("{:<cw$}", ""));

        // ── ENVIRONMENT (left) ─── | ── SIMULATION (right) ──
        let env_header = Self::build_section_header("ENVIRONMENT", Self::LEFT_COL);
        let sim_header = Self::build_section_header("SIMULATION", Self::RIGHT_COL);
        lines.push(format!(
            "{:<cw$}",
            Self::build_two_col_row(&env_header, &sim_header)
        ));
        lines.push(format!("{:<cw$}", ""));

        // Preset | Trail bar
        let trail_row = format!("  Trail   {}  {:.1}%", trail_bar, trail_percent);
        lines.push(format!(
            "{:<cw$}",
            Self::build_two_col_row(
                &format!("  Preset  {:>18}", Self::truncate(preset_name, 18)),
                &trail_row
            )
        ));

        // Palette | Entropy bar
        let entropy_row = format!("  Entropy {}  {:.1}", entropy_bar, entropy);
        lines.push(format!(
            "{:<cw$}",
            Self::build_two_col_row(
                &format!("  Palette {:>18}", Self::truncate(palette_name, 18)),
                &entropy_row
            )
        ));

        // Grid | Agents
        lines.push(format!(
            "{:<cw$}",
            Self::build_two_col_row(
                &format!("  Grid    {:>18}", grid_str),
                &format!("  Agents        {:>10}", Self::format_count(agent_count))
            )
        ));

        // Term | Max Trail
        lines.push(format!(
            "{:<cw$}",
            Self::build_two_col_row(
                &format!("  Term    {:>18}", term_str),
                &format!("  Max Trail     {:>7.2}×", trail_max)
            )
        ));

        // Seed | Frames
        lines.push(format!(
            "{:<cw$}",
            Self::build_two_col_row(
                &format!("  Seed    {:>18}", Self::truncate(&seed_str, 18)),
                &format!(
                    "  Frames        {:>10}",
                    Self::format_count(frame_count as usize)
                )
            )
        ));

        // Init | Time
        lines.push(format!(
            "{:<cw$}",
            Self::build_two_col_row(
                &format!("  Init    {:>18}", init_mode),
                &format!("  Time          {:>10}", elapsed_str)
            )
        ));

        // ── SYSTEM (full-width header) ──
        lines.push(format!("{:<cw$}", ""));
        lines.push(format!("{:<cw$}", Self::build_section_header("SYSTEM", cw)));
        lines.push(format!("{:<cw$}", ""));

        // CPU bar (left) | SIMD status (right)
        let cpu_row = format!("  CPU   {}  {:.0}%", cpu_bar, cpu_percent);
        lines.push(format!(
            "{:<cw$}",
            Self::build_two_col_row(
                &cpu_row,
                &format!("  SIMD          {:>10}", Self::build_status(simd_enabled))
            )
        ));

        // Mem bar (left) | Auto Reset (right)
        let mem_label = format!("{:.1} MB", memory_mb);
        let mem_row = format!("  Mem   {}  {}", mem_bar, mem_label);
        let auto_reset_right = format!("  Auto Reset    {:>10}", Self::build_status(auto_reset));
        lines.push(format!(
            "{:<cw$}",
            Self::build_two_col_row(&mem_row, &auto_reset_right)
        ));

        // Color | Charset (always separate columns)
        lines.push(format!(
            "{:<cw$}",
            Self::build_two_col_row(
                &format!("  Color   {:>18}", Self::truncate(color_mode, 18)),
                &format!("  Charset       {:>10}", Self::truncate(charset, 10))
            )
        ));

        // Optional food source row
        if let Some(food_display) = food_str.as_deref() {
            lines.push(format!(
                "{:<cw$}",
                Self::build_two_col_row(
                    &format!("  Food    {:>18}", Self::truncate(food_display, 18)),
                    ""
                )
            ));
        }

        // Extra info: species/attractors/obstacles (left) | empty (right)
        if attractor_count > 0 || obstacle_count > 0 || species_count > 1 {
            #[cfg(feature = "multi-species")]
            let spc_segment = format!("Spc:{} ", species_count);
            #[cfg(not(feature = "multi-species"))]
            let spc_segment = String::new();
            lines.push(format!(
                "{:<cw$}",
                Self::build_two_col_row(
                    &format!(
                        "  {}Att:{} Obs:{}",
                        spc_segment, attractor_count, obstacle_count
                    ),
                    ""
                )
            ));
        }

        // ── PALETTE strip ──
        lines.push(format!("{:<cw$}", ""));
        lines.push(format!(
            "{:<cw$}",
            Self::build_section_header("PALETTE", cw)
        ));
        // Palette gradient strip: 78 colored ▄ chars
        let palette_strip: String = (0..cw).map(|_| '▄').collect();
        lines.push(format!("{:<cw$}", palette_strip));

        // Build the overlay via PanelBuilder using raw lines
        let mut overlay = PanelBuilder::new(cw, None)
            .with_padding(Padding::new(2, 0, 2, 2))
            .with_title("DASHBOARD")
            .with_title_box();

        for line in &lines {
            overlay = overlay.add_single(line.clone(), Left);
        }

        let mut rendered = overlay.build_overlay();
        rendered.rich_lines = Some(Self::generate_rich_lines(
            &rendered.lines,
            fps,
            accent,
            panel_style,
            palette_colors,
        ));
        rendered
    }

    fn truncate(s: &str, max: usize) -> &str {
        let len = s.chars().count();
        if len <= max {
            s
        } else {
            // Find the byte offset of the `max`-th character boundary
            match s.char_indices().nth(max) {
                Some((byte_pos, _)) => &s[..byte_pos],
                None => s,
            }
        }
    }

    fn format_count(n: usize) -> String {
        if n >= 1_000_000 {
            format!("{:.1}M", n as f32 / 1_000_000.0)
        } else if n >= 1_000 {
            format!("{:.0}k", n as f32 / 1000.0)
        } else {
            format!("{}", n)
        }
    }

    /// Calculates centered position for the dashboard overlay.
    pub fn calculate_position(term_width: usize, term_height: usize) -> (usize, usize) {
        let x = (term_width.saturating_sub(Self::WIDTH)) / 2;
        let y = (term_height.saturating_sub(27)) / 2;
        (x, y)
    }

    /// Generates per-cell color overrides for the dashboard rich rendering.
    fn generate_rich_lines(
        lines: &[String],
        _fps: f32,
        accent: RgbColor,
        panel_style: &PanelStyle,
        palette_colors: &[RgbColor],
    ) -> Vec<Vec<RichCell>> {
        let muted = panel_style.muted;
        let text_secondary = panel_style.text_secondary;

        lines
            .iter()
            .map(|line| {
                let chars: Vec<char> = line.chars().collect();
                let n = chars.len();
                // Content starts at col 3 (border=1, padding.left=2)
                let content_start = 3.min(n);
                let content_end = n.saturating_sub(3);

                // Border rows (top/bottom) have no space at col 1 — skip all coloring.
                if n >= 2 && chars[1] != ' ' {
                    return chars.iter().map(|&c| (c, None, None)).collect();
                }

                // Detect section header rows: contain " ── "
                let is_section_header = {
                    let s: String = chars[content_start.min(n)..content_end.min(n)]
                        .iter()
                        .collect();
                    s.contains(" ── ")
                };

                // Detect palette strip: all content chars are '▄'
                let is_palette_strip = content_start < content_end
                    && chars[content_start..content_end].iter().all(|&c| c == '▄');

                // Detect FPS row
                let is_fps_row = content_start + 5 < n
                    && chars[content_start + 2..].starts_with(&['F', 'P', 'S']);

                // Detect divider row (contains '│' at approx col 40)
                let divider_col = chars.iter().position(|&c| c == '│');

                if is_palette_strip {
                    // Per-char palette color from sampled gradient
                    chars
                        .iter()
                        .enumerate()
                        .map(|(i, &c)| {
                            let fg = if i >= content_start && i < content_end {
                                let palette_idx = i - content_start;
                                palette_colors.get(palette_idx).copied()
                            } else {
                                None
                            };
                            (c, fg, None)
                        })
                        .collect()
                } else if is_section_header {
                    chars
                        .iter()
                        .enumerate()
                        .map(|(i, &c)| {
                            let fg = if i >= content_start && i < content_end {
                                if matches!(c, '─' | ' ') {
                                    Some(muted)
                                } else {
                                    Some(text_secondary)
                                }
                            } else {
                                None
                            };
                            (c, fg, None)
                        })
                        .collect()
                } else if is_fps_row {
                    chars
                        .iter()
                        .enumerate()
                        .map(|(i, &c)| {
                            let col = i.saturating_sub(content_start);
                            // FPS value: digits after bar — "  FPS   [bar]  N  avg N"
                            let fg = if i >= content_start
                                && i < content_end
                                && (8..52).contains(&col)
                                && (c.is_ascii_digit() || c == '.')
                            {
                                Some(accent)
                            } else if matches!(c, '█' | '░')
                                && i >= content_start
                                && i < content_end
                            {
                                // Progress bar coloring (accent fills, muted empties)
                                if c == '█' {
                                    Some(accent)
                                } else {
                                    Some(muted)
                                }
                            } else {
                                None
                            };
                            (c, fg, None)
                        })
                        .collect()
                } else {
                    // Generic row: color divider, status dots, progress bars
                    chars
                        .iter()
                        .enumerate()
                        .map(|(i, &c)| {
                            let fg = if Some(i) == divider_col {
                                Some(muted)
                            } else if c == '◦' {
                                Some(accent)
                            } else if c == '○' {
                                Some(muted)
                            } else if c == '█' && i >= content_start && i < content_end {
                                Some(accent)
                            } else if c == '░' && i >= content_start && i < content_end {
                                Some(muted)
                            } else {
                                None
                            };
                            (c, fg, None)
                        })
                        .collect()
                }
            })
            .collect()
    }
}

fn format_elapsed_time(seconds: f32) -> String {
    let total_secs = seconds as u64;
    let hours = total_secs / 3600;
    let minutes = (total_secs % 3600) / 60;
    let secs = total_secs % 60;

    if hours > 0 {
        format!("{}:{:02}:{:02}", hours, minutes, secs)
    } else {
        format!("{}:{:02}", minutes, secs)
    }
}

#[cfg(test)]
mod dashboard_tests {
    use super::*;
    use crate::render::theme::GRUVBOX_DARK;

    fn make_palette_colors() -> Vec<RgbColor> {
        (0..78)
            .map(|i| RgbColor {
                r: i as u8 * 3,
                g: 100,
                b: 200,
            })
            .collect()
    }

    #[test]
    fn test_dashboard_overlay_format() {
        let palette_colors = make_palette_colors();
        let accent = RgbColor {
            r: 57,
            g: 211,
            b: 83,
        };
        let rendered = DashboardOverlay::build_overlay(
            50000,
            1234567.0,
            8000000.0,
            8.5,
            5.5,
            30.0,
            28.5,
            1234,
            125.5,
            400,
            400,
            3,
            1,
            2,
            12.5,
            85.0,
            false,
            "Organic",
            "Heat",
            &palette_colors,
            120,
            40,
            "Random",
            "TrueColor",
            "HalfBlock",
            true,
            0.90,
            22.5,
            123456789,
            &None,
            0,
            false,
            accent,
            &GRUVBOX_DARK,
        );

        assert!(!rendered.lines.is_empty());
        // Solid-block borders
        assert!(
            rendered.lines[0].starts_with('█'),
            "Top border should start with solid block █, got: {}",
            rendered.lines[0]
        );
        assert!(
            rendered.lines.last().unwrap().starts_with('█'),
            "Bottom border should start with solid block █"
        );

        // All lines should be exactly WIDTH chars
        for (i, line) in rendered.lines.iter().enumerate() {
            assert_eq!(
                line.chars().count(),
                DashboardOverlay::WIDTH,
                "Line {} has wrong width: '{}' (expected {}, got {})",
                i,
                line,
                DashboardOverlay::WIDTH,
                line.chars().count()
            );
        }
    }

    #[test]
    fn test_dashboard_overlay_contains_key_data() {
        let palette_colors = make_palette_colors();
        let accent = RgbColor {
            r: 57,
            g: 211,
            b: 83,
        };
        let rendered = DashboardOverlay::build_overlay(
            50000,
            1234567.0,
            8000000.0,
            8.5,
            5.5,
            30.0,
            28.5,
            1234,
            125.5,
            400,
            400,
            0,
            0,
            1,
            12.5,
            85.0,
            false,
            "Organic",
            "Heat",
            &palette_colors,
            120,
            40,
            "Random",
            "TrueColor",
            "HalfBlock",
            true,
            0.90,
            22.5,
            42,
            &None,
            0,
            false,
            accent,
            &GRUVBOX_DARK,
        );

        let all_text: String = rendered.lines.join("\n");
        assert!(
            all_text.contains("PERFORMANCE"),
            "Missing PERFORMANCE section"
        );
        assert!(
            all_text.contains("ENVIRONMENT"),
            "Missing ENVIRONMENT section"
        );
        assert!(
            all_text.contains("SIMULATION"),
            "Missing SIMULATION section"
        );
        assert!(all_text.contains("PALETTE"), "Missing PALETTE section");
        assert!(all_text.contains("Organic"), "Missing preset name");
        assert!(all_text.contains("Heat"), "Missing palette name");
        assert!(all_text.contains("FPS"), "Missing FPS row");
    }

    #[test]
    fn test_dashboard_overlay_position() {
        let (x, y) = DashboardOverlay::calculate_position(120, 40);
        assert_eq!(x, (120 - DashboardOverlay::WIDTH) / 2);
        assert_eq!(y, (40usize.saturating_sub(27)) / 2);
    }

    #[test]
    fn test_entropy_calculation() {
        let uniform = vec![1.0; 40000];
        let entropy_uniform = DashboardOverlay::calculate_entropy(&uniform, 100);
        assert!(
            entropy_uniform < 2.0,
            "uniform should have low entropy, got {}",
            entropy_uniform
        );

        let varied: Vec<f32> = (0..40000).map(|i| i as f32 / 400.0).collect();
        let entropy_varied = DashboardOverlay::calculate_entropy(&varied, 100);
        assert!(
            entropy_varied > entropy_uniform,
            "varied ({}) should have higher entropy than uniform ({})",
            entropy_varied,
            entropy_uniform
        );
    }

    #[test]
    fn test_entropy_empty_trail() {
        let empty: Vec<f32> = vec![];
        let entropy = DashboardOverlay::calculate_entropy(&empty, 10);
        assert_eq!(entropy, 0.0);
    }

    #[test]
    fn test_format_elapsed_time() {
        assert_eq!(format_elapsed_time(30.0), "0:30");
        assert_eq!(format_elapsed_time(90.0), "1:30");
        assert_eq!(format_elapsed_time(3661.0), "1:01:01");
        assert_eq!(format_elapsed_time(0.0), "0:00");
    }
}

#[cfg(test)]
mod status_line_tests {
    use super::*;
    use crate::cli::Palette;
    use crate::render::dither::DitherMode;
    use crate::simulation::config::Preset;

    #[test]
    fn test_status_line_narrow_terminal_40_cols() {
        let (status, _) = OverlayRenderer::build_status_line(
            false,
            Preset::Organic,
            1.0,
            Palette::Organic,
            DitherMode::None,
            40,
            Some(50000),
            Some("Mean3x3"),
            false,
            false,
            None,
        );
        // At 40 cols: should only have preset and time
        assert!(status.contains("Organic"));
        assert!(status.contains("1.0×"));
        // Should not have palette or population (too narrow)
        assert!(!status.contains("50k"));
    }

    #[test]
    fn test_status_line_medium_terminal_80_cols() {
        let (status, _) = OverlayRenderer::build_status_line(
            false,
            Preset::Network,
            2.5,
            Palette::Heat,
            DitherMode::None,
            80,
            Some(50000),
            Some("Mean3x3"),
            false,
            false,
            None,
        );
        // At 80 cols: should have preset, time, palette, and population
        assert!(status.contains("Network"));
        assert!(status.contains("2.5×"));
        assert!(status.contains("Heat"));
        assert!(status.contains("50k"));
        // Should not have diffusion kernel (needs 90+)
        assert!(!status.contains("Mean3x3"));
        // Should not have help text (needs 100+)
        assert!(!status.contains("?"));
    }

    #[test]
    fn test_status_line_wide_terminal_120_cols() {
        let (status, _) = OverlayRenderer::build_status_line(
            false,
            Preset::Exploratory,
            1.5,
            Palette::Ocean,
            DitherMode::None,
            120,
            Some(30000),
            Some("Gaussian"),
            false,
            false,
            None,
        );
        // At 120 cols: should have everything including help
        assert!(status.contains("Exploratory"));
        assert!(status.contains("1.5×"));
        assert!(status.contains("Ocean"));
        assert!(status.contains("30k"));
        assert!(status.contains("Gaussian"));
        assert!(status.contains("?"));
    }

    #[test]
    fn test_status_line_paused() {
        let (status, colors) = OverlayRenderer::build_status_line(
            true,
            Preset::Organic,
            1.0,
            Palette::Organic,
            DitherMode::None,
            120,
            Some(50000),
            Some("Mean3x3"),
            false,
            false,
            None,
        );
        assert!(status.contains("⏸ PAUSED"));
        // PAUSED should have amber color overrides
        let amber = RgbColor {
            r: 215,
            g: 153,
            b: 33,
        };
        assert!(colors.iter().any(|(_, c)| *c == amber));
    }

    #[test]
    fn test_status_line_with_dither() {
        let (status, _) = OverlayRenderer::build_status_line(
            false,
            Preset::Organic,
            1.0,
            Palette::Organic,
            DitherMode::Ordered {
                intensity: 0.5,
                matrix: crate::render::dither::DitherMatrix::Bayer4x4,
            },
            80,
            Some(50000),
            Some("Mean3x3"),
            false,
            false,
            None,
        );
        assert!(status.contains("D 0.5×"));
    }

    #[test]
    fn test_status_line_without_optional_params() {
        let (status, _) = OverlayRenderer::build_status_line(
            false,
            Preset::Organic,
            1.0,
            Palette::Organic,
            DitherMode::None,
            120,
            None,
            None,
            false,
            false,
            None,
        );
        // Should still work without population or diffusion kernel
        assert!(status.contains("Organic"));
        assert!(status.contains("1.0×"));
    }

    #[test]
    fn test_keyboard_hints_overlay_format() {
        let hints_lines = KeyboardHintsOverlay::build_overlay(RgbColor {
            r: 180,
            g: 220,
            b: 100,
        });

        // Solid-block borders
        for line in &hints_lines.lines {
            assert!(
                line.starts_with('█') || line.starts_with('▀') || line.starts_with('▄'),
                "Line should start with solid block char, got: {}",
                line
            );
            assert!(
                line.ends_with('█') || line.ends_with('▀') || line.ends_with('▄'),
                "Line should end with solid block char, got: {}",
                line
            );
        }

        // All lines should be KeyboardHintsOverlay::WIDTH chars wide
        for line in &hints_lines.lines {
            assert_eq!(
                line.chars().count(),
                KeyboardHintsOverlay::WIDTH,
                "Keyboard hints line has unexpected width ({}): {}",
                line.chars().count(),
                line
            );
        }
    }

    #[test]
    fn test_preset_comparison_overlay() {
        use crate::cli::PauseStyle;
        use crate::render::palette::IntensityMapping;
        use crate::simulation::config::{InitMode, SimConfig};

        let mut state = crate::terminal::control::RuntimeState::new(
            42,
            InitMode::Random,
            Preset::Organic,
            0,
            0,
            crate::terminal::control::MouseInteractionMode::Disabled,
            0.0,
            IntensityMapping::linear(),
            &SimConfig::default(),
            PauseStyle::Vignette,
            false,
            false,
        );
        state.sensor_angle = 90.0; // Changed from default

        let lines = PresetComparisonOverlay::build_overlay(&state, Preset::Organic);
        assert!(!lines.lines.is_empty());
        let content_lines = lines
            .lines
            .iter()
            .filter(|l| l.contains("Sensor Angle"))
            .collect::<Vec<_>>();
        assert!(!content_lines.is_empty());
        // Should show modified marker ⚙
        assert!(content_lines[0].contains('⚙'));
    }

    #[test]
    fn test_config_browser_overlay_items() {
        let configs = vec![crate::config_manager::SavedConfig {
            name: "Test Config".to_string(),
            description: None,
            population: 10000,
            sensor_angle: 0.0,
            sensor_distance: 0.0,
            rotation_angle: 0.0,
            step_size: 0.0,
            decay_factor: 0.0,
            deposit_amount: 0.0,
            max_brightness: 0.0,
            diffusion_kernel: "mean3x3".to_string(),
            diffusion_sigma: 0.0,
            palette: "Forest".to_string(),
            charset: "ascii".to_string(),
            reverse_palette: false,
            invert_palette: false,
            warmup_frames: 0,
            food_persist: false,
            auto_reset: false,
            grid: false,
            grid_style: None,
            init_mode: "random".to_string(),
            food_path: None,
            background_color: None,
            intensity_mapping: None,
            intensity_mapping_base: None,
            intensity_mapping_gamma: None,
            intensity_mapping_levels: None,
            window_frame: "frame".to_string(),
            chrome_style: "minimal".to_string(),
            aspect: "3:2".to_string(),
            window_padding: "auto".to_string(),
            show_status_bar: false,
            min_sim_size: "20x10".to_string(),
            min_frame_size: "12x6".to_string(),
        }];

        let lines = ConfigBrowserOverlay::build_overlay(&configs, 0);
        assert!(lines.lines.iter().any(|l| l.contains("Test Config")));
        assert!(lines.lines.iter().any(|l| l.contains("10k agents")));
    }

    fn make_saved_config(name: &str) -> crate::config_manager::SavedConfig {
        crate::config_manager::SavedConfig {
            name: name.to_string(),
            description: None,
            population: 10000,
            sensor_angle: 0.0,
            sensor_distance: 0.0,
            rotation_angle: 0.0,
            step_size: 0.0,
            decay_factor: 0.0,
            deposit_amount: 0.0,
            max_brightness: 0.0,
            diffusion_kernel: "mean3x3".to_string(),
            diffusion_sigma: 0.0,
            palette: "Forest".to_string(),
            charset: "ascii".to_string(),
            reverse_palette: false,
            invert_palette: false,
            warmup_frames: 0,
            food_persist: false,
            auto_reset: false,
            grid: false,
            grid_style: None,
            init_mode: "random".to_string(),
            food_path: None,
            background_color: None,
            intensity_mapping: None,
            intensity_mapping_base: None,
            intensity_mapping_gamma: None,
            intensity_mapping_levels: None,
            window_frame: "frame".to_string(),
            chrome_style: "minimal".to_string(),
            aspect: "3:2".to_string(),
            window_padding: "auto".to_string(),
            show_status_bar: false,
            min_sim_size: "20x10".to_string(),
            min_frame_size: "12x6".to_string(),
        }
    }

    #[test]
    fn test_config_browser_window_helper() {
        // total <= max_visible: always anchored at top.
        assert_eq!(ConfigBrowserOverlay::config_browser_window(0, 5, 9), 0);
        assert_eq!(ConfigBrowserOverlay::config_browser_window(4, 5, 9), 0);
        assert_eq!(ConfigBrowserOverlay::config_browser_window(8, 9, 9), 0);

        // Selection within the first window: anchored at top.
        assert_eq!(ConfigBrowserOverlay::config_browser_window(0, 15, 9), 0);
        assert_eq!(ConfigBrowserOverlay::config_browser_window(8, 15, 9), 0);

        // Selection past the first window: scroll just enough to reveal it.
        // selected 11/15 with max 9 -> start 3 (showing 3..12 includes 11).
        assert_eq!(ConfigBrowserOverlay::config_browser_window(11, 15, 9), 3);
        assert_eq!(ConfigBrowserOverlay::config_browser_window(9, 15, 9), 1);

        // Selection at the end: clamp so window never runs past total.
        // selected 14/15 with max 9 -> start 6 (showing 6..15).
        assert_eq!(ConfigBrowserOverlay::config_browser_window(14, 15, 9), 6);

        // Out-of-range selection is clamped to the last item.
        assert_eq!(ConfigBrowserOverlay::config_browser_window(99, 15, 9), 6);
    }

    #[test]
    fn test_config_browser_scrolls_to_selection() {
        let configs: Vec<_> = (0..15)
            .map(|i| make_saved_config(&format!("Config{:02}", i)))
            .collect();

        // Select an entry well beyond the first window (index 11 of 15).
        let overlay = ConfigBrowserOverlay::build_overlay(&configs, 11);

        // The selected entry must have scrolled into view, marked with the caret.
        assert!(
            overlay
                .lines
                .iter()
                .any(|l| l.contains("›") && l.contains("Config11")),
            "selected entry Config11 should be visible with the selection marker; got:\n{}",
            overlay.lines.join("\n")
        );

        // Entries that scrolled off the top should no longer be rendered.
        assert!(
            !overlay.lines.iter().any(|l| l.contains("Config00")),
            "Config00 should have scrolled off the top"
        );

        // An "above" indicator should be present since we scrolled down.
        assert!(
            overlay.lines.iter().any(|l| l.contains("above")),
            "expected an 'above' scroll indicator; got:\n{}",
            overlay.lines.join("\n")
        );
    }

    #[test]
    fn test_config_browser_top_anchored_with_below_indicator() {
        let configs: Vec<_> = (0..15)
            .map(|i| make_saved_config(&format!("Config{:02}", i)))
            .collect();

        let overlay = ConfigBrowserOverlay::build_overlay(&configs, 0);

        // First entry visible and marked.
        assert!(overlay
            .lines
            .iter()
            .any(|l| l.contains("›") && l.contains("Config00")));
        // A "below" indicator should be present (more configs off the bottom).
        assert!(
            overlay.lines.iter().any(|l| l.contains("below")),
            "expected a 'below' scroll indicator; got:\n{}",
            overlay.lines.join("\n")
        );
        // No "above" indicator when anchored at top.
        assert!(!overlay.lines.iter().any(|l| l.contains("above")));
    }

    #[test]
    fn test_config_browser_bottom_anchored_constant_height() {
        let configs: Vec<_> = (0..15)
            .map(|i| make_saved_config(&format!("Config{:02}", i)))
            .collect();
        let total = configs.len();

        // Scrolled fully to the bottom: select the last entry (index 14 of 15).
        let bottom = ConfigBrowserOverlay::build_overlay(&configs, total - 1);

        // (a) The selected (last) entry is visible and marked.
        assert!(
            bottom
                .lines
                .iter()
                .any(|l| l.contains("›") && l.contains("Config14")),
            "selected last entry Config14 should be visible with the selection marker; got:\n{}",
            bottom.lines.join("\n")
        );

        // (b) An "above" indicator is present (earlier entries scrolled off the top).
        assert!(
            bottom.lines.iter().any(|l| l.contains("above")),
            "expected an 'above' scroll indicator at the bottom; got:\n{}",
            bottom.lines.join("\n")
        );

        // (c) No "below" indicator text since there is nothing past the window.
        assert!(
            !bottom.lines.iter().any(|l| l.contains("below")),
            "expected no 'below' scroll indicator at the bottom; got:\n{}",
            bottom.lines.join("\n")
        );

        // Panel height must stay constant across scroll positions. A mid-scroll
        // render (which DOES emit a "below" indicator) must have the same number
        // of rows as the bottom render (which emits a blank line in its place).
        let mid = ConfigBrowserOverlay::build_overlay(&configs, 11);
        assert!(
            mid.lines.iter().any(|l| l.contains("below")),
            "mid-scroll render should still show a 'below' indicator; got:\n{}",
            mid.lines.join("\n")
        );
        assert_eq!(
            bottom.lines.len(),
            mid.lines.len(),
            "panel body row count must be identical at the bottom and mid-scroll \
             (Fix 1: constant ▼ slot); bottom={}, mid={}",
            bottom.lines.len(),
            mid.lines.len()
        );
    }
}

// --- PauseOverlay ---

/// VCR-style pause screen: logo rendered in palette colors + blinking badge.
pub struct PauseOverlay;

impl PauseOverlay {
    /// Build a logo overlay using sculpted quadrant characters with dual-color.
    ///
    /// Uses `map_sculpted_outline` (16 block elements + 4 triangle fills) for
    /// shape fidelity, with independent fg/bg per cell. The `brightness_map`
    /// must be `(logo_w * 2) x (logo_h * 2)` pixels (2×2 quadrants per cell).
    #[allow(clippy::too_many_arguments)]
    pub fn build_logo(
        brightness_map: &[f32],
        logo_w: usize,
        logo_h: usize,
        palette: Palette,
        reverse: bool,
        invert: bool,
        hue_shift: f32,
        mapping: Option<&crate::render::palette::IntensityMapping>,
    ) -> RenderedOverlay {
        use crate::render::charset::map_quadrant;
        use crate::render::palette::map_brightness_rgb;

        const THRESHOLD: f32 = 0.12;

        let pixel_w = logo_w * 2;
        let mut rich_lines: Vec<Vec<RichCell>> = Vec::with_capacity(logo_h);
        let mut lines: Vec<String> = Vec::with_capacity(logo_h);

        for row in 0..logo_h {
            let mut rich_row: Vec<RichCell> = Vec::with_capacity(logo_w);
            let mut line = String::with_capacity(logo_w * 3);
            for col in 0..logo_w {
                // Sample 2×2 quadrants
                let tl_idx = (row * 2) * pixel_w + col * 2;
                let tr_idx = tl_idx + 1;
                let bl_idx = (row * 2 + 1) * pixel_w + col * 2;
                let br_idx = bl_idx + 1;

                let tl_raw = brightness_map.get(tl_idx).copied().unwrap_or(0.0);
                let tr_raw = brightness_map.get(tr_idx).copied().unwrap_or(0.0);
                let bl_raw = brightness_map.get(bl_idx).copied().unwrap_or(0.0);
                let br_raw = brightness_map.get(br_idx).copied().unwrap_or(0.0);

                // Apply intensity mapping to pixel values before thresholding
                // so log/exp/perlin etc. affect shape and visibility
                let tl = if let Some(m) = mapping {
                    m.apply(tl_raw.clamp(0.0, 1.0))
                } else {
                    tl_raw
                };
                let tr = if let Some(m) = mapping {
                    m.apply(tr_raw.clamp(0.0, 1.0))
                } else {
                    tr_raw
                };
                let bl = if let Some(m) = mapping {
                    m.apply(bl_raw.clamp(0.0, 1.0))
                } else {
                    bl_raw
                };
                let br = if let Some(m) = mapping {
                    m.apply(br_raw.clamp(0.0, 1.0))
                } else {
                    br_raw
                };

                // Count "on" quadrants and accumulate brightness
                let vals = [tl, tr, bl, br];
                let mut total_brightness: f32 = 0.0;
                let mut on_count: u32 = 0;
                for &v in &vals {
                    if v > THRESHOLD {
                        total_brightness += v;
                        on_count += 1;
                    }
                }

                if on_count == 0 {
                    // Fully transparent — dimmed sim shows through
                    rich_row.push((' ', None, None));
                    line.push(' ');
                } else {
                    let ch = map_quadrant(tl, tr, bl, br, THRESHOLD);
                    let avg = total_brightness / on_count as f32;
                    // Pass None for mapping here — already applied above
                    let fg_color =
                        map_brightness_rgb(avg, palette.clone(), reverse, invert, hue_shift, None);

                    if on_count == 4 {
                        // Full block — both fg and bg colored
                        rich_row.push((ch, Some(fg_color), Some(fg_color)));
                    } else {
                        // Partial — fg colored, bg transparent
                        rich_row.push((ch, Some(fg_color), None));
                    }
                    line.push(ch);
                }
            }
            rich_lines.push(rich_row);
            lines.push(line);
        }

        // Trim empty rows from top and bottom so centering works on visible content
        let is_empty_row = |row: &Vec<RichCell>| row.iter().all(|&(ch, _, _)| ch == ' ');
        let first_nonempty = rich_lines
            .iter()
            .position(|r| !is_empty_row(r))
            .unwrap_or(0);
        let last_nonempty = rich_lines
            .iter()
            .rposition(|r| !is_empty_row(r))
            .unwrap_or(rich_lines.len().saturating_sub(1));
        let rich_lines = rich_lines[first_nonempty..=last_nonempty].to_vec();
        let lines = lines[first_nonempty..=last_nonempty].to_vec();

        RenderedOverlay {
            lines,
            title_box: None,
            rich_lines: Some(rich_lines),
        }
    }

    /// Build a blinking "⏸ PAUSED" badge.
    ///
    /// `visible` controls whether the badge text is shown (for blink effect).
    pub fn build_badge(visible: bool) -> RenderedOverlay {
        let content = if visible {
            "  ⏸  PAUSED  "
        } else {
            "              "
        };
        let accent = RgbColor {
            r: 220,
            g: 180,
            b: 60,
        };
        let rich_row: Vec<RichCell> = content.chars().map(|c| (c, Some(accent), None)).collect();
        RenderedOverlay {
            lines: vec![content.to_string()],
            title_box: None,
            rich_lines: Some(vec![rich_row]),
        }
    }
}

// --- ExpandedChromeOverlay ---

/// Builds the 2-row title block and 2-row footer for expanded window chrome.
///
/// This is a pure data-builder — it produces plain strings with no ANSI escape
/// codes. The caller is responsible for positioning and rendering them into the
/// frame buffer at the appropriate rows.
pub struct ExpandedChromeOverlay;

impl ExpandedChromeOverlay {
    /// Builds the 2-row title block shown at the top of the window chrome.
    ///
    /// Returns `[row1, row2]` as plain strings (no ANSI). Row 1 contains the
    /// app name and preset. Row 2 contains palette, charset, and agent count.
    ///
    /// # Parameters
    /// - `preset`: Active simulation preset
    /// - `palette`: Active color palette
    /// - `charset_str`: Human-readable charset name (e.g. "HalfBlock")
    /// - `population`: Number of agents (used to compute k-value)
    /// - `_width`: Terminal width (reserved for future truncation logic)
    pub fn build_title_block(
        preset: Preset,
        palette: Palette,
        charset_str: &str,
        population: usize,
        _width: usize,
    ) -> [String; 2] {
        let preset_str = preset_name(preset);
        let palette_str = palette_name(palette);
        let pop_k = population / 1000;
        [
            format!("  \u{25C9} tslime \u{00B7} {}", preset_str),
            format!(
                "  {} palette \u{00B7} {} \u{00B7} {}k ag.",
                palette_str, charset_str, pop_k
            ),
        ]
    }

    /// Builds footer row 1 (status) by delegating to `OverlayRenderer::build_status_line`.
    ///
    /// Returns the status string and per-character color overrides, identical to
    /// what the status bar would show in windowed mode.
    #[allow(clippy::too_many_arguments)]
    pub fn build_footer_status(
        is_paused: bool,
        preset: Preset,
        time_scale: f32,
        palette: Palette,
        dither_mode: DitherMode,
        width: usize,
        population: Option<usize>,
        diffusion_kernel: Option<&str>,
        can_undo: bool,
        can_redo: bool,
        accent: Option<RgbColor>,
    ) -> (String, Vec<(usize, RgbColor)>) {
        OverlayRenderer::build_status_line(
            is_paused,
            preset,
            time_scale,
            palette,
            dither_mode,
            width,
            population,
            diffusion_kernel,
            can_undo,
            can_redo,
            accent,
        )
    }

    /// Builds footer row 2: context-sensitive keybind hints.
    ///
    /// When `is_modal_open` is true (e.g. config browser overlay is showing),
    /// the hints switch to modal navigation keys. Otherwise, the standard
    /// running-mode shortcuts are displayed.
    ///
    /// # Parameters
    /// - `is_modal_open`: Whether a modal overlay is currently focused
    /// - `_width`: Terminal width (reserved for future truncation logic)
    pub fn build_footer_keybinds(is_modal_open: bool, _width: usize) -> String {
        if is_modal_open {
            "  \u{2191}\u{2193} navigate \u{00B7} enter select \u{00B7} esc close".to_string()
        } else {
            "  q quit \u{00B7} h help \u{00B7} space pause \u{00B7} c cycle palette \u{00B7} \\ dashboard"
                .to_string()
        }
    }
}

#[cfg(test)]
mod expanded_chrome_tests {
    use super::*;

    #[test]
    fn test_title_block_row1_contains_app_and_preset() {
        let rows = ExpandedChromeOverlay::build_title_block(
            Preset::Organic,
            Palette::Forest,
            "HalfBlock",
            50_000,
            80,
        );
        assert!(rows[0].contains("tslime"), "row0: {}", rows[0]);
        assert!(
            rows[0].contains("organic") || rows[0].contains("Organic"),
            "row0: {}",
            rows[0]
        );
    }

    #[test]
    fn test_title_block_row2_contains_palette_charset_population() {
        let rows = ExpandedChromeOverlay::build_title_block(
            Preset::Organic,
            Palette::Forest,
            "HalfBlock",
            50_000,
            80,
        );
        assert!(
            rows[1].contains("Forest") || rows[1].contains("forest"),
            "row1: {}",
            rows[1]
        );
        assert!(rows[1].contains("HalfBlock"), "row1: {}", rows[1]);
        assert!(rows[1].contains("50k"), "row1: {}", rows[1]);
    }

    #[test]
    fn test_footer_keybinds_running() {
        let hint = ExpandedChromeOverlay::build_footer_keybinds(false, 80);
        assert!(hint.contains("q quit"), "hint: {}", hint);
        assert!(hint.contains("space pause"), "hint: {}", hint);
    }

    #[test]
    fn test_footer_keybinds_modal() {
        let hint = ExpandedChromeOverlay::build_footer_keybinds(true, 80);
        assert!(hint.contains("esc close"), "hint: {}", hint);
    }
}
