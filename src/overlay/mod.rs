//! Overlay system module.
//!
//! Provides centralized state management, standardized interfaces,
//! and input handling for terminal overlays.

pub mod input;
pub mod layout;
pub mod state;
pub mod trait_impl;

pub use input::{KeyHint, OverlayInputHandler};
pub use layout::{ContentId, OverlayLayout, RowType};
pub use state::OverlayState;
pub use trait_impl::Overlay;

use crate::render::panel::RenderedOverlay;

/// Types of overlays that can be displayed.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum OverlayType {
    /// Help overlay (legacy, now part of controls).
    Help,
    /// Controls overlay.
    Controls,
    /// Dashboard overlay (merged stats + info).
    Dashboard,
    /// Config browser overlay.
    ConfigBrowser,
    /// Config save dialog overlay.
    ConfigSave,
    /// Preset comparison overlay.
    PresetComparison,
    /// Keyboard hints overlay.
    KeyboardHints,
    /// Palette editor overlay.
    PaletteEditor,
    /// Pause badge overlay.
    PauseBadge,
    /// Pause logo overlay.
    PauseLogo,
    /// Notification toast overlay.
    Notification,
}

impl OverlayType {
    /// Returns true if this overlay type captures keyboard input.
    pub fn captures_input(self) -> bool {
        matches!(
            self,
            OverlayType::Controls
                | OverlayType::PaletteEditor
                | OverlayType::ConfigBrowser
                | OverlayType::ConfigSave
        )
    }

    /// Returns true if this overlay should close on Escape key.
    pub fn closes_on_escape(self) -> bool {
        !matches!(self, OverlayType::PauseBadge | OverlayType::PauseLogo)
    }

    /// Returns the z-order priority (higher = rendered on top).
    pub fn z_order(self) -> u8 {
        match self {
            OverlayType::PauseLogo => 0,
            OverlayType::PauseBadge => 1,
            OverlayType::Controls => 2,
            OverlayType::Dashboard => 3,
            OverlayType::Notification => 4,
            OverlayType::Help => 10,
            OverlayType::ConfigBrowser => 10,
            OverlayType::ConfigSave => 10,
            OverlayType::KeyboardHints => 10,
            OverlayType::PresetComparison => 10,
            OverlayType::PaletteEditor => 10,
        }
    }
}

/// Collection of all overlay data for rendering.
#[derive(Default)]
pub struct OverlayCollection<'a> {
    /// Pause screen logo overlay.
    pub pause_logo: Option<(&'a RenderedOverlay, usize, usize)>,
    /// Pause screen badge overlay.
    pub pause_badge: Option<(&'a RenderedOverlay, usize, usize)>,
    /// Controls help overlay.
    pub controls: Option<(&'a RenderedOverlay, usize, usize)>,
    /// Dashboard overlay.
    pub dashboard: Option<(&'a RenderedOverlay, usize, usize)>,
    /// Notification overlay.
    pub notification: Option<(&'a RenderedOverlay, usize, usize)>,
    /// Config browser overlay.
    pub config_browser: Option<(&'a RenderedOverlay, usize, usize)>,
    /// Config save overlay.
    pub config_save: Option<(&'a RenderedOverlay, usize, usize)>,
    /// Keyboard hints overlay.
    pub keyboard_hints: Option<(&'a RenderedOverlay, usize, usize)>,
    /// Preset comparison overlay.
    pub preset_comparison: Option<(&'a RenderedOverlay, usize, usize)>,
    /// Palette editor overlay.
    pub palette_editor: Option<(&'a RenderedOverlay, usize, usize)>,
}

type OverlayEntry<'a> = (OverlayType, Option<(&'a RenderedOverlay, usize, usize)>);

impl<'a> OverlayCollection<'a> {
    /// Returns all overlays sorted by z-order.
    pub fn iter_by_z_order(&self) -> Vec<OverlayEntry<'a>> {
        let mut overlays: Vec<(OverlayType, _, u8)> = vec![
            (
                OverlayType::PauseLogo,
                self.pause_logo,
                OverlayType::PauseLogo.z_order(),
            ),
            (
                OverlayType::PauseBadge,
                self.pause_badge,
                OverlayType::PauseBadge.z_order(),
            ),
            (
                OverlayType::Controls,
                self.controls,
                OverlayType::Controls.z_order(),
            ),
            (
                OverlayType::Dashboard,
                self.dashboard,
                OverlayType::Dashboard.z_order(),
            ),
            (
                OverlayType::Notification,
                self.notification,
                OverlayType::Notification.z_order(),
            ),
            (
                OverlayType::ConfigBrowser,
                self.config_browser,
                OverlayType::ConfigBrowser.z_order(),
            ),
            (
                OverlayType::ConfigSave,
                self.config_save,
                OverlayType::ConfigSave.z_order(),
            ),
            (
                OverlayType::KeyboardHints,
                self.keyboard_hints,
                OverlayType::KeyboardHints.z_order(),
            ),
            (
                OverlayType::PresetComparison,
                self.preset_comparison,
                OverlayType::PresetComparison.z_order(),
            ),
            (
                OverlayType::PaletteEditor,
                self.palette_editor,
                OverlayType::PaletteEditor.z_order(),
            ),
        ];

        overlays.sort_by_key(|(_, _, z)| *z);
        overlays.into_iter().map(|(t, o, _)| (t, o)).collect()
    }
}
