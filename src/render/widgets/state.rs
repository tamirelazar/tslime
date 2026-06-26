use crate::render::palette::RgbColor;
use crate::render::theme::PanelStyle;

/// Functional state of a parameter value, driving its color.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ParamState {
    /// Value matches its configured default.
    Default,
    /// Value has been changed interactively.
    Modified,
    /// Value was supplied by the command line.
    Cli,
    /// Value is display-only and has no editable state.
    Display,
}

/// Map a ParamState to a theme color. Reads ONLY from PanelStyle — no hardcoded RGB.
pub fn state_color(state: ParamState, st: &PanelStyle) -> RgbColor {
    match state {
        ParamState::Cli => st.cli_color,
        ParamState::Modified => st.accent_modified,
        ParamState::Default | ParamState::Display => st.state_default,
    }
}

/// Color for a parameter's *value text*. Like [`state_color`] but keeps a value
/// at its default legible: a muted default value reads as disabled, so Default
/// and Display fold to `text_primary` while Modified/Cli still carry their state
/// color. Single source of truth for both the console value row and the ambient
/// tuner value — do not re-roll this match inline.
pub fn value_color(state: ParamState, st: &PanelStyle) -> RgbColor {
    match state {
        ParamState::Cli => st.cli_color,
        ParamState::Modified => st.accent_modified,
        ParamState::Default | ParamState::Display => st.text_primary,
    }
}

#[cfg(test)]
mod state_tests {
    use super::*;
    use crate::render::theme::GRUVBOX_DARK;

    #[test]
    fn state_color_reads_only_tokens() {
        assert_eq!(
            state_color(ParamState::Cli, &GRUVBOX_DARK),
            GRUVBOX_DARK.cli_color
        );
        assert_eq!(
            state_color(ParamState::Modified, &GRUVBOX_DARK),
            GRUVBOX_DARK.accent_modified
        );
        assert_eq!(
            state_color(ParamState::Default, &GRUVBOX_DARK),
            GRUVBOX_DARK.state_default
        );
        assert_eq!(
            state_color(ParamState::Display, &GRUVBOX_DARK),
            GRUVBOX_DARK.state_default
        );
    }
}
