use crate::cli::Palette;
use crate::render::dither::DitherMode;
use crate::render::palette::RgbColor;
use crate::render::theme::PanelStyle;
use crate::simulation::config::Attractor;
use crate::simulation::config::MouseAttractor;
use crate::simulation::config::Obstacle;
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

    /// Stats overlay configuration
    pub const STATS: OverlayConfig = OverlayConfig {
        width: 32,
        height_padding: 1,
        width_padding: 2,
        text_color_256: 245,
        bg_color_256: 236,
        has_border: true,
    };

    /// Info overlay configuration
    pub const INFO: OverlayConfig = OverlayConfig {
        width: 28,
        height_padding: 1,
        width_padding: 1,
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
        width: 60,
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
    pub const WIDTH: usize = 64;
    /// Content width (inner drawable area).
    const CONTENT_WIDTH: usize = 54; // 64 - 2(border) - 2*4(padding)

    /// Builds the keyboard hints overlay content.
    pub fn build_overlay() -> RenderedOverlay {
        use TextAlignment::Left;

        PanelBuilder::new(Self::CONTENT_WIDTH, None)
            .with_padding(Padding::new(1, 0, 4, 4))
            .with_title("KEYBOARD SHORTCUTS")
            .with_title_box()
            .add_empty()
            .add_single("SIMULATION                VISUALS", Left)
            .add_single("Space    : Pause          c, Shift+C : Palette", Left)
            .add_single("p        : Palette Editor", Left)
            .add_single("r        : Restart        o          : Palette Shift", Left)
            .add_single(
                "q, Esc   : Quit           x          : Invert Palette",
                Left,
            )
            .add_single(
                "+, -     : Time Scale     z          : Reverse Palette",
                Left,
            )
            .add_empty()
            .add_single("PRESETS                   POST-PROCESSING", Left)
            .add_single("1-7      : Presets        d, D       : Dither Mode", Left)
            .add_single("8        : Randomize      [, ]       : Dither Inten.", Left)
            .add_single(
                "0        : Defaults       b          : Auto Normalize",
                Left,
            )
            .add_single("                          v          : Motion Blur", Left)
            .add_single(
                "SYSTEM                    n, Shift+N : Max Brightness",
                Left,
            )
            .add_single("h        : Controls       m, M       : Intensity Map", Left)
            .add_single("?, |     : Help/Info       f          : Fast Mode", Left)
            .add_single("\\        : Stats           g          : Save PNG", Left)
            .add_single("Tab      : Category       Ctrl+S     : Save Config", Left)
            .add_single("                          Ctrl+L     : Load Config", Left)
            .add_empty()
            .add_single("DETAILED CONTROLS (Use Shift to decrease values)", Left)
            .add_single("A: Sensor Angle   J: Sensor Dist    T: Turn Angle", Left)
            .add_single("S: Step Size      E: Decay Factor   I: Deposit Amt", Left)
            .add_single("K: Diff Kernel    ;: Diff Sigma     L: Attractor Str", Left)
            .add_single("W: Wind Dir       U: Terrain Type   Y: Terrain Str", Left)
            .add_single(",: Mouse Mode", Left)
            .add_empty()
            .add_single("Press any key to close this help", Left)
            .build_overlay()
    }

    /// Calculates center position for the overlay.
    pub fn calculate_position(term_width: usize, term_height: usize) -> (usize, usize) {
        let x = (term_width.saturating_sub(Self::WIDTH)) / 2;
        let y = (term_height.saturating_sub(30)) / 2;
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
            format!("{:.1}°", current.turn_angle),
            format!("{:.1}°", defaults.turn_angle),
            (current.turn_angle - defaults.turn_angle).abs() > 0.01,
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
            "Max Bright",
            format!("{:.1}x", current.max_brightness),
            format!("{:.1}x", defaults.max_brightness),
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
            builder = builder.add_empty();
            for (i, config) in configs.iter().enumerate().take(9) {
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

            if configs.len() > 9 {
                builder = builder.add_single(format!("... and {} more", configs.len() - 9), Left);
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
    #[allow(dead_code)]
    #[allow(clippy::too_many_arguments)]
    /// Builds the status bar string displayed at the bottom of the screen.
    ///
    /// Uses `▸` as a segment separator for a clean powerline-inspired look.
    /// Segments are added in priority order; lower-priority segments are omitted
    /// when the terminal is too narrow to fit them.
    ///
    /// Layout (left to right):
    /// ```text
    ///   PRESET  ▸  1.0×  ▸  PALETTE  ▸  50k  [Z Y]  ⏸ PAUSED  ·  ? help
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
        const SEP: &str = "  ▸  ";
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
            left.push_str("▸  ");
            // Color swatch: two block chars tinted with the palette accent
            if let Some(accent_color) = accent {
                let swatch_start = left.chars().count();
                left.push_str("▨ ");
                color_overrides.push((swatch_start, accent_color));
                color_overrides.push((swatch_start + 1, accent_color));
            }
            left.push_str(&format!("{}  ", palette_text));
        }

        // Add population if space permits
        if let Some(pop) = population {
            if width >= 68 {
                left.push_str(&format!("▸  {}k  ", pop / 1000));
            }
        }

        // Add diffusion kernel if space permits
        if let Some(kernel) = diffusion_kernel {
            if width >= 88 {
                left.push_str(&format!("▸  {}  ", kernel));
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
                left.push_str(&format!("▸  {}  ", d));
            }
        }

        // Color all ▸ separator characters in the left segment with muted color
        let separator_positions: Vec<usize> = left
            .chars()
            .enumerate()
            .filter_map(|(i, c)| if c == '▸' { Some(i) } else { None })
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

    #[allow(dead_code)]
    /// Calculates the X position for the status line (left-aligned or centered).
    pub fn status_line_x(status_line: &str, width: usize) -> usize {
        if status_line.len() < width {
            2
        } else {
            width.saturating_sub(status_line.len() + 2)
        }
    }

    #[allow(dead_code)]
    /// Calculates the X position for the paused indicator.
    pub fn paused_overlay_x(_width: usize) -> usize {
        let paused_text = "[ PAUSED ]";
        _width.saturating_sub(paused_text.len() + 2)
    }

    #[allow(dead_code)]
    /// Appends attractor help information to the help window.
    pub fn build_help_with_attractors(base_help: &[&str], attractors: &[Attractor]) -> Vec<String> {
        let mut lines: Vec<String> = base_help.iter().map(|s| s.to_string()).collect();

        if !attractors.is_empty() {
            // ATTRACTOR config: WIDTH=42, padding=1 → CONTENT_WIDTH=38
            let mut builder = PanelBuilder::new(38, None)
                .with_padding(Padding::new(0, 0, 1, 1))
                .with_title("ATTRACTORS");

            for (i, attractor) in attractors.iter().enumerate() {
                let kind = if attractor.strength > 0.0 {
                    "attract"
                } else {
                    "repel"
                };
                let strength = attractor.strength.abs();
                builder = builder.add_single(
                    format!(
                        "{:2}: ({:>4},{:>4}) {:^7} s: {:>4.1}x",
                        i + 1,
                        attractor.x as i32,
                        attractor.y as i32,
                        kind,
                        strength,
                    ),
                    TextAlignment::Left,
                );
            }

            lines.push(String::new());
            lines.extend(builder.build());
        }

        lines
    }

    #[allow(dead_code)]
    /// Appends obstacle help information to the help window.
    pub fn build_help_with_obstacles(base_help: &[&str], obstacles: &[Obstacle]) -> Vec<String> {
        let mut lines: Vec<String> = base_help.iter().map(|s| s.to_string()).collect();

        if !obstacles.is_empty() {
            // OBSTACLE config: WIDTH=42, padding=1 → CONTENT_WIDTH=38
            let mut builder = PanelBuilder::new(38, None)
                .with_padding(Padding::new(0, 0, 1, 1))
                .with_title("OBSTACLES");

            for (i, obstacle) in obstacles.iter().enumerate() {
                let line = match obstacle {
                    Obstacle::Circle { x, y, radius } => format!(
                        "{:2}: circle ({:>4},{:>4}) r: {:>4.1}px",
                        i + 1,
                        *x as i32,
                        *y as i32,
                        radius,
                    ),
                    Obstacle::Rect {
                        x,
                        y,
                        width,
                        height,
                    } => format!(
                        "{:2}: rect  ({:>4},{:>4}) {:>4.1}x{:>4.1}px",
                        i + 1,
                        *x as i32,
                        *y as i32,
                        width,
                        height,
                    ),
                    Obstacle::Image {
                        path,
                        x: _,
                        y: _,
                        width,
                        height,
                        invert: _,
                        threshold: _,
                    } => {
                        let filename = std::path::Path::new(path)
                            .file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or(path);
                        format!(
                            "{:2}: image {:>15} {:>3}x{:>3}px",
                            i + 1,
                            &filename[..filename.len().min(15)],
                            width,
                            height,
                        )
                    }
                };
                builder = builder.add_single(line, TextAlignment::Left);
            }

            lines.push(String::new());
            lines.extend(builder.build());
        }

        lines
    }

    #[allow(dead_code)]
    /// Appends mouse attractor help information to the help window.
    pub fn build_help_with_mouse_attractors(
        base_help: &[&str],
        mouse_attractors: &[MouseAttractor],
        _sim_width: usize,
        _sim_height: usize,
    ) -> Vec<String> {
        let mut lines: Vec<String> = base_help.iter().map(|s| s.to_string()).collect();

        if !mouse_attractors.is_empty() {
            // MOUSE_ATTRACTOR config: WIDTH=46, padding=1 → CONTENT_WIDTH=42
            let mut builder = PanelBuilder::new(42, None)
                .with_padding(Padding::new(0, 0, 1, 1))
                .with_title("MOUSE ATTRACTORS");

            for (i, ma) in mouse_attractors.iter().enumerate() {
                let kind = if ma.strength > 0.0 {
                    "attract"
                } else {
                    "repel"
                };
                let remaining = ma.timeout_seconds - ma.created_at.elapsed().as_secs_f32();
                let remaining_str = if remaining > 0.0 {
                    format!("{:.1}s", remaining)
                } else {
                    "expired".to_string()
                };
                builder = builder.add_single(
                    format!(
                        "{:2}: ({:>4},{:>4}) {:^7} s: {:>4.1}x {:>7}",
                        i + 1,
                        ma.x as i32,
                        ma.y as i32,
                        kind,
                        ma.strength.abs(),
                        remaining_str,
                    ),
                    TextAlignment::Left,
                );
            }

            lines.push(String::new());
            lines.extend(builder.build());
        }

        lines
    }

    #[cfg(test)]
    #[allow(dead_code)]
    pub fn check_attractor_section_lengths(lines: &[String], base_help_len: usize) -> bool {
        if lines.len() <= base_help_len {
            return true;
        }
        let attractor_section_start = base_help_len + 1; // Skip empty line after base help
        let attractor_lines = &lines[attractor_section_start..];
        if attractor_lines.is_empty() {
            return true;
        }
        let target_len = attractor_lines[0].chars().count();
        // Skip potential title/border differences if needed, but PanelBuilder should be consistent
        attractor_lines
            .iter()
            .all(|line| line.chars().count() == target_len)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::simulation::config::Attractor;

    #[test]
    fn test_attractor_overlay_no_attractors() {
        let attractors: Vec<Attractor> = vec![];
        let base_help = [
            "╭─ tslime controls ──────────────────────╮",
            "│ h: Toggle help                         │",
            "╰────────────────────────────────────────╯",
        ];
        let lines = OverlayRenderer::build_help_with_attractors(&base_help, &attractors);
        assert_eq!(lines, base_help);
    }

    #[test]
    fn test_attractor_overlay_single_attractor() {
        let attractors = vec![Attractor::new(200.0, 200.0, 1.0)];
        let base_help = [
            "╭─ tslime controls ──────────────────────╮",
            "│ h: Toggle help                         │",
            "╰────────────────────────────────────────╯",
        ];
        let lines = OverlayRenderer::build_help_with_attractors(&base_help, &attractors);
        assert!(
            lines.len() > base_help.len(),
            "Should add attractor section"
        );
        assert!(
            OverlayRenderer::check_attractor_section_lengths(&lines, base_help.len()),
            "Single attractor overlay should have consistent line lengths"
        );
    }

    #[test]
    fn test_attractor_overlay_max_strength() {
        let attractors = vec![Attractor::new(100.0, 100.0, 10.0)];
        let base_help = [
            "╭─ tslime controls ──────────────────────╮",
            "│ h: Toggle help                         │",
            "╰────────────────────────────────────────╯",
        ];
        let lines = OverlayRenderer::build_help_with_attractors(&base_help, &attractors);
        assert!(
            lines.len() > base_help.len(),
            "Should add attractor section"
        );
        assert!(
            OverlayRenderer::check_attractor_section_lengths(&lines, base_help.len()),
            "Max strength attractor should still have consistent line lengths"
        );
    }

    #[test]
    fn test_attractor_overlay_negative_coordinates() {
        let attractors = vec![Attractor::new(-50.0, -100.0, 1.0)];
        let base_help = [
            "╭─ tslime controls ──────────────────────╮",
            "│ h: Toggle help                         │",
            "╰────────────────────────────────────────╯",
        ];
        let lines = OverlayRenderer::build_help_with_attractors(&base_help, &attractors);
        assert!(
            lines.len() > base_help.len(),
            "Should add attractor section"
        );
        assert!(
            OverlayRenderer::check_attractor_section_lengths(&lines, base_help.len()),
            "Negative coordinates should still have consistent line lengths"
        );
    }

    #[test]
    fn test_attractor_overlay_multiple_attractors() {
        let attractors = vec![
            Attractor::new(200.0, 200.0, 1.0),
            Attractor::new(100.0, 100.0, -0.5),
            Attractor::new(300.0, 150.0, 2.0),
        ];
        let base_help = [
            "╭─ tslime controls ──────────────────────╮",
            "│ h: Toggle help                         │",
            "╰────────────────────────────────────────╯",
        ];
        let lines = OverlayRenderer::build_help_with_attractors(&base_help, &attractors);
        assert!(
            lines.len() > base_help.len(),
            "Should add attractor section"
        );
        assert!(
            OverlayRenderer::check_attractor_section_lengths(&lines, base_help.len()),
            "Multiple attractors should have consistent line lengths"
        );
    }

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
        assert_eq!(y, 35);
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
        assert_eq!(OverlayRenderer::paused_overlay_x(80), 68);
    }

    #[test]
    fn test_build_help_with_obstacles() {
        let base_help = ["base"];
        let obstacles = vec![
            Obstacle::Circle {
                x: 1.0,
                y: 2.0,
                radius: 3.0,
            },
            Obstacle::Rect {
                x: 1.0,
                y: 2.0,
                width: 3.0,
                height: 4.0,
            },
            Obstacle::Image {
                path: "test.png".to_string(),
                x: 0.0,
                y: 0.0,
                width: 10,
                height: 10,
                invert: false,
                threshold: 0.5,
            },
        ];
        let lines = OverlayRenderer::build_help_with_obstacles(&base_help, &obstacles);
        assert!(lines.len() > 1);
        assert!(lines.iter().any(|l| l.contains("circle")));
        assert!(lines.iter().any(|l| l.contains("rect")));
        assert!(lines.iter().any(|l| l.contains("image")));
    }

    #[test]
    fn test_build_help_with_mouse_attractors() {
        let base_help = ["base"];
        let mas = vec![MouseAttractor::new(10.0, 10.0, 1.0, 3.0)];
        let lines = OverlayRenderer::build_help_with_mouse_attractors(&base_help, &mas, 100, 100);
        assert!(lines.len() > 1);
        assert!(lines.iter().any(|l| l.contains("attract")));
    }
}

const SPARKLINE_CHARS: [char; 8] = [' ', '▂', '▃', '▄', '▅', '▆', '▇', '█'];

fn build_sparkline(history: &std::collections::VecDeque<f32>, min: f32, max: f32) -> String {
    let chars = SPARKLINE_CHARS;
    let mut sparkline = String::with_capacity(20);

    // Fill with empty if history is small
    for _ in 0..(20usize.saturating_sub(history.len())) {
        sparkline.push(' ');
    }

    for &val in history {
        let normalized = if max > min {
            ((val - min) / (max - min)).clamp(0.0, 0.999)
        } else {
            0.0
        };
        let idx = (normalized * chars.len() as f32) as usize;
        sparkline.push(chars[idx]);
    }

    format!("{:<20}", sparkline)
}

/// Generates per-cell color overrides for the stats overlay rich rendering.
///
/// Colors:
/// - FPS number: green (≥55 fps), amber (≥25 fps), red (< 25 fps)
/// - Sparkline bars: gradient from muted gray → `accent` based on bar height
fn generate_stats_rich_lines(
    lines: &[String],
    fps: f32,
    accent: RgbColor,
    panel_style: &PanelStyle,
) -> Vec<Vec<RichCell>> {
    let muted = panel_style.muted;
    let fps_color = if fps >= 55.0 {
        panel_style.accent_fps_good
    } else if fps >= 25.0 {
        panel_style.accent_fps_warn
    } else {
        panel_style.accent_error
    };

    lines
        .iter()
        .map(|line| {
            let chars: Vec<char> = line.chars().collect();
            let n = chars.len();
            // Content starts at col 3 (border=1, padding.left=2)
            let content_start = 3.min(n);
            let content_end = n.saturating_sub(3);

            // Detect sparkline row: all non-space content chars are sparkline chars
            let is_sparkline = content_start < content_end
                && chars[content_start..content_end]
                    .iter()
                    .all(|&c| matches!(c, ' ' | '▂' | '▃' | '▄' | '▅' | '▆' | '▇' | '█'));

            // Detect FPS row: content starts with "FPS:"
            let is_fps_row =
                content_start < n && chars[content_start..].starts_with(&['F', 'P', 'S', ':']);

            if is_sparkline {
                chars
                    .iter()
                    .enumerate()
                    .map(|(i, &c)| {
                        let fg = if i >= content_start && i < content_end {
                            let spark_idx = SPARKLINE_CHARS.iter().position(|&sc| sc == c);
                            if let Some(idx) = spark_idx {
                                if idx == 0 {
                                    None
                                } else {
                                    let t = idx as f32 / (SPARKLINE_CHARS.len() - 1) as f32;
                                    let r = (muted.r as f32
                                        + (accent.r as f32 - muted.r as f32) * t)
                                        as u8;
                                    let g = (muted.g as f32
                                        + (accent.g as f32 - muted.g as f32) * t)
                                        as u8;
                                    let b = (muted.b as f32
                                        + (accent.b as f32 - muted.b as f32) * t)
                                        as u8;
                                    Some(RgbColor { r, g, b })
                                }
                            } else {
                                None
                            }
                        } else {
                            None
                        };
                        (c, fg, None)
                    })
                    .collect()
            } else if is_fps_row {
                // Format: "FPS: {:>11.0} ({:>4.0})" — fps value occupies cols 5..16
                chars
                    .iter()
                    .enumerate()
                    .map(|(i, &c)| {
                        let col = i.saturating_sub(content_start);
                        let fg = if (5..16).contains(&col) && (c.is_ascii_digit() || c == '.') {
                            Some(fps_color)
                        } else {
                            None
                        };
                        (c, fg, None)
                    })
                    .collect()
            } else {
                chars.iter().map(|&c| (c, None, None)).collect()
            }
        })
        .collect()
}

/// Overlay showing real-time statistics.
pub struct StatsOverlay;

impl StatsOverlay {
    /// Total rendered width of the stats window.
    pub const WIDTH: usize = 32;
    /// Content width (inner drawable area).
    const CONTENT_WIDTH: usize = 26; // 32 - 2(border) - 2*2(padding)

    #[allow(clippy::too_many_arguments)]
    /// Builds the stats overlay content.
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
        _term_width: usize,
        fps_history: &std::collections::VecDeque<f32>,
        entropy_history: &std::collections::VecDeque<f32>,
        density_history: &std::collections::VecDeque<f32>,
        accent: RgbColor,
        panel_style: &PanelStyle,
    ) -> RenderedOverlay {
        use TextAlignment::Left;

        let trail_percent = if trail_capacity > 0.0 {
            (trail_sum / trail_capacity * 100.0).min(99.9)
        } else {
            0.0
        };

        let elapsed_str = format_elapsed_time(elapsed_seconds);
        let grid_str = format!("{}x{}", grid_width, grid_height);

        let fps_spark = build_sparkline(fps_history, 0.0, 60.0);
        let entropy_spark = build_sparkline(entropy_history, 0.0, 8.0);
        let density_spark = build_sparkline(density_history, 0.0, 1.0);

        let mut overlay = PanelBuilder::new(Self::CONTENT_WIDTH, None)
            .with_padding(Padding::new(2, 0, 2, 2))
            .with_title("STATS")
            .with_title_box()
            // Simulation stats
            .add_single(format!("Agents:   {:>15}", agent_count), Left)
            .add_single(format!("Trail:    {:>14.1}%", trail_percent), Left)
            .add_single(format!("{:<26}", density_spark), Left)
            .add_single(format!("Trail Max: {:>13.2}x", trail_max), Left)
            .add_single(format!("Entropy:   {:>15.2}", entropy), Left)
            .add_single(format!("{:<26}", entropy_spark), Left)
            .add_single(format!("FPS: {:>11.0} ({:>4.0})", fps, avg_fps), Left)
            .add_single(format!("{:<26}", fps_spark), Left)
            .add_single(format!("Frames:    {:>15}", frame_count), Left)
            .add_single(format!("Time:      {:>15}", elapsed_str), Left)
            .add_empty()
            // Section separator
            .add_separator()
            // System stats
            .add_single(format!("Grid:     {:>15}", grid_str), Left)
            .add_single(format!("Attractor: {:>13}", attractor_count), Left)
            .add_single(format!("Obstacle:  {:>13}", obstacle_count), Left)
            .add_single(format!("Species:   {:>14}", species_count), Left)
            .add_single(format!("Memory:    {:>11.1} MB", memory_mb), Left)
            .add_single(format!("CPU:       {:>14.0}%", cpu_percent), Left)
            .build_overlay();

        overlay.rich_lines = Some(generate_stats_rich_lines(
            &overlay.lines,
            fps,
            accent,
            panel_style,
        ));
        overlay
    }

    /// Calculates centered position for the stats overlay.
    pub fn calculate_position(term_width: usize, term_height: usize) -> (usize, usize) {
        let x = (term_width.saturating_sub(Self::WIDTH)) / 2;
        let y = (term_height.saturating_sub(24)) / 2;
        (x, y)
    }

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
}

/// Overlay showing general simulation info.
pub struct InfoOverlay;

impl InfoOverlay {
    /// Total rendered width of the info window.
    pub const WIDTH: usize = 28;
    /// Content width (inner drawable area).
    const CONTENT_WIDTH: usize = 22; // 28 - 2(border) - 2*2(padding)

    #[allow(clippy::too_many_arguments)]
    /// Builds the info overlay content.
    pub fn build_overlay(
        sim_width: usize,
        sim_height: usize,
        term_width: usize,
        term_height: usize,
        init_mode: &str,
        color_mode: &str,
        charset: &str,
        simd_enabled: bool,
        food_source: &Option<String>,
        warmup_frames: usize,
        _warmup_brightness: f32,
        _warmup_decay: f32,
        auto_reset: bool,
        _auto_reset_threshold: f32,
        _auto_reset_duration: usize,
    ) -> RenderedOverlay {
        use TextAlignment::Left;

        let resolution_str = format!("{}x{}", sim_width, sim_height);
        let term_str = format!("{}x{}", term_width, term_height);
        let simd_str = if simd_enabled { "On" } else { "Off" };

        let mut builder = PanelBuilder::new(Self::CONTENT_WIDTH, None)
            .with_padding(Padding::new(2, 0, 2, 2))
            .with_title("INFO")
            .with_title_box()
            .add_single(format!("Res:       {:>13}", resolution_str), Left)
            .add_single(format!("Term:      {:>13}", term_str), Left)
            .add_single(format!("Init:      {:>13}", init_mode), Left)
            .add_single(format!("Color:     {:>13}", color_mode), Left)
            .add_single(format!("Char:      {:>13}", charset), Left)
            .add_single(format!("SIMD:      {:>13}", simd_str), Left);

        if let Some(food) = food_source {
            let food_name = std::path::Path::new(food)
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or(food);
            let truncated = if food_name.len() > 12 {
                &food_name[..12]
            } else {
                food_name
            };
            builder = builder.add_single(format!("Food:      {:>13}", truncated), Left);
        }

        if warmup_frames > 0 {
            builder = builder.add_single(format!("Warm:      {:>6} frames", warmup_frames), Left);
        }

        if auto_reset {
            builder = builder.add_single(format!("Auto:      {:>13}", "On"), Left);
        }

        builder.build_overlay()
    }

    /// Calculates centered position for the info overlay.
    pub fn calculate_position(term_width: usize, term_height: usize) -> (usize, usize) {
        let x = (term_width.saturating_sub(Self::WIDTH)) / 2;
        let y = (term_height.saturating_sub(17)) / 2;
        (x, y)
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
mod stats_tests {
    use super::*;
    use crate::render::theme::GRUVBOX_DARK;

    #[test]
    fn test_stats_overlay_format() {
        let history = std::collections::VecDeque::from(vec![0.5f32; 20]);
        let lines = StatsOverlay::build_overlay(
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
            80,
            &history,
            &history,
            &history,
            RgbColor {
                r: 57,
                g: 211,
                b: 83,
            },
            &GRUVBOX_DARK,
        );

        assert!(!lines.lines.is_empty());
        // Solid-block borders
        assert!(
            lines.lines[0].starts_with('█'),
            "Top border should start with solid block █, got: {}",
            lines.lines[0]
        );
        assert!(
            lines.lines.last().unwrap().starts_with('█'),
            "Bottom border should start with solid block █"
        );

        // All lines should be exactly WIDTH chars
        for (i, line) in lines.lines.iter().enumerate() {
            assert_eq!(
                line.chars().count(),
                StatsOverlay::WIDTH,
                "Line {} has wrong width: '{}' (expected {}, got {})",
                i,
                line,
                StatsOverlay::WIDTH,
                line.chars().count()
            );
        }
    }

    #[test]
    fn test_stats_overlay_position() {
        let (x, y) = StatsOverlay::calculate_position(80, 40);
        assert_eq!(x, 24);
        assert_eq!(y, 8);
        let (x2, y2) = StatsOverlay::calculate_position(120, 50);
        assert_eq!(x2, 44);
        assert_eq!(y2, 13);
    }

    #[test]
    fn test_stats_overlay_with_zero_values() {
        let history = std::collections::VecDeque::new();
        let lines = StatsOverlay::build_overlay(
            0,
            0.0,
            1000000.0,
            0.0,
            0.0,
            0.0,
            0.0,
            0,
            0.0,
            400,
            400,
            0,
            0,
            0,
            0.0,
            0.0,
            80,
            &history,
            &history,
            &history,
            RgbColor {
                r: 57,
                g: 211,
                b: 83,
            },
            &GRUVBOX_DARK,
        );

        assert!(!lines.lines.is_empty());
        assert!(lines.lines.iter().any(|l| l.contains("0.0%")));
    }

    #[test]
    fn test_entropy_calculation() {
        let uniform = vec![1.0; 40000];
        let entropy_uniform = StatsOverlay::calculate_entropy(&uniform, 100);
        assert!(
            entropy_uniform < 2.0,
            "uniform should have low entropy, got {}",
            entropy_uniform
        );

        let varied: Vec<f32> = (0..40000).map(|i| i as f32 / 400.0).collect();
        let entropy_varied = StatsOverlay::calculate_entropy(&varied, 100);
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
        let entropy = StatsOverlay::calculate_entropy(&empty, 10);
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
mod info_tests {
    use super::*;

    #[test]
    fn test_info_overlay_format() {
        let lines = InfoOverlay::build_overlay(
            400,
            400,
            80,
            24,
            "Random",
            "TrueColor",
            "HalfBlock",
            false,
            &None,
            0,
            1.0,
            0.85,
            false,
            0.5,
            0,
        );

        assert!(!lines.lines.is_empty());
        // Solid-block borders
        assert!(
            lines.lines[0].starts_with('█'),
            "Top border should start with solid block █, got: {}",
            lines.lines[0]
        );
        assert!(
            lines.lines.last().unwrap().starts_with('█'),
            "Bottom border should start with solid block █"
        );

        // All lines should be exactly WIDTH chars
        for (i, line) in lines.lines.iter().enumerate() {
            assert_eq!(
                line.chars().count(),
                InfoOverlay::WIDTH,
                "Line {} has wrong width: '{}' (expected {} chars, got {})",
                i,
                line,
                InfoOverlay::WIDTH,
                line.chars().count()
            );
        }
    }

    #[test]
    fn test_info_overlay_with_optional_fields() {
        let lines = InfoOverlay::build_overlay(
            800,
            600,
            120,
            40,
            "Central",
            "EightBit",
            "ASCII",
            true,
            &Some("food.png".to_string()),
            100,
            1.5,
            0.9,
            true,
            0.3,
            300,
        );

        assert!(!lines.lines.is_empty());

        // All lines should be exactly WIDTH chars
        for (i, line) in lines.lines.iter().enumerate() {
            assert_eq!(
                line.chars().count(),
                InfoOverlay::WIDTH,
                "Line {} has wrong width: '{}' (expected {} chars, got {})",
                i,
                line,
                InfoOverlay::WIDTH,
                line.chars().count()
            );
        }

        // Should contain optional fields
        assert!(lines.lines.iter().any(|l| l.contains("Food")));
        assert!(lines.lines.iter().any(|l| l.contains("Warm")));
        assert!(lines.lines.iter().any(|l| l.contains("Auto")));
    }

    #[test]
    fn test_info_overlay_position() {
        let (x, y) = InfoOverlay::calculate_position(80, 40);
        assert_eq!(x, 26);
        assert_eq!(y, 11);
        let (x2, y2) = InfoOverlay::calculate_position(120, 50);
        assert_eq!(x2, 46);
        assert_eq!(y2, 16);
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
        let hints_lines = KeyboardHintsOverlay::build_overlay();

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
        }];

        let lines = ConfigBrowserOverlay::build_overlay(&configs, 0);
        assert!(lines.lines.iter().any(|l| l.contains("Test Config")));
        assert!(lines.lines.iter().any(|l| l.contains("10k agents")));
    }
}
