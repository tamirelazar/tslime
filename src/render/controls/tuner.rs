//! Tuner depth of the Controls Instrument: ambient bottom-docked edge strip.
//!
//! The Tuner is the "play" surface — intentionally off-house-style (no PanelBuilder chrome,
//! no title box). It renders as a hand-built [`RichCell`] grid with a dim-solid bg matte
//! so it visually floats above the art without PanelBuilder borders.
//!
//! Layout (rows from top):
//! ```text
//!   row 0:  ▔▔▔▔…  (dim top border line)
//!   row 1:  RECENT  <label sparkline value>  …  (recently-touched params)
//!   row 2:  (blank strip row)
//!   row 3:  ← focused param label ─── heatmap slider ─── value ●→  (or kind-specific widget)
//!   row 4:  ←→ tune · ↑↓ pick · fades when idle
//!   row 5:  (bottom margin / status-bar clearance)
//! ```
//!
//! The caller positions the overlay at the bottom of the terminal and wires depth
//! dispatch in `build_controls`.

use crate::render::controls::registry::ParamKind;
use crate::render::controls::value::heatmap_slider;
use crate::render::controls::{ParamState, ParamView};
use crate::render::palette::RgbColor;
use crate::render::panel::{RenderedOverlay, RichCell};
use crate::render::theme::PanelStyle;
use crate::render::widgets::RowBuf;

// ── Layout constants ──────────────────────────────────────────────────────────

/// Total strip width in characters.
///
/// 80 is a safe minimum; the runner may choose wider values.
const STRIP_W: usize = 80;

/// Number of rows in the tuner strip (including top border + bottom margin).
const STRIP_H: usize = 6;

/// Width of the heatmap slider in the focused-param row.
const SLIDER_W: usize = 44;

/// Max number of recent params shown in the RECENT row.
const MAX_RECENT: usize = 4;

/// Reserved columns for RECENT label + gap.
const RECENT_LABEL_W: usize = 8; // "RECENT  "

/// Columns reserved per recent-param slot (label + sparkline + value).
const RECENT_SLOT_W: usize = 18;

// ── Kind-aware focused-param widget ──────────────────────────────────────────

/// Render the focused-param row into `row`.
///
/// Layout (within the strip):
/// ```text
///  col 2  : "[key] Label"           ← accent
///  col 22 : kind widget             ← heatmap/enum/toggle/action/read-only
///  right  : value text              ← accent (or muted for CLI/Display)
/// ```
fn render_focused(
    row: &mut RowBuf,
    w: usize,
    pv: &ParamView,
    accent: RgbColor,
    style: &PanelStyle,
    truecolor: bool,
) {
    // Left: "[key] Label", truncated to 18 chars.
    let head = format!("[{}] {}", pv.desc.key_hint, pv.desc.label);
    let head: String = head.chars().take(18).collect();
    row.put(2, &head, Some(accent), None);

    // Right: value text
    let val: String = pv.value_text.chars().take(12).collect();
    let val_start = w.saturating_sub(val.len() + 2);
    let val_col = match pv.state {
        ParamState::Cli | ParamState::Display => style.muted,
        _ => accent,
    };
    row.put(val_start, &val, Some(val_col), None);

    // Middle: kind-aware widget
    let widget_start = 22usize;
    let widget_end = val_start.saturating_sub(2);
    if widget_end <= widget_start {
        return;
    }
    let widget_w = (widget_end - widget_start).min(SLIDER_W);

    match pv.desc.kind {
        ParamKind::Numeric => {
            if let (Some(ratio), Some(def_ratio)) = (pv.ratio, pv.def_ratio) {
                let cells =
                    heatmap_slider(ratio, def_ratio, widget_w, truecolor, accent, style.muted);
                row.put_cells(widget_start, &cells, None);
            } else {
                // No ratio: show dimmed value hint
                row.put(widget_start, "─── (no range) ───", Some(style.muted), None);
            }
        }
        ParamKind::Enum => {
            let s = format!("‹ {} ›", pv.value_text);
            let s: String = s.chars().take(widget_w).collect();
            row.put(widget_start, &s, Some(accent), None);
        }
        ParamKind::Toggle => {
            let is_on = pv.value_text.eq_ignore_ascii_case("on")
                || pv.value_text == "true"
                || pv.value_text == "1";
            let (pill, col) = if is_on {
                ("[ ON  ]", accent)
            } else {
                ("[ OFF ]", style.muted)
            };
            row.put(widget_start, pill, Some(col), None);
        }
        ParamKind::Action => {
            row.put(widget_start, "↵ run", Some(accent), None);
        }
        ParamKind::CliReadonly | ParamKind::Display => {
            // Value is shown on the right; the widget slot just flags read-only.
            row.put(widget_start, "(read-only)", Some(style.muted), None);
        }
    }
}

