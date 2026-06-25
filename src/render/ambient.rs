//! Ambient instrument surface: BASE ↔ TUNE ↔ MSG state machine.
//!
//! The ambient surface is the bottom-docked "always-on" instrument strip.
//! It selects which state to render via [`resolve`] and renders it into a
//! fixed-height [`RenderedOverlay`] block via [`build_ambient`].
//!
//! # State priority (high to low)
//!
//! 1. `Msg` with `level == Error` (sticky or non-sticky, while live)
//! 2. Any live `Msg` (non-Error)
//! 3. Live `Tune`
//! 4. `Base` (always present; never expires)
//!
//! "Live" means: `sticky == true` OR `now <= until`.

use crate::render::controls::ParamKind;
use crate::render::motion::{breath, lerp_rgb};
use crate::render::palette::RgbColor;
use crate::render::panel::{footer_hints, RenderedOverlay, RichCell};
use crate::render::theme::PanelStyle;
use crate::render::widgets::{gauge, swatch, value_color, ParamState, RowBuf};
use crate::terminal::state::NotificationLevel;

// ── Layout constants ──────────────────────────────────────────────────────────

/// Number of rows in the ambient strip (matching tuner height).
const STRIP_H: usize = 6;

/// Minimum strip width in characters.
const STRIP_W: usize = 80;

// ── Public types ──────────────────────────────────────────────────────────────

/// Status context for the BASE state of the ambient instrument.
///
/// All strings are pre-formatted by the caller (preset name, palette name, etc.)
/// so the renderer has no dependency on config enum types.
#[derive(Clone, Debug)]
pub struct BaseStatus {
    /// Human-readable preset name (e.g. `"Organic"`).
    pub preset_name: String,
    /// Human-readable palette name (e.g. `"Forest"`).
    pub palette_name: String,
    /// Time scale multiplier string (e.g. `"1.0×"`).
    pub time_scale_text: String,
    /// Optional population in agents (displayed as `Nk`).
    pub population: Option<usize>,
    /// Optional dither-mode label (e.g. `"D 0.5×"`, `"ED"`).
    pub dither_label: Option<String>,
    /// Whether an undo entry is available.
    pub can_undo: bool,
    /// Whether a redo entry is available.
    pub can_redo: bool,
    /// Palette accent color for the swatch block.
    pub accent: RgbColor,
    /// Whether the simulation is paused.
    pub is_paused: bool,
}

impl Default for BaseStatus {
    fn default() -> Self {
        Self {
            preset_name: String::new(),
            palette_name: String::new(),
            time_scale_text: String::new(),
            population: None,
            dither_label: None,
            can_undo: false,
            can_redo: false,
            accent: RgbColor { r: 0, g: 0, b: 0 },
            is_paused: false,
        }
    }
}

/// A focused-param view for the TUNE state.
#[derive(Clone, Debug)]
pub struct TuneView {
    /// Short human-readable label, e.g. `"Sensor Angle"`.
    pub label: String,
    /// Pre-formatted display value (e.g. `"45.0°"`, `"On"`, `"Forest"`). The
    /// caller formats this from the same source as the Controls param views so
    /// the TUNE row reads identically.
    pub value_text: String,
    /// Normalized gauge position in `[0, 1]` for the value marker.
    pub value: f32,
    /// `(min, max)` range for the gauge. With normalized `value`/`default`
    /// callers pass `(0.0, 1.0)`.
    pub range: (f32, f32),
    /// Normalized default position in `[0, 1]`; shown as a reference marker.
    pub default: f32,
    /// Functional state (drives color selection via L2 tokens).
    pub state: ParamState,
    /// Whether to render the value gauge. `true` for numeric params (the gauge
    /// is meaningful); `false` for enum/toggle params, where the value lives in
    /// `value_text` and an empty gauge would misleadingly read as "minimum".
    pub show_gauge: bool,
    /// Parameter kind — drives the affordance and footer-hint grammar (a numeric
    /// param tunes with `←→`, an action runs with `↵`, etc.).
    pub kind: ParamKind,
}

/// State of the ambient instrument surface.
///
/// `Base` is the idle resting state. `Tune` overrides it while a parameter is
/// being adjusted. `Msg` overrides everything (Error-level messages rank
/// highest).
#[derive(Clone, Debug)]
pub enum AmbientState {
    /// Idle status row — shows running metrics / preset label / palette swatch.
    Base,
    /// Parameter-focus overlay — shows a gauge for the focused param.
    Tune {
        /// The focused parameter.
        param: TuneView,
        /// Expires at this simulation time (seconds). Ignored when `sticky`.
        until: f32,
    },
    /// Notification message.
    Msg {
        /// Urgency level (drives icon + color).
        level: NotificationLevel,
        /// Message body.
        text: String,
        /// When `true` the message persists regardless of `until`.
        sticky: bool,
        /// Expires at this simulation time (seconds). Checked only when `!sticky`.
        until: f32,
    },
}

// ── Resolution ────────────────────────────────────────────────────────────────

/// Returns `true` when the entry is still live at `now`.
///
/// Public so the runner can prune expired non-sticky entries each frame while
/// keeping the always-live `Base` sentinel.
pub fn ambient_state_is_live(state: &AmbientState, now: f32) -> bool {
    match state {
        AmbientState::Base => true,
        AmbientState::Tune { until, .. } => now <= *until,
        AmbientState::Msg { sticky, until, .. } => *sticky || now <= *until,
    }
}

/// Priority rank — higher wins. Error MSG = 3, other MSG = 2, TUNE = 1, BASE = 0.
fn priority(state: &AmbientState) -> u8 {
    match state {
        AmbientState::Base => 0,
        AmbientState::Tune { .. } => 1,
        AmbientState::Msg {
            level: NotificationLevel::Error,
            ..
        } => 3,
        AmbientState::Msg { .. } => 2,
    }
}

