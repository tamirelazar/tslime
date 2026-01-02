use crate::cli::Palette;
use crate::render::dither::DitherMode;
use crate::simulation::config::Attractor;
use crate::simulation::config::Obstacle;
use crate::simulation::config::Preset;
use crate::terminal::control::{palette_name, preset_name};

pub struct OverlayRenderer;

impl OverlayRenderer {
    pub fn build_status_line(
        _is_paused: bool,
        preset: Preset,
        time_scale: f32,
        palette: Palette,
        dither_mode: DitherMode,
        _width: usize,
    ) -> String {
        let paused_text = if _is_paused { " [PAUSED]" } else { "" };
        let preset_text = preset_name(preset);
        let palette_text = palette_name(palette);
        let time_text = format!("{:.1}x", time_scale);
        let dither_text = match dither_mode {
            DitherMode::None => "".to_string(),
            DitherMode::Ordered { intensity, .. } => format!(" D:{:.1}", intensity),
            DitherMode::ErrorDiffusion { .. } => " ED".to_string(),
            DitherMode::Hybrid { intensity, .. } => format!(" H:{:.1}", intensity),
        };

        format!(
            "{} | {} | {} | {}{}",
            preset_text, time_text, palette_text, dither_text, paused_text
        )
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

    pub fn build_help_with_attractors(base_help: &[&str], attractors: &[Attractor]) -> Vec<String> {
        let mut lines: Vec<String> = base_help.iter().map(|s| s.to_string()).collect();

        if !attractors.is_empty() {
            lines.push(String::new());
            lines.push("┌─ Attractors─────────────────────────────┐".to_string());

            for (i, attractor) in attractors.iter().enumerate() {
                let kind = if attractor.strength > 0.0 {
                    "attract"
                } else {
                    "repel"
                };
                let strength = attractor.strength.abs();
                lines.push(format!(
                    "│{:2}: ({:>4},{:>4}) {:^7} s: {:>4.1}          │",
                    i + 1,
                    attractor.x as i32,
                    attractor.y as i32,
                    kind,
                    strength,
                ));
            }

            lines.push("└─────────────────────────────────────────┘".to_string());
        }

        lines
    }

    pub fn build_help_with_obstacles(base_help: &[&str], obstacles: &[Obstacle]) -> Vec<String> {
        let mut lines: Vec<String> = base_help.iter().map(|s| s.to_string()).collect();

        if !obstacles.is_empty() {
            lines.push(String::new());
            lines.push("┌─ Obstacles──────────────────────────────┐".to_string());

            for (i, obstacle) in obstacles.iter().enumerate() {
                match obstacle {
                    Obstacle::Circle { x, y, radius } => {
                        lines.push(format!(
                            "│{:2}: circle ({:>4},{:>4}) r: {:>4.1}        │",
                            i + 1,
                            *x as i32,
                            *y as i32,
                            radius,
                        ));
                    }
                    Obstacle::Rect {
                        x,
                        y,
                        width,
                        height,
                    } => {
                        lines.push(format!(
                            "│{:2}: rect  ({:>4},{:>4}) {:>4.1}x{:>4.1}   │",
                            i + 1,
                            *x as i32,
                            *y as i32,
                            width,
                            height,
                        ));
                    }
                }
            }

            lines.push("└─────────────────────────────────────────┘".to_string());
        }

        lines
    }

    #[cfg(test)]
    #[allow(dead_code)]
    pub fn check_attractor_section_lengths(lines: &[String], base_help_len: usize) -> bool {
        if lines.len() <= base_help_len {
            return true;
        }
        let attractor_section_start = base_help_len + 1; // Skip empty line after base help
        let attractor_lines = &lines[attractor_section_start..];
        if attractor_lines.is_empty() {
            return true;
        }
        let target_len = attractor_lines[0].chars().count();
        attractor_lines
            .iter()
            .all(|line| line.chars().count() == target_len)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::simulation::config::Attractor;

    #[test]
    fn test_attractor_overlay_no_attractors() {
        let attractors: Vec<Attractor> = vec![];
        let base_help = [
            "┌─ tslime controls ───────────────────────┐",
            "│ h: Toggle help                          │",
            "└─────────────────────────────────────────┘",
        ];
        let lines = OverlayRenderer::build_help_with_attractors(&base_help, &attractors);
        assert_eq!(lines, base_help);
    }

    #[test]
    fn test_attractor_overlay_single_attractor() {
        let attractors = vec![Attractor::new(200.0, 200.0, 1.0)];
        let base_help = [
            "┌─ tslime controls ───────────────────────┐",
            "│ h: Toggle help                          │",
            "└─────────────────────────────────────────┘",
        ];
        let lines = OverlayRenderer::build_help_with_attractors(&base_help, &attractors);
        assert!(
            lines.len() > base_help.len(),
            "Should add attractor section"
        );
        assert!(
            OverlayRenderer::check_attractor_section_lengths(&lines, base_help.len()),
            "Single attractor overlay should have consistent line lengths"
        );
    }

    #[test]
    fn test_attractor_overlay_max_strength() {
        let attractors = vec![Attractor::new(100.0, 100.0, 10.0)];
        let base_help = [
            "┌─ tslime controls ───────────────────────┐",
            "│ h: Toggle help                          │",
            "└─────────────────────────────────────────┘",
        ];
        let lines = OverlayRenderer::build_help_with_attractors(&base_help, &attractors);
        assert!(
            lines.len() > base_help.len(),
            "Should add attractor section"
        );
        assert!(
            OverlayRenderer::check_attractor_section_lengths(&lines, base_help.len()),
            "Max strength attractor should still have consistent line lengths"
        );
    }

    #[test]
    fn test_attractor_overlay_negative_coordinates() {
        let attractors = vec![Attractor::new(-50.0, -100.0, 1.0)];
        let base_help = [
            "┌─ tslime controls ───────────────────────┐",
            "│ h: Toggle help                          │",
            "└─────────────────────────────────────────┘",
        ];
        let lines = OverlayRenderer::build_help_with_attractors(&base_help, &attractors);
        assert!(
            lines.len() > base_help.len(),
            "Should add attractor section"
        );
        assert!(
            OverlayRenderer::check_attractor_section_lengths(&lines, base_help.len()),
            "Negative coordinates should still have consistent line lengths"
        );
    }

    #[test]
    fn test_attractor_overlay_multiple_attractors() {
        let attractors = vec![
            Attractor::new(200.0, 200.0, 1.0),
            Attractor::new(100.0, 100.0, -0.5),
            Attractor::new(300.0, 150.0, 2.0),
        ];
        let base_help = [
            "┌─ tslime controls ───────────────────────┐",
            "│ h: Toggle help                          │",
            "└─────────────────────────────────────────┘",
        ];
        let lines = OverlayRenderer::build_help_with_attractors(&base_help, &attractors);
        assert!(
            lines.len() > base_help.len(),
            "Should add attractor section"
        );
        assert!(
            OverlayRenderer::check_attractor_section_lengths(&lines, base_help.len()),
            "Multiple attractors should have consistent line lengths"
        );
    }
}
