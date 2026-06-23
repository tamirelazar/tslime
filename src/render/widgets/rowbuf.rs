use crate::render::palette::RgbColor;
use crate::render::panel::RichCell;

/// A single overlay row under construction: parallel character, foreground,
/// and background layers.
///
/// Merged superset of the former `console.rs` and `tuner.rs` copies.
/// Width and offsets count Unicode scalar values, not terminal display columns;
/// callers must supply glyphs that each occupy a single terminal column.
pub struct RowBuf {
    chars: Vec<char>,
    fg: Vec<Option<RgbColor>>,
    bg: Vec<Option<RgbColor>>,
}

impl RowBuf {
    /// Creates a transparent row with no matte background.
    pub fn new(w: usize) -> Self {
        Self {
            chars: vec![' '; w],
            fg: vec![None; w],
            bg: vec![None; w],
        }
    }

    /// Creates a row pre-filled with a matte background.
    pub fn new_matte(w: usize, bg: RgbColor) -> Self {
        Self {
            chars: vec![' '; w],
            fg: vec![None; w],
            bg: vec![Some(bg); w],
        }
    }

    /// Writes text and any set color layers, clipping at the row width.
    pub fn put(&mut self, at: usize, s: &str, fg: Option<RgbColor>, bg: Option<RgbColor>) {
        let (Some(chars), Some(fg_layer), Some(bg_layer)) = (
            self.chars.get_mut(at..),
            self.fg.get_mut(at..),
            self.bg.get_mut(at..),
        ) else {
            return;
        };
        let destination = chars.iter_mut().zip(fg_layer).zip(bg_layer);
        for (((dst_ch, dst_fg), dst_bg), ch) in destination.zip(s.chars()) {
            *dst_ch = ch;
            if fg.is_some() {
                *dst_fg = fg;
            }
            if bg.is_some() {
                *dst_bg = bg;
            }
        }
    }

    /// Writes colored cells and an optional background, clipping at the row width.
    pub fn put_cells(&mut self, at: usize, cells: &[(char, RgbColor)], bg: Option<RgbColor>) {
        let (Some(chars), Some(fg_layer), Some(bg_layer)) = (
            self.chars.get_mut(at..),
            self.fg.get_mut(at..),
            self.bg.get_mut(at..),
        ) else {
            return;
        };
        let destination = chars.iter_mut().zip(fg_layer).zip(bg_layer);
        for (((dst_ch, dst_fg), dst_bg), &(ch, col)) in destination.zip(cells) {
            *dst_ch = ch;
            *dst_fg = Some(col);
            if bg.is_some() {
                *dst_bg = bg;
            }
        }
    }

    /// Sets the background across a clipped range.
    pub fn set_bg(&mut self, range: std::ops::Range<usize>, bg: RgbColor) {
        let start = range.start.min(self.bg.len());
        let end = range.end.min(self.bg.len());
        if start < end {
            self.bg[start..end].fill(Some(bg));
        }
    }

    /// Overlays another row, preserving destination layers that are unset in the source.
    pub fn blit(&mut self, at: usize, other: &RowBuf) {
        let (Some(chars), Some(fg_layer), Some(bg_layer)) = (
            self.chars.get_mut(at..),
            self.fg.get_mut(at..),
            self.bg.get_mut(at..),
        ) else {
            return;
        };
        let destination = chars.iter_mut().zip(fg_layer).zip(bg_layer);
        let source = other.chars.iter().zip(&other.fg).zip(&other.bg);
        for (((dst_ch, dst_fg), dst_bg), ((src_ch, src_fg), src_bg)) in destination.zip(source) {
            *dst_ch = *src_ch;
            if src_fg.is_some() {
                *dst_fg = *src_fg;
            }
            if src_bg.is_some() {
                *dst_bg = *src_bg;
            }
        }
    }

    /// Consumes the row and merges its parallel layers into rich cells.
    pub fn into_rich(self) -> Vec<RichCell> {
        self.chars
            .into_iter()
            .zip(self.fg)
            .zip(self.bg)
            .map(|((ch, fg), bg)| (ch, fg, bg))
            .collect()
    }

    /// Returns the row's character layer as text.
    pub fn text(&self) -> String {
        self.chars.iter().collect()
    }
}

#[cfg(test)]
mod rowbuf_tests {
    use super::*;
    use crate::render::palette::RgbColor;

    #[test]
    fn put_writes_chars_and_clips_at_width() {
        let mut r = RowBuf::new(4);
        r.put(2, "ABCD", Some(RgbColor::new(1, 2, 3)), None);
        assert_eq!(r.text(), "  AB");
    }

