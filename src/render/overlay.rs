use crate::cli::Palette;
use crate::render::dither::DitherMode;
use crate::simulation::config::Attractor;
use crate::simulation::config::MouseAttractor;
use crate::simulation::config::Obstacle;
use crate::simulation::config::Preset;
use crate::terminal::control::{palette_name, preset_name};

/// Builder for creating box-drawn overlay windows with validated dimensions.
///
/// This ensures all lines fit within borders and padding, preventing width overflow.
pub struct WindowBuilder {
    width: usize,       // Total width including borders
    padding: usize,     // Padding on each side (left/right)
    inner_width: usize, // Calculated: width - 2*border - 2*padding
}

impl WindowBuilder {
    /// Creates a new WindowBuilder with the specified total width and padding.
    ///
    /// # Arguments
    /// * `width` - Total width including borders (e.g., 42 for a 42-char wide window)
    /// * `padding` - Number of spaces padding on each side (typically 1)
    ///
    /// # Panics
    /// Panics if width is too small for borders and padding (minimum: 2*border + 2*padding + 1)
    pub fn new(width: usize, padding: usize) -> Self {
        const BORDER_WIDTH: usize = 1;
        let min_width = 2 * BORDER_WIDTH + 2 * padding + 1;

        if width < min_width {
            panic!(
                "Window width {} is too small for borders and padding (minimum: {})",
                width, min_width
            );
        }

        let inner_width = width - 2 * BORDER_WIDTH - 2 * padding;

        Self {
            width,
            padding,
            inner_width,
        }
    }

    /// Returns the inner content width (excluding borders and padding)
    pub fn inner_width(&self) -> usize {
        self.inner_width
    }

    /// Builds a window with content lines, validating all fit within inner width.
    ///
    /// # Arguments
    /// * `title` - Optional title for top border (e.g., "HELP", "STATS")
    /// * `content` - Content lines (must each be <= inner_width chars)
    ///
    /// # Returns
    /// `Ok(Vec<String>)` with the complete window, or `Err(String)` if validation fails
    pub fn build(&self, title: Option<&str>, content: &[String]) -> Result<Vec<String>, String> {
        // Validate content lines
        for (i, line) in content.iter().enumerate() {
            let line_len = line.chars().count();
            if line_len > self.inner_width {
                return Err(format!(
                    "Content line {} is too long ({} chars, max {}): '{}'",
                    i, line_len, self.inner_width, line
                ));
            }
        }

        let mut lines = Vec::with_capacity(content.len() + 2);

        // Top border
        lines.push(self.build_top_border(title));

        // Content lines
        for line in content {
            lines.push(self.build_content_line(line));
        }

        // Bottom border
        lines.push(self.build_bottom_border());

        Ok(lines)
    }

    /// Builds the top border, optionally with a title.
    fn build_top_border(&self, title: Option<&str>) -> String {
        if let Some(title) = title {
            let title_with_spaces = format!(" {} ", title);
            let title_len = title_with_spaces.chars().count();
            let remaining = self.width.saturating_sub(2 + title_len); // -2 for corners
            let left_dashes = 1; // At least one dash after ╭
            let right_dashes = remaining.saturating_sub(left_dashes);

            format!(
                "╭{}{}{}╮",
                "─".repeat(left_dashes),
                title_with_spaces,
                "─".repeat(right_dashes)
            )
        } else {
            format!("╭{}╮", "─".repeat(self.width - 2))
        }
    }

    /// Builds a content line with borders and padding.
    fn build_content_line(&self, content: &str) -> String {
        let content_len = content.chars().count();
        let padding_right = self.inner_width.saturating_sub(content_len);

        format!(
            "│{}{}{}{}│",
            " ".repeat(self.padding),
            content,
            " ".repeat(padding_right),
            " ".repeat(self.padding)
        )
    }

    /// Builds the bottom border.
    fn build_bottom_border(&self) -> String {
        format!("╰{}╯", "─".repeat(self.width - 2))
    }

    /// Builds a separator line (for dividing sections within a window).
    pub fn build_separator(&self) -> String {
        format!("├{}┤", "─".repeat(self.width - 2))
    }
}

