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
use crate::render::palette::{map_brightness_rgb, truecolor_ansi, IntensityMapping, Palette};

/// Render a trail map to a truecolor ANSI frame.
///
/// * `trail` — row-major simulation trail map (`sim_width * sim_height`).
/// * `cols` / `rows` — target terminal grid. In half-block mode each row holds
///   two vertical subpixels (effective vertical resolution `rows * 2`).
/// * `gain` — white-point divisor; raw trail values are divided by this before
///   the palette/charset lookup (higher = darker).
#[allow(clippy::too_many_arguments)]
pub fn render_ansi(
    trail: &[f32],
    sim_width: usize,
    sim_height: usize,
    cols: usize,
    rows: usize,
    palette: Palette,
    charset: Charset,
    gain: f32,
) -> String {
    let mut frame = DownsampledFrame::new(cols, rows);
    downsample(trail, sim_width, sim_height, cols, rows, &mut frame);
    render_ansi_cells(frame.cells(), cols, rows, palette, charset, gain)
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
    gain: f32,
) -> String {
    let inv_gain = if gain > 0.0 { 1.0 / gain } else { 1.0 };
    let ascii = matches!(charset, Charset::Ascii);
    // Default render tone curve (matches RenderArtDefaults): a log curve that
    // lifts dim trail values so the network reads at low brightness.
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
                let brightness = (top + bottom) * 0.5;
                let fg = map_brightness_rgb(
                    brightness,
                    palette.clone(),
                    false,
                    false,
                    0.0,
                    Some(&mapping),
                );
                if last_fg != Some(fg) {
                    out.push_str(&truecolor_ansi(fg.r, fg.g, fg.b, true));
                    last_fg = Some(fg);
                }
                out.push(map_brightness(
                    mapping.apply(brightness),
                    None,
                    Charset::Ascii,
                ));
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::render::charset::Charset;
    use crate::render::palette::Palette;

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