/// Select the highest-priority live state from `states`.
///
/// Priority order (high → low): error-MSG > MSG > TUNE > BASE.
/// Expired non-sticky entries are skipped. There must be at least one `Base`
/// entry (it never expires); if `states` is empty a `Base` reference returned
/// from a static is not available, so callers should always include one.
///
/// # Panics
///
/// Panics if `states` is empty (no `Base` sentinel present).
pub fn resolve(states: &[AmbientState], now: f32) -> &AmbientState {
    states
        .iter()
        .filter(|s| ambient_state_is_live(s, now))
        .max_by_key(|s| priority(s))
        .expect("resolve: states slice must contain at least one entry (Base sentinel)")
}

/// Debounce/refresh the hold window of an active [`AmbientState::Tune`].
///
/// When `active` is a `Tune`, sets `until = now + hold` so each fresh adjust
/// extends the surfacing window (rapid adjusts keep the gauge live). When
/// `active` is any other variant this is a no-op — the caller is responsible
/// for replacing it with a fresh `Tune` first.
pub fn bump_tune(active: &mut AmbientState, now: f32, hold: f32) {
    if let AmbientState::Tune { until, .. } = active {
        *until = now + hold;
    }
}

/// Surface a focused-param `param` in the ambient `states`, debouncing any
/// existing `Tune`.
///
/// Changing a parameter must ALWAYS reveal its tuner. The redundant `Info`-level
/// value-echo toast that param-adjust handlers push (e.g. `"Motion blur: 3"`)
/// would otherwise outrank the `Tune` and hide it — the tuner already shows the
/// value + gauge, so that echo is pure double-feedback. We drop live `Info` Msgs
/// here. Higher-relevance Msgs (`Warning`/`Error`) are preserved: they remain
/// "specifically relevant enough" to momentarily hide the tuner while live.
///
/// At most one `Tune` entry is kept (an existing one is refreshed + debounced
/// via [`bump_tune`]); otherwise a fresh `Tune` is pushed.
pub fn surface_tune(states: &mut Vec<AmbientState>, param: TuneView, now: f32, hold: f32) {
    // Drop the redundant value-echo (Info) toast so it can't outrank the tuner.
    states.retain(|s| {
        !matches!(
            s,
            AmbientState::Msg {
                level: NotificationLevel::Info,
                ..
            }
        )
    });
    match states
        .iter_mut()
        .find(|s| matches!(s, AmbientState::Tune { .. }))
    {
        Some(slot) => {
            *slot = AmbientState::Tune { param, until: now };
            bump_tune(slot, now, hold);
        }
        None => {
            states.push(AmbientState::Tune {
                param,
                until: now + hold,
            });
        }
    }
}

// ── Renderer ──────────────────────────────────────────────────────────────────

/// Build the ambient instrument overlay for `state`.
///
/// Always emits exactly [`STRIP_H`] rows regardless of state (reserves strip
/// height so the terminal layout is stable). The strip width is `width` clamped
/// to at least [`STRIP_W`].
///
/// # Parameters
///
/// - `state`  — the resolved [`AmbientState`] to render.
/// - `width`  — terminal width in columns.
/// - `st`     — the active [`PanelStyle`] for colour tokens. No hardcoded RGB.
/// - `base`   — status context for the BASE arm; ignored for TUNE and MSG arms.
/// - `now`    — monotonic phase clock (seconds); drives the TUNE breath pulse.
///
/// Returns a [`RenderedOverlay`] with `title_box = None` and `rich_lines`
/// populated. The `lines` field carries the plain-text representation.
/// Content width of the centered ambient modal interior (excludes border).
const MODAL_INNER_W: usize = 56;

/// Build the single BASE status row (preset · time · swatch · palette · pop …
/// right-aligned undo/redo/paused/help), filling `w` columns. No border.
fn base_content_row(w: usize, st: &PanelStyle, base: &BaseStatus) -> RowBuf {
    let mut row = RowBuf::new_matte(w, st.status_bar_bg);
    let mut col = 2usize;

    // preset name
    row.put(col, &base.preset_name, Some(st.text_primary), None);
    col += base.preset_name.chars().count();

    // separator
    row.put(col, "  ◦  ", Some(st.muted), None);
    col += 5;

    // time scale
    row.put(col, &base.time_scale_text, Some(st.text_primary), None);
    col += base.time_scale_text.chars().count();

    // palette swatch + name (if space)
    if w >= 52 {
        row.put(col, "  ◦  ", Some(st.muted), None);
        col += 5;
        let sw = swatch(base.accent);
        let sw_w = sw.len();
        row.put_cells(col, &sw, None);
        col += sw_w;
        row.put(col, "  ", None, None);
        col += 2;
        row.put(col, &base.palette_name, Some(st.text_primary), None);
        col += base.palette_name.chars().count() + 2;
    }

    // population (if space)
    if let Some(pop) = base.population {
        if w >= 68 {
            let pop_str = format!("◦  {}k  ", pop / 1000);
            row.put(col, &pop_str, Some(st.muted), None);
            col += pop_str.chars().count();
        }
    }

    // dither label (if present + space)
    if let Some(ref dither) = base.dither_label {
        if w >= 60 {
            let d_str = format!("◦  {}  ", dither);
            row.put(col, &d_str, Some(st.muted), None);
            col += d_str.chars().count();
        }
    }

    // Right-side indicators: undo / redo / paused / ? help
    let mut right_parts: Vec<(String, RgbColor)> = Vec::new();

    if base.can_undo || base.can_redo {
        let undo_char = if base.can_undo { "↺" } else { "·" };
        let redo_char = if base.can_redo { "↻" } else { "·" };
        right_parts.push((undo_char.to_string(), st.accent_success));
        right_parts.push((" ".to_string(), st.text_primary));
        right_parts.push((redo_char.to_string(), st.accent_info));
        right_parts.push(("  ".to_string(), st.text_primary));
    }

    if base.is_paused {
        right_parts.push(("⏸ PAUSED  ".to_string(), st.accent_warning));
    }

    if w >= 100 {
        right_parts.push(("?".to_string(), st.accent_info));
        right_parts.push((" help  ".to_string(), st.muted));
    }

    // Place right-side indicators right-aligned
    let right_chars: usize = right_parts.iter().map(|(s, _)| s.chars().count()).sum();
    let right_start = w.saturating_sub(right_chars);

    if right_start > col {
        let mut rc = right_start;
        for (text, color) in &right_parts {
            row.put(rc, text, Some(*color), None);
            rc += text.chars().count();
        }
    }

    row
}

