use crate::cli::Palette;
use crate::render::palette::{hsv_to_rgb, interpolate_gradient, rgb_to_hsv, HsvColor, RgbColor};

/// Number of colors in the palette gradient.
pub const PALETTE_COLOR_COUNT: usize = 11;

// ─── Rich cell types ──────────────────────────────────────────────────────────

/// A single terminal cell that carries per-cell color data.
///
/// `fg` / `bg` of `None` means "use the overlay's default color".
#[derive(Clone, Debug)]
pub struct RichCell {
    /// The character to display.
    pub ch: char,
    /// Foreground color (None = default overlay fg).
    pub fg: Option<RgbColor>,
    /// Background color (None = default overlay bg).
    pub bg: Option<RgbColor>,
}

/// A line of rich cells (per-cell color data).
pub type RichLine = Vec<RichCell>;

// ─── Helper builders ─────────────────────────────────────────────────────────

/// Convert a plain string to a `RichLine` with fixed fg/bg (for borders and labels).
fn rich_text(s: &str, fg: Option<RgbColor>, bg: Option<RgbColor>) -> RichLine {
    s.chars().map(|ch| RichCell { ch, fg, bg }).collect()
}

/// Build a gradient strip: each cell is '█' with bg = interpolated color.
fn build_gradient_strip(colors: &[RgbColor; PALETTE_COLOR_COUNT], width: usize) -> RichLine {
    let stops: Vec<crate::render::palette::GradientStop> = colors
        .iter()
        .enumerate()
        .map(|(i, &color)| crate::render::palette::GradientStop {
            position: i as f32 / (PALETTE_COLOR_COUNT - 1) as f32,
            color,
        })
        .collect();

    (0..width)
        .map(|i| {
            let t = i as f32 / (width - 1).max(1) as f32;
            let color = interpolate_gradient(&stops, t);
            RichCell {
                ch: '█',
                fg: Some(color),
                bg: Some(color),
            }
        })
        .collect()
}

/// Build a swatch row: 11 colored blocks, selected one wrapped with `[` `]`.
fn build_swatches(
    colors: &[RgbColor; PALETTE_COLOR_COUNT],
    selected: usize,
    width: usize,
) -> RichLine {
    let mut line: RichLine = Vec::new();

    // We render each swatch as 4 chars:  " ██ " or "[██]"
    // That's 11 * 4 = 44 chars base. Pad to `width`.
    for (i, &color) in colors.iter().enumerate() {
        if i == selected {
            line.push(RichCell {
                ch: '[',
                fg: None,
                bg: None,
            });
            line.push(RichCell {
                ch: '█',
                fg: Some(color),
                bg: Some(color),
            });
            line.push(RichCell {
                ch: '█',
                fg: Some(color),
                bg: Some(color),
            });
            line.push(RichCell {
                ch: ']',
                fg: None,
                bg: None,
            });
        } else {
            line.push(RichCell {
                ch: ' ',
                fg: None,
                bg: None,
            });
            line.push(RichCell {
                ch: '█',
                fg: Some(color),
                bg: Some(color),
            });
            line.push(RichCell {
                ch: '█',
                fg: Some(color),
                bg: Some(color),
            });
            line.push(RichCell {
                ch: ' ',
                fg: None,
                bg: None,
            });
        }
    }

    // Pad or truncate to exactly `width`
    line.truncate(width);
    while line.len() < width {
        line.push(RichCell {
            ch: ' ',
            fg: None,
            bg: None,
        });
    }
    line
}

/// Build index label row below swatches: " 1  2  3 … 11"
fn build_swatch_labels(width: usize) -> RichLine {
    let mut s = String::new();
    for i in 1..=PALETTE_COLOR_COUNT {
        if i < 10 {
            s.push_str(&format!(" {:1}  ", i));
        } else {
            s.push_str(&format!("{:2}  ", i));
        }
    }
    let mut line = rich_text(&s, None, None);
    line.truncate(width);
    while line.len() < width {
        line.push(RichCell {
            ch: ' ',
            fg: None,
            bg: None,
        });
    }
    line
}

/// Build a hue rainbow bar: cell i has bg = HSV(hue=i/width*360, s=1, v=1).
fn build_hue_bar(width: usize) -> RichLine {
    (0..width)
        .map(|i| {
            let h = i as f32 / width as f32 * 360.0;
            let color = hsv_to_rgb(HsvColor { h, s: 1.0, v: 1.0 });
            RichCell {
                ch: '█',
                fg: Some(color),
                bg: Some(color),
            }
        })
        .collect()
}

