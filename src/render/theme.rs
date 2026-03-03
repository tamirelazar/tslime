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
    bg_color: RgbColor::new(0x28, 0x28, 0x28), // #282828 - dark background
    focus_color: RgbColor::new(0xFA, 0xBD, 0x2F), // #FABD2F - yellow accent
    unfocus_color: RgbColor::new(0x78, 0x5F, 0x26), // #785F26 - dimmed yellow
    text_primary: RgbColor::new(0xEB, 0xDB, 0xB2), // #EBDBB2 - light text
    text_secondary: RgbColor::new(0xB9, 0xAA, 0x8C), // #B9AA8C - muted text
    border_color: RgbColor::new(0x69, 0x63, 0x58), // #696358 - warm gray
    indicator_width: 0,
    status_bar_bg: RgbColor::new(0x10, 0x12, 0x13), // #101213 - darker bg
    notification_bg: RgbColor::new(0x21, 0x26, 0x23), // #212623 - slightly lighter
    vertical_padding: 0,
    accent_info: RgbColor::new(0x83, 0xA5, 0x98), // #83A598 - aqua
    accent_success: RgbColor::new(0xB8, 0xBB, 0x26), // #B8BB26 - green
    accent_warning: RgbColor::new(0xFA, 0xBD, 0x2F), // #FABD2F - yellow
    accent_error: RgbColor::new(0xFB, 0x49, 0x34), // #FB4934 - red
    accent_active: RgbColor::new(0xFA, 0xBD, 0x2F), // #FABD2F - yellow
    muted: RgbColor::new(0x66, 0x5C, 0x54),       // #665C54 - dark4
    accent_modified: RgbColor::new(0xFE, 0x80, 0x19), // #FE8019 - orange
    accent_fps_good: RgbColor::new(0x8E, 0xC0, 0x7C), // #8EC07C - bright green
    accent_fps_warn: RgbColor::new(0xD7, 0x99, 0x21), // #D79921 - amber
};

/// Slime Mold Dark — a bioluminescent palette inspired by Physarum polycephalum.
///
/// Deep blue-black backgrounds with electric green and cyan highlights
/// evoke the glowing, living quality of the simulated organism.
pub const SLIME_DARK: PanelStyle = PanelStyle {
    bg_color: RgbColor::new(0x0B, 0x10, 0x0E), // #0B100E - near-black green
    focus_color: RgbColor::new(0x39, 0xD3, 0x53), // #39D353 - bioluminescent green
    unfocus_color: RgbColor::new(0x18, 0x48, 0x28), // #184828 - dim green
    text_primary: RgbColor::new(0xBE, 0xE6, 0xCD), // #BEE6CD - soft green-white
    text_secondary: RgbColor::new(0x64, 0x9B, 0x76), // #649B76 - muted green-gray
    border_color: RgbColor::new(0x1A, 0x38, 0x24), // #1A3824 - subtle dark green
    indicator_width: 0,
    status_bar_bg: RgbColor::new(0x06, 0x0A, 0x08), // #060A08 - near-black
    notification_bg: RgbColor::new(0x0F, 0x1A, 0x13), // #0F1A13 - slightly lighter
    vertical_padding: 0,
    accent_info: RgbColor::new(0x00, 0xBC, 0xD4), // #00BCD4 - cyan
    accent_success: RgbColor::new(0x39, 0xD3, 0x53), // #39D353 - electric green
    accent_warning: RgbColor::new(0xFF, 0xB7, 0x00), // #FFB700 - amber
    accent_error: RgbColor::new(0xFF, 0x47, 0x57), // #FF4757 - coral red
    accent_active: RgbColor::new(0x39, 0xD3, 0x53), // #39D353 - electric green
    muted: RgbColor::new(0x24, 0x3A, 0x2C),       // #243A2C - muted green-gray
    accent_modified: RgbColor::new(0xFF, 0xB7, 0x00), // #FFB700 - amber
    accent_fps_good: RgbColor::new(0x39, 0xD3, 0x53), // #39D353 - electric green
    accent_fps_warn: RgbColor::new(0xFF, 0xB7, 0x00), // #FFB700 - amber
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
