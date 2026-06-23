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

use crate::render::panel::{RenderedOverlay, RichCell};
use crate::render::theme::PanelStyle;
use crate::render::widgets::{gauge, swatch, value_state, ParamState, RowBuf};
use crate::terminal::state::NotificationLevel;

// ── Layout constants ──────────────────────────────────────────────────────────

/// Number of rows in the ambient strip (matching tuner height).
const STRIP_H: usize = 6;

/// Minimum strip width in characters.
const STRIP_W: usize = 80;

// ── Public types ──────────────────────────────────────────────────────────────

/// A focused-param view for the TUNE state.
#[derive(Clone, Debug)]
pub struct TuneView {
    /// Short human-readable label, e.g. `"Sensor Angle"`.
    pub label: String,
    /// Current parameter value.
    pub value: f32,
    /// `(min, max)` range for the gauge.
    pub range: (f32, f32),
    /// Default value; shown as a reference marker on the gauge.
    pub default: f32,
    /// Functional state (drives color selection via L2 tokens).
    pub state: ParamState,
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
fn is_live(state: &AmbientState, now: f32) -> bool {
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
        .filter(|s| is_live(s, now))
        .max_by_key(|s| priority(s))
        .expect("resolve: states slice must contain at least one entry (Base sentinel)")
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
///
/// Returns a [`RenderedOverlay`] with `title_box = None` and `rich_lines`
/// populated. The `lines` field carries the plain-text representation.
pub fn build_ambient(state: &AmbientState, width: usize, st: &PanelStyle) -> RenderedOverlay {
    let w = width.max(STRIP_W);
    let mut bufs: Vec<RowBuf> = Vec::with_capacity(STRIP_H);

    // ── row 0: dim top-border line ────────────────────────────────────────────
    {
        let mut border = RowBuf::new_matte(w, st.status_bar_bg);
        for c in 0..w {
            border.put(c, "▔", Some(st.border_color), None);
        }
        bufs.push(border);
    }

    match state {
        // ── BASE: idle status ─────────────────────────────────────────────────
        AmbientState::Base => {
            // row 1: "BASE" label + palette swatch hint
            {
                let mut row = RowBuf::new_matte(w, st.status_bar_bg);
                row.put(2, "BASE", Some(st.muted), None);
                let sw = swatch(st.accent_active);
                row.put_cells(8, &sw, None);
                row.put(10, "ambient instrument", Some(st.muted), None);
                bufs.push(row);
            }
            // rows 2-4: blank
            for _ in 0..3 {
                bufs.push(RowBuf::new_matte(w, st.status_bar_bg));
            }
        }

        // ── TUNE: parameter-focus gauge ───────────────────────────────────────
        AmbientState::Tune { param, .. } => {
            // row 1: label
            {
                let mut row = RowBuf::new_matte(w, st.status_bar_bg);
                let lbl: String = param.label.chars().take(20).collect();
                let lbl_cells = value_state(&lbl, param.state, st);
                row.put_cells(2, &lbl_cells, None);
                bufs.push(row);
            }
            // row 2: gauge
            {
                let mut row = RowBuf::new_matte(w, st.status_bar_bg);
                let gauge_w = (w.saturating_sub(4)).min(60);
                let g = gauge(param.value, param.range, param.default, gauge_w, st);
                row.put_cells(2, &g, None);
                bufs.push(row);
            }
            // row 3: value text
            {
                let mut row = RowBuf::new_matte(w, st.status_bar_bg);
                let val = format!("{:.3}", param.value);
                let val_cells = value_state(&val, param.state, st);
                row.put_cells(2, &val_cells, None);
                bufs.push(row);
            }
            // row 4: hint
            {
                let mut row = RowBuf::new_matte(w, st.status_bar_bg);
                row.put(2, "←→ tune", Some(st.muted), None);
                bufs.push(row);
            }
        }

        // ── MSG: notification ─────────────────────────────────────────────────
        AmbientState::Msg { level, text, .. } => {
            // row 1: icon + text
            {
                let mut row = RowBuf::new_matte(w, st.status_bar_bg);
                let color = level_color(*level, st);
                let icon = level.icon();
                row.put(2, icon, Some(color), None);
                let icon_w = icon.chars().count();
                let msg: String = text.chars().take(w.saturating_sub(icon_w + 4)).collect();
                row.put(2 + icon_w + 1, &msg, Some(color), None);
                bufs.push(row);
            }
            // rows 2-4: blank
            for _ in 0..3 {
                bufs.push(RowBuf::new_matte(w, st.status_bar_bg));
            }
        }
    }

    // ── row 5: bottom margin ──────────────────────────────────────────────────
    bufs.push(RowBuf::new_matte(w, st.status_bar_bg));

    debug_assert_eq!(
        bufs.len(),
        STRIP_H,
        "build_ambient must always emit STRIP_H rows"
    );

    // ── Assemble ──────────────────────────────────────────────────────────────
    let lines: Vec<String> = bufs.iter().map(|b| b.text()).collect();
    let rich_lines: Vec<Vec<RichCell>> = bufs.into_iter().map(|b| b.into_rich()).collect();

    RenderedOverlay {
        lines,
        title_box: None,
        rich_lines: Some(rich_lines),
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
            value: 0.5,
            range: (0.0, 1.0),
            default: 0.5,
            state: ParamState::Default,
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
        let ov = build_ambient(&AmbientState::Base, STRIP_W, &st);
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
        let ov = build_ambient(&state, STRIP_W, &st);
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
        let ov = build_ambient(&state, STRIP_W, &st);
        assert_eq!(ov.lines.len(), STRIP_H);
        assert_eq!(ov.rich_lines.unwrap().len(), STRIP_H);
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
        let ov = build_ambient(&state, STRIP_W, &st);
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
        let ov = build_ambient(&state, STRIP_W, &st);
        let combined: String = ov.lines.concat();
        assert!(combined.contains("saved"), "MSG should render text");
        assert!(combined.contains('✓'), "Success MSG should render ✓ icon");
    }
}
