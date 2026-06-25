//! Field-plate specimen art for the Console detail pane: a field-notebook
//! *engraving* of each parameter, plus a portable glyph-profile system and a
//! material-aware renderer.
//!
//! The house style is **stipple + ASCII line-art** — every parameter is its own
//! living specimen. There are NO shade-blocks (`░▒▓█`): they render as a solid
//! fill on a meaningful share of terminals, collapsing every gradient into a
//! green rectangle. Tone comes from **stipple density** (`· : * ●`) instead,
//! which renders identically everywhere.
//!
//! ## Three materials (the one law)
//!
//! Every cell is one of exactly three materials, inferred from its *authored*
//! glyph (before any [`Profile`] downgrade):
//!
//! | Material        | What it is                         | Glyphs                                   | Color                     |
//! |-----------------|------------------------------------|------------------------------------------|---------------------------|
//! | **ORGANISM**    | the living thing (body, trail…)    | stipple `· : * ●` + line-art `( ) / \ - _ . , ' \u{60} ~` | active palette ramp       |
//! | **MEASUREMENT** | anything imposed *on* the specimen | wrapped in `«…»`: rulers, arrows, value  | `accent_active` (amber)   |
//! | **ANNOTATION**  | labels & captions                  | letters / digits / `°`                   | `text_secondary` (grey)   |
//!
//! Mnemonic: *green is alive, amber is measured, grey is named.* The organism
//! owns the palette, so the pane re-dyes the instant the user changes palette;
//! amber and grey never carry palette color.

use crate::render::controls::registry::ParamId;
use crate::render::palette::{map_brightness_rgb, Palette, RgbColor};
use crate::render::theme::PanelStyle;
use crate::render::widgets::RowBuf;

/// Glyph-rendering profile. Authored art is downgraded through the active one.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum Profile {
    /// Final glyphs (`● ‹ ›` etc.) drawn as-authored. Prettier; capability-gated.
    Fancy,
    /// ASCII + box-drawing only. Cell-perfect everywhere. Ships.
    #[default]
    Safe,
    /// Pure ASCII (no box-drawing). Ultimate fallback.
    Ascii,
}

/// Downgrade one authored glyph for the active profile, preserving width 1:1.
///
/// The stipple kit is almost entirely cell-perfect ASCII already; only the
/// nucleus `●`, the dense dot `·`, the measurement arrows/marks, and (in
/// `Ascii`) the box-drawing rulers need remapping.
pub fn map_glyph(ch: char, p: Profile) -> char {
    match p {
        Profile::Fancy => ch,
        Profile::Safe => match ch {
            '●' => '@',
            '·' => '.',
            '→' | '▸' => '>',
            '◂' | '‹' => '<',
            '›' => '>',
            '▲' | '▴' => '^',
            '═' => '=',
            '−' => '-',
            '╴' => '_', // caret underline
            // box-drawing rulers ─ │ ├ ┤ pass through (cell-perfect in modern emulators);
            // ASCII line-art ( ) / \ - _ . , ' ` ~ : * and `°` pass through too.
            other => other,
        },
        Profile::Ascii => match ch {
            '●' => '@',
            '·' => '.',
            '→' | '▸' => '>',
            '◂' | '‹' => '<',
            '›' => '>',
            '▲' | '▴' => '^',
            '═' => '=',
            '−' => '-',
            '╴' => '_',
            '°' => ' ',
            '│' => '|',
            '─' => '-',
            '├' | '┤' => '+',
            other => other,
        },
    }
}

/// True if `ch` is in a range every terminal renders single-width and draws
/// itself: ASCII `0x20–7E`, the degree sign `°`, box-drawing `U+2500–257F`,
/// block `U+2580–259F`, and Braille `U+2800–28FF` (the sim's own charset).
#[allow(dead_code)]
pub(crate) fn is_safe_width1(ch: char) -> bool {
    let u = ch as u32;
    u == 0x00B0
        || (0x20..=0x7E).contains(&u)
        || (0x2500..=0x257F).contains(&u)
        || (0x2580..=0x259F).contains(&u)
        || (0x2800..=0x28FF).contains(&u)
}

