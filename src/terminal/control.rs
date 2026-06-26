//! Runtime controls and state management (re-export module).
//!
//! This module re-exports types from `state` and `input` modules for backwards compatibility.
//! New code should use `terminal::state` and `terminal::input` directly.

pub use crate::terminal::state::*;

pub use crate::terminal::input::{charset_name, handle_key_event, palette_name, preset_name};
