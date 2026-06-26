//! Overlay-aware input handling.
//!
//! Provides a trait for overlays to declare and handle their own keyboard input,
//! enabling clean separation of concerns between the main loop and overlay-specific
//! key bindings.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

/// A key binding hint for display in help text.
///
/// Used by overlays to declare which keys they respond to,
/// enabling dynamic help generation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyHint {
    /// The key combination (e.g., "Tab", "Ctrl+S", "↑")
    pub key: String,
    /// Description of what the key does
    pub description: String,
}

impl KeyHint {
    /// Creates a new key hint.
    ///
    /// # Example
    /// ```
    /// use tslime::overlay::input::KeyHint;
    ///
    /// let hint = KeyHint::new("Tab", "Cycle category");
    /// ```
    pub fn new(key: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            description: description.into(),
        }
    }

    /// Creates a key hint for arrow keys.
    pub fn arrow(direction: &str, description: impl Into<String>) -> Self {
        let arrow = match direction {
            "up" | "↑" => "↑",
            "down" | "↓" => "↓",
            "left" | "←" => "←",
            "right" | "→" => "→",
            _ => direction,
        };
        Self::new(arrow, description)
    }

    /// Formats the hint as "key → description".
    pub fn format(&self) -> String {
        format!("{} → {}", self.key, self.description)
    }
}

/// Trait for overlays that handle keyboard input.
///
/// Implement this trait to give an overlay control over specific key bindings.
/// The main input loop will delegate to the overlay's handler when that overlay
/// is active.
pub trait OverlayInputHandler {
    /// Handles a keyboard event.
    ///
    /// Returns `true` if the event was consumed by this overlay,
    /// `false` if it should be passed to other handlers.
    fn handle_key(&mut self, key: &KeyEvent) -> bool;

    /// Returns key binding hints for this overlay.
    ///
    /// These hints can be displayed in help text or keyboard reference.
    fn key_hints(&self) -> Vec<KeyHint>;

    /// Returns true if this overlay wants to capture all input.
    ///
    /// When true, unhandled keys are not passed to global handlers.
    /// When false (default), unhandled keys fall through.
    fn captures_all_input(&self) -> bool {
        false
    }

    /// Called when the overlay is opened.
    fn on_open(&mut self) {}

    /// Called when the overlay is closed.
    fn on_close(&mut self) {}
}

/// Helper functions for common key matching patterns.
pub mod key_matchers {
    use super::*;

    /// Returns true if the key is Tab without modifiers.
    pub fn is_tab(key: &KeyEvent) -> bool {
        key.code == KeyCode::Tab && key.modifiers == KeyModifiers::NONE
    }

    /// Returns true if the key is Shift+Tab.
    pub fn is_backtab(key: &KeyEvent) -> bool {
        key.code == KeyCode::BackTab
            || (key.code == KeyCode::Tab && key.modifiers.contains(KeyModifiers::SHIFT))
    }

    /// Returns true if the key is Escape.
    pub fn is_escape(key: &KeyEvent) -> bool {
        key.code == KeyCode::Esc
    }

    /// Returns true if the key is Enter.
    pub fn is_enter(key: &KeyEvent) -> bool {
        matches!(key.code, KeyCode::Enter | KeyCode::Char('\n'))
    }

    /// Returns the arrow direction, or `None` if the key is not an arrow key.
    pub fn is_arrow(key: &KeyEvent) -> Option<Direction> {
        match key.code {
            KeyCode::Up => Some(Direction::Up),
            KeyCode::Down => Some(Direction::Down),
            KeyCode::Left => Some(Direction::Left),
            KeyCode::Right => Some(Direction::Right),
            _ => None,
        }
    }

    /// Returns true if the key is the given character with exactly the given modifiers.
    pub fn is_char(key: &KeyEvent, c: char, modifiers: KeyModifiers) -> bool {
        key.code == KeyCode::Char(c) && key.modifiers == modifiers
    }

    /// Direction enum for arrow keys.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum Direction {
        /// Up arrow key.
        Up,
        /// Down arrow key.
        Down,
        /// Left arrow key.
        Left,
        /// Right arrow key.
        Right,
    }
}

/// A composite input handler that tries multiple handlers in order.
pub struct CompositeHandler {
    handlers: Vec<Box<dyn OverlayInputHandler>>,
}

impl CompositeHandler {
    /// Creates a new empty composite handler.
    pub fn new() -> Self {
        Self {
            handlers: Vec::new(),
        }
    }

    /// Adds a handler to the chain.
    pub fn add<H: OverlayInputHandler + 'static>(&mut self, handler: H) {
        self.handlers.push(Box::new(handler));
    }
}

