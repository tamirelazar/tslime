//! Rendering pipeline from simulation data to terminal output: downsampling,
//! color mapping, character selection, dithering, and overlays.
//!
//! # Pipeline Overview
//!
//! 1. **Downsampling** (`downsample`): Converts high-resolution trail map to terminal dimensions
//! 2. **Color Mapping** (`palette`): Maps trail brightness to RGB colors using selected palette
//! 3. **Character Selection** (`charset`): Chooses appropriate Unicode characters based on brightness
//! 4. **Dithering** (`dither`, `error_diffusion`): Applies dithering for smoother gradients
//! 5. **Overlays** (`overlay`, `panel`): Renders UI elements like controls and statistics
//!
//! # Example
//!
//! ```rust,no_run
//! use tslime::render::downsample::{downsample, DownsampledFrame};
//! use tslime::render::palette::{Palette, map_brightness_rgb};
//!
//! let trail_data = vec![0.5f32; 400 * 400]; // Simulated trail data
//! let mut frame = DownsampledFrame::new(80, 24);
//! downsample(&trail_data, 400, 400, 80, 24, &mut frame);
//!
//! // Map brightness to color
//! let color = map_brightness_rgb(0.5, Palette::Organic, false, false, 0.0, None);
//! ```

/// Adaptive brightness normalization logic.
pub mod adaptive_brightness;
/// Ambient instrument surface and state machine.
#[cfg(feature = "terminal")]
pub mod ambient;
/// Crossterm-free half-block ANSI frame rendering (for wasm / xterm.js).
pub mod ansi;
/// Color anti-aliasing for subcell-shape charsets.
pub mod antialiasing;
/// Character set definitions and mapping logic.
pub mod charset;
/// Standalone (non-palette) color constants.
pub mod color_constants;
/// Two-depth Controls instrument surface.
#[cfg(feature = "terminal")]
pub mod controls;
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
/// Motion math helpers (lerp, breath).
#[cfg(feature = "terminal")]
pub mod motion;
/// Color palette definitions and conversions.
pub mod palette;
/// Theme/color scheme definitions.
pub mod theme;
/// Reusable rendering widgets and layout tokens.
#[cfg(feature = "terminal")]
pub mod widgets;
/// Window layout geometry computation (aspect-ratio-correct sim rect).
pub mod window;
/// Window frame rendering for terminal display.
#[cfg(feature = "terminal")]
pub mod window_frame;

#[cfg(feature = "terminal")]
/// General overlay rendering utilities (help, stats, etc.).
pub mod overlay;
#[cfg(feature = "terminal")]
/// Interactive palette editor for custom color schemes.
pub mod palette_editor;
#[cfg(feature = "terminal")]
/// Panel styling and theme definitions.
pub mod panel;

#[cfg(feature = "terminal")]
/// Config browser and save-dialog overlay builders (hand-rolled; no ratatui dep).
pub mod ratatui_adapter;

#[cfg(feature = "terminal")]
/// Preset-switch transition overlays (figlet / typed readout).
pub mod transition;