// --- END WindowBuilder ---

/// Overlay showing keyboard shortcuts.
pub struct KeyboardHintsOverlay;

impl KeyboardHintsOverlay {
    /// Width of the keyboard hints window.
    pub const WIDTH: usize = 60;

    /// Builds the keyboard hints overlay content.
    pub fn build_overlay() -> Vec<String> {
        let builder = WindowBuilder::new(Self::WIDTH, 2);
        let content = vec![
            "".to_string(),
            "SIMULATION                VISUALS".to_string(),
            "p, Space : Pause          c, Shift+C : Palette".to_string(),
            "r        : Restart        o          : Palette Shift".to_string(),
            "q, Esc   : Quit           x          : Invert Palette".to_string(),
            "+, -     : Time Scale     z          : Reverse Palette".to_string(),
            "".to_string(),
            "PRESETS                   POST-PROCESSING".to_string(),
            "1-7      : Presets        d, m       : Dither Mode".to_string(),
            "8        : Randomize      [, ]       : Dither Inten.".to_string(),
            "0        : Defaults       b          : Auto Normalize".to_string(),
            "                          v          : Motion Blur".to_string(),
            "SYSTEM                    n, Shift+N : Max Brightness".to_string(),
            "h        : Controls       f          : Fast Mode".to_string(),
            "?, |     : Help/Info      g          : Save PNG".to_string(),
            "\\        : Stats          Ctrl+S     : Save Config".to_string(),
            "Tab      : Category       Ctrl+L     : Load Config".to_string(),
            "".to_string(),
            "DETAILED CONTROLS (Use Shift to decrease values)".to_string(),
            "A: Sensor Angle   J: Sensor Dist    T: Turn Angle".to_string(),
            "S: Step Size      E: Decay Factor   I: Deposit Amt".to_string(),
            "K: Diff Kernel    ;: Diff Sigma     L: Attractor Str".to_string(),
            "W: Wind Dir       U: Terrain Type   Y: Terrain Str".to_string(),
            ",: Mouse Mode".to_string(),
            "".to_string(),
            "Press any key to close this help".to_string(),
        ];

        builder
            .build(Some("KEYBOARD SHORTCUTS"), &content)
            .unwrap_or_default()
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
    /// Width of the comparison window.
    pub const WIDTH: usize = 62;

    /// Builds the comparison overlay showing modified parameters.
    pub fn build_overlay(
        current: &crate::terminal::control::RuntimeState,
        preset: Preset,
    ) -> Vec<String> {
        let defaults = crate::terminal::control::DefaultValues::from_preset(preset);
        let preset_name = preset_name(preset);
        let builder = WindowBuilder::new(Self::WIDTH, 2);

        let mut content = vec![
            "".to_string(),
            "Parameter        │ Current      │ Preset Default".to_string(),
            "──────────────────┼──────────────┼──────────────────────".to_string(),
        ];

        let mut add_row = |name: &str, cur: String, def: String, modif: bool| {
            let marker = if modif { "*" } else { " " };
            content.push(format!(
                "{} {:<16} │ {:<12} │ {:<18}",
                marker, name, cur, def
            ));
        };

        add_row(
            "Sensor Angle",
            format!("{:.1}°", current.sensor_angle),
            format!("{:.1}°", defaults.sensor_angle),
            (current.sensor_angle - defaults.sensor_angle).abs() > 0.01,
        );
        add_row(
            "Sensor Dist",
            format!("{:.1}px", current.sensor_distance),
            format!("{:.1}px", defaults.sensor_distance),
            (current.sensor_distance - defaults.sensor_distance).abs() > 0.01,
        );
        add_row(
            "Turn Angle",
            format!("{:.1}°", current.turn_angle),
            format!("{:.1}°", defaults.turn_angle),
            (current.turn_angle - defaults.turn_angle).abs() > 0.01,
        );
        add_row(
            "Step Size",
            format!("{:.1}px", current.step_size),
            format!("{:.1}px", defaults.step_size),
            (current.step_size - defaults.step_size).abs() > 0.01,
        );
        add_row(
            "Decay Factor",
            format!("{:.3}x", current.decay_factor),
            format!("{:.3}x", defaults.decay_factor),
            (current.decay_factor - defaults.decay_factor).abs() > 0.001,
        );
        add_row(
            "Deposit Amt",
            format!("{:.1}x", current.deposit_amount),
            format!("{:.1}x", defaults.deposit_amount),
            (current.deposit_amount - defaults.deposit_amount).abs() > 0.01,
        );
        add_row(
            "Diff Sigma",
            format!("{:.2}x", current.diffusion_sigma),
            format!("{:.2}x", defaults.diffusion_sigma),
            (current.diffusion_sigma - defaults.diffusion_sigma).abs() > 0.01,
        );
        add_row(
            "Max Bright",
            format!("{:.1}x", current.max_brightness),
            format!("{:.1}x", defaults.max_brightness),
            (current.max_brightness - defaults.max_brightness).abs() > 0.01,
        );

        content.push("".to_string());
        content.push("Press Enter to Apply Preset     Esc to Close".to_string());

        builder
            .build(
                Some(&format!("PRESET COMPARISON: {}", preset_name)),
                &content,
            )
            .unwrap_or_default()
    }

    /// Calculates center position for the comparison overlay.
    pub fn calculate_position(term_width: usize, term_height: usize) -> (usize, usize) {
        let x = (term_width.saturating_sub(Self::WIDTH)) / 2;
        let y = (term_height.saturating_sub(15)) / 2;
        (x, y)
    }
}

#[allow(dead_code)]
/// Overlay shown during warmup phase.
pub struct WarmupOverlay;

#[allow(dead_code)]
impl WarmupOverlay {
    /// Builds the warmup status message.
    pub fn build_overlay(frame_counter: usize, max_frames: usize) -> Vec<String> {
        // Create a pulsing effect using sine wave
        let progress = (frame_counter as f32 / 30.0 * std::f32::consts::PI)
            .sin()
            .abs();
        let opacity = (progress * 10.0) as usize;

        let dots = ".".repeat(opacity.min(3));
        let message = format!("Press any key to begin{}", dots);
        let frame_info = format!("Warmup: {}/{}", frame_counter, max_frames);

        vec![message, frame_info]
    }

