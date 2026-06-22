//! Window layout computation for the simulation viewport.
//!
//! This module provides pure geometry calculations for positioning and sizing
//! the simulation window inside the terminal, given an aspect ratio, padding
//! settings, and fallback thresholds. No I/O is performed here.

use crate::simulation::config::{Aspect, TerminalSizeThreshold, WindowPadding};

/// Default frame ring thickness in **columns** on each of the left/right sides.
///
/// The ring = the border (1 cell at the outer edge) plus a background **matte**
/// between the border and the simulation. This is the default; the matte is
/// configurable via `SimConfig::frame_matte_cols` (ring = matte + 1 border cell).
/// Horizontal is wider than [`FRAME_RING_ROWS`] to offset the ~1:2 terminal cell
/// aspect so the matte reads as visually even. Used by `Window::default` and the
/// no-layout (fullscreen) render path.
pub const FRAME_RING_COLS: usize = crate::config_defaults::frame_matte::DEFAULT_COLS + 1;

/// Default frame ring thickness in **rows** on each of the top/bottom sides.
/// See [`FRAME_RING_COLS`].
pub const FRAME_RING_ROWS: usize = crate::config_defaults::frame_matte::DEFAULT_ROWS + 1;

/// How the window layout was resolved when terminal space was limited.
pub enum FallbackMode {
    /// Normal layout with padding and centered frame.
    Normal,
    /// No padding; frame hugs terminal edges (simulation fits without padding).
    EdgeHug,
    /// Terminal too small even for a frame; simulation fills the entire terminal.
    Fullscreen,
}

/// Computed positions and dimensions for the frame and simulation areas.
pub struct WindowLayout {
    /// Outer padding in terminal cells (0 in EdgeHug/Fullscreen modes).
    pub pad: usize,
    /// Frame top-left column (0-based).
    pub frame_x: usize,
    /// Frame top-left row (0-based).
    pub frame_y: usize,
    /// Frame width in terminal columns (includes 1-cell border on each side).
    pub frame_w: usize,
    /// Frame height in terminal rows (includes 1-cell border on each side).
    pub frame_h: usize,
    /// Simulation area top-left column (frame_x + 1 in Normal/EdgeHug).
    pub sim_x: usize,
    /// Simulation area top-left row (frame_y + 1 in Normal/EdgeHug).
    pub sim_y: usize,
    /// Simulation area width in terminal columns.
    pub sim_w: usize,
    /// Simulation area height in terminal rows.
    pub sim_h: usize,
    /// Which fallback mode was used to compute this layout.
    pub fallback: FallbackMode,
}

/// Computes terminal layout geometry for a windowed simulation viewport.
///
/// Given terminal dimensions, `compute_rects` returns a [`WindowLayout`] that
/// describes where the border frame and inner simulation area should be drawn.
/// Aspect ratio is preserved; fallbacks activate when space is too tight.
pub struct Window {
    /// Desired visual aspect ratio of the simulation area.
    pub aspect: Aspect,
    /// Outer padding strategy (auto 5% or fixed cells).
    pub padding: WindowPadding,
    /// Frame ring thickness in columns per side (border + matte). The sim is
    /// inset by this on the left/right.
    pub ring_cols: usize,
    /// Frame ring thickness in rows per side (border + matte). The sim is inset
    /// by this on the top/bottom.
    pub ring_rows: usize,
    /// Minimum sim size before triggering EdgeHug fallback.
    pub min_sim_size: TerminalSizeThreshold,
    /// Minimum sim size before triggering Fullscreen fallback.
    pub min_frame_size: TerminalSizeThreshold,
}

impl Default for Window {
    fn default() -> Self {
        Self {
            aspect: Aspect::default(),
            padding: WindowPadding::Auto,
            ring_cols: FRAME_RING_COLS,
            ring_rows: FRAME_RING_ROWS,
            min_sim_size: TerminalSizeThreshold {
                width: 20,
                height: 10,
            },
            min_frame_size: TerminalSizeThreshold {
                width: 12,
                height: 6,
            },
        }
    }
}

