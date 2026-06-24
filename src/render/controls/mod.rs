//! Two-depth Controls "Instrument" surface (Tuner ⇄ Console). See
//! docs/superpowers/specs/2026-06-23-controls-instrument-ui-design.md.

pub mod registry;
pub use registry::{ParamDesc, ParamId, ParamKind};

pub mod value;
pub use value::{gauge, heatmap_slider, sparkline};

pub mod console;
pub use console::{build_console, ParamState, ParamView};

pub mod tuner;
pub use tuner::build_tuner;

use crate::render::palette::RgbColor;
use crate::render::panel::RenderedOverlay;
use crate::render::theme::PanelStyle;

/// Dispatch the active Controls depth to its renderer.
///
/// Returns the [`RenderedOverlay`] to draw, or `None` when there is nothing to
/// draw (`Closed`).
///
/// - `Console` → [`build_console`].
/// - `Tuner` → [`build_tuner`] using `params[focus]` as the focused parameter.
///   `recent` is passed as an empty slice (no rolling history source yet; the
///   tuner falls back to showing the focused param in the RECENT row).
///   Returns `None` only when `params` is empty.
/// - `Closed` → `None`.
///
/// `truecolor` gates the heatmap-slider gradient; pass `true` for 24-bit
/// terminals and `false` for 256-colour or below.
///
/// `term_width` is forwarded to [`build_tuner`] so the strip fills the full
/// terminal width instead of the default [`STRIP_W`] minimum.
#[allow(clippy::too_many_arguments)]
pub fn build_controls(
    depth: ControlsDepth,
    category: usize,
    focus: usize,
    params: &[ParamView],
    style: &PanelStyle,
    accent: RgbColor,
    truecolor: bool,
    term_width: usize,
) -> Option<RenderedOverlay> {
    match depth {
        ControlsDepth::Console => Some(build_console(category, focus, params, style, accent)),
        ControlsDepth::Tuner => {
            // Clamp focus to a valid index; bail gracefully when params is empty.
            let focused = params.get(focus.min(params.len().saturating_sub(1)))?;
            Some(build_tuner(
                focused,
                &[],
                style,
                accent,
                truecolor,
                term_width,
            ))
        }
        ControlsDepth::Closed => None,
    }
}

/// Which depth of the Controls surface is showing.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum ControlsDepth {
    /// Surface hidden.
    #[default]
    Closed,
    /// Ambient bottom-docked play surface.
    Tuner,
    /// Opaque master-detail study surface.
    Console,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn depth_default_is_closed() {
        assert_eq!(ControlsDepth::default(), ControlsDepth::Closed);
    }

    #[test]
    fn build_controls_dispatches_on_depth() {
        let style = crate::render::theme::SLIME_DARK;
        let accent = RgbColor { r: 0, g: 200, b: 0 };
        let fixture = vec![ParamView {
            desc: ParamDesc {
                id: ParamId::SensorAngle,
                key_hint: "A/a",
                label: "Sensor Angle",
                kind: ParamKind::Numeric,
            },
            value_text: "30.0°".to_string(),
            ratio: Some(0.5),
            def_ratio: Some(0.3),
            state: ParamState::Modified,
        }];

        assert!(
            build_controls(
                ControlsDepth::Console,
                0,
                0,
                &fixture,
                &style,
                accent,
                true,
                80
            )
            .is_some(),
            "Console depth must produce an overlay"
        );
        assert!(
            build_controls(
                ControlsDepth::Closed,
                0,
                0,
                &fixture,
                &style,
                accent,
                true,
                80
            )
            .is_none(),
            "Closed depth must produce no overlay"
        );
        assert!(
            build_controls(
                ControlsDepth::Tuner,
                0,
                0,
                &fixture,
                &style,
                accent,
                true,
                80
            )
            .is_some(),
            "Tuner depth must produce an overlay once a focused param exists"
        );
        assert!(
            build_controls(ControlsDepth::Tuner, 0, 0, &[], &style, accent, true, 80).is_none(),
            "Tuner depth with empty params must produce no overlay"
        );
    }
}
