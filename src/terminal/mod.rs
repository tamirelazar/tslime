//! Terminal abstraction layer for cross-platform terminal handling.
//!
//! This module provides a unified interface for terminal operations across different
//! platforms (Windows, macOS, Linux). It handles raw mode, mouse tracking, screen
//! buffers, input polling, and rendering.
//!
//! # Architecture
//!
//! The terminal module is organized into several sub-modules:
//!
//! - **control**: Runtime controls and state management
//! - **detection**: Terminal capability detection (colors, Unicode support, etc.)
//! - **frame_buffer**: Double-buffered frame storage for efficient rendering
//! - **input**: Non-blocking input polling for keyboard and mouse events
//! - **renderer**: High-level terminal renderer with overlay support
//! - **screen**: Alternate screen buffer management (enter/exit raw mode)
//! - **signal**: Signal handling for graceful shutdown (Ctrl+C, SIGTERM)
//! - **state**: Runtime state and control types
//! - **timing**: Frame timing and FPS synchronization
//!
//! # Example
//!
//! ```rust,no_run
//! use tslime::terminal::screen::TerminalScreen;
//! use tslime::terminal::{enable_mouse_tracking, disable_mouse_tracking};
//!
//! // Create and setup terminal screen
//! let mut screen = TerminalScreen::new();
//! screen.setup().unwrap();
//!
//! // Enable mouse tracking
//! enable_mouse_tracking().unwrap();
//!
//! // ... run your application ...
//!
//! // Cleanup
//! disable_mouse_tracking().unwrap();
//! screen.teardown().unwrap();
//! ```

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