/// Build the TUNE content rows (label · value, gauge, hint), each `w` wide.
fn tune_content_rows(w: usize, st: &PanelStyle, param: &TuneView, now: f32) -> Vec<RowBuf> {
    // Subtle breathing pulse on the accent so the focused param feels
    // "live" while held. Stays within [1-depth, 1] (see motion::breath).
    let pulse = breath(now, 4.0, 0.15);
    let pulsed_accent = lerp_rgb(st.status_bar_bg, st.accent_active, pulse);
    let mut rows = Vec::with_capacity(3);

    // row: label   value/affordance. The label is always legible (a muted label
    // reads as disabled); the value carries the state color but stays readable —
    // Default folds to text_primary rather than the muted state token.
    {
        let mut row = RowBuf::new_matte(w, st.status_bar_bg);
        let lbl: String = param.label.chars().take(20).collect();
        let mut col = 2usize;
        row.put(col, &lbl, Some(st.text_primary), None);
        col += lbl.chars().count() + 3;
        match param.kind {
            // Actions carry no value — show the run affordance instead.
            ParamKind::Action => {
                row.put(col, "↵ run", Some(st.accent_active), None);
            }
            _ => {
                let val: String = param.value_text.chars().take(16).collect();
                row.put(col, &val, Some(value_color(param.state, st)), None);
            }
        }
        rows.push(row);
    }
    // row: gauge (value marker + default tick), accent breathing. Numeric params
    // only — enum/toggle params skip it (the value text carries the meaning and
    // an empty gauge would read as "minimum").
    if param.show_gauge {
        let mut row = RowBuf::new_matte(w, st.status_bar_bg);
        let gauge_w = (w.saturating_sub(4)).min(60);
        let mut g = gauge(param.value, param.range, param.default, gauge_w, st);
        for cell in g.iter_mut() {
            if cell.1 == st.accent_active {
                cell.1 = pulsed_accent;
            }
        }
        row.put_cells(2, &g, None);
        rows.push(row);
    }
    // row: hint — per-kind verb grammar, centered. A numeric param tunes with
    // ←→; an enum cycles; a toggle/action commits with ↵; read-only params can
    // only be picked past.
    {
        let mut row = RowBuf::new_matte(w, st.status_bar_bg);
        let hint = match param.kind {
            ParamKind::Numeric => footer_hints(&[("←→", "tune"), ("↑↓", "pick"), ("esc", "close")]),
            ParamKind::Enum => footer_hints(&[("←→", "cycle"), ("↑↓", "pick"), ("esc", "close")]),
            ParamKind::Toggle => footer_hints(&[("↵", "toggle"), ("↑↓", "pick"), ("esc", "close")]),
            ParamKind::Action => footer_hints(&[("↵", "run"), ("↑↓", "pick"), ("esc", "close")]),
            ParamKind::CliReadonly | ParamKind::Display => {
                footer_hints(&[("↑↓", "pick"), ("esc", "close")])
            }
        };
        let start = 2 + center_offset(&hint, w.saturating_sub(2));
        row.put(start, &hint, Some(st.muted), None);
        rows.push(row);
    }
    rows
}

/// Horizontal offset to center `text` within `width` columns (0 if it overflows).
fn center_offset(text: &str, width: usize) -> usize {
    width.saturating_sub(text.chars().count()) / 2
}

/// Greedy word-wrap `text` into lines no wider than `width` columns.
///
/// Breaks on ASCII spaces, never mid-word, so a notification reads as whole
/// words across lines instead of being amputated (e.g. "…on GitH"). A single
/// word longer than `width` is hard-broken into `width`-column chunks as a last
/// resort. Always returns at least one line (an empty line for empty input).
fn wrap_words(text: &str, width: usize) -> Vec<String> {
    if width == 0 {
        return vec![String::new()];
    }
    let mut lines: Vec<String> = Vec::new();
    let mut cur = String::new();
    let mut cur_len = 0usize;
    for word in text.split(' ') {
        let wlen = word.chars().count();
        // A single oversized word can't fit any line — flush, then hard-break it.
        if wlen > width {
            if !cur.is_empty() {
                lines.push(std::mem::take(&mut cur));
                cur_len = 0;
            }
            for ch in word.chars() {
                if cur_len == width {
                    lines.push(std::mem::take(&mut cur));
                    cur_len = 0;
                }
                cur.push(ch);
                cur_len += 1;
            }
            continue;
        }
        let sep = usize::from(!cur.is_empty());
        if cur_len + sep + wlen > width {
            lines.push(std::mem::take(&mut cur));
            cur.push_str(word);
            cur_len = wlen;
        } else {
            if sep == 1 {
                cur.push(' ');
            }
            cur.push_str(word);
            cur_len += sep + wlen;
        }
    }
    lines.push(cur);
    lines
}

/// Build the MSG content rows (icon + wrapped text, optional dismiss hint),
/// each `w` wide. Long messages wrap across rows so no word is truncated; the
/// icon sits on the first row and continuation rows align under the text.
fn msg_content_rows(
    w: usize,
    st: &PanelStyle,
    level: NotificationLevel,
    text: &str,
    sticky: bool,
) -> Vec<RowBuf> {
    let color = level_color(level, st);
    let icon = level.icon();
    let icon_w = icon.chars().count();
    let text_col = 2 + icon_w + 1;
    let avail = w.saturating_sub(icon_w + 4).max(1);
    let wrapped = wrap_words(text, avail);
    let mut rows = Vec::with_capacity(wrapped.len() + 1);
    for (i, line) in wrapped.iter().enumerate() {
        let mut row = RowBuf::new_matte(w, st.status_bar_bg);
        if i == 0 {
            row.put(2, icon, Some(color), None);
        }
        row.put(text_col, line, Some(color), None);
        rows.push(row);
    }
    {
        let mut row = RowBuf::new_matte(w, st.status_bar_bg);
        if sticky && matches!(level, NotificationLevel::Error) {
            let hint = footer_hints(&[("esc", "dismiss")]);
            let start = 2 + center_offset(&hint, w.saturating_sub(2));
            row.put(start, &hint, Some(st.muted), None);
        }
        rows.push(row);
    }
    rows
}

