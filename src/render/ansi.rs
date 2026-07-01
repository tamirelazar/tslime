//! Crossterm-free ANSI frame rendering.
//!
//! Produces the same look the TUI emits, but as a plain escape-sequence
//! `String` with no `crossterm` or `terminal`-feature dependency — so it can
//! run in a WebAssembly build and be fed straight to a browser terminal
//! emulator (xterm.js).
//!
//! Two charsets are supported:
//! * **HalfBlock** — `▀` (upper half block) with foreground = top subpixel and
//!   background = bottom subpixel, giving 2× vertical resolution per row.
//! * **Ascii** — one density glyph per cell (` .:-=+*#%@`), foreground-only on
//!   the emulator background.

use crate::render::charset::{map_brightness, Charset};
use crate::render::downsample::{downsample, Cell, DownsampledFrame};
use crate::render::grid::GridRenderer;
use crate::render::palette::{
    map_brightness_rgb, truecolor_ansi, IntensityMapping, Palette, RgbColor,
};

/// Render a trail map to a truecolor ANSI frame.
///
/// * `trail` — row-major simulation trail map (`sim_width * sim_height`).
/// * `cols` / `rows` — target terminal grid. In half-block mode each row holds
///   two vertical subpixels (effective vertical resolution `rows * 2`).
/// * `max_brightness` — white-point divisor; raw trail values are divided by
///   this before the palette/charset lookup (mirrors `SimConfig::max_brightness`;
///   higher = darker).
#[allow(clippy::too_many_arguments)]
pub fn render_ansi(
    trail: &[f32],
    sim_width: usize,
    sim_height: usize,
    cols: usize,
    rows: usize,
    palette: Palette,
    charset: Charset,
    max_brightness: f32,
) -> String {
    let mut frame = DownsampledFrame::new(cols, rows);
    downsample(trail, sim_width, sim_height, cols, rows, &mut frame);
    render_ansi_cells(frame.cells(), cols, rows, palette, charset, max_brightness)
}

/// Render pre-downsampled cells to a truecolor ANSI frame. Split out from
/// [`render_ansi`] so a caller can share the downsampled `cells` with adaptive
/// brightness (which needs the same buffer to compute the white point).
pub fn render_ansi_cells(
    cells: &[Cell],
    cols: usize,
    rows: usize,
    palette: Palette,
    charset: Charset,
    max_brightness: f32,
) -> String {
    let inv_gain = if max_brightness > 0.0 {
        1.0 / max_brightness
    } else {
        1.0
    };
    let ascii = matches!(charset, Charset::Ascii);
    // Default tone curve (matches RenderArtDefaults): logarithmic base 10,
    // lifting dim trail values so the slime network remains visible at low intensity.
    let mapping = IntensityMapping::logarithmic(10.0);
    let mut out = String::with_capacity(cols * rows * 20 + rows * 8);
    out.push_str("\x1b[H");

    for y in 0..rows {
        // Absolute cursor move to column 1 keeps frames aligned even if the
        // emulator wrapped or the previous frame was shorter.
        out.push_str("\x1b[");
        out.push_str(&(y + 1).to_string());
        out.push_str(";1H");

        let (mut last_fg, mut last_bg) = (None, None);
        for x in 0..cols {
            let cell = &cells[y * cols + x];
            let top = (cell.top * inv_gain).clamp(0.0, 1.0);
            let bottom = (cell.bottom * inv_gain).clamp(0.0, 1.0);

            if ascii {
                // One density glyph per cell, foreground-only on the emulator bg.
                let (fg, glyph) = ascii_cell_fg_glyph(cell, inv_gain, &palette, &mapping);
                if last_fg != Some(fg) {
                    out.push_str(&truecolor_ansi(fg.r, fg.g, fg.b, true));
                    last_fg = Some(fg);
                }
                out.push(glyph);
            } else {
                let fg =
                    map_brightness_rgb(top, palette.clone(), false, false, 0.0, Some(&mapping));
                let bg =
                    map_brightness_rgb(bottom, palette.clone(), false, false, 0.0, Some(&mapping));
                if last_fg != Some(fg) {
                    out.push_str(&truecolor_ansi(fg.r, fg.g, fg.b, true));
                    last_fg = Some(fg);
                }
                if last_bg != Some(bg) {
                    out.push_str(&truecolor_ansi(bg.r, bg.g, bg.b, false));
                    last_bg = Some(bg);
                }
                out.push('\u{2580}'); // ▀
            }
        }
    }

    out.push_str("\x1b[0m");
    out
}

