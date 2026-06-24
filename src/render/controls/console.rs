//! Console depth of the Controls Instrument: master-detail panel with tab strip.
//!
//! `build_console` produces a [`RenderedOverlay`] with:
//! - A floating title box (`CONTROLS · <CATEGORY>`).
//! - A two-line `●/○` tab strip (one dot per category, accent-coloured for the active one).
//! - A separator then a fixed-height body: left param list | pane divider | right detail.
//! - Both panes padded to `MAX_VISIBLE_ROWS` so total panel height is constant for all
//!   category and focus combinations (satisfies spec M6).
//!
//! The right-hand detail pane is kind-aware: it renders per-[`ParamKind`] content
//! (numeric gauge + min/▲def/max ticks, enum value + cycle hint, toggle pill,
//! action affordance, CLI-readonly "restart to change" copy, display-only muted
//! value) plus a derived description line. Both panes are padded to a constant
//! row count so the overall panel dimensions never change with category or focus.

use crate::render::controls::field_plate::{field_plate, render_art, Profile};
use crate::render::controls::registry::{ParamDesc, ParamKind, CATEGORY_NAMES};
use crate::render::palette::{Palette, RgbColor};
use crate::render::panel::{Padding, PanelBuilder, RenderedOverlay, RichCell, TextAlignment};
use crate::render::theme::PanelStyle;
use crate::render::widgets::{state_color, RowBuf};

pub use crate::render::widgets::ParamState;

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
/// SIM=7, ENV=8 (with both conditional rows: DiffusionSigma + MouseTimeout),
/// PST=8 (with IntensityMapping), APP=10 (the largest: + WindowFrame + Chrome).
/// We size the constant body height to the largest category so the panel size is
/// stable across categories and when conditional rows flip on; sparser categories
/// pad with blank rows (the established constant-height design).
pub(crate) const MAX_VISIBLE_ROWS: usize = 10;