    /// Calculates position for warmup overlay (bottom center).
    pub fn calculate_position(term_width: usize, term_height: usize) -> (usize, usize) {
        // Center horizontally, bottom third vertically
        let y = (term_height * 2) / 3;
        let x = term_width / 2;
        (x, y)
    }
}

/// Overlay for browsing saved configurations.
pub struct ConfigBrowserOverlay;

impl ConfigBrowserOverlay {
    /// Width of the browser window.
    pub const WIDTH: usize = 56;

    /// Builds the configuration list overlay.
    pub fn build_overlay(
        configs: &[crate::config_manager::SavedConfig],
        selected_index: usize,
    ) -> Vec<String> {
        let builder = WindowBuilder::new(Self::WIDTH, 2);
        let mut content = Vec::new();

        if configs.is_empty() {
            content.push("".to_string());
            content.push("No saved configurations".to_string());
            content.push("".to_string());
            content.push("Press Ctrl+S to save current settings".to_string());
            content.push("".to_string());
        } else {
            content.push("".to_string());
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
                content.push(line);
            }

            if configs.len() > 9 {
                content.push(format!("... and {} more", configs.len() - 9));
            }

            content.push("".to_string());
            content.push("↑/↓: Navigate  Enter: Load  Del: Delete".to_string());
        }

        content.push("Esc: Cancel".to_string());

        builder
            .build(Some("SAVED CONFIGURATIONS"), &content)
            .unwrap_or_default()
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
    /// Builds the save dialog overlay.
    pub fn build_overlay(name_input: &str) -> Vec<String> {
        let builder = WindowBuilder::new(38, 1);
        let content = vec![
            "".to_string(),
            format!("Name: {:<25}", name_input),
            "".to_string(),
            "Enter: Save    Esc: Cancel".to_string(),
        ];
        builder
            .build(Some("SAVE CONFIGURATION"), &content)
            .unwrap_or_default()
    }

