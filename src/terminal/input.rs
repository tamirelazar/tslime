//! Input handling for the terminal application.
//!
//! This module provides a non-blocking input poller that handles keyboard
//! and mouse events using `crossterm`, and maps keyboard events to control actions.

use crate::cli::Palette;
use crate::render::charset::Charset;
use crate::simulation::config::Preset;
use crate::terminal::state::ControlAction;
pub use crate::terminal::state::MousePosition;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, MouseEventKind};
use std::io;
use std::time::Duration;

/// The type of mouse interaction detected.
pub enum MouseEventType {
    /// Mouse button was pressed down.
    Down,
    /// Mouse is being dragged with a button held down.
    Drag,
    /// Mouse moved without buttons pressed.
    Moved,
}

/// Handles non-blocking input polling for keyboard and mouse events.
pub struct InputPoller {
    poll_timeout: Duration,
}

impl InputPoller {
    /// Create a new input poller with zero timeout (non-blocking).
    pub fn new() -> Self {
        Self {
            poll_timeout: Duration::from_millis(0),
        }
    }

    /// Set the timeout duration for polling operations.
    pub fn set_poll_timeout(&mut self, timeout: Duration) {
        self.poll_timeout = timeout;
    }

    /// Check for a keyboard event.
    ///
    /// Returns `Ok(Some(KeyEvent))` if a key was pressed within the timeout.
    /// Returns `Ok(None)` if no key was pressed.
    /// Returns `Err` if polling failed.
    pub fn poll_keypress(&self) -> io::Result<Option<KeyEvent>> {
        if event::poll(self.poll_timeout)? {
            if let Event::Key(key_event) = event::read()? {
                if key_event.kind == KeyEventKind::Press {
                    return Ok(Some(key_event));
                }
            }
        }
        Ok(None)
    }

    /// Check for a mouse event.
    ///
    /// Returns `Ok(Some((MousePosition, MouseEventType)))` if a relevant mouse event occurred.
    /// Ignores mouse up and scroll events for now.
    pub fn poll_mouse_event(&self) -> io::Result<Option<(MousePosition, MouseEventType)>> {
        if event::poll(self.poll_timeout)? {
            if let Event::Mouse(mouse_event) = event::read()? {
                let event_type = if matches!(mouse_event.kind, MouseEventKind::Down(_)) {
                    MouseEventType::Down
                } else if matches!(mouse_event.kind, MouseEventKind::Drag(_)) {
                    MouseEventType::Drag
                } else if matches!(mouse_event.kind, MouseEventKind::Moved) {
                    MouseEventType::Moved
                } else {
                    return Ok(None);
                };

                return Ok(Some((
                    MousePosition {
                        x: mouse_event.column.saturating_sub(1) as usize,
                        y: mouse_event.row.saturating_sub(1) as usize,
                    },
                    event_type,
                )));
            }
        }
        Ok(None)
    }

    /// Read all pending events from the input queue.
    ///
    /// Useful for flushing the input buffer or processing multiple events in one tick.
    pub fn drain_all_events(&self) -> io::Result<Vec<Event>> {
        let mut events = Vec::new();
        while event::poll(Duration::from_millis(0))? {
            events.push(event::read()?);
        }
        Ok(events)
    }

    /// Check if the given key event corresponds to an exit command (e.g., 'q' or 'Q').
    pub fn is_exit_key(key_event: &KeyEvent) -> bool {
        // Note: Esc is handled separately to close overlays first
        matches!(key_event.code, KeyCode::Char('q') | KeyCode::Char('Q'))
    }
}

impl Default for InputPoller {
    fn default() -> Self {
        Self::new()
    }
}

