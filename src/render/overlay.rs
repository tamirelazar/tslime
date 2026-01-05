use crate::cli::Palette;
use crate::render::dither::DitherMode;
use crate::simulation::config::Attractor;
use crate::simulation::config::MouseAttractor;
use crate::simulation::config::Obstacle;
use crate::simulation::config::Preset;
use crate::terminal::control::{palette_name, preset_name};

pub struct HelpOverlay;

impl HelpOverlay {
    pub fn build_overlay() -> Vec<String> {
        vec![
            "┌─ HELP ─────────────────────────────────┐".to_string(),
            "│ p: Pause    r: Restart      q: Quit    │".to_string(),
            "│ h: Controls  ?: This help     \\: Stats │".to_string(),
            "│ +/-: Speed  c/C: Palette   1-7: Preset │".to_string(),
            "│ d: Dither   m: Mode        [/]: Adjust │".to_string(),
            "│                                        │".to_string(),
            "│ Press h for detailed controls          │".to_string(),
            "└────────────────────────────────────────┘".to_string(),
        ]
    }

    pub fn width() -> usize {
        42
    }
}

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
                    Obstacle::Image {
                        path,
                        x: _,
                        y: _,
                        width,
                        height,
                        invert: _,
                        threshold: _,
                    } => {
                        let filename = std::path::Path::new(path)
                            .file_name()
                            .map(|n| n.to_string_lossy().into_owned())
                            .unwrap_or_else(|| path.clone());
                        lines.push(format!(
                            "│{:2}: image {:>20} {:>3}x{:>3}    │",
                            i + 1,
                            &filename[..filename.len().min(20)],
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

    pub fn build_help_with_mouse_attractors(
        base_help: &[&str],
        mouse_attractors: &[MouseAttractor],
        _sim_width: usize,
        _sim_height: usize,
    ) -> Vec<String> {
        let mut lines: Vec<String> = base_help.iter().map(|s| s.to_string()).collect();

        if !mouse_attractors.is_empty() {
            lines.push(String::new());
            lines.push("┌─ Mouse Attractors ──────────────────────┐".to_string());

            for (i, ma) in mouse_attractors.iter().enumerate() {
                let kind = if ma.strength > 0.0 {
                    "attract"
                } else {
                    "repel"
                };
                let remaining = ma.timeout_seconds - ma.created_at.elapsed().as_secs_f32();
                let remaining_str = if remaining > 0.0 {
                    format!("{:.1}s", remaining)
                } else {
                    "expired".to_string()
                };
                lines.push(format!(
                    "│{:2}: ({:>4},{:>4}) {:^7} s: {:>4.1} {:>6} │",
                    i + 1,
                    ma.x as i32,
                    ma.y as i32,
                    kind,
                    ma.strength.abs(),
                    remaining_str,
                ));
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

pub struct StatsOverlay;

impl StatsOverlay {
    pub const WIDTH: usize = 20;

    #[allow(clippy::too_many_arguments)]
    pub fn build_overlay(
        agent_count: usize,
        trail_sum: f32,
        trail_capacity: f32,
        entropy: f32,
        fps: f32,
        avg_fps: f32,
        frame_count: u64,
        elapsed_seconds: f32,
        _term_width: usize,
    ) -> Vec<String> {
        let trail_percent = if trail_capacity > 0.0 {
            (trail_sum / trail_capacity * 100.0).min(99.9)
        } else {
            0.0
        };

        let elapsed_str = format_elapsed_time(elapsed_seconds);

        vec![
            "┌─ STATS ──────────┐".to_string(),
            format!("│ Agents: {:>8} │", agent_count),
            format!("│ Trail:  {:>7.1}% │", trail_percent),
            format!("│ Entropy: {:>7.2} │", entropy),
            format!("│ FPS: {:>4.0} ({:>4.0}) │", fps, avg_fps),
            format!("│ Frames: {:>8} │", frame_count),
            format!("│ Time: {:>10} │", elapsed_str),
            "└──────────────────┘".to_string(),
        ]
    }

    pub fn calculate_x_position(term_width: usize) -> usize {
        if term_width > Self::WIDTH + 2 {
            term_width.saturating_sub(Self::WIDTH + 2)
        } else {
            1
        }
    }

    pub fn calculate_entropy(trail_map: &[f32], sample_rate: usize) -> f32 {
        if trail_map.is_empty() {
            return 0.0;
        }

        const NUM_BINS: usize = 16;
        let mut bins = [0usize; NUM_BINS];
        let mut total_samples = 0usize;

        for (i, &value) in trail_map.iter().enumerate() {
            if i % sample_rate == 0 && value > 0.01 {
                let normalized = (value / 10.0).clamp(0.0, 0.9999);
                let bin_idx = (normalized * NUM_BINS as f32) as usize;
                bins[bin_idx] += 1;
                total_samples += 1;
            }
        }

        if total_samples < 2 {
            return 0.0;
        }

        let mut entropy = 0.0f32;
        for &count in bins.iter() {
            if count > 0 {
                let p = count as f32 / total_samples as f32;
                entropy -= p * p.log2();
            }
        }

        let max_entropy = (NUM_BINS as f32).log2();
        if max_entropy > 0.0 {
            (entropy / max_entropy * 8.0).clamp(0.0, 8.0)
        } else {
            0.0
        }
    }
}

fn format_elapsed_time(seconds: f32) -> String {
    let total_secs = seconds as u64;
    let hours = total_secs / 3600;
    let minutes = (total_secs % 3600) / 60;
    let secs = total_secs % 60;

    if hours > 0 {
        format!("{}:{:02}:{:02}", hours, minutes, secs)
    } else {
        format!("{}:{:02}", minutes, secs)
    }
}

#[cfg(test)]
mod stats_tests {
    use super::*;

    #[test]
    fn test_stats_overlay_format() {
        let lines = StatsOverlay::build_overlay(
            50000,
            1234567.0,
            8000000.0,
            5.5,
            30.0,
            28.5,
            1234,
            125.5,
            80,
        );

        assert!(!lines.is_empty());
        assert!(lines[0].starts_with('┌'));
        assert!(lines.last().unwrap().starts_with('└'));
        assert!(lines.iter().all(|l| l.starts_with('│') || l.starts_with('┌') || l.starts_with('└')));

        // New compact format is 20 chars wide
        let max_len = lines.iter().map(|l| l.chars().count()).max().unwrap();
        assert_eq!(max_len, StatsOverlay::WIDTH);
    }

    #[test]
    fn test_stats_overlay_position() {
        assert_eq!(StatsOverlay::calculate_x_position(80), 58);
        assert_eq!(StatsOverlay::calculate_x_position(120), 98);
        assert_eq!(StatsOverlay::calculate_x_position(20), 1);
    }

    #[test]
    fn test_stats_overlay_with_zero_values() {
        let lines = StatsOverlay::build_overlay(
            0,
            0.0,
            1000000.0,
            0.0,
            0.0,
            0.0,
            0,
            0.0,
            80,
        );

        assert!(!lines.is_empty());
        assert!(lines.iter().any(|l| l.contains("0.0%")));
    }

    #[test]
    fn test_entropy_calculation() {
        let uniform = vec![1.0; 40000];
        let entropy_uniform = StatsOverlay::calculate_entropy(&uniform, 100);
        eprintln!("uniform entropy: {}", entropy_uniform);
        assert!(entropy_uniform < 2.0, "uniform should have low entropy, got {}", entropy_uniform);

        let varied: Vec<f32> = (0..40000).map(|i| i as f32 / 400.0).collect();
        let entropy_varied = StatsOverlay::calculate_entropy(&varied, 100);
        eprintln!("varied entropy: {}", entropy_varied);
        assert!(entropy_varied > entropy_uniform, "varied ({}) should have higher entropy than uniform ({})", entropy_varied, entropy_uniform);
    }

    #[test]
    fn test_entropy_empty_trail() {
        let empty: Vec<f32> = vec![];
        let entropy = StatsOverlay::calculate_entropy(&empty, 10);
        assert_eq!(entropy, 0.0);
    }

    #[test]
    fn test_format_elapsed_time() {
        assert_eq!(format_elapsed_time(30.0), "0:30");
        assert_eq!(format_elapsed_time(90.0), "1:30");
        assert_eq!(format_elapsed_time(3661.0), "1:01:01");
        assert_eq!(format_elapsed_time(0.0), "0:00");
    }
}
