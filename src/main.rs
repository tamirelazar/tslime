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
            return tslime::gui::run();
        }
    }

    tslime::app::run()
}
