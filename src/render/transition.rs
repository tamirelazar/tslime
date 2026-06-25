//! Preset-switch transition overlays.
//!
//! When the user switches presets at runtime the app can announce it three ways
//! (user-selectable via `--transition`):
//!
//! - [`TransitionStyle::Toast`] — the default ambient notification ("Applied
//!   preset: X"); handled by the existing notification path, not here.
//! - [`TransitionStyle::Figlet`] — the preset name in big block letters, centered,
//!   fading up then out.
//! - [`TransitionStyle::Type`] — a letter-spaced "instrument readout" typed
//!   character-by-character over a dimmed band, with a blinking caret and an
//!   accent underline.
//!
//! Figlet/Type are drawn directly onto the [`FrameBuffer`] after the sim frame so
//! the dimmed band can read the field colors beneath it. An optional one-line
//! tagline (off by default) renders beneath the name.

use crate::render::palette::{map_brightness_rgb, IntensityMapping, Palette, RgbColor};
use crate::simulation::config::TransitionStyle;
use crate::terminal::frame_buffer::{Cell, FrameBuffer};

/// Total duration of a figlet/type transition, in seconds.
pub const TRANSITION_SECS: f32 = 1.8;

const INK_DARK: RgbColor = RgbColor { r: 6, g: 8, b: 10 };
/// Near-white the typed readout is lifted toward, for legibility on warm palettes.
const PAPER: RgbColor = RgbColor {
    r: 244,
    g: 246,
    b: 250,
};
const SHADE: [char; 6] = [' ', '·', '░', '▒', '▓', '█'];

/// A live transition to draw this frame. `elapsed` is seconds since the switch.
pub struct TransitionView {
    /// Which announcement style to render (never `Toast` here).
    pub style: TransitionStyle,
    /// Preset name to display.
    pub name: String,
    /// Optional tagline shown beneath the name.
    pub tagline: Option<String>,
    /// Seconds elapsed since the switch (drives the animation).
    pub elapsed: f32,
}

/// Palette context needed to color the transition the same way the sim is colored.
#[derive(Clone)]
pub struct PaletteCtx<'a> {
    /// Active palette.
    pub palette: Palette,
    /// Whether the palette ramp is reversed.
    pub reverse: bool,
    /// Whether the palette is inverted.
    pub invert: bool,
    /// Hue-shift applied to the palette, in degrees.
    pub hue_shift: f32,
    /// Optional intensity (tone) mapping applied before the gradient lookup.
    pub mapping: Option<&'a IntensityMapping>,
}

// ── timing ────────────────────────────────────────────────────────────────────
/// Smooth 0→1→0 envelope over the transition: quick rise, hold, gentle fall.
fn envelope(t: f32) -> f32 {
    let p = (t / TRANSITION_SECS).clamp(0.0, 1.0);
    if p < 0.18 {
        (p / 0.18).powf(0.7)
    } else if p < 0.72 {
        1.0
    } else {
        (1.0 - (p - 0.72) / 0.28).clamp(0.0, 1.0)
    }
}
fn ease_out(p: f32) -> f32 {
    let p = p.clamp(0.0, 1.0);
    1.0 - (1.0 - p) * (1.0 - p)
}

// ── color helpers ───────────────────────────────────────────────────────────
fn lerp_u8(a: u8, b: u8, t: f32) -> u8 {
    (a as f32 + (b as f32 - a as f32) * t)
        .round()
        .clamp(0.0, 255.0) as u8
}
fn lerp_rgb(a: RgbColor, b: RgbColor, t: f32) -> RgbColor {
    RgbColor {
        r: lerp_u8(a.r, b.r, t),
        g: lerp_u8(a.g, b.g, t),
        b: lerp_u8(a.b, b.b, t),
    }
}
fn sample(ctx: &PaletteCtx, brightness: f32) -> RgbColor {
    map_brightness_rgb(
        brightness.clamp(0.0, 1.0),
        ctx.palette.clone(),
        ctx.reverse,
        ctx.invert,
        ctx.hue_shift,
        ctx.mapping,
    )
}

