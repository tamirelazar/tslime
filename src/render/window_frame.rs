//! Window frame rendering for terminal display.
//!
//! This module draws the chrome ring around the simulation viewport. The
//! windowed layout ([`crate::render::window`]) reserves an **aspect-aware ring**
//! (`ring_cols` columns left/right, `ring_rows` rows top/bottom, configurable via
//! `SimConfig::frame_matte_cols`/`frame_matte_rows`) and insets the simulation by
//! that ring, so the simulation never renders under the border. The horizontal
//! ring is wider than the vertical one so the border + inner matte read as
//! visually even on terminals whose cells are roughly 1:2 (width:height). The
//! renderer fills the ring with an outer accent band plus an inner
//! background-colored matte, which also keeps trail content from bleeding past
//! the frame in the blit path (`render_window_frame_at`, which skips blank cells).

use crate::render::palette::RgbColor;
use crate::simulation::config::WindowFrame;
use crate::terminal::frame_buffer::{Cell, FrameBuffer};

/// Renders window frames on the frame buffer.
pub struct WindowFrameRenderer {
    mode: WindowFrame,
    accent_color: RgbColor,
    /// Background color for the inner separator ring (and negative space).
    /// `None` leaves those cells blank (transparent in the blit path).
    background_color: Option<RgbColor>,
    /// Frame ring thickness in columns per side (border + matte).
    ring_cols: usize,
    /// Frame ring thickness in rows per side (border + matte).
    ring_rows: usize,
}

impl WindowFrameRenderer {
    /// Creates a new window frame renderer with the specified mode, accent
    /// color, optional background color, and per-side frame ring thickness
    /// (`ring_cols`/`ring_rows` = border + background matte).
    pub fn new(
        mode: WindowFrame,
        accent_color: RgbColor,
        background_color: Option<RgbColor>,
        ring_cols: usize,
        ring_rows: usize,
    ) -> Self {
        Self {
            mode,
            accent_color,
            background_color,
            ring_cols: ring_cols.max(1),
            ring_rows: ring_rows.max(1),
        }
    }

    /// A background-fill cell: a space carrying the configured background color
    /// so it counts as non-blank (and thus blits over simulation content). When
    /// no background is configured, a plain blank space is used.
    fn bg_cell(&self) -> Cell {
        match self.background_color {
            Some(c) => Cell::new(' ').with_bg(c),
            None => Cell::new(' '),
        }
    }

    /// Renders the window frame onto the frame buffer.
    pub fn render(&self, buffer: &mut FrameBuffer) {
        match self.mode {
            WindowFrame::None => {}
            WindowFrame::Accented => self.render_accented(buffer),
            WindowFrame::Glow => self.render_glow(buffer),
            WindowFrame::Frame => self.render_frame(buffer),
        }
    }

    /// Minimum buffer width to draw a ring (two rings + at least 2 sim cols).
    fn min_width(&self) -> usize {
        self.ring_cols * 2 + 2
    }

    /// Minimum buffer height to draw a ring (two rings + at least 2 sim rows).
    fn min_height(&self) -> usize {
        self.ring_rows * 2 + 2
    }

    /// True when `(x, y)` falls inside the frame ring (not the sim interior).
    fn in_ring(&self, x: usize, y: usize, width: usize, height: usize) -> bool {
        x < self.ring_cols
            || x >= width - self.ring_cols
            || y < self.ring_rows
            || y >= height - self.ring_rows
    }

    /// Renders a solid accent border with a background separator just inside it.
    /// The ring splits into an outer accent band (half the ring thickness on
    /// each axis) and an inner background separator filling the rest.
    fn render_accented(&self, buffer: &mut FrameBuffer) {
        let width = buffer.width();
        let height = buffer.height();
        if width < self.min_width() || height < self.min_height() {
            return;
        }
        let color = self.accent_color;
        let bg = self.bg_cell();
        // Outer accent band thickness (the remaining ring is separator).
        let acc_c = (self.ring_cols / 2).max(1);
        let acc_r = (self.ring_rows / 2).max(1);

        for y in 0..height {
            for x in 0..width {
                if !self.in_ring(x, y, width, height) {
                    continue;
                }
                let is_accent = x < acc_c || x >= width - acc_c || y < acc_r || y >= height - acc_r;
                if is_accent {
                    buffer.set_cell(x, y, Cell::new('█').with_fg(color));
                } else {
                    buffer.set_cell(x, y, bg);
                }
            }
        }
    }

    /// Renders a gradient border fading from accent color inward, filling the
    /// ring from the outer edge toward the simulation.
    fn render_glow(&self, buffer: &mut FrameBuffer) {
        let width = buffer.width();
        let height = buffer.height();
        if width < self.min_width() || height < self.min_height() {
            return;
        }

        for y in 0..height {
            for x in 0..width {
                if !self.in_ring(x, y, width, height) {
                    continue;
                }
                // Normalized depth from the outer edge along each axis; the
                // shallower axis wins so corners read as the outer (bright) band.
                let dc = x.min(width - 1 - x) as f32 / self.ring_cols as f32;
                let dr = y.min(height - 1 - y) as f32 / self.ring_rows as f32;
                let depth = dc.min(dr).clamp(0.0, 1.0);
                let alpha = 1.0 - depth * 0.7;
                let ch = if depth < 0.34 {
                    '█'
                } else if depth < 0.67 {
                    '▓'
                } else {
                    '▒'
                };
                buffer.set_cell(
                    x,
                    y,
                    Cell::new(ch).with_fg(self.accent_color.with_alpha(alpha)),
                );
            }
        }
    }

