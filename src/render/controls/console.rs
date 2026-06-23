//! Console depth of the Controls Instrument: master-detail panel with tab strip.
//!
//! `build_console` produces a [`RenderedOverlay`] with:
//! - A floating title box (`CONTROLS · <CATEGORY>`).
//! - A two-line `●/○` tab strip (one dot per category, accent-coloured for the active one).
//! - A separator then a fixed-height body: left param list | pane divider | right detail.
//! - Both panes padded to `MAX_VISIBLE_ROWS` so total panel height is constant for all
//!   category and focus combinations (satisfies spec M6).
//!
//! Full kind-aware detail rendering is deferred to Task 10; this module provides the
//! chrome skeleton + constant dims. The right-hand detail pane shows a minimal value
//! summary that Task 10 will replace with richer content per [`ParamKind`].

use crate::render::controls::registry::{ParamDesc, ParamKind, CATEGORY_NAMES};
use crate::render::palette::RgbColor;
use crate::render::panel::{Padding, PanelBuilder, RenderedOverlay, RichCell, TextAlignment};
use crate::render::theme::PanelStyle;

// ── Layout constants ──────────────────────────────────────────────────────────

/// Total content width (inner drawable area, before padding/border).
const CW: usize = 66;

/// Left-pane visible columns (param list).
const LEFT_W: usize = 24;

/// Column position of the vertical pane divider (0-indexed within content).
const DIVIDER_AT: usize = 25;

/// Column position where the right detail pane begins.
const RIGHT_AT: usize = 27;

/// Width of the right detail pane (content width minus the space consumed by
/// left pane + divider + gap).
const RIGHT_W: usize = CW - RIGHT_AT;

/// Maximum visible parameter count across ALL categories (including conditionals).
///
/// SIM=7, APP=7, PST=7 (three-way tie).  ENV with both conditional rows
/// (DiffusionSigma + MouseTimeout) reaches 8. We use 8 as the constant body
/// height so the panel size is stable even when those conditionals flip on.
pub const MAX_VISIBLE_ROWS: usize = 8;

/// Background color for the focused row in the left list.
const FOCUS_BG: RgbColor = RgbColor {
    r: 34,
    g: 52,
    b: 40,
};

/// Color used for CLI-readonly parameters.
const CLI_RED: RgbColor = RgbColor {
    r: 204,
    g: 102,
    b: 102,
};

// ── Public types ──────────────────────────────────────────────────────────────

/// Semantic state of a parameter as of the last render tick.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ParamState {
    /// Value equals the current preset/session default.
    Default,
    /// Value has been changed from the default during this session.
    Modified,
    /// Value was supplied via CLI and cannot be changed at runtime.
    Cli,
    /// Read-only display value (no keybind available).
    Display,
}

/// Caller-assembled view data for one parameter row.
///
/// The runner builds these from live [`crate::terminal::state::RuntimeState`] values
/// so that `build_console` itself remains pure and testable without any app state.
pub struct ParamView {
    /// Static descriptor (id, label, key_hint, kind).
    pub desc: ParamDesc,
    /// Formatted current value string (e.g. `"30.0°"` or `"Wrap"`).
    pub value_text: String,
    /// Current value as a ratio in `[0.0, 1.0]` (for gauge bar); `None` for
    /// non-numeric params.
    pub ratio: Option<f32>,
    /// Default value as a ratio in `[0.0, 1.0]` (for gauge tick); `None` for
    /// non-numeric params.
    pub def_ratio: Option<f32>,
    /// Semantic state — drives colour selection.
    pub state: ParamState,
}

// ── Internal row buffer ───────────────────────────────────────────────────────

/// A character + optional per-cell colours for one content row.
struct RowBuf {
    chars: Vec<char>,
    fg: Vec<Option<RgbColor>>,
    bg: Vec<Option<RgbColor>>,
}

impl RowBuf {
    fn new(w: usize) -> Self {
        Self {
            chars: vec![' '; w],
            fg: vec![None; w],
            bg: vec![None; w],
        }
    }

