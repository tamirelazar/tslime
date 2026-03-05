//! Layout-aware position tracking for overlays.
//!
//! Replaces fragile hardcoded row indices with semantic content identifiers,
//! making overlay layouts maintainable and self-documenting.

/// Identifies specific content elements within an overlay layout.
///
/// These identifiers are used to track where specific content appears
/// in the overlay, enabling rich text coloring and other positional
/// operations without relying on fragile hardcoded indices.
///
/// # Example
/// ```
/// use tslime::overlay::{OverlayLayout, ContentId};
///
/// let mut layout = OverlayLayout::new();
/// layout.add_content(ContentId::LightnessSlider);
/// layout.add_content(ContentId::ChromaSlider);
///
/// // Later, get the actual row index:
/// let l_row = layout.row_of(ContentId::LightnessSlider).unwrap();
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ContentId {
    // Palette Editor Content
    /// Stop selector row (diamond indicators for palette colors).
    StopSelector,
    /// Label row for color stops ("1 2 3 ... 11 ALL").
    StopLabels,
    /// Lightness slider row.
    LightnessSlider,
    /// Chroma slider row.
    ChromaSlider,
    /// Hue slider row.
    HueSlider,
    /// First hint row (arrow key indicators).
    HintArrows,
    /// Second hint row (adjust indicators).
    HintAdjust,
    /// Third hint row (Tab navigation).
    HintTab,
    /// Gradient preview strip row.
    GradientStrip,
    /// Color info row.
    ColorInfo,

    // Controls Overlay Content
    /// Tab indicator row (●/○ markers).
    TabIndicators,
    /// Tab label row (SIM, ENV, APP, etc.).
    TabLabels,
    /// Parameter row for sensor angle.
    SensorAngleRow,
    /// Parameter row for sensor distance.
    SensorDistanceRow,
    /// Parameter row for turn angle.
    TurnAngleRow,
    /// Parameter row for step size.
    StepSizeRow,
    /// Parameter row for decay factor.
    DecayRow,
    /// Parameter row for deposit amount.
    DepositRow,
    /// Parameter row for time scale.
    TimeScaleRow,
    /// Footer row with key hints.
    FooterHints,

    // Dashboard Content
    /// FPS row with progress bar.
    FpsRow,
    /// Trail usage row.
    TrailRow,
    /// Entropy row.
    EntropyRow,
    /// CPU usage row.
    CpuRow,
    /// Memory usage row.
    MemoryRow,

    // Generic Content
    /// Title row.
    Title,
    /// Separator line.
    Separator,
    /// Empty spacer row.
    Empty,
    /// Custom content with a string identifier.
    Custom(&'static str),
}

/// Types of rows in an overlay layout.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RowType {
    /// Empty spacer row.
    Empty,
    /// Row containing identifiable content.
    Content(ContentId),
    /// Separator line.
    Separator,
    /// Title row.
    Title,
}

/// Tracks the layout of an overlay with semantic content identifiers.
///
/// This struct builds a mapping between content identifiers and their
/// actual row indices, allowing overlays to reference content by semantic
/// meaning rather than fragile numeric indices.
///
/// # Example
/// ```
/// use tslime::overlay::{OverlayLayout, ContentId};
///
/// let mut layout = OverlayLayout::new();
///
/// // Build the layout
/// layout.add_empty();
/// layout.add_title();
/// layout.add_empty();
/// let slider_row = layout.add_content(ContentId::LightnessSlider);
/// layout.add_content(ContentId::ChromaSlider);
///
/// // Query the layout
/// assert_eq!(layout.row_of(ContentId::LightnessSlider), Some(slider_row));
/// assert_eq!(layout.row_count(), 5);
/// ```
#[derive(Debug, Clone)]
pub struct OverlayLayout {
    rows: Vec<RowType>,
}

impl Default for OverlayLayout {
    fn default() -> Self {
        Self::new()
    }
}

impl OverlayLayout {
    /// Creates a new empty layout.
    pub fn new() -> Self {
        Self { rows: Vec::new() }
    }

    /// Adds an empty spacer row and returns its index.
    pub fn add_empty(&mut self) -> usize {
        let idx = self.rows.len();
        self.rows.push(RowType::Empty);
        idx
    }

