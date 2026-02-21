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
    /// Notification background color.
    pub notification_bg: RgbColor,
    /// Vertical padding (empty lines at top/bottom of panels).
    pub vertical_padding: usize,
}

/// Default Gruvbox Dark-inspired panel style.
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
        r: 146,
        g: 131,
        b: 116,
    },
    text_primary: RgbColor {
        r: 235,
        g: 219,
        b: 178,
    },
    text_secondary: RgbColor {
        r: 168,
        g: 153,
        b: 132,
    },
    border_color: RgbColor {
        r: 146,
        g: 131,
        b: 116,
    },
    indicator_width: 0,
    status_bar_bg: RgbColor {
        r: 29,
        g: 32,
        b: 33,
    },
    notification_bg: RgbColor {
        r: 33,
        g: 38,
        b: 35,
    },
    vertical_padding: 0,
};

impl Default for PanelStyle {
    fn default() -> Self {
        GRUVBOX_DARK
    }
}
