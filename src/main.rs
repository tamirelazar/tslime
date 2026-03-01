use std::io;

fn main() -> io::Result<()> {
    #[cfg(feature = "gui")]
    {
        use std::io::IsTerminal;
        let is_gui_child = std::env::var_os("TSLIME_GUI_CHILD").is_some();
        if !is_gui_child && !std::io::stdin().is_terminal() {
            return tslime::gui::run();
        }
    }

    tslime::app::run()
}
