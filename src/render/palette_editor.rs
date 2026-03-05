use crate::cli::Palette;
use crate::render::palette::{
    interpolate_gradient, oklch_to_rgb, oklch_to_srgb, srgb_to_oklch, GradientStop, OklchColor,
    RgbColor,
};
use crate::render::panel::{Padding, PanelBuilder, RenderedOverlay, RichCell, TextAlignment};

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

/// Length of the OKLch slider track in characters.
const TRACK_LEN: usize = 38;

/// Maximum chroma value for slider display and clamping.
const MAX_CHROMA: f32 = 0.4;

/// Row indices for the palette editor overlay layout.
/// These are 0-indexed positions within the overlay content.
mod rows {
    /// Stop selector row (diamond indicators for palette colors).
    pub const STOP_SELECTOR: usize = 3;

    /// Lightness slider row.
    pub const LIGHTNESS_SLIDER: usize = 9;

    /// Chroma slider row.
    pub const CHROMA_SLIDER: usize = 12;

    /// Hue slider row.
    pub const HUE_SLIDER: usize = 15;

    /// First hint row (arrow key indicators).
    pub const HINT_ARROWS: usize = 18;

    /// Second hint row (adjust indicators).
    pub const HINT_ADJUST: usize = 19;

    /// Third hint row (Tab navigation).
    pub const HINT_TAB: usize = 20;

    /// Gradient preview strip row.
    pub const GRADIENT_STRIP: usize = 27;
}

// ─── Component enum ──────────────────────────────────────────────────────────

/// Component of the OKLch color being edited.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditorComponent {
    /// Lightness component (0.0-1.0).
    Lightness,
    /// Chroma component (0.0-~0.4).
    Chroma,
    /// Hue component (0-360 degrees).
    Hue,
}

impl EditorComponent {
    /// Cycle to the next component in the L→C→H→L sequence.
    pub fn next(self) -> Self {
        match self {
            Self::Lightness => Self::Chroma,
            Self::Chroma => Self::Hue,
            Self::Hue => Self::Lightness,
        }
    }

    /// Cycle to the previous component in L←C←H←L sequence.
    pub fn prev(self) -> Self {
        match self {
            Self::Lightness => Self::Hue,
            Self::Chroma => Self::Lightness,
            Self::Hue => Self::Chroma,
        }
    }
}

// ─── Editor mode ─────────────────────────────────────────────────────────────

/// Current mode of the palette editor.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditorMode {
    /// Editing colors in the OKLch picker.
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
    /// Currently selected OKLch component being edited.
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
    /// Stored hue values for each color to preserve hue when chroma → 0.
    /// When chroma is near 0, hue becomes "powerless" (undefined per CSS Color Module Level 4).
    /// These stored values ensure hue is preserved during grayscale transitions.
    pub stored_hues: [f32; PALETTE_COLOR_COUNT],
}

impl PaletteEditorState {
    /// Create a new palette editor state from the given palette.
    pub fn new(palette: &Palette) -> Self {
        let colors = get_palette_colors(palette);
        let base_palette_name = palette.name().to_string();

        // Initialize stored hues from the initial palette colors
        let mut stored_hues = [0.0f32; PALETTE_COLOR_COUNT];
        for (i, &color) in colors.iter().enumerate() {
            let oklch = srgb_to_oklch(color);
            // If hue is NaN (color is grayscale), default to 0
            stored_hues[i] = if oklch.h.is_nan() { 0.0 } else { oklch.h };
        }

        Self {
            mode: EditorMode::Editing,
            selected_color_index: PALETTE_COLOR_COUNT,
            selected_component: EditorComponent::Lightness,
            colors,
            original_colors: colors,
            base_palette_name,
            is_modified: false,
            save_name_input: String::new(),
            saved_palette_index: 0,
            saved_palettes_list: Vec::new(),
            stored_hues,
        }
    }

    /// True when the special "ALL" slot is selected (index == PALETTE_COLOR_COUNT).
    pub fn is_all_selected(&self) -> bool {
        self.selected_color_index == PALETTE_COLOR_COUNT
    }

