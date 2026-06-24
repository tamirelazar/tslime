//! Field-plate specimen art for the Console detail pane: a portable glyph-
//! profile system, per-parameter art data, and a material-aware renderer.
//!
//! Art is authored ONCE in an expressive "fancy" glyph set and downgraded
//! through the active [`Profile`] before placement. `Safe` (the default, what
//! ships) uses only ASCII + box-drawing + block-element ranges, which every
//! mainstream terminal renders single-width — so nothing shears.

use crate::render::controls::registry::ParamId;
use crate::render::palette::Palette;

/// Glyph-rendering profile. Authored art is downgraded through the active one.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum Profile {
    /// Geometric shapes + arrows. Prettier; opt-in (capability-gated). Unused this batch.
    Fancy,
    /// ASCII + box + block only. Cell-perfect everywhere. Ships.
    #[default]
    Safe,
    /// Pure ASCII (no block elements). Ultimate fallback. Unused this batch.
    Ascii,
}

/// Downgrade one authored glyph for the active profile, preserving width 1:1.
/// In `Safe`, block/box/ASCII pass through unchanged; only ambiguous-width
/// geometric shapes, arrows, and a few marks are remapped.
pub fn map_glyph(ch: char, p: Profile) -> char {
    match p {
        Profile::Fancy => ch,
        Profile::Safe => match ch {
            '●' => '█',
            '◉' => '▓',
            '◌' => ':',
            '○' => 'o',
            '·' | '∙' => '.',
            '╱' => '/',
            '╲' => '\\',
            '→' => '>',
            '←' => '<',
            '↑' => '^',
            '↓' => 'v',
            '↗' => '/',
            '↘' => '\\',
            '↖' => '\\',
            '↙' => '/',
            '▲' | '▴' => '^',
            '▼' | '▾' => 'v',
            '◂' => '<',
            '▸' => '>',
            '‹' => '<',
            '›' => '>',
            '╴' => '_',
            '×' => 'x',
            '°' => ' ',
            '↵' => ' ',
            other => other,
        },
        Profile::Ascii => match ch {
            '·' | '∙' => '.',
            '░' => ':',
            '▒' => '+',
            '▓' => '*',
            '█' => '#',
            '◌' => 'o',
            '◉' | '●' => '@',
            '○' => 'o',
            '╱' => '/',
            '╲' => '\\',
            '│' => '|',
            '─' => '-',
            '┌' | '┐' | '└' | '┘' | '├' | '┤' | '┬' | '┴' | '┼' => '+',
            '→' => '>',
            '←' => '<',
            '↑' => '^',
            '↓' => 'v',
            '↗' => '/',
            '↘' => '\\',
            '↖' => '\\',
            '↙' => '/',
            '▲' | '▴' => '^',
            '▼' | '▾' => 'v',
            '◂' => '<',
            '▸' => '>',
            '‹' => '<',
            '›' => '>',
            '╴' => '-',
            '▁' => '_',
            '▂' | '▃' => '.',
            '▄' | '▅' => '-',
            '▆' | '▇' => '=',
            '×' => 'x',
            '°' => ' ',
            '↵' => ' ',
            '✱' => '*',
            other => other,
        },
    }
}

/// True if `ch` is in a range every terminal renders single-width and draws
/// itself: ASCII `0x20–7E`, box-drawing `U+2500–257F`, block `U+2580–259F`.
#[allow(dead_code)]
pub(crate) fn is_safe_width1(ch: char) -> bool {
    let u = ch as u32;
    (0x20..=0x7E).contains(&u) || (0x2500..=0x257F).contains(&u) || (0x2580..=0x259F).contains(&u)
}

/// MASS brightness for a glyph (None = not a MASS glyph). Drives palette tint.
#[allow(dead_code)]
pub(crate) fn shade_brightness(ch: char) -> Option<f32> {
    Some(match ch {
        '·' | '∙' => 0.14,
        '░' => 0.28,
        '◌' => 0.34,
        '▒' => 0.52,
        '▓' => 0.76,
        '◉' | '●' => 0.92,
        '█' => 1.0,
        _ => return None,
    })
}

