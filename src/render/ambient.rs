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

use crate::render::palette::RgbColor;
use crate::render::panel::{RenderedOverlay, RichCell};
use crate::render::theme::PanelStyle;
use crate::render::motion::{breath, lerp_rgb};
use crate::render::widgets::{gauge, swatch, value_state, ParamState, RowBuf};
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
pub fn build_ambient(
    state: &AmbientState,
    width: usize,
    st: &PanelStyle,
    base: &BaseStatus,
    now: f32,
) -> RenderedOverlay {
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
            // ── row 1: preset · time_scale · swatch · palette ─────────────────
            {
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

                bufs.push(row);
            }
            // ── rows 2-4: blank content rows ─────────────────────────────────
            for _ in 0..3 {
                bufs.push(RowBuf::new_matte(w, st.status_bar_bg));
            }
        }

        // ── TUNE: parameter-focus gauge ───────────────────────────────────────
        AmbientState::Tune { param, .. } => {
            // Subtle breathing pulse on the accent so the focused param feels
            // "live" while held. Stays within [1-depth, 1] (see motion::breath).
            let pulse = breath(now, 4.0, 0.15);
            let pulsed_accent = lerp_rgb(st.status_bar_bg, st.accent_active, pulse);

            // row 1: label  ·  value
            {
                let mut row = RowBuf::new_matte(w, st.status_bar_bg);
                let lbl: String = param.label.chars().take(20).collect();
                let lbl_cells = value_state(&lbl, param.state, st);
                let mut col = 2usize;
                row.put_cells(col, &lbl_cells, None);
                col += lbl.chars().count();
                row.put(col, "   ", None, None);
                col += 3;
                let val: String = param.value_text.chars().take(16).collect();
                let val_cells = value_state(&val, param.state, st);
                row.put_cells(col, &val_cells, None);
                bufs.push(row);
            }
            // row 2: gauge (value marker + default tick), accent breathing
            {
                let mut row = RowBuf::new_matte(w, st.status_bar_bg);
                let gauge_w = (w.saturating_sub(4)).min(60);
                let mut g = gauge(param.value, param.range, param.default, gauge_w, st);
                // Apply the breath pulse to the accent cells (filled bar + value
                // marker) so the gauge gently pulses without touching the muted
                // track or the default tick.
                for cell in g.iter_mut() {
                    if cell.1 == st.accent_active {
                        cell.1 = pulsed_accent;
                    }
                }
                row.put_cells(2, &g, None);
                bufs.push(row);
            }
            // row 3: blank spacer
            bufs.push(RowBuf::new_matte(w, st.status_bar_bg));
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
            value_text: "0.5".to_string(),
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
        let ov = build_ambient(&AmbientState::Base, STRIP_W, &st, &BaseStatus::default(), 0.0);
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
