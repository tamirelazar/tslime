//! L2 shared widget kit: primitives + spacing rhythm, all consuming L1 PanelStyle tokens.

mod kit;
mod rowbuf;
mod state;
pub use kit::*;
pub use rowbuf::RowBuf;
pub use state::{state_color, value_color, ParamState};

/// Named vertical spacing steps for consistent layout rhythm.
pub mod spacing {
    /// No gap between rows.
    pub const TIGHT: usize = 0;
    /// One blank row between related rows.
    pub const ROW: usize = 1;
    /// Two blank rows between distinct sections.
    pub const SECTION: usize = 2;
}

#[cfg(test)]
mod spacing_tests {
    use super::spacing;

    #[allow(clippy::assertions_on_constants)]
    #[test]
    fn rhythm_steps_are_ordered() {
        assert!(spacing::TIGHT < spacing::ROW);
        assert!(spacing::ROW < spacing::SECTION);
    }
}
