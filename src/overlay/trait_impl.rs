//! Standardized overlay trait.
//!
//! Defines a common interface that all overlays must implement,
//! enabling polymorphic handling and consistent behavior.

use crate::overlay::OverlayType;
use crate::render::panel::RenderedOverlay;
use crate::render::theme::PanelStyle;

/// Standardized interface for all terminal overlays.
///
/// This trait defines the common behavior that all overlays must implement,
/// including building their content, calculating position, and declaring
/// their input handling characteristics.
pub trait Overlay {
    /// The type of data needed to build this overlay.
    ///
    /// Varies by overlay (e.g., `RgbColor` for keyboard hints,
    /// `ControlsData` for the controls overlay).
    type BuildData;

    /// Returns the overlay type enum variant for this overlay.
    fn overlay_type() -> OverlayType;

    /// Builds the overlay content into a `RenderedOverlay` (lines plus
    /// optional rich color data) from the overlay-specific build data and
    /// the current panel style.
    fn build(data: &Self::BuildData, style: &PanelStyle) -> RenderedOverlay;

    /// Calculates the (x, y) of the overlay's top-left corner from the
    /// terminal and overlay dimensions (both (width, height)).
    ///
    /// The default implementation centers the overlay; override for custom
    /// placement (see the `*_position` helpers).
    fn calculate_position(
        term_size: (usize, usize),
        overlay_size: (usize, usize),
    ) -> (usize, usize) {
        let (term_w, term_h) = term_size;
        let (overlay_w, overlay_h) = overlay_size;

        let x = term_w.saturating_sub(overlay_w) / 2;
        let y = term_h.saturating_sub(overlay_h) / 2;

        (x, y)
    }

    /// Returns whether this overlay should close when the Escape key is pressed.
    ///
    /// The default is `true`. Non-closable overlays (like pause indicators)
    /// should return `false`.
    fn closes_on_escape() -> bool {
        true
    }

    /// Returns whether this overlay captures all keyboard input.
    ///
    /// When `true`, keyboard events are passed to this overlay's input handler
    /// first. When `false`, events flow through to global handlers.
    ///
    /// The default is `false`. Interactive overlays like Controls or PaletteEditor
    /// should return `true`.
    fn captures_input() -> bool {
        false
    }

    /// Returns the minimum terminal size required to display this overlay.
    ///
    /// The default is (0, 0), meaning no minimum size.
    /// Overlays with specific size requirements can override this.
    fn min_size() -> (usize, usize) {
        (0, 0)
    }
}

/// Helper function to calculate position for top-left placement.
pub fn top_left_position(
    _term_size: (usize, usize),
    _overlay_size: (usize, usize),
) -> (usize, usize) {
    (0, 0)
}

/// Helper function to calculate position for top-right placement.
pub fn top_right_position(
    term_size: (usize, usize),
    overlay_size: (usize, usize),
) -> (usize, usize) {
    let x = term_size.0.saturating_sub(overlay_size.0);
    (x, 0)
}

/// Helper function to calculate position for bottom-left placement.
pub fn bottom_left_position(
    term_size: (usize, usize),
    overlay_size: (usize, usize),
) -> (usize, usize) {
    let y = term_size.1.saturating_sub(overlay_size.1);
    (0, y)
}

/// Helper function to calculate position for bottom-right placement.
pub fn bottom_right_position(
    term_size: (usize, usize),
    overlay_size: (usize, usize),
) -> (usize, usize) {
    let x = term_size.0.saturating_sub(overlay_size.0);
    let y = term_size.1.saturating_sub(overlay_size.1);
    (x, y)
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestOverlay;

    impl Overlay for TestOverlay {
        type BuildData = ();

        fn overlay_type() -> OverlayType {
            OverlayType::Controls
        }

        fn build(_data: &Self::BuildData, _style: &PanelStyle) -> RenderedOverlay {
            // Minimal test overlay
            RenderedOverlay {
                lines: vec!["Test".to_string()],
                title_box: None,
                rich_lines: None,
            }
        }
    }

    #[test]
    fn test_default_position_calculation() {
        let pos = TestOverlay::calculate_position((100, 50), (60, 20));
        assert_eq!(pos, (20, 15)); // Centered: (100-60)/2=20, (50-20)/2=15
    }

    #[test]
    fn test_position_with_small_terminal() {
        let pos = TestOverlay::calculate_position((50, 20), (60, 25));
        assert_eq!(pos, (0, 0)); // Saturating sub gives 0
    }

    #[test]
    fn test_default_flags() {
        assert!(TestOverlay::closes_on_escape());
        assert!(!TestOverlay::captures_input());
        assert_eq!(TestOverlay::min_size(), (0, 0));
    }

    #[test]
    fn test_position_helpers() {
        assert_eq!(top_left_position((100, 50), (60, 20)), (0, 0));
        assert_eq!(top_right_position((100, 50), (60, 20)), (40, 0));
        assert_eq!(bottom_left_position((100, 50), (60, 20)), (0, 30));
        assert_eq!(bottom_right_position((100, 50), (60, 20)), (40, 30));
    }
}