/// Maps a keyboard event to a control action.
pub fn handle_key_event(key_event: &KeyEvent) -> ControlAction {
    use crossterm::event::KeyModifiers;

    if key_event.modifiers.contains(KeyModifiers::CONTROL) {
        match key_event.code {
            KeyCode::Char('s') | KeyCode::Char('S') => return ControlAction::ShowConfigSaveDialog,
            KeyCode::Char('l') | KeyCode::Char('L') => return ControlAction::ShowConfigBrowser,
            KeyCode::Char('b') | KeyCode::Char('B') => return ControlAction::ShowConfigBrowser,
            KeyCode::Char('z') | KeyCode::Char('Z') => {
                if key_event.modifiers.contains(KeyModifiers::SHIFT) {
                    return ControlAction::Redo;
                } else {
                    return ControlAction::Undo;
                }
            }
            KeyCode::Char('y') | KeyCode::Char('Y') => return ControlAction::Redo,
            _ => {}
        }
    }

    match key_event.code {
        KeyCode::Char(' ') => ControlAction::TogglePause,
        KeyCode::Char('p') | KeyCode::Char('P') => ControlAction::ShowPaletteEditor,
        KeyCode::Char('r') | KeyCode::Char('R') => ControlAction::Restart,
        KeyCode::Char('1') => ControlAction::SetPreset(Preset::Network),
        KeyCode::Char('2') => ControlAction::SetPreset(Preset::Exploratory),
        KeyCode::Char('3') => ControlAction::SetPreset(Preset::Tendrils),
        KeyCode::Char('4') => ControlAction::SetPreset(Preset::Organic),
        KeyCode::Char('5') => ControlAction::SetPreset(Preset::Minimal),
        KeyCode::Char('6') => ControlAction::SetPreset(Preset::Moss),
        KeyCode::Char('7') => ControlAction::SetPreset(Preset::Zen),
        KeyCode::Char('!') => ControlAction::ComparePreset(Preset::Network),
        KeyCode::Char('@') => ControlAction::ComparePreset(Preset::Exploratory),
        KeyCode::Char('#') => ControlAction::ComparePreset(Preset::Tendrils),
        KeyCode::Char('$') => ControlAction::ComparePreset(Preset::Organic),
        KeyCode::Char('%') => ControlAction::ComparePreset(Preset::Minimal),
        KeyCode::Char('^') => ControlAction::ComparePreset(Preset::Moss),
        KeyCode::Char('&') => ControlAction::ComparePreset(Preset::Zen),
        KeyCode::Char('8') => ControlAction::RandomizeParams,
        KeyCode::Char('9') => ControlAction::CycleTheme,
        KeyCode::Char('*') => ControlAction::CycleThemeReverse,
        KeyCode::Char('+') | KeyCode::Char('=') => ControlAction::AdjustTimeScale(0.5),
        KeyCode::Char('-') | KeyCode::Char('_') => ControlAction::AdjustTimeScale(-0.5),
        KeyCode::Char('C') if key_event.modifiers.contains(KeyModifiers::SHIFT) => {
            ControlAction::CyclePaletteReverse
        }
        KeyCode::Char('c') => ControlAction::CyclePalette,
        KeyCode::Char('?') => ControlAction::ToggleKeyboardHints,
        KeyCode::Char('h') | KeyCode::Char('H') => ControlAction::ToggleControls,
        KeyCode::Esc => ControlAction::CloseOverlays,
        KeyCode::Char('d') => ControlAction::ToggleDither,
        KeyCode::Char('D') => ControlAction::CycleDitherMode,
        KeyCode::Char('m') => ControlAction::CycleIntensityMapping,
        KeyCode::Char('M') => ControlAction::CycleIntensityMappingReverse,
        KeyCode::Char('[') | KeyCode::Char('{') => ControlAction::AdjustDitherIntensity(-0.1),
        KeyCode::Char(']') | KeyCode::Char('}') => ControlAction::AdjustDitherIntensity(0.1),
        KeyCode::Char('q') | KeyCode::Char('Q') => ControlAction::Quit,
        KeyCode::Tab => ControlAction::CycleOptionsCategory,
        KeyCode::BackTab => ControlAction::CycleOptionsCategoryReverse,
        KeyCode::Char('A') | KeyCode::Char('a') => {
            if key_event.modifiers.contains(KeyModifiers::SHIFT) {
                ControlAction::AdjustSensorAngle(-1.0)
            } else {
                ControlAction::AdjustSensorAngle(1.0)
            }
        }
        KeyCode::Char('J') | KeyCode::Char('j') => {
            if key_event.modifiers.contains(KeyModifiers::SHIFT) {
                ControlAction::AdjustSensorDistance(-1.0)
            } else {
                ControlAction::AdjustSensorDistance(1.0)
            }
        }
        KeyCode::Char('T') | KeyCode::Char('t') => {
            if key_event.modifiers.contains(KeyModifiers::SHIFT) {
                ControlAction::AdjustTurnAngle(-1.0)
            } else {
                ControlAction::AdjustTurnAngle(1.0)
            }
        }
        KeyCode::Char('S') | KeyCode::Char('s') => {
            if key_event.modifiers.contains(KeyModifiers::SHIFT) {
                ControlAction::AdjustStepSize(-0.1)
            } else {
                ControlAction::AdjustStepSize(0.1)
            }
        }
        KeyCode::Char('E') | KeyCode::Char('e') => {
            if key_event.modifiers.contains(KeyModifiers::SHIFT) {
                ControlAction::AdjustDecay(-0.01)
            } else {
                ControlAction::AdjustDecay(0.01)
            }
        }
        KeyCode::Char('I') | KeyCode::Char('i') => {
            if key_event.modifiers.contains(KeyModifiers::SHIFT) {
                ControlAction::AdjustDeposit(-1.0)
            } else {
                ControlAction::AdjustDeposit(1.0)
            }
        }
        KeyCode::Char('K') | KeyCode::Char('k') => ControlAction::CycleDiffusionKernel,
        KeyCode::Char(';') | KeyCode::Char(':') => {
            if key_event.modifiers.contains(KeyModifiers::SHIFT) {
                ControlAction::AdjustDiffusionSigma(-0.1)
            } else {
                ControlAction::AdjustDiffusionSigma(0.1)
            }
        }
        KeyCode::Char('L') | KeyCode::Char('l') => {
            if key_event.modifiers.contains(KeyModifiers::SHIFT) {
                ControlAction::AdjustAttractorStrength(-0.5)
            } else {
                ControlAction::AdjustAttractorStrength(0.5)
            }
        }
        KeyCode::Char(',') | KeyCode::Char('<') => ControlAction::CycleMouseMode,
        KeyCode::Char('W') => ControlAction::CycleWindDirectionReverse,
        KeyCode::Char('w') => ControlAction::CycleWindDirection,
        KeyCode::Char('Y') | KeyCode::Char('y') => {
            if key_event.modifiers.contains(KeyModifiers::SHIFT) {
                ControlAction::AdjustTerrainStrength(-0.5)
            } else {
                ControlAction::AdjustTerrainStrength(0.5)
            }
        }
        KeyCode::Char('U') | KeyCode::Char('u') => ControlAction::CycleTerrainType,
        KeyCode::Char('B') | KeyCode::Char('b') => ControlAction::ToggleAutoNormalize,
        KeyCode::Char('V') | KeyCode::Char('v') => ControlAction::CycleMotionBlur,
        KeyCode::Char('N') | KeyCode::Char('n') => {
            // The control reads as brightness (up = brighter), but the engine
            // stores a normalization white-point that it *divides* by, so a
            // brighter image means a *lower* stored value. Un-shifted = brighter
            // (negative delta), Shift = dimmer (positive delta), matching the
            // increase/decrease convention of every other adjustable parameter.
            if key_event.modifiers.contains(KeyModifiers::SHIFT) {
                ControlAction::AdjustMaxBrightness(5.0)
            } else {
                ControlAction::AdjustMaxBrightness(-5.0)
            }
        }
        KeyCode::Char('G') | KeyCode::Char('g') => ControlAction::SaveFrameToPng,
        KeyCode::Char('F') | KeyCode::Char('f') => ControlAction::ToggleFastMode,
        KeyCode::Char('O') | KeyCode::Char('o') => ControlAction::CyclePaletteShiftSpeed,
        KeyCode::Char('X') | KeyCode::Char('x') => ControlAction::ToggleInvertPalette,
        KeyCode::Char('Z') | KeyCode::Char('z') => ControlAction::ToggleReversePalette,
        KeyCode::Char('0') => ControlAction::ResetToDefaults,
        KeyCode::Char('\\') | KeyCode::Char('|') => ControlAction::ToggleDashboard,
        KeyCode::Char('/') => ControlAction::ShowPaletteEditor,
        KeyCode::Char('`') => ControlAction::CycleCharset,
        KeyCode::Char('~') => ControlAction::CycleCharsetReverse,
        KeyCode::Char('\'') => ControlAction::ToggleTrailAge,
        KeyCode::Char('.') => ControlAction::ToggleTrailDelta,
        KeyCode::Char('>') => ControlAction::ToggleGradientMagnitude,
        KeyCode::Char('(') => ControlAction::CycleWindowFrameReverse,
        KeyCode::Char(')') => ControlAction::CycleWindowFrame,
        KeyCode::F(11) => ControlAction::ToggleFullscreen,
        #[cfg(feature = "audio")]
        KeyCode::F(2) => ControlAction::ToggleChoir,
        _ => ControlAction::None,
    }
}

