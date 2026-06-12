//! Overlay system module.
//!
//! Provides centralized state management, standardized interfaces,
//! and input handling for terminal overlays.

pub mod input;
pub mod input_manager;
pub mod layout;
pub mod state;
pub mod trait_impl;

pub use input::{KeyHint, OverlayInputHandler};
pub use input_manager::{OverlayInputManager, OverlayInputResult};
pub use layout::{ContentId, OverlayLayout, RowType};
pub use state::OverlayState;
pub use trait_impl::Overlay;

use crate::render::panel::RenderedOverlay;
use crossterm::event::KeyModifiers;

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

    /// Returns the keybind character(s) that toggle this overlay.
    pub fn toggle_keys(self) -> &'static [char] {
        match self {
            OverlayType::Controls => &['h', 'H'],
            OverlayType::KeyboardHints => &['?'],
            OverlayType::Dashboard => &['\\', '|'],
            OverlayType::PaletteEditor => &['p', 'P', '/'],
            OverlayType::ConfigBrowser => &[], // Ctrl+L/B only
            OverlayType::ConfigSave => &[],    // Ctrl+S only
            _ => &[],
        }
    }

    /// Returns true if this key should toggle this overlay.
    pub fn is_toggle_key(self, key: char, modifiers: KeyModifiers) -> bool {
        match self {
            OverlayType::ConfigBrowser => {
                modifiers.contains(KeyModifiers::CONTROL) && matches!(key, 'l' | 'L' | 'b' | 'B')
            }
            OverlayType::ConfigSave => {
                modifiers.contains(KeyModifiers::CONTROL) && matches!(key, 's' | 'S')
            }
            _ => modifiers == KeyModifiers::NONE && self.toggle_keys().contains(&key),
        }
    }

    /// Returns true if this overlay blocks all other keys when open.
    pub fn blocks_other_keys(self) -> bool {
        self.captures_input()
    }

    /// Returns true if this overlay handles Escape key internally.
    /// For these overlays, Escape processing is delegated to their specialized handler.
    pub fn handles_escape_internally(self) -> bool {
        matches!(self, OverlayType::PaletteEditor)
    }

    /// Returns the overlay type that would be toggled by this key, if any.
    pub fn from_toggle_key(key: char, modifiers: KeyModifiers) -> Option<OverlayType> {
        [
            OverlayType::Controls,
            OverlayType::KeyboardHints,
            OverlayType::Dashboard,
            OverlayType::PaletteEditor,
            OverlayType::ConfigBrowser,
            OverlayType::ConfigSave,
        ]
        .into_iter()
        .find(|&overlay| overlay.is_toggle_key(key, modifiers))
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
