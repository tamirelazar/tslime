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
    /// FPS good (≥55): displayed green.
    pub accent_fps_good: RgbColor,
    /// FPS warning (≥25, <55): displayed amber.
    pub accent_fps_warn: RgbColor,
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
    accent_fps_good: RgbColor {
        r: 142,
        g: 192,
        b: 124,
    }, // Gruvbox green
    accent_fps_warn: RgbColor {
        r: 215,
        g: 153,
        b: 33,
    }, // Amber
};

/// Slime Mold Dark — a bioluminescent palette inspired by Physarum polycephalum.
///
/// Deep blue-black backgrounds with electric green and cyan highlights
/// evoke the glowing, living quality of the simulated organism.
pub const SLIME_DARK: PanelStyle = PanelStyle {
    bg_color: RgbColor {
        r: 11,
        g: 16,
        b: 14,
    }, // Near-black with deep green tint
    focus_color: RgbColor {
        r: 57,
        g: 211,
        b: 83,
    }, // Bioluminescent green
    unfocus_color: RgbColor {
        r: 24,
        g: 72,
        b: 40,
    }, // Dim green
    text_primary: RgbColor {
        r: 190,
        g: 230,
        b: 205,
    }, // Soft green-white
    text_secondary: RgbColor {
        r: 100,
        g: 155,
        b: 118,
    }, // Muted green-gray
    border_color: RgbColor {
        r: 26,
        g: 56,
        b: 36,
    }, // Very subtle dark green — harmonious on all panel backgrounds
    indicator_width: 0,
    status_bar_bg: RgbColor { r: 6, g: 10, b: 8 }, // Near-black for status bar
    notification_bg: RgbColor {
        r: 15,
        g: 26,
        b: 19,
    }, // Slightly lighter near-black with green tint
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
        r: 36,
        g: 58,
        b: 44,
    }, // Very muted green-gray
    accent_modified: RgbColor {
        r: 255,
        g: 183,
        b: 0,
    }, // Amber for modified values
    accent_fps_good: RgbColor {
        r: 57,
        g: 211,
        b: 83,
    }, // Electric green (same as focus)
    accent_fps_warn: RgbColor {
        r: 255,
        g: 183,
        b: 0,
    }, // Amber (same as accent_warning)
};

/// Nord — Arctic, origin-aligned color palette.
///
/// Cool blue-gray tones drawn from the Nord color system, evoking ice, tundra,
/// and Nordic landscapes. Uses Polar Night backgrounds with Frost blue accents
/// and Aurora status colors.
pub const NORD: PanelStyle = PanelStyle {
    bg_color: RgbColor {
        r: 46,
        g: 52,
        b: 64,
    }, // nord1 — polar night
    focus_color: RgbColor {
        r: 136,
        g: 192,
        b: 208,
    }, // nord8 — frost blue
    unfocus_color: RgbColor {
        r: 67,
        g: 76,
        b: 94,
    }, // nord2 — slightly lighter polar night
    text_primary: RgbColor {
        r: 236,
        g: 239,
        b: 244,
    }, // nord6 — snow storm (bright)
    text_secondary: RgbColor {
        r: 216,
        g: 222,
        b: 233,
    }, // nord5 — snow storm (mid)
    border_color: RgbColor {
        r: 59,
        g: 66,
        b: 82,
    }, // nord3 — subtle polar night border
    indicator_width: 0,
    status_bar_bg: RgbColor {
        r: 36,
        g: 41,
        b: 51,
    }, // nord0 — darkest polar night
    notification_bg: RgbColor {
        r: 42,
        g: 47,
        b: 59,
    }, // between nord0 and nord1
    vertical_padding: 0,
    accent_info: RgbColor {
        r: 136,
        g: 192,
        b: 208,
    }, // nord8 — frost blue
    accent_success: RgbColor {
        r: 163,
        g: 190,
        b: 140,
    }, // nord14 — aurora green
    accent_warning: RgbColor {
        r: 235,
        g: 203,
        b: 139,
    }, // nord13 — aurora yellow
    accent_error: RgbColor {
        r: 191,
        g: 97,
        b: 106,
    }, // nord11 — aurora red
    accent_active: RgbColor {
        r: 136,
        g: 192,
        b: 208,
    }, // nord8 — frost blue (same as focus)
    muted: RgbColor {
        r: 76,
        g: 86,
        b: 106,
    }, // nord3+4 blend
    accent_modified: RgbColor {
        r: 208,
        g: 135,
        b: 112,
    }, // nord12 — aurora orange
    accent_fps_good: RgbColor {
        r: 163,
        g: 190,
        b: 140,
    }, // aurora green
    accent_fps_warn: RgbColor {
        r: 235,
        g: 203,
        b: 139,
    }, // aurora yellow
};