/// Returns the display name of a preset.
pub fn preset_name(preset: Preset) -> &'static str {
    match preset {
        Preset::Network => "Network",
        Preset::Exploratory => "Exploratory",
        Preset::Tendrils => "Tendrils",
        Preset::Organic => "Organic",
        Preset::Minimal => "Minimal",
        Preset::Moss => "Moss",
        Preset::Cosmic => "Cosmic",
        Preset::Fire => "Fire",
        Preset::Zen => "Zen",
        Preset::Storm => "Storm",
        Preset::River => "River",
        Preset::Ethereal => "Ethereal",
        Preset::PetriDish => "PetriDish",
        Preset::Vortex => "Vortex",
        Preset::Lightning => "Lightning",
        Preset::Crystal => "Crystal",
        Preset::ChaosEdge => "ChaosEdge",
        Preset::Blob => "Blob",
        Preset::Worm => "Worm",
        Preset::Pulse => "Pulse",
        Preset::Coral => "Coral",
        Preset::Flocking => "Flocking",
        Preset::Maze => "Maze",
        Preset::Ripple => "Ripple",
        Preset::Vortex36 => "Vortex36",
        Preset::Chameleon => "Chameleon",
        Preset::DynamicTendrils => "DynamicTendrils",
        Preset::MorphingCoral => "MorphingCoral",
        Preset::ReactiveSwarm => "ReactiveSwarm",
        Preset::DuelingModulators => "DuelingModulators",
    }
}