/// ORGANISM brightness for a glyph (`None` = not an organism glyph). Drives the
/// palette tint. This is the stipple ramp doubling as the brightness ramp:
///
/// ```text
/// glyph    ·,.    :'`    ~     ()/\-_   *     ●
/// bright   0.16   0.42   0.50  0.55     0.72  1.00
/// ```
///
/// Stipple `· : * ●` is the tone scale; the ASCII line-art (`( ) / \ - _`, the
/// rounded corners `. , ' \u{60}`, the cilium `~`) reads as faint-green membrane.
pub(crate) fn mass_brightness(ch: char) -> Option<f32> {
    Some(match ch {
        '·' | ',' | '.' => 0.16,
        ':' | '\'' | '`' => 0.42,
        '~' => 0.50,
        '(' | ')' | '/' | '\\' | '-' | '_' => 0.55,
        '*' => 0.72,
        '●' => 1.0,
        _ => return None,
    })
}

/// Resolve a `⟦name⟧` palette tag to a `Palette` (PALETTE strip swatches).
pub(crate) fn parse_palette_opt(name: &str) -> Option<Palette> {
    Some(match name.to_ascii_lowercase().as_str() {
        "slime" => Palette::Slime,
        "organic" => Palette::Organic,
        "heat" => Palette::Heat,
        "ocean" => Palette::Ocean,
        "mold" => Palette::Mold,
        "cosmic" => Palette::Cosmic,
        "ethereal" => Palette::Ethereal,
        "forest" => Palette::Forest,
        "neon" => Palette::Neon,
        _ => return None,
    })
}

/// Lowest brightness an organism cell renders at, so the faintest stipple still
/// reads against the panel bg. The [`mass_brightness`] ramp is remapped into
/// `[ORGANISM_FLOOR, 1.0]` at paint time (the semantic ramp itself is unchanged).
const ORGANISM_FLOOR: f32 = 0.40;

/// Color one art cell by its AUTHORED glyph (the three-material contract),
/// independent of the later glyph downgrade. Applied per cell in inference order:
///
/// 1. `measure` (inside a `«…»` run) → MEASUREMENT, `accent_active` (amber).
/// 2. `under_hand` (a matched live option) → IGNITION, `accent_ignite` (warm).
/// 3. organism glyph → palette-tinted by [`mass_brightness`] (`pal_override`
///    swaps the palette for PALETTE-strip `⟦name⟧` swatches).
/// 4. otherwise (letter / digit / `°` / apparatus punctuation) → ANNOTATION grey.
fn cell_color(
    ch: char,
    measure: bool,
    under_hand: bool,
    pal_override: Option<Palette>,
    st: &PanelStyle,
    base_palette: Palette,
) -> Option<RgbColor> {
    if ch == ' ' {
        return None;
    }
    if measure {
        // MEASUREMENT wins over everything (a labeled dimension line is all amber).
        return Some(st.accent_active);
    }
    if under_hand {
        // The single element under the user's hand glows warm.
        return Some(st.accent_ignite);
    }
    if let Some(b) = mass_brightness(ch) {
        // Lift the ramp into a visible band: at full sim brightness the faintest
        // stipple (`·` = 0.16) maps to a near-black green that vanishes against the
        // panel bg, so the dot-gradient stops reading. Floor it while preserving the
        // tonal order so dense stipple still reads brighter than sparse.
        let p = pal_override.unwrap_or(base_palette);
        let display = ORGANISM_FLOOR + (1.0 - ORGANISM_FLOOR) * b;
        return Some(map_brightness_rgb(display, p, false, false, 0.0, None));
    }
    // ANNOTATION: stage labels, numerals, degree marks, apparatus punctuation.
    Some(st.text_secondary)
}