    /// Write `s` starting at column `at` with optional per-character fg/bg.
    fn put(&mut self, at: usize, s: &str, fg: Option<RgbColor>, bg: Option<RgbColor>) {
        for (i, ch) in s.chars().enumerate() {
            let c = at + i;
            if c < self.chars.len() {
                self.chars[c] = ch;
                if fg.is_some() {
                    self.fg[c] = fg;
                }
                if bg.is_some() {
                    self.bg[c] = bg;
                }
            }
        }
    }

    /// Write coloured character cells (from a gauge/heatmap result) starting at `at`.
    fn put_cells(&mut self, at: usize, cells: &[(char, RgbColor)], bg: Option<RgbColor>) {
        for (i, (ch, col)) in cells.iter().enumerate() {
            let c = at + i;
            if c < self.chars.len() {
                self.chars[c] = *ch;
                self.fg[c] = Some(*col);
                if bg.is_some() {
                    self.bg[c] = bg;
                }
            }
        }
    }

    /// Fill a range of cells with `bg`.
    fn set_bg(&mut self, range: std::ops::Range<usize>, bg: RgbColor) {
        for c in range {
            if c < self.bg.len() {
                self.bg[c] = Some(bg);
            }
        }
    }

    /// Blit `other` onto `self` starting at column `at`, skipping `None` overrides.
    fn blit(&mut self, at: usize, other: &RowBuf) {
        for i in 0..other.chars.len() {
            let c = at + i;
            if c < self.chars.len() {
                self.chars[c] = other.chars[i];
                if other.fg[i].is_some() {
                    self.fg[c] = other.fg[i];
                }
                if other.bg[i].is_some() {
                    self.bg[c] = other.bg[i];
                }
            }
        }
    }

    fn text(&self) -> String {
        self.chars.iter().collect()
    }
}

/// Row kind for the `assemble` helper.
enum Rk {
    Sep,
    Buf(RowBuf),
}

// ── Internal helpers ──────────────────────────────────────────────────────────

/// Build the two-line `●/○` tab strip centred in `content_w`.
///
/// Returns `(indicator_row, label_row)`.
fn tab_rows(content_w: usize, active: usize, st: &PanelStyle) -> (RowBuf, RowBuf) {
    let labels = CATEGORY_NAMES;
    let mut ind: Vec<String> = Vec::new();
    let mut lab: Vec<String> = Vec::new();
    for (i, l) in labels.iter().enumerate() {
        ind.push(format!(
            "{:^w$}",
            if i == active { '●' } else { '○' },
            w = l.len()
        ));
        lab.push(l.to_string());
    }
    let ind_s = format!("{:^w$}", ind.join("  "), w = content_w);
    let lab_s = format!("{:^w$}", lab.join("  "), w = content_w);

    let mut ir = RowBuf::new(content_w);
    for (c, ch) in ind_s.chars().enumerate() {
        let col = if ch == '●' {
            Some(st.accent_active)
        } else if ch == '○' {
            Some(st.muted)
        } else {
            None
        };
        ir.put(c, &ch.to_string(), col, None);
    }

    let mut lr = RowBuf::new(content_w);
    let active_l = labels[active];
    let lc: Vec<char> = lab_s.chars().collect();
    let al: Vec<char> = active_l.chars().collect();
    let start = lc.windows(al.len()).position(|w| w == al.as_slice());
    for (c, ch) in lc.iter().enumerate() {
        let hot = start.is_some_and(|s| c >= s && c < s + al.len());
        lr.put(
            c,
            &ch.to_string(),
            Some(if hot { st.accent_active } else { st.muted }),
            None,
        );
    }
    (ir, lr)
}