/// Build a saturation bar: left=grey, right=full color at given hue/value.
fn build_sat_bar(width: usize, hue: f32, val: f32) -> RichLine {
    (0..width)
        .map(|i| {
            let s = i as f32 / (width - 1).max(1) as f32;
            let color = hsv_to_rgb(HsvColor { h: hue, s, v: val });
            RichCell {
                ch: '█',
                fg: Some(color),
                bg: Some(color),
            }
        })
        .collect()
}

/// Build a value bar: left=black, right=full brightness at given hue/saturation.
fn build_val_bar(width: usize, hue: f32, sat: f32) -> RichLine {
    (0..width)
        .map(|i| {
            let v = i as f32 / (width - 1).max(1) as f32;
            let color = hsv_to_rgb(HsvColor { h: hue, s: sat, v });
            RichCell {
                ch: '█',
                fg: Some(color),
                bg: Some(color),
            }
        })
        .collect()
}

/// Build a cursor indicator row: plain `^` at `cursor_frac` position.
fn build_cursor_row(width: usize, cursor_frac: f32) -> RichLine {
    let pos = ((cursor_frac * (width - 1) as f32).round() as usize).min(width - 1);
    (0..width)
        .map(|i| {
            let ch = if i == pos { '^' } else { ' ' };
            RichCell {
                ch,
                fg: None,
                bg: None,
            }
        })
        .collect()
}

/// Pad a RichLine to exactly `width` cells using a space with no color.
fn pad_line(mut line: RichLine, width: usize) -> RichLine {
    line.truncate(width);
    while line.len() < width {
        line.push(RichCell {
            ch: ' ',
            fg: None,
            bg: None,
        });
    }
    line
}

/// Wrap a content RichLine in box-drawing borders "║ <content> ║".
fn box_row(inner: RichLine) -> RichLine {
    let mut row = vec![
        RichCell {
            ch: '║',
            fg: None,
            bg: None,
        },
        RichCell {
            ch: ' ',
            fg: None,
            bg: None,
        },
    ];
    row.extend(inner);
    row.push(RichCell {
        ch: ' ',
        fg: None,
        bg: None,
    });
    row.push(RichCell {
        ch: '║',
        fg: None,
        bg: None,
    });
    row
}

/// Build a full-width box border string as a RichLine.
fn box_border(width: usize, left: char, fill: char, right: char) -> RichLine {
    let mut row = vec![RichCell {
        ch: left,
        fg: None,
        bg: None,
    }];
    for _ in 0..width {
        row.push(RichCell {
            ch: fill,
            fg: None,
            bg: None,
        });
    }
    row.push(RichCell {
        ch: right,
        fg: None,
        bg: None,
    });
    row
}

// ─── Component enum ──────────────────────────────────────────────────────────

/// Component of the HSV color being edited.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditorComponent {
    /// Hue component (0-360 degrees).
    Hue,
    /// Saturation component (0-1).
    Saturation,
    /// Value/brightness component (0-1).
    Value,
    /// Color selector (choosing which color to edit).
    ColorSelector,
    /// Save dialog for naming and saving palettes.
    SaveDialog,
}

impl EditorComponent {
    /// Cycle to the next component in the H→S→V→H sequence.
    pub fn next(self) -> Self {
        match self {
            Self::Hue => Self::Saturation,
            Self::Saturation => Self::Value,
            Self::Value => Self::Hue,
            other => other,
        }
    }

    /// Cycle to the previous component in H←S←V←H sequence.
    pub fn prev(self) -> Self {
        match self {
            Self::Hue => Self::Value,
            Self::Saturation => Self::Hue,
            Self::Value => Self::Saturation,
            other => other,
        }
    }
}

// ─── Editor mode ─────────────────────────────────────────────────────────────

/// Current mode of the palette editor.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditorMode {
    /// Editing colors in the HSV picker.
    Editing,
    /// Save dialog for naming and saving the current palette.
    SaveDialog,
    /// Load dialog for selecting a saved palette.
    LoadDialog,
}

// ─── Editor state ────────────────────────────────────────────────────────────

