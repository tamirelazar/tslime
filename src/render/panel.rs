use super::palette::RgbColor;
use super::theme::PanelStyle;

/// Text alignment within a column or cell.
#[derive(Clone, Debug, Copy, PartialEq)]
pub enum TextAlignment {
    /// Left-aligned text
    Left,
    /// Center-aligned text
    Center,
    /// Right-aligned text
    Right,
}

/// Title alignment within the panel header.
#[derive(Clone, Debug, Copy, PartialEq)]
pub enum TitleAlignment {
    /// Left-aligned title
    Left,
    /// Center-aligned title
    Center,
}

/// Size configuration for a panel (kept for API compatibility).
#[derive(Clone, Debug, Default)]
pub struct PanelSize {
    /// Width in characters.
    pub width: usize,
    /// Height in characters.
    pub height: usize,
}

impl PanelSize {
    /// Creates a new PanelSize with the given dimensions.
    pub fn new(width: usize, height: usize) -> Self {
        Self { width, height }
    }

    /// Calculates the inner width after subtracting padding and border.
    pub fn inner_width(&self, padding: &Padding, border: bool) -> usize {
        let border_width = if border { 2 } else { 0 };
        self.width
            .saturating_sub(padding.left + padding.right + border_width)
    }

    /// Calculates the inner height after subtracting padding and border.
    pub fn inner_height(&self, padding: &Padding, border: bool) -> usize {
        let border_height = if border { 2 } else { 0 };
        self.height
            .saturating_sub(padding.top + padding.bottom + border_height)
    }
}

/// Padding configuration for panel content.
#[derive(Clone, Debug, Default)]
pub struct Padding {
    /// Top padding in lines.
    pub top: usize,
    /// Bottom padding in lines.
    pub bottom: usize,
    /// Left padding in characters.
    pub left: usize,
    /// Right padding in characters.
    pub right: usize,
}

impl Padding {
    /// Creates a new Padding with individual values for each side.
    pub fn new(top: usize, bottom: usize, left: usize, right: usize) -> Self {
        Self {
            top,
            bottom,
            left,
            right,
        }
    }

    /// Creates uniform padding for all sides.
    pub fn uniform(all: usize) -> Self {
        Self {
            top: all,
            bottom: all,
            left: all,
            right: all,
        }
    }

    /// Creates padding with vertical and horizontal values.
    pub fn vertical(vert: usize, horizontal: usize) -> Self {
        Self {
            top: vert,
            bottom: vert,
            left: horizontal,
            right: horizontal,
        }
    }

    /// Creates padding with horizontal and vertical values (alias for vertical).
    pub fn horizontal(horizontal: usize, vertical: usize) -> Self {
        Self {
            top: vertical,
            bottom: vertical,
            left: horizontal,
            right: horizontal,
        }
    }
}

/// Column layout configuration for panel content.
#[derive(Clone, Debug, Copy, PartialEq)]
pub enum ColumnLayout {
    /// Single column spanning full width.
    Single,
    /// Two equal columns (50/50 split).
    TwoEqual,
    /// Two columns with left wider (60/40 split).
    TwoLeftWide,
    /// Two columns with right wider (40/60 split).
    TwoRightWide,
    /// Four equal columns (25/25/25/25 split).
    FourEqual,
}

impl ColumnLayout {
    /// Returns the column width ratios for two-column layouts.
    ///
    /// For `FourEqual`, returns the combined left/right halves.
    pub fn column_ratios(&self, total_width: usize) -> (usize, usize) {
        match self {
            ColumnLayout::Single => (total_width, 0),
            ColumnLayout::TwoEqual => {
                let left = total_width / 2;
                (left, total_width - left)
            }
            ColumnLayout::TwoLeftWide => {
                let left = (total_width * 6) / 10;
                (left, total_width - left)
            }
            ColumnLayout::TwoRightWide => {
                let left = (total_width * 4) / 10;
                (left, total_width - left)
            }
            ColumnLayout::FourEqual => {
                let left = total_width / 2;
                (left, total_width - left)
            }
        }
    }

