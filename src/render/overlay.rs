use crate::cli::Palette;
use crate::simulation::config::Preset;
use crate::terminal::control::{palette_name, preset_name};

pub struct OverlayRenderer;

impl OverlayRenderer {
    pub fn build_status_line(
        is_paused: bool,
        preset: Preset,
        time_scale: f32,
        palette: Palette,
        _width: usize,
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

    pub fn paused_overlay_x(_width: usize) -> usize {
        let paused_text = "[ PAUSED ]";
        _width.saturating_sub(paused_text.len() + 2)
    }
}
