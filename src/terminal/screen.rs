#[cfg(unix)]
use crate::terminal::signal::request_shutdown;
use crossterm::{
    cursor, execute,
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
};
#[cfg(unix)]
use signal_hook::low_level::{register, unregister};
#[cfg(unix)]
use signal_hook::SigId;
use std::io::{self, Stdout};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

pub struct TerminalScreen {
    stdout: Stdout,
    is_active: bool,
    resize_flag: Arc<AtomicBool>,
    #[cfg(unix)]
    sigwinch_id: Option<SigId>,
    #[cfg(unix)]
    sigint_id: Option<SigId>,
    #[cfg(unix)]
    sigterm_id: Option<SigId>,
}

impl TerminalScreen {
    pub fn new() -> Self {
        let resize_flag = Arc::new(AtomicBool::new(false));
        Self {
            stdout: io::stdout(),
            is_active: false,
            resize_flag,
            #[cfg(unix)]
            sigwinch_id: None,
            #[cfg(unix)]
            sigint_id: None,
            #[cfg(unix)]
            sigterm_id: None,
        }
    }

    pub fn setup(&mut self) -> io::Result<()> {
        if self.is_active {
            return Ok(());
        }

        execute!(self.stdout, EnterAlternateScreen, cursor::Hide)?;
        terminal::enable_raw_mode()?;
        self.is_active = true;

        #[cfg(unix)]
        {
            use signal_hook::consts::SIGINT;

            let id = unsafe {
                register(SIGINT, || {
                    request_shutdown();
                })
            }
            .expect("Failed to register SIGINT handler");
            self.sigint_id = Some(id);
        }

        #[cfg(unix)]
        {
            use signal_hook::consts::SIGTERM;

            let id = unsafe {
                register(SIGTERM, || {
                    request_shutdown();
                })
            }
            .expect("Failed to register SIGTERM handler");
            self.sigterm_id = Some(id);
        }

        #[cfg(unix)]
        {
            use signal_hook::consts::SIGWINCH;

            let flag = Arc::clone(&self.resize_flag);
            let id = unsafe {
                register(SIGWINCH, move || {
                    flag.store(true, Ordering::SeqCst);
                })
            }
            .expect("Failed to register SIGWINCH handler");
            self.sigwinch_id = Some(id);
        }

        Ok(())
    }

    pub fn get_size(&self) -> io::Result<(u16, u16)> {
        terminal::size()
    }

    pub fn check_resize(&self) -> bool {
        self.resize_flag.swap(false, Ordering::SeqCst)
    }

    pub fn teardown(&mut self) -> io::Result<()> {
        if !self.is_active {
            return Ok(());
        }

        #[cfg(unix)]
        {
            if let Some(id) = self.sigwinch_id.take() {
                unregister(id);
            }
            if let Some(id) = self.sigint_id.take() {
                unregister(id);
            }
            if let Some(id) = self.sigterm_id.take() {
                unregister(id);
            }
        }

        terminal::disable_raw_mode()?;
        execute!(self.stdout, LeaveAlternateScreen, cursor::Show)?;
        self.is_active = false;
        Ok(())
    }

    #[allow(dead_code)]
    pub fn is_active(&self) -> bool {
        self.is_active
    }

    #[allow(dead_code)]
    pub fn clear(&mut self) -> io::Result<()> {
        if !self.is_active {
            return Ok(());
        }
        execute!(self.stdout, terminal::Clear(terminal::ClearType::All))
    }
}

impl Default for TerminalScreen {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for TerminalScreen {
    fn drop(&mut self) {
        let _ = self.teardown();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_terminal_screen_creation() {
        let screen = TerminalScreen::new();
        assert!(!screen.is_active());
        #[cfg(unix)]
        {
            assert!(screen.sigwinch_id.is_none());
            assert!(screen.sigint_id.is_none());
            assert!(screen.sigterm_id.is_none());
        }
    }

    #[test]
    fn test_terminal_screen_default() {
        let screen = TerminalScreen::default();
        assert!(!screen.is_active());
    }

    #[test]
    fn test_resize_flag_initial() {
        let screen = TerminalScreen::new();
        assert!(!screen.check_resize());
    }

    #[test]
    fn test_setup_idempotent() {
        let mut screen = TerminalScreen::new();
        let result1 = screen.setup();
        if result1.is_ok() {
            assert!(screen.is_active());
            let result2 = screen.setup();
            assert!(result2.is_ok());
            assert!(screen.is_active());
        } else {
            assert!(!screen.is_active());
        }
    }

    #[test]
    fn test_teardown_idempotent() {
        let mut screen = TerminalScreen::new();
        let result1 = screen.teardown();
        assert!(result1.is_ok());
        assert!(!screen.is_active());
        let result2 = screen.teardown();
        assert!(result2.is_ok());
        assert!(!screen.is_active());
    }
}