    /// Returns the four column widths for `FourEqual` layout.
    ///
    /// Distributes `total_width` as evenly as possible across 4 columns.
    pub fn four_column_ratios(total_width: usize) -> [usize; 4] {
        let base = total_width / 4;
        let rem = total_width % 4;
        [
            base + usize::from(rem > 0),
            base + usize::from(rem > 1),
            base + usize::from(rem > 2),
            base,
        ]
    }

    /// Returns true if this is a single-column layout.
    pub fn is_single(&self) -> bool {
        matches!(self, ColumnLayout::Single)
    }

    /// Returns true if this is a two-column layout.
    pub fn is_two_column(&self) -> bool {
        matches!(
            self,
            ColumnLayout::TwoEqual | ColumnLayout::TwoLeftWide | ColumnLayout::TwoRightWide
        )
    }

    /// Returns true if this is a four-column layout.
    pub fn is_four_column(&self) -> bool {
        matches!(self, ColumnLayout::FourEqual)
    }
}

/// Border character configuration for panel drawing.
#[derive(Clone, Debug)]
pub struct BorderConfig {
    /// Top-left corner character.
    pub top_left: char,
    /// Top-right corner character.
    pub top_right: char,
    /// Bottom-left corner character.
    pub bottom_left: char,
    /// Bottom-right corner character.
    pub bottom_right: char,
    /// Top horizontal line character.
    pub top_horizontal: char,
    /// Bottom horizontal line character.
    pub bottom_horizontal: char,
    /// Vertical line character.
    pub vertical: char,
    /// Left intersection character (T-shape).
    pub left_intersection: char,
    /// Right intersection character (T-shape).
    pub right_intersection: char,
}

impl Default for BorderConfig {
    fn default() -> Self {
        Self {
            top_left: '▀',
            top_right: '▀',
            bottom_left: '▄',
            bottom_right: '▄',
            top_horizontal: '▀',
            bottom_horizontal: '▄',
            vertical: '█',
            left_intersection: '▌',
            right_intersection: '▐',
        }
    }
}

impl BorderConfig {
    /// Box drawing characters (╭╮╰╯│).
    pub fn box_drawing() -> Self {
        Self {
            top_left: '╭',
            top_right: '╮',
            bottom_left: '╰',
            bottom_right: '╯',
            top_horizontal: '─',
            bottom_horizontal: '─',
            vertical: '│',
            left_intersection: '├',
            right_intersection: '┤',
        }
    }

    /// Solid block border (default — half-blocks for horizontal lines).
    pub fn solid_blocks() -> Self {
        Self::default()
    }

    /// Simple ASCII border (+-+|+).
    pub fn simple() -> Self {
        Self {
            top_left: '+',
            top_right: '+',
            bottom_left: '+',
            bottom_right: '+',
            top_horizontal: '-',
            bottom_horizontal: '-',
            vertical: '|',
            left_intersection: '+',
            right_intersection: '+',
        }
    }
}

/// A single row in a panel.
#[derive(Clone, Debug)]
pub enum PanelRow {
    /// Empty line (blank row within border and padding).
    Empty,
    /// Horizontal separator between sections.
    Separator,
    /// Single line of text with alignment.
    Single {
        /// The text content.
        text: String,
        /// How to align the text within `content_width`.
        align: TextAlignment,
    },
    /// Two-column row with independent alignment for each column.
    TwoCol {
        /// Left column content.
        left: String,
        /// Right column content.
        right: String,
        /// Alignment for the left column.
        left_align: TextAlignment,
        /// Alignment for the right column.
        right_align: TextAlignment,
    },
    /// Four-column row with independent alignment for each column.
    FourCol {
        /// Contents of all four columns.
        cells: [String; 4],
        /// Alignments for all four columns.
        aligns: [TextAlignment; 4],
    },
}

