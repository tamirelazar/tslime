//! Centralized overlay state management.
//!
//! Provides a single source of truth for overlay visibility,
//! eliminating the scattered boolean flags pattern.

use crate::overlay::OverlayType;
use crate::render::palette_editor::PaletteEditorState;

/// Centralized state for all overlays.
///
/// This struct provides a single source of truth for overlay visibility,
/// replacing the scattered boolean flags (show_controls, show_dashboard, etc.)
/// that previously existed in RuntimeState.
///
/// # Example
/// ```
/// use tslime::overlay::{OverlayState, OverlayType};
///
/// let mut state = OverlayState::default();
///
/// // Open controls overlay
/// state.open(OverlayType::Controls);
/// assert!(state.is_open(OverlayType::Controls));
///
/// // Opening another overlay closes the first (mutual exclusivity)
/// state.open(OverlayType::Dashboard);
/// assert!(!state.is_open(OverlayType::Controls));
/// assert!(state.is_open(OverlayType::Dashboard));
/// ```
#[derive(Debug, Clone, Default)]
pub struct OverlayState {
    /// Which overlay is currently visible (mutually exclusive).
    /// None means no overlay is open.
    active: Option<OverlayType>,

    /// Palette editor sub-state.
    /// This is separate because it has complex internal state (editing mode, selected color, etc.)
    pub palette_editor: Option<PaletteEditorState>,
}

impl OverlayState {
    /// Creates a new overlay state with no active overlay.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns true if the specified overlay type is currently open.
    ///
    /// # Example
    /// ```
    /// use tslime::overlay::{OverlayState, OverlayType};
    ///
    /// let state = OverlayState::default();
    /// assert!(!state.is_open(OverlayType::Controls));
    /// ```
    pub fn is_open(&self, overlay: OverlayType) -> bool {
        self.active == Some(overlay)
    }

    /// Returns the currently active overlay type, if any.
    pub fn active(&self) -> Option<OverlayType> {
        self.active
    }

    /// Opens the specified overlay, closing any currently open overlay.
    ///
    /// This enforces mutual exclusivity - only one overlay can be open at a time.
    /// If you need to open an overlay without closing others, use `open_exclusive`.
    ///
    /// # Example
    /// ```
    /// use tslime::overlay::{OverlayState, OverlayType};
    ///
    /// let mut state = OverlayState::default();
    /// state.open(OverlayType::Controls);
    /// state.open(OverlayType::Dashboard);
    /// // Controls was automatically closed
    /// assert!(!state.is_open(OverlayType::Controls));
    /// ```
    pub fn open(&mut self, overlay: OverlayType) {
        self.active = Some(overlay);
    }

    /// Closes the currently active overlay.
    ///
    /// # Example
    /// ```
    /// use tslime::overlay::{OverlayState, OverlayType};
    ///
    /// let mut state = OverlayState::default();
    /// state.open(OverlayType::Controls);
    /// state.close();
    /// assert!(!state.any_open());
    /// ```
    pub fn close(&mut self) {
        self.active = None;
    }

    /// Toggles the specified overlay.
    ///
    /// If the overlay is already open, it will be closed.
    /// If it's closed, it will be opened (and any other overlay closed).
    ///
    /// # Example
    /// ```
    /// use tslime::overlay::{OverlayState, OverlayType};
    ///
    /// let mut state = OverlayState::default();
    /// state.toggle(OverlayType::Controls);
    /// assert!(state.is_open(OverlayType::Controls));
    /// state.toggle(OverlayType::Controls);
    /// assert!(!state.any_open());
    /// ```
    pub fn toggle(&mut self, overlay: OverlayType) {
        if self.is_open(overlay) {
            self.close();
        } else {
            self.open(overlay);
        }
    }

    /// Returns true if any overlay is currently open.
    ///
    /// # Example
    /// ```
    /// use tslime::overlay::{OverlayState, OverlayType};
    ///
    /// let mut state = OverlayState::default();
    /// assert!(!state.any_open());
    /// state.open(OverlayType::Controls);
    /// assert!(state.any_open());
    /// ```
    pub fn any_open(&self) -> bool {
        self.active.is_some()
    }

    /// Opens the palette editor with the given initial state.
    ///
    /// This is a convenience method that handles the palette_editor state
    /// in addition to setting the active overlay.
    pub fn open_palette_editor(&mut self, state: PaletteEditorState) {
        self.palette_editor = Some(state);
        self.open(OverlayType::PaletteEditor);
    }

    /// Closes the palette editor and clears its state.
    pub fn close_palette_editor(&mut self) {
        self.palette_editor = None;
        if self.is_open(OverlayType::PaletteEditor) {
            self.close();
        }
    }

    /// Returns true if the palette editor is open.
    pub fn is_palette_editor_open(&self) -> bool {
        self.is_open(OverlayType::PaletteEditor) && self.palette_editor.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_state() {
        let state = OverlayState::default();
        assert!(!state.any_open());
        assert_eq!(state.active(), None);
    }

    #[test]
    fn test_open_and_close() {
        let mut state = OverlayState::default();

        state.open(OverlayType::Controls);
        assert!(state.is_open(OverlayType::Controls));
        assert!(state.any_open());

        state.close();
        assert!(!state.is_open(OverlayType::Controls));
        assert!(!state.any_open());
    }

    #[test]
    fn test_mutual_exclusivity() {
        let mut state = OverlayState::default();

        state.open(OverlayType::Controls);
        assert!(state.is_open(OverlayType::Controls));

        state.open(OverlayType::Dashboard);
        assert!(!state.is_open(OverlayType::Controls));
        assert!(state.is_open(OverlayType::Dashboard));
    }

    #[test]
    fn test_toggle() {
        let mut state = OverlayState::default();

        // Toggle on
        state.toggle(OverlayType::Controls);
        assert!(state.is_open(OverlayType::Controls));

        // Toggle off
        state.toggle(OverlayType::Controls);
        assert!(!state.is_open(OverlayType::Controls));

        // Toggle different overlay closes first
        state.toggle(OverlayType::Controls);
        state.toggle(OverlayType::Dashboard);
        assert!(!state.is_open(OverlayType::Controls));
        assert!(state.is_open(OverlayType::Dashboard));
    }

    #[test]
    fn test_is_open_false_for_different_overlay() {
        let mut state = OverlayState::default();
        state.open(OverlayType::Controls);

        assert!(state.is_open(OverlayType::Controls));
        assert!(!state.is_open(OverlayType::Dashboard));
        assert!(!state.is_open(OverlayType::KeyboardHints));
    }
}
