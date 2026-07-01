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

/// Full-terminal frame geometry. Three concentric zones per side, from the
/// terminal edge inward: an **outer padding** band (`pad_*`, dark/blank — the
/// grid shows through it), a **glow border** band (`ring_*`, drawn as a glow
/// when a glow accent is supplied to [`render_ansi_framed`]), and the interior
/// field. `pad_*` defaults to 0, collapsing this to the legacy ring+interior
/// layout.
pub struct FrameGeometry {
    /// Full terminal columns.
    pub cols: usize,
    /// Full terminal rows.
    pub rows: usize,
    /// Glow border thickness per side, in columns (0 = no border).
    pub ring_cols: usize,
    /// Glow border thickness per side, in rows (0 = no border).
    pub ring_rows: usize,
    /// Outer dark-padding thickness per side, in columns (0 = none). Lies
    /// *outside* the glow border; the full-terminal grid shows through it.
    pub pad_cols: usize,
    /// Outer dark-padding thickness per side, in rows. See [`Self::pad_cols`].
    pub pad_rows: usize,
}

impl FrameGeometry {
    /// Total inset per side (outer padding + glow border), in (cols, rows).
    fn inset(&self) -> (usize, usize) {
        (
            self.pad_cols + self.ring_cols,
            self.pad_rows + self.ring_rows,
        )
    }

    /// Interior dimensions after subtracting padding + border from both sides.
    pub fn interior(&self) -> (usize, usize) {
        let (ic, ir) = self.inset();
        (
            self.cols.saturating_sub(2 * ic),
            self.rows.saturating_sub(2 * ir),
        )
    }
}