// ── frame-buffer cell writes (set both RGB + 256 so either color mode renders) ─
fn put_cell(buf: &mut FrameBuffer, x: usize, y: usize, ch: char, fg: RgbColor) {
    let mut cell = Cell::new(ch);
    cell.fg_color_rgb = Some(fg);
    cell.fg_color_256 = Some(crate::render::palette::rgb_to_256(fg));
    buf.set_cell(x, y, cell);
}
fn put_str(buf: &mut FrameBuffer, x: usize, y: usize, s: &str, fg: RgbColor) {
    for (i, ch) in s.chars().enumerate() {
        put_cell(buf, x + i, y, ch, fg);
    }
}
/// Darken one cell's foreground toward ink (keeping its glyph), so text drawn
/// on top reads over any background — bright dense fields included.
fn dim_cell(buf: &mut FrameBuffer, x: usize, y: usize, amt: f32) {
    if x >= buf.width || y * buf.width + x >= buf.cells.len() {
        return;
    }
    let cur = read_fg(buf, x, y);
    let dimmed = lerp_rgb(cur, INK_DARK, amt.clamp(0.0, 1.0));
    let ch = buf.cells[y * buf.width + x].char;
    put_cell(buf, x, y, ch, dimmed);
}
/// Read a cell's current foreground RGB (best-effort; ink-dark if unknown).
fn read_fg(buf: &FrameBuffer, x: usize, y: usize) -> RgbColor {
    let idx = y * buf.width + x;
    buf.cells
        .get(idx)
        .and_then(|c| c.fg_color_rgb)
        .unwrap_or(INK_DARK)
}

// ── 4×6 block font (pixels rendered 2 cells wide → near-square glyphs) ────────
fn glyph(ch: char) -> [&'static str; 6] {
    match ch.to_ascii_uppercase() {
        'A' => [".##.", "#..#", "#..#", "####", "#..#", "#..#"],
        'B' => ["###.", "#..#", "###.", "#..#", "#..#", "###."],
        'C' => [".###", "#...", "#...", "#...", "#...", ".###"],
        'D' => ["###.", "#..#", "#..#", "#..#", "#..#", "###."],
        'E' => ["####", "#...", "###.", "#...", "#...", "####"],
        'F' => ["####", "#...", "###.", "#...", "#...", "#..."],
        'G' => [".###", "#...", "#...", "#.##", "#..#", ".###"],
        'H' => ["#..#", "#..#", "####", "#..#", "#..#", "#..#"],
        'I' => ["####", ".##.", ".##.", ".##.", ".##.", "####"],
        'J' => ["####", "...#", "...#", "...#", "#..#", ".##."],
        'K' => ["#..#", "#.#.", "##..", "##..", "#.#.", "#..#"],
        'L' => ["#...", "#...", "#...", "#...", "#...", "####"],
        'M' => ["#..#", "####", "####", "#..#", "#..#", "#..#"],
        'N' => ["#..#", "##.#", "##.#", "#.##", "#.##", "#..#"],
        'O' => [".##.", "#..#", "#..#", "#..#", "#..#", ".##."],
        'P' => ["###.", "#..#", "###.", "#...", "#...", "#..."],
        'Q' => [".##.", "#..#", "#..#", "#..#", "#.#.", ".###"],
        'R' => ["###.", "#..#", "###.", "##..", "#.#.", "#..#"],
        'S' => [".###", "#...", ".##.", "...#", "...#", "###."],
        'T' => ["####", ".##.", ".##.", ".##.", ".##.", ".##."],
        'U' => ["#..#", "#..#", "#..#", "#..#", "#..#", ".##."],
        'V' => ["#..#", "#..#", "#..#", "#..#", ".##.", ".##."],
        'W' => ["#..#", "#..#", "#..#", "####", "####", "#..#"],
        'X' => ["#..#", "#..#", ".##.", ".##.", "#..#", "#..#"],
        'Y' => ["#..#", "#..#", ".##.", ".##.", ".##.", ".##."],
        'Z' => ["####", "...#", ".##.", "##..", "#...", "####"],
        '-' => ["....", "....", "####", "####", "....", "...."],
        _ => ["....", "....", "....", "....", "....", "...."], // space + unknown
    }
}

const GLYPH_W: usize = 4;
const GLYPH_H: usize = 6;

/// Rendered width of `name` when each "on" pixel is `px_w` cells wide
/// (glyphs separated by a 1-cell gap).
fn figlet_width(name: &str, px_w: usize) -> usize {
    let n = name.chars().count();
    n * (GLYPH_W * px_w) + n.saturating_sub(1)
}