/// Build the PAUSE content rows (icon + label, resume hint), each `w` wide.
fn pause_content_rows(w: usize, st: &PanelStyle) -> Vec<RowBuf> {
    let mut rows = Vec::with_capacity(2);
    {
        let mut row = RowBuf::new_matte(w, st.status_bar_bg);
        row.put(2, "⏸  Paused", Some(st.accent_warning), None);
        rows.push(row);
    }
    {
        let mut row = RowBuf::new_matte(w, st.status_bar_bg);
        let hint = "space to resume";
        let start = 2 + center_offset(hint, w.saturating_sub(2));
        row.put(start, hint, Some(st.muted), None);
        rows.push(row);
    }
    rows
}

/// Full-width strip renderer: top border line, state content, bottom margin —
/// always exactly `STRIP_H` rows. The live render path now uses the compact
/// [`build_base_row`] (always-on status) and the centered [`build_ambient_modal`]
/// (TUNE/MSG/pause) instead; this reference renderer is retained as the canonical
/// exercise of the shared `*_content_row(s)` helpers under test.
pub fn build_ambient(
    state: &AmbientState,
    width: usize,
    st: &PanelStyle,
    base: &BaseStatus,
    now: f32,
) -> RenderedOverlay {
    let w = width.max(STRIP_W);
    let mut bufs: Vec<RowBuf> = Vec::with_capacity(STRIP_H);

    // row 0: dim top-border line
    {
        let mut border = RowBuf::new_matte(w, st.status_bar_bg);
        for c in 0..w {
            border.put(c, "▔", Some(st.border_color), None);
        }
        bufs.push(border);
    }

    // Content rows (1..=3 depending on state), padded out below to STRIP_H-1.
    let mut content: Vec<RowBuf> = match state {
        AmbientState::Base => vec![base_content_row(w, st, base)],
        AmbientState::Tune { param, .. } => tune_content_rows(w, st, param, now),
        AmbientState::Msg {
            level,
            text,
            sticky,
            ..
        } => msg_content_rows(w, st, *level, text, *sticky),
    };
    bufs.append(&mut content);

    // Pad to STRIP_H-1 content rows, then the bottom margin row.
    while bufs.len() < STRIP_H - 1 {
        bufs.push(RowBuf::new_matte(w, st.status_bar_bg));
    }
    bufs.push(RowBuf::new_matte(w, st.status_bar_bg));

    debug_assert_eq!(
        bufs.len(),
        STRIP_H,
        "build_ambient must always emit STRIP_H rows"
    );

    let lines: Vec<String> = bufs.iter().map(|b| b.text()).collect();
    let rich_lines: Vec<Vec<RichCell>> = bufs.into_iter().map(|b| b.into_rich()).collect();
    RenderedOverlay {
        lines,
        title_box: None,
        rich_lines: Some(rich_lines),
    }
}

/// Build the compact, single-row BASE status line at `width` columns (no
/// border). Docked at the bottom interior of the window frame when the
/// always-on status line is enabled.
pub fn build_base_row(width: usize, st: &PanelStyle, base: &BaseStatus) -> RenderedOverlay {
    let row = base_content_row(width, st, base);
    let line = row.text();
    let rich = row.into_rich();
    RenderedOverlay {
        lines: vec![line],
        title_box: None,
        rich_lines: Some(vec![rich]),
    }
}

/// A horizontal modal border row (`left` + `mid`×inner + `right`), `inner+2`
/// wide, tinted with the theme's `border_color` over the panel `bg_color` — so
/// the modal frame + interior match the Controls/Dashboard panels exactly.
fn modal_border_row(left: &str, mid: &str, right: &str, inner: usize, st: &PanelStyle) -> RowBuf {
    let mut row = RowBuf::new_matte(inner + 2, st.bg_color);
    row.put(0, left, Some(st.border_color), None);
    for c in 1..=inner {
        row.put(c, mid, Some(st.border_color), None);
    }
    row.put(inner + 1, right, Some(st.border_color), None);
    row
}