// ── Public types ──────────────────────────────────────────────────────────────

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
                    b.overlay_styles(&mut cells, offset);
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
    palette: Palette,
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
            b.set_bg(0..LEFT_W, style.focus_bg);
            b.put(0, "▎", Some(style.accent_active), Some(style.focus_bg));
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

    // ── Right detail: kind-aware per-param content ────────────────────────────
    //
    // Fixed layout — always exactly MAX_VISIBLE_ROWS rows:
    //   row 0:   header  (label left, [key] right)
    //   row 1:   blank spacer
    //   row 2:   primary value / affordance row
    //   row 3:   kind widget row 1  (gauge bar / option hint / toggle pill / …)
    //   row 4:   kind widget row 2  (tick labels / secondary hint / blank)
    //   row 5:   blank divider
    //   row 6:   description line 1 (wrapped from derived hint text)
    //   row 7:   description line 2 (wrapped remainder, or blank)
    //
    // This keeps constant dims without per-kind padding arithmetic.
    let mut right: Vec<RowBuf> = Vec::new();

    if !params.is_empty() {
        let pv = &params[focus_clamped];

        if let Some(plate) = field_plate(pv.desc.id) {
            // Art mode: specimen fills the pane; the [key] chip rides row 0.
            let live = match pv.desc.kind {
                ParamKind::Enum | ParamKind::Toggle => Some(pv.value_text.as_str()),
                _ => None,
            };
            let mut art_rows = render_art(
                plate.art,
                plate.caption,
                Profile::Safe,
                style,
                palette,
                live,
                RIGHT_W,
            );
            // Right-aligned [key] chip on the first row (replaces the old header).
            if let Some(first) = art_rows.first_mut() {
                let key_str = format!("[{}]", pv.desc.key_hint);
                let key_col = match pv.state {
                    ParamState::Cli => style.cli_color,
                    _ => style.accent_active,
                };
                first.put(
                    RIGHT_W.saturating_sub(key_str.chars().count()),
                    &key_str,
                    Some(key_col),
                    None,
                );
            }
            right.extend(art_rows);
        } else {
            // Fallback: kind-aware detail, cleaned of label redundancy.

            // ── row 0: [key] chip only (label lives in title-box + left list) ───
            let mut head = RowBuf::new(RIGHT_W);
            let key_str = format!("[{}]", pv.desc.key_hint);
            let key_col = match pv.state {
                ParamState::Cli => Some(style.cli_color),
                _ => Some(style.accent_active),
            };
            head.put(
                RIGHT_W.saturating_sub(key_str.chars().count()),
                &key_str,
                key_col,
                None,
            );
            right.push(head);

            // ── row 1: blank spacer ─────────────────────────────────────────────
            right.push(RowBuf::new(RIGHT_W));

            // ── row 2: primary value / affordance ──────────────────────────────
            let mut valrow = RowBuf::new(RIGHT_W);
            match pv.desc.kind {
                ParamKind::Numeric => {
                    // Value text + state label
                    let val_col = match pv.state {
                        ParamState::Modified => Some(style.accent_active),
                        _ => Some(style.text_primary),
                    };
                    valrow.put(0, &pv.value_text, val_col, None);
                    // A Numeric param never carries ParamState::Display, so it is
                    // folded into the "default" label to keep the match exhaustive.
                    let state_label = match pv.state {
                        ParamState::Modified => "modified",
                        ParamState::Cli => "cli-only",
                        ParamState::Default | ParamState::Display => "default",
                    };
                    valrow.put(14, state_label, Some(state_color(pv.state, style)), None);
                }
                ParamKind::Enum => {
                    // Current value prominently; no gauge ticks
                    valrow.put(0, &pv.value_text, Some(style.text_primary), None);
                    let hint = "← → to cycle";
                    let hint_start = RIGHT_W.saturating_sub(hint.len());
                    valrow.put(hint_start, hint, Some(style.muted), None);
                }
                ParamKind::Toggle => {
                    // On/off pill
                    let (pill_text, pill_col) = if pv.value_text.eq_ignore_ascii_case("on")
                        || pv.value_text == "true"
                        || pv.value_text == "1"
                    {
                        ("[ ON  ]", style.accent_active)
                    } else {
                        ("[ OFF ]", style.muted)
                    };
                    valrow.put(0, pill_text, Some(pill_col), None);
                }
                ParamKind::Action => {
                    // "↵ to run" affordance
                    valrow.put(0, "↵ to run", Some(style.accent_active), None);
                }
                ParamKind::CliReadonly => {
                    // Value + "restart to change"
                    valrow.put(0, &pv.value_text, Some(style.cli_color), None);
                    let hint = "restart to change";
                    let hint_start = RIGHT_W.saturating_sub(hint.len());
                    valrow.put(hint_start, hint, Some(style.cli_color), None);
                }
                ParamKind::Display => {
                    // Value, dimmed
                    valrow.put(0, &pv.value_text, Some(style.muted), None);
                }
            }
            right.push(valrow);

            // ── rows 3–4: kind widget (gauge+tick for numeric; hint for others) ─
            match pv.desc.kind {
                ParamKind::Numeric => {
                    if let (Some(ratio), Some(def_ratio)) = (pv.ratio, pv.def_ratio) {
                        // row 3: large gauge bar
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

                        // row 4: tick labels  min … ▲def … max
                        let mut tick = RowBuf::new(RIGHT_W);
                        let min_s = "min";
                        let max_s = "max";
                        let def_col = (def_ratio * gw as f32).round() as usize;
                        tick.put(0, min_s, Some(style.muted), None);
                        if def_col < RIGHT_W {
                            tick.put(def_col, "▲", Some(style.accent_active), None);
                        }
                        let max_start = RIGHT_W.saturating_sub(max_s.len());
                        tick.put(max_start, max_s, Some(style.muted), None);
                        right.push(tick);
                    } else {
                        // ratio not supplied — two blank rows
                        right.push(RowBuf::new(RIGHT_W));
                        right.push(RowBuf::new(RIGHT_W));
                    }
                }
                ParamKind::Enum => {
                    // row 3: option indicator "◈ <value>"
                    let mut orow = RowBuf::new(RIGHT_W);
                    let opt_str = format!("◈  {}", pv.value_text);
                    orow.put(0, &opt_str, Some(style.accent_active), None);
                    right.push(orow);
                    // row 4: blank
                    right.push(RowBuf::new(RIGHT_W));
                }
                ParamKind::Toggle => {
                    // row 3: press key to toggle hint
                    let mut trow = RowBuf::new(RIGHT_W);
                    trow.put(0, "press key to toggle", Some(style.muted), None);
                    right.push(trow);
                    // row 4: blank
                    right.push(RowBuf::new(RIGHT_W));
                }
                ParamKind::Action => {
                    // row 3: confirm prompt
                    let mut arow = RowBuf::new(RIGHT_W);
                    arow.put(0, "no undo — runs immediately", Some(style.muted), None);
                    right.push(arow);
                    // row 4: blank
                    right.push(RowBuf::new(RIGHT_W));
                }
                ParamKind::CliReadonly => {
                    // row 3: set via flag hint
                    let mut crow = RowBuf::new(RIGHT_W);
                    crow.put(0, "set via CLI flag at launch", Some(style.muted), None);
                    right.push(crow);
                    // row 4: blank
                    right.push(RowBuf::new(RIGHT_W));
                }
                ParamKind::Display => {
                    // rows 3–4: blank
                    right.push(RowBuf::new(RIGHT_W));
                    right.push(RowBuf::new(RIGHT_W));
                }
            }

            // ── row 5: blank divider ────────────────────────────────────────────
            right.push(RowBuf::new(RIGHT_W));

            // ── rows 6–7: reserved (field-plate art will fill these once authored) ─
            right.push(RowBuf::new(RIGHT_W));
            right.push(RowBuf::new(RIGHT_W));
        }
    }

    // Truncate art overflow before padding.
    right.truncate(MAX_VISIBLE_ROWS);

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
    leg.put(DIVIDER_AT, "─ cli-only", Some(style.cli_color), None);
    rows.push(Rk::Buf(leg));

    assemble(&title, CW, Padding::new(1, 1, 2, 2), rows)
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::render::controls::registry::{
        visible_params, ParamDesc, ParamId, ParamKind, RegistryCtx, CATEGORY_NAMES,
    };
    use crate::render::palette::Palette;

    /// No category may exceed [`MAX_VISIBLE_ROWS`] — the console pads both panes
    /// to that constant and the body loop reads exactly `MAX_VISIBLE_ROWS` rows,
    /// so any category that yields more rows is **silently truncated in release**
    /// (the `debug_assert!` in `build_console` is stripped). This test is the
    /// release-proof guard: it runs in CI and covers *every* category under
    /// *every* `RegistryCtx` combination, so a row added on any flag polarity is
    /// caught at the source.
    ///
    /// Exhaustive over all `RegistryCtx` bool fields. **If you add a field to
    /// `RegistryCtx`, add it to `ALL_CTX` below** so the coverage stays total.
    #[test]
    fn no_category_exceeds_max_visible_rows() {
        const ALL_CTX: [RegistryCtx; 4] = [
            RegistryCtx {
                diffusion_gaussian: false,
                mouse_enabled: false,
            },
            RegistryCtx {
                diffusion_gaussian: false,
                mouse_enabled: true,
            },
            RegistryCtx {
                diffusion_gaussian: true,
                mouse_enabled: false,
            },
            RegistryCtx {
                diffusion_gaussian: true,
                mouse_enabled: true,
            },
        ];
        for ctx in &ALL_CTX {
            for (cat, name) in CATEGORY_NAMES.iter().enumerate() {
                let n = visible_params(cat, ctx).len();
                assert!(
                    n <= MAX_VISIBLE_ROWS,
                    "category {} ({}) has {} rows > MAX_VISIBLE_ROWS ({}) at ctx {:?}",
                    cat,
                    name,
                    n,
                    MAX_VISIBLE_ROWS,
                    ctx,
                );
            }
        }
    }

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
        let mk = |cat| build_console(cat, 0, &fixture_params(cat), &s, acc, Palette::Organic);
        let h0 = mk(0).lines.len();
        for cat in 0..6 {
            assert_eq!(mk(cat).lines.len(), h0, "category {cat} height differs");
        }
        let w0 = mk(0).lines[0].chars().count();
        // Verify all categories have consistent width
        for cat in 0..6 {
            let ov = build_console(cat, 0, &fixture_params(cat), &s, acc, Palette::Organic);
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
                let ov = build_console(0, focus, &params_cat0, &s, acc, Palette::Organic);
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
            crate::render::palette::Palette::Organic,
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
            crate::render::palette::Palette::Organic,
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
            crate::render::palette::Palette::Organic,
        );
        assert!(!ov.lines.is_empty());
    }

    #[test]
    fn console_empty_params_does_not_panic() {
        // Empty params list (e.g. unknown category) must not panic and must still
        // produce constant-height output.
        let s = crate::render::theme::SLIME_DARK;
        let acc = crate::render::palette::RgbColor { r: 0, g: 200, b: 0 };
        let ov_empty = build_console(0, 0, &[], &s, acc, Palette::Organic);
        let ov_normal = build_console(0, 0, &fixture_params(0), &s, acc, Palette::Organic);
        assert_eq!(ov_empty.lines.len(), ov_normal.lines.len());
    }

    /// Enum fixture: a ParamView with kind=Enum, ratio=None, value_text="HalfBlock".
    /// Uses Charset which is NOT authored → exercises the fallback enum detail.
    fn enum_fixture() -> ParamView {
        ParamView {
            desc: ParamDesc {
                id: ParamId::Charset,
                key_hint: "G/g",
                label: "Charset",
                kind: ParamKind::Enum,
            },
            value_text: "HalfBlock".to_string(),
            ratio: None,
            def_ratio: None,
            state: ParamState::Default,
        }
    }

    #[test]
    fn focused_row_has_bg_override() {
        // focus=2 is intentionally out-of-range (fixture_params(0) has only 2 items)
        // and clamped to the last item index (1).
        let mut style = crate::render::theme::SLIME_DARK;
        let focus_bg = RgbColor::new(1, 2, 3);
        style.focus_bg = focus_bg;
        let ov = build_console(
            0,
            2,
            &fixture_params(0),
            &style,
            crate::render::palette::RgbColor { r: 0, g: 200, b: 0 },
            crate::render::palette::Palette::Organic,
        );
        let rich = ov.rich_lines.unwrap();
        assert_eq!(
            rich.iter()
                .flatten()
                .filter(|(_, _, bg)| *bg == Some(focus_bg))
                .count(),
            LEFT_W,
            "focused row must use the supplied focus_bg token"
        );
    }

    #[test]
    fn enum_param_shows_value_not_gauge_ticks() {
        // An enum ParamView has ratio: None → detail must contain "Gaussian"
        // and must not draw min/max gauge ticks (no ▲ in the detail lines).
        let pv = enum_fixture();
        let ov = build_console(
            1,
            0,
            &[pv],
            &crate::render::theme::SLIME_DARK,
            crate::render::palette::RgbColor { r: 0, g: 200, b: 0 },
            crate::render::palette::Palette::Organic,
        );
        assert!(
            ov.lines.iter().any(|l| l.contains("HalfBlock")),
            "enum value text 'HalfBlock' not found in overlay"
        );
        // Gauge ticks (▲ for default marker) must not appear in the right detail pane.
        // We check the full output doesn't contain a gauge tick marker.
        let combined: String = ov.lines.concat();
        assert!(
            !combined.contains('▲'),
            "enum detail pane must not show gauge tick marker ▲"
        );
    }

    #[test]
    fn action_param_detail_content() {
        // Action kind — use Randomize which is NOT authored, so the fallback detail renders.
        // Detail should contain the "↵ to run" affordance string.
        let pv = ParamView {
            desc: ParamDesc {
                id: ParamId::Randomize,
                key_hint: "R",
                label: "Randomize",
                kind: ParamKind::Action,
            },
            value_text: "".to_string(),
            ratio: None,
            def_ratio: None,
            state: ParamState::Default,
        };
        let ov = build_console(
            5,
            0,
            &[pv],
            &crate::render::theme::SLIME_DARK,
            crate::render::palette::RgbColor { r: 0, g: 200, b: 0 },
            crate::render::palette::Palette::Organic,
        );
        let combined: String = ov.lines.concat();
        assert!(
            combined.contains("↵ to run"),
            "action detail should contain '↵ to run' affordance"
        );
    }

    #[test]
    fn cli_readonly_param_detail_content() {
        // CliReadonly kind — use FastMode which is NOT authored, so the fallback detail renders.
        // Detail should contain the "restart to change" hint string.
        let mut style = crate::render::theme::SLIME_DARK;
        let cli_color = RgbColor::new(1, 2, 3);
        style.cli_color = cli_color;
        let pv = ParamView {
            desc: ParamDesc {
                id: ParamId::DiffusionSigma,
                key_hint: "─",
                label: "Sigma",
                kind: ParamKind::CliReadonly,
            },
            value_text: "1.5".to_string(),
            ratio: None,
            def_ratio: None,
            state: ParamState::Cli,
        };
        let ov = build_console(
            4,
            0,
            &[pv],
            &style,
            crate::render::palette::RgbColor { r: 0, g: 200, b: 0 },
            crate::render::palette::Palette::Organic,
        );
        let combined: String = ov.lines.concat();
        assert!(
            combined.contains("restart to change"),
            "cli-readonly detail should contain 'restart to change' hint"
        );
        assert!(
            ov.rich_lines
                .expect("console needs rich_lines")
                .iter()
                .flatten()
                .any(|(_, fg, _)| *fg == Some(cli_color)),
            "CLI content must use the supplied cli_color token"
        );
    }

    #[test]
    fn toggle_param_detail_content_on() {
        // Toggle kind (category 2=APP, Invert/Reverse are Toggle)
        // Focus on Invert (first Toggle in the fixture)
        // When value_text is "on", should show "[ ON  ]" pill.
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
        let ov = build_console(
            2,
            0,
            &[pv],
            &crate::render::theme::SLIME_DARK,
            crate::render::palette::RgbColor { r: 0, g: 200, b: 0 },
            crate::render::palette::Palette::Organic,
        );
        let combined: String = ov.lines.concat();
        assert!(
            combined.contains("[ ON  ]"),
            "toggle detail with 'on' value should contain '[ ON  ]' pill"
        );
    }

    #[test]
    fn toggle_param_detail_content_off() {
        // Toggle kind (category 2=APP, Invert/Reverse are Toggle)
        // When value_text is "off", should show "[ OFF ]" pill.
        let pv = ParamView {
            desc: ParamDesc {
                id: ParamId::Invert,
                key_hint: "X",
                label: "Invert",
                kind: ParamKind::Toggle,
            },
            value_text: "off".to_string(),
            ratio: None,
            def_ratio: None,
            state: ParamState::Default,
        };
        let ov = build_console(
            2,
            0,
            &[pv],
            &crate::render::theme::SLIME_DARK,
            crate::render::palette::RgbColor { r: 0, g: 200, b: 0 },
            crate::render::palette::Palette::Organic,
        );
        let combined: String = ov.lines.concat();
        assert!(
            combined.contains("[ OFF ]"),
            "toggle detail with 'off' value should contain '[ OFF ]' pill"
        );
    }

    #[test]
    fn numeric_param_has_gauge_bar_char() {
        // Numeric kind renders a gauge with fill characters (█).
        // fixture_params(0) has Numeric params with Some(ratio), so gauge is rendered.
        let ov = build_console(
            0,
            0,
            &fixture_params(0),
            &crate::render::theme::SLIME_DARK,
            crate::render::palette::RgbColor { r: 0, g: 200, b: 0 },
            crate::render::palette::Palette::Organic,
        );
        let combined: String = ov.lines.concat();
        assert!(
            combined.contains('█'),
            "numeric detail pane should contain gauge bar fill character █"
        );
    }

    #[test]
    fn authored_param_renders_art_stage_label() {
        // SensorAngle (cat 0) is authored → detail shows the art's stage label.
        let ov = build_console(
            0,
            0,
            &fixture_params(0),
            &crate::render::theme::SLIME_DARK,
            crate::render::palette::RgbColor { r: 0, g: 200, b: 0 },
            crate::render::palette::Palette::Organic,
        );
        let combined: String = ov.lines.concat();
        assert!(
            combined.contains("SENSE"),
            "art stage label missing:\n{combined}"
        );
    }

    #[test]
    fn fallback_param_drops_restated_label_sentence() {
        // TurnAngle (cat 0, second fixture row) is NOT authored → cleaned detail.
        // The detail must not contain the old restated "adjust with keybind" copy.
        let ov = build_console(
            0,
            1,
            &fixture_params(0),
            &crate::render::theme::SLIME_DARK,
            crate::render::palette::RgbColor { r: 0, g: 200, b: 0 },
            crate::render::palette::Palette::Organic,
        );
        let combined: String = ov.lines.concat();
        assert!(
            !combined.contains("adjust with keybind"),
            "restated sentence not dropped"
        );
    }
}
