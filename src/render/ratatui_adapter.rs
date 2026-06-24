//! Config browser and save-dialog overlay builders.
//!
//! Builds [`RenderedOverlay`] instances for the config browser and save dialog using
//! the hand-rolled [`PanelBuilder`] chrome (title box, block border, footer).
//! No ratatui dependency — scroll-window math is done by [`scroll_start`], and
//! the editable field caret is handled by [`stamp_caret`] + `tui-input`.

use crate::config_manager::NamedProfile;
use crate::render::panel::{
    footer_hints, Padding, PanelBuilder, RenderedOverlay, RichCell, TextAlignment,
};
use crate::render::theme::PanelStyle;

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

/// Compute the first-visible row index so `selected` stays within the visible window.
///
/// Mirrors `ConfigBrowserOverlay::config_browser_window`: pins the selection to the
/// bottom of the `[start, start + max_visible)` window and clamps so the window never
/// runs past the end of the list. Replaces `list_scroll_offset` for all callers that
/// only need the start index (no ratatui).
pub fn scroll_start(total: usize, selected: usize, max_visible: usize) -> usize {
    if total <= max_visible || max_visible == 0 {
        return 0;
    }
    let sel = selected.min(total - 1);
    sel.saturating_sub(max_visible - 1).min(total - max_visible)
}