/// Build the ambient surface as a bordered, centered modal for the TUNE / MSG /
/// pause states. Width is fixed at `MODAL_INNER_W + 2`; height varies with the
/// state's content (plus one blank padding row top and bottom).
///
/// The border uses solid-block glyphs (`█ ▀ ▄`) that fill the cell to its edges
/// — matching the Dashboard/Controls panels — and is tinted with the theme's
/// `border_color`. The interior is matted with the panel `bg_color`, so the
/// modal's frame outline and background read identically to the Controls
/// console rather than as a separate chrome layer.
pub fn build_ambient_modal(state: &AmbientState, st: &PanelStyle, now: f32) -> RenderedOverlay {
    let inner = MODAL_INNER_W;

    // Resolve the content rows for this modal. A resting Base state only reaches
    // here when paused, so render the pause card.
    let content: Vec<RowBuf> = match state {
        AmbientState::Tune { param, .. } => tune_content_rows(inner, st, param, now),
        AmbientState::Msg {
            level,
            text,
            sticky,
            ..
        } => msg_content_rows(inner, st, *level, text, *sticky),
        AmbientState::Base => pause_content_rows(inner, st),
    };

    // Assemble: solid-block top border, blank pad, content, blank pad, bottom
    // border. Half-block top (▀) / bottom (▄) lines reach the cell edges so the
    // modal has no empty ring; full-block (█) corners and sides frame it.
    let mut bufs: Vec<RowBuf> = Vec::with_capacity(content.len() + 4);
    bufs.push(modal_border_row("█", "▀", "█", inner, st));
    let mut inner_rows: Vec<RowBuf> = Vec::with_capacity(content.len() + 2);
    inner_rows.push(RowBuf::new_matte(inner, st.bg_color));
    inner_rows.extend(content);
    inner_rows.push(RowBuf::new_matte(inner, st.bg_color));
    for r in inner_rows {
        let cells = r.into_rich();
        let mut line = RowBuf::new_matte(inner + 2, st.bg_color);
        line.put(0, "█", Some(st.border_color), None);
        // blit content cells at column 1
        for (i, (ch, fg, _bg)) in cells.into_iter().enumerate() {
            let mut tmp = [0u8; 4];
            line.put(1 + i, ch.encode_utf8(&mut tmp), fg, None);
        }
        line.put(inner + 1, "█", Some(st.border_color), None);
        bufs.push(line);
    }
    bufs.push(modal_border_row("█", "▄", "█", inner, st));

    let lines: Vec<String> = bufs.iter().map(|b| b.text()).collect();
    let rich_lines: Vec<Vec<RichCell>> = bufs.into_iter().map(|b| b.into_rich()).collect();
    RenderedOverlay {
        lines,
        title_box: None,
        rich_lines: Some(rich_lines),
    }
}

/// Construct a [`AmbientState::Msg`] with the correct per-level duration.
///
/// Durations are deliberately short — the message renders as a centered modal
/// over the sim, so it must clear quickly rather than linger in the eyeline.
///
/// - `Info` / `Success` → 1.5 seconds.
/// - `Warning` → 2.5 seconds.
/// - `Error` → sticky (`until = f32::INFINITY`).
///
/// `now` should be `runtime_state.phase_clock`.
pub fn msg(level: NotificationLevel, text: String, now: f32) -> AmbientState {
    match level {
        NotificationLevel::Info | NotificationLevel::Success => AmbientState::Msg {
            level,
            text,
            sticky: false,
            until: now + 1.5,
        },
        NotificationLevel::Warning => AmbientState::Msg {
            level,
            text,
            sticky: false,
            until: now + 2.5,
        },
        NotificationLevel::Error => AmbientState::Msg {
            level,
            text,
            sticky: true,
            until: f32::INFINITY,
        },
    }
}

/// Map a [`NotificationLevel`] to its theme color token.
fn level_color(level: NotificationLevel, st: &PanelStyle) -> crate::render::palette::RgbColor {
    match level {
        NotificationLevel::Info => st.accent_info,
        NotificationLevel::Success => st.accent_success,
        NotificationLevel::Warning => st.accent_warning,
        NotificationLevel::Error => st.accent_error,
    }
}