    /// Circular mean hue, linear mean lightness/chroma across all stops.
    /// Uses stored hues to preserve hue values when chroma is near 0 (powerless).
    fn average_oklch(&self) -> OklchColor {
        use crate::render::palette::OKLCH_EPSILON;

        let mut sin_sum = 0.0f32;
        let mut cos_sum = 0.0f32;
        let mut l_sum = 0.0f32;
        let mut c_sum = 0.0f32;
        let mut hue_count = 0;

        for (i, &color) in self.colors.iter().enumerate() {
            let oklch = srgb_to_oklch(color);
            l_sum += oklch.l;
            c_sum += oklch.c;

            // Use stored hue to avoid NaN when chroma ≈ 0
            // Only colors with non-negligible chroma contribute to hue average
            let hue = if oklch.c >= OKLCH_EPSILON {
                oklch.h
            } else {
                self.stored_hues[i]
            };

            if hue.is_finite() {
                let h_rad = hue.to_radians();
                sin_sum += h_rad.sin();
                cos_sum += h_rad.cos();
                hue_count += 1;
            }
        }

        let n = PALETTE_COLOR_COUNT as f32;
        let avg_h = if hue_count > 0 {
            let avg = sin_sum.atan2(cos_sum).to_degrees();
            if avg < 0.0 {
                avg + 360.0
            } else {
                avg
            }
        } else {
            0.0 // Default when no valid hues
        };

        OklchColor {
            l: l_sum / n,
            c: c_sum / n,
            h: avg_h,
        }
    }

    /// Get the OKLch color of the currently selected color (average when ALL selected).
    /// Uses stored hue when chroma is near 0 to preserve the intended hue value.
    pub fn current_oklch(&self) -> OklchColor {
        use crate::render::palette::OKLCH_EPSILON;

        if self.is_all_selected() {
            self.average_oklch()
        } else {
            let idx = self.selected_color_index;
            let mut oklch = srgb_to_oklch(self.colors[idx]);

            // When chroma is near 0, hue becomes "powerless" (NaN).
            // Use the stored hue to preserve the intended color.
            if oklch.c < OKLCH_EPSILON || oklch.h.is_nan() {
                oklch.h = self.stored_hues[idx];
            }

            oklch
        }
    }

    /// Set the RGB color of the currently selected color (no-op when ALL selected).
    pub fn set_current_color(&mut self, rgb: RgbColor) {
        if self.selected_color_index < PALETTE_COLOR_COUNT {
            self.colors[self.selected_color_index] = rgb;
            self.is_modified = true;
        }
    }

    /// Adjust all colors (or the selected one) using `f` to modify the OKLch value.
    /// Also updates stored_hues when chroma transitions from 0 to non-zero.
    fn adjust_oklch<F: Fn(&mut OklchColor)>(&mut self, f: F) {
        use crate::render::palette::OKLCH_EPSILON;

        if self.is_all_selected() {
            for i in 0..PALETTE_COLOR_COUNT {
                let mut oklch = srgb_to_oklch(self.colors[i]);
                let old_c = oklch.c;
                f(&mut oklch);

                // If chroma was 0 and is now non-zero, use stored hue
                if old_c < OKLCH_EPSILON && oklch.c >= OKLCH_EPSILON {
                    oklch.h = self.stored_hues[i];
                }

                self.colors[i] = oklch_to_rgb(oklch);

                // Update stored hue if chroma is still non-zero
                if oklch.c >= OKLCH_EPSILON {
                    self.stored_hues[i] = oklch.h;
                }
            }
            self.is_modified = true;
        } else {
            let idx = self.selected_color_index;
            let mut oklch = srgb_to_oklch(self.colors[idx]);
            let old_c = oklch.c;
            f(&mut oklch);

            // If chroma was 0 and is now non-zero, use stored hue
            if old_c < OKLCH_EPSILON && oklch.c >= OKLCH_EPSILON {
                oklch.h = self.stored_hues[idx];
            }

            self.set_current_color(oklch_to_rgb(oklch));

            // Update stored hue if chroma is still non-zero
            if oklch.c >= OKLCH_EPSILON {
                self.stored_hues[idx] = oklch.h;
            }
        }
    }