/// Builder for creating panel content with borders, padding, and layouts.
///
/// # Width semantics
///
/// `content_width` is the drawable inner area — the number of characters
/// available for text. Border and padding are **additive**:
///
/// ```text
/// total_width = border(1) + padding.left + content_width + padding.right + border(1)
/// ```
///
/// Adding or removing border/padding never changes `content_width`.
/// All alignment is relative to `content_width`.
pub struct PanelBuilder {
    content_width: usize,
    content_height: Option<usize>,
    padding: Padding,
    title: Option<String>,
    title_alignment: TitleAlignment,
    border: Option<BorderConfig>,
    border_color: Option<RgbColor>,
    column_layout: ColumnLayout,
    rows: Vec<PanelRow>,
    style: PanelStyle,
}

impl PanelBuilder {
    /// Creates a new PanelBuilder with the given content dimensions.
    ///
    /// # Parameters
    /// - `content_width`: The drawable inner area (before padding and border).
    /// - `content_height`: Fixed row count (`None` = dynamic, render all rows).
    pub fn new(content_width: usize, content_height: Option<usize>) -> Self {
        Self {
            content_width,
            content_height,
            padding: Padding::default(),
            title: None,
            title_alignment: TitleAlignment::Center,
            border: Some(BorderConfig::solid_blocks()),
            border_color: None,
            column_layout: ColumnLayout::Single,
            rows: Vec::new(),
            style: PanelStyle::default(),
        }
    }

    /// Sets the padding for all sides.
    pub fn with_padding(mut self, padding: Padding) -> Self {
        self.padding = padding;
        self
    }

    /// Sets the panel title (placed on the top border line, centered by default).
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Sets the title alignment.
    pub fn with_title_alignment(mut self, alignment: TitleAlignment) -> Self {
        self.title_alignment = alignment;
        self
    }

    /// Sets the border configuration.
    pub fn with_border(mut self, chars: BorderConfig) -> Self {
        self.border = Some(chars);
        self
    }

    /// Removes the border.
    pub fn with_no_border(mut self) -> Self {
        self.border = None;
        self
    }

    /// Sets the border color.
    pub fn with_border_color(mut self, color: RgbColor) -> Self {
        self.border_color = Some(color);
        self
    }

    /// Sets the column layout.
    pub fn with_columns(mut self, layout: ColumnLayout) -> Self {
        self.column_layout = layout;
        self
    }

    /// Sets the panel style.
    pub fn with_style(mut self, style: PanelStyle) -> Self {
        self.style = style;
        self
    }

    /// Adds an empty (blank) row.
    pub fn add_empty(mut self) -> Self {
        self.rows.push(PanelRow::Empty);
        self
    }

    /// Adds a horizontal separator row.
    pub fn add_separator(mut self) -> Self {
        self.rows.push(PanelRow::Separator);
        self
    }

    /// Adds a single text row with the given alignment.
    pub fn add_single(mut self, text: impl Into<String>, align: TextAlignment) -> Self {
        self.rows.push(PanelRow::Single {
            text: text.into(),
            align,
        });
        self
    }

    /// Adds a two-column row with independent alignment for each side.
    pub fn add_two_col(
        mut self,
        left: impl Into<String>,
        right: impl Into<String>,
        left_align: TextAlignment,
        right_align: TextAlignment,
    ) -> Self {
        self.rows.push(PanelRow::TwoCol {
            left: left.into(),
            right: right.into(),
            left_align,
            right_align,
        });
        self
    }

    /// Adds a four-column row with independent alignment for each cell.
    pub fn add_four_col(mut self, cells: [String; 4], aligns: [TextAlignment; 4]) -> Self {
        self.rows.push(PanelRow::FourCol { cells, aligns });
        self
    }

    /// Replaces all rows with the given pre-built row list.
    pub fn with_rows(mut self, rows: Vec<PanelRow>) -> Self {
        self.rows = rows;
        self
    }

    // ── Query helpers ─────────────────────────────────────────────────────────

