#![cfg_attr(all(windows, feature = "gui"), windows_subsystem = "windows")]

use std::io;

fn main() -> io::Result<()> {
    #[cfg(feature = "gui")]
    {
        use std::io::IsTerminal;
        let is_gui_child = std::env::var_os("TSLIME_GUI_CHILD").is_some();
        let no_tty = !std::io::stdin().is_terminal();

        // On Linux, also require a display server to be available; otherwise
        // fall through to TUI (headless server, SSH without X forwarding, etc.).
        #[cfg(target_os = "linux")]
        let has_display =
            std::env::var_os("DISPLAY").is_some() || std::env::var_os("WAYLAND_DISPLAY").is_some();
        #[cfg(not(target_os = "linux"))]
        let has_display = true;

        if !is_gui_child && no_tty && has_display {
            // GUI backends (winit/iced) must run on the main thread, notably
            // on macOS — do not move this onto a worker thread.
            return tslime::gui::run();
        }
    }

    // Windows gives the main thread only a 1 MiB stack (vs 8 MiB on
    // Linux/macOS). clap building its command for the large Args struct, plus
    // the simulation's deep call chains, overflows it at startup. Run the TUI
    // app on a worker thread with a generous stack for cross-platform parity.
    std::thread::Builder::new()
        .stack_size(16 * 1024 * 1024)
        .spawn(tslime::app::run)
        .expect("failed to spawn tslime worker thread")
        .join()
        .expect("tslime worker thread panicked")
}