/// Map one interior field cell (ASCII charset) to `(fg, glyph)`, pre-grid.
///
/// Extracted from [`render_ansi_cells`]'s ASCII branch so both it and
/// [`render_ansi_framed`] share a single definition of the per-cell mapping.
fn ascii_cell_fg_glyph(
    cell: &Cell,
    inv_gain: f32,
    palette: &Palette,
    mapping: &IntensityMapping,
) -> (RgbColor, char) {
    let top = (cell.top * inv_gain).clamp(0.0, 1.0);
    let bottom = (cell.bottom * inv_gain).clamp(0.0, 1.0);
    let brightness = (top + bottom) * 0.5;
    let fg = map_brightness_rgb(
        brightness,
        palette.clone(),
        false,
        false,
        0.0,
        Some(mapping),
    );
    let glyph = map_brightness(mapping.apply(brightness), None, Charset::Ascii);
    (fg, glyph)
}

/// Full-terminal frame geometry: an outer ring (glow, added in Task 2) around
/// an interior field of cells.
pub struct FrameGeometry {
    /// Full terminal columns.
    pub cols: usize,
    /// Full terminal rows.
    pub rows: usize,
    /// Glow ring thickness per side, in columns (0 = no ring).
    pub ring_cols: usize,
    /// Glow ring thickness per side, in rows (0 = no ring).
    pub ring_rows: usize,
}

impl FrameGeometry {
    /// Interior dimensions after subtracting the ring from both sides.
    pub fn interior(&self) -> (usize, usize) {
        (
            self.cols.saturating_sub(2 * self.ring_cols),
            self.rows.saturating_sub(2 * self.ring_rows),
        )
    }
}

