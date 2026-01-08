pub mod control;
pub mod detection;
pub mod input;
pub mod output;
pub mod screen;
pub mod signal;
pub mod timing;

use std::io::{self, Write};

pub fn enable_mouse_tracking() -> io::Result<()> {
    print!("\x1b[?1003h\x1b[?1006h\x1b[?1015h");
    io::stdout().flush()?;
    Ok(())
}

pub fn disable_mouse_tracking() -> io::Result<()> {
    print!("\x1b[?1003l\x1b[?1006l\x1b[?1015l");
    io::stdout().flush()?;
    Ok(())
}
