use crate::cli::Palette;
use crate::simulation::config::Preset;
use crate::terminal::control::{palette_name, preset_name};

pub struct OverlayRenderer;

pub struct HelpOverlay {
    lines: [&'static str; 9],
}

impl HelpOverlay {
    pub fn new() -> Self {
        Self {
            lines: [
                "┌─ tslime controls ───────────────────────┐",
                "│ p: Pause/Resume                         │",
                "│ r: Restart                              │",
                "│ 1-4: Presets (Network,Exploratory,etc) │",
                "│ +/-: Time scale (0.5x - 4.0x)           │",
                "│ c: Cycle palette                        │",
                "│ h: Toggle this help                     │",
                "│ q: Quit                                 │",
                "└────────────────────────────────────────┘",
            ],
        }
    }

    pub fn x(&self) -> usize {
        2
    }

    pub fn y(&self) -> usize {
        2
    }

    pub fn width(&self) -> usize {
        42
    }

    pub fn height(&self) -> usize {
        9
    }

    pub fn lines(&self) -> &[&'static str; 9] {
        &self.lines
    }
}

impl OverlayRenderer {
    pub fn build_status_line(
        is_paused: bool,
        preset: Preset,
        time_scale: f32,
        palette: Palette,
        width: usize,
    ) -> String {
        let paused_text = if is_paused { " [PAUSED]" } else { "" };
        let preset_text = preset_name(preset);
        let palette_text = palette_name(palette);
        let time_text = format!("{:.1}x", time_scale);

        format!("{} | {} | {} | {}", preset_text, time_text, palette_text, paused_text)
    }

    pub fn status_line_x(status_line: &str, width: usize) -> usize {
        if status_line.len() < width {
            2
        } else {
            width.saturating_sub(status_line.len() + 2)
        }
    }

    pub fn paused_overlay_x(width: usize) -> usize {
        let paused_text = "[ PAUSED ]";
        width.saturating_sub(paused_text.len() + 2)
    }
}