    /// Calculates center position for the save dialog.
    pub fn calculate_position(term_width: usize, term_height: usize) -> (usize, usize) {
        let x = (term_width.saturating_sub(38)) / 2;
        let y = (term_height.saturating_sub(5)) / 2;
        (x, y)
    }
}

/// Utilities for rendering overlay elements (status line, help lists).
pub struct OverlayRenderer;

impl OverlayRenderer {
    #[allow(dead_code)]
    #[allow(clippy::too_many_arguments)]
    /// Builds the status bar string displayed at the bottom of the screen.
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
    ) -> String {
        let preset_text = preset_name(preset);
        let palette_text = palette_name(palette);
        let time_text = format!("{:.1}x", time_scale);

        let undo_redo_text = if can_undo || can_redo {
            format!(
                " [{}{}]",
                if can_undo { "Z" } else { " " },
                if can_redo { "Y" } else { " " }
            )
        } else {
            "".to_string()
        };

        let dither_text = match dither_mode {
            DitherMode::None => "".to_string(),
            DitherMode::Ordered { intensity, .. } => format!(" D:{:.1}x", intensity),
            DitherMode::ErrorDiffusion { .. } => " ED".to_string(),
            DitherMode::Hybrid { intensity, .. } => format!(" H:{:.1}x", intensity),
        };

        let paused_text = if _is_paused { " [PAUSED]" } else { "" };
        let help_text = if width >= 100 { " ? for help" } else { "" };

        // Build components with priority for truncation
        let mut status = format!("{} │ {}", preset_text, time_text);

        // Add palette if space permits
        if width >= 50 {
            status.push_str(&format!(" │ {}", palette_text));
        }

        // Add population if provided and space permits
        if let Some(pop) = population {
            if width >= 70 {
                let pop_k = pop / 1000;
                status.push_str(&format!(" │ {}k", pop_k));
            }
        }

        // Add diffusion kernel if provided and space permits
        if let Some(kernel) = diffusion_kernel {
            if width >= 90 {
                status.push_str(&format!(" │ {}", kernel));
            }
        }

        // Add dither if present
        if !dither_text.is_empty() && width >= 60 {
            status.push_str(&dither_text);
        }

        // Always add paused and help at the end
        status.push_str(&undo_redo_text);
        status.push_str(paused_text);
        status.push_str(help_text);

        status
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
            let builder = WindowBuilder::new(42, 1);
            let mut content = Vec::new();

            for (i, attractor) in attractors.iter().enumerate() {
                let kind = if attractor.strength > 0.0 {
                    "attract"
                } else {
                    "repel"
                };
                let strength = attractor.strength.abs();
                content.push(format!(
                    "{:2}: ({:>4},{:>4}) {:^7} s: {:>4.1}x",
                    i + 1,
                    attractor.x as i32,
                    attractor.y as i32,
                    kind,
                    strength,
                ));
            }

            lines.push(String::new());
            lines.extend(
                builder
                    .build(Some("ATTRACTORS"), &content)
                    .unwrap_or_default(),
            );
        }

