//! Spike: drive overlay chrome with ratatui *widgets only* — no backend, no `Terminal`.
//!
//! ## Why this shape
//!
//! ratatui's `Terminal`/backend keeps an internal back-buffer and writes diffs to
//! stdout. Letting it own the screen alongside our hand-rolled full-frame ANSI
//! writer (`FrameBuffer::build_frame_string`) is the unsupported "two writers, one
//! screen" trap that corrupts the sim on resize/redraw.
//!
//! Instead we use ratatui purely as a *layout + widget library*: render widgets into
//! a detached [`ratatui::buffer::Buffer`] (which never touches stdout), then blit that
//! buffer's cells into our existing [`RenderedOverlay`] contract. The compositor and
//! the flicker-tuned emission path stay 100% ours. This is Option B from the spike.
//!
//! What we get for free here that is hand-rolled today:
//! - `ListState`-driven scroll offset that keeps the selection visible (replaces
//!   `ConfigBrowserOverlay::config_browser_window`).
//! - `Scrollbar` widget.
//! - `Block` borders + title + inner-rect layout (replaces `PanelBuilder` border math).

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Color;
use ratatui::widgets::{List, ListItem, ListState, StatefulWidget};

use crate::config_manager::NamedProfile;
use crate::render::palette::RgbColor;
use crate::render::panel::{Padding, PanelBuilder, RenderedOverlay, RichCell, TextAlignment};
use crate::render::theme::PanelStyle;

/// Map a ratatui `Color` to our `RgbColor`. `Reset` (and anything we don't paint
/// explicitly) becomes `None` so the underlying sim cell shows through.
fn color_to_rgb(c: Color) -> Option<RgbColor> {
    match c {
        Color::Rgb(r, g, b) => Some(RgbColor::new(r, g, b)),
        Color::Reset => None,
        // We style every widget with explicit Rgb below, so named colors are only a
        // safety net. Map the basics; leave the rest transparent.
        Color::Black => Some(RgbColor::new(0, 0, 0)),
        Color::White => Some(RgbColor::new(0xEE, 0xEE, 0xEE)),
        Color::Gray => Some(RgbColor::new(0x88, 0x88, 0x88)),
        Color::DarkGray => Some(RgbColor::new(0x44, 0x44, 0x44)),
        _ => None,
    }
}

/// Convert a fully-rendered detached ratatui `Buffer` into our overlay contract.
///
/// Each buffer cell becomes a [`RichCell`] `(char, fg, bg)`. We emit both `lines`
/// (plain text, first char of each cell's symbol) and `rich_lines` (per-cell colour),
/// matching what the renderer already composites via `draw_text_overlay` +
/// `draw_rich_overlay`.
///
/// Note: wide glyphs occupy two ratatui cells (the second has an empty symbol); we map
/// one `char` per cell, so wide chars in overlay text would shift by one. Overlay
/// chrome is ASCII/box-drawing today, so this is fine for the spike — flagged for a
/// real migration.
pub fn buffer_to_overlay(buf: &Buffer) -> RenderedOverlay {
    let area = buf.area;
    let mut lines: Vec<String> = Vec::with_capacity(area.height as usize);
    let mut rich: Vec<Vec<RichCell>> = Vec::with_capacity(area.height as usize);

    for y in 0..area.height {
        let mut line = String::with_capacity(area.width as usize);
        let mut rich_row: Vec<RichCell> = Vec::with_capacity(area.width as usize);
        for x in 0..area.width {
            let cell = &buf[(x, y)];
            let ch = cell.symbol().chars().next().unwrap_or(' ');
            line.push(ch);
            rich_row.push((ch, color_to_rgb(cell.fg), color_to_rgb(cell.bg)));
        }
        lines.push(line);
        rich.push(rich_row);
    }

    RenderedOverlay {
        lines,
        title_box: None,
        rich_lines: Some(rich),
    }
}

/// Inner content width (`56 - 2 border - 2*2 padding`).
const BROWSER_CONTENT_WIDTH: usize = 50;
/// Max config rows shown at once.
const BROWSER_MAX_VISIBLE: usize = 9;

/// Format one config row body (no selection marker — the caller adds that).
fn config_row_text(index: usize, c: &NamedProfile) -> String {
    let palette = c
        .overrides
        .palette
        .as_ref()
        .map(|p| p.name())
        .unwrap_or("?");
    let pop = c.overrides.population.unwrap_or(0) / 1000;
    format!("{} {} - {} - {}k agents", index + 1, c.name, palette, pop)
}

