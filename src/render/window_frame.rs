//! Window frame rendering for terminal display.
//!
//! This module provides window frame visualization effects around the simulation area,
//! supporting multiple styles from simple accent frames to reactive glow effects.

use crate::render::palette::RgbColor;
use crate::simulation::config::WindowFrame;
use crate::terminal::frame_buffer::{Cell, FrameBuffer};

/// Renders window frames on the frame buffer.
pub struct WindowFrameRenderer {
    mode: WindowFrame,
    accent_color: RgbColor,
}

impl WindowFrameRenderer {
    /// Creates a new window frame renderer with the specified mode and accent color.
    pub fn new(mode: WindowFrame, accent_color: RgbColor) -> Self {
        Self { mode, accent_color }
    }

    /// Renders the window frame onto the frame buffer.
    ///
    /// The `activity` parameter is used for reactive mode and should contain
    /// activity levels (0.0-1.0) for each simulation cell.
    pub fn render(&self, buffer: &mut FrameBuffer, activity: Option<&[f32]>) {
        match self.mode {
            WindowFrame::None => {}
            WindowFrame::Negative => self.render_negative(buffer),
            WindowFrame::Accented => self.render_accented(buffer),
            WindowFrame::Glow => self.render_glow(buffer),
            WindowFrame::Reactive => {
                if let Some(act) = activity {
                    self.render_reactive(buffer, act);
                } else {
                    self.render_accented(buffer);
                }
            }
            WindowFrame::Food => self.render_food(buffer),
            WindowFrame::Frame => self.render_frame(buffer),
        }
    }

    /// Renders a solid accent-colored border.
    fn render_accented(&self, buffer: &mut FrameBuffer) {
        let width = buffer.width();
        let height = buffer.height();
        let color = self.accent_color;

        // Top and bottom borders
        for x in 0..width {
            buffer.set_cell(x, 0, Cell::new('█').with_fg(color));
            buffer.set_cell(x, height - 1, Cell::new('█').with_fg(color));
        }

        // Left and right borders
        for y in 1..height - 1 {
            buffer.set_cell(0, y, Cell::new('█').with_fg(color));
            buffer.set_cell(width - 1, y, Cell::new('█').with_fg(color));
        }
    }

    /// Renders a gradient border fading from accent color inward.
    fn render_glow(&self, buffer: &mut FrameBuffer) {
        let width = buffer.width();
        let height = buffer.height();

        for i in 0..3 {
            let alpha = 1.0 - (i as f32 * 0.3);
            let color = self.accent_color.with_alpha(alpha);
            let block_char = match i {
                0 => '█',
                1 => '▓',
                _ => '▒',
            };

            // Draw gradient layers
            for x in i..width - i {
                buffer.set_cell(x, i, Cell::new(block_char).with_fg(color));
                buffer.set_cell(x, height - 1 - i, Cell::new(block_char).with_fg(color));
            }
            for y in i..height - i {
                buffer.set_cell(i, y, Cell::new(block_char).with_fg(color));
                buffer.set_cell(width - 1 - i, y, Cell::new(block_char).with_fg(color));
            }
        }
    }

    /// Renders a border that responds to nearby agent activity.
    fn render_reactive(&self, buffer: &mut FrameBuffer, activity: &[f32]) {
        let width = buffer.width();
        let height = buffer.height();

        for y in 0..height {
            for x in 0..width {
                // Check if this is a border cell (2-cell thickness)
                let is_border = x < 2 || x >= width - 2 || y < 2 || y >= height - 2;
                if !is_border {
                    continue;
                }

                // Get activity from nearest simulation cell
                let sim_x = x.saturating_sub(2).min(width.saturating_sub(5));
                let sim_y = y.saturating_sub(2).min(height.saturating_sub(5));

                // Calculate activity index (simulation area is smaller due to border)
                let sim_width = width.saturating_sub(4);
                if sim_width == 0 {
                    continue;
                }
                let idx = sim_y * sim_width + sim_x;
                let activity_level = activity.get(idx).copied().unwrap_or(0.0).clamp(0.0, 1.0);

                if activity_level > 0.01 {
                    // Blend accent color with brightness based on activity
                    let brightness = (activity_level * 255.0) as u8;
                    let base = RgbColor::new(brightness, brightness, brightness);
                    let color = base.blend(&self.accent_color, activity_level);
                    buffer.set_cell(x, y, Cell::new('█').with_fg(color));
                } else {
                    // Dim border when no activity
                    let dim_color = self.accent_color.with_alpha(0.1);
                    buffer.set_cell(x, y, Cell::new('░').with_fg(dim_color));
                }
            }
        }
    }