// ── Tests ──────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod ambient_tests {
    use super::*;

    fn tune_stub() -> TuneView {
        TuneView {
            label: "Stub Param".to_string(),
            value_text: "0.5".to_string(),
            value: 0.5,
            range: (0.0, 1.0),
            default: 0.5,
            state: ParamState::Default,
            show_gauge: true,
            kind: ParamKind::Numeric,
        }
    }

    #[test]
    fn error_msg_outranks_everything() {
        let states = vec![
            AmbientState::Base,
            AmbientState::Tune {
                param: tune_stub(),
                until: 100.0,
            },
            AmbientState::Msg {
                level: NotificationLevel::Error,
                text: "boom".into(),
                sticky: true,
                until: f32::INFINITY,
            },
        ];
        assert!(matches!(
            resolve(&states, 1.0),
            AmbientState::Msg {
                level: NotificationLevel::Error,
                ..
            }
        ));
    }

    #[test]
    fn expired_tune_falls_back_to_base() {
        let states = vec![
            AmbientState::Base,
            AmbientState::Tune {
                param: tune_stub(),
                until: 5.0,
            },
        ];
        assert!(matches!(resolve(&states, 6.0), AmbientState::Base)); // now > until
    }

    #[test]
    fn nonerror_msg_outranks_tune_while_live() {
        let states = vec![
            AmbientState::Tune {
                param: tune_stub(),
                until: 100.0,
            },
            AmbientState::Msg {
                level: NotificationLevel::Success,
                text: "saved".into(),
                sticky: false,
                until: 10.0,
            },
        ];
        assert!(matches!(resolve(&states, 1.0), AmbientState::Msg { .. }));
    }

    // ── build_ambient always-STRIP_H invariant ────────────────────────────────

    #[test]
    fn build_ambient_base_emits_strip_h_rows() {
        let st = crate::render::theme::GRUVBOX_DARK;
        let ov = build_ambient(
            &AmbientState::Base,
            STRIP_W,
            &st,
            &BaseStatus::default(),
            0.0,
        );
        assert_eq!(ov.lines.len(), STRIP_H);
        assert_eq!(ov.rich_lines.unwrap().len(), STRIP_H);
    }

    #[test]
    fn build_ambient_tune_emits_strip_h_rows() {
        let st = crate::render::theme::GRUVBOX_DARK;
        let state = AmbientState::Tune {
            param: tune_stub(),
            until: 100.0,
        };
        let ov = build_ambient(&state, STRIP_W, &st, &BaseStatus::default(), 0.0);
        assert_eq!(ov.lines.len(), STRIP_H);
        assert_eq!(ov.rich_lines.unwrap().len(), STRIP_H);
    }

    #[test]
    fn build_ambient_msg_emits_strip_h_rows() {
        let st = crate::render::theme::GRUVBOX_DARK;
        let state = AmbientState::Msg {
            level: NotificationLevel::Error,
            text: "something broke".into(),
            sticky: true,
            until: f32::INFINITY,
        };
        let ov = build_ambient(&state, STRIP_W, &st, &BaseStatus::default(), 0.0);
        assert_eq!(ov.lines.len(), STRIP_H);
        assert_eq!(ov.rich_lines.unwrap().len(), STRIP_H);
    }

    // ── New windowed builders: compact base row + centered modal ───────────────

    #[test]
    fn build_base_row_is_single_row_at_width() {
        let st = crate::render::theme::GRUVBOX_DARK;
        let ov = build_base_row(80, &st, &BaseStatus::default());
        assert_eq!(ov.lines.len(), 1, "base row must be exactly one line");
        assert_eq!(ov.lines[0].chars().count(), 80, "base row fills its width");
        assert_eq!(ov.rich_lines.as_ref().unwrap().len(), 1);
    }

    #[test]
    fn build_ambient_modal_is_bordered_box() {
        let st = crate::render::theme::GRUVBOX_DARK;
        let state = AmbientState::Msg {
            level: NotificationLevel::Info,
            text: "saved".into(),
            sticky: false,
            until: 10.0,
        };
        let ov = build_ambient_modal(&state, &st, 0.0);
        let top = &ov.lines[0];
        let bottom = ov.lines.last().unwrap();
        // All rows share a fixed modal width (inner + 2 borders).
        let w = MODAL_INNER_W + 2;
        assert!(ov.lines.iter().all(|l| l.chars().count() == w));
        // Solid-block borders reach the cell edges (no thin box-drawing ring).
        assert!(top.starts_with('█') && top.ends_with('█'), "top border");
        assert!(top[3..top.len() - 3].contains('▀'), "top half-block fill");
        assert!(
            bottom.starts_with('█') && bottom.ends_with('█'),
            "bottom border"
        );
        assert!(
            bottom[3..bottom.len() - 3].contains('▄'),
            "bottom half-block fill"
        );
        // Interior rows are side-bordered with full blocks.
        assert!(ov.lines[1..ov.lines.len() - 1]
            .iter()
            .all(|l| l.starts_with('█') && l.ends_with('█')));
    }

    #[test]
    fn build_ambient_modal_pause_card_from_base() {
        let st = crate::render::theme::GRUVBOX_DARK;
        // A Base state rendered as a modal is the pause card.
        let ov = build_ambient_modal(&AmbientState::Base, &st, 0.0);
        let joined = ov.lines.join("\n");
        assert!(joined.contains("Paused"), "pause card shows Paused");
    }

    #[test]
    fn build_ambient_modal_frame_and_bg_match_controls_panel() {
        // The modal frame outline + background must use the same theme tokens as
        // the Controls/Dashboard panels (border_color over bg_color), NOT the
        // palette accent over status_bar_bg.
        let st = crate::render::theme::GRUVBOX_DARK;
        let state = AmbientState::Msg {
            level: NotificationLevel::Info,
            text: "saved".into(),
            sticky: false,
            until: 10.0,
        };
        let ov = build_ambient_modal(&state, &st, 0.0);
        let rich = ov.rich_lines.expect("modal has rich lines");

        // Every block-glyph border cell is tinted with border_color (frame),
        // and every cell — border and interior — is matted with bg_color.
        for row in &rich {
            for (ch, fg, bg) in row {
                if matches!(ch, '█' | '▀' | '▄') {
                    assert_eq!(*fg, Some(st.border_color), "frame glyph uses border_color");
                }
                assert_eq!(
                    *bg,
                    Some(st.bg_color),
                    "modal interior matted with bg_color"
                );
            }
        }
    }

    // ── Additional resolve edge cases ─────────────────────────────────────────

    #[test]
    fn expired_non_sticky_msg_falls_back_to_base() {
        let states = vec![
            AmbientState::Base,
            AmbientState::Msg {
                level: NotificationLevel::Info,
                text: "gone".into(),
                sticky: false,
                until: 3.0,
            },
        ];
        assert!(matches!(resolve(&states, 10.0), AmbientState::Base));
    }

    #[test]
    fn sticky_msg_survives_past_until() {
        let states = vec![
            AmbientState::Base,
            AmbientState::Msg {
                level: NotificationLevel::Warning,
                text: "sticky".into(),
                sticky: true,
                until: 1.0,
            },
        ];
        assert!(matches!(resolve(&states, 999.0), AmbientState::Msg { .. }));
    }

    #[test]
    fn build_ambient_tune_contains_label() {
        let st = crate::render::theme::GRUVBOX_DARK;
        let state = AmbientState::Tune {
            param: tune_stub(),
            until: 100.0,
        };
        let ov = build_ambient(&state, STRIP_W, &st, &BaseStatus::default(), 0.0);
        let combined: String = ov.lines.concat();
        assert!(
            combined.contains("Stub Param"),
            "TUNE should render the param label"
        );
    }

    #[test]
    fn build_ambient_msg_contains_icon_and_text() {
        let st = crate::render::theme::GRUVBOX_DARK;
        let state = AmbientState::Msg {
            level: NotificationLevel::Success,
            text: "saved".into(),
            sticky: false,
            until: 10.0,
        };
        let ov = build_ambient(&state, STRIP_W, &st, &BaseStatus::default(), 0.0);
        let combined: String = ov.lines.concat();
        assert!(combined.contains("saved"), "MSG should render text");
        assert!(combined.contains('✓'), "Success MSG should render ✓ icon");
    }

    // ── Task 12: BASE status uses L2 tokens — Nord colors differ from Gruvbox ──

    /// Collect all fg RgbColor values from the rich_lines of the first row (row 1)
    /// that are not None — these are the status indicator colors.
    fn collect_fg_colors(
        rich_lines: &[Vec<crate::render::panel::RichCell>],
    ) -> Vec<crate::render::palette::RgbColor> {
        rich_lines
            .iter()
            .flat_map(|row| row.iter().filter_map(|(_, fg, _)| *fg))
            .collect()
    }

    #[test]
    fn base_status_gruvbox_and_nord_differ_in_colors() {
        // Render BASE status on two different themes with a recognizable accent color.
        // Because both themes route undo/redo/help through their own token colors,
        // the fg color sets MUST differ — proving the per-theme bug is fixed.
        let base = BaseStatus {
            preset_name: "Organic".to_string(),
            palette_name: "Forest".to_string(),
            time_scale_text: "1.0×".to_string(),
            population: Some(50_000),
            dither_label: None,
            can_undo: true,
            can_redo: true,
            accent: crate::render::palette::RgbColor {
                r: 200,
                g: 100,
                b: 50,
            },
            is_paused: false,
        };

        let gruvbox = crate::render::theme::GRUVBOX_DARK;
        let nord = crate::render::theme::NORD;

        // Use a wide terminal so ? help is rendered (requires ≥100 cols)
        let width = 120;

        let ov_gruvbox = build_ambient(&AmbientState::Base, width, &gruvbox, &base, 0.0);
        let ov_nord = build_ambient(&AmbientState::Base, width, &nord, &base, 0.0);

        let colors_gruvbox = collect_fg_colors(&ov_gruvbox.rich_lines.unwrap());
        let colors_nord = collect_fg_colors(&ov_nord.rich_lines.unwrap());

        // The token colors must differ between themes — if they don't,
        // the hardcoded RGB is still leaking.
        assert_ne!(
            colors_gruvbox, colors_nord,
            "NORD and GRUVBOX_DARK BASE status must use different fg colors (token-driven)"
        );

        // Spot-check: ↺ uses accent_success, which differs between themes
        // Gruvbox accent_success = #B8BB26 (yellowish green)
        // Nord    accent_success = #A3BE8C (green)
        assert_ne!(
            gruvbox.accent_success, nord.accent_success,
            "Test precondition: themes must have different accent_success"
        );

        // Verify gruvbox uses its token for accent_success (undo indicator)
        assert!(
            colors_gruvbox.contains(&gruvbox.accent_success),
            "Gruvbox BASE should use gruvbox.accent_success for ↺: {:?}",
            colors_gruvbox
        );

        // Verify nord uses its token for accent_success
        assert!(
            colors_nord.contains(&nord.accent_success),
            "Nord BASE should use nord.accent_success for ↺: {:?}",
            colors_nord
        );
    }
}