    /// Renders a thin-line box at the outer ring edge, with a background **matte**
    /// between the box and the simulation. The matte (the rest of the ring) gives
    /// a visible gap so trail content does not touch the border. The matte is
    /// wider in columns than rows so it reads as visually even (see the
    /// `ring_cols`/`ring_rows` fields).
    fn render_frame(&self, buffer: &mut FrameBuffer) {
        let width = buffer.width();
        let height = buffer.height();
        let color = self.accent_color;
        if width < self.min_width() || height < self.min_height() {
            return;
        }

        let bg = self.bg_cell();
        // Fill the whole ring with the background matte first.
        for y in 0..height {
            for x in 0..width {
                if self.in_ring(x, y, width, height) {
                    buffer.set_cell(x, y, bg);
                }
            }
        }

        // Box at the outer ring edge (the matte sits between it and the sim).
        buffer.set_cell(0, 0, Cell::new('┌').with_fg(color));
        buffer.set_cell(width - 1, 0, Cell::new('┐').with_fg(color));
        buffer.set_cell(0, height - 1, Cell::new('└').with_fg(color));
        buffer.set_cell(width - 1, height - 1, Cell::new('┘').with_fg(color));

        for x in 1..width - 1 {
            buffer.set_cell(x, 0, Cell::new('─').with_fg(color));
            buffer.set_cell(x, height - 1, Cell::new('─').with_fg(color));
        }
        for y in 1..height - 1 {
            buffer.set_cell(0, y, Cell::new('│').with_fg(color));
            buffer.set_cell(width - 1, y, Cell::new('│').with_fg(color));
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

    use crate::cli::ColorMode;

    const RING_C: usize = 5;
    const RING_R: usize = 2;

    #[test]
    fn test_window_frame_renderer_creation() {
        let color = RgbColor::new(255, 0, 0);
        let renderer = WindowFrameRenderer::new(WindowFrame::Accented, color, None, RING_C, RING_R);
        assert_eq!(renderer.mode, WindowFrame::Accented);
        assert_eq!(renderer.thickness(), 1);
        assert!(!renderer.reduces_display_area());
    }

    #[test]
    fn test_frame_thickness() {
        let color = RgbColor::new(255, 0, 0);
        let renderer = WindowFrameRenderer::new(WindowFrame::Frame, color, None, RING_C, RING_R);
        assert_eq!(renderer.thickness(), 2);
        assert!(!renderer.reduces_display_area());
    }

    #[test]
    fn test_glow_thickness() {
        let color = RgbColor::new(255, 0, 0);
        let renderer = WindowFrameRenderer::new(WindowFrame::Glow, color, None, RING_C, RING_R);
        assert_eq!(renderer.thickness(), 3);
        assert!(!renderer.reduces_display_area());
    }

    /// The simulation interior (inset by the frame ring on every side) must
    /// never be touched by the Frame border. This is the regression guard for
    /// the frame-escape bug: the box sits at the outer ring edge with a matte
    /// between it and the sim.
    #[test]
    fn test_frame_does_not_draw_into_sim_interior() {
        let accent = RgbColor::new(255, 0, 0);
        let (w, h) = (24usize, 14usize);
        let mut buffer = FrameBuffer::new(w, h, ColorMode::TrueColor, None);
        // Seed the whole buffer with a sentinel glyph so any frame write shows.
        for y in 0..h {
            for x in 0..w {
                buffer.set_cell(x, y, Cell::new('S'));
            }
        }
        let renderer = WindowFrameRenderer::new(WindowFrame::Frame, accent, None, RING_C, RING_R);
        renderer.render(&mut buffer);

        // Interior = inset by the ring on every side; must remain the sentinel.
        for y in RING_R..h - RING_R {
            for x in RING_C..w - RING_C {
                assert_eq!(
                    buffer.get_cell(x, y).char,
                    'S',
                    "frame wrote into sim interior at ({x},{y})"
                );
            }
        }
        // The box sits at the outer ring edge (top-left corner); a matte
        // separates it from the sim interior.
        assert_eq!(buffer.get_cell(0, 0).char, '┌');
    }

    /// With a background color set, the inner matte is filled with a non-blank
    /// background cell (so it covers trail content in the blit path).
    #[test]
    fn test_accented_separator_uses_background_color() {
        let accent = RgbColor::new(255, 0, 0);
        let bg = RgbColor::new(10, 20, 30);
        let mut buffer = FrameBuffer::new(24, 14, ColorMode::TrueColor, None);
        let renderer =
            WindowFrameRenderer::new(WindowFrame::Accented, accent, Some(bg), RING_C, RING_R);
        renderer.render(&mut buffer);
        // A matte cell (inside the accent band) carries the background color.
        let acc_c = (RING_C / 2).max(1);
        let sep = buffer.get_cell(acc_c, RING_R - 1);
        assert_eq!(sep.bg_color_rgb, Some(bg), "matte should carry bg color");
    }
}