/// Pixel width that keeps `name` within `avail` cells: 2 cells when it fits
/// (near-square glyphs), else 1 (long names on narrow terminals).
fn fit_px_w(name: &str, avail: usize) -> usize {
    if figlet_width(name, 2) <= avail {
        2
    } else {
        1
    }
}

/// Draw the active transition onto the frame buffer. No-op for `Toast` or once
/// the transition has fully elapsed.
///
/// `content_x` is the inclusive-start/exclusive-end column range of the area
/// *inside* the window frame border; full-width effects (the TYPE dim band) clip
/// to it so they stop at the frame instead of bleeding over the matte/border.
pub fn draw_transition(
    buf: &mut FrameBuffer,
    view: &TransitionView,
    width: usize,
    height: usize,
    content_x: (usize, usize),
    ctx: &PaletteCtx,
) {
    if view.elapsed >= TRANSITION_SECS || width == 0 || height == 0 {
        return;
    }
    let name = view.name.to_uppercase();
    match view.style {
        TransitionStyle::Toast => {}
        TransitionStyle::Figlet => draw_figlet(
            buf,
            &name,
            view.tagline.as_deref(),
            width,
            height,
            view.elapsed,
            ctx,
        ),
        TransitionStyle::Type => draw_type(
            buf,
            &name,
            view.tagline.as_deref(),
            width,
            height,
            content_x,
            view.elapsed,
            ctx,
        ),
    }
}

// ── FIGLET ────────────────────────────────────────────────────────────────────
fn draw_figlet(
    buf: &mut FrameBuffer,
    name: &str,
    tagline: Option<&str>,
    width: usize,
    height: usize,
    t: f32,
    ctx: &PaletteCtx,
) {
    let alpha = envelope(t);
    if alpha <= 0.0 {
        return;
    }
    let px_w = fit_px_w(name, width);
    let adv = GLYPH_W * px_w + 1; // per-letter advance incl. 1-cell gap
    let fw = figlet_width(name, px_w);
    let block_h = if tagline.is_some() {
        GLYPH_H + 2
    } else {
        GLYPH_H
    };
    let x0 = width.saturating_sub(fw) / 2;
    let y0 = height.saturating_sub(block_h) / 2;

    // Scrim: darken the glyph-block bounding box (with a little padding) before
    // drawing letters, so the name reads over ANY field — including the dense,
    // bright presets where un-backed glyphs vanish.
    let pad_x = px_w;
    let sx0 = x0.saturating_sub(pad_x);
    let sx1 = (x0 + fw + pad_x).min(width);
    let sy0 = y0.saturating_sub(1);
    let sy1 = (y0 + block_h + 1).min(height);
    let scrim = 0.82 * alpha;
    for yy in sy0..sy1 {
        for xx in sx0..sx1 {
            dim_cell(buf, xx, yy, scrim);
        }
    }

    for (gi, ch) in name.chars().enumerate() {
        let rows = glyph(ch);
        let gx = x0 + gi * adv;
        for (ry, rowstr) in rows.iter().enumerate() {
            let yy = y0 + ry;
            if yy >= height {
                continue;
            }
            let grad = ry as f32 / (GLYPH_H as f32 - 1.0);
            let on = lerp_rgb(sample(ctx, 0.55), sample(ctx, 1.0), grad);
            for (cx, pix) in rowstr.chars().enumerate() {
                if pix != '#' {
                    continue;
                }
                for dx in 0..px_w {
                    let cc = gx + cx * px_w + dx;
                    if cc >= width {
                        continue;
                    }
                    let shown = lerp_rgb(read_fg(buf, cc, yy), on, alpha);
                    let g = SHADE[((alpha * 5.0).round() as usize).min(5)];
                    put_cell(buf, cc, yy, g, shown);
                }
            }
        }
    }
    if let Some(tag) = tagline {
        let tw = tag.chars().count();
        let tx = width.saturating_sub(tw) / 2;
        let ty = y0 + GLYPH_H + 1;
        if ty < height {
            let tcol = lerp_rgb(INK_DARK, sample(ctx, 0.7), alpha);
            put_str(buf, tx, ty, tag, tcol);
        }
    }
}

