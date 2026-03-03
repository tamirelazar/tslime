/// Adaptive brightness normalization logic.
pub mod adaptive_brightness;
/// Character set definitions and mapping logic.
pub mod charset;
/// Standalone (non-palette) color constants.
pub mod color_constants;
/// Dithering algorithms (ordered, error diffusion).
pub mod dither;
/// Downsampling from simulation grid to terminal grid.
pub mod downsample;
/// Error diffusion specific implementation.
pub mod error_diffusion;
/// Color gradient data for palettes.
pub mod gradients;
/// Background grid rendering.
pub mod grid;
/// Controls overlay rendering.
pub mod options_overlay;
/// General overlay rendering utilities (help, stats, etc.).
pub mod overlay;
/// Color palette definitions and conversions.
pub mod palette;
/// Interactive palette editor for custom color schemes.
pub mod palette_editor;
/// Panel styling and theme definitions.
pub mod panel;
/// Theme/color scheme definitions.
pub mod theme;
