//! Field-plate specimen art for the Console detail pane: a portable glyph-
//! profile system, per-parameter art data, and a material-aware renderer.
//!
//! Art is authored ONCE in an expressive "fancy" glyph set and downgraded
//! through the active [`Profile`] before placement. `Safe` (the default, what
//! ships) uses only ASCII + box-drawing + block-element ranges, which every
//! mainstream terminal renders single-width — so nothing shears.

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
}