/// Render a full terminal frame from interior-sized field cells, with an
/// optional grid overlay on the interior. The outer ring (glow, added in
/// Task 2) is emitted as blank space for now.
///
/// * `field_cells` — the FIELD downsampled to interior dims (`geom.interior()`).
/// * `grid` — pre-initialized to interior dims, or `None` for no grid.
#[allow(clippy::too_many_arguments)]
pub fn render_ansi_framed(
    field_cells: &[Cell],
    geom: &FrameGeometry,
    palette: Palette,
    charset: Charset,
    max_brightness: f32,
    grid: Option<&GridRenderer>,
    grid_color: RgbColor,
    grid_opacity: f32,
    glow_accent: Option<RgbColor>,
) -> String {
    debug_assert!(matches!(charset, Charset::Ascii), "info path is ASCII-only");
    let inv_gain = if max_brightness > 0.0 {
        1.0 / max_brightness
    } else {
        1.0
    };
    let mapping = IntensityMapping::logarithmic(10.0);
    let (iw, ih) = geom.interior();
    let mut out = String::with_capacity(geom.cols * geom.rows * 20 + geom.rows * 8);
    out.push_str("\x1b[H");
    for y in 0..geom.rows {
        out.push_str("\x1b[");
        out.push_str(&(y + 1).to_string());
        out.push_str(";1H");
        let mut last_fg: Option<RgbColor> = None;
        for x in 0..geom.cols {
            let in_ring = x < geom.ring_cols
                || x >= geom.cols.saturating_sub(geom.ring_cols)
                || y < geom.ring_rows
                || y >= geom.rows.saturating_sub(geom.ring_rows);
            if in_ring {
                if let Some(accent) = glow_accent {
                    let dc = x.min(geom.cols - 1 - x) as f32 / geom.ring_cols.max(1) as f32;
                    let dr = y.min(geom.rows - 1 - y) as f32 / geom.ring_rows.max(1) as f32;
                    let depth = dc.min(dr).clamp(0.0, 1.0);
                    let alpha = 1.0 - depth * 0.7;
                    let ch = if depth < 0.34 {
                        '\u{2588}'
                    } else if depth < 0.67 {
                        '\u{2593}'
                    } else {
                        '\u{2592}'
                    };
                    let fg = accent.with_alpha(alpha);
                    if last_fg != Some(fg) {
                        out.push_str(&truecolor_ansi(fg.r, fg.g, fg.b, true));
                        last_fg = Some(fg);
                    }
                    out.push(ch);
                } else {
                    out.push(' ');
                    last_fg = None;
                }
                continue;
            }
            let ix = x - geom.ring_cols;
            let iy = y - geom.ring_rows;
            let (mut fg, glyph) =
                ascii_cell_fg_glyph(&field_cells[iy * iw + ix], inv_gain, &palette, &mapping);
            if let Some(g) = grid {
                if g.is_grid_position(ix, iy, iw, ih) {
                    fg = g.blend_color(grid_color, fg, grid_opacity);
                }
            }
            if last_fg != Some(fg) {
                out.push_str(&truecolor_ansi(fg.r, fg.g, fg.b, true));
                last_fg = Some(fg);
            }
            out.push(glyph);
        }
    }
    out.push_str("\x1b[0m");
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::render::charset::Charset;
    use crate::render::grid::{GridRenderer, GridStyle};
    use crate::render::palette::{Palette, RgbColor};

    #[test]
    fn framed_grid_recolors_only_grid_columns() {
        // 6x1 interior, no ring. Uniform mid brightness so every field cell maps to
        // the same fg; a grid line must therefore be the ONLY color break in the row.
        let interior = vec![
            Cell {
                top: 0.5,
                bottom: 0.5,
                ..Default::default()
            };
            6
        ];
        let geom = FrameGeometry {
            cols: 6,
            rows: 1,
            ring_cols: 0,
            ring_rows: 0,
        };
        let mut grid = GridRenderer::new(
            GridStyle::Cross,
            3,
            RgbColor {
                r: 0x8f,
                g: 0x8f,
                b: 0x55,
            },
            0.35,
            false,
        );
        grid.initialize(6, 1); // interior dims
        let framed = render_ansi_framed(
            &interior,
            &geom,
            Palette::Warm,
            Charset::Ascii,
            1.0,
            Some(&grid),
            RgbColor {
                r: 0x8f,
                g: 0x8f,
                b: 0x55,
            },
            0.35,
            None,
        );
        let plain = render_ansi_framed(
            &interior,
            &geom,
            Palette::Warm,
            Charset::Ascii,
            1.0,
            None,
            RgbColor { r: 0, g: 0, b: 0 },
            0.0,
            None,
        );
        // grid at cols {2,4} for size 3 over width 6 → framed differs from plain, and
        // both are deterministic + nonempty.
        assert_ne!(framed, plain, "grid must recolor at least one cell");
        assert!(framed.contains('\x1b'));
        assert_eq!(
            framed,
            render_ansi_framed(
                &interior,
                &geom,
                Palette::Warm,
                Charset::Ascii,
                1.0,
                Some(&grid),
                RgbColor {
                    r: 0x8f,
                    g: 0x8f,
                    b: 0x55,
                },
                0.35,
                None,
            ),
            "deterministic"
        );
    }

    #[test]
    fn framed_glow_fills_ring_with_block_glyphs() {
        // 4x4 full frame, ring 1x1 → 2x2 interior. Corners are the outer (bright) band.
        let interior = vec![Cell::default(); 2 * 2];
        let geom = FrameGeometry {
            cols: 4,
            rows: 4,
            ring_cols: 1,
            ring_rows: 1,
        };
        let accent = RgbColor {
            r: 0xff,
            g: 0xcc,
            b: 0x66,
        };
        let framed = render_ansi_framed(
            &interior,
            &geom,
            Palette::Warm,
            Charset::Ascii,
            1.0,
            None,
            RgbColor { r: 0, g: 0, b: 0 },
            0.0,
            Some(accent),
        );
        // Outer ring cells use the full block; the frame must contain █.
        assert!(
            framed.contains('\u{2588}'),
            "glow ring must draw █ at the outer band"
        );
        // Deterministic.
        let again = render_ansi_framed(
            &interior,
            &geom,
            Palette::Warm,
            Charset::Ascii,
            1.0,
            None,
            RgbColor { r: 0, g: 0, b: 0 },
            0.0,
            Some(accent),
        );
        assert_eq!(framed, again);
    }

    #[test]
    fn render_ansi_is_deterministic_and_nonempty() {
        // A small fixed trail: 4x2 sim downsampled to a 2x1 grid.
        let trail = vec![0.0, 0.1, 0.5, 0.9, 0.2, 0.3, 0.8, 1.0];
        let a = render_ansi(&trail, 4, 2, 2, 1, Palette::Warm, Charset::Ascii, 1.0);
        let b = render_ansi(&trail, 4, 2, 2, 1, Palette::Warm, Charset::Ascii, 1.0);
        assert_eq!(a, b, "same inputs must produce identical output");
        assert!(
            !a.is_empty(),
            "output must contain escape sequences + glyphs"
        );
        assert!(
            a.contains('\x1b'),
            "truecolor ANSI must include ESC sequences"
        );
    }
}
