//! Embedded GUI terminal window using iced_term.
//!
//! Launched automatically when stdin is not a TTY (e.g., double-clicked from
//! Finder/Explorer). Spawns the tslime binary itself inside a PTY so the child
//! process sees a real TTY and runs in normal TUI mode.

mod fonts;

use crate::gui::fonts::FIRA_CODE_NERD_FONT;
use iced::{window, Length, Size, Subscription, Task, Theme};
use iced_term::TerminalView;
use std::io;

/// Entry point: launch the iced window hosting a PTY-backed terminal.
pub fn run() -> io::Result<()> {
    // Mark child processes so they skip GUI detection and run in TUI mode.
    // Called before iced starts any threads, so no concurrent env access.
    std::env::set_var("TSLIME_GUI_CHILD", "1");

    iced::application(App::new, App::update, App::view)
        .title(App::title)
        .window_size(Size::new(1200.0, 800.0))
        .decorations(false)
        .resizable(true)
        .subscription(App::subscription)
        .font(include_bytes!(
            "../../assets/fonts/FiraCodeNerdFont-Regular.ttf"
        ))
        .run()
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))
}

// ---------------------------------------------------------------------------
// Application state
// ---------------------------------------------------------------------------

struct App {
    title: String,
    term: iced_term::Terminal,
}

#[derive(Debug, Clone)]
enum Event {
    Terminal(iced_term::Event),
}

impl App {
    fn new() -> (Self, Task<Event>) {
        let exe = std::env::current_exe().expect("cannot resolve current executable path");
        // Forward all original CLI args so the PTY child inherits the same config.
        let args: Vec<String> = std::env::args().skip(1).collect();

        let settings = iced_term::settings::Settings {
            font: iced_term::settings::FontSettings {
                size: 14.0,
                scale_factor: 1.20, // ~35% extra line height, matching Ghostty config
                font_type: FIRA_CODE_NERD_FONT,
            },
            theme: iced_term::settings::ThemeSettings::new(Box::new(gruvbox_material_dark())),
            backend: iced_term::settings::BackendSettings {
                program: exe.to_string_lossy().into_owned(),
                args,
                ..Default::default()
            },
        };

        let term = iced_term::Terminal::new(0, settings).expect("failed to create PTY terminal");

        (
            Self {
                title: String::from("tslime"),
                term,
            },
            Task::none(),
        )
    }

    fn title(&self) -> String {
        self.title.clone()
    }

    fn subscription(&self) -> Subscription<Event> {
        self.term.subscription().map(Event::Terminal)
    }

    fn update(&mut self, event: Event) -> Task<Event> {
        match event {
            Event::Terminal(iced_term::Event::BackendCall(_, cmd)) => {
                match self.term.handle(iced_term::Command::ProxyToBackend(cmd)) {
                    iced_term::actions::Action::Shutdown => {
                        return window::latest().and_then(window::close);
                    }
                    iced_term::actions::Action::ChangeTitle(t) => {
                        self.title = t;
                    }
                    iced_term::actions::Action::Ignore => {}
                }
            }
        }
        Task::none()
    }

    fn view(&self) -> iced::Element<'_, Event, Theme, iced::Renderer> {
        iced::widget::container(TerminalView::show(&self.term).map(Event::Terminal))
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}

// ---------------------------------------------------------------------------
// Gruvbox Material Dark colour palette
//
// Background: #1C2021  (user's custom bg from Ghostty config)
// Foreground: #d4be98  (gruvbox-material fg)
// ---------------------------------------------------------------------------

fn gruvbox_material_dark() -> iced_term::ColorPalette {
    iced_term::ColorPalette {
        background: String::from("#1C2021"),
        foreground: String::from("#d4be98"),
        black: String::from("#3c3836"),
        red: String::from("#ea6962"),
        green: String::from("#a9b665"),
        yellow: String::from("#d8a657"),
        blue: String::from("#7daea3"),
        magenta: String::from("#d3869b"),
        cyan: String::from("#89b482"),
        white: String::from("#d4be98"),
        bright_black: String::from("#928374"),
        bright_red: String::from("#ea6962"),
        bright_green: String::from("#a9b665"),
        bright_yellow: String::from("#d8a657"),
        bright_blue: String::from("#7daea3"),
        bright_magenta: String::from("#d3869b"),
        bright_cyan: String::from("#89b482"),
        bright_white: String::from("#ebdbb2"),
        // dim variants — slightly muted versions of the normal colours
        dim_foreground: String::from("#928374"),
        dim_black: String::from("#282828"),
        dim_red: String::from("#9d0006"),
        dim_green: String::from("#79740e"),
        dim_yellow: String::from("#b57614"),
        dim_blue: String::from("#076678"),
        dim_magenta: String::from("#8f3f71"),
        dim_cyan: String::from("#427b58"),
        dim_white: String::from("#928374"),
        ..Default::default()
    }
}
