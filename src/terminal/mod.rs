/// Runtime controls and state management.
pub mod control;
/// Terminal capability detection.
pub mod detection;
/// Frame buffer for terminal rendering.
pub mod frame_buffer;
/// Input handling (keyboard/mouse).
pub mod input;
/// Terminal renderer with overlay support.
pub mod renderer;
/// Alternate screen buffer management.
pub mod screen;
/// Signal handling (Ctrl+C, resize).
pub mod signal;
/// Runtime state and control types.
pub mod state;
/// Frame timing and synchronization.
pub mod timing;

use std::io::{self, Write};

/// Enables mouse tracking in the terminal.
///
/// Sends ANSI escape codes to enable:
/// - Any event tracking (1003)
/// - SGR extended coordinates (1006)
/// - URXVT extended coordinates (1015)
pub fn enable_mouse_tracking() -> io::Result<()> {
    print!("\x1b[?1003h\x1b[?1006h\x1b[?1015h");
    io::stdout().flush()?;
    Ok(())
}

/// Disables mouse tracking in the terminal.
///
/// Sends ANSI escape codes to disable the tracking modes enabled by `enable_mouse_tracking`.
pub fn disable_mouse_tracking() -> io::Result<()> {
    print!("\x1b[?1003l\x1b[?1006l\x1b[?1015l");
    io::stdout().flush()?;
    Ok(())
}