    /// Returns the total rendered width (border + padding + content + padding + border).
    pub fn total_width(&self) -> usize {
        let border_width = if self.border.is_some() { 2 } else { 0 };
        border_width + self.padding.left + self.content_width + self.padding.right
    }

    /// Returns the total rendered height for fixed-height panels.
    pub fn total_height(&self) -> usize {
        let border_height = if self.border.is_some() { 2 } else { 0 };
        let content_rows = self.content_height.unwrap_or(self.rows.len());
        border_height + self.padding.top + content_rows + self.padding.bottom
    }

    /// Returns the content width (inner drawable area, before padding and border).
    pub fn content_width(&self) -> usize {
        self.content_width
    }

    /// Returns the content width (alias for callers using the old name).
    pub fn inner_width(&self) -> usize {
        self.content_width
    }

    /// Returns the total width (alias for callers using the old name).
    pub fn width(&self) -> usize {
        self.total_width()
    }

    // ── Public render helpers (usable without consuming the builder) ──────────

    /// Renders a single text row with border and padding.
    pub fn render_single_row(&self, text: &str, align: TextAlignment) -> String {
        let aligned = self.align_text(text, self.content_width, align);
        self.wrap_content_line(&aligned)
    }

    /// Renders a two-column row with border and padding.
    pub fn render_two_col_row(
        &self,
        left: &str,
        right: &str,
        la: TextAlignment,
        ra: TextAlignment,
    ) -> String {
        let (lw, rw) = self.column_layout.column_ratios(self.content_width);
        let left_str = self.align_text(left, lw, la);
        let right_str = self.align_text(right, rw, ra);
        self.wrap_content_line(&format!("{}{}", left_str, right_str))
    }

    /// Renders a four-column row with border and padding.
    pub fn render_four_col_row(&self, cells: &[&str; 4], aligns: &[TextAlignment; 4]) -> String {
        let widths = ColumnLayout::four_column_ratios(self.content_width);
        let mut content = String::with_capacity(self.content_width);
        for (i, &cell) in cells.iter().enumerate() {
            content.push_str(&self.align_text(cell, widths[i], aligns[i]));
        }
        self.wrap_content_line(&content)
    }

    /// Renders an empty (blank) content line with border and padding.
    pub fn render_empty_content_line(&self) -> String {
        let inner = self.padding.left + self.content_width + self.padding.right;
        if let Some(ref border) = self.border {
            format!(
                "{}{}{}",
                border.vertical,
                " ".repeat(inner),
                border.vertical
            )
        } else {
            " ".repeat(self.total_width())
        }
    }

    /// Renders a horizontal separator line with border chars.
    pub fn render_separator_line(&self) -> String {
        let inner = self.padding.left + self.content_width + self.padding.right;
        if let Some(ref border) = self.border {
            format!(
                "{}{}{}",
                border.left_intersection,
                border.top_horizontal.to_string().repeat(inner),
                border.right_intersection
            )
        } else {
            " ".repeat(self.total_width())
        }
    }

    /// Alias for `render_separator_line`.
    pub fn render_separator(&self) -> String {
        self.render_separator_line()
    }

    /// Alias for `render_separator_line` (legacy name).
    pub fn build_separator(&self) -> String {
        self.render_separator_line()
    }

    // ── Build ─────────────────────────────────────────────────────────────────