impl Default for CompositeHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl OverlayInputHandler for CompositeHandler {
    fn handle_key(&mut self, key: &KeyEvent) -> bool {
        for handler in &mut self.handlers {
            if handler.handle_key(key) {
                return true;
            }
        }
        false
    }

    fn key_hints(&self) -> Vec<KeyHint> {
        self.handlers.iter().flat_map(|h| h.key_hints()).collect()
    }
}

/// Input handler for simple overlays that only need Escape to close.
pub struct SimpleCloseHandler;

impl OverlayInputHandler for SimpleCloseHandler {
    fn handle_key(&mut self, key: &KeyEvent) -> bool {
        key_matchers::is_escape(key)
    }

    fn key_hints(&self) -> Vec<KeyHint> {
        vec![KeyHint::new("Esc", "Close")]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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
    fn test_key_hint_creation() {
        let hint = KeyHint::new("Tab", "Cycle category");
        assert_eq!(hint.key, "Tab");
        assert_eq!(hint.description, "Cycle category");
        assert_eq!(hint.format(), "Tab → Cycle category");
    }

    #[test]
    fn test_key_hint_arrow() {
        let hint = KeyHint::arrow("up", "Increase value");
        assert_eq!(hint.key, "↑");

        let hint2 = KeyHint::arrow("→", "Next item");
        assert_eq!(hint2.key, "→");
    }

    #[test]
    fn test_key_matchers() {
        let tab = make_key(KeyCode::Tab, KeyModifiers::NONE);
        assert!(key_matchers::is_tab(&tab));

        let shift_tab = make_key(KeyCode::Tab, KeyModifiers::SHIFT);
        assert!(key_matchers::is_backtab(&shift_tab));

        let esc = make_key(KeyCode::Esc, KeyModifiers::NONE);
        assert!(key_matchers::is_escape(&esc));

        let enter = make_key(KeyCode::Enter, KeyModifiers::NONE);
        assert!(key_matchers::is_enter(&enter));

        let up = make_key(KeyCode::Up, KeyModifiers::NONE);
        assert_eq!(
            key_matchers::is_arrow(&up),
            Some(key_matchers::Direction::Up)
        );
    }

    struct TestHandler {
        last_key: Option<String>,
    }

    impl OverlayInputHandler for TestHandler {
        fn handle_key(&mut self, key: &KeyEvent) -> bool {
            match key.code {
                KeyCode::Char(c) => {
                    self.last_key = Some(c.to_string());
                    true
                }
                _ => false,
            }
        }

        fn key_hints(&self) -> Vec<KeyHint> {
            vec![KeyHint::new("a", "Test action")]
        }
    }

    #[test]
    fn test_handler_trait() {
        let mut handler = TestHandler { last_key: None };

        let key = make_key(KeyCode::Char('a'), KeyModifiers::NONE);
        assert!(handler.handle_key(&key));
        assert_eq!(handler.last_key, Some("a".to_string()));

        let key2 = make_key(KeyCode::Tab, KeyModifiers::NONE);
        assert!(!handler.handle_key(&key2));

        let hints = handler.key_hints();
        assert_eq!(hints.len(), 1);
        assert_eq!(hints[0].key, "a");
    }

    #[test]
    fn test_composite_handler() {
        let mut composite = CompositeHandler::new();

        struct HandlerA;
        impl OverlayInputHandler for HandlerA {
            fn handle_key(&mut self, key: &KeyEvent) -> bool {
                key.code == KeyCode::Char('a')
            }
            fn key_hints(&self) -> Vec<KeyHint> {
                vec![KeyHint::new("a", "Action A")]
            }
        }

        struct HandlerB;
        impl OverlayInputHandler for HandlerB {
            fn handle_key(&mut self, key: &KeyEvent) -> bool {
                key.code == KeyCode::Char('b')
            }
            fn key_hints(&self) -> Vec<KeyHint> {
                vec![KeyHint::new("b", "Action B")]
            }
        }

        composite.add(HandlerA);
        composite.add(HandlerB);

        assert!(composite.handle_key(&make_key(KeyCode::Char('a'), KeyModifiers::NONE)));
        assert!(composite.handle_key(&make_key(KeyCode::Char('b'), KeyModifiers::NONE)));
        assert!(!composite.handle_key(&make_key(KeyCode::Char('c'), KeyModifiers::NONE)));

        let hints = composite.key_hints();
        assert_eq!(hints.len(), 2);
    }

    #[test]
    fn test_simple_close_handler() {
        let mut handler = SimpleCloseHandler;

        assert!(handler.handle_key(&make_key(KeyCode::Esc, KeyModifiers::NONE)));
        assert!(!handler.handle_key(&make_key(KeyCode::Char('a'), KeyModifiers::NONE)));

        let hints = handler.key_hints();
        assert_eq!(hints.len(), 1);
        assert_eq!(hints[0].key, "Esc");
    }
}