    #[test]
    fn put_at_max_offset_clips_without_wrapping() {
        let mut r = RowBuf::new(2);
        r.put(
            usize::MAX,
            "AB",
            Some(RgbColor::new(1, 2, 3)),
            Some(RgbColor::new(4, 5, 6)),
        );
        assert_eq!(r.into_rich(), vec![(' ', None, None); 2]);
    }

    #[test]
    fn new_matte_fills_bg() {
        let bg = RgbColor::new(10, 16, 12);
        let r = RowBuf::new_matte(3, bg);
        let rich = r.into_rich();
        assert!(rich.iter().all(|(_, _, b)| *b == Some(bg)));
    }

    #[test]
    fn put_cells_optionally_overrides_bg() {
        let mut r = RowBuf::new(3);
        r.put_cells(
            0,
            &[('x', RgbColor::new(9, 9, 9))],
            Some(RgbColor::new(1, 1, 1)),
        );
        let rich = r.into_rich();
        assert_eq!(
            rich[0],
            (
                'x',
                Some(RgbColor::new(9, 9, 9)),
                Some(RgbColor::new(1, 1, 1))
            )
        );
    }

    #[test]
    fn put_cells_at_max_offset_clips_without_wrapping() {
        let mut r = RowBuf::new(2);
        r.put_cells(
            usize::MAX,
            &[('A', RgbColor::new(1, 2, 3)), ('B', RgbColor::new(4, 5, 6))],
            Some(RgbColor::new(7, 8, 9)),
        );
        assert_eq!(r.into_rich(), vec![(' ', None, None); 2]);
    }

    #[test]
    fn set_bg_clips_huge_range_to_row_width() {
        let bg = RgbColor::new(3, 4, 5);
        let mut r = RowBuf::new(3);
        r.set_bg(1..usize::MAX, bg);
        let rich = r.into_rich();
        assert_eq!(rich[0].2, None);
        assert!(rich[1..].iter().all(|cell| cell.2 == Some(bg)));
    }

    #[test]
    fn set_bg_ignores_empty_reversed_and_out_of_bounds_ranges() {
        let matte = RgbColor::new(1, 2, 3);
        let mut r = RowBuf::new_matte(3, matte);
        let bg = RgbColor::new(4, 5, 6);
        r.set_bg(1..1, bg);
        r.set_bg(std::ops::Range { start: 2, end: 1 }, bg);
        r.set_bg(3..usize::MAX, bg);
        assert!(r.into_rich().iter().all(|cell| cell.2 == Some(matte)));
    }

    #[test]
    fn zero_width_row_operations_are_noops() {
        let fg = RgbColor::new(1, 2, 3);
        let bg = RgbColor::new(4, 5, 6);
        let mut r = RowBuf::new(0);
        r.put(0, "x", Some(fg), Some(bg));
        r.put_cells(0, &[('x', fg)], Some(bg));
        r.set_bg(0..usize::MAX, bg);
        r.blit(0, &RowBuf::new_matte(1, bg));
        assert_eq!(r.text(), "");
        assert!(r.into_rich().is_empty());
    }

    #[test]
    fn blit_merges_only_set_layers() {
        let matte = RgbColor::new(5, 5, 5);
        let dst_fg = RgbColor::new(1, 1, 1);
        let src_fg = RgbColor::new(2, 2, 2);
        let src_bg = RgbColor::new(8, 8, 8);
        let mut dst = RowBuf::new_matte(4, matte);
        dst.put(1, "  ", Some(dst_fg), None);
        let mut src = RowBuf::new(2);
        src.put(0, "h", Some(src_fg), None);
        src.put(1, "i", None, Some(src_bg));
        dst.blit(1, &src);
        let rich = dst.into_rich();
        assert_eq!(rich[1], ('h', Some(src_fg), Some(matte)));
        assert_eq!(rich[2], ('i', Some(dst_fg), Some(src_bg)));
    }

    #[test]
    fn blit_at_max_offset_clips_without_wrapping() {
        let mut dst = RowBuf::new(2);
        let mut src = RowBuf::new(2);
        src.put(
            0,
            "AB",
            Some(RgbColor::new(1, 2, 3)),
            Some(RgbColor::new(4, 5, 6)),
        );
        dst.blit(usize::MAX, &src);
        assert_eq!(dst.into_rich(), vec![(' ', None, None); 2]);
    }
}
