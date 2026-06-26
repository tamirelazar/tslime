use crate::render::palette::RgbColor;
use crate::render::theme::PanelStyle;

/// Renders a gauge containing exactly `width` cells, or no cells when `width` is zero.
///
/// Values outside a valid finite range are clamped. Non-finite values or bounds,
/// and degenerate or reversed ranges, collapse to the lower-bound position. The
/// value marker overwrites the bar, then the default marker overwrites the value
/// marker when both occupy the same cell.
pub fn gauge(
    value: f32,
    range: (f32, f32),
    default: f32,
    width: usize,
    st: &PanelStyle,
) -> Vec<(char, RgbColor)> {
    if width == 0 {
        return Vec::new();
    }

    let value_ratio = normalize(value, range);
    let filled = ((value_ratio * width as f32).round() as usize).min(width);
    let mut cells = vec![('░', st.muted); width];
    cells[..filled].fill(('█', st.accent_active));

    let value_tick = tick_column(value_ratio, width);
    cells[value_tick] = ('│', st.accent_active);

    let default_tick = tick_column(normalize(default, range), width);
    cells[default_tick] = ('▲', st.state_default);
    cells
}

/// Renders values as an eight-level sparkline normalized across finite samples.
///
/// An empty slice returns an empty string. Constant and all-invalid non-empty
/// slices render the lowest bar for every sample; individual non-finite samples
/// also render at that lowest level.
pub fn sparkline(values: &[f32]) -> String {
    const BARS: [char; 8] = ['▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];

    let mut min = f32::INFINITY;
    let mut max = f32::NEG_INFINITY;
    for &value in values {
        if value.is_finite() {
            min = min.min(value);
            max = max.max(value);
        }
    }

    let range = (min, max);
    let mut result = String::with_capacity(values.len().saturating_mul('█'.len_utf8()));
    for &value in values {
        let level = (normalize(value, range) * 7.0).round() as usize;
        result.push(BARS[level]);
    }
    result
}

/// Renders a one-cell color swatch.
pub fn swatch(accent: RgbColor) -> Vec<(char, RgbColor)> {
    vec![('■', accent)]
}

/// Returns a one-cell horizontal divider without allocating.
pub fn separator(st: &PanelStyle) -> (char, RgbColor) {
    ('─', st.muted)
}

/// Renders colored legend labels separated by one space.
///
/// Each separating space uses the preceding label's color, avoiding an
/// invented neutral color when no [`PanelStyle`] is available.
/// Labels are iterated as Unicode scalar values; callers must ensure each scalar
/// occupies one terminal column.
pub fn legend(entries: &[(&str, RgbColor)]) -> Vec<(char, RgbColor)> {
    let label_chars = entries
        .iter()
        .map(|(label, _)| label.chars().count())
        .sum::<usize>();
    let capacity = label_chars.saturating_add(entries.len().saturating_sub(1));
    let mut cells = Vec::with_capacity(capacity);
    let mut preceding_color = None;

    for &(label, color) in entries {
        if let Some(space_color) = preceding_color {
            cells.push((' ', space_color));
        }
        cells.extend(label.chars().map(|ch| (ch, color)));
        preceding_color = Some(color);
    }
    cells
}

/// Truncates `s` to at most `max` visible characters (Unicode scalar values).
///
/// Returns `s` unchanged when its char count is already within `max`. When it
/// exceeds `max`, takes the first `max - 1` chars and appends the single-char
/// ellipsis `…` so the total visible width equals exactly `max`.
///
/// `max` must be at least 1; passing 0 returns an empty string.
pub fn truncate_ellipsis(s: &str, max: usize) -> String {
    if max == 0 {
        return String::new();
    }
    let chars: Vec<char> = s.chars().collect();
    if chars.len() <= max {
        s.to_string()
    } else {
        let mut result: String = chars[..max - 1].iter().collect();
        result.push('…');
        result
    }
}

fn normalize(value: f32, (lo, hi): (f32, f32)) -> f32 {
    if !value.is_finite() || !lo.is_finite() || !hi.is_finite() || hi <= lo {
        return 0.0;
    }
    let value = f64::from(value);
    let lo = f64::from(lo);
    let hi = f64::from(hi);
    ((value - lo) / (hi - lo)).clamp(0.0, 1.0) as f32
}

fn tick_column(ratio: f32, width: usize) -> usize {
    ((ratio * width.saturating_sub(1) as f32).round() as usize).min(width - 1)
}

#[cfg(test)]
mod kit_tests {
    use super::*;
    use crate::render::theme::GRUVBOX_DARK;

    #[test]
    fn gauge_fills_proportionally_and_marks_ticks() {
        let g = gauge(5.0, (0.0, 10.0), 0.0, 4, &GRUVBOX_DARK);
        assert_eq!(g.len(), 4);
        assert_eq!(
            g,
            vec![
                ('▲', GRUVBOX_DARK.state_default),
                ('█', GRUVBOX_DARK.accent_active),
                ('│', GRUVBOX_DARK.accent_active),
                ('░', GRUVBOX_DARK.muted),
            ]
        );
    }

    #[test]
    fn zero_width_gauge_is_empty() {
        assert!(gauge(5.0, (0.0, 10.0), 0.0, 0, &GRUVBOX_DARK).is_empty());
    }

    #[test]
    fn gauge_places_ticks_at_endpoints_and_default_wins_collisions() {
        assert_eq!(
            gauge(0.0, (0.0, 1.0), 1.0, 3, &GRUVBOX_DARK),
            vec![
                ('│', GRUVBOX_DARK.accent_active),
                ('░', GRUVBOX_DARK.muted),
                ('▲', GRUVBOX_DARK.state_default),
            ]
        );
        assert_eq!(
            gauge(1.0, (0.0, 1.0), 1.0, 1, &GRUVBOX_DARK),
            vec![('▲', GRUVBOX_DARK.state_default)]
        );
    }

    #[test]
    fn gauge_collapses_invalid_ranges_and_nonfinite_values_safely() {
        let collapsed = vec![
            ('▲', GRUVBOX_DARK.state_default),
            ('░', GRUVBOX_DARK.muted),
            ('░', GRUVBOX_DARK.muted),
        ];
        assert_eq!(gauge(1.0, (1.0, 1.0), 1.0, 3, &GRUVBOX_DARK), collapsed);
        assert_eq!(
            gauge(f32::NAN, (0.0, 1.0), f32::INFINITY, 3, &GRUVBOX_DARK),
            collapsed
        );
        assert_eq!(gauge(0.5, (1.0, 0.0), 0.5, 3, &GRUVBOX_DARK), collapsed);
    }

    #[test]
    fn normalization_preserves_extreme_finite_range() {
        let range = (-f32::MAX, f32::MAX);
        assert_eq!(normalize(-f32::MAX, range), 0.0);
        assert!((normalize(0.0, range) - 0.5).abs() <= f32::EPSILON);
        assert_eq!(normalize(f32::MAX, range), 1.0);
    }

    #[test]
    fn extreme_finite_ranges_reach_midpoint_and_upper_levels() {
        let range = (-f32::MAX, f32::MAX);
        let upper: String = gauge(f32::MAX, range, -f32::MAX, 5, &GRUVBOX_DARK)
            .iter()
            .map(|(ch, _)| ch)
            .collect();
        let middle: String = gauge(0.0, range, -f32::MAX, 5, &GRUVBOX_DARK)
            .iter()
            .map(|(ch, _)| ch)
            .collect();

        assert_eq!(upper, "▲███│");
        assert_eq!(middle, "▲█│░░");
        assert_eq!(sparkline(&[-f32::MAX, 0.0, f32::MAX]), "▁▅█");
    }

    #[test]
    fn sparkline_maps_min_and_max() {
        let s = sparkline(&[0.0, 1.0]);
        let cs: Vec<char> = s.chars().collect();
        assert_eq!(cs[0], '▁');
        assert_eq!(cs[1], '█');
    }

    #[test]
    fn empty_sparkline_is_empty() {
        assert!(sparkline(&[]).is_empty());
    }

    #[test]
    fn constant_and_all_invalid_sparklines_use_the_lowest_level() {
        assert_eq!(sparkline(&[3.0, 3.0]), "▁▁");
        assert_eq!(
            sparkline(&[f32::NAN, f32::INFINITY, f32::NEG_INFINITY]),
            "▁▁▁"
        );
    }

    #[test]
    fn swatch_is_block_in_accent() {
        let accent = RgbColor::new(7, 8, 9);
        assert_eq!(swatch(accent), vec![('■', accent)]);
    }

    #[test]
    fn separator_is_one_muted_divider() {
        assert_eq!(separator(&GRUVBOX_DARK), ('─', GRUVBOX_DARK.muted));
    }

    #[test]
    fn truncate_adds_ellipsis_only_when_cut() {
        assert_eq!(truncate_ellipsis("hello", 10), "hello");
        assert_eq!(truncate_ellipsis("hello world", 8), "hello w…");
    }

    #[test]
    fn truncate_ellipsis_zero_max_is_empty() {
        assert_eq!(truncate_ellipsis("hello", 0), "");
    }

    #[test]
    fn truncate_ellipsis_exact_fit_unchanged() {
        assert_eq!(truncate_ellipsis("hello", 5), "hello");
    }

    #[test]
    fn legend_separates_colored_labels_with_one_preceding_color_space() {
        let cli = RgbColor::new(1, 2, 3);
        let modified = RgbColor::new(4, 5, 6);
        assert_eq!(
            legend(&[("C", cli), ("M", modified)]),
            vec![('C', cli), (' ', cli), ('M', modified)]
        );
    }
}