#[cfg(test)]
mod tune_tests {
    use super::*;

    fn tune_stub() -> TuneView {
        TuneView {
            label: "Stub Param".to_string(),
            value_text: "0.5".to_string(),
            value: 0.5,
            range: (0.0, 1.0),
            default: 0.5,
            state: ParamState::Default,
            show_gauge: true,
            kind: ParamKind::Numeric,
        }
    }

    #[test]
    fn rapid_adjust_extends_hold() {
        let mut active = AmbientState::Tune {
            param: tune_stub(),
            until: 2.5,
        };
        bump_tune(&mut active, /*now*/ 2.0, /*hold*/ 2.5); // adjust again at t=2.0
        if let AmbientState::Tune { until, .. } = active {
            assert!((until - 4.5).abs() < 1e-6); // 2.0 + 2.5
        } else {
            panic!("expected Tune");
        }
    }

    #[test]
    fn surface_tune_pushes_when_absent() {
        let mut states = vec![AmbientState::Base];
        surface_tune(&mut states, tune_stub(), 1.0, 2.5);
        assert_eq!(
            states
                .iter()
                .filter(|s| matches!(s, AmbientState::Tune { .. }))
                .count(),
            1,
            "a fresh Tune is pushed"
        );
    }

    #[test]
    fn surface_tune_debounces_existing_tune() {
        let mut states = vec![
            AmbientState::Base,
            AmbientState::Tune {
                param: tune_stub(),
                until: 2.0,
            },
        ];
        surface_tune(&mut states, tune_stub(), 3.0, 2.5);
        // Still exactly one Tune, hold extended to now+hold.
        let tunes: Vec<_> = states
            .iter()
            .filter_map(|s| match s {
                AmbientState::Tune { until, .. } => Some(*until),
                _ => None,
            })
            .collect();
        assert_eq!(tunes.len(), 1, "no duplicate Tune");
        assert!((tunes[0] - 5.5).abs() < 1e-6, "hold refreshed to now+hold");
    }

    #[test]
    fn surface_tune_drops_redundant_info_echo() {
        // The value-echo Info toast must not survive to outrank the tuner.
        let mut states = vec![
            AmbientState::Base,
            AmbientState::Msg {
                level: NotificationLevel::Info,
                text: "Motion blur: 3".into(),
                sticky: false,
                until: 100.0,
            },
        ];
        surface_tune(&mut states, tune_stub(), 1.0, 2.5);
        assert!(
            !states.iter().any(|s| matches!(s, AmbientState::Msg { .. })),
            "redundant Info echo dropped"
        );
        // The tuner now wins resolution.
        assert!(matches!(resolve(&states, 1.0), AmbientState::Tune { .. }));
    }

    #[test]
    fn surface_tune_keeps_relevant_warning() {
        // A Warning is "relevant enough" — it survives and outranks the tuner.
        let mut states = vec![
            AmbientState::Base,
            AmbientState::Msg {
                level: NotificationLevel::Warning,
                text: "Brightness is auto-normalized".into(),
                sticky: false,
                until: 100.0,
            },
        ];
        surface_tune(&mut states, tune_stub(), 1.0, 2.5);
        assert!(
            states.iter().any(|s| matches!(
                s,
                AmbientState::Msg {
                    level: NotificationLevel::Warning,
                    ..
                }
            )),
            "relevant Warning preserved"
        );
        assert!(matches!(resolve(&states, 1.0), AmbientState::Msg { .. }));
    }

