use std::env;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorCapability {
    TrueColor,
    Bits256,
}

#[derive(Debug, Clone)]
pub struct TerminalCapabilities {
    pub color_capability: ColorCapability,
    pub estimated_refresh_rate: f32,
    pub supports_mouse_tracking: bool,
    pub terminal_name: Option<String>,
}

impl TerminalCapabilities {
    pub fn detect() -> Self {
        Self {
            color_capability: detect_truecolor(),
            estimated_refresh_rate: estimate_refresh_rate(),
            supports_mouse_tracking: detect_mouse_support(),
            terminal_name: detect_terminal_name(),
        }
    }

    pub fn auto_select_color_mode(
        &self,
        requested: Option<super::super::cli::ColorMode>,
    ) -> super::super::cli::ColorMode {
        match requested {
            Some(mode) => mode,
            None => match self.color_capability {
                ColorCapability::TrueColor => super::super::cli::ColorMode::TrueColor,
                ColorCapability::Bits256 => super::super::cli::ColorMode::Bits256,
            },
        }
    }
}

fn detect_truecolor() -> ColorCapability {
    if env::var("COLORTERM")
        .ok()
        .map(|v| v.to_lowercase())
        .filter(|v| v.contains("truecolor") || v.contains("24bit"))
        .is_some()
    {
        return ColorCapability::TrueColor;
    }

    if let Ok(term) = env::var("TERM_PROGRAM") {
        let term_lower = term.to_lowercase();
        if term_lower.contains("iterm")
            || term_lower.contains("terminal.app")
            || term_lower.contains("warp")
        {
            return ColorCapability::TrueColor;
        }
    }

    if let Ok(term) = env::var("TERM") {
        let term_lower = term.to_lowercase();
        if term_lower.contains("xterm-256color") || term_lower.contains("screen-256color") {
            return ColorCapability::Bits256;
        }
    }

    ColorCapability::Bits256
}

fn estimate_refresh_rate() -> f32 {
    if let Ok(fps_str) = env::var("TSLIME_REFRESH_RATE") {
        if let Ok(fps) = fps_str.parse::<f32>() {
            if (10.0..=144.0).contains(&fps) {
                return fps;
            }
        }
    }

    60.0
}

fn detect_mouse_support() -> bool {
    let term = env::var("TERM").unwrap_or_default().to_lowercase();
    let term_program = env::var("TERM_PROGRAM").unwrap_or_default().to_lowercase();

    !term.contains("dumb")
        && (term_program.contains("iterm")
            || term_program.contains("terminal.app")
            || term_program.contains("warp")
            || term.contains("xterm")
            || term.contains("screen")
            || term.contains("tmux"))
}

fn detect_terminal_name() -> Option<String> {
    env::var("TERM_PROGRAM").or_else(|_| env::var("TERM")).ok()
}

pub fn log_capabilities(caps: &TerminalCapabilities, verbose: bool) {
    if !verbose {
        return;
    }

    eprintln!(
        "[Terminal] Color: {:?}, Refresh: {:.0}Hz, Mouse: {}, Term: {:?}",
        caps.color_capability,
        caps.estimated_refresh_rate,
        caps.supports_mouse_tracking,
        caps.terminal_name
    );
}