/// Catppuccin Mocha — a warm dark lavender theme.
///
/// Based on the Catppuccin Mocha palette: rich purples and pinks on a dark
/// warm background, with soft pastel accents.
pub const CATPPUCCIN_MOCHA: PanelStyle = PanelStyle {
    bg_color: RgbColor {
        r: 30,
        g: 30,
        b: 46,
    }, // base
    focus_color: RgbColor {
        r: 203,
        g: 166,
        b: 247,
    }, // mauve
    unfocus_color: RgbColor {
        r: 62,
        g: 53,
        b: 86,
    }, // surface0 with mauve tint
    text_primary: RgbColor {
        r: 205,
        g: 214,
        b: 244,
    }, // text
    text_secondary: RgbColor {
        r: 166,
        g: 173,
        b: 200,
    }, // subtext1
    border_color: RgbColor {
        r: 54,
        g: 58,
        b: 79,
    }, // surface0
    indicator_width: 0,
    status_bar_bg: RgbColor {
        r: 17,
        g: 17,
        b: 27,
    }, // crust
    notification_bg: RgbColor {
        r: 24,
        g: 24,
        b: 37,
    }, // mantle
    vertical_padding: 0,
    accent_info: RgbColor {
        r: 137,
        g: 220,
        b: 235,
    }, // sky
    accent_success: RgbColor {
        r: 166,
        g: 227,
        b: 161,
    }, // green
    accent_warning: RgbColor {
        r: 249,
        g: 226,
        b: 175,
    }, // yellow
    accent_error: RgbColor {
        r: 243,
        g: 139,
        b: 168,
    }, // red
    accent_active: RgbColor {
        r: 203,
        g: 166,
        b: 247,
    }, // mauve (same as focus)
    muted: RgbColor {
        r: 88,
        g: 91,
        b: 112,
    }, // overlay0
    accent_modified: RgbColor {
        r: 250,
        g: 179,
        b: 135,
    }, // peach
    accent_fps_good: RgbColor {
        r: 166,
        g: 227,
        b: 161,
    }, // green
    accent_fps_warn: RgbColor {
        r: 249,
        g: 226,
        b: 175,
    }, // yellow
};

/// Tokyo Night — deep indigo nights and neon city lights.
///
/// Inspired by the Tokyo Night VS Code theme: deep blue-purple backgrounds
/// with electric blue focus colors and vibrant accent hues.
pub const TOKYO_NIGHT: PanelStyle = PanelStyle {
    bg_color: RgbColor {
        r: 26,
        g: 27,
        b: 38,
    }, // bg
    focus_color: RgbColor {
        r: 122,
        g: 162,
        b: 247,
    }, // blue
    unfocus_color: RgbColor {
        r: 41,
        g: 46,
        b: 66,
    }, // bg_dark
    text_primary: RgbColor {
        r: 192,
        g: 202,
        b: 245,
    }, // fg
    text_secondary: RgbColor {
        r: 169,
        g: 177,
        b: 214,
    }, // fg_dark
    border_color: RgbColor {
        r: 41,
        g: 46,
        b: 66,
    }, // bg_dark — subtle border matching bg family
    indicator_width: 0,
    status_bar_bg: RgbColor {
        r: 16,
        g: 16,
        b: 28,
    }, // deeper than bg
    notification_bg: RgbColor {
        r: 22,
        g: 22,
        b: 36,
    }, // slightly lighter than status bar
    vertical_padding: 0,
    accent_info: RgbColor {
        r: 125,
        g: 207,
        b: 255,
    }, // cyan
    accent_success: RgbColor {
        r: 158,
        g: 206,
        b: 106,
    }, // green
    accent_warning: RgbColor {
        r: 224,
        g: 175,
        b: 104,
    }, // yellow
    accent_error: RgbColor {
        r: 247,
        g: 118,
        b: 142,
    }, // red
    accent_active: RgbColor {
        r: 187,
        g: 154,
        b: 247,
    }, // purple
    muted: RgbColor {
        r: 86,
        g: 95,
        b: 137,
    }, // comment
    accent_modified: RgbColor {
        r: 255,
        g: 158,
        b: 100,
    }, // orange
    accent_fps_good: RgbColor {
        r: 158,
        g: 206,
        b: 106,
    }, // green
    accent_fps_warn: RgbColor {
        r: 224,
        g: 175,
        b: 104,
    }, // yellow
};

impl Default for PanelStyle {
    fn default() -> Self {
        GRUVBOX_DARK
    }
}

/// Available UI themes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Theme {
    /// Gruvbox Dark-inspired theme.
    GruvboxDark,
    /// Slime Mold Dark bioluminescent theme.
    SlimeDark,
    /// Nord Arctic blue-gray theme.
    Nord,
    /// Catppuccin Mocha warm lavender theme.
    CatppuccinMocha,
    /// Tokyo Night deep indigo theme.
    TokyoNight,
}

impl Theme {
    /// Returns the display name of this theme.
    pub fn name(&self) -> &'static str {
        match self {
            Theme::GruvboxDark => "GruvboxDark",
            Theme::SlimeDark => "SlimeDark",
            Theme::Nord => "Nord",
            Theme::CatppuccinMocha => "CatppuccinMocha",
            Theme::TokyoNight => "TokyoNight",
        }
    }

    /// Returns the `PanelStyle` for this theme.
    pub fn style(&self) -> PanelStyle {
        match self {
            Theme::GruvboxDark => GRUVBOX_DARK,
            Theme::SlimeDark => SLIME_DARK,
            Theme::Nord => NORD,
            Theme::CatppuccinMocha => CATPPUCCIN_MOCHA,
            Theme::TokyoNight => TOKYO_NIGHT,
        }
    }
}

/// All available themes in cycling order.
pub const ALL_THEMES: [Theme; 5] = [
    Theme::GruvboxDark,
    Theme::SlimeDark,
    Theme::Nord,
    Theme::CatppuccinMocha,
    Theme::TokyoNight,
];