/// State of the palette editor overlay.
#[derive(Debug, Clone)]
pub struct PaletteEditorState {
    /// Current editor mode (Editing, SaveDialog, LoadDialog).
    pub mode: EditorMode,
    /// Index of the currently selected color (0-10).
    pub selected_color_index: usize,
    /// Currently selected HSV component being edited.
    pub selected_component: EditorComponent,
    /// Current palette colors.
    pub colors: [RgbColor; PALETTE_COLOR_COUNT],
    /// Original colors when editor was opened (for discard/reset).
    pub original_colors: [RgbColor; PALETTE_COLOR_COUNT],
    /// Name of the base palette being edited.
    pub base_palette_name: String,
    /// Whether the palette has been modified.
    pub is_modified: bool,
    /// Input buffer for save dialog.
    pub save_name_input: String,
    /// Index of the currently selected saved palette in load dialog.
    pub saved_palette_index: usize,
    /// List of saved palettes from storage.
    pub saved_palettes_list: Vec<crate::palette_manager::SavedPalette>,
}

impl PaletteEditorState {
    /// Create a new palette editor state from the given palette.
    pub fn new(palette: &Palette) -> Self {
        let colors = get_palette_colors(palette);
        let base_palette_name = palette.name().to_string();

        Self {
            mode: EditorMode::Editing,
            selected_color_index: 0,
            selected_component: EditorComponent::Hue,
            colors,
            original_colors: colors,
            base_palette_name,
            is_modified: false,
            save_name_input: String::new(),
            saved_palette_index: 0,
            saved_palettes_list: Vec::new(),
        }
    }

    /// Get the HSV color of the currently selected color.
    pub fn current_hsv(&self) -> HsvColor {
        rgb_to_hsv(self.colors[self.selected_color_index])
    }

    /// Set the RGB color of the currently selected color.
    pub fn set_current_color(&mut self, rgb: RgbColor) {
        self.colors[self.selected_color_index] = rgb;
        self.is_modified = true;
    }

    /// Adjust the hue of the currently selected color.
    pub fn adjust_hue(&mut self, delta: f32) {
        let mut hsv = self.current_hsv();
        hsv.h = (hsv.h + delta + 360.0) % 360.0;
        self.set_current_color(hsv_to_rgb(hsv));
    }

    /// Adjust the saturation of the currently selected color.
    pub fn adjust_saturation(&mut self, delta: f32) {
        let mut hsv = self.current_hsv();
        hsv.s = (hsv.s + delta).clamp(0.0, 1.0);
        self.set_current_color(hsv_to_rgb(hsv));
    }

    /// Adjust the value/brightness of the currently selected color.
    pub fn adjust_value(&mut self, delta: f32) {
        let mut hsv = self.current_hsv();
        hsv.v = (hsv.v + delta).clamp(0.0, 1.0);
        self.set_current_color(hsv_to_rgb(hsv));
    }

    /// Adjust the currently selected component by `delta`.
    pub fn adjust_selected_component(&mut self, delta: f32) {
        match self.selected_component {
            EditorComponent::Hue => self.adjust_hue(delta * 360.0),
            EditorComponent::Saturation => self.adjust_saturation(delta),
            EditorComponent::Value => self.adjust_value(delta),
            _ => {}
        }
    }

    /// Reset colors to the original values when editor was opened.
    pub fn reset_to_original(&mut self) {
        self.colors = self.original_colors;
        self.is_modified = false;
    }

    /// Select the next color in the palette.
    pub fn select_next_color(&mut self) {
        self.selected_color_index = (self.selected_color_index + 1) % PALETTE_COLOR_COUNT;
    }

    /// Select the previous color in the palette.
    pub fn select_prev_color(&mut self) {
        self.selected_color_index = if self.selected_color_index == 0 {
            PALETTE_COLOR_COUNT - 1
        } else {
            self.selected_color_index - 1
        };
    }

    /// Get the display name for the current palette state.
    pub fn display_name(&self) -> String {
        if self.is_modified {
            format!("{} (modified)", self.base_palette_name)
        } else {
            self.base_palette_name.clone()
        }
    }
}

/// Get the 11 gradient colors from a palette.
fn get_palette_colors(palette: &Palette) -> [RgbColor; PALETTE_COLOR_COUNT] {
    use crate::render::palette::get_gradient_stops;

    let stops = get_gradient_stops(palette);
    let mut colors = [RgbColor { r: 0, g: 0, b: 0 }; PALETTE_COLOR_COUNT];

    for (i, color) in colors.iter_mut().enumerate() {
        let t = i as f32 / (PALETTE_COLOR_COUNT - 1) as f32;
        *color = interpolate_gradient(&stops, t);
    }

    colors
}

// ─── Overlay renderer ────────────────────────────────────────────────────────