    /// Adds a title row and returns its index.
    pub fn add_title(&mut self) -> usize {
        let idx = self.rows.len();
        self.rows.push(RowType::Title);
        idx
    }

    /// Adds a separator row and returns its index.
    pub fn add_separator(&mut self) -> usize {
        let idx = self.rows.len();
        self.rows.push(RowType::Separator);
        idx
    }

    /// Adds a content row with the given identifier and returns its index.
    ///
    /// # Example
    /// ```
    /// use tslime::overlay::{OverlayLayout, ContentId};
    ///
    /// let mut layout = OverlayLayout::new();
    /// let slider_idx = layout.add_content(ContentId::LightnessSlider);
    /// ```
    pub fn add_content(&mut self, id: ContentId) -> usize {
        let idx = self.rows.len();
        self.rows.push(RowType::Content(id));
        idx
    }

    /// Returns the row index of the first occurrence of the given content.
    ///
    /// Returns `None` if the content is not found in the layout.
    ///
    /// # Example
    /// ```
    /// use tslime::overlay::{OverlayLayout, ContentId};
    ///
    /// let mut layout = OverlayLayout::new();
    /// layout.add_content(ContentId::LightnessSlider);
    /// layout.add_content(ContentId::ChromaSlider);
    ///
    /// assert_eq!(layout.row_of(ContentId::LightnessSlider), Some(0));
    /// assert_eq!(layout.row_of(ContentId::ChromaSlider), Some(1));
    /// assert_eq!(layout.row_of(ContentId::HueSlider), None);
    /// ```
    pub fn row_of(&self, id: ContentId) -> Option<usize> {
        self.rows
            .iter()
            .position(|row| matches!(row, RowType::Content(row_id) if *row_id == id))
    }

    /// Returns all row indices containing the given content.
    ///
    /// This is useful when the same content type appears multiple times.
    pub fn rows_of(&self, id: ContentId) -> Vec<usize> {
        self.rows
            .iter()
            .enumerate()
            .filter_map(|(idx, row)| {
                if matches!(row, RowType::Content(row_id) if *row_id == id) {
                    Some(idx)
                } else {
                    None
                }
            })
            .collect()
    }

    /// Returns the total number of rows in the layout.
    pub fn row_count(&self) -> usize {
        self.rows.len()
    }

    /// Returns the type of row at the given index.
    ///
    /// Returns `None` if the index is out of bounds.
    pub fn row_type(&self, index: usize) -> Option<RowType> {
        self.rows.get(index).copied()
    }

    /// Returns true if the layout contains the given content.
    pub fn contains(&self, id: ContentId) -> bool {
        self.row_of(id).is_some()
    }

    /// Returns an iterator over all rows.
    pub fn iter(&self) -> impl Iterator<Item = (usize, RowType)> + '_ {
        self.rows.iter().enumerate().map(|(idx, row)| (idx, *row))
    }

    /// Returns the index of the first occurrence of the given content type.
    ///
    /// Unlike `row_of` which matches specific ContentId, this matches
    /// the general category (e.g., any Slider content).
    pub fn first_of_category<F>(&self, predicate: F) -> Option<usize>
    where
        F: Fn(ContentId) -> bool,
    {
        self.rows.iter().enumerate().find_map(|(idx, row)| {
            if let RowType::Content(id) = row {
                if predicate(*id) {
                    return Some(idx);
                }
            }
            None
        })
    }
}

/// Extension trait for easier content matching.
pub trait ContentIdExt {
    /// Returns true if this is a slider content.
    fn is_slider(&self) -> bool;
    /// Returns true if this is a hint content.
    fn is_hint(&self) -> bool;
    /// Returns true if this is a parameter row.
    fn is_parameter(&self) -> bool;
}

impl ContentIdExt for ContentId {
    fn is_slider(&self) -> bool {
        matches!(
            self,
            ContentId::LightnessSlider | ContentId::ChromaSlider | ContentId::HueSlider
        )
    }

    fn is_hint(&self) -> bool {
        matches!(
            self,
            ContentId::HintArrows | ContentId::HintAdjust | ContentId::HintTab
        )
    }