impl Window {
    /// Computes the frame and simulation rectangles for a terminal of
    /// `term_w` columns by `term_h` rows, including which fallback mode applied.
    pub fn compute_rects(&self, term_w: usize, term_h: usize) -> WindowLayout {
        let pad = match self.padding {
            WindowPadding::Auto => {
                let min_dim = term_w.min(term_h);
                ((min_dim as f32 * 0.05).floor() as usize).max(2)
            }
            WindowPadding::Fixed(n) => n,
        };

        // Available space after padding; reserve the frame ring per side (outer
        // accent edge + inner background separator). Horizontal is thicker than
        // vertical (see FRAME_RING_COLS/ROWS) for visual balance. The simulation
        // is inset by the ring so it never renders under the border.
        let avail_w = term_w.saturating_sub(pad * 2);
        let avail_h = term_h.saturating_sub(pad * 2);
        let inner_w = avail_w.saturating_sub(self.ring_cols * 2);
        let inner_h = avail_h.saturating_sub(self.ring_rows * 2);

        // Fit sim to aspect (cell_ratio = sim_cells_w / sim_cells_h)
        let cell_ratio = self.aspect.cell_ratio();
        let (sim_w, sim_h) = if inner_w == 0 || inner_h == 0 {
            (0, 0)
        } else {
            let w_from_h = (inner_h as f32 * cell_ratio) as usize;
            if w_from_h <= inner_w {
                (w_from_h, inner_h)
            } else {
                (inner_w, (inner_w as f32 / cell_ratio) as usize)
            }
        };

        // Fullscreen fallback: can't fit even min_frame_size
        if sim_w < self.min_frame_size.width || sim_h < self.min_frame_size.height {
            return WindowLayout {
                pad: 0,
                frame_x: 0,
                frame_y: 0,
                frame_w: term_w,
                frame_h: term_h,
                sim_x: 0,
                sim_y: 0,
                sim_w: term_w,
                sim_h: term_h,
                fallback: FallbackMode::Fullscreen,
            };
        }

        // Edge-hug fallback: sim too small with padding, but OK without
        if sim_w < self.min_sim_size.width || sim_h < self.min_sim_size.height {
            let eh_sim_w = term_w.saturating_sub(self.ring_cols * 2);
            let eh_sim_h = term_h.saturating_sub(self.ring_rows * 2);
            return WindowLayout {
                pad: 0,
                frame_x: 0,
                frame_y: 0,
                frame_w: term_w,
                frame_h: term_h,
                sim_x: self.ring_cols,
                sim_y: self.ring_rows,
                sim_w: eh_sim_w,
                sim_h: eh_sim_h,
                fallback: FallbackMode::EdgeHug,
            };
        }

        // Normal: center frame in terminal (aspect-aware ring on each side)
        let frame_w = sim_w + self.ring_cols * 2;
        let frame_h = sim_h + self.ring_rows * 2;
        let frame_x = term_w.saturating_sub(frame_w) / 2;
        let frame_y = term_h.saturating_sub(frame_h) / 2;

        WindowLayout {
            pad,
            frame_x,
            frame_y,
            frame_w,
            frame_h,
            sim_x: frame_x + self.ring_cols,
            sim_y: frame_y + self.ring_rows,
            sim_w,
            sim_h,
            fallback: FallbackMode::Normal,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_window() -> Window {
        Window::default()
    }

    #[test]
    fn test_auto_padding_minimum() {
        // Terminal large enough for Normal mode: padding should be at least 2
        let w = default_window();
        let layout = w.compute_rects(80, 40);
        assert!(layout.pad >= 2);
    }

    #[test]
    fn test_auto_padding_proportional() {
        // Large terminal: padding > 2 (5% of min dim = 5% of 100 = 5)
        let w = default_window();
        let layout = w.compute_rects(200, 100);
        assert_eq!(layout.pad, 5); // floor(0.05 * 100) = 5
    }

    #[test]
    fn test_fixed_padding() {
        let mut w = default_window();
        w.padding = WindowPadding::Fixed(3);
        let layout = w.compute_rects(80, 40);
        assert_eq!(layout.pad, 3);
    }

    #[test]
    fn test_frame_is_inside_padding() {
        let w = default_window();
        let layout = w.compute_rects(120, 60);
        assert!(layout.frame_x >= layout.pad);
        assert!(layout.frame_y >= layout.pad);
        assert!(layout.frame_x + layout.frame_w <= 120 - layout.pad);
        assert!(layout.frame_y + layout.frame_h <= 60 - layout.pad);
    }

    #[test]
    fn test_sim_inside_frame() {
        let w = default_window();
        let layout = w.compute_rects(120, 60);
        // Aspect-aware ring: FRAME_RING_COLS left/right, FRAME_RING_ROWS top/bottom.
        assert_eq!(layout.sim_x, layout.frame_x + FRAME_RING_COLS);
        assert_eq!(layout.sim_y, layout.frame_y + FRAME_RING_ROWS);
        assert_eq!(layout.sim_w, layout.frame_w - FRAME_RING_COLS * 2);
        assert_eq!(layout.sim_h, layout.frame_h - FRAME_RING_ROWS * 2);
    }

    #[test]
    fn test_aspect_ratio_respected() {
        let w = default_window(); // 3:2, cell_ratio = 3.0
        let layout = w.compute_rects(120, 60);
        // sim_w / sim_h should be close to cell_ratio (3.0)
        let ratio = layout.sim_w as f32 / layout.sim_h as f32;
        assert!((ratio - 3.0).abs() <= 1.0, "ratio was {}", ratio);
    }

    #[test]
    fn test_edge_hug_fallback() {
        // Terminal too small for padding but big enough for frame
        let mut w = default_window();
        w.min_sim_size = TerminalSizeThreshold {
            width: 50,
            height: 30,
        }; // force edge-hug
        w.min_frame_size = TerminalSizeThreshold {
            width: 5,
            height: 3,
        };
        let layout = w.compute_rects(30, 15);
        assert!(matches!(layout.fallback, FallbackMode::EdgeHug));
        assert_eq!(layout.pad, 0);
        assert_eq!(layout.frame_x, 0);
    }

    #[test]
    fn test_fullscreen_fallback() {
        // Terminal too small even for frame
        let mut w = default_window();
        w.min_frame_size = TerminalSizeThreshold {
            width: 100,
            height: 50,
        };
        let layout = w.compute_rects(20, 10);
        assert!(matches!(layout.fallback, FallbackMode::Fullscreen));
        assert_eq!(layout.sim_x, 0);
        assert_eq!(layout.sim_y, 0);
        assert_eq!(layout.sim_w, 20);
        assert_eq!(layout.sim_h, 10);
    }
}
