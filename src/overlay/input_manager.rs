//! Centralized overlay input handling.
//!
//! SINGLE SOURCE OF TRUTH for overlay input behavior:
//! 1. All overlays toggle on their keybind (open → close, closed → open)
//! 2. All overlays close on Escape (if closes_on_escape())
//! 3. When an overlay is open, keys that would toggle a DIFFERENT overlay are blocked
//! 4. All other keys pass through normally (e.g., 'c' for cycle palette works)
//! 5. To switch overlays: close current, then open new

use crate::overlay::{OverlayState, OverlayType};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

/// Result of handling input for an overlay
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OverlayInputResult {
    /// Input was consumed (blocked), don't process further
    Consumed,
    /// Close current overlay, don't process the key further
    CloseOverlay,
    /// Input not handled, pass to next handler (no overlay open or key not related to overlays)
    NotHandled,
}

/// Centralized overlay input manager - SINGLE SOURCE OF TRUTH
pub struct OverlayInputManager;

impl OverlayInputManager {
    /// Handle input when an overlay might be open
    ///
    /// Returns:
    /// - `CloseOverlay` if the overlay's toggle key or Escape was pressed
    /// - `Consumed` if key would toggle a different overlay (prevent switching)
    /// - `NotHandled` if no overlay is open, or key is unrelated to overlays
    pub fn handle_input(state: &OverlayState, key: &KeyEvent) -> OverlayInputResult {
        let Some(active) = state.active() else {
            return OverlayInputResult::NotHandled;
        };

        // Rule 1: Check if this is the overlay's own toggle key
        if let KeyCode::Char(c) = key.code {
            if active.is_toggle_key(c, key.modifiers) {
                return OverlayInputResult::CloseOverlay;
            }
        }

        // Rule 2: Escape closes overlay (if supported and not handled internally)
        if key.code == KeyCode::Esc && active.closes_on_escape() {
            if active.handles_escape_internally() {
                // Let the overlay handle Escape internally (e.g., palette editor dialog modes)
                return OverlayInputResult::NotHandled;
            }
            return OverlayInputResult::CloseOverlay;
        }

        // Rule 3: Block keys that would toggle a different overlay
        // This prevents accidental overlay switching
        // NOTE: Don't block Control-modified keys (Ctrl+S, Ctrl+L, etc.) as these
        // are typically handled by the active overlay, not toggle keys
        if !key.modifiers.contains(KeyModifiers::CONTROL) {
            if let KeyCode::Char(c) = key.code {
                if let Some(other) = OverlayType::from_toggle_key(c, key.modifiers) {
                    if other != active {
                        return OverlayInputResult::Consumed;
                    }
                }
            }
        }

        // Rule 4: Let all other keys pass through
        // Keys like 'c' (cycle palette), Space (pause), etc. should work
        OverlayInputResult::NotHandled
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::overlay::OverlayType;
    use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};

    fn make_key(code: KeyCode, modifiers: KeyModifiers) -> KeyEvent {
        KeyEvent {
            code,
            modifiers,
            kind: KeyEventKind::Press,
            state: KeyEventState::empty(),
        }
    }

    #[test]
    fn test_no_overlay_not_handled() {
        let state = OverlayState::default();
        let key = make_key(KeyCode::Char('p'), KeyModifiers::NONE);

        assert_eq!(
            OverlayInputManager::handle_input(&state, &key),
            OverlayInputResult::NotHandled
        );
    }

    #[test]
    fn test_escape_closes_overlay() {
        let mut state = OverlayState::default();
        state.open(OverlayType::Controls);

        let esc = make_key(KeyCode::Esc, KeyModifiers::NONE);
        assert_eq!(
            OverlayInputManager::handle_input(&state, &esc),
            OverlayInputResult::CloseOverlay
        );
    }

    #[test]
    fn test_toggle_key_closes_overlay() {
        let mut state = OverlayState::default();
        state.open(OverlayType::Controls);

        let h = make_key(KeyCode::Char('h'), KeyModifiers::NONE);
        assert_eq!(
            OverlayInputManager::handle_input(&state, &h),
            OverlayInputResult::CloseOverlay
        );
    }

