//! L2 shared widget kit: primitives + spacing rhythm, all consuming L1 PanelStyle tokens.

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

    // This runtime test intentionally documents the public rhythm invariant.
    #[allow(clippy::assertions_on_constants)]
    #[test]
    fn rhythm_steps_are_ordered() {
        assert!(spacing::TIGHT < spacing::ROW);
        assert!(spacing::ROW < spacing::SECTION);
    }
}