// ── Public builder ────────────────────────────────────────────────────────────

/// Build the Tuner ambient edge-strip overlay.
///
/// - `focused`: the currently focused parameter (shown as a kind-aware widget in the
///   focused row).
/// - `recent`: recently-touched parameters shown in the RECENT ambient row. Each is
///   displayed as `label  value`; [`ParamView`] carries no history slice, so the row
///   renders a single-sample sparkline rather than a rolling history.
/// - `style`: the active [`PanelStyle`] for colour selection.
/// - `accent`: caller-supplied accent colour from the active simulation palette.
/// - `truecolor`: when `true`, the heatmap slider uses a green→red gradient; when
///   `false`, it falls back to a solid accent colour for 256-colour terminals.
/// - `width`: terminal width in columns; the strip fills this width (clamped to at
///   least [`STRIP_W`] so layout invariants hold).
///
/// Returns a [`RenderedOverlay`] with:
/// - `title_box = None` (no chrome — the Tuner is chrome-light by design).
/// - `rich_lines` populated with the per-cell bg matte and fg colours.
/// - `lines` carrying the plain-text representation (for non-rich renderers).
pub fn build_tuner(
    focused: &ParamView,
    recent: &[ParamView],
    style: &PanelStyle,
    accent: RgbColor,
    truecolor: bool,
    width: usize,
) -> RenderedOverlay {
    let w = width.max(STRIP_W);
    let mut bufs: Vec<RowBuf> = Vec::with_capacity(STRIP_H);

    // ── row 0: dim top-border line (▔ repeated across width) ────────────────
    let mut border = RowBuf::new_matte(w, style.status_bar_bg);
    for c in 0..w {
        border.put(c, "▔", Some(style.border_color), None);
    }
    bufs.push(border);

    // ── row 1: RECENT ambient row ─────────────────────────────────────────────
    // ParamView has no history slice, so each entry renders a single-sample sparkline.
    {
        let mut rrow = RowBuf::new_matte(w, style.status_bar_bg);
        rrow.put(2, "RECENT", Some(style.muted), None);
        let sources: Vec<&ParamView> = if recent.is_empty() {
            // Fall back to the focused param so the row is never empty.
            vec![focused]
        } else {
            recent.iter().take(MAX_RECENT).collect()
        };
        let mut rx = RECENT_LABEL_W + 2;
        for pv in sources {
            if rx + RECENT_SLOT_W > w {
                break;
            }
            // Label (up to 10 chars)
            let lbl: String = pv.desc.label.chars().take(10).collect();
            rrow.put(rx, &lbl, Some(style.muted), None);
            rx += lbl.chars().count() + 1;

            // Sparkline-of-one from current ratio (or a block char for non-numeric).
            let spark = if let Some(r) = pv.ratio {
                crate::render::controls::value::sparkline(&[r])
            } else {
                "━".to_string()
            };
            let spark_col = match pv.state {
                ParamState::Modified => style.accent_modified,
                ParamState::Cli => style.cli_color,
                _ => style.muted,
            };
            rrow.put(rx, &spark, Some(spark_col), None);
            rx += spark.chars().count() + 1;

            // Value text (up to 8 chars)
            let val: String = pv.value_text.chars().take(8).collect();
            rrow.put(rx, &val, Some(spark_col), None);
            rx += val.chars().count() + 2;
        }
        bufs.push(rrow);
    }

    // ── row 2: blank strip row ────────────────────────────────────────────────
    bufs.push(RowBuf::new_matte(w, style.status_bar_bg));

    // ── row 3: focused-param kind-aware widget ────────────────────────────────
    {
        let mut frow = RowBuf::new_matte(w, style.status_bar_bg);
        render_focused(&mut frow, w, focused, accent, style, truecolor);
        bufs.push(frow);
    }

    // ── row 4: hint line ──────────────────────────────────────────────────────
    {
        let mut hint = RowBuf::new_matte(w, style.status_bar_bg);
        hint.put(
            2,
            "←→ tune · ↑↓ pick · fades when idle",
            Some(style.muted),
            None,
        );
        bufs.push(hint);
    }

    // ── row 5: bottom margin (clearance for status-bar row) ──────────────────
    bufs.push(RowBuf::new_matte(w, style.status_bar_bg));

    // ── Assemble ──────────────────────────────────────────────────────────────
    let lines: Vec<String> = bufs.iter().map(|b| b.text()).collect();
    let rich_lines: Vec<Vec<RichCell>> = bufs.into_iter().map(|b| b.into_rich()).collect();

    RenderedOverlay {
        lines,
        title_box: None,
        rich_lines: Some(rich_lines),
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::render::controls::registry::{ParamDesc, ParamId, ParamKind};

    /// Numeric fixture: ratio Some(0.6), def_ratio Some(0.3).
    fn numeric_fixture() -> ParamView {
        ParamView {
            desc: ParamDesc {
                id: ParamId::SensorAngle,
                key_hint: "A/a",
                label: "Sensor Angle",
                kind: ParamKind::Numeric,
            },
            value_text: "54.0°".to_string(),
            ratio: Some(0.6),
            def_ratio: Some(0.3),
            state: ParamState::Modified,
        }
    }

    #[test]
    fn tuner_emits_rich_grid_with_matte() {
        let mut s = crate::render::theme::SLIME_DARK;
        s.status_bar_bg = RgbColor::new(1, 2, 3);
        let ov = build_tuner(
            &numeric_fixture(),
            &[],
            &s,
            crate::render::palette::RgbColor { r: 0, g: 200, b: 0 },
            true,
            STRIP_W,
        );
        let rich = ov.rich_lines.expect("tuner needs rich_lines");
        assert!(
            rich.iter()
                .flatten()
                .all(|(_, _, bg)| *bg == Some(s.status_bar_bg)),
            "matte must use the supplied theme's status-bar background"
        );
        assert!(
            ov.title_box.is_none(),
            "tuner is chrome-light, no title box"
        );
    }

    #[test]
    fn tuner_has_correct_strip_height() {
        let s = crate::render::theme::SLIME_DARK;
        let ov = build_tuner(
            &numeric_fixture(),
            &[],
            &s,
            crate::render::palette::RgbColor { r: 0, g: 200, b: 0 },
            true,
            STRIP_W,
        );
        assert_eq!(
            ov.lines.len(),
            STRIP_H,
            "tuner must have exactly STRIP_H rows"
        );
    }

    #[test]
    fn tuner_numeric_focused_has_slider_chars() {
        let s = crate::render::theme::SLIME_DARK;
        let ov = build_tuner(
            &numeric_fixture(),
            &[],
            &s,
            crate::render::palette::RgbColor { r: 0, g: 200, b: 0 },
            false, // 256-colour — still produces ━/● chars
            STRIP_W,
        );
        let combined: String = ov.lines.concat();
        assert!(
            combined.contains('●') || combined.contains('━'),
            "numeric focused param should render heatmap slider characters"
        );
    }

    #[test]
    fn tuner_enum_focused_shows_chevrons() {
        let s = crate::render::theme::SLIME_DARK;
        let pv = ParamView {
            desc: ParamDesc {
                id: ParamId::DiffusionKernel,
                key_hint: "K",
                label: "Diffusion",
                kind: ParamKind::Enum,
            },
            value_text: "Gaussian".to_string(),
            ratio: None,
            def_ratio: None,
            state: ParamState::Default,
        };
        let ov = build_tuner(
            &pv,
            &[],
            &s,
            crate::render::palette::RgbColor { r: 0, g: 200, b: 0 },
            true,
            STRIP_W,
        );
        let combined: String = ov.lines.concat();
        assert!(
            combined.contains('‹'),
            "enum focused param should render ‹ value ›"
        );
    }

    #[test]
    fn tuner_toggle_on_shows_pill() {
        let s = crate::render::theme::SLIME_DARK;
        let pv = ParamView {
            desc: ParamDesc {
                id: ParamId::Invert,
                key_hint: "X",
                label: "Invert",
                kind: ParamKind::Toggle,
            },
            value_text: "on".to_string(),
            ratio: None,
            def_ratio: None,
            state: ParamState::Default,
        };
        let ov = build_tuner(
            &pv,
            &[],
            &s,
            crate::render::palette::RgbColor { r: 0, g: 200, b: 0 },
            true,
            STRIP_W,
        );
        let combined: String = ov.lines.concat();
        assert!(combined.contains("[ ON  ]"), "toggle 'on' should show pill");
    }

    #[test]
    fn tuner_action_focused_shows_run() {
        let s = crate::render::theme::SLIME_DARK;
        let pv = ParamView {
            desc: ParamDesc {
                id: ParamId::Reset,
                key_hint: "0",
                label: "Reset",
                kind: ParamKind::Action,
            },
            value_text: String::new(),
            ratio: None,
            def_ratio: None,
            state: ParamState::Default,
        };
        let ov = build_tuner(
            &pv,
            &[],
            &s,
            crate::render::palette::RgbColor { r: 0, g: 200, b: 0 },
            true,
            STRIP_W,
        );
        let combined: String = ov.lines.concat();
        assert!(
            combined.contains("↵ run"),
            "action focused param should show '↵ run'"
        );
    }

    #[test]
    fn tuner_recent_row_shows_recent_labels() {
        let s = crate::render::theme::SLIME_DARK;
        let recent = vec![ParamView {
            desc: ParamDesc {
                id: ParamId::TurnAngle,
                key_hint: "T/t",
                label: "Turn Angle",
                kind: ParamKind::Numeric,
            },
            value_text: "45.0°".to_string(),
            ratio: Some(0.9),
            def_ratio: Some(0.9),
            state: ParamState::Default,
        }];
        let ov = build_tuner(
            &numeric_fixture(),
            &recent,
            &s,
            crate::render::palette::RgbColor { r: 0, g: 200, b: 0 },
            true,
            STRIP_W,
        );
        // RECENT label appears in the strip
        let combined: String = ov.lines.concat();
        assert!(
            combined.contains("RECENT"),
            "RECENT row must contain RECENT label"
        );
        assert!(
            combined.contains("Turn Angle") || combined.contains("Turn"),
            "recent param label should appear"
        );
    }

    #[test]
    fn tuner_width_threading_wider_than_strip_w() {
        let s = crate::render::theme::SLIME_DARK;
        let wide = 160usize;
        let ov = build_tuner(
            &numeric_fixture(),
            &[],
            &s,
            crate::render::palette::RgbColor { r: 0, g: 200, b: 0 },
            true,
            wide,
        );
        let rich = ov.rich_lines.as_ref().unwrap();
        for row in rich.iter() {
            assert_eq!(row.len(), wide, "wider strip must fill full terminal width");
        }
        for line in ov.lines.iter() {
            assert_eq!(
                line.chars().count(),
                wide,
                "lines width must match threaded terminal width"
            );
        }
    }

    #[test]
    fn tuner_rich_lines_width_matches_lines() {
        let s = crate::render::theme::SLIME_DARK;
        let ov = build_tuner(
            &numeric_fixture(),
            &[],
            &s,
            crate::render::palette::RgbColor { r: 0, g: 200, b: 0 },
            true,
            STRIP_W,
        );
        let rich = ov.rich_lines.as_ref().unwrap();
        for (li, (line, rrow)) in ov.lines.iter().zip(rich.iter()).enumerate() {
            assert_eq!(
                line.chars().count(),
                rrow.len(),
                "line {li} char count mismatch between lines and rich_lines"
            );
        }
    }
}