/// Overlay renderer for the palette editor.
pub struct PaletteEditorOverlay;

/// Inner content width (between box borders + single space padding on each side).
const INNER_W: usize = 52;

impl PaletteEditorOverlay {
    /// Total width of the overlay in characters (including box borders).
    pub const WIDTH: usize = INNER_W + 4; // "║ " + content + " ║"
    /// Total height of the overlay in characters.
    pub const HEIGHT: usize = 22;

    /// Build the overlay lines for the current editor state.
    pub fn build_overlay(state: &PaletteEditorState) -> Vec<RichLine> {
        match state.mode {
            EditorMode::Editing => Self::build_editing_overlay(state),
            EditorMode::SaveDialog => Self::build_save_dialog_overlay(state),
            EditorMode::LoadDialog => {
                Self::build_load_dialog_overlay(state, &state.saved_palettes_list)
            }
        }
    }

    fn build_editing_overlay(state: &PaletteEditorState) -> Vec<RichLine> {
        let hsv = state.current_hsv();
        let rgb = state.colors[state.selected_color_index];

        // ── Row builders ────────────────────────────────────────────────────
        let top_border = box_border(INNER_W + 2, '╔', '═', '╗');
        let sep_border = box_border(INNER_W + 2, '╠', '═', '╣');
        let bot_border = box_border(INNER_W + 2, '╚', '═', '╝');

        let blank_row = box_row(pad_line(Vec::new(), INNER_W));

        // Title row
        let title_str = "PALETTE EDITOR";
        let pad_l = (INNER_W.saturating_sub(title_str.len())) / 2;
        let pad_r = INNER_W.saturating_sub(title_str.len() + pad_l);
        let title_line = pad_line(
            rich_text(
                &format!("{}{}{}", " ".repeat(pad_l), title_str, " ".repeat(pad_r)),
                None,
                None,
            ),
            INNER_W,
        );
        let title_row = box_row(title_line);

        // Gradient strip (interpolated across full inner width)
        let strip = pad_line(build_gradient_strip(&state.colors, INNER_W), INNER_W);
        let strip_row = box_row(strip);

        // Swatches + labels
        let swatches = build_swatches(&state.colors, state.selected_color_index, INNER_W);
        let swatch_row = box_row(swatches);
        let labels = build_swatch_labels(INNER_W);
        let labels_row = box_row(labels);

        // Color info row: large swatch block + RGB values
        let color_info_str = format!(
            "Color {:2}    ██  R:{:3} G:{:3} B:{:3}",
            state.selected_color_index + 1,
            rgb.r,
            rgb.g,
            rgb.b
        );
        let mut color_info_line: RichLine = Vec::new();
        // Label part
        let label_part = format!("Color {:2}    ", state.selected_color_index + 1);
        color_info_line.extend(rich_text(&label_part, None, None));
        // Color swatch (2 block chars with the current color)
        color_info_line.push(RichCell {
            ch: '█',
            fg: Some(rgb),
            bg: Some(rgb),
        });
        color_info_line.push(RichCell {
            ch: '█',
            fg: Some(rgb),
            bg: Some(rgb),
        });
        // RGB values
        let rgb_part = format!("  R:{:3} G:{:3} B:{:3}", rgb.r, rgb.g, rgb.b);
        color_info_line.extend(rich_text(&rgb_part, None, None));
        let _ = color_info_str; // suppress unused warning
        let color_row = box_row(pad_line(color_info_line, INNER_W));

        // Hue label + bar + cursor
        let hue_label = format!("  Hue   {:6.1}°", hsv.h);
        let hue_label_row = box_row(pad_line(rich_text(&hue_label, None, None), INNER_W));
        let hue_bar = pad_line(build_hue_bar(INNER_W), INNER_W);
        let hue_bar_row = box_row(hue_bar);
        let hue_cursor_frac = hsv.h / 360.0;
        let hue_cursor = if state.selected_component == EditorComponent::Hue {
            pad_line(build_cursor_row(INNER_W, hue_cursor_frac), INNER_W)
        } else {
            pad_line(rich_text(&" ".repeat(INNER_W), None, None), INNER_W)
        };
        let hue_cursor_row = box_row(hue_cursor);

        // Saturation label + bar + cursor
        let sat_label = format!("  Sat   {:.2}", hsv.s);
        let sat_label_row = box_row(pad_line(rich_text(&sat_label, None, None), INNER_W));
        let sat_bar = pad_line(build_sat_bar(INNER_W, hsv.h, hsv.v), INNER_W);
        let sat_bar_row = box_row(sat_bar);
        let sat_cursor = if state.selected_component == EditorComponent::Saturation {
            pad_line(build_cursor_row(INNER_W, hsv.s), INNER_W)
        } else {
            pad_line(rich_text(&" ".repeat(INNER_W), None, None), INNER_W)
        };
        let sat_cursor_row = box_row(sat_cursor);

        // Value label + bar + cursor
        let val_label = format!("  Val   {:.2}", hsv.v);
        let val_label_row = box_row(pad_line(rich_text(&val_label, None, None), INNER_W));
        let val_bar = pad_line(build_val_bar(INNER_W, hsv.h, hsv.s), INNER_W);
        let val_bar_row = box_row(val_bar);
        let val_cursor = if state.selected_component == EditorComponent::Value {
            pad_line(build_cursor_row(INNER_W, hsv.v), INNER_W)
        } else {
            pad_line(rich_text(&" ".repeat(INNER_W), None, None), INNER_W)
        };
        let val_cursor_row = box_row(val_cursor);

        // Key hint rows
        let hint1 = "  ←/→: Stop  ↑/↓: Adjust  Tab: H→S→V  R: Reset";
        let hint2 = "  Enter: Apply & Close   Ctrl+S: Save   Esc: Discard";
        let hint1_row = box_row(pad_line(rich_text(hint1, None, None), INNER_W));
        let hint2_row = box_row(pad_line(rich_text(hint2, None, None), INNER_W));

        vec![
            top_border,         // 0  ╔══╗
            title_row,          // 1  ║ PALETTE EDITOR ║
            sep_border.clone(), // 2  ╠══╣
            strip_row,          // 3  gradient strip
            blank_row.clone(),  // 4
            swatch_row,         // 5  swatches
            labels_row,         // 6  index labels
            blank_row.clone(),  // 7
            color_row,          // 8  Color N  ██  R:xxx G:xxx B:xxx
            blank_row.clone(),  // 9
            hue_label_row,      // 10 Hue 240.0°
            hue_bar_row,        // 11 rainbow bar
            hue_cursor_row,     // 12 cursor
            sat_label_row,      // 13 Sat 0.78
            sat_bar_row,        // 14 saturation bar
            sat_cursor_row,     // 15 cursor
            val_label_row,      // 16 Val 0.92
            val_bar_row,        // 17 value bar
            val_cursor_row,     // 18 cursor
            sep_border,         // 19 ╠══╣
            hint1_row,          // 20 key hints
            hint2_row,          // 21 key hints
            bot_border,         // 22 ╚══╝
        ]
    }