    /// Adjust the hue of the selected color(s) by `delta` degrees.
    /// Also updates stored_hues to preserve the new hue value.
    pub fn adjust_hue(&mut self, delta: f32) {
        // First, update stored hues for all affected colors
        if self.is_all_selected() {
            for i in 0..PALETTE_COLOR_COUNT {
                self.stored_hues[i] = (self.stored_hues[i] + delta + 360.0) % 360.0;
            }
        } else {
            let idx = self.selected_color_index;
            self.stored_hues[idx] = (self.stored_hues[idx] + delta + 360.0) % 360.0;
        }

        // Apply the hue adjustment, using stored hues as base when current hue is NaN
        self.adjust_oklch_with_hue_override(delta);
    }

    /// Helper to adjust OKLch with proper hue handling for NaN cases.
    /// Uses stored_hue as base when current chroma is too low.
    fn adjust_oklch_with_hue_override(&mut self, delta: f32) {
        use crate::render::palette::OKLCH_EPSILON;

        if self.is_all_selected() {
            for i in 0..PALETTE_COLOR_COUNT {
                let mut oklch = srgb_to_oklch(self.colors[i]);

                // Use stored hue as base if current hue is NaN (powerless)
                let base_hue = if oklch.h.is_nan() || oklch.c < OKLCH_EPSILON {
                    self.stored_hues[i]
                } else {
                    oklch.h
                };

                oklch.h = (base_hue + delta + 360.0) % 360.0;
                self.colors[i] = oklch_to_rgb(oklch);

                // Update stored hue if chroma is still non-zero
                if oklch.c >= OKLCH_EPSILON {
                    self.stored_hues[i] = oklch.h;
                }
            }
            self.is_modified = true;
        } else {
            let idx = self.selected_color_index;
            let mut oklch = srgb_to_oklch(self.colors[idx]);

            // Use stored hue as base if current hue is NaN (powerless)
            let base_hue = if oklch.h.is_nan() || oklch.c < OKLCH_EPSILON {
                self.stored_hues[idx]
            } else {
                oklch.h
            };

            oklch.h = (base_hue + delta + 360.0) % 360.0;
            self.colors[idx] = oklch_to_rgb(oklch);

            // Update stored hue if chroma is still non-zero
            if oklch.c >= OKLCH_EPSILON {
                self.stored_hues[idx] = oklch.h;
            }

            self.is_modified = true;
        }
    }

    /// Adjust the chroma of the selected color(s) by `delta`.
    /// Preserves stored_hues so hue can be restored when chroma increases from 0.
    pub fn adjust_chroma(&mut self, delta: f32) {
        self.adjust_oklch(|oklch| oklch.c = (oklch.c + delta).clamp(0.0, MAX_CHROMA));
    }

    /// Adjust the lightness of the selected color(s) by `delta`.
    pub fn adjust_lightness(&mut self, delta: f32) {
        self.adjust_oklch(|oklch| oklch.l = (oklch.l + delta).clamp(0.0, 1.0));
    }

    /// Adjust the currently selected component by `delta`.
    pub fn adjust_selected_component(&mut self, delta: f32) {
        match self.selected_component {
            EditorComponent::Lightness => self.adjust_lightness(delta),
            EditorComponent::Chroma => self.adjust_chroma(delta),
            EditorComponent::Hue => self.adjust_hue(delta * 360.0),
        }
    }

