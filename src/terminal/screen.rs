use crossterm::{
    cursor, execute,
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::io::{self, Stdout};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

pub struct TerminalScreen {
    stdout: Stdout,
    is_active: bool,
    resize_flag: Arc<AtomicBool>,
}

impl TerminalScreen {
    pub fn new() -> Self {
        let resize_flag = Arc::new(AtomicBool::new(false));
        Self {
            stdout: io::stdout(),
            is_active: false,
            resize_flag,
        }
    }

    pub fn setup(&mut self) -> io::Result<()> {
        if self.is_active {
            return Ok(());
        }

        execute!(self.stdout, EnterAlternateScreen, cursor::Hide,)?;
        terminal::enable_raw_mode()?;
        self.is_active = true;

        #[cfg(unix)]
        {
            let flag = Arc::clone(&self.resize_flag);
            signal_hook::flag::register(signal_hook::consts::SIGWINCH, flag)
                .expect("Failed to register SIGWINCH handler");
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

        terminal::disable_raw_mode()?;
        execute!(self.stdout, LeaveAlternateScreen, cursor::Show,)?;
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
    }

    #[test]
    fn test_terminal_screen_default() {
        let screen = TerminalScreen::default();
        assert!(!screen.is_active());
    }
}
