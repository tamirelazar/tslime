//! Two-depth Controls "Instrument" surface (Tuner ⇄ Console). See
//! docs/superpowers/specs/2026-06-23-controls-instrument-ui-design.md.

pub mod registry;
pub use registry::{ParamDesc, ParamId, ParamKind};

pub mod value;
pub use value::{gauge, heatmap_slider, sparkline};

pub mod console;
pub use console::{build_console, ParamState, ParamView};

use crate::render::palette::RgbColor;
use crate::render::panel::RenderedOverlay;
use crate::render::theme::PanelStyle;

/// Dispatch the active Controls depth to its renderer.
///
/// Returns the [`RenderedOverlay`] to draw, or `None` when there is nothing to
/// draw (`Closed`).
///
/// - `Console` → [`build_console`].
/// - `Tuner` → `None` for now; the real Tuner render is wired in Task 12. The
///   surface visibility gate in the runner still tracks open/close, so a `None`
///   here simply means "draw nothing this frame" until the Tuner builder lands.
/// - `Closed` → `None`.
#[doc = "Tuner depth currently returns `None`; its renderer is wired in Task 12."]
pub fn build_controls(
    depth: ControlsDepth,
    category: usize,
    focus: usize,
    params: &[ParamView],
    style: &PanelStyle,
    accent: RgbColor,
) -> Option<RenderedOverlay> {
    match depth {
        ControlsDepth::Console => Some(build_console(category, focus, params, style, accent)),
        // Task 12 wires the Tuner renderer here; until then, nothing to draw.
        ControlsDepth::Tuner => None,
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
            build_controls(ControlsDepth::Console, 0, 0, &fixture, &style, accent).is_some(),
            "Console depth must produce an overlay"
        );
        assert!(
            build_controls(ControlsDepth::Closed, 0, 0, &fixture, &style, accent).is_none(),
            "Closed depth must produce no overlay"
        );
        assert!(
            build_controls(ControlsDepth::Tuner, 0, 0, &fixture, &style, accent).is_none(),
            "Tuner depth is wired in Task 12; for now it produces no overlay"
        );
    }
}
