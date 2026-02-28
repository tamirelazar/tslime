use std::io;

fn main() -> io::Result<()> {
    #[cfg(feature = "gui")]
    {
        use std::io::IsTerminal;
        if !std::io::stdin().is_terminal() {
            return tslime::gui::run();
        }
    }

    tslime::app::run()
}