/// Returns (rendered row, `Some(span)` if this line lit an option run).
///
/// A `⟦caret⟧`-only line fills `╴` (→ `_` in Safe) across `caret_span` in
/// `accent_ignite`. A `⟦opt:tag⟧…⟦⟧` run is lit (IGNITION) when `live_select`
/// (lowercased) contains `tag`; unmatched options render as ANNOTATION.
#[allow(clippy::too_many_arguments)]
fn render_one(
    line: &str,
    off: usize,
    profile: Profile,
    st: &PanelStyle,
    base_palette: Palette,
    width: usize,
    live_select: Option<&str>,
    caret_span: Option<(usize, usize)>,
) -> (RowBuf, Option<(usize, usize)>) {
    let trimmed = line.trim();
    if trimmed == "⟦caret⟧" {
        let mut row = RowBuf::new(width);
        if let Some((s, e)) = caret_span {
            for c in s..e {
                row.put_cells(c, &[(map_glyph('╴', profile), st.accent_ignite)], None);
            }
        }
        return (row, None);
    }

    let mut row = RowBuf::new(width);
    let mut col = off;
    let mut measure = false; // inside a «…» run (MEASUREMENT, amber)
    let mut pal_override: Option<Palette> = None;
    let mut opt_lit = false; // inside a matched ⟦opt:..⟧ run (IGNITION)
    let mut lit_span: Option<(usize, usize)> = None;
    let mut lit_start = 0usize;
    let mut chars = line.chars().peekable();
    while let Some(ch) = chars.next() {
        match ch {
            '«' => {
                measure = true;
                continue;
            }
            '»' => {
                measure = false;
                continue;
            }
            '⟦' => {
                let mut name = String::new();
                for c2 in chars.by_ref() {
                    if c2 == '⟧' {
                        break;
                    }
                    name.push(c2);
                }
                if let Some(tag) = name.strip_prefix("opt:") {
                    let matched = live_select
                        .map(|v| v.to_ascii_lowercase().contains(&tag.to_ascii_lowercase()))
                        .unwrap_or(false);
                    opt_lit = matched;
                    if matched {
                        lit_start = col;
                    }
                } else if name.is_empty() {
                    // run close
                    if opt_lit {
                        lit_span = Some((lit_start, col));
                    }
                    opt_lit = false;
                    pal_override = None;
                } else if name == "caret" {
                    // handled above as a whole line; ignore inline
                } else {
                    pal_override = parse_palette_opt(&name);
                }
                continue;
            }
            _ => {}
        }
        let color = cell_color(
            ch,
            measure,
            opt_lit,
            pal_override.clone(),
            st,
            base_palette.clone(),
        );
        let glyph = map_glyph(ch, profile);
        if let Some(c) = color {
            row.put_cells(col, &[(glyph, c)], None);
        }
        col += 1;
    }
    (row, lit_span)
}

/// Build the detail-pane rows for a field plate, composed for breathing room:
///
/// ```text
/// row 0      blank        top pad — lifts the specimen off the title border
///                                   (and gives the [key] chip its own corner)
/// rows 1..   specimen     centered art
/// row  N     blank        figure / caption separator
/// rows N+1.. caption       reading text, indented to the specimen's left margin
/// ```
///
/// The caption shares the centered art block's left edge (`off`, clamped) rather
/// than hugging the pane divider, so figure and caption read as one composition.
pub fn render_art(
    art: &str,
    caption: &str,
    profile: Profile,
    st: &PanelStyle,
    base_palette: Palette,
    live_select: Option<&str>,
    width: usize,
) -> Vec<RowBuf> {
    let (ded, bw) = dedent_and_measure(art);
    let off = width.saturating_sub(bw) / 2;
    let mut rows: Vec<RowBuf> = Vec::new();
    // Top pad: a row of air below the chrome so the specimen never butts the border.
    rows.push(RowBuf::new(width));
    let mut caret_span: Option<(usize, usize)> = None;
    for line in ded.lines() {
        let (row, span) = render_one(
            line,
            off,
            profile,
            st,
            base_palette.clone(),
            width,
            live_select,
            caret_span,
        );
        if span.is_some() {
            caret_span = span;
        }
        rows.push(row);
    }
    if !caption.is_empty() {
        rows.push(RowBuf::new(width)); // figure / caption separator
        let cap_indent = off.clamp(CAPTION_GUTTER_MIN, CAPTION_GUTTER_MAX);
        for cap in caption.lines() {
            let mut r = RowBuf::new(width);
            r.put(cap_indent, cap, Some(st.text_secondary), None);
            rows.push(r);
        }
    }
    rows
}