/// True for STRUCTURE glyphs (rays, axes, arrows, RULE, caret, brackets, ticks).
#[allow(dead_code)]
pub(crate) fn is_structural(ch: char) -> bool {
    matches!(
        ch,
        '╲' | '╱'
            | '│'
            | '─'
            | '►'
            | '◄'
            | '▶'
            | '◀'
            | '├'
            | '┤'
            | '┬'
            | '┴'
            | '↵'
            | '→'
            | '←'
            | '↑'
            | '↓'
            | '◈'
            | '┌'
            | '┐'
            | '└'
            | '┘'
            | '↗'
            | '↘'
            | '↖'
            | '↙'
            | '◂'
            | '▸'
            | '▴'
            | '▾'
            | '╴'
            | '▲'
            | '▼'
            | '‹'
            | '›'
            | '▁'
            | '▂'
            | '▃'
            | '▄'
            | '▅'
            | '▆'
            | '▇'
            | '┼'
            | '━'
    )
}

/// Resolve a `⟦name⟧` palette tag to a `Palette` (PALETTE strip swatches).
#[allow(dead_code)]
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

/// One parameter's specimen art + reading caption.
#[allow(dead_code)]
pub struct FieldPlate {
    /// Authored ASCII-art specimen; may contain `«»` and `⟦⟧` authoring markers.
    pub art: &'static str,
    /// Short reading text (1-2 lines) displayed below the art.
    pub caption: &'static str,
}

