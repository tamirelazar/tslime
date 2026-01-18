//! # tslime
//!
//! A lightweight terminal screensaver simulating the mesmerizing growth patterns
//! of *Physarum polycephalum* (slime mold).
//!
//! ## Overview
//!
//! tslime implements the agent-based model from Jeff Jones' 2010 paper
//! *"Characteristics of Pattern Formation and Evolution in Approximations of
//! Physarum Transport Networks."*
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
//! let trail = sim.trail_map_blended();
//! ```

pub mod app;
pub mod cli;
pub mod config_manager;
pub mod export;
pub mod render;
pub mod simulation;
pub mod terminal;

// Re-export commonly used types
pub use simulation::Simulation;