/// Caption left margin: a small, consistent gutter so reading text lifts off the
/// pane divider without stealing width from the line (the pane is only 39 cols).
const CAPTION_GUTTER_MIN: usize = 2;
const CAPTION_GUTTER_MAX: usize = 3;

/// One parameter's specimen art + reading caption.
pub struct FieldPlate {
    /// Authored ASCII-art specimen; may contain `«»` and `⟦⟧` authoring markers.
    pub art: &'static str,
    /// Short reading text (1-2 lines) displayed below the art.
    pub caption: &'static str,
}

/// The authored field plate for `id`, or `None` for params not yet drawn (they
/// fall back to the kind-aware detail pane).
///
/// PROTOTYPE: only the two specimens under sign-off are authored
/// ([`ParamId::SensorAngle`], [`ParamId::Decay`]); the rest return `None` until
/// the visual direction is approved, then propagate against the design briefs.
pub fn field_plate(id: ParamId) -> Option<FieldPlate> {
    let (art, caption) = match id {
        // SENSOR ANGLE — the sampling cone seen from overhead. A body at left, a
        // stippled cone opening to the right (densest dead-ahead), degree ticks,
        // and an amber `‹── cone = 45° ──›` dimension line.
        ParamId::SensorAngle => (
            "          ·  ·  :  *      +22°\n        ·  ·  :  *  *\n  ● «‹»·  ·  :  *  *  ●   0°\n        ·  ·  :  *  *\n          ·  ·  :  *      −22°\n        «‹──── cone = 45° ────›»",
            "overhead view of the sampling cone\ndensest dead-ahead, faint at edges",
        ),
        // DECAY — a deposited frond losing substance over time: a dense head
        // `●*:·.` at "now" scattering to faint stipple toward "older →".
        ParamId::Decay => (
            " now               older «→»\n ●*:·.\n ●*:·. ·  ·  ·   ·    ·     ·\n ●*:·. ·  ·   ·    ·     ·\n ●*:·. ·  ·  ·   ·    ·\n     «▲» half-life",
            "a deposit fades each frame\nhigh decay = long tail, low = gone",
        ),
        _ => return None,
    };
    Some(FieldPlate { art, caption })
}

/// Visible column count of an art line, ignoring the consumed authoring markers
/// (`«` `»` and `⟦…⟧`). Trailing spaces are not counted.
pub(crate) fn visible_width(line: &str) -> usize {
    let mut n = 0usize;
    let mut chars = line.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '«' | '»' => {}
            '⟦' => {
                for c2 in chars.by_ref() {
                    if c2 == '⟧' {
                        break;
                    }
                }
            }
            _ => n += 1,
        }
    }
    let trail = line.chars().rev().take_while(|c| *c == ' ').count();
    n.saturating_sub(trail)
}