    fn build_save_dialog_overlay(state: &PaletteEditorState) -> Vec<RichLine> {
        let inner_w = 38usize;
        let top = box_border(inner_w + 2, '╔', '═', '╗');
        let sep = box_border(inner_w + 2, '╠', '═', '╣');
        let bot = box_border(inner_w + 2, '╚', '═', '╝');
        let blank = box_row(pad_line(Vec::new(), inner_w));

        let title_row = box_row(pad_line(rich_text("   SAVE PALETTE", None, None), inner_w));
        let name_str = format!("Name: {:<25}", state.save_name_input);
        let name_row = box_row(pad_line(rich_text(&name_str, None, None), inner_w));
        let hint_row = box_row(pad_line(
            rich_text("  Enter: Save    Esc: Cancel", None, None),
            inner_w,
        ));

        vec![
            top,
            title_row,
            sep,
            blank.clone(),
            name_row,
            blank,
            hint_row,
            bot,
        ]
    }

    fn build_load_dialog_overlay(
        state: &PaletteEditorState,
        saved_palettes: &[crate::palette_manager::SavedPalette],
    ) -> Vec<RichLine> {
        let inner_w = 38usize;
        let top = box_border(inner_w + 2, '╔', '═', '╗');
        let sep = box_border(inner_w + 2, '╠', '═', '╣');
        let bot = box_border(inner_w + 2, '╚', '═', '╝');
        let blank = box_row(pad_line(Vec::new(), inner_w));

        let title_row = box_row(pad_line(rich_text("   LOAD PALETTE", None, None), inner_w));

        let mut lines = vec![top, title_row, sep, blank.clone()];

        if saved_palettes.is_empty() {
            lines.push(box_row(pad_line(
                rich_text("  No saved palettes yet", None, None),
                inner_w,
            )));
        } else {
            for (i, palette) in saved_palettes.iter().enumerate().take(8) {
                let marker = if i == state.saved_palette_index {
                    "›"
                } else {
                    " "
                };
                let truncated = if palette.name.len() > 28 {
                    &palette.name[..28]
                } else {
                    &palette.name
                };
                let entry = format!(" {} {:2}. {}", marker, i + 1, truncated);
                lines.push(box_row(pad_line(rich_text(&entry, None, None), inner_w)));
            }
        }

        lines.push(blank);
        lines.push(box_row(pad_line(
            rich_text("  ↑/↓: Select  Enter: Load", None, None),
            inner_w,
        )));
        lines.push(box_row(pad_line(
            rich_text("  Esc: Cancel", None, None),
            inner_w,
        )));
        lines.push(bot);
        lines
    }

