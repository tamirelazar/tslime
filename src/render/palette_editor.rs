use crate::cli::Palette;
use crate::render::palette::{
    hsv_to_rgb, interpolate_gradient, rgb_to_hsv, GradientStop, HsvColor, RgbColor,
};
use crate::render::panel::{Padding, PanelBuilder, RenderedOverlay, TextAlignment};

/// Number of colors in the palette gradient.
pub const PALETTE_COLOR_COUNT: usize = 11;

/// Number of spaces between keybind and label columns in the keybind section.
/// Set this to adjust column spacing in the Palette Editor keybinds panel.
pub const KEYBIND_LABEL_GAP: usize = 3;

/// Inner content width (between box borders + single space padding on each side).
const INNER_W: usize = 52;

/// Column offset from the start of an overlay line to the first content character.
/// With Padding::COMPACT (left=1) and a border: border(1) + padding.left(1) = 2.
const CONTENT_OFFSET: usize = 2;

/// Length of the HSV slider track in characters.
const TRACK_LEN: usize = 38;

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
    /// Index of the currently selected color (0-10 = individual stop, 11 = ALL).
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

    /// True when the special "ALL" slot is selected (index == PALETTE_COLOR_COUNT).
    pub fn is_all_selected(&self) -> bool {
        self.selected_color_index == PALETTE_COLOR_COUNT
    }

    /// Circular mean hue, linear mean sat/val across all stops.
    fn average_hsv(&self) -> HsvColor {
        let mut sin_sum = 0.0f32;
        let mut cos_sum = 0.0f32;
        let mut s_sum = 0.0f32;
        let mut v_sum = 0.0f32;
        for &color in &self.colors {
            let hsv = rgb_to_hsv(color);
            let h_rad = hsv.h.to_radians();
            sin_sum += h_rad.sin();
            cos_sum += h_rad.cos();
            s_sum += hsv.s;
            v_sum += hsv.v;
        }
        let n = PALETTE_COLOR_COUNT as f32;
        let avg_h = sin_sum.atan2(cos_sum).to_degrees();
        let avg_h = if avg_h < 0.0 { avg_h + 360.0 } else { avg_h };
        HsvColor {
            h: avg_h,
            s: s_sum / n,
            v: v_sum / n,
        }
    }

    /// Get the HSV color of the currently selected color (average when ALL selected).
    pub fn current_hsv(&self) -> HsvColor {
        if self.is_all_selected() {
            self.average_hsv()
        } else {
            rgb_to_hsv(self.colors[self.selected_color_index])
        }
    }

    /// Set the RGB color of the currently selected color (no-op when ALL selected).
    pub fn set_current_color(&mut self, rgb: RgbColor) {
        if self.selected_color_index < PALETTE_COLOR_COUNT {
            self.colors[self.selected_color_index] = rgb;
            self.is_modified = true;
        }
    }

    /// Adjust the hue of the selected color(s) by `delta` degrees.
    pub fn adjust_hue(&mut self, delta: f32) {
        if self.is_all_selected() {
            for i in 0..PALETTE_COLOR_COUNT {
                let mut hsv = rgb_to_hsv(self.colors[i]);
                hsv.h = (hsv.h + delta + 360.0) % 360.0;
                self.colors[i] = hsv_to_rgb(hsv);
            }
            self.is_modified = true;
        } else {
            let mut hsv = rgb_to_hsv(self.colors[self.selected_color_index]);
            hsv.h = (hsv.h + delta + 360.0) % 360.0;
            self.set_current_color(hsv_to_rgb(hsv));
        }
    }

    /// Adjust the saturation of the selected color(s) by `delta`.
    pub fn adjust_saturation(&mut self, delta: f32) {
        if self.is_all_selected() {
            for i in 0..PALETTE_COLOR_COUNT {
                let mut hsv = rgb_to_hsv(self.colors[i]);
                hsv.s = (hsv.s + delta).clamp(0.0, 1.0);
                self.colors[i] = hsv_to_rgb(hsv);
            }
            self.is_modified = true;
        } else {
            let mut hsv = rgb_to_hsv(self.colors[self.selected_color_index]);
            hsv.s = (hsv.s + delta).clamp(0.0, 1.0);
            self.set_current_color(hsv_to_rgb(hsv));
        }
    }

    /// Adjust the value/brightness of the selected color(s) by `delta`.
    pub fn adjust_value(&mut self, delta: f32) {
        if self.is_all_selected() {
            for i in 0..PALETTE_COLOR_COUNT {
                let mut hsv = rgb_to_hsv(self.colors[i]);
                hsv.v = (hsv.v + delta).clamp(0.0, 1.0);
                self.colors[i] = hsv_to_rgb(hsv);
            }
            self.is_modified = true;
        } else {
            let mut hsv = rgb_to_hsv(self.colors[self.selected_color_index]);
            hsv.v = (hsv.v + delta).clamp(0.0, 1.0);
            self.set_current_color(hsv_to_rgb(hsv));
        }
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

    /// Select the next color in the palette (wraps through ALL slot at index 11).
    pub fn select_next_color(&mut self) {
        self.selected_color_index = (self.selected_color_index + 1) % (PALETTE_COLOR_COUNT + 1);
    }

    /// Select the previous color in the palette (wraps back through ALL slot).
    pub fn select_prev_color(&mut self) {
        self.selected_color_index = if self.selected_color_index == 0 {
            PALETTE_COLOR_COUNT // wraps back to ALL slot
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

// ─── Private content builders ─────────────────────────────────────────────────

/// Build the stop selector row: 11 color slots + 1 ALL slot, each 4 chars wide.
/// Returns 48 chars (12 × 4) for centering within INNER_W.
fn build_swatches_str(selected: usize) -> String {
    let mut s = String::with_capacity(48);
    for i in 0..PALETTE_COLOR_COUNT {
        if i == selected {
            s.push_str("[◆] ");
        } else {
            s.push_str(" ◆  ");
        }
    }
    if selected == PALETTE_COLOR_COUNT {
        s.push_str("[○] ");
    } else {
        s.push_str(" ○  ");
    }
    s
}

/// Build the stop index label row: " 1   2  … 11  ALL", returns 48 chars for centering.
fn build_swatch_labels_str() -> String {
    let mut s = String::with_capacity(48);
    for i in 1..=PALETTE_COLOR_COUNT {
        if i < 10 {
            s.push_str(&format!(" {i}  "));
        } else {
            s.push_str(&format!("{i}  "));
        }
    }
    s.push_str("ALL ");
    s
}

/// Build hex code string for centering.
fn build_hex_str(rgb: RgbColor) -> String {
    format!("#{:02x}{:02x}{:02x}", rgb.r, rgb.g, rgb.b)
}

/// Build HSV slider label (with arrows if active).
fn build_slider_label(is_active: bool, comp: char, value_str: &str) -> String {
    if is_active {
        format!("◀ {} {} ▶", comp, value_str)
    } else {
        format!("  {} {}  ", comp, value_str)
    }
}

/// Build HSV slider bar (38 chars with ◆ cursor).
fn build_slider_bar(frac: f32) -> String {
    let cursor_pos =
        ((frac.clamp(0.0, 1.0) * (TRACK_LEN - 1) as f32).round() as usize).min(TRACK_LEN - 1);
    (0..TRACK_LEN)
        .map(|i| if i == cursor_pos { '◆' } else { '█' })
        .collect()
}

/// Build per-cell color overrides for the editing overlay.
///
/// New layout (16 rows, 0-indexed):
/// 0  top border
/// 1  gradient strip    ← ▄ with fg=color(t), bg=color(t+Δ)
/// 2  empty
/// 3  stop selector     ← ◆ diamonds colored by stop color
/// 4  swatch labels
/// 5  empty
/// 6  color info        ← ◆ swatch colored by stop color
/// 7  separator
/// 8  H slider          ← rainbow track, ◆ in white on colored bg
/// 9  S slider          ← sat gradient track
/// 10 V slider          ← val gradient track
/// 11 empty
/// 12 separator
/// 13 hint row 1
/// 14 hint row 2
/// 15 bottom border
#[allow(clippy::type_complexity)]
fn build_editor_rich_lines(
    state: &PaletteEditorState,
    lines: &[String],
    hsv: HsvColor,
    text_primary: RgbColor,
    accent: RgbColor,
) -> Vec<Vec<(char, Option<RgbColor>, Option<RgbColor>)>> {
    let mut rich: Vec<Vec<(char, Option<RgbColor>, Option<RgbColor>)>> = lines
        .iter()
        .map(|l| l.chars().map(|c| (c, None, None)).collect())
        .collect();

    let rgb_for_display = if state.is_all_selected() {
        hsv_to_rgb(hsv)
    } else {
        state.colors[state.selected_color_index]
    };

    let stops: Vec<GradientStop> = state
        .colors
        .iter()
        .enumerate()
        .map(|(i, &color)| GradientStop {
            position: i as f32 / (PALETTE_COLOR_COUNT - 1) as f32,
            color,
        })
        .collect();

    // Line 3: centered stops — shift column offset by +2 for centering
    if rich.len() > 3 {
        for i in 0..PALETTE_COLOR_COUNT {
            let col = CONTENT_OFFSET + 2 + i * 4 + 1;
            if col < rich[3].len() {
                rich[3][col].1 = Some(state.colors[i]);
            }
        }
        let all_col = CONTENT_OFFSET + 2 + PALETTE_COLOR_COUNT * 4 + 1;
        if all_col < rich[3].len() {
            let all_color = if state.is_all_selected() {
                RgbColor {
                    r: 255,
                    g: 255,
                    b: 255,
                }
            } else {
                RgbColor {
                    r: 140,
                    g: 140,
                    b: 140,
                }
            };
            rich[3][all_col].1 = Some(all_color);
        }
    }

    // Line 6: hex code — centered
    if rich.len() > 6 {
        let hex_start = CONTENT_OFFSET + (INNER_W - 7) / 2;
        for i in 0..7 {
            if hex_start + i < rich[6].len() {
                rich[6][hex_start + i].1 = Some(rgb_for_display);
            }
        }
    }

    // Color hint keys (lines 20-26) with accent color
    // Special case: line 20 is "← select →", accent both arrow characters
    if rich.len() > 20 {
        let arrow_line = &mut rich[20];
        for (_idx, (c, fg, _bg)) in arrow_line.iter_mut().enumerate() {
            if *c == '←' || *c == '→' {
                *fg = Some(accent);
            }
        }
    }
    // Special case: line 21 is "↑ adjust ↓", accent both arrow characters
    if rich.len() > 21 {
        let arrow_line = &mut rich[21];
        for (_idx, (c, fg, _bg)) in arrow_line.iter_mut().enumerate() {
            if *c == '↑' || *c == '↓' {
                *fg = Some(accent);
            }
        }
    }
    // Line 22: accent "Tab" and arrows on the single centered line
    if rich.len() > 22 {
        let tab_line = &mut rich[22];
        let tab_word = "Tab";
        let line_str: String = tab_line.iter().map(|(c, _, _)| *c).collect();
        // Accent "Tab" plus 2 chars to the left
        if let Some(tab_pos) = line_str.find(tab_word) {
            let accent_start = tab_pos.saturating_sub(2);
            let accent_end = (tab_pos + tab_word.len()).min(tab_line.len());
            for col in accent_start..accent_end {
                tab_line[col].1 = Some(accent);
            }
        }
        // Accent all arrows "→" in the line
        for (_idx, (c, fg, _bg)) in tab_line.iter_mut().enumerate() {
            if *c == '→' {
                *fg = Some(accent);
            }
        }
        // If you want to accent the spaces between "Tab" and "H" as well, modify here (not requested).
    }
    // Remaining color keys: accent color for centered lines
    let key_patterns = ["r", "Enter", "Ctrl+S", "Esc"];
    let hint_line_indices = [23, 24, 25, 26];
    for (i, &line_idx) in hint_line_indices.iter().enumerate() {
        if line_idx < rich.len() && i < key_patterns.len() {
            let pattern = key_patterns[i];
            let line_str: String = rich[line_idx].iter().map(|(c, _, _)| *c).collect();
            if let Some(start_pos) = line_str.find(pattern) {
                // Only accent the key substring, NOT any left padding
                for col in start_pos..(start_pos + pattern.len()).min(rich[line_idx].len()) {
                    rich[line_idx][col].1 = Some(accent);
                }
            }
        }
    }

    // H bar (line 11): hue gradient, apply current S/V
    let h_cursor = ((hsv.h / 360.0) * (TRACK_LEN - 1) as f32).round() as usize;
    if rich.len() > 11 {
        let h_start = CONTENT_OFFSET + 7; // centered offset
        for i in 0..TRACK_LEN {
            let col = h_start + i;
            if col < rich[11].len() {
                let h = i as f32 / (TRACK_LEN - 1) as f32 * 360.0;
                let color = hsv_to_rgb(HsvColor {
                    h,
                    s: hsv.s,
                    v: hsv.v,
                });
                if i == h_cursor {
                    rich[11][col] = ('◆', Some(text_primary), Some(color));
                } else {
                    rich[11][col] = ('█', Some(color), Some(color));
                }
            }
        }
    }

    // S bar (line 14)
    let s_cursor = (hsv.s * (TRACK_LEN - 1) as f32).round() as usize;
    if rich.len() > 14 {
        let s_start = CONTENT_OFFSET + 7;
        for i in 0..TRACK_LEN {
            let col = s_start + i;
            if col < rich[14].len() {
                let s = i as f32 / (TRACK_LEN - 1) as f32;
                let color = hsv_to_rgb(HsvColor {
                    h: hsv.h,
                    s,
                    v: hsv.v,
                });
                if i == s_cursor {
                    rich[14][col] = ('◆', Some(text_primary), Some(color));
                } else {
                    rich[14][col] = ('█', Some(color), Some(color));
                }
            }
        }
    }

    // V bar (line 17)
    let v_cursor = (hsv.v * (TRACK_LEN - 1) as f32).round() as usize;
    if rich.len() > 17 {
        let v_start = CONTENT_OFFSET + 7;
        for i in 0..TRACK_LEN {
            let col = v_start + i;
            if col < rich[17].len() {
                let v = i as f32 / (TRACK_LEN - 1) as f32;
                let color = hsv_to_rgb(HsvColor {
                    h: hsv.h,
                    s: hsv.s,
                    v,
                });
                if i == v_cursor {
                    rich[17][col] = ('◆', Some(text_primary), Some(color));
                } else {
                    rich[17][col] = ('█', Some(color), Some(color));
                }
            }
        }
    }

    // Gradient strip (line 28)
    if rich.len() > 28 {
        for i in 0..INNER_W {
            let col = CONTENT_OFFSET + i;
            if col < rich[28].len() {
                let t = i as f32 / (INNER_W - 1).max(1) as f32;
                let t_next = (t + 1.5 / INNER_W as f32).min(1.0);
                let fg_color = interpolate_gradient(&stops, t);
                let bg_color = interpolate_gradient(&stops, t_next);
                rich[28][col] = ('▄', Some(fg_color), Some(bg_color));
            }
        }
    }

    rich
}

// ─── Overlay renderer ────────────────────────────────────────────────────────

/// Overlay renderer for the palette editor.
pub struct PaletteEditorOverlay;

impl PaletteEditorOverlay {
    /// Total width of the overlay in characters (including border and padding).
    /// border(1) + padding.left(1) + INNER_W(52) + padding.right(1) + border(1) = 56
    pub const WIDTH: usize = INNER_W + 4;

    /// Total height of the overlay in characters (all 9 items implemented).
    /// top_border(1) + 28 content rows + bottom_border(1) = 30
    pub const HEIGHT: usize = 30;

    /// Build the overlay for the current editor state.
    pub fn build_overlay(
        state: &PaletteEditorState,
        panel_style: &crate::render::theme::PanelStyle,
        accent: RgbColor,
    ) -> RenderedOverlay {
        match state.mode {
            EditorMode::Editing => Self::build_editing_overlay(state, panel_style, accent),
            EditorMode::SaveDialog => Self::build_save_dialog_overlay(state),
            EditorMode::LoadDialog => {
                Self::build_load_dialog_overlay(state, &state.saved_palettes_list)
            }
        }
    }

    fn build_editing_overlay(
        state: &PaletteEditorState,
        panel_style: &crate::render::theme::PanelStyle,
        accent: RgbColor,
    ) -> RenderedOverlay {
        let hsv = state.current_hsv();
        let rgb_for_display = if state.is_all_selected() {
            hsv_to_rgb(hsv)
        } else {
            state.colors[state.selected_color_index]
        };

        let gradient_str = "▄".repeat(INNER_W);
        let swatches_str = build_swatches_str(state.selected_color_index);
        let labels_str = build_swatch_labels_str();
        let hex_str = build_hex_str(rgb_for_display);

        let h_active = state.selected_component == EditorComponent::Hue;
        let s_active = state.selected_component == EditorComponent::Saturation;
        let v_active = state.selected_component == EditorComponent::Value;

        let h_label = build_slider_label(h_active, 'H', &format!("{:.1}°", hsv.h));
        let h_bar = build_slider_bar(hsv.h / 360.0);
        let s_label = build_slider_label(s_active, 'S', &format!("{:.2}", hsv.s));
        let s_bar = build_slider_bar(hsv.s);
        let v_label = build_slider_label(v_active, 'V', &format!("{:.2}", hsv.v));
        let v_bar = build_slider_bar(hsv.v);

        // First line is special: centered "← select →"
        let first_hint_line = "← select →";
        // Second line is also special: centered "↑ adjust ↓"
        let second_hint_line = "↑ adjust ↓";
        // Remaining hints for column alignment (without previous "↑/↓      adjust value")
        // Single centered hint instead of two columns for Tab
        let tab_hint_line = "Tab   H → S → V";
        let hints = [
            ("r", "reset"),
            ("Enter", "apple"),
            ("Ctrl+S", "save palette"),
            ("Esc", "discard"),
        ];
        // For single-line, centered hints block—no manual pad, PanelBuilder will center.
        let key_label_lines: Vec<String> = hints
            .iter()
            .map(|(key, label)| format!("{key} {label}"))
            .collect();

        let mut overlay = PanelBuilder::new(INNER_W, None)
            .with_padding(Padding::COMPACT)
            .with_title("PALETTE EDITOR")
            .with_title_box()
            .add_empty() // line 1
            .add_empty() // line 2
            .add_single(swatches_str, TextAlignment::Center) // line 3
            .add_single(labels_str, TextAlignment::Center) // line 4
            .add_empty() // line 5
            .add_single(hex_str, TextAlignment::Center) // line 6
            .add_empty() // line 7
            .add_separator() // line 8
            .add_empty() // line 9
            .add_single(h_label, TextAlignment::Center) // line 10
            .add_single(h_bar, TextAlignment::Center) // line 11
            .add_empty() // line 12
            .add_single(s_label, TextAlignment::Center) // line 13
            .add_single(s_bar, TextAlignment::Center) // line 14
            .add_empty() // line 15
            .add_single(v_label, TextAlignment::Center) // line 16
            .add_single(v_bar, TextAlignment::Center) // line 17
            .add_empty() // line 18
            .add_separator() // line 19
            // Add the special centered keybind line
            .add_single(first_hint_line.to_string(), TextAlignment::Center) // line 20
            // Add the up/down centered keybind line
            .add_single(second_hint_line.to_string(), TextAlignment::Center) // line 21
            // Add single centered Tab H → S → V line
            .add_single(tab_hint_line.to_string(), TextAlignment::Center) // line 22
            // Add remaining keybind lines (no Tab) left-aligned
            .add_single(key_label_lines[0].clone(), TextAlignment::Center) // line 23
            .add_single(key_label_lines[1].clone(), TextAlignment::Center) // line 24
            .add_single(key_label_lines[2].clone(), TextAlignment::Center) // line 25
            .add_single(key_label_lines[3].clone(), TextAlignment::Center) // line 26
            .add_separator() // line 27
            .add_single(gradient_str, TextAlignment::Left) // line 28
            .build_overlay();

        overlay.rich_lines = Some(build_editor_rich_lines(
            state,
            &overlay.lines,
            hsv,
            panel_style.text_primary,
            accent,
        ));
        overlay
    }

    fn build_save_dialog_overlay(state: &PaletteEditorState) -> RenderedOverlay {
        let name_str = format!("Name: {:<25}", state.save_name_input);

        PanelBuilder::new(38, None)
            .with_padding(Padding::COMPACT)
            .with_title("SAVE PALETTE")
            .with_title_box()
            .add_empty()
            .add_single(name_str, TextAlignment::Left)
            .add_empty()
            .add_single(
                "  Enter: Save    Esc: Cancel".to_string(),
                TextAlignment::Left,
            )
            .build_overlay()
    }

    fn build_load_dialog_overlay(
        state: &PaletteEditorState,
        saved_palettes: &[crate::palette_manager::SavedPalette],
    ) -> RenderedOverlay {
        let mut builder = PanelBuilder::new(38, None)
            .with_padding(Padding::COMPACT)
            .with_title("LOAD PALETTE")
            .with_title_box()
            .add_empty();

        if saved_palettes.is_empty() {
            builder =
                builder.add_single("  No saved palettes yet".to_string(), TextAlignment::Left);
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
                builder = builder.add_single(entry, TextAlignment::Left);
            }
        }

        builder
            .add_empty()
            .add_single(
                "  ↑/↓: Select  Enter: Load".to_string(),
                TextAlignment::Left,
            )
            .add_single("  Esc: Cancel".to_string(), TextAlignment::Left)
            .build_overlay()
    }

    /// Calculate the centered position for the overlay.
    ///
    /// Adds 1 to y so the title box drawn at y-1 stays on screen.
    pub fn calculate_position(term_width: usize, term_height: usize) -> (usize, usize) {
        let x = (term_width.saturating_sub(Self::WIDTH)) / 2;
        let y = (term_height.saturating_sub(Self::HEIGHT + 1)) / 2 + 1;
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

        // Wraps back to ALL slot (index 11) from 0.
        state.select_prev_color();
        assert_eq!(state.selected_color_index, PALETTE_COLOR_COUNT);
    }

    #[test]
    fn test_color_navigation_wraps_forward_through_all() {
        let mut state = PaletteEditorState::new(&Palette::Forest);
        // Navigate forward through all 11 color stops to the ALL slot.
        for _ in 0..PALETTE_COLOR_COUNT {
            state.select_next_color();
        }
        assert_eq!(state.selected_color_index, PALETTE_COLOR_COUNT); // ALL slot

        // One more wraps back to 0.
        state.select_next_color();
        assert_eq!(state.selected_color_index, 0);
    }

    #[test]
    fn test_is_all_selected() {
        let mut state = PaletteEditorState::new(&Palette::Forest);
        assert!(!state.is_all_selected());

        state.selected_color_index = PALETTE_COLOR_COUNT;
        assert!(state.is_all_selected());
    }

    #[test]
    fn test_all_selected_adjust_applies_to_all_stops() {
        let mut state = PaletteEditorState::new(&Palette::Forest);
        let original_colors = state.colors;

        state.selected_color_index = PALETTE_COLOR_COUNT; // ALL slot
        state.adjust_hue(90.0);

        assert!(state.is_modified);
        // Every stop should have changed.
        for i in 0..PALETTE_COLOR_COUNT {
            assert_ne!(
                state.colors[i], original_colors[i],
                "Stop {} should have changed after ALL hue adjust",
                i
            );
        }
    }

    #[test]
    fn test_average_hsv_calculation() {
        let mut state = PaletteEditorState::new(&Palette::Forest);

        // Set all colors to pure red (hue=0, sat=1, val=1).
        for color in state.colors.iter_mut() {
            *color = RgbColor { r: 255, g: 0, b: 0 };
        }

        let avg = state.average_hsv();
        assert!(avg.s > 0.9, "Average saturation should be ~1.0");
        assert!(avg.v > 0.9, "Average value should be ~1.0");
    }

    #[test]
    fn test_hsv_adjustment() {
        let mut state = PaletteEditorState::new(&Palette::Forest);

        state.colors[0] = RgbColor { r: 255, g: 0, b: 0 }; // Pure red
        let original_hue = state.current_hsv().h;
        assert!(
            (original_hue - 0.0).abs() < 0.1 || (original_hue - 360.0).abs() < 0.1,
            "Red should have hue ~0"
        );

        state.adjust_hue(10.0);
        assert!(state.is_modified);

        let new_hue = state.current_hsv().h;
        let diff = (new_hue - 10.0).abs();
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
        let panel_style = crate::render::theme::GRUVBOX_DARK;
        let accent =
            crate::render::palette::palette_accent_color(&Palette::Forest, false, false, 0.0, None);
        let overlay = PaletteEditorOverlay::build_overlay(&state, &panel_style, accent);
        assert!(!overlay.lines.is_empty());
        for (i, line) in overlay.lines.iter().enumerate() {
            assert_eq!(
                line.chars().count(),
                PaletteEditorOverlay::WIDTH,
                "Line {} has wrong width: {} ('{}')",
                i,
                line.chars().count(),
                line
            );
        }
    }

    #[test]
    fn test_build_overlay_height() {
        let state = PaletteEditorState::new(&Palette::Forest);
        let panel_style = crate::render::theme::GRUVBOX_DARK;
        let accent =
            crate::render::palette::palette_accent_color(&Palette::Forest, false, false, 0.0, None);
        let overlay = PaletteEditorOverlay::build_overlay(&state, &panel_style, accent);
        assert_eq!(overlay.lines.len(), PaletteEditorOverlay::HEIGHT);
    }

    #[test]
    fn test_swatches_str_width() {
        let s = build_swatches_str(0);
        assert_eq!(s.chars().count(), 48); // 12 × 4 for centering
    }

    #[test]
    fn test_swatches_str_all_selected_width() {
        let s = build_swatches_str(PALETTE_COLOR_COUNT); // ALL slot
        assert_eq!(s.chars().count(), 48);
    }

    #[test]
    fn test_swatch_labels_str_width() {
        let s = build_swatch_labels_str();
        assert_eq!(s.chars().count(), 48);
    }

    #[test]
    fn test_hex_str_width() {
        let rgb = RgbColor {
            r: 255,
            g: 128,
            b: 0,
        };
        let s = build_hex_str(rgb);
        assert_eq!(s.chars().count(), 7); // #rrggbb
    }

    #[test]
    fn test_slider_label_width() {
        let s = build_slider_label(true, 'H', "180.0°");
        assert!(s.chars().count() > 0);
    }

    #[test]
    fn test_slider_bar_width() {
        let s = build_slider_bar(0.5);
        assert_eq!(s.chars().count(), TRACK_LEN);
    }

    #[test]
    fn test_slider_bar_cursor_position() {
        let s = build_slider_bar(0.0);
        assert!(s.starts_with('◆'));
        let s = build_slider_bar(1.0);
        assert!(s.ends_with('◆'));
    }

    #[test]
    fn test_build_overlay_all_selected() {
        let mut state = PaletteEditorState::new(&Palette::Forest);
        state.selected_color_index = PALETTE_COLOR_COUNT; // ALL slot
        let panel_style = crate::render::theme::GRUVBOX_DARK;
        let accent =
            crate::render::palette::palette_accent_color(&Palette::Forest, false, false, 0.0, None);
        let overlay = PaletteEditorOverlay::build_overlay(&state, &panel_style, accent);
        assert_eq!(overlay.lines.len(), PaletteEditorOverlay::HEIGHT);
        for (i, line) in overlay.lines.iter().enumerate() {
            assert_eq!(
                line.chars().count(),
                PaletteEditorOverlay::WIDTH,
                "Line {} has wrong width in ALL mode",
                i
            );
        }
    }
}
