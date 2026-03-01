//! Embedded food image for simulation initialization.
//!
//! This image is baked into the binary to work both in development
//! and when the app is bundled as a .app.

pub const FOOD_IMAGE_PNG: &[u8] = include_bytes!("../assets/tslime_logo.png");