/// Option-B config browser: **chrome is the existing `PanelBuilder`** (title box,
/// block border, footer — pixel-identical to the current app), with hand-rolled
/// scroll-window math keeping the selection visible.
///
/// `active_name` — when `Some(name)`, the row whose `config.name == name` gets a `▶`
/// prefix to mark it as the currently-loaded config (in addition to the `›` cursor).
pub fn build_config_browser(
    configs: &[NamedProfile],
    selected: usize,
    active_name: Option<&str>,
) -> RenderedOverlay {
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
            .add_single(footer_hints(&[("esc", "cancel")]), Left)
            .build_overlay();
    }

    let total = configs.len();
    let selected = selected.min(total - 1);
    let start = scroll_start(total, selected, BROWSER_MAX_VISIBLE);
    let end = (start + BROWSER_MAX_VISIBLE).min(total);

    if start > 0 {
        builder = builder.add_single(format!("▲ {} above", start), Left);
    } else {
        builder = builder.add_empty();
    }
    for (i, c) in configs.iter().enumerate().skip(start).take(end - start) {
        let cursor = if i == selected { "›" } else { " " };
        let active = if active_name == Some(c.name.as_str()) {
            "▶"
        } else {
            " "
        };
        builder = builder.add_single(format!("{cursor}{active}{}", config_row_text(i, c)), Left);
    }
    if end < total {
        builder = builder.add_single(format!("▼ {} below", total - end), Left);
    } else {
        builder = builder.add_empty();
    }

    builder
        .add_empty()
        .add_single(
            footer_hints(&[
                ("↑↓", "navigate"),
                ("↵", "load"),
                ("del", "delete"),
                ("esc", "cancel"),
            ]),
            Left,
        )
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
        .add_single(footer_hints(&[("↵", "save"), ("esc", "cancel")]), Left)
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
        let overlay = build_config_browser(&configs, 0, None);
        let w = overlay.lines[0].chars().count();
        assert_eq!(w, BROWSER_WIDTH_TOTAL);
        assert!(overlay.lines.iter().all(|l| l.chars().count() == w));
        let tb = overlay.title_box.expect("title box present");
        assert!(tb.lines.iter().any(|l| l.contains("SAVED CONFIGURATIONS")));
    }

    /// Row format matches the app: `{cursor}{active}{n} {name} - {palette} - {pop}k agents`.
    /// Exact prefix for selected non-active row is `"› "` (after border + left padding).
    #[test]
    fn row_format_and_marker_match_app() {
        let configs = vec![profile("alpha"), profile("beta")];
        let overlay = build_config_browser(&configs, 0, None);
        let text = overlay.lines.join("\n");
        // Panel wraps lines with border (█) + left padding (2 spaces), so skip 3 chars.
        // Content format: {cursor}{active}{index} {name} ...
        let alpha_line = text.lines().find(|l| l.contains("alpha")).unwrap_or("");
        // Extract content after border and left padding: chars 3..
        let alpha_content = alpha_line.chars().skip(3).collect::<String>();
        assert!(
            alpha_content.starts_with("› 1 "),
            "selected alpha row must start with '› 1 ' (after border+padding):\n{alpha_line}\ncontent: {alpha_content}"
        );
        // Second row (not selected) should have space in cursor slot, then active slot, then index
        let beta_line = text.lines().find(|l| l.contains("beta")).unwrap_or("");
        let beta_content = beta_line.chars().skip(3).collect::<String>();
        assert!(
            beta_content.starts_with("  2 "),
            "non-selected beta row must start with '  2 ' (space+space+index, after border+padding):\n{beta_line}\ncontent: {beta_content}"
        );
        // Footer hints present (B1 had dropped these). Unified instrument-voice grammar.
        assert!(text.contains("↑↓ navigate"), "nav footer missing:\n{text}");
        assert!(text.contains("esc cancel"), "esc footer missing:\n{text}");
    }

    #[test]
    fn empty_state_renders_hint() {
        let overlay = build_config_browser(&[], 0, None);
        let text = overlay.lines.join("\n");
        assert!(
            text.contains("No saved configurations"),
            "empty hint missing:\n{text}"
        );
    }

    /// Hand-rolled scroll_start keeps the selection visible — no ratatui ListState.
    #[test]
    fn selection_stays_visible_when_scrolled_past_window() {
        let configs: Vec<_> = (0..30).map(|i| profile(&format!("cfg{i:02}"))).collect();
        let overlay = build_config_browser(&configs, 29, None);
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

    /// The `▶` marker appears on the row whose name matches `active_name`.
    #[test]
    fn active_marker_shown_for_active_config() {
        let configs = vec![profile("alpha"), profile("beta"), profile("gamma")];
        // beta is the active config, alpha is selected (cursor).
        let overlay = build_config_browser(&configs, 0, Some("beta"));
        let text = overlay.lines.join("\n");
        // Panel wraps lines with border (█) + left padding (2 spaces), so skip 3 chars.
        // Content format: {cursor}{active}{index} {name} ...

        // alpha row: selected but NOT active — prefix should be "› " (cursor + space for active slot).
        let alpha_line = text.lines().find(|l| l.contains("alpha")).unwrap_or("");
        let alpha_content = alpha_line.chars().skip(3).collect::<String>();
        assert!(
            alpha_content.starts_with("› "),
            "alpha (selected, not active) row prefix must be '› ' (after border+padding):\n{alpha_line}\ncontent: {alpha_content}"
        );
        assert!(
            !alpha_content.contains("▶"),
            "alpha content should not contain active marker:\n{alpha_content}"
        );

        // beta row: NOT selected but IS active — prefix should be " ▶" (space for cursor + active marker).
        let beta_line = text.lines().find(|l| l.contains("beta")).unwrap_or("");
        let beta_content = beta_line.chars().skip(3).collect::<String>();
        assert!(
            beta_content.starts_with(" ▶"),
            "beta (not selected, active) row prefix must be ' ▶' (after border+padding):\n{beta_line}\ncontent: {beta_content}"
        );

        // gamma row: neither selected nor active — prefix should be "  " (two spaces).
        let gamma_line = text.lines().find(|l| l.contains("gamma")).unwrap_or("");
        let gamma_content = gamma_line.chars().skip(3).collect::<String>();
        assert!(
            gamma_content.starts_with("  "),
            "gamma (not selected, not active) row prefix must be '  ' (after border+padding):\n{gamma_line}\ncontent: {gamma_content}"
        );
        assert!(
            !gamma_content.contains("▶"),
            "gamma should not have active marker:\n{gamma_content}"
        );
    }

    /// When no `active_name` is given, `▶` never appears.
    #[test]
    fn no_active_marker_when_active_name_is_none() {
        let configs: Vec<_> = (0..5).map(|i| profile(&format!("cfg{i}"))).collect();
        let overlay = build_config_browser(&configs, 2, None);
        let text = overlay.lines.join("\n");
        assert!(
            !text.contains("▶"),
            "should be no active marker when active_name=None:\n{text}"
        );
    }

    /// `scroll_start` unit tests: matches `ConfigBrowserOverlay::config_browser_window` behaviour.
    #[test]
    fn scroll_start_matches_overlay_window_helper() {
        // total <= max_visible: always 0.
        assert_eq!(scroll_start(5, 0, 9), 0);
        assert_eq!(scroll_start(5, 4, 9), 0);
        // Selection within first window: anchored at top.
        assert_eq!(scroll_start(15, 0, 9), 0);
        assert_eq!(scroll_start(15, 8, 9), 0);
        // Selection past first window.
        assert_eq!(scroll_start(15, 9, 9), 1);
        assert_eq!(scroll_start(15, 11, 9), 3);
        // Selection at the end: clamped.
        assert_eq!(scroll_start(15, 14, 9), 6);
        // Out-of-range selection clamped.
        assert_eq!(scroll_start(15, 99, 9), 6);
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