/// The authored field plate for `id`, or `None` for params not yet authored.
/// 9 authored this batch; the remaining ~38 fall back to the kind-aware detail.
#[allow(dead_code)]
pub fn field_plate(id: ParamId) -> Option<FieldPlate> {
    let (art, caption) = match id {
        ParamId::SensorAngle => (
            "  SENSE · sample 3 points\n\n       ╱·░▒▓\n  █──→  ·░▒▓   fwd\n       ╲·░▒▓\n\n  5° │─«▲»──────│ 90°",
            "L/R sensors splay this wide.\nnarrow = networks · wide = chaos",
        ),
        ParamId::Decay => (
            "  DECAY · trail kept / frame\n\n  now\n  █▓▓▒▒▒░░░░·· · ·  ·\n                older →\n  .50 │───────«▲»─│ .99\n        .88  «.90»  .92",
            "one deposit, aging left to right.\nlow = ghostly · high = long-lived",
        ),
        ParamId::DiffusionKernel => (
            "  DIFFUSE · cell bleeds out\n\n  ⟦opt:mean⟧[ MEAN 3×3 ]⟦⟧    ⟦opt:gauss⟧gauss 5×5⟦⟧\n     ▒▓█▓▒          ·░▒▓▒░·\n     sharp·fast     soft·organic\n  ⟦caret⟧\n  ◂ ▸ flip       active = caret",
            "how a cell bleeds to neighbours.\ncrisp box vs. soft organic bloom",
        ),
        ParamId::Wind => (
            "  FORCE · drift bias, all agents\n      ↖   ↑   ↗\n        ╲ │ ╱     drift\n  ←──── █ ──→   ░▒▓→\n        ╱ │ ╲\n      ↙   ↓   ↘\n  ⟦opt:east⟧[ E → ]⟦⟧   · center = Off",
            "constant force on every agent.\npick a bearing · center = Off",
        ),
        ParamId::Palette => (
            "  RENDER · trail pigment\n\n  rail  ⟦slime⟧▒▓█⟦⟧ ⟦organic⟧▒▓█⟦⟧ ⟦heat⟧▒▓█⟦⟧ ⟦ocean⟧▒▓█⟦⟧\n  focus «‹» ⟦heat⟧·░▒▓█⟦⟧ «›»  «HEAT»\n  ◂ ▸ scroll",
            "the trail's pigment.\neach swatch is that palette's own ramp",
        ),
        ParamId::IntensityMapping => (
            "  RENDER · brightness curve\n\n  ⟦opt:lin⟧lin⟦⟧  ⟦opt:log⟧log⟦⟧  ⟦opt:exp⟧exp⟦⟧  ⟦opt:pow⟧pow⟦⟧  ⟦opt:sqrt⟧sqrt⟦⟧\n      ▃▅▆▇████\n  ·░░▒▒▓▓██   mapped ramp\n  ◂ ▸ pick curve",
            "brightness response curve.\nlog lifts dim trails into view",
        ),
        ParamId::TrailDelta => (
            "  RENDER · show only change\n\n  ⟦opt:on⟧● on⟦⟧    ⟦opt:off⟧○ off⟦⟧\n\n  full field       changed only\n  ░▒▓▓▒░           ·  ▒    ·",
            "lights only cells that changed.\noff = full field · on = motion only",
        ),
        ParamId::Population => (
            "  AGENTS · set at launch\n\n  ·░▒▓ population ▓▒░·\n\n  · ◌ ░ ▒ ◉ ● ◉ ▒ ░ ◌ ·\n  density at this count",
            "agent count, fixed at launch.\nthe field's grain at this scale",
        ),
        ParamId::Reset => (
            "  SYS · params → defaults\n\n\n       «[  RESET  ]»  ↵\n",
            "returns every param to preset.\na quiet, deliberate action",
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
#[allow(dead_code)]
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

    /// The full authored glyph kit (handoff §3). Every glyph must downgrade to a
    /// single-width safe-range glyph under `Safe` — this turns "holds up across
    /// emulators" into a test, not a hope.
    const KIT: &str = "█·░▒▓◌◉●╱─╲→↑←↓↗↘↖↙│▲▼╴◂▸‹›○▁▂▃▄▅▆▇[]↵°×";

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
        // 4 visible chars ("ab cd"? no) -> count real glyphs only, markers consumed.
        assert_eq!(visible_width("«ab»"), 2);
        assert_eq!(visible_width("⟦heat⟧▒▓█⟦⟧"), 3);
        assert_eq!(visible_width("xy   "), 2); // trailing spaces excluded
    }

    #[test]
    fn dedent_strips_common_indent() {
        let (out, w) = dedent_and_measure("    ab\n    cde");
        assert_eq!(out, "ab\ncde");
        assert_eq!(w, 3);
    }

    #[test]
    fn nine_params_are_authored() {
        use crate::render::controls::registry::ParamId::*;
        for id in [
            SensorAngle,
            Decay,
            DiffusionKernel,
            Wind,
            Palette,
            IntensityMapping,
            TrailDelta,
            Population,
            Reset,
        ] {
            assert!(field_plate(id).is_some(), "{id:?} should be authored");
        }
        assert!(field_plate(crate::render::controls::registry::ParamId::TurnAngle).is_none());
    }

    /// Every authored plate must render Safe-clean: each glyph (excluding the
    /// consumed markers) downgrades to a single-width safe-range glyph.
    #[test]
    fn authored_plates_are_safe_clean() {
        use crate::render::controls::registry::ParamId::*;
        for id in [
            SensorAngle,
            Decay,
            DiffusionKernel,
            Wind,
            Palette,
            IntensityMapping,
            TrailDelta,
            Population,
            Reset,
        ] {
            let plate = field_plate(id).unwrap();
            // strip markers, then check each remaining glyph
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
        }
    }

    #[test]
    fn shade_and_structure_classify() {
        assert_eq!(shade_brightness('█'), Some(1.0));
        assert!(shade_brightness('/').is_none());
        assert!(is_structural('╱'));
        assert!(!is_structural('█'));
    }
}