    /// Builds the panel and returns all rendered lines.
    ///
    /// Every line has the same width (`total_width()`).
    pub fn build(self) -> Vec<String> {
        let mut lines = Vec::new();

        // Top border (with optional title embedded in the border line)
        if self.border.is_some() {
            lines.push(self.render_top_border());
        }

        // Top padding rows
        for _ in 0..self.padding.top {
            lines.push(self.render_empty_content_line());
        }

        // Content rows
        let max_rows = self.content_height.unwrap_or(self.rows.len());
        for row in self.rows.iter().take(max_rows) {
            let line = match row {
                PanelRow::Empty => self.render_empty_content_line(),
                PanelRow::Separator => self.render_separator_line(),
                PanelRow::Single { text, align } => self.render_single_row(text, *align),
                PanelRow::TwoCol {
                    left,
                    right,
                    left_align,
                    right_align,
                } => self.render_two_col_row(left, right, *left_align, *right_align),
                PanelRow::FourCol { cells, aligns } => {
                    let cell_refs: [&str; 4] = [&cells[0], &cells[1], &cells[2], &cells[3]];
                    self.render_four_col_row(&cell_refs, aligns)
                }
            };
            lines.push(line);
        }

        // Fill remaining rows if fixed height
        if let Some(h) = self.content_height {
            let rendered_content =
                lines
                    .len()
                    .saturating_sub(if self.border.is_some() { 1 } else { 0 })
                    - self.padding.top;
            for _ in rendered_content..h {
                lines.push(self.render_empty_content_line());
            }
        }

        // Bottom padding rows
        for _ in 0..self.padding.bottom {
            lines.push(self.render_empty_content_line());
        }

        // Bottom border
        if self.border.is_some() {
            lines.push(self.render_bottom_border());
        }

        lines
    }

    // ── Private helpers ───────────────────────────────────────────────────────

    fn render_top_border(&self) -> String {
        let border = self.border.as_ref().unwrap();
        let inner = self.padding.left + self.content_width + self.padding.right;

        if let Some(ref title) = self.title {
            let title_str = format!(" {} ", title);
            let title_len = title_str.chars().count();
            let remaining = inner.saturating_sub(title_len);
            let (left_fill, right_fill) = match self.title_alignment {
                TitleAlignment::Center => {
                    let l = remaining / 2;
                    (l, remaining - l)
                }
                TitleAlignment::Left => (0, remaining),
            };
            format!(
                "{}{}{}{}{}",
                border.top_left,
                border.top_horizontal.to_string().repeat(left_fill),
                title_str,
                border.top_horizontal.to_string().repeat(right_fill),
                border.top_right
            )
        } else {
            format!(
                "{}{}{}",
                border.top_left,
                border.top_horizontal.to_string().repeat(inner),
                border.top_right
            )
        }
    }

    fn render_bottom_border(&self) -> String {
        let border = self.border.as_ref().unwrap();
        let inner = self.padding.left + self.content_width + self.padding.right;
        format!(
            "{}{}{}",
            border.bottom_left,
            border.bottom_horizontal.to_string().repeat(inner),
            border.bottom_right
        )
    }

    fn wrap_content_line(&self, content: &str) -> String {
        if let Some(ref border) = self.border {
            format!(
                "{}{}{}{}{}",
                border.vertical,
                " ".repeat(self.padding.left),
                content,
                " ".repeat(self.padding.right),
                border.vertical
            )
        } else {
            format!(
                "{}{}{}",
                " ".repeat(self.padding.left),
                content,
                " ".repeat(self.padding.right)
            )
        }
    }