    /// Reset colors to the original values when editor was opened.
    /// Also resets stored_hues to match the original colors.
    pub fn reset_to_original(&mut self) {
        self.colors = self.original_colors;

        // Recalculate stored hues from original colors
        for (i, &color) in self.colors.iter().enumerate() {
            let oklch = srgb_to_oklch(color);
            self.stored_hues[i] = if oklch.h.is_nan() { 0.0 } else { oklch.h };
        }
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
            s.push_str(" ◉  ");
        } else {
            s.push_str(" ○  ");
        }
    }
    if selected == PALETTE_COLOR_COUNT {
        s.push_str(" ◉  ");
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

/// Build OKLch slider label (with arrows if active).
fn build_slider_label(is_active: bool, comp: char, value_str: &str) -> String {
    if is_active {
        format!("◀ {} {} ▶", comp, value_str)
    } else {
        format!("  {} {}  ", comp, value_str)
    }
}

/// Build OKLch slider bar (38 chars with ◆ cursor).
fn build_slider_bar(frac: f32) -> String {
    let cursor_pos =
        ((frac.clamp(0.0, 1.0) * (TRACK_LEN - 1) as f32).round() as usize).min(TRACK_LEN - 1);
    (0..TRACK_LEN)
        .map(|i| if i == cursor_pos { '◆' } else { '█' })
        .collect()
}

/// Build per-cell color overrides for the editing overlay.
///
/// Layout (rows are 0-indexed within overlay content):
/// 0  top border
/// 1  gradient strip    ← ▄ with fg=color(t), bg=color(t+Δ)
/// 2  empty
/// 3  stop selector     ← ◆ diamonds colored by stop color
/// 4  swatch labels
/// 5  empty
/// 6  color info        ← ◆ swatch colored by stop color
/// 7  separator
/// 8  L slider label
/// 9  L slider          ← lightness gradient track
/// 10 empty
/// 11 C slider label
/// 12 C slider          ← chroma gradient track
/// 13 empty
/// 14 H slider label
/// 15 H slider          ← hue rainbow track
/// 16 empty
/// 17 separator
/// 18-24 hint rows
/// 25 empty
/// 26 separator
/// 27 gradient preview strip
fn build_editor_rich_lines(
    state: &PaletteEditorState,
    lines: &[String],
    oklch: OklchColor,
    text_primary: RgbColor,
    accent: RgbColor,
    panel_bg: RgbColor,
) -> Vec<Vec<RichCell>> {
    let mut rich: Vec<Vec<RichCell>> = lines
        .iter()
        .map(|l| l.chars().map(|c| (c, None, None)).collect())
        .collect();

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
    if rich.len() > rows::STOP_SELECTOR {
        for i in 0..PALETTE_COLOR_COUNT {
            let col = CONTENT_OFFSET + 2 + i * 4 + 1;
            if col < rich[rows::STOP_SELECTOR].len() {
                rich[rows::STOP_SELECTOR][col].1 = Some(state.colors[i]);
            }
        }
        let all_col = CONTENT_OFFSET + 2 + PALETTE_COLOR_COUNT * 4 + 1;
        if all_col < rich[rows::STOP_SELECTOR].len() {
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
            rich[rows::STOP_SELECTOR][all_col].1 = Some(all_color);
        }
    }

    // Color hint keys (lines 18-24) with accent color.

    // Line 18: "← select →" — accent arrow characters
    if rich.len() > rows::HINT_ARROWS {
        for (c, fg, _) in rich[rows::HINT_ARROWS].iter_mut() {
            if *c == '←' || *c == '→' {
                *fg = Some(accent);
            }
        }
    }

    // Line 19: "↑ adjust ↓" — accent arrow characters
    if rich.len() > rows::HINT_ADJUST {
        for (c, fg, _) in rich[rows::HINT_ADJUST].iter_mut() {
            if *c == '↑' || *c == '↓' {
                *fg = Some(accent);
            }
        }
    }

    // Line 20: "Tab  L → C → H" — accent "Tab" and "→" arrows.
    // Search tab_line directly (char-indexed) to avoid the byte-vs-char mismatch
    // that arises when line_str.find() returns a byte offset: the '│' border glyph
    // is 3 bytes but 1 char, shifting all subsequent byte offsets by +2.
    if rich.len() > rows::HINT_TAB {
        let tab_line = &mut rich[rows::HINT_TAB];
        let tab_pos = (0..tab_line.len().saturating_sub(2)).find(|&i| {
            tab_line[i].0 == 'T' && tab_line[i + 1].0 == 'a' && tab_line[i + 2].0 == 'b'
        });
        if let Some(pos) = tab_pos {
            for col in pos..(pos + 3).min(tab_line.len()) {
                tab_line[col].1 = Some(accent);
            }
        }
        for (c, fg, _) in tab_line.iter_mut() {
            if *c == '→' {
                *fg = Some(accent);
            }
        }
    }

    // Lines 21-24: each line is "{key}  {label}" centered independently.
    // Search for the key chars directly in the rich line (char-indexed) to avoid
    // the byte-vs-char offset bug that byte-based string search would introduce.
    let hint_keys = ["r", "Enter", "Ctrl+S", "Esc"];
    for (i, &line_idx) in [21usize, 22, 23, 24].iter().enumerate() {
        if line_idx >= rich.len() {
            continue;
        }
        let key_chars: Vec<char> = hint_keys[i].chars().collect();
        let key_len = key_chars.len();
        let line = &mut rich[line_idx];
        let pos = (0..line.len().saturating_sub(key_len.saturating_sub(1))).find(|&j| {
            key_chars
                .iter()
                .enumerate()
                .all(|(k, &c)| line.get(j + k).map(|(ch, _, _)| *ch == c).unwrap_or(false))
        });
        if let Some(pos) = pos {
            for col in pos..pos + key_len {
                if col < line.len() {
                    line[col].1 = Some(accent);
                }
            }
        }
    }

    // L bar (lightness slider): dark-to-light gradient at current chroma and hue.
    let l_cursor = (oklch.l * (TRACK_LEN - 1) as f32).round() as usize;
    if rich.len() > rows::LIGHTNESS_SLIDER {
        let l_start = CONTENT_OFFSET + 7; // centered offset
        for i in 0..TRACK_LEN {
            let col = l_start + i;
            if col < rich[rows::LIGHTNESS_SLIDER].len() {
                let l = i as f32 / (TRACK_LEN - 1) as f32;
                let color = oklch_to_srgb(l, oklch.c.min(0.15), oklch.h);
                if i == l_cursor {
                    rich[rows::LIGHTNESS_SLIDER][col] = ('▓', Some(text_primary), Some(color));
                } else {
                    rich[rows::LIGHTNESS_SLIDER][col] = ('░', Some(color), Some(panel_bg));
                }
            }
        }
    }

    // C bar (chroma slider): gray-to-vivid gradient at current lightness and hue.
    let c_frac = (oklch.c / MAX_CHROMA).clamp(0.0, 1.0);
    let c_cursor = (c_frac * (TRACK_LEN - 1) as f32).round() as usize;
    if rich.len() > rows::CHROMA_SLIDER {
        let c_start = CONTENT_OFFSET + 7;
        for i in 0..TRACK_LEN {
            let col = c_start + i;
            if col < rich[rows::CHROMA_SLIDER].len() {
                let c = (i as f32 / (TRACK_LEN - 1) as f32) * MAX_CHROMA;
                let color = oklch_to_srgb(oklch.l.max(0.4), c, oklch.h);
                if i == c_cursor {
                    rich[rows::CHROMA_SLIDER][col] = ('▓', Some(text_primary), Some(color));
                } else {
                    rich[rows::CHROMA_SLIDER][col] = ('░', Some(color), Some(panel_bg));
                }
            }
        }
    }

    // H bar (hue slider): rainbow at current lightness and chroma.
    let h_cursor = ((oklch.h / 360.0) * (TRACK_LEN - 1) as f32).round() as usize;
    if rich.len() > rows::HUE_SLIDER {
        let h_start = CONTENT_OFFSET + 7;
        for i in 0..TRACK_LEN {
            let col = h_start + i;
            if col < rich[rows::HUE_SLIDER].len() {
                let h = i as f32 / (TRACK_LEN - 1) as f32 * 360.0;
                let color = oklch_to_srgb(oklch.l.max(0.5), oklch.c.max(0.08), h);
                if i == h_cursor {
                    rich[rows::HUE_SLIDER][col] = ('▓', Some(text_primary), Some(color));
                } else {
                    rich[rows::HUE_SLIDER][col] = ('░', Some(color), Some(panel_bg));
                }
            }
        }
    }

    // Gradient strip
    if rich.len() > rows::GRADIENT_STRIP {
        for i in 0..INNER_W {
            let col = CONTENT_OFFSET + i;
            if col < rich[rows::GRADIENT_STRIP].len() {
                let t = i as f32 / (INNER_W - 1).max(1) as f32;
                let t_next = (t + 1.5 / INNER_W as f32).min(1.0);
                let fg_color = interpolate_gradient(&stops, t);
                let bg_color = interpolate_gradient(&stops, t_next);
                rich[rows::GRADIENT_STRIP][col] = ('▄', Some(fg_color), Some(bg_color));
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

    /// Total height of the overlay in characters.
    /// top_border(1) + 27 content rows + bottom_border(1) = 29
    pub const HEIGHT: usize = 29;

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
        let oklch = state.current_oklch();

        let gradient_str = "▄".repeat(INNER_W);
        let swatches_str = build_swatches_str(state.selected_color_index);
        let labels_str = build_swatch_labels_str();

        let l_active = state.selected_component == EditorComponent::Lightness;
        let c_active = state.selected_component == EditorComponent::Chroma;
        let h_active = state.selected_component == EditorComponent::Hue;

        let l_label = build_slider_label(l_active, 'L', &format!("{:.3}", oklch.l));
        let l_bar = build_slider_bar(oklch.l);
        let c_label = build_slider_label(c_active, 'C', &format!("{:.3}", oklch.c));
        let c_bar = build_slider_bar((oklch.c / MAX_CHROMA).clamp(0.0, 1.0));
        let h_label = build_slider_label(h_active, 'H', &format!("{:.1}°", oklch.h));
        let h_bar = build_slider_bar(oklch.h / 360.0);

        let first_hint_line = "← select →";
        let second_hint_line = "↑ adjust ↓";
        let tab_hint_line = "Tab  L → C → H";
        let hints = [
            ("r", "reset"),
            ("Enter", "apply"),
            ("Ctrl+S", "save palette"),
            ("Esc", "discard"),
        ];
        // Each line is centered independently as a single text block.
        let key_label_lines: Vec<String> = hints
            .iter()
            .map(|(key, label)| format!("{key}  {label}"))
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
            .add_separator() // line 6
            .add_empty() // line 7
            .add_single(l_label, TextAlignment::Center) // line 8
            .add_single(l_bar, TextAlignment::Center) // line 9
            .add_empty() // line 10
            .add_single(c_label, TextAlignment::Center) // line 11
            .add_single(c_bar, TextAlignment::Center) // line 12
            .add_empty() // line 13
            .add_single(h_label, TextAlignment::Center) // line 14
            .add_single(h_bar, TextAlignment::Center) // line 15
            .add_empty() // line 16
            .add_separator() // line 17
            .add_single(first_hint_line.to_string(), TextAlignment::Center) // line 18
            .add_single(second_hint_line.to_string(), TextAlignment::Center) // line 19
            .add_single(tab_hint_line.to_string(), TextAlignment::Center) // line 20
            .add_single(key_label_lines[0].clone(), TextAlignment::Center) // line 21
            .add_single(key_label_lines[1].clone(), TextAlignment::Center) // line 22
            .add_single(key_label_lines[2].clone(), TextAlignment::Center) // line 23
            .add_single(key_label_lines[3].clone(), TextAlignment::Center) // line 24
            .add_empty() // line 25
            .add_separator() // line 26
            .add_single(gradient_str, TextAlignment::Left) // line 27
            .build_overlay();

        overlay.rich_lines = Some(build_editor_rich_lines(
            state,
            &overlay.lines,
            oklch,
            panel_style.text_primary,
            accent,
            panel_style.bg_color,
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
    fn test_adjust_hue_updates_stored_hues() {
        let mut state = PaletteEditorState::new(&Palette::Forest);
        state.selected_color_index = 3;

        let initial_stored_hue = state.stored_hues[3];
        state.adjust_hue(45.0);

        // Stored hue should be updated exactly
        assert!(
            (state.stored_hues[3] - (initial_stored_hue + 45.0) % 360.0).abs() < 0.1,
            "Stored hue should be updated when adjusting hue"
        );

        // Current hue should be close (allowing for 8-bit RGB quantization error)
        let current_hue = state.current_oklch().h;
        let hue_diff = (current_hue - state.stored_hues[3]).abs();
        assert!(
            hue_diff < 5.0,
            "Current hue {} should be close to stored hue {} (diff={})",
            current_hue,
            state.stored_hues[3],
            hue_diff
        );
    }

    #[test]
    fn test_color_navigation() {
        let mut state = PaletteEditorState::new(&Palette::Forest);

        // Starting at ALL (index 11), next goes to 0
        state.select_next_color();
        assert_eq!(state.selected_color_index, 0);

        // Prev from 0 wraps to ALL (11)
        state.select_prev_color();
        assert_eq!(state.selected_color_index, PALETTE_COLOR_COUNT);

        // Prev from ALL (11) goes to 10
        state.select_prev_color();
        assert_eq!(state.selected_color_index, 10);
    }

    #[test]
    fn test_color_navigation_wraps_forward_through_all() {
        let mut state = PaletteEditorState::new(&Palette::Forest);
        // Starting at ALL (index 11), navigate forward through all 12 positions
        // (11 colors + 1 ALL slot) to return to ALL.
        for _ in 0..(PALETTE_COLOR_COUNT + 1) {
            state.select_next_color();
        }
        assert_eq!(state.selected_color_index, PALETTE_COLOR_COUNT); // ALL slot

        // One more wraps back to 0.
        state.select_next_color();
        assert_eq!(state.selected_color_index, 0);
    }

    #[test]
    fn test_is_all_selected() {
        let state = PaletteEditorState::new(&Palette::Forest);
        assert!(state.is_all_selected());

        let mut state2 = PaletteEditorState::new(&Palette::Forest);
        state2.selected_color_index = 0;
        assert!(!state2.is_all_selected());
    }

    #[test]
    fn test_all_selected_adjust_applies_to_all_stops() {
        let mut state = PaletteEditorState::new(&Palette::Forest);
        let original_colors = state.colors;

        state.selected_color_index = PALETTE_COLOR_COUNT; // ALL slot
        state.adjust_hue(90.0);

        assert!(state.is_modified);
        // Every stop should have changed.
        for (i, _) in original_colors.iter().enumerate().take(PALETTE_COLOR_COUNT) {
            assert_ne!(
                state.colors[i], original_colors[i],
                "Stop {} should have changed after ALL hue adjust",
                i
            );
        }
    }

    #[test]
    fn test_average_oklch_calculation() {
        let mut state = PaletteEditorState::new(&Palette::Forest);

        // Set all colors to pure red (high chroma in OKLch).
        for color in state.colors.iter_mut() {
            *color = RgbColor { r: 255, g: 0, b: 0 };
        }

        let avg = state.average_oklch();
        assert!(avg.c > 0.2, "Average chroma of pure red should be high");
        assert!(
            avg.l > 0.4,
            "Average lightness of pure red should be moderate"
        );
    }

    #[test]
    fn test_oklch_adjustment() {
        let mut state = PaletteEditorState::new(&Palette::Forest);

        // Use a mid-range color (moderate chroma) where 8-bit quantization is less severe.
        state.selected_color_index = 0;
        state.colors[0] = RgbColor {
            r: 100,
            g: 150,
            b: 80,
        };
        let original_hue = state.current_oklch().h;

        // Large shift to overcome 8-bit RGB quantization noise.
        state.adjust_hue(45.0);
        assert!(state.is_modified);

        let new_hue = state.current_oklch().h;
        let actual_shift = ((new_hue - original_hue + 540.0) % 360.0) - 180.0;
        assert!(
            (actual_shift - 45.0).abs() < 10.0,
            "hue should shift by ~45°: original={}, new={}, actual_shift={}",
            original_hue,
            new_hue,
            actual_shift
        );
    }

    #[test]
    fn test_chroma_clamping() {
        let mut state = PaletteEditorState::new(&Palette::Forest);

        // Select a single stop to avoid average-based measurement.
        state.selected_color_index = 5;
        state.adjust_chroma(-2.0);
        assert!(
            state.current_oklch().c < 0.02,
            "Chroma should clamp near 0, got {}",
            state.current_oklch().c
        );

        state.adjust_chroma(2.0);
        // After 8-bit RGB roundtrip, chroma may be slightly less than MAX_CHROMA
        // due to gamut clamping, but should be in the high range.
        assert!(
            state.current_oklch().c > 0.15,
            "Chroma should be high after large positive adjustment, got {}",
            state.current_oklch().c
        );
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
        assert_eq!(EditorComponent::Lightness.next(), EditorComponent::Chroma);
        assert_eq!(EditorComponent::Chroma.next(), EditorComponent::Hue);
        assert_eq!(EditorComponent::Hue.next(), EditorComponent::Lightness);

        assert_eq!(EditorComponent::Lightness.prev(), EditorComponent::Hue);
        assert_eq!(EditorComponent::Hue.prev(), EditorComponent::Chroma);
        assert_eq!(EditorComponent::Chroma.prev(), EditorComponent::Lightness);
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

    fn build_hex_str(rgb: RgbColor) -> String {
        format!("#{:02x}{:02x}{:02x}", rgb.r, rgb.g, rgb.b)
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

    #[test]
    fn test_chroma_zero_preserves_hue_single_color() {
        // Regression test: when chroma goes to 0 and back, hue should be preserved
        let mut state = PaletteEditorState::new(&Palette::Forest);

        // Select a single color with a non-zero hue (not grayscale)
        state.selected_color_index = 5;
        let initial_hue = state.current_oklch().h;

        // Verify we have a valid initial hue (not NaN and not near 0/red)
        assert!(
            initial_hue > 10.0,
            "Initial hue should be non-red for this test, got {}",
            initial_hue
        );

        // Reduce chroma to 0 (monochrome)
        state.adjust_chroma(-2.0);
        assert!(
            state.current_oklch().c < 0.02,
            "Chroma should be near 0, got {}",
            state.current_oklch().c
        );

        // Hue should still be preserved in stored_hues
        assert!(
            (state.stored_hues[5] - initial_hue).abs() < 1.0,
            "Stored hue should be preserved when chroma=0, stored={}, initial={}",
            state.stored_hues[5],
            initial_hue
        );

        // Increase chroma back
        state.adjust_chroma(0.1);

        // Hue should be restored, not become 0 (red)
        let restored_hue = state.current_oklch().h;
        assert!(
            restored_hue > 10.0,
            "Restored hue should not be red (0°), got {}. Hue was not preserved during chroma=0 transition!",
            restored_hue
        );
    }

    #[test]
    fn test_chroma_zero_preserves_hue_all_selected() {
        // Test the ALL selection mode with chroma=0 transition
        let mut state = PaletteEditorState::new(&Palette::Forest);

        // Store initial average hue
        let _initial_avg_hue = state.current_oklch().h;

        // Reduce chroma to 0 for all colors
        state.adjust_chroma(-2.0);

        // Verify chroma is near 0
        assert!(
            state.current_oklch().c < 0.02,
            "Average chroma should be near 0"
        );

        // Increase chroma back
        state.adjust_chroma(0.15);

        // The average hue should not have become 0 (red)
        let restored_hue = state.current_oklch().h;
        assert!(
            restored_hue > 20.0 || restored_hue < 340.0,
            "Restored average hue should not be near red (0°), got {}. Stored hues were not preserved!",
            restored_hue
        );
    }

    #[test]
    fn test_stored_hues_initialized_correctly() {
        let state = PaletteEditorState::new(&Palette::Forest);

        // Verify stored hues match the actual hues of the palette colors
        for (i, &color) in state.colors.iter().enumerate() {
            let oklch = srgb_to_oklch(color);
            let expected_hue = if oklch.h.is_nan() { 0.0 } else { oklch.h };
            assert!(
                (state.stored_hues[i] - expected_hue).abs() < 0.1,
                "Stored hue {} should match actual hue {} for color {}",
                state.stored_hues[i],
                expected_hue,
                i
            );
        }
    }

    #[test]
    fn test_reset_restores_stored_hues() {
        let mut state = PaletteEditorState::new(&Palette::Forest);
        let initial_stored_hues = state.stored_hues;

        // Modify hues
        state.adjust_hue(90.0);

        // Reset
        state.reset_to_original();

        // Stored hues should be restored
        for (i, &initial_hue) in initial_stored_hues
            .iter()
            .enumerate()
            .take(PALETTE_COLOR_COUNT)
        {
            assert!(
                (state.stored_hues[i] - initial_hue).abs() < 0.1,
                "Stored hue {} should be reset to original",
                i
            );
        }
    }
}
