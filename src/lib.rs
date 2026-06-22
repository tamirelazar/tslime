//! # tslime
//!
//! A lightweight terminal screensaver simulating the mesmerizing growth patterns
//! of *Physarum polycephalum* (slime mold).
//!
//! ## Overview
//!
//! tslime implements the agent-based model from Jones (2010), *"Characteristics
//! of Pattern Formation and Evolution in Approximations of Physarum Transport
//! Networks"* (full citation in [`simulation`]).
//!
//! The simulation involves:
//! 1. **Sense**: Agents sample pheromone trails at three points
//! 2. **Rotate**: Adjust heading based on sensed values
//! 3. **Move**: Update position based on heading and step size
//! 4. **Deposit**: Add pheromone to trail map
//! 5. **Diffuse**: Apply blur to spread pheromone
//! 6. **Decay**: Multiply all trail values by decay factor
//!
//! ## Example
//!
//! ```rust,no_run
//! use tslime::Simulation;
//! use tslime::simulation::config::{SimConfig, InitMode};
//!
//! let config = SimConfig::default();
//! let mut sim = Simulation::new(400, 400, config, 42, InitMode::Random, 0);
//!
//! // Run simulation steps
//! for _ in 0..100 {
//!     sim.update(1.0);
//! }
//!
//! // Get trail map for rendering
//! let mut trail = Vec::new();
//! sim.trail_map_blended(&mut trail);
//! ```
//!
#![warn(missing_docs)]

/// Application entry point and high-level logic.
#[cfg(feature = "terminal")]
pub mod app;
/// Restart-only application-level runtime configuration (warmup, auto-reset, grid, food-persist).
pub(crate) mod app_config;
/// Choir-mode audio, after Miranda, Adamatzky & Jones (2011), "Sounds
/// Synthesis with Slime Mould of Physarum Polycephalum".
#[cfg(feature = "audio")]
pub mod audio;
/// Command-line argument parsing and configuration.
pub mod cli;
/// Centralized configuration defaults.
pub mod config_defaults;
/// Configuration management (load/save/delete).
#[cfg(feature = "terminal")]
pub mod config_manager;
/// Error types for structured error handling.
pub mod error;
/// Parameter space exploration for preset discovery.
#[cfg(feature = "terminal")]
pub mod exploration;
/// Export functionality (GIF, WebM, PNG).
#[cfg(feature = "terminal")]
pub mod export;
/// Overlay system (state management, rendering, input).
#[cfg(feature = "terminal")]
pub mod overlay;
/// Saved palette management.
#[cfg(feature = "terminal")]
pub mod palette_manager;
/// Per-preset app-runtime defaults.
pub(crate) mod preset_app_defaults;
/// Per-preset optional sim-layer overrides.
pub(crate) mod preset_sim_defaults;
/// Shared resolved lever set for presets and saved configs.
pub(crate) mod profile;
/// Single all-Option authored partial (sim ⊕ render ⊕ seed).
pub(crate) mod profile_overrides;
/// Rendering logic (ASCII/Unicode, color palettes, dithering).
pub mod render;
/// Per-preset render/art-layer defaults.
pub(crate) mod render_art_defaults;
/// Core simulation logic (agents, trail map).
pub mod simulation;
/// Terminal handling (input, output, raw mode).
#[cfg(feature = "terminal")]
pub mod terminal;
/// Validation utilities for configuration.
pub mod validation;

/// Embedded GUI terminal window (iced_term) for double-click launch.
#[cfg(feature = "gui")]
pub mod gui;

/// Embedded food image data.
mod food_image;

// Re-export commonly used types
pub use simulation::Simulation;