    /// Calculate the centered position for the overlay.
    pub fn calculate_position(term_width: usize, term_height: usize) -> (usize, usize) {
        let x = (term_width.saturating_sub(Self::WIDTH)) / 2;
        let y = (term_height.saturating_sub(Self::HEIGHT)) / 2;
        (x, y)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_palette_editor_state_creation() {
        let state = PaletteEditorState::new(&Palette::Forest);
        assert_eq!(state.selected_color_index, 0);
        assert!(!state.is_modified);
        assert_eq!(state.mode, EditorMode::Editing);
    }

    #[test]
    fn test_color_navigation() {
        let mut state = PaletteEditorState::new(&Palette::Forest);

        state.select_next_color();
        assert_eq!(state.selected_color_index, 1);

        state.select_prev_color();
        assert_eq!(state.selected_color_index, 0);

        state.select_prev_color();
        assert_eq!(state.selected_color_index, PALETTE_COLOR_COUNT - 1);
    }

    #[test]
    fn test_hsv_adjustment() {
        let mut state = PaletteEditorState::new(&Palette::Forest);

        // Set a known color
        state.colors[0] = RgbColor { r: 255, g: 0, b: 0 }; // Pure red
        let original_hue = state.current_hsv().h;
        assert!(
            (original_hue - 0.0).abs() < 0.1 || (original_hue - 360.0).abs() < 0.1,
            "Red should have hue ~0"
        );

        state.adjust_hue(10.0);
        assert!(state.is_modified);

        let new_hue = state.current_hsv().h;
        let expected = 10.0;
        let diff = (new_hue - expected).abs();
        assert!(
            diff < 1.0,
            "hue should change by ~10 degrees: new={}, diff={}",
            new_hue,
            diff
        );
    }

    #[test]
    fn test_saturation_clamping() {
        let mut state = PaletteEditorState::new(&Palette::Forest);

        state.adjust_saturation(-2.0);
        assert!((state.current_hsv().s - 0.0).abs() < 0.01);

        state.adjust_saturation(2.0);
        assert!((state.current_hsv().s - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_reset() {
        let mut state = PaletteEditorState::new(&Palette::Forest);
        let original = state.colors[0];

        state.adjust_hue(45.0);
        assert!(state.is_modified);

        state.reset_to_original();
        assert!(!state.is_modified);
        assert_eq!(state.colors[0], original);
    }

    #[test]
    fn test_component_cycle() {
        assert_eq!(EditorComponent::Hue.next(), EditorComponent::Saturation);
        assert_eq!(EditorComponent::Saturation.next(), EditorComponent::Value);
        assert_eq!(EditorComponent::Value.next(), EditorComponent::Hue);

        assert_eq!(EditorComponent::Hue.prev(), EditorComponent::Value);
        assert_eq!(EditorComponent::Value.prev(), EditorComponent::Saturation);
        assert_eq!(EditorComponent::Saturation.prev(), EditorComponent::Hue);
    }

    #[test]
    fn test_build_overlay_produces_lines() {
        let state = PaletteEditorState::new(&Palette::Forest);
        let lines = PaletteEditorOverlay::build_overlay(&state);
        assert!(!lines.is_empty());
        // Each line should have WIDTH cells
        for (i, line) in lines.iter().enumerate() {
            assert_eq!(
                line.len(),
                PaletteEditorOverlay::WIDTH,
                "Line {} has wrong width: {}",
                i,
                line.len()
            );
        }
    }

    #[test]
    fn test_rich_text() {
        let line = rich_text("abc", None, None);
        assert_eq!(line.len(), 3);
        assert_eq!(line[0].ch, 'a');
        assert_eq!(line[1].ch, 'b');
        assert_eq!(line[2].ch, 'c');
    }
}