/// Index of the first visible row so `selected` stays on-screen, computed by ratatui's
/// `ListState` — the same "keep selection visible" logic its `List` widget uses.
///
/// This replaces the hand-rolled scroll-window arithmetic
/// (`ConfigBrowserOverlay::config_browser_window`) and fixes the palette load dialog's
/// `take(N)` truncation, which hid selections past the window. We render `total` blank
/// items into a detached 1×`max_visible` buffer and read back the settled offset.
pub fn list_scroll_offset(total: usize, selected: usize, max_visible: usize) -> usize {
    if total <= max_visible || max_visible == 0 {
        return 0;
    }
    let area = Rect::new(0, 0, 1, max_visible as u16);
    let mut buf = Buffer::empty(area);
    let items: Vec<ListItem> = (0..total).map(|_| ListItem::new(" ")).collect();
    let mut state = ListState::default();
    state.select(Some(selected.min(total - 1)));
    StatefulWidget::render(List::new(items), area, &mut buf, &mut state);
    state.offset()
}

/// Stamp a block caret onto a built dialog overlay's editable field. Locates `label`
/// in the overlay lines and highlights the cell at `label_end + cursor`, so callers keep
/// their own `PanelBuilder` chrome and just delegate caret placement. `field_width`
/// clamps the cursor. Caret = `accent_active` background, `bg_color` foreground.
pub fn stamp_caret(
    overlay: &mut RenderedOverlay,
    label: &str,
    field_width: usize,
    cursor: usize,
    style: &PanelStyle,
) {
    let cursor = cursor.min(field_width.saturating_sub(1));
    let mut rich: Vec<Vec<RichCell>> = overlay
        .lines
        .iter()
        .map(|l| l.chars().map(|c| (c, None, None)).collect())
        .collect();
    if let Some((row_idx, row)) = overlay
        .lines
        .iter()
        .enumerate()
        .find(|(_, l)| l.contains(label))
    {
        // `find` returns a byte index; labels here are ASCII so it equals the char index.
        let label_start = row.find(label).unwrap_or(0);
        let caret_col = label_start + label.chars().count() + cursor;
        if let Some(cell) = rich.get_mut(row_idx).and_then(|r| r.get_mut(caret_col)) {
            cell.1 = Some(style.bg_color);
            cell.2 = Some(style.accent_active);
        }
    }
    overlay.rich_lines = Some(rich);
}

/// Option-B config browser: **chrome is the existing `PanelBuilder`** (title box,
/// block border, footer — pixel-identical to the current app), while the scroll window
/// is driven by [`list_scroll_offset`] (ratatui `ListState`) instead of hand-rolled math.
pub fn build_config_browser(configs: &[NamedProfile], selected: usize) -> RenderedOverlay {
    use TextAlignment::Left;

    let mut builder = PanelBuilder::new(BROWSER_CONTENT_WIDTH, None)
        .with_padding(Padding::new(1, 1, 2, 2))
        .with_title("SAVED CONFIGURATIONS")
        .with_title_box();

    if configs.is_empty() {
        return builder
            .add_empty()
            .add_single("No saved configurations", Left)
            .add_empty()
            .add_single("Press Ctrl+S to save current settings", Left)
            .add_empty()
            .add_single("Esc: Cancel", Left)
            .build_overlay();
    }

    let total = configs.len();
    let selected = selected.min(total - 1);
    let start = list_scroll_offset(total, selected, BROWSER_MAX_VISIBLE);
    let end = (start + BROWSER_MAX_VISIBLE).min(total);

    if start > 0 {
        builder = builder.add_single(format!("▲ {} above", start), Left);
    } else {
        builder = builder.add_empty();
    }
    for (i, c) in configs.iter().enumerate().skip(start).take(end - start) {
        let marker = if i == selected { "›" } else { " " };
        builder = builder.add_single(format!("{marker}{}", config_row_text(i, c)), Left);
    }
    if end < total {
        builder = builder.add_single(format!("▼ {} below", total - end), Left);
    } else {
        builder = builder.add_empty();
    }

    builder
        .add_empty()
        .add_single("↑/↓: Navigate  Enter: Load  Del: Delete", Left)
        .add_single("Esc: Cancel", Left)
        .build_overlay()
}

/// Content width of the save dialog (`38 - 2 border - 2*1 padding`).
const SAVE_CONTENT_WIDTH: usize = 34;
/// Visible width of the editable name field.
const SAVE_FIELD_WIDTH: usize = 25;

