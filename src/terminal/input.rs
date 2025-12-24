use crossterm::event::{self, Event, KeyEvent, KeyEventKind};
use std::io;
use std::time::Duration;

pub struct InputPoller {
    poll_timeout: Duration,
}

impl InputPoller {
    pub fn new() -> Self {
        Self {
            poll_timeout: Duration::from_millis(0),
        }
    }

    #[allow(dead_code)]
    pub fn set_poll_timeout(&mut self, timeout: Duration) {
        self.poll_timeout = timeout;
    }

    pub fn poll_keypress(&self) -> io::Result<bool> {
        if event::poll(self.poll_timeout)? {
            if let Event::Key(KeyEvent {
                kind: KeyEventKind::Press,
                ..
            }) = event::read()?
            {
                return Ok(true);
            }
        }
        Ok(false)
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
