//! Two-depth Controls "Instrument" surface (Tuner ⇄ Console). See
//! docs/superpowers/specs/2026-06-23-controls-instrument-ui-design.md.

pub mod registry;
pub use registry::{ParamDesc, ParamId, ParamKind};

pub mod value;
pub use value::{gauge, heatmap_slider, sparkline};

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
}