/// Simple word-wrap: break `s` into lines of at most `w` characters.
///
/// Reserved for Task 10's description field rendering.
#[allow(dead_code)]
fn wrap_text(s: &str, w: usize) -> Vec<String> {
    let mut out = Vec::new();
    let mut line = String::new();
    for word in s.split_whitespace() {
        if line.chars().count() + word.len() + if line.is_empty() { 0 } else { 1 } > w {
            out.push(std::mem::take(&mut line));
        }
        if !line.is_empty() {
            line.push(' ');
        }
        line.push_str(word);
    }
    if !line.is_empty() {
        out.push(line);
    }
    out
}

/// Return the state colour for a parameter given its semantic state.
fn state_color(state: ParamState, st: &PanelStyle) -> RgbColor {
    match state {
        ParamState::Cli => CLI_RED,
        ParamState::Modified => st.accent_modified,
        ParamState::Default | ParamState::Display => st.muted,
    }
}

/// Assemble a list of [`Rk`] content rows + PanelBuilder chrome → [`RenderedOverlay`].
///
/// Mirrors `examples/controls_prototype.rs::assemble`: each [`Rk::Buf`] is added to
/// the builder as a plain-text row, then a rich blit pass injects per-cell colours.
fn assemble(title: &str, content_w: usize, pad: Padding, rows: Vec<Rk>) -> RenderedOverlay {
    let prefix = 1 + pad.top; // top border + top-padding rows
    let offset = 1 + pad.left; // left border + left-padding cols

    let mut builder = PanelBuilder::new(content_w, None)
        .with_padding(Padding::new(pad.top, pad.bottom, pad.left, pad.right))
        .with_title(title)
        .with_title_box();

    let mut bufs: Vec<Option<RowBuf>> = Vec::new();
    for r in rows {
        match r {
            Rk::Sep => {
                builder = builder.add_separator();
                bufs.push(None);
            }
            Rk::Buf(b) => {
                builder = builder.add_single(b.text(), TextAlignment::Left);
                bufs.push(Some(b));
            }
        }
    }

    let mut ov = builder.build_overlay();

    // Rich blit: inject per-cell fg/bg overrides from RowBufs.
    let rich: Vec<Vec<RichCell>> = ov
        .lines
        .iter()
        .enumerate()
        .map(|(li, line)| {
            let mut cells: Vec<RichCell> = line.chars().map(|ch| (ch, None, None)).collect();
            if li >= prefix {
                if let Some(Some(b)) = bufs.get(li - prefix) {
                    for i in 0..b.chars.len() {
                        if let Some(cell) = cells.get_mut(offset + i) {
                            if b.fg[i].is_some() {
                                cell.1 = b.fg[i];
                            }
                            if b.bg[i].is_some() {
                                cell.2 = b.bg[i];
                            }
                        }
                    }
                }
            }
            cells
        })
        .collect();

    ov.rich_lines = Some(rich);
    ov
}

// ── Public builder ────────────────────────────────────────────────────────────

