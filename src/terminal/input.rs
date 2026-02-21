//! Input handling for the terminal application.
//!
//! This module provides a non-blocking input poller that handles keyboard
//! and mouse events using `crossterm`.

pub use crate::terminal::control::MousePosition;
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
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self {
            poll_timeout: Duration::from_millis(0),
        }
    }

    /// Set the timeout duration for polling operations.
    #[allow(dead_code)]
    pub fn set_poll_timeout(&mut self, timeout: Duration) {
        self.poll_timeout = timeout;
    }

    /// Check for a keyboard event.
    ///
    /// Returns `Ok(Some(KeyEvent))` if a key was pressed within the timeout.
    /// Returns `Ok(None)` if no key was pressed.
    /// Returns `Err` if polling failed.
    #[allow(dead_code)]
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
    #[allow(dead_code)]
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
        use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};
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
}
