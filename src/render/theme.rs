use crate::render::palette::RgbColor;

/// Panel styling configuration for OpenCode-style UI.
#[derive(Clone, Debug)]
pub struct PanelStyle {
    /// Background color for panels.
    pub bg_color: RgbColor,
    /// Color for focused indicator (left edge).
    pub focus_color: RgbColor,
    /// Color for unfocused indicator (left edge).
    pub unfocus_color: RgbColor,
    /// Primary text color (for main content).
    pub text_primary: RgbColor,
    /// Secondary text color (for labels, hints).
    pub text_secondary: RgbColor,
    /// Border/frame color.
    pub border_color: RgbColor,
    /// Width of the focus indicator (in characters).
    pub indicator_width: usize,
    /// Status bar background color.
    pub status_bar_bg: RgbColor,
    /// Notification background color (default/info level).
    pub notification_bg: RgbColor,
    /// Vertical padding (empty lines at top/bottom of panels).
    pub vertical_padding: usize,
    /// Accent color for info-level notifications and highlights.
    pub accent_info: RgbColor,
    /// Accent color for success-level notifications.
    pub accent_success: RgbColor,
    /// Accent color for warning-level notifications.
    pub accent_warning: RgbColor,
    /// Accent color for error-level notifications.
    pub accent_error: RgbColor,
    /// Accent color for active/selected tab or category.
    pub accent_active: RgbColor,
    /// Muted color for inactive tabs and dimmed content.
    pub muted: RgbColor,
    /// Color for modified/changed parameter markers.
    pub accent_modified: RgbColor,
}

/// Gruvbox Dark-inspired panel style.
pub const GRUVBOX_DARK: PanelStyle = PanelStyle {
    bg_color: RgbColor {
        r: 40,
        g: 40,
        b: 40,
    },
    focus_color: RgbColor {
        r: 250,
        g: 189,
        b: 47,
    },
    unfocus_color: RgbColor {
        r: 120,
        g: 95,
        b: 38,
    },
    text_primary: RgbColor {
        r: 235,
        g: 219,
        b: 178,
    },
    text_secondary: RgbColor {
        r: 185,
        g: 170,
        b: 140,
    },
    border_color: RgbColor {
        r: 105,
        g: 99,
        b: 88,
    }, // Warm gray — subtle amber tint, very low saturation
    indicator_width: 0,
    status_bar_bg: RgbColor {
        r: 16,
        g: 18,
        b: 19,
    },
    notification_bg: RgbColor {
        r: 33,
        g: 38,
        b: 35,
    },
    vertical_padding: 0,
    accent_info: RgbColor {
        r: 131,
        g: 165,
        b: 152,
    }, // Gruvbox aqua
    accent_success: RgbColor {
        r: 184,
        g: 187,
        b: 38,
    }, // Gruvbox green
    accent_warning: RgbColor {
        r: 250,
        g: 189,
        b: 47,
    }, // Gruvbox yellow
    accent_error: RgbColor {
        r: 251,
        g: 73,
        b: 52,
    }, // Gruvbox red
    accent_active: RgbColor {
        r: 250,
        g: 189,
        b: 47,
    }, // Gruvbox yellow (same as focus)
    muted: RgbColor {
        r: 102,
        g: 92,
        b: 84,
    }, // Gruvbox dark4
    accent_modified: RgbColor {
        r: 254,
        g: 128,
        b: 25,
    }, // Gruvbox orange
};

/// Slime Mold Dark — a bioluminescent palette inspired by Physarum polycephalum.
///
/// Deep blue-black backgrounds with electric green and cyan highlights
/// evoke the glowing, living quality of the simulated organism.
pub const SLIME_DARK: PanelStyle = PanelStyle {
    bg_color: RgbColor {
        r: 14,
        g: 18,
        b: 22,
    }, // Near-black with blue tint
    focus_color: RgbColor {
        r: 57,
        g: 211,
        b: 83,
    }, // Bioluminescent green
    unfocus_color: RgbColor {
        r: 28,
        g: 85,
        b: 46,
    }, // Slightly lighter dim green
    text_primary: RgbColor {
        r: 195,
        g: 232,
        b: 211,
    }, // Soft green-white
    text_secondary: RgbColor {
        r: 108,
        g: 168,
        b: 128,
    }, // Brighter green-gray
    border_color: RgbColor {
        r: 38,
        g: 115,
        b: 62,
    }, // Visible bioluminescent green
    indicator_width: 0,
    status_bar_bg: RgbColor { r: 8, g: 12, b: 14 }, // Very dark background for status bar
    notification_bg: RgbColor {
        r: 18,
        g: 30,
        b: 24,
    }, // Dark green-tinted notification bg
    vertical_padding: 0,
    accent_info: RgbColor {
        r: 0,
        g: 188,
        b: 212,
    }, // Cyan (bioluminescent)
    accent_success: RgbColor {
        r: 57,
        g: 211,
        b: 83,
    }, // Electric green
    accent_warning: RgbColor {
        r: 255,
        g: 183,
        b: 0,
    }, // Amber
    accent_error: RgbColor {
        r: 255,
        g: 71,
        b: 87,
    }, // Coral red
    accent_active: RgbColor {
        r: 57,
        g: 211,
        b: 83,
    }, // Electric green for active tab
    muted: RgbColor {
        r: 42,
        g: 65,
        b: 52,
    }, // Very muted green-gray
    accent_modified: RgbColor {
        r: 255,
        g: 183,
        b: 0,
    }, // Amber for modified values
};

impl Default for PanelStyle {
    fn default() -> Self {
        GRUVBOX_DARK
    }
}