/// Strip the block's common leading indent; return (dedented art, visible width).
pub(crate) fn dedent_and_measure(art: &str) -> (String, usize) {
    let min = art
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| l.chars().take_while(|c| *c == ' ').count())
        .min()
        .unwrap_or(0);
    let out: String = art
        .lines()
        .map(|l| l.chars().skip(min).collect::<String>())
        .collect::<Vec<_>>()
        .join("\n");
    let w = out.lines().map(visible_width).max().unwrap_or(0);
    (out, w)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn st() -> PanelStyle {
        crate::render::theme::GRUVBOX_DARK
    }

    #[test]
    fn render_art_consumes_markers_and_centers() {
        let rows = render_art(
            "«‹» x",
            "cap",
            Profile::Safe,
            &st(),
            Palette::Heat,
            None,
            20,
        );
        let text: String = rows.iter().map(|r| r.text()).collect::<Vec<_>>().join("|");
        // No marker glyphs survive.
        assert!(!text.contains('«') && !text.contains('»') && !text.contains('⟦'));
        // Each row is exactly `width` wide.
        assert!(rows.iter().all(|r| r.text().chars().count() == 20));
    }

    /// Flatten all rendered rows into one rich-cell stream (pad-agnostic search).
    fn all_cells(rows: Vec<RowBuf>) -> Vec<(char, Option<RgbColor>, Option<RgbColor>)> {
        rows.into_iter().flat_map(|r| r.into_rich()).collect()
    }

    #[test]
    fn measurement_run_uses_accent_active() {
        let s = st();
        // "«▲»" -> mapped to "^", colored accent_active (amber measurement).
        let cells = all_cells(render_art(
            "«▲»",
            "",
            Profile::Safe,
            &s,
            Palette::Heat,
            None,
            12,
        ));
        let lit = cells.iter().find(|(ch, _, _)| *ch == '^');
        assert_eq!(lit.and_then(|(_, fg, _)| *fg), Some(s.accent_active));
    }

    #[test]
    fn organism_is_palette_tinted() {
        let s = st();
        // "●" -> mapped to "@" in Safe, colored from the AUTHORED glyph's brightness.
        let cells = all_cells(render_art(
            "●",
            "",
            Profile::Safe,
            &s,
            Palette::Heat,
            None,
            6,
        ));
        let mass = cells.iter().find(|(ch, _, _)| *ch == '@');
        let expect = map_brightness_rgb(1.0, Palette::Heat, false, false, 0.0, None);
        assert_eq!(mass.and_then(|(_, fg, _)| *fg), Some(expect));
    }

    #[test]
    fn letters_and_degree_are_annotation() {
        let s = st();
        let cells = all_cells(render_art(
            "A 7 °",
            "",
            Profile::Safe,
            &s,
            Palette::Heat,
            None,
            12,
        ));
        for want in ['A', '7', '°'] {
            let c = cells.iter().find(|(ch, _, _)| *ch == want);
            assert_eq!(
                c.and_then(|(_, fg, _)| *fg),
                Some(s.text_secondary),
                "{want:?} should be annotation grey"
            );
        }
    }

    /// The full authored glyph kit. Every glyph must downgrade to a single-width
    /// safe-range glyph under `Safe` — a test, not a hope.
    const KIT: &str = "● · : * ( ) / \\ - _ . , ' ` ~ ‹ › ◂ ▸ → ─ │ ├ ┤ ═ ▲ − ° [ ]";

    #[test]
    fn safe_profile_maps_every_kit_glyph_to_single_width() {
        for ch in KIT.chars() {
            let mapped = map_glyph(ch, Profile::Safe);
            assert!(
                is_safe_width1(mapped),
                "Safe map of {ch:?} -> {mapped:?} is not in a safe single-width range"
            );
        }
    }

    #[test]
    fn profile_default_is_safe() {
        assert_eq!(Profile::default(), Profile::Safe);
    }

    #[test]
    fn visible_width_ignores_markers() {
        assert_eq!(visible_width("«ab»"), 2);
        assert_eq!(visible_width("⟦heat⟧·:*⟦⟧"), 3);
        assert_eq!(visible_width("xy   "), 2); // trailing spaces excluded
    }

    #[test]
    fn dedent_strips_common_indent() {
        let (out, w) = dedent_and_measure("    ab\n    cde");
        assert_eq!(out, "ab\ncde");
        assert_eq!(w, 3);
    }

    #[test]
    fn prototype_params_are_authored() {
        use crate::render::controls::registry::ParamId::*;
        for id in [SensorAngle, Decay] {
            assert!(field_plate(id).is_some(), "{id:?} should be authored");
        }
        // Not-yet-drawn params fall back to the kind-aware pane.
        assert!(field_plate(crate::render::controls::registry::ParamId::TurnAngle).is_none());
    }

    /// Every authored plate must render Safe-clean: each glyph (excluding the
    /// consumed markers) downgrades to a single-width safe-range glyph, and it
    /// must fit the detail pane (top pad + ≤ 6 art rows + blank + 2 caption = 10).
    #[test]
    fn authored_plates_are_safe_clean_and_fit() {
        use crate::render::controls::registry::ParamId::*;
        for id in [SensorAngle, Decay] {
            let plate = field_plate(id).unwrap();
            // Width 1 under Safe.
            let mut chars = plate.art.chars().peekable();
            while let Some(c) = chars.next() {
                match c {
                    '«' | '»' | '\n' => {}
                    '⟦' => {
                        for c2 in chars.by_ref() {
                            if c2 == '⟧' {
                                break;
                            }
                        }
                    }
                    _ => {
                        let m = map_glyph(c, Profile::Safe);
                        assert!(is_safe_width1(m), "{id:?}: {c:?} -> {m:?} not safe-width-1");
                    }
                }
            }
            // Row budget: top pad + art + forced blank + 2 caption lines <= 10 rows.
            let art_rows = plate.art.lines().count();
            assert!(
                art_rows <= 6,
                "{id:?}: {art_rows} art rows > 6 (pane overflows)"
            );
        }
    }

    #[test]
    fn organism_and_measurement_classify() {
        // Stipple + line-art are organism; rulers/arrows are not.
        assert_eq!(mass_brightness('●'), Some(1.0));
        assert_eq!(mass_brightness('·'), Some(0.16));
        assert_eq!(mass_brightness('~'), Some(0.50));
        assert!(mass_brightness('─').is_none());
        assert!(mass_brightness('A').is_none());
    }

    #[test]
    fn live_option_lights_matching_run() {
        let s = st();
        let art = "  ⟦opt:mean⟧[ MEAN ]⟦⟧   ⟦opt:gauss⟧gauss⟦⟧\n  ⟦caret⟧";
        // live value "Gaussian" -> gauss run lit, mean run not.
        let cells = all_cells(render_art(
            art,
            "",
            Profile::Safe,
            &s,
            Palette::Heat,
            Some("Gaussian"),
            40,
        ));
        // 'g' from "gauss" is ignited; 'M' from "MEAN" is annotation (text_secondary).
        let g = cells.iter().find(|(ch, _, _)| *ch == 'g');
        let m = cells.iter().find(|(ch, _, _)| *ch == 'M');
        assert_eq!(g.and_then(|(_, fg, _)| *fg), Some(s.accent_ignite));
        assert_eq!(m.and_then(|(_, fg, _)| *fg), Some(s.text_secondary));
        // caret row drew at least one '_' in accent_ignite under the gauss span.
        assert!(cells
            .iter()
            .any(|(ch, fg, _)| *ch == '_' && *fg == Some(s.accent_ignite)));
    }

    #[test]
    fn live_option_other_value_lights_other_run() {
        let s = st();
        let art = "  ⟦opt:mean⟧[ MEAN ]⟦⟧   ⟦opt:gauss⟧gauss⟦⟧";
        let cells = all_cells(render_art(
            art,
            "",
            Profile::Safe,
            &s,
            Palette::Heat,
            Some("Mean 3x3"),
            40,
        ));
        let m = cells.iter().find(|(ch, _, _)| *ch == 'M');
        assert_eq!(m.and_then(|(_, fg, _)| *fg), Some(s.accent_ignite));
    }
}