    #[test]
    fn enum_param_omits_gauge_row() {
        // A non-numeric param (show_gauge=false) renders label+hint but no gauge
        // bar, so an empty gauge can't read as "minimum value".
        let st = crate::render::theme::GRUVBOX_DARK;
        let mut enum_view = tune_stub();
        enum_view.label = "Intensity".into();
        enum_view.value_text = "Exponential".into();
        enum_view.show_gauge = false;
        let state = AmbientState::Tune {
            param: enum_view,
            until: 100.0,
        };
        let ov = build_ambient_modal(&state, &st, 0.0);
        let combined: String = ov.lines.concat();
        assert!(combined.contains("Intensity"), "enum renders label");
        assert!(combined.contains("Exponential"), "enum renders value text");
        assert!(combined.contains("←→ tune"), "enum keeps the tune hint");
        // The gauge default tick (▲) and filled bar (█ outside the modal border)
        // must not appear in the interior content. Borders use █, so check the ▲.
        assert!(
            !combined.contains('▲'),
            "enum param must not render a gauge tick"
        );
    }

    #[test]
    fn bump_tune_no_op_on_non_tune() {
        // bump_tune only refreshes a Tune; on Base/Msg it leaves the state alone.
        let mut base = AmbientState::Base;
        bump_tune(&mut base, 10.0, 2.5);
        assert!(matches!(base, AmbientState::Base));
    }

    #[test]
    fn build_ambient_tune_contains_value_and_gauge() {
        let st = crate::render::theme::GRUVBOX_DARK;
        let state = AmbientState::Tune {
            param: TuneView {
                label: "Sensor Angle".to_string(),
                value_text: "45.0°".to_string(),
                value: 0.5,
                range: (0.0, 1.0),
                default: 0.3,
                state: ParamState::Modified,
                show_gauge: true,
                kind: ParamKind::Numeric,
            },
            until: 100.0,
        };
        let ov = build_ambient(&state, STRIP_W, &st, &BaseStatus::default(), 0.0);
        let combined: String = ov.lines.concat();
        assert!(combined.contains("Sensor Angle"), "TUNE renders label");
        assert!(combined.contains("45.0°"), "TUNE renders formatted value");
        // gauge glyphs: filled bar, value marker, and default tick.
        assert!(combined.contains('█'), "gauge filled bar");
        assert!(combined.contains('▲'), "gauge default tick");
    }
}

#[cfg(test)]
mod msg_tests {
    use super::*;
    #[test]
    fn error_is_sticky() {
        let m = AmbientState::Msg {
            level: NotificationLevel::Error,
            text: "x".into(),
            sticky: true,
            until: f32::INFINITY,
        };
        // never expires under resolve
        let states = vec![AmbientState::Base, m];
        assert!(matches!(resolve(&states, 1e9), AmbientState::Msg { .. }));
    }
    #[test]
    fn success_expires_after_3s() {
        let states = vec![
            AmbientState::Base,
            AmbientState::Msg {
                level: NotificationLevel::Success,
                text: "s".into(),
                sticky: false,
                until: 3.0,
            },
        ];
        assert!(matches!(resolve(&states, 3.5), AmbientState::Base));
    }
    #[test]
    fn msg_constructor_info_expires_in_1p5s() {
        let m = msg(NotificationLevel::Info, "hi".into(), 0.0);
        if let AmbientState::Msg { sticky, until, .. } = m {
            assert!(!sticky);
            assert!((until - 1.5).abs() < 1e-6);
        } else {
            panic!("expected Msg");
        }
    }
    #[test]
    fn msg_constructor_warning_expires_in_2p5s() {
        let m = msg(NotificationLevel::Warning, "warn".into(), 0.0);
        if let AmbientState::Msg { sticky, until, .. } = m {
            assert!(!sticky);
            assert!((until - 2.5).abs() < 1e-6);
        } else {
            panic!("expected Msg");
        }
    }
    #[test]
    fn msg_constructor_error_is_sticky() {
        let m = msg(NotificationLevel::Error, "err".into(), 0.0);
        if let AmbientState::Msg { sticky, until, .. } = m {
            assert!(sticky);
            assert_eq!(until, f32::INFINITY);
        } else {
            panic!("expected Msg");
        }
    }

    // ── Notification word-wrap (no mid-word truncation) ────────────────────────

    #[test]
    fn wrap_words_keeps_short_text_on_one_line() {
        assert_eq!(wrap_words("hi there", 20), vec!["hi there".to_string()]);
    }

    #[test]
    fn wrap_words_breaks_on_word_boundary_not_mid_word() {
        // Regression: the dither dev-only toast (53 chars) used to truncate to
        // "…on GitH" inside a 51-col message area. Now it wraps, intact.
        let text = "Dither is dev-only - see help-wanted issues on GitHub";
        let lines = wrap_words(text, 51);
        assert!(lines.len() >= 2, "long text must wrap: {lines:?}");
        assert!(
            lines.iter().all(|l| l.chars().count() <= 51),
            "no line exceeds the width: {lines:?}"
        );
        // The trailing word survives whole — no "GitH" amputation.
        assert!(
            lines.iter().any(|l| l.contains("GitHub")),
            "GitHub must appear intact: {lines:?}"
        );
        // Reassembling the words reproduces the original message (lossless).
        assert_eq!(lines.join(" "), text);
    }

    #[test]
    fn wrap_words_hard_breaks_an_oversized_word() {
        let lines = wrap_words("supercalifragilistic", 5);
        assert!(lines.iter().all(|l| l.chars().count() <= 5));
        assert_eq!(lines.concat(), "supercalifragilistic");
    }

    #[test]
    fn wrap_words_empty_yields_one_empty_line() {
        assert_eq!(wrap_words("", 10), vec![String::new()]);
    }

    #[test]
    fn msg_content_rows_does_not_truncate_github() {
        let st = crate::render::theme::GRUVBOX_DARK;
        let text = "Dither is dev-only - see help-wanted issues on GitHub";
        let rows = msg_content_rows(MODAL_INNER_W, &st, NotificationLevel::Info, text, false);
        // Concatenated message-row text contains the full word, not "GitH".
        let joined: String = rows.iter().map(|r| r.text()).collect();
        assert!(
            joined.contains("GitHub"),
            "rendered notification must not amputate GitHub:\n{joined}"
        );
    }
}