    /// Renders a food-colored border (attracts agents).
    fn render_food(&self, buffer: &mut FrameBuffer) {
        let width = buffer.width();
        let height = buffer.height();

        // Food color - bioluminescent green matching the slime theme
        let food_color = RgbColor::new(0x39, 0xD3, 0x53);

        // Top and bottom borders
        for x in 0..width {
            buffer.set_cell(x, 0, Cell::new('█').with_fg(food_color));
            buffer.set_cell(x, height - 1, Cell::new('█').with_fg(food_color));
        }

        // Left and right borders
        for y in 1..height - 1 {
            buffer.set_cell(0, y, Cell::new('█').with_fg(food_color));
            buffer.set_cell(width - 1, y, Cell::new('█').with_fg(food_color));
        }
    }

    /// Clears the border area (negative space - no simulation rendered there).
    /// Uses 1 row on top/bottom and 2 columns on left/right.
    fn render_negative(&self, buffer: &mut FrameBuffer) {
        let width = buffer.width();
        let height = buffer.height();

        if width < 5 || height < 3 {
            return;
        }

        // Clear top and bottom rows (1 row each)
        for x in 0..width {
            buffer.set_cell(x, 0, Cell::new(' '));
            buffer.set_cell(x, height - 1, Cell::new(' '));
        }

        // Clear left and right 2 columns
        for y in 1..height - 1 {
            buffer.set_cell(0, y, Cell::new(' '));
            buffer.set_cell(1, y, Cell::new(' '));
            buffer.set_cell(width - 2, y, Cell::new(' '));
            buffer.set_cell(width - 1, y, Cell::new(' '));
        }
    }

    /// Renders a frame around the visible simulation area.
    /// This combines negative space with a visible border around the simulation.
    /// Uses 1 row on top/bottom and 2 columns on left/right.
    /// The frame hugs the simulation cells at row 1 and columns 2/width-3.
    fn render_frame(&self, buffer: &mut FrameBuffer) {
        let width = buffer.width();
        let height = buffer.height();
        let color = self.accent_color;

        if width < 6 || height < 3 {
            return;
        }

        // First clear the outer border (negative space)
        // Clear top and bottom rows (1 row each)
        for x in 0..width {
            buffer.set_cell(x, 0, Cell::new(' '));
            buffer.set_cell(x, height - 1, Cell::new(' '));
        }

        // Clear left and right 2 columns
        for y in 1..height - 1 {
            buffer.set_cell(0, y, Cell::new(' '));
            buffer.set_cell(1, y, Cell::new(' '));
            buffer.set_cell(width - 2, y, Cell::new(' '));
            buffer.set_cell(width - 1, y, Cell::new(' '));
        }

        // Then draw the frame hugging the simulation area
        // The frame is at row 1 (just below top cleared row)
        // and columns 2 to width-3 (just inside the 2-column side borders)
        // Top-left corner
        buffer.set_cell(2, 1, Cell::new('┌').with_fg(color));
        // Top-right corner
        buffer.set_cell(width - 3, 1, Cell::new('┐').with_fg(color));
        // Bottom-left corner
        buffer.set_cell(2, height - 2, Cell::new('└').with_fg(color));
        // Bottom-right corner
        buffer.set_cell(width - 3, height - 2, Cell::new('┘').with_fg(color));

        // Top and bottom edges
        for x in 3..width - 3 {
            buffer.set_cell(x, 1, Cell::new('─').with_fg(color));
            buffer.set_cell(x, height - 2, Cell::new('─').with_fg(color));
        }

        // Left and right edges
        for y in 2..height - 2 {
            buffer.set_cell(2, y, Cell::new('│').with_fg(color));
            buffer.set_cell(width - 3, y, Cell::new('│').with_fg(color));
        }
    }

    /// Returns true if this mode reduces the available simulation area.
    pub fn reduces_display_area(&self) -> bool {
        self.mode.reduces_display_area()
    }

    /// Returns the window frame thickness in cells.
    pub fn thickness(&self) -> usize {
        self.mode.thickness()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_window_frame_renderer_creation() {
        let color = RgbColor::new(255, 0, 0);
        let renderer = WindowFrameRenderer::new(WindowFrame::Accented, color);
        assert_eq!(renderer.mode, WindowFrame::Accented);
        assert_eq!(renderer.thickness(), 1);
        assert!(!renderer.reduces_display_area());
    }

    #[test]
    fn test_negative_space_reduces_area() {
        let color = RgbColor::new(255, 0, 0);
        let renderer = WindowFrameRenderer::new(WindowFrame::Negative, color);
        assert!(renderer.reduces_display_area());
        assert_eq!(renderer.thickness(), 2);
    }

    #[test]
    fn test_glow_thickness() {
        let color = RgbColor::new(255, 0, 0);
        let renderer = WindowFrameRenderer::new(WindowFrame::Glow, color);
        assert_eq!(renderer.thickness(), 3);
        assert!(!renderer.reduces_display_area());
    }
}