/// Returns the display name of a palette.
pub fn palette_name(palette: Palette) -> &'static str {
    match palette {
        Palette::Organic => "Organic",
        Palette::Heat => "Heat",
        Palette::Ocean => "Ocean",
        Palette::Mono => "Mono",
        Palette::Forest => "Forest",
        Palette::Neon => "Neon",
        Palette::Warm => "Warm",
        Palette::Vibrant => "Vibrant",
        Palette::LegibleMono => "LegibleMono",
        Palette::Slime => "Slime",
        Palette::Mold => "Mold",
        Palette::Fungus => "Fungus",
        Palette::Swamp => "Swamp",
        Palette::Moss => "Moss",
        Palette::Cosmic => "Cosmic",
        Palette::Ethereal => "Ethereal",
        Palette::Custom(_) => "Custom",
    }
}

/// Returns the display name of a charset.
pub fn charset_name(charset: &Charset) -> &'static str {
    match charset {
        Charset::HalfBlock => "HalfBlock",
        Charset::HalfBlockDual => "HalfBlockDual",
        Charset::Ascii => "ASCII",
        Charset::Braille => "Braille",
        Charset::Quadrant => "Quadrant",
        Charset::Shade => "Shade",
        Charset::Points => "Points",
        Charset::Sculpted => "Sculpted",
        Charset::CustomAscii(_) => "Custom",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_input_poller_creation() {
        let poller = InputPoller::new();
        assert_eq!(poller.poll_timeout, Duration::from_millis(0));
    }

    #[test]
    fn test_input_poller_default() {
        let poller = InputPoller::default();
        assert_eq!(poller.poll_timeout, Duration::from_millis(0));
    }

    #[test]
    fn test_set_poll_timeout() {
        let mut poller = InputPoller::new();
        poller.set_poll_timeout(Duration::from_millis(100));
        assert_eq!(poller.poll_timeout, Duration::from_millis(100));
    }

    #[test]
    fn test_is_exit_key() {
        use crossterm::event::{KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};
        let q = KeyEvent {
            code: KeyCode::Char('q'),
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: KeyEventState::empty(),
        };
        assert!(InputPoller::is_exit_key(&q));

        let q_upper = KeyEvent {
            code: KeyCode::Char('Q'),
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: KeyEventState::empty(),
        };
        assert!(InputPoller::is_exit_key(&q_upper));

        let x = KeyEvent {
            code: KeyCode::Char('x'),
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: KeyEventState::empty(),
        };
        assert!(!InputPoller::is_exit_key(&x));
    }

    #[test]
    fn test_handle_key_event_basic() {
        use crossterm::event::{KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};

        // Test space key
        let space = KeyEvent {
            code: KeyCode::Char(' '),
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: KeyEventState::empty(),
        };
        assert!(matches!(
            handle_key_event(&space),
            ControlAction::TogglePause
        ));

        // Test 'c' key
        let c = KeyEvent {
            code: KeyCode::Char('c'),
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: KeyEventState::empty(),
        };
        assert!(matches!(handle_key_event(&c), ControlAction::CyclePalette));

        // Test 'q' key
        let q = KeyEvent {
            code: KeyCode::Char('q'),
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: KeyEventState::empty(),
        };
        assert!(matches!(handle_key_event(&q), ControlAction::Quit));
    }

    #[test]
    fn test_handle_key_event_control() {
        use crossterm::event::{KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};

        // Test Ctrl+S
        let ctrl_s = KeyEvent {
            code: KeyCode::Char('s'),
            modifiers: KeyModifiers::CONTROL,
            kind: KeyEventKind::Press,
            state: KeyEventState::empty(),
        };
        assert!(matches!(
            handle_key_event(&ctrl_s),
            ControlAction::ShowConfigSaveDialog
        ));

        // Test Ctrl+Z
        let ctrl_z = KeyEvent {
            code: KeyCode::Char('z'),
            modifiers: KeyModifiers::CONTROL,
            kind: KeyEventKind::Press,
            state: KeyEventState::empty(),
        };
        assert!(matches!(handle_key_event(&ctrl_z), ControlAction::Undo));
    }

    #[test]
    fn test_preset_name() {
        assert_eq!(preset_name(Preset::Network), "Network");
        assert_eq!(preset_name(Preset::Organic), "Organic");
        assert_eq!(preset_name(Preset::Zen), "Zen");
    }

    #[test]
    fn test_palette_name() {
        assert_eq!(palette_name(Palette::Organic), "Organic");
        assert_eq!(palette_name(Palette::Heat), "Heat");
        assert_eq!(palette_name(Palette::Moss), "Moss");
    }

    #[test]
    fn test_charset_name() {
        assert_eq!(charset_name(&Charset::HalfBlock), "HalfBlock");
        assert_eq!(charset_name(&Charset::Ascii), "ASCII");
        assert_eq!(charset_name(&Charset::Braille), "Braille");
    }

    #[test]
    fn test_brightness_keys_brighten_on_unshifted() {
        use crossterm::event::{KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};

        // Un-shifted 'n' must brighten. The engine divides by the stored
        // white-point, so brighter = a *negative* delta (lower white-point).
        let n = KeyEvent {
            code: KeyCode::Char('n'),
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: KeyEventState::empty(),
        };
        match handle_key_event(&n) {
            ControlAction::AdjustMaxBrightness(delta) => assert!(
                delta < 0.0,
                "un-shifted brightness key must lower the white-point (brighter), got {delta}"
            ),
            other => panic!("expected AdjustMaxBrightness, got {other:?}"),
        }

        // Shift+N must dim => positive delta (higher white-point).
        let shift_n = KeyEvent {
            code: KeyCode::Char('N'),
            modifiers: KeyModifiers::SHIFT,
            kind: KeyEventKind::Press,
            state: KeyEventState::empty(),
        };
        match handle_key_event(&shift_n) {
            ControlAction::AdjustMaxBrightness(delta) => assert!(
                delta > 0.0,
                "shifted brightness key must raise the white-point (dimmer), got {delta}"
            ),
            other => panic!("expected AdjustMaxBrightness, got {other:?}"),
        }
    }

    #[test]
    fn test_f11_maps_to_toggle_fullscreen() {
        use crossterm::event::{KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};
        let event = KeyEvent {
            code: KeyCode::F(11),
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: KeyEventState::empty(),
        };
        assert!(matches!(
            handle_key_event(&event),
            ControlAction::ToggleFullscreen
        ));
    }
}