    fn is_parameter(&self) -> bool {
        matches!(
            self,
            ContentId::SensorAngleRow
                | ContentId::SensorDistanceRow
                | ContentId::TurnAngleRow
                | ContentId::StepSizeRow
                | ContentId::DecayRow
                | ContentId::DepositRow
                | ContentId::TimeScaleRow
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_layout() {
        let layout = OverlayLayout::new();
        assert_eq!(layout.row_count(), 0);
        assert_eq!(layout.row_of(ContentId::LightnessSlider), None);
    }

    #[test]
    fn test_add_rows() {
        let mut layout = OverlayLayout::new();

        let empty_idx = layout.add_empty();
        let title_idx = layout.add_title();
        let sep_idx = layout.add_separator();
        let content_idx = layout.add_content(ContentId::LightnessSlider);

        assert_eq!(empty_idx, 0);
        assert_eq!(title_idx, 1);
        assert_eq!(sep_idx, 2);
        assert_eq!(content_idx, 3);
        assert_eq!(layout.row_count(), 4);
    }

    #[test]
    fn test_row_lookup() {
        let mut layout = OverlayLayout::new();

        layout.add_empty();
        layout.add_content(ContentId::LightnessSlider);
        layout.add_content(ContentId::ChromaSlider);
        layout.add_empty();
        layout.add_content(ContentId::HueSlider);

        assert_eq!(layout.row_of(ContentId::LightnessSlider), Some(1));
        assert_eq!(layout.row_of(ContentId::ChromaSlider), Some(2));
        assert_eq!(layout.row_of(ContentId::HueSlider), Some(4));
        assert_eq!(layout.row_of(ContentId::StopSelector), None);
    }

    #[test]
    fn test_contains() {
        let mut layout = OverlayLayout::new();
        layout.add_content(ContentId::LightnessSlider);

        assert!(layout.contains(ContentId::LightnessSlider));
        assert!(!layout.contains(ContentId::ChromaSlider));
    }

    #[test]
    fn test_rows_of_multiple() {
        let mut layout = OverlayLayout::new();

        layout.add_content(ContentId::HintArrows);
        layout.add_content(ContentId::HintAdjust);
        layout.add_content(ContentId::HintTab);

        let hint_rows: Vec<_> = layout
            .iter()
            .filter(|(_, row)| matches!(row, RowType::Content(id) if id.is_hint()))
            .map(|(idx, _)| idx)
            .collect();

        assert_eq!(hint_rows, vec![0, 1, 2]);
    }

    #[test]
    fn test_content_id_ext() {
        assert!(ContentId::LightnessSlider.is_slider());
        assert!(ContentId::ChromaSlider.is_slider());
        assert!(ContentId::HueSlider.is_slider());
        assert!(!ContentId::StopSelector.is_slider());

        assert!(ContentId::HintArrows.is_hint());
        assert!(!ContentId::LightnessSlider.is_hint());

        assert!(ContentId::SensorAngleRow.is_parameter());
        assert!(!ContentId::LightnessSlider.is_parameter());
    }

    #[test]
    fn test_row_type_access() {
        let mut layout = OverlayLayout::new();
        layout.add_empty();
        layout.add_title();
        layout.add_separator();
        layout.add_content(ContentId::LightnessSlider);

        assert_eq!(layout.row_type(0), Some(RowType::Empty));
        assert_eq!(layout.row_type(1), Some(RowType::Title));
        assert_eq!(layout.row_type(2), Some(RowType::Separator));
        assert_eq!(
            layout.row_type(3),
            Some(RowType::Content(ContentId::LightnessSlider))
        );
        assert_eq!(layout.row_type(99), None);
    }

    #[test]
    fn test_first_of_category() {
        let mut layout = OverlayLayout::new();

        layout.add_empty();
        layout.add_content(ContentId::SensorAngleRow);
        layout.add_content(ContentId::SensorDistanceRow);
        layout.add_content(ContentId::LightnessSlider);

        let first_param = layout.first_of_category(|id| id.is_parameter());
        let first_slider = layout.first_of_category(|id| id.is_slider());

        assert_eq!(first_param, Some(1));
        assert_eq!(first_slider, Some(3));
    }
}