/// Render a full terminal frame from interior-sized field cells, with an
/// optional grid overlay. The glow border is drawn when `glow_accent` is
/// `Some(_)`, or emitted as blank space when `None`; any outer padding
/// ([`FrameGeometry::pad_cols`]/`pad_rows`) is blank.
///
/// * `field_cells` — the FIELD downsampled to interior dims (`geom.interior()`).
/// * `grid` — pre-initialized to interior dims (legacy path) or to the FULL
///   terminal dims (`geom.cols`×`geom.rows`) when `grid_on_empty` is set, so the
///   grid spans the outer padding as well. `None` for no grid.
/// * `grid_on_empty` — **opt-in**; when `true`, grid lines are drawn on *empty*
///   (or near-black) cells across the WHOLE terminal — including the outer
///   padding, giving a constant grid band around the frame — by substituting a
///   box-drawing glyph (`┼`/`│`/`─`) in a dimmed grid color, like the native
///   live TUI (`FrameBuffer::render_grid_background`). Lit / glow-border cells
///   are left untouched so content takes precedence. When `false` (default) the
///   grid is a foreground-only recolor of every *interior* grid cell — the
///   legacy behavior, kept byte-identical for existing callers.
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
    grid_on_empty: bool,
) -> String {
    debug_assert!(matches!(charset, Charset::Ascii), "info path is ASCII-only");
    let inv_gain = if max_brightness > 0.0 {
        1.0 / max_brightness
    } else {
        1.0
    };
    let mapping = IntensityMapping::logarithmic(10.0);
    let (iw, ih) = geom.interior();
    let (inset_c, inset_r) = geom.inset();
    // Dimmed grid color (grid_color scaled by opacity), matching the native
    // `render_grid_background` blend against a dark background.
    let grid_dim = RgbColor {
        r: (grid_color.r as f32 * grid_opacity) as u8,
        g: (grid_color.g as f32 * grid_opacity) as u8,
        b: (grid_color.b as f32 * grid_opacity) as u8,
    };
    let mut out = String::with_capacity(geom.cols * geom.rows * 20 + geom.rows * 8);
    out.push_str("\x1b[H");
    for y in 0..geom.rows {
        out.push_str("\x1b[");
        out.push_str(&(y + 1).to_string());
        out.push_str(";1H");
        let mut last_fg: Option<RgbColor> = None;
        for x in 0..geom.cols {
            // Zone classification. Padding is the band outside the glow border;
            // the glow border is the band between padding and interior.
            let in_pad = x < geom.pad_cols
                || x >= geom.cols.saturating_sub(geom.pad_cols)
                || y < geom.pad_rows
                || y >= geom.rows.saturating_sub(geom.pad_rows);
            let in_inset = x < inset_c
                || x >= geom.cols.saturating_sub(inset_c)
                || y < inset_r
                || y >= geom.rows.saturating_sub(inset_r);
            let in_glow = in_inset && !in_pad;

            // Resolve the base cell (before the grid overlay): a glyph plus an
            // optional foreground. `None` fg = a blank/dark cell (no SGR).
            let (mut fg, mut glyph): (Option<RgbColor>, char) = if in_pad {
                (None, ' ')
            } else if in_glow {
                if let Some(accent) = glow_accent {
                    // Depth into the glow band from its outer edge (the padding
                    // boundary). Reduces to the legacy formula when pad == 0.
                    let dc = (x - geom.pad_cols).min(
                        (geom.cols - 1)
                            .saturating_sub(geom.pad_cols)
                            .saturating_sub(x),
                    ) as f32
                        / geom.ring_cols.max(1) as f32;
                    let dr = (y - geom.pad_rows).min(
                        (geom.rows - 1)
                            .saturating_sub(geom.pad_rows)
                            .saturating_sub(y),
                    ) as f32
                        / geom.ring_rows.max(1) as f32;
                    let depth = dc.min(dr).clamp(0.0, 1.0);
                    let alpha = 1.0 - depth * 0.7;
                    let ch = if depth < 0.34 {
                        '\u{2588}'
                    } else if depth < 0.67 {
                        '\u{2593}'
                    } else {
                        '\u{2592}'
                    };
                    (Some(accent.with_alpha(alpha)), ch)
                } else {
                    (None, ' ')
                }
            } else {
                let ix = x - inset_c;
                let iy = y - inset_r;
                let (cfg, cglyph) =
                    ascii_cell_fg_glyph(&field_cells[iy * iw + ix], inv_gain, &palette, &mapping);
                (Some(cfg), cglyph)
            };

            if let Some(g) = grid {
                if grid_on_empty {
                    // Full-terminal grid on empty / near-black cells (padding +
                    // sparse field); glow-border cells are opaque and skipped.
                    let is_empty = glyph == ' '
                        || fg.map_or(true, |c| (c.r as u32 + c.g as u32 + c.b as u32) < 30);
                    if is_empty && g.is_grid_position(x, y, geom.cols, geom.rows) {
                        let (on_v, on_h) = g.get_grid_lines(x, y);
                        glyph = match (on_v, on_h) {
                            (true, true) => '\u{253c}',  // ┼
                            (true, false) => '\u{2502}', // │
                            (false, true) => '\u{2500}', // ─
                            (false, false) => glyph,
                        };
                        if glyph != ' ' {
                            fg = Some(grid_dim);
                        }
                    }
                } else if !in_inset {
                    // Legacy: foreground-only recolor of every interior grid cell.
                    if g.is_grid_position(x - inset_c, y - inset_r, iw, ih) {
                        if let Some(cell_fg) = fg {
                            fg = Some(g.blend_color(grid_color, cell_fg, grid_opacity));
                        }
                    }
                }
            }

            match fg {
                Some(c) => {
                    if last_fg != Some(c) {
                        out.push_str(&truecolor_ansi(c.r, c.g, c.b, true));
                        last_fg = Some(c);
                    }
                }
                None => last_fg = None,
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
            pad_cols: 0,
            pad_rows: 0,
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
            false,
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
            false,
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
                false,
            ),
            "deterministic"
        );
    }

    #[test]
    fn grid_on_empty_draws_box_glyphs_on_empty_cells() {
        // 6x1 interior, no ring, all-EMPTY cells (brightness 0 → space glyph).
        // Cross grid size 3 → vertical lines at interior cols {2,4}.
        let interior = vec![Cell::default(); 6];
        let geom = FrameGeometry {
            cols: 6,
            rows: 1,
            ring_cols: 0,
            ring_rows: 0,
            pad_cols: 0,
            pad_rows: 0,
        };
        let color = RgbColor {
            r: 0x8f,
            g: 0x8f,
            b: 0x55,
        };
        let mut grid = GridRenderer::new(GridStyle::Cross, 3, color, 0.35, false);
        grid.initialize(6, 1);

        // Legacy (fg-only) grid over an all-empty field: every glyph is a space,
        // so no visible line — no box-drawing glyph is emitted.
        let legacy = render_ansi_framed(
            &interior,
            &geom,
            Palette::Warm,
            Charset::Ascii,
            1.0,
            Some(&grid),
            color,
            0.35,
            None,
            false,
        );
        assert!(
            !legacy.contains('\u{2502}'),
            "legacy path must not draw box-drawing grid glyphs"
        );

        // Opt-in: empty grid columns become a vertical box-drawing glyph.
        let on_empty = render_ansi_framed(
            &interior,
            &geom,
            Palette::Warm,
            Charset::Ascii,
            1.0,
            Some(&grid),
            color,
            0.35,
            None,
            true,
        );
        assert!(
            on_empty.contains('\u{2502}'),
            "grid-on-empty must draw │ on empty grid columns"
        );
        assert_ne!(legacy, on_empty, "opt-in must change the frame");
    }

    #[test]
    fn grid_on_empty_spans_outer_padding() {
        // 12x1, no glow ring, 3-col outer padding → 6-col interior. Grid size 6
        // over the full terminal → vertical lines at cols {2,10}, both inside the
        // outer padding band → a box glyph must be drawn OUTSIDE the field.
        let interior = vec![Cell::default(); 6];
        let geom = FrameGeometry {
            cols: 12,
            rows: 1,
            ring_cols: 0,
            ring_rows: 0,
            pad_cols: 3,
            pad_rows: 0,
        };
        let color = RgbColor {
            r: 0x8f,
            g: 0x8f,
            b: 0x55,
        };
        let mut grid = GridRenderer::new(GridStyle::Cross, 6, color, 0.35, false);
        grid.initialize(12, 1); // FULL terminal dims for the grid_on_empty path
        let out = render_ansi_framed(
            &interior,
            &geom,
            Palette::Warm,
            Charset::Ascii,
            1.0,
            Some(&grid),
            color,
            0.35,
            None,
            true,
        );
        assert!(
            out.contains('\u{2502}'),
            "grid must draw │ across the outer padding band"
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
            pad_cols: 0,
            pad_rows: 0,
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
            false,
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
            false,
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