    #[test]
    fn test_other_overlay_keys_blocked() {
        let mut state = OverlayState::default();
        state.open(OverlayType::Controls);

        // Dashboard key should be blocked, not switch
        let backslash = make_key(KeyCode::Char('\\'), KeyModifiers::NONE);
        assert_eq!(
            OverlayInputManager::handle_input(&state, &backslash),
            OverlayInputResult::Consumed
        );

        // Palette editor key should be blocked
        let p = make_key(KeyCode::Char('p'), KeyModifiers::NONE);
        assert_eq!(
            OverlayInputManager::handle_input(&state, &p),
            OverlayInputResult::Consumed
        );
    }

    #[test]
    fn test_non_overlay_keys_pass_through() {
        let mut state = OverlayState::default();
        state.open(OverlayType::Controls);

        // Normal keys like 'c' (cycle palette) should pass through
        let c = make_key(KeyCode::Char('c'), KeyModifiers::NONE);
        assert_eq!(
            OverlayInputManager::handle_input(&state, &c),
            OverlayInputResult::NotHandled
        );

        // Space (pause) should pass through
        let space = make_key(KeyCode::Char(' '), KeyModifiers::NONE);
        assert_eq!(
            OverlayInputManager::handle_input(&state, &space),
            OverlayInputResult::NotHandled
        );
    }

    #[test]
    fn test_palette_editor_toggle_keys() {
        let mut state = OverlayState::default();
        state.open(OverlayType::PaletteEditor);

        // 'p' should close palette editor
        let p = make_key(KeyCode::Char('p'), KeyModifiers::NONE);
        assert_eq!(
            OverlayInputManager::handle_input(&state, &p),
            OverlayInputResult::CloseOverlay
        );

        // '/' should also close palette editor
        let slash = make_key(KeyCode::Char('/'), KeyModifiers::NONE);
        assert_eq!(
            OverlayInputManager::handle_input(&state, &slash),
            OverlayInputResult::CloseOverlay
        );
    }

    #[test]
    fn test_dashboard_toggle_keys() {
        let mut state = OverlayState::default();
        state.open(OverlayType::Dashboard);

        // '\\' should close dashboard
        let backslash = make_key(KeyCode::Char('\\'), KeyModifiers::NONE);
        assert_eq!(
            OverlayInputManager::handle_input(&state, &backslash),
            OverlayInputResult::CloseOverlay
        );

        // '|' should also close dashboard
        let pipe = make_key(KeyCode::Char('|'), KeyModifiers::NONE);
        assert_eq!(
            OverlayInputManager::handle_input(&state, &pipe),
            OverlayInputResult::CloseOverlay
        );
    }

    #[test]
    fn test_keyboard_hints_toggle_key() {
        let mut state = OverlayState::default();
        state.open(OverlayType::KeyboardHints);

        // '?' should close keyboard hints
        let question = make_key(KeyCode::Char('?'), KeyModifiers::NONE);
        assert_eq!(
            OverlayInputManager::handle_input(&state, &question),
            OverlayInputResult::CloseOverlay
        );
    }

    #[test]
    fn test_ctrl_keys_for_config_overlays() {
        let mut state = OverlayState::default();
        state.open(OverlayType::ConfigBrowser);

        // Ctrl+L should close config browser
        let ctrl_l = make_key(KeyCode::Char('l'), KeyModifiers::CONTROL);
        assert_eq!(
            OverlayInputManager::handle_input(&state, &ctrl_l),
            OverlayInputResult::CloseOverlay
        );

        // Regular 'l' should pass through (not an overlay toggle key)
        let l = make_key(KeyCode::Char('l'), KeyModifiers::NONE);
        assert_eq!(
            OverlayInputManager::handle_input(&state, &l),
            OverlayInputResult::NotHandled
        );
    }

    #[test]
    fn test_palette_editor_escape_delegated() {
        let mut state = OverlayState::default();
        state.open(OverlayType::PaletteEditor);

        // Escape should be delegated to palette editor's internal handler
        // (for handling dialog modes specially)
        let esc = make_key(KeyCode::Esc, KeyModifiers::NONE);
        assert_eq!(
            OverlayInputManager::handle_input(&state, &esc),
            OverlayInputResult::NotHandled
        );
    }

    #[test]
    fn test_regular_overlay_escape_closes() {
        let mut state = OverlayState::default();
        state.open(OverlayType::Controls);

        // Escape should close regular overlays immediately
        let esc = make_key(KeyCode::Esc, KeyModifiers::NONE);
        assert_eq!(
            OverlayInputManager::handle_input(&state, &esc),
            OverlayInputResult::CloseOverlay
        );
    }
}
