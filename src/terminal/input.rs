use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, MouseEventKind};
use std::io;
use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MousePosition {
    pub x: usize,
    pub y: usize,
}

pub enum MouseEventType {
    Down,
    Drag,
    Moved,
}

pub struct InputPoller {
    poll_timeout: Duration,
}

impl InputPoller {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self {
            poll_timeout: Duration::from_millis(0),
        }
    }

    #[allow(dead_code)]
    pub fn set_poll_timeout(&mut self, timeout: Duration) {
        self.poll_timeout = timeout;
    }

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

    #[allow(dead_code)]
    pub fn poll_mouse_event(&self) -> io::Result<Option<(MousePosition, MouseEventType)>> {
        if event::poll(Duration::from_millis(1))? {
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
                        x: mouse_event.column as usize - 1,
                        y: mouse_event.row as usize - 1,
                    },
                    event_type,
                )));
            }
        }
        Ok(None)
    }

    pub fn drain_all_events(&self) -> io::Result<Vec<Event>> {
        let mut events = Vec::new();
        while event::poll(Duration::from_millis(0))? {
            events.push(event::read()?);
        }
        Ok(events)
    }

    pub fn is_exit_key(key_event: &KeyEvent) -> bool {
        matches!(
            key_event.code,
            KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Esc
        )
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
}
