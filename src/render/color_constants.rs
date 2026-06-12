//! Color constants for standalone (non-palette) colors.
//!
//! This module provides named constants for colors that are used directly
//! in the simulation and are NOT part of the color palette system.
//! Palette colors should be referenced through the Palette enum, not here.

/// Default color constants used across the simulation.
pub mod default {
    /// Default forest green color for species.
    /// Used as the fallback color when no specific preset color is specified.
    pub const FOREST_GREEN: &str = "228b22";
}

/// Preset-specific color constants.
/// These colors are tied to specific presets and are not part of the palette system.
pub mod presets {
    /// Moss preset color - organic moss green.
    pub const MOSS_GREEN: &str = "4a7a4a";

    /// Cosmic preset color - deep purple.
    pub const COSMIC_PURPLE: &str = "8a2be2";

    /// Fire preset color - orange red.
    pub const FIRE_ORANGE: &str = "ff4500";

    /// Zen preset color - pure white.
    pub const ZEN_WHITE: &str = "ffffff";

    /// Storm preset color - steel blue.
    pub const STORM_BLUE: &str = "4682b4";

    /// River preset color - dodger blue.
    pub const RIVER_BLUE: &str = "1e90ff";

    /// Ethereal preset color - lavender.
    pub const ETHEREAL_LAVENDER: &str = "e6e6fa";

    /// PetriDish preset color - yellowish mold.
    pub const MOLD_YELLOW: &str = "d4ff00";

    /// Vortex preset color - medium purple.
    pub const VORTEX_PURPLE: &str = "9370db";

    /// Lightning preset color - cyan/electric blue.
    pub const LIGHTNING_CYAN: &str = "00ffff";

    /// Crystal preset color - powder blue/ice.
    pub const CRYSTAL_ICE: &str = "b0e0e6";

    /// ChaosEdge preset color - tomato red.
    pub const CHAOS_RED: &str = "ff6347";

    /// Blob preset color - lime green.
    pub const BLOB_LIME: &str = "32cd32";

    /// Worm preset color - goldenrod.
    pub const WORM_GOLD: &str = "daa520";
}

/// UI and background color constants.
pub mod ui {
    /// Pure black for backgrounds.
    pub const BLACK: &str = "000000";

    /// Pure white for text/highlights.
    pub const WHITE: &str = "ffffff";

    /// Default grid color.
    pub const GRID_DEFAULT: &str = "ffffff";
}

/// Returns the default species color as a hex string.
pub fn default_species_color() -> String {
    default::FOREST_GREEN.to_string()
}