    fn align_text(&self, text: &str, width: usize, alignment: TextAlignment) -> String {
        let text_len = text.chars().count();
        if text_len >= width {
            // Truncate to fit
            return text.chars().take(width).collect();
        }
        let remaining = width - text_len;
        match alignment {
            TextAlignment::Left => format!("{}{}", text, " ".repeat(remaining)),
            TextAlignment::Center => {
                let left = remaining / 2;
                let right = remaining - left;
                format!("{}{}{}", " ".repeat(left), text, " ".repeat(right))
            }
            TextAlignment::Right => format!("{}{}", " ".repeat(remaining), text),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_total_width_with_border_and_padding() {
        let panel = PanelBuilder::new(44, None).with_padding(Padding::new(1, 1, 2, 2));
        // border(1) + left_pad(2) + content(44) + right_pad(2) + border(1) = 50
        assert_eq!(panel.total_width(), 50);
    }

    #[test]
    fn test_total_width_no_border() {
        let panel = PanelBuilder::new(44, None)
            .with_padding(Padding::new(1, 1, 2, 2))
            .with_no_border();
        // left_pad(2) + content(44) + right_pad(2) = 48
        assert_eq!(panel.total_width(), 48);
    }

    #[test]
    fn test_build_lines_all_same_width() {
        let lines = PanelBuilder::new(26, None)
            .with_padding(Padding::new(1, 1, 2, 2))
            .with_title("TEST")
            .add_single("Hello world", TextAlignment::Left)
            .add_separator()
            .add_empty()
            .add_single("Right aligned", TextAlignment::Right)
            .build();

        let expected_width = 1 + 2 + 26 + 2 + 1; // 32
        for (i, line) in lines.iter().enumerate() {
            assert_eq!(
                line.chars().count(),
                expected_width,
                "Line {} has wrong width: '{}'",
                i,
                line
            );
        }
    }

    #[test]
    fn test_top_border_with_title() {
        let panel = PanelBuilder::new(26, None)
            .with_padding(Padding::new(1, 1, 2, 2))
            .with_title("STATS");
        let border_line = panel.render_top_border();
        assert!(
            border_line.starts_with('▀'),
            "Should start with solid block"
        );
        assert!(border_line.ends_with('▀'), "Should end with solid block");
        assert!(border_line.contains("STATS"), "Should contain title");
        assert_eq!(border_line.chars().count(), 32);
    }

    #[test]
    fn test_separator_line_width() {
        let panel = PanelBuilder::new(26, None).with_padding(Padding::new(1, 1, 2, 2));
        let sep = panel.render_separator_line();
        assert!(sep.starts_with('▌'));
        assert!(sep.ends_with('▐'));
        assert_eq!(sep.chars().count(), 32);
    }

    #[test]
    fn test_four_column_ratios() {
        let widths = ColumnLayout::four_column_ratios(54);
        assert_eq!(widths.iter().sum::<usize>(), 54);
        // At least approximately equal
        for w in widths {
            assert!(w >= 13 && w <= 14);
        }
    }

    #[test]
    fn test_four_col_row_width() {
        let panel = PanelBuilder::new(54, None)
            .with_padding(Padding::new(1, 1, 2, 2))
            .with_columns(ColumnLayout::FourEqual)
            .add_four_col(
                [
                    "p, Space".to_string(),
                    "Pause".to_string(),
                    "c, C".to_string(),
                    "Palette".to_string(),
                ],
                [
                    TextAlignment::Left,
                    TextAlignment::Left,
                    TextAlignment::Left,
                    TextAlignment::Left,
                ],
            );
        let lines = panel.build();
        let expected = 1 + 2 + 54 + 2 + 1; // 60
        for line in &lines {
            assert_eq!(line.chars().count(), expected);
        }
    }

    #[test]
    fn test_border_is_additive() {
        let cw = 44usize;
        let panel = PanelBuilder::new(cw, None).with_padding(Padding::new(1, 1, 2, 2));
        // total = border(2) + padding(4) + content(44) = 50
        assert_eq!(panel.total_width(), cw + 2 + 4);
    }

    #[test]
    fn test_solid_block_default_border() {
        let lines = PanelBuilder::new(10, None).build();
        assert!(
            lines[0].starts_with('▀'),
            "Top border should be solid block ▀"
        );
        assert!(
            lines.last().unwrap().starts_with('▄'),
            "Bottom border should be solid block ▄"
        );
    }

    #[test]
    fn test_two_col_row_width() {
        let panel = PanelBuilder::new(44, None)
            .with_padding(Padding::new(1, 1, 2, 2))
            .with_columns(ColumnLayout::TwoEqual)
            .add_two_col(
                "Left content",
                "Right content",
                TextAlignment::Left,
                TextAlignment::Right,
            );
        let lines = panel.build();
        let expected = 50;
        for line in &lines {
            assert_eq!(line.chars().count(), expected);
        }
    }
}