/// Option-B config-save dialog: PanelBuilder chrome plus a `tui_input`-backed editable
/// field with a block caret ([`stamp_caret`]). `value`/`cursor` come from
/// `tui_input::Input`, enabling mid-string insert/delete and Home/End/arrows — the
/// hand-rolled version only did append + backspace with the cursor pinned to the end.
pub fn build_config_save(value: &str, cursor: usize, style: &PanelStyle) -> RenderedOverlay {
    use TextAlignment::Left;

    const LABEL: &str = "Name: ";
    let name_line = format!("{LABEL}{value:<SAVE_FIELD_WIDTH$}");

    let mut overlay = PanelBuilder::new(SAVE_CONTENT_WIDTH, None)
        .with_padding(Padding::new(0, 0, 1, 1))
        .with_title("SAVE CONFIGURATION")
        .with_title_box()
        .add_empty()
        .add_empty()
        .add_single(&name_line, Left)
        .add_empty()
        .add_single("Enter: Save    Esc: Cancel", Left)
        .build_overlay();

    stamp_caret(&mut overlay, LABEL, SAVE_FIELD_WIDTH, cursor, style);
    overlay
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::profile_overrides::ProfileOverrides;

    fn profile(name: &str) -> NamedProfile {
        NamedProfile {
            name: name.to_string(),
            description: None,
            overrides: ProfileOverrides::default(),
        }
    }

    /// All overlay lines share the panel width (PanelBuilder invariant), and the
    /// floating title box carries the title.
    #[test]
    fn chrome_dims_and_title_box() {
        let configs = vec![profile("alpha"), profile("beta")];
        let overlay = build_config_browser(&configs, 0);
        let w = overlay.lines[0].chars().count();
        assert_eq!(w, BROWSER_WIDTH_TOTAL);
        assert!(overlay.lines.iter().all(|l| l.chars().count() == w));
        let tb = overlay.title_box.expect("title box present");
        assert!(tb.lines.iter().any(|l| l.contains("SAVED CONFIGURATIONS")));
    }

    /// Row format matches the app: `›{n} {name} - {palette} - {pop}k agents`.
    #[test]
    fn row_format_and_marker_match_app() {
        let configs = vec![profile("alpha"), profile("beta")];
        let overlay = build_config_browser(&configs, 0);
        let text = overlay.lines.join("\n");
        assert!(
            text.contains("›1 alpha - "),
            "selected row format wrong:\n{text}"
        );
        assert!(
            text.contains(" 2 beta - "),
            "second row format wrong:\n{text}"
        );
        // Footer hints present (B1 had dropped these).
        assert!(
            text.contains("↑/↓: Navigate"),
            "nav footer missing:\n{text}"
        );
        assert!(text.contains("Esc: Cancel"), "esc footer missing:\n{text}");
    }

    #[test]
    fn empty_state_renders_hint() {
        let overlay = build_config_browser(&[], 0);
        let text = overlay.lines.join("\n");
        assert!(
            text.contains("No saved configurations"),
            "empty hint missing:\n{text}"
        );
    }

    /// The win over hand-rolled code: `ListState` keeps the selection visible and the
    /// `▲ N above` indicator is derived from its offset — no `config_browser_window`.
    #[test]
    fn selection_stays_visible_when_scrolled_past_window() {
        let configs: Vec<_> = (0..30).map(|i| profile(&format!("cfg{i:02}"))).collect();
        let overlay = build_config_browser(&configs, 29);
        let text = overlay.lines.join("\n");
        assert!(
            text.contains("cfg29"),
            "selected (last) entry scrolled off:\n{text}"
        );
        assert!(text.contains('›'), "highlight marker missing:\n{text}");
        assert!(
            text.contains("▲ "),
            "above-scroll indicator missing:\n{text}"
        );
        assert!(
            !text.contains("▼ "),
            "should be at bottom, no below indicator:\n{text}"
        );
    }

    // The general detached-buffer adapter still round-trips dims (used for future
    // full-widget overlays like a ratatui-rendered palette grid).
    #[test]
    fn buffer_adapter_roundtrips_dims() {
        let area = Rect::new(0, 0, 10, 3);
        let buf = Buffer::empty(area);
        let overlay = buffer_to_overlay(&buf);
        assert_eq!(overlay.lines.len(), 3);
        assert!(overlay.lines.iter().all(|l| l.chars().count() == 10));
    }

    const BROWSER_WIDTH_TOTAL: usize = 56;

    /// The caret cell tracks the cursor mid-string (the win over append-only editing).
    #[test]
    fn save_dialog_caret_follows_cursor() {
        let style = crate::render::theme::PanelStyle::default();
        let value = "myconfig";
        // Cursor in the middle of the string.
        let overlay = build_config_save(value, 3, &style);
        let text = overlay.lines.join("\n");
        assert!(
            text.contains("Name: myconfig"),
            "field text missing:\n{text}"
        );

        let rich = overlay.rich_lines.expect("caret needs rich_lines");
        // Exactly one cell has a caret background.
        let caret_cells: Vec<(usize, usize)> = rich
            .iter()
            .enumerate()
            .flat_map(|(r, row)| {
                row.iter()
                    .enumerate()
                    .filter(|(_, c)| c.2 == Some(style.accent_active))
                    .map(move |(col, _)| (r, col))
            })
            .collect();
        assert_eq!(caret_cells.len(), 1, "expected exactly one caret cell");

        // Caret column = label offset + "Name: ".len() + cursor(3).
        let (row, col) = caret_cells[0];
        let label_col = rich[row]
            .iter()
            .map(|c| c.0)
            .collect::<String>()
            .find("Name: ")
            .unwrap();
        assert_eq!(col, label_col + "Name: ".chars().count() + 3);
    }
}