/// Build the Console master-detail overlay.
///
/// - `category`: active category index `0..6` (selects the active tab dot and the title).
/// - `focus`: focused row index within `params` (clamped if out of range).
/// - `params`: caller-built [`ParamView`] slice for the active category.
/// - `style`: [`PanelStyle`] (colour palette).
/// - `accent`: caller-supplied accent colour (e.g. from the active simulation palette).
///
/// The returned [`RenderedOverlay`] has a constant height for all valid `(category, focus,
/// params)` combinations because both the left list and the right detail pane are
/// padded to [`MAX_VISIBLE_ROWS`] blank rows.
pub fn build_console(
    category: usize,
    focus: usize,
    params: &[ParamView],
    style: &PanelStyle,
    _accent: RgbColor,
) -> RenderedOverlay {
    let cat = category.min(CATEGORY_NAMES.len().saturating_sub(1));
    let title = format!("CONTROLS · {}", CATEGORY_NAMES[cat]);
    let (ir, lr) = tab_rows(CW, cat, style);

    // ── Left list ─────────────────────────────────────────────────────────────
    let focus_clamped = if params.is_empty() {
        0
    } else {
        focus.min(params.len() - 1)
    };

    let mut left: Vec<RowBuf> = Vec::new();
    for (i, pv) in params.iter().enumerate() {
        let mut b = RowBuf::new(LEFT_W);
        let focused = i == focus_clamped;
        if focused {
            b.set_bg(0..LEFT_W, FOCUS_BG);
            b.put(0, "▎", Some(style.accent_active), Some(FOCUS_BG));
        }
        let marker = match pv.state {
            ParamState::Modified => "✱",
            _ => " ",
        };
        let marker_col = match pv.state {
            ParamState::Modified => Some(style.accent_modified),
            _ => None,
        };
        b.put(1, marker, marker_col, None);

        let lcol = if focused || pv.state == ParamState::Modified {
            style.text_primary
        } else {
            style.muted
        };
        b.put(3, pv.desc.label, Some(lcol), None);

        // Gauge (6 chars) at the right end of the left pane, for numeric params.
        if let (Some(ratio), Some(def_ratio)) = (pv.ratio, pv.def_ratio) {
            let g = crate::render::controls::value::gauge(
                ratio,
                def_ratio,
                6,
                state_color(pv.state, style),
                style.accent_active,
                style.muted,
            );
            b.put_cells(LEFT_W - 6, &g, None);
        }
        left.push(b);
    }

    // Pad left to constant height.
    debug_assert!(
        left.len() <= MAX_VISIBLE_ROWS,
        "left pane overflow: {} > {}",
        left.len(),
        MAX_VISIBLE_ROWS
    );
    while left.len() < MAX_VISIBLE_ROWS {
        left.push(RowBuf::new(LEFT_W));
    }

    // ── Right detail (minimal/placeholder — Task 10 owns full kind-awareness) ─
    let mut right: Vec<RowBuf> = Vec::new();

    if !params.is_empty() {
        let pv = &params[focus_clamped];

        // Header: label + key hint
        let mut head = RowBuf::new(RIGHT_W);
        head.put(0, pv.desc.label, Some(style.text_primary), None);
        let key_str = format!("[{}]", pv.desc.key_hint);
        let key_col = match pv.state {
            ParamState::Cli => Some(CLI_RED),
            _ => Some(style.accent_active),
        };
        let key_start = RIGHT_W.saturating_sub(key_str.len());
        head.put(key_start, &key_str, key_col, None);
        right.push(head);

        right.push(RowBuf::new(RIGHT_W)); // blank spacer

        // Value row: value text + state label
        let mut valrow = RowBuf::new(RIGHT_W);
        let val_col = match pv.state {
            ParamState::Modified => Some(style.accent_active),
            _ => Some(style.text_primary),
        };
        valrow.put(0, &pv.value_text, val_col, None);
        let state_label = match pv.state {
            ParamState::Default => "default",
            ParamState::Modified => "modified",
            ParamState::Cli => "cli-only",
            ParamState::Display => "display",
        };
        valrow.put(14, state_label, Some(state_color(pv.state, style)), None);
        right.push(valrow);

        // Gauge row (numeric only)
        if let (Some(ratio), Some(def_ratio)) = (pv.ratio, pv.def_ratio) {
            let gw = RIGHT_W.saturating_sub(2);
            let mut grow = RowBuf::new(RIGHT_W);
            grow.put_cells(
                0,
                &crate::render::controls::value::gauge(
                    ratio,
                    def_ratio,
                    gw,
                    state_color(pv.state, style),
                    style.accent_active,
                    style.muted,
                ),
                None,
            );
            right.push(grow);

            // Tick row: min label, default marker, max label
            let mut tick = RowBuf::new(RIGHT_W);
            let min_s = "0";
            let max_s = "max";
            let def_col = (def_ratio * gw as f32) as usize;
            tick.put(0, min_s, Some(style.muted), None);
            if def_col < RIGHT_W {
                tick.put(def_col, "▲", Some(style.accent_active), None);
            }
            let max_start = RIGHT_W.saturating_sub(max_s.len());
            tick.put(max_start, max_s, Some(style.muted), None);
            right.push(tick);

            right.push(RowBuf::new(RIGHT_W)); // blank before description
        } else {
            // Non-numeric: just a blank where the gauge would be
            right.push(RowBuf::new(RIGHT_W));
            right.push(RowBuf::new(RIGHT_W));
        }

        // Kind label (Task 10 will expand this per-kind)
        let kind_label = match pv.desc.kind {
            ParamKind::Numeric => "numeric",
            ParamKind::Enum => "enum",
            ParamKind::Toggle => "toggle",
            ParamKind::Action => "action",
            ParamKind::CliReadonly => "cli-only",
            ParamKind::Display => "display",
        };
        let mut krow = RowBuf::new(RIGHT_W);
        krow.put(0, kind_label, Some(style.text_secondary), None);
        right.push(krow);
    }

    // Pad right to constant height.
    debug_assert!(
        right.len() <= MAX_VISIBLE_ROWS,
        "right pane overflow: {} > {}",
        right.len(),
        MAX_VISIBLE_ROWS
    );
    while right.len() < MAX_VISIBLE_ROWS {
        right.push(RowBuf::new(RIGHT_W));
    }

    // ── Compose body rows ─────────────────────────────────────────────────────
    let mut rows: Vec<Rk> = vec![Rk::Buf(ir), Rk::Buf(lr), Rk::Sep];

    for r in 0..MAX_VISIBLE_ROWS {
        let mut row = RowBuf::new(CW);
        row.blit(0, &left[r]);
        row.put(DIVIDER_AT, "│", Some(style.muted), None);
        row.blit(RIGHT_AT, &right[r]);
        rows.push(Rk::Buf(row));
    }

    rows.push(Rk::Sep);

    // Legend row
    let mut leg = RowBuf::new(CW);
    leg.put(0, "✱ modified", Some(style.accent_modified), None);
    leg.put(13, "│ default", Some(style.accent_active), None);
    leg.put(DIVIDER_AT, "─ cli-only", Some(CLI_RED), None);
    rows.push(Rk::Buf(leg));

    assemble(&title, CW, Padding::new(1, 1, 2, 2), rows)
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::render::controls::registry::{ParamDesc, ParamId, ParamKind};

    /// Minimal fixture: one or two params per category, enough to drive layout.
    fn fixture_params(cat: usize) -> Vec<ParamView> {
        let make = |id: ParamId,
                    key_hint: &'static str,
                    label: &'static str,
                    kind: ParamKind,
                    value_text: &'static str,
                    ratio: Option<f32>,
                    def_ratio: Option<f32>,
                    state: ParamState|
         -> ParamView {
            ParamView {
                desc: ParamDesc {
                    id,
                    key_hint,
                    label,
                    kind,
                },
                value_text: value_text.to_string(),
                ratio,
                def_ratio,
                state,
            }
        };
        match cat {
            0 => vec![
                make(
                    ParamId::SensorAngle,
                    "A/a",
                    "Sensor Angle",
                    ParamKind::Numeric,
                    "30.0°",
                    Some(0.5),
                    Some(0.3),
                    ParamState::Modified,
                ),
                make(
                    ParamId::TurnAngle,
                    "T/t",
                    "Turn Angle",
                    ParamKind::Numeric,
                    "45.0°",
                    Some(0.9),
                    Some(0.9),
                    ParamState::Default,
                ),
            ],
            1 => vec![make(
                ParamId::DiffusionKernel,
                "K",
                "Diffusion",
                ParamKind::Enum,
                "Box",
                None,
                None,
                ParamState::Default,
            )],
            2 => vec![make(
                ParamId::Palette,
                "c/C",
                "Palette",
                ParamKind::Enum,
                "Aurora",
                None,
                None,
                ParamState::Default,
            )],
            3 => vec![make(
                ParamId::Brightness,
                "N/n",
                "Brightness",
                ParamKind::Numeric,
                "100",
                Some(0.5),
                Some(0.5),
                ParamState::Default,
            )],
            4 => vec![make(
                ParamId::Population,
                "─",
                "Population",
                ParamKind::CliReadonly,
                "50k",
                None,
                None,
                ParamState::Cli,
            )],
            5 => vec![make(
                ParamId::Reset,
                "0",
                "Reset",
                ParamKind::Action,
                "",
                None,
                None,
                ParamState::Default,
            )],
            _ => vec![],
        }
    }

    #[test]
    fn console_dims_constant_across_categories_and_focus() {
        let s = crate::render::theme::SLIME_DARK;
        let acc = crate::render::palette::RgbColor { r: 0, g: 200, b: 0 };
        let mk = |cat| build_console(cat, 0, &fixture_params(cat), &s, acc);
        let h0 = mk(0).lines.len();
        for cat in 0..6 {
            assert_eq!(mk(cat).lines.len(), h0, "category {cat} height differs");
        }
        let w0 = mk(0).lines[0].chars().count();
        // Verify all categories have consistent width
        for cat in 0..6 {
            let ov = build_console(cat, 0, &fixture_params(cat), &s, acc);
            assert!(
                ov.lines.iter().all(|l| l.chars().count() == w0),
                "cat {cat} width varies"
            );
        }
        // For categories with multiple params, vary focus to exercise numeric-vs-non-numeric
        // right-pane branching (which has different pre-pad row counts)
        if fixture_params(0).len() >= 2 {
            let params_cat0 = fixture_params(0);
            for focus in 0..params_cat0.len() {
                let ov = build_console(0, focus, &params_cat0, &s, acc);
                assert_eq!(ov.lines.len(), h0, "cat 0 focus {focus} height differs");
                assert!(
                    ov.lines.iter().all(|l| l.chars().count() == w0),
                    "cat 0 focus {focus} width varies"
                );
            }
        }
    }

    #[test]
    fn console_has_title_box_and_active_tab() {
        let ov = build_console(
            0,
            0,
            &fixture_params(0),
            &crate::render::theme::SLIME_DARK,
            crate::render::palette::RgbColor { r: 0, g: 200, b: 0 },
        );
        assert!(ov
            .title_box
            .as_ref()
            .unwrap()
            .lines
            .iter()
            .any(|l| l.contains("CONTROLS")));
        assert!(ov.lines.iter().any(|l| l.contains('●')));
    }

    #[test]
    fn console_right_pane_shows_focused_param_label() {
        let params = fixture_params(0);
        let ov = build_console(
            0,
            0,
            &params,
            &crate::render::theme::SLIME_DARK,
            crate::render::palette::RgbColor { r: 0, g: 200, b: 0 },
        );
        // The first param's label should appear somewhere in the body lines.
        let combined: String = ov.lines.concat();
        assert!(
            combined.contains("Sensor Angle"),
            "focused label not found in overlay"
        );
    }

    #[test]
    fn console_focus_clamps_to_params_len() {
        // Passing an out-of-range focus index must not panic.
        let params = fixture_params(5); // only 1 param
        let ov = build_console(
            5,
            99,
            &params,
            &crate::render::theme::SLIME_DARK,
            crate::render::palette::RgbColor { r: 0, g: 200, b: 0 },
        );
        assert!(!ov.lines.is_empty());
    }

    #[test]
    fn console_empty_params_does_not_panic() {
        // Empty params list (e.g. unknown category) must not panic and must still
        // produce constant-height output.
        let s = crate::render::theme::SLIME_DARK;
        let acc = crate::render::palette::RgbColor { r: 0, g: 200, b: 0 };
        let ov_empty = build_console(0, 0, &[], &s, acc);
        let ov_normal = build_console(0, 0, &fixture_params(0), &s, acc);
        assert_eq!(ov_empty.lines.len(), ov_normal.lines.len());
    }
}