// ── TYPE ────────────────────────────────────────────────────────────────────
#[allow(clippy::too_many_arguments)]
fn draw_type(
    buf: &mut FrameBuffer,
    name: &str,
    tagline: Option<&str>,
    width: usize,
    height: usize,
    content_x: (usize, usize),
    t: f32,
    ctx: &PaletteCtx,
) {
    let p = (t / TRANSITION_SECS).clamp(0.0, 1.0);
    let n = name.chars().count();
    let typed = ((p / 0.45) * n as f32).floor().clamp(0.0, n as f32) as usize;
    let fade = if p < 0.85 {
        1.0
    } else {
        ease_out(1.0 - (p - 0.85) / 0.15)
    };
    if fade <= 0.0 {
        return;
    }
    let y = height / 2;

    // 1) dim a band behind the readout so glyphs read over the field. Clip to
    //    the frame interior so the band stops at the border instead of dimming
    //    the matte/border cells. The band ends at the underline when there's no
    //    tagline (avoids a dead empty row below) and extends to the tagline row
    //    when one is shown.
    let r0 = y.saturating_sub(1);
    let band_bottom = if tagline.is_some() { y + 3 } else { y + 1 };
    let r1 = band_bottom.min(height.saturating_sub(1));
    let (bx0, bx1) = (content_x.0.min(width), content_x.1.min(width));
    let amt = 0.86 * fade;
    for ry in r0..=r1 {
        for cx in bx0..bx1 {
            dim_cell(buf, cx, ry, amt);
        }
    }

    // 2) name, letter-spaced, with a blinking caret while typing
    let shown: String = name.chars().take(typed).collect();
    let spaced: String = shown
        .chars()
        .flat_map(|c| [c, ' '])
        .collect::<String>()
        .trim_end()
        .to_string();
    let caret = (t * 3.0).sin() > 0.0 && p < 0.6;
    let line = if caret {
        format!("{} \u{2588}", spaced)
    } else {
        spaced
    };
    let full_w = n * 2;
    let x = width.saturating_sub(full_w) / 2;
    // Lift the typed ink toward white so it stays legible on warm/low-luminance
    // palettes (organic orange, vinescii green) — keep the palette hue, raise
    // brightness. The accent underline below carries the pure palette color.
    let ink = lerp_rgb(sample(ctx, 1.0), PAPER, 0.55);
    let col = lerp_rgb(INK_DARK, ink, fade);
    if y < height {
        put_str(buf, x, y, &line, col);
    }

    // 3) accent underline + optional tagline once the name finishes typing
    if typed >= n {
        let rule_w = (n * 2).saturating_sub(1).min(width);
        let ra = ((p - 0.45) / 0.2).clamp(0.0, 1.0) * fade;
        let rcol = lerp_rgb(INK_DARK, sample(ctx, 0.85), ra);
        if y + 1 < height {
            for i in 0..rule_w {
                if x + i < width {
                    put_cell(buf, x + i, y + 1, '─', rcol);
                }
            }
        }
        if let Some(tag) = tagline {
            let ta = ((p - 0.45) / 0.25).clamp(0.0, 1.0) * fade;
            let tw = tag.chars().count();
            let tx = width.saturating_sub(tw) / 2;
            if y + 3 < height {
                let tcol = lerp_rgb(INK_DARK, sample(ctx, 0.7), ta);
                put_str(buf, tx, y + 3, tag, tcol);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn style_parses_round_trips() {
        for s in ["toast", "figlet", "type"] {
            let p: TransitionStyle = s.parse().unwrap();
            assert_eq!(p.to_string(), s);
        }
        assert!("nope".parse::<TransitionStyle>().is_err());
    }

    #[test]
    fn figlet_width_grows_with_length() {
        assert!(figlet_width("AB", 2) > figlet_width("A", 2));
        assert_eq!(figlet_width("A", 2), GLYPH_W * 2);
    }

    #[test]
    fn fit_px_w_shrinks_long_names() {
        assert_eq!(fit_px_w("ABC", 200), 2);
        assert_eq!(fit_px_w("CONSTELLATIONS", 100), 1);
    }

    #[test]
    fn default_is_toast() {
        assert_eq!(TransitionStyle::default(), TransitionStyle::Toast);
    }
}