        lines
    }

    #[allow(dead_code)]
    /// Appends obstacle help information to the help window.
    pub fn build_help_with_obstacles(base_help: &[&str], obstacles: &[Obstacle]) -> Vec<String> {
        let mut lines: Vec<String> = base_help.iter().map(|s| s.to_string()).collect();

        if !obstacles.is_empty() {
            let builder = WindowBuilder::new(42, 1);
            let mut content = Vec::new();

            for (i, obstacle) in obstacles.iter().enumerate() {
                match obstacle {
                    Obstacle::Circle { x, y, radius } => {
                        content.push(format!(
                            "{:2}: circle ({:>4},{:>4}) r: {:>4.1}px",
                            i + 1,
                            *x as i32,
                            *y as i32,
                            radius,
                        ));
                    }
                    Obstacle::Rect {
                        x,
                        y,
                        width,
                        height,
                    } => {
                        content.push(format!(
                            "{:2}: rect  ({:>4},{:>4}) {:>4.1}x{:>4.1}px",
                            i + 1,
                            *x as i32,
                            *y as i32,
                            width,
                            height,
                        ));
                    }
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
                        content.push(format!(
                            "{:2}: image {:>15} {:>3}x{:>3}px",
                            i + 1,
                            &filename[..filename.len().min(15)],
                            width,
                            height,
                        ));
                    }
                }
            }

            lines.push(String::new());
            lines.extend(
                builder
                    .build(Some("OBSTACLES"), &content)
                    .unwrap_or_default(),
            );
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
            let builder = WindowBuilder::new(46, 1);
            let mut content = Vec::new();

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
                content.push(format!(
                    "{:2}: ({:>4},{:>4}) {:^7} s: {:>4.1}x {:>7}",
                    i + 1,
                    ma.x as i32,
                    ma.y as i32,
                    kind,
                    ma.strength.abs(),
                    remaining_str,
                ));
            }

            lines.push(String::new());
            lines.extend(
                builder
                    .build(Some("MOUSE ATTRACTORS"), &content)
                    .unwrap_or_default(),
            );
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
        // Skip potential title/border differences if needed, but WindowBuilder should be consistent
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
    fn test_window_builder_content_too_long() {
        let builder = WindowBuilder::new(20, 1);
        let content = vec!["this line is definitely too long for the builder".to_string()];
        let result = builder.build(None, &content);
        assert!(result.is_err());
    }

    #[test]
    #[should_panic]
    fn test_window_builder_too_small() {
        let _ = WindowBuilder::new(2, 1);
    }

    #[test]
    fn test_window_builder_separator() {
        let builder = WindowBuilder::new(10, 1);
        let sep = builder.build_separator();
        assert!(sep.starts_with('├'));
        assert!(sep.ends_with('┤'));
        assert_eq!(sep.chars().count(), 10);
    }

    #[test]
    fn test_keyboard_hints_position() {
        let (x, y) = KeyboardHintsOverlay::calculate_position(100, 100);
        assert_eq!(x, 20);
        assert_eq!(y, 35);
    }

    #[test]
    fn test_warmup_overlay() {
        let lines = WarmupOverlay::build_overlay(10, 100);
        assert_eq!(lines.len(), 2);
        assert!(lines[1].contains("10/100"));
        let (x, y) = WarmupOverlay::calculate_position(100, 90);
        assert_eq!(x, 50);
        assert_eq!(y, 60);
    }

    #[test]
    fn test_config_browser_overlay_empty() {
        let lines = ConfigBrowserOverlay::build_overlay(&[], 0);
        assert!(lines.iter().any(|l| l.contains("No saved configurations")));
        let (x, y) = ConfigBrowserOverlay::calculate_position(100, 100);
        assert_eq!(x, 22);
    }

    #[test]
    fn test_config_save_overlay() {
        let lines = ConfigSaveOverlay::build_overlay("test");
        assert!(lines.iter().any(|l| l.contains("test")));
        let (x, y) = ConfigSaveOverlay::calculate_position(100, 100);
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

fn build_sparkline(history: &std::collections::VecDeque<f32>, min: f32, max: f32) -> String {
    let chars = [' ', '▂', '▃', '▄', '▅', '▆', '▇', '█'];
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

/// Overlay showing real-time statistics.
pub struct StatsOverlay;

impl StatsOverlay {
    /// Width of the stats window.
    pub const WIDTH: usize = 32;

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
    ) -> Vec<String> {
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

        let builder = WindowBuilder::new(Self::WIDTH, 2);
        let content = vec![
            format!("Agents:   {:>15}", agent_count),
            format!("Trail:    {:>14.1}%", trail_percent),
            format!("{:<26}", density_spark),
            format!("Trail Max: {:>13.2}x", trail_max),
            format!("Entropy:   {:>15.2}", entropy),
            format!("{:<26}", entropy_spark),
            format!("FPS: {:>11.0} ({:>4.0})", fps, avg_fps),
            format!("{:<26}", fps_spark),
            format!("Frames:    {:>15}", frame_count),
            format!("Time:      {:>15}", elapsed_str),
        ];

        let mut lines = builder
            .build(Some("STATS"), &content)
            .unwrap_or_else(|e| vec![format!("Error: {}", e)]);

        if lines.len() > 1 {
            lines.pop();
            lines.push(builder.build_separator());

            let system_content = vec![
                format!("Grid:     {:>15}", grid_str),
                format!("Attractor: {:>13}", attractor_count),
                format!("Obstacle:  {:>13}", obstacle_count),
                format!("Species:   {:>14}", species_count),
                format!("Memory:    {:>11.1} MB", memory_mb),
                format!("CPU:       {:>14.0}%", cpu_percent),
            ];

            for line in system_content {
                lines.push(builder.build_content_line(&line));
            }
            lines.push(builder.build_bottom_border());
        }

        lines
    }

    /// Calculates the X position for the stats overlay (top-right).
    pub fn calculate_x_position(term_width: usize) -> usize {
        if term_width > Self::WIDTH + 2 {
            term_width.saturating_sub(Self::WIDTH + 2)
        } else {
            1
        }
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
    /// Width of the info window.
    pub const WIDTH: usize = 28;

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
    ) -> Vec<String> {
        let resolution_str = format!("{}x{}", sim_width, sim_height);
        let term_str = format!("{}x{}", term_width, term_height);
        let simd_str = if simd_enabled { "On" } else { "Off" };

        let builder = WindowBuilder::new(Self::WIDTH, 1);
        let mut content = vec![
            format!("Res:       {:>13}", resolution_str),
            format!("Term:      {:>13}", term_str),
            format!("Init:      {:>13}", init_mode),
            format!("Color:     {:>13}", color_mode),
            format!("Char:      {:>13}", charset),
            format!("SIMD:      {:>13}", simd_str),
        ];

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
            content.push(format!("Food:      {:>13}", truncated));
        }

        if warmup_frames > 0 {
            content.push(format!("Warm:      {:>6} frames", warmup_frames));
        }

        if auto_reset {
            content.push(format!("Auto:      {:>13}", "On"));
        }

        builder.build(Some("INFO"), &content).unwrap_or_default()
    }

    /// Calculates X position for the info overlay.
    pub fn calculate_x_position(term_width: usize) -> usize {
        if term_width > Self::WIDTH + 2 {
            term_width.saturating_sub(Self::WIDTH + 2)
        } else {
            1
        }
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

    #[test]
    fn test_stats_overlay_format() {
        let history = std::collections::VecDeque::from(vec![0.5f32; 20]);
        let lines = StatsOverlay::build_overlay(
            50000, 1234567.0, 8000000.0, 8.5, 5.5, 30.0, 28.5, 1234, 125.5, 400, 400, 3, 1, 2,
            12.5, 85.0, 80, &history, &history, &history,
        );

        assert!(!lines.is_empty());
        assert!(lines[0].starts_with('╭'));
        assert!(lines.last().unwrap().starts_with('╰'));

        // All lines should be exactly WIDTH chars
        for (i, line) in lines.iter().enumerate() {
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
        assert_eq!(StatsOverlay::calculate_x_position(80), 46);
        assert_eq!(StatsOverlay::calculate_x_position(120), 86);
        assert_eq!(StatsOverlay::calculate_x_position(24), 1);
    }

    #[test]
    fn test_stats_overlay_with_zero_values() {
        let history = std::collections::VecDeque::new();
        let lines = StatsOverlay::build_overlay(
            0, 0.0, 1000000.0, 0.0, 0.0, 0.0, 0.0, 0, 0.0, 400, 400, 0, 0, 0, 0.0, 0.0, 80,
            &history, &history, &history,
        );

        assert!(!lines.is_empty());
        assert!(lines.iter().any(|l| l.contains("0.0%")));
    }

    #[test]
    fn test_entropy_calculation() {
        let uniform = vec![1.0; 40000];
        let entropy_uniform = StatsOverlay::calculate_entropy(&uniform, 100);
        eprintln!("uniform entropy: {}", entropy_uniform);
        assert!(
            entropy_uniform < 2.0,
            "uniform should have low entropy, got {}",
            entropy_uniform
        );

        let varied: Vec<f32> = (0..40000).map(|i| i as f32 / 400.0).collect();
        let entropy_varied = StatsOverlay::calculate_entropy(&varied, 100);
        eprintln!("varied entropy: {}", entropy_varied);
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

        assert!(!lines.is_empty());
        assert!(lines[0].starts_with('╭'));
        assert!(lines.last().unwrap().starts_with('╰'));

        // All lines should be exactly WIDTH chars
        for (i, line) in lines.iter().enumerate() {
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

        assert!(!lines.is_empty());

        // All lines should be exactly WIDTH chars
        for (i, line) in lines.iter().enumerate() {
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
        assert!(lines.iter().any(|l| l.contains("Food")));
        assert!(lines.iter().any(|l| l.contains("Warm")));
        assert!(lines.iter().any(|l| l.contains("Auto")));
    }

    #[test]
    fn test_info_overlay_position() {
        assert_eq!(InfoOverlay::calculate_x_position(80), 50);
        assert_eq!(InfoOverlay::calculate_x_position(120), 90);
        assert_eq!(InfoOverlay::calculate_x_position(28), 1);
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
        let status = OverlayRenderer::build_status_line(
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
        );
        // At 40 cols: should only have preset and time
        assert!(status.contains("Organic"));
        assert!(status.contains("1.0x"));
        // Should not have palette or population (too narrow)
        assert!(!status.contains("50k"));
    }

    #[test]
    fn test_status_line_medium_terminal_80_cols() {
        let status = OverlayRenderer::build_status_line(
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
        );
        // At 80 cols: should have preset, time, palette, and population
        assert!(status.contains("Network"));
        assert!(status.contains("2.5x"));
        assert!(status.contains("Heat"));
        assert!(status.contains("50k"));
        // Should not have diffusion kernel (needs 90+)
        assert!(!status.contains("Mean3x3"));
        // Should not have help text (needs 100+)
        assert!(!status.contains("?"));
    }

    #[test]
    fn test_status_line_wide_terminal_120_cols() {
        let status = OverlayRenderer::build_status_line(
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
        );
        // At 120 cols: should have everything including help
        assert!(status.contains("Exploratory"));
        assert!(status.contains("1.5x"));
        assert!(status.contains("Ocean"));
        assert!(status.contains("30k"));
        assert!(status.contains("Gaussian"));
        assert!(status.contains("? for help"));
    }

    #[test]
    fn test_status_line_paused() {
        let status = OverlayRenderer::build_status_line(
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
        );
        assert!(status.contains("[PAUSED]"));
    }

    #[test]
    fn test_status_line_with_dither() {
        let status = OverlayRenderer::build_status_line(
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
        );
        assert!(status.contains("D:0.5"));
    }

    #[test]
    fn test_status_line_without_optional_params() {
        let status = OverlayRenderer::build_status_line(
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
        );
        // Should still work without population or diffusion kernel
        assert!(status.contains("Organic"));
        assert!(status.contains("1.0x"));
    }

    #[test]
    fn test_keyboard_hints_overlay_format() {
        let hints_lines = KeyboardHintsOverlay::build_overlay();

        for line in &hints_lines {
            assert!(line.starts_with('│') || line.starts_with('╭') || line.starts_with('╰'));
            assert!(line.ends_with('│') || line.ends_with('╮') || line.ends_with('╯'));
        }

        // All lines should be KeyboardHintsOverlay::WIDTH chars wide
        for line in &hints_lines {
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
        let mut state = crate::terminal::control::RuntimeState::new(
            42,
            crate::simulation::config::InitMode::Random,
            Preset::Organic,
            0,
            crate::terminal::control::MouseInteractionMode::Disabled,
            0.0,
        );
        state.sensor_angle = 90.0; // Changed from default

        let lines = PresetComparisonOverlay::build_overlay(&state, Preset::Organic);
        assert!(!lines.is_empty());
        let content_lines = lines
            .iter()
            .filter(|l| l.contains("Sensor Angle"))
            .collect::<Vec<_>>();
        assert!(!content_lines.is_empty());
        // Should show modified marker *
        assert!(content_lines[0].contains('*'));
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
        }];

        let lines = ConfigBrowserOverlay::build_overlay(&configs, 0);
        assert!(lines.iter().any(|l| l.contains("Test Config")));
        assert!(lines.iter().any(|l| l.contains("10k agents")));
    }
}
