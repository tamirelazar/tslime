//! Gradient data for color palettes.
//!
//! This module contains all the predefined color gradient data used by the palette system.
//! Each palette is defined in three representations:
//! - 256-color ANSI indices (11 stops) — for terminals without true-color support
//! - Legacy RGB (11 stops) — kept for reference and 256-color fallback calculation
//! - OKLch perceptual color space (11 stops) — the primary source for true-color rendering
//!
//! OKLch stop definitions follow Björn Ottosson's OKLab color space; see
//! `palette.rs` (`oklch_to_srgb`) for the conversion chain and full reference.

use crate::cli::Palette;
use crate::render::palette::{OklchStop, RgbColor};
const ORGANIC_GRADIENT: [u8; 11] = [232, 22, 28, 34, 40, 46, 82, 118, 154, 190, 226];

const ORGANIC_RGB: [RgbColor; 11] = [
    RgbColor {
        r: 18,
        g: 18,
        b: 18,
    },
    RgbColor {
        r: 40,
        g: 40,
        b: 40,
    },
    RgbColor {
        r: 70,
        g: 20,
        b: 20,
    },
    RgbColor {
        r: 100,
        g: 40,
        b: 40,
    },
    RgbColor {
        r: 130,
        g: 50,
        b: 40,
    },
    RgbColor {
        r: 160,
        g: 50,
        b: 50,
    },
    RgbColor {
        r: 120,
        g: 100,
        b: 50,
    },
    RgbColor {
        r: 100,
        g: 130,
        b: 60,
    },
    RgbColor {
        r: 80,
        g: 160,
        b: 80,
    },
    RgbColor {
        r: 100,
        g: 190,
        b: 130,
    },
    RgbColor {
        r: 150,
        g: 220,
        b: 200,
    },
];

const HEAT_GRADIENT: [u8; 11] = [232, 52, 88, 124, 160, 196, 202, 208, 214, 220, 226];

const HEAT_RGB: [RgbColor; 11] = [
    RgbColor {
        r: 40,
        g: 20,
        b: 20,
    },
    RgbColor {
        r: 40,
        g: 20,
        b: 20,
    },
    RgbColor {
        r: 70,
        g: 20,
        b: 20,
    },
    RgbColor {
        r: 110,
        g: 20,
        b: 20,
    },
    RgbColor {
        r: 150,
        g: 20,
        b: 20,
    },
    RgbColor {
        r: 190,
        g: 40,
        b: 30,
    },
    RgbColor {
        r: 200,
        g: 70,
        b: 40,
    },
    RgbColor {
        r: 210,
        g: 100,
        b: 50,
    },
    RgbColor {
        r: 220,
        g: 140,
        b: 60,
    },
    RgbColor {
        r: 230,
        g: 180,
        b: 80,
    },
    RgbColor {
        r: 240,
        g: 220,
        b: 180,
    },
];

const OCEAN_GRADIENT: [u8; 11] = [232, 17, 18, 19, 20, 21, 27, 33, 39, 45, 51];

const OCEAN_RGB: [RgbColor; 11] = [
    RgbColor {
        r: 18,
        g: 18,
        b: 18,
    },
    RgbColor {
        r: 20,
        g: 20,
        b: 50,
    },
    RgbColor {
        r: 20,
        g: 25,
        b: 60,
    },
    RgbColor {
        r: 20,
        g: 30,
        b: 70,
    },
    RgbColor {
        r: 20,
        g: 40,
        b: 80,
    },
    RgbColor {
        r: 25,
        g: 50,
        b: 100,
    },
    RgbColor {
        r: 30,
        g: 70,
        b: 130,
    },
    RgbColor {
        r: 40,
        g: 90,
        b: 160,
    },
    RgbColor {
        r: 50,
        g: 110,
        b: 190,
    },
    RgbColor {
        r: 60,
        g: 140,
        b: 220,
    },
    RgbColor {
        r: 80,
        g: 170,
        b: 240,
    },
];

const MONO_GRADIENT: [u8; 11] = [232, 234, 236, 238, 240, 242, 244, 246, 248, 250, 252];

const MONO_RGB: [RgbColor; 11] = [
    RgbColor {
        r: 18,
        g: 18,
        b: 18,
    },
    RgbColor {
        r: 35,
        g: 35,
        b: 35,
    },
    RgbColor {
        r: 55,
        g: 55,
        b: 55,
    },
    RgbColor {
        r: 75,
        g: 75,
        b: 75,
    },
    RgbColor {
        r: 95,
        g: 95,
        b: 95,
    },
    RgbColor {
        r: 115,
        g: 115,
        b: 115,
    },
    RgbColor {
        r: 135,
        g: 135,
        b: 135,
    },
    RgbColor {
        r: 155,
        g: 155,
        b: 155,
    },
    RgbColor {
        r: 175,
        g: 175,
        b: 175,
    },
    RgbColor {
        r: 195,
        g: 195,
        b: 195,
    },
    RgbColor {
        r: 215,
        g: 215,
        b: 215,
    },
];

const FOREST_GRADIENT: [u8; 11] = [22, 22, 34, 34, 40, 40, 118, 118, 154, 118, 40];

const FOREST_RGB: [RgbColor; 11] = [
    RgbColor {
        r: 20,
        g: 40,
        b: 20,
    },
    RgbColor {
        r: 30,
        g: 50,
        b: 25,
    },
    RgbColor {
        r: 40,
        g: 60,
        b: 30,
    },
    RgbColor {
        r: 50,
        g: 80,
        b: 35,
    },
    RgbColor {
        r: 60,
        g: 100,
        b: 40,
    },
    RgbColor {
        r: 70,
        g: 120,
        b: 50,
    },
    RgbColor {
        r: 80,
        g: 140,
        b: 60,
    },
    RgbColor {
        r: 100,
        g: 160,
        b: 80,
    },
    RgbColor {
        r: 120,
        g: 180,
        b: 100,
    },
    RgbColor {
        r: 150,
        g: 200,
        b: 130,
    },
    RgbColor {
        r: 180,
        g: 220,
        b: 170,
    },
];

const NEON_GRADIENT: [u8; 11] = [17, 27, 39, 51, 87, 123, 159, 195, 201, 225, 195];

const NEON_RGB: [RgbColor; 11] = [
    RgbColor { r: 30, g: 0, b: 50 },
    RgbColor {
        r: 40,
        g: 10,
        b: 60,
    },
    RgbColor {
        r: 50,
        g: 20,
        b: 80,
    },
    RgbColor {
        r: 60,
        g: 40,
        b: 100,
    },
    RgbColor {
        r: 80,
        g: 70,
        b: 130,
    },
    RgbColor {
        r: 100,
        g: 100,
        b: 160,
    },
    RgbColor {
        r: 120,
        g: 130,
        b: 190,
    },
    RgbColor {
        r: 140,
        g: 160,
        b: 220,
    },
    RgbColor {
        r: 170,
        g: 190,
        b: 240,
    },
    RgbColor {
        r: 200,
        g: 220,
        b: 255,
    },
    RgbColor {
        r: 150,
        g: 60,
        b: 200,
    },
];

const WARM_GRADIENT: [u8; 11] = [52, 94, 130, 166, 202, 208, 214, 220, 226, 226, 226];

const WARM_RGB: [RgbColor; 11] = [
    RgbColor {
        r: 40,
        g: 20,
        b: 20,
    },
    RgbColor {
        r: 60,
        g: 30,
        b: 20,
    },
    RgbColor {
        r: 80,
        g: 40,
        b: 25,
    },
    RgbColor {
        r: 110,
        g: 55,
        b: 30,
    },
    RgbColor {
        r: 140,
        g: 70,
        b: 35,
    },
    RgbColor {
        r: 170,
        g: 90,
        b: 45,
    },
    RgbColor {
        r: 200,
        g: 110,
        b: 60,
    },
    RgbColor {
        r: 210,
        g: 140,
        b: 80,
    },
    RgbColor {
        r: 220,
        g: 170,
        b: 100,
    },
    RgbColor {
        r: 230,
        g: 200,
        b: 140,
    },
    RgbColor {
        r: 240,
        g: 230,
        b: 200,
    },
];

const VIBRANT_GRADIENT: [u8; 11] = [197, 209, 221, 193, 157, 121, 85, 49, 51, 87, 231];

const VIBRANT_RGB: [RgbColor; 11] = [
    RgbColor {
        r: 50,
        g: 20,
        b: 60,
    },
    RgbColor {
        r: 60,
        g: 40,
        b: 80,
    },
    RgbColor {
        r: 80,
        g: 60,
        b: 100,
    },
    RgbColor {
        r: 100,
        g: 80,
        b: 80,
    },
    RgbColor {
        r: 120,
        g: 100,
        b: 60,
    },
    RgbColor {
        r: 140,
        g: 120,
        b: 40,
    },
    RgbColor {
        r: 160,
        g: 140,
        b: 30,
    },
    RgbColor {
        r: 180,
        g: 160,
        b: 30,
    },
    RgbColor {
        r: 200,
        g: 150,
        b: 40,
    },
    RgbColor {
        r: 220,
        g: 140,
        b: 60,
    },
    RgbColor {
        r: 240,
        g: 130,
        b: 80,
    },
];

const LEGIBLEMONO_GRADIENT: [u8; 11] = [236, 240, 244, 248, 250, 251, 252, 253, 254, 255, 255];

const LEGIBLEMONO_RGB: [RgbColor; 11] = [
    RgbColor {
        r: 30,
        g: 30,
        b: 30,
    },
    RgbColor {
        r: 50,
        g: 50,
        b: 50,
    },
    RgbColor {
        r: 70,
        g: 70,
        b: 70,
    },
    RgbColor {
        r: 90,
        g: 90,
        b: 90,
    },
    RgbColor {
        r: 110,
        g: 110,
        b: 110,
    },
    RgbColor {
        r: 130,
        g: 130,
        b: 130,
    },
    RgbColor {
        r: 150,
        g: 150,
        b: 150,
    },
    RgbColor {
        r: 170,
        g: 170,
        b: 170,
    },
    RgbColor {
        r: 190,
        g: 190,
        b: 190,
    },
    RgbColor {
        r: 210,
        g: 210,
        b: 210,
    },
    RgbColor {
        r: 230,
        g: 230,
        b: 230,
    },
];

const SLIME_GRADIENT: [u8; 11] = [22, 28, 34, 40, 76, 82, 118, 154, 190, 226, 231];

const SLIME_RGB: [RgbColor; 11] = [
    RgbColor {
        r: 20,
        g: 40,
        b: 20,
    },
    RgbColor { r: 0, g: 95, b: 0 },
    RgbColor {
        r: 0,
        g: 135,
        b: 35,
    },
    RgbColor { r: 0, g: 175, b: 0 },
    RgbColor {
        r: 50,
        g: 200,
        b: 50,
    },
    RgbColor {
        r: 95,
        g: 215,
        b: 0,
    },
    RgbColor {
        r: 130,
        g: 230,
        b: 130,
    },
    RgbColor {
        r: 160,
        g: 240,
        b: 150,
    },
    RgbColor {
        r: 190,
        g: 250,
        b: 180,
    },
    RgbColor {
        r: 220,
        g: 255,
        b: 200,
    },
    RgbColor {
        r: 255,
        g: 255,
        b: 255,
    },
];

const MOLD_GRADIENT: [u8; 11] = [236, 100, 106, 112, 142, 148, 149, 150, 191, 192, 193];

const MOLD_RGB: [RgbColor; 11] = [
    RgbColor {
        r: 40,
        g: 40,
        b: 40,
    },
    RgbColor {
        r: 135,
        g: 135,
        b: 0,
    },
    RgbColor {
        r: 175,
        g: 165,
        b: 0,
    },
    RgbColor {
        r: 195,
        g: 185,
        b: 40,
    },
    RgbColor {
        r: 215,
        g: 200,
        b: 80,
    },
    RgbColor {
        r: 225,
        g: 210,
        b: 120,
    },
    RgbColor {
        r: 230,
        g: 215,
        b: 130,
    },
    RgbColor {
        r: 235,
        g: 220,
        b: 145,
    },
    RgbColor {
        r: 175,
        g: 235,
        b: 175,
    },
    RgbColor {
        r: 180,
        g: 240,
        b: 180,
    },
    RgbColor {
        r: 185,
        g: 245,
        b: 185,
    },
];

const FUNGUS_GRADIENT: [u8; 11] = [232, 54, 90, 126, 125, 163, 164, 165, 137, 143, 223];

const FUNGUS_RGB: [RgbColor; 11] = [
    RgbColor {
        r: 25,
        g: 20,
        b: 25,
    },
    RgbColor { r: 95, g: 0, b: 95 },
    RgbColor {
        r: 135,
        g: 0,
        b: 135,
    },
    RgbColor {
        r: 195,
        g: 0,
        b: 195,
    },
    RgbColor {
        r: 215,
        g: 0,
        b: 175,
    },
    RgbColor {
        r: 155,
        g: 105,
        b: 145,
    },
    RgbColor {
        r: 165,
        g: 115,
        b: 155,
    },
    RgbColor {
        r: 175,
        g: 125,
        b: 165,
    },
    RgbColor {
        r: 175,
        g: 150,
        b: 75,
    },
    RgbColor {
        r: 215,
        g: 205,
        b: 100,
    },
    RgbColor {
        r: 230,
        g: 240,
        b: 255,
    },
];

const SWAMP_GRADIENT: [u8; 11] = [232, 233, 234, 236, 239, 242, 65, 66, 72, 78, 79];

const SWAMP_RGB: [RgbColor; 11] = [
    RgbColor {
        r: 18,
        g: 18,
        b: 18,
    },
    RgbColor {
        r: 35,
        g: 35,
        b: 35,
    },
    RgbColor {
        r: 55,
        g: 55,
        b: 55,
    },
    RgbColor {
        r: 80,
        g: 85,
        b: 75,
    },
    RgbColor {
        r: 105,
        g: 110,
        b: 100,
    },
    RgbColor {
        r: 130,
        g: 140,
        b: 125,
    },
    RgbColor {
        r: 0,
        g: 130,
        b: 90,
    },
    RgbColor {
        r: 0,
        g: 135,
        b: 110,
    },
    RgbColor {
        r: 0,
        g: 150,
        b: 120,
    },
    RgbColor {
        r: 0,
        g: 175,
        b: 140,
    },
    RgbColor {
        r: 0,
        g: 190,
        b: 150,
    },
];

const MOSS_GRADIENT: [u8; 11] = [22, 22, 28, 34, 40, 70, 76, 112, 148, 184, 220];

const MOSS_RGB: [RgbColor; 11] = [
    RgbColor {
        r: 20,
        g: 35,
        b: 20,
    },
    RgbColor {
        r: 25,
        g: 45,
        b: 22,
    },
    RgbColor {
        r: 35,
        g: 60,
        b: 28,
    },
    RgbColor {
        r: 45,
        g: 80,
        b: 35,
    },
    RgbColor {
        r: 60,
        g: 100,
        b: 40,
    },
    RgbColor {
        r: 80,
        g: 120,
        b: 50,
    },
    RgbColor {
        r: 100,
        g: 140,
        b: 65,
    },
    RgbColor {
        r: 120,
        g: 160,
        b: 80,
    },
    RgbColor {
        r: 145,
        g: 175,
        b: 95,
    },
    RgbColor {
        r: 170,
        g: 190,
        b: 115,
    },
    RgbColor {
        r: 195,
        g: 210,
        b: 140,
    },
];

const COSMIC_GRADIENT: [u8; 11] = [53, 57, 98, 129, 165, 201, 207, 213, 219, 225, 231];

const COSMIC_RGB: [RgbColor; 11] = [
    RgbColor {
        r: 20,
        g: 10,
        b: 40,
    },
    RgbColor {
        r: 30,
        g: 15,
        b: 60,
    },
    RgbColor {
        r: 50,
        g: 20,
        b: 90,
    },
    RgbColor {
        r: 70,
        g: 30,
        b: 120,
    },
    RgbColor {
        r: 90,
        g: 50,
        b: 150,
    },
    RgbColor {
        r: 120,
        g: 80,
        b: 180,
    },
    RgbColor {
        r: 150,
        g: 110,
        b: 200,
    },
    RgbColor {
        r: 180,
        g: 140,
        b: 220,
    },
    RgbColor {
        r: 200,
        g: 170,
        b: 235,
    },
    RgbColor {
        r: 220,
        g: 200,
        b: 245,
    },
    RgbColor {
        r: 240,
        g: 230,
        b: 255,
    },
];

const ETHEREAL_GRADIENT: [u8; 11] = [232, 183, 189, 195, 201, 207, 218, 224, 225, 225, 224];

const ETHEREAL_RGB: [RgbColor; 11] = [
    RgbColor {
        r: 15,
        g: 15,
        b: 20,
    },
    RgbColor {
        r: 60,
        g: 50,
        b: 80,
    },
    RgbColor {
        r: 90,
        g: 80,
        b: 110,
    },
    RgbColor {
        r: 120,
        g: 110,
        b: 140,
    },
    RgbColor {
        r: 150,
        g: 140,
        b: 170,
    },
    RgbColor {
        r: 180,
        g: 170,
        b: 200,
    },
    RgbColor {
        r: 210,
        g: 200,
        b: 225,
    },
    RgbColor {
        r: 230,
        g: 220,
        b: 240,
    },
    RgbColor {
        r: 245,
        g: 235,
        b: 250,
    },
    RgbColor {
        r: 250,
        g: 240,
        b: 255,
    },
    RgbColor {
        r: 255,
        g: 240,
        b: 250,
    },
];

/// Returns the 256-color ANSI gradient for a given palette.
///
/// Each gradient contains 11 color stops ranging from dark to bright.
/// For custom palettes, returns the forest gradient as a fallback.
pub fn get_256_gradient(palette: Palette) -> &'static [u8; 11] {
    match palette {
        Palette::Organic => &ORGANIC_GRADIENT,
        Palette::Heat => &HEAT_GRADIENT,
        Palette::Ocean => &OCEAN_GRADIENT,
        Palette::Mono => &MONO_GRADIENT,
        Palette::Forest => &FOREST_GRADIENT,
        Palette::Neon => &NEON_GRADIENT,
        Palette::Warm => &WARM_GRADIENT,
        Palette::Vibrant => &VIBRANT_GRADIENT,
        Palette::LegibleMono => &LEGIBLEMONO_GRADIENT,
        Palette::Slime => &SLIME_GRADIENT,
        Palette::Mold => &MOLD_GRADIENT,
        Palette::Fungus => &FUNGUS_GRADIENT,
        Palette::Swamp => &SWAMP_GRADIENT,
        Palette::Moss => &MOSS_GRADIENT,
        Palette::Cosmic => &COSMIC_GRADIENT,
        Palette::Ethereal => &ETHEREAL_GRADIENT,
        Palette::Jade => &JADE_GRADIENT,
        Palette::Amber => &AMBER_GRADIENT,
        Palette::Slate => &SLATE_GRADIENT,
        Palette::Pastel => &PASTEL_GRADIENT,
        Palette::Ink => &INK_GRADIENT,
        Palette::Copper => &COPPER_GRADIENT,
        Palette::Custom(_) => &FOREST_GRADIENT,
    }
}

/// Returns the RGB color gradient for a given palette.
///
/// Each gradient contains 11 RGB color stops ranging from dark to bright.
/// For custom palettes, this function will panic - custom palettes require
/// special handling via direct color interpolation.
pub fn get_rgb_gradient(palette: Palette) -> &'static [RgbColor; 11] {
    match palette {
        Palette::Organic => &ORGANIC_RGB,
        Palette::Heat => &HEAT_RGB,
        Palette::Ocean => &OCEAN_RGB,
        Palette::Mono => &MONO_RGB,
        Palette::Forest => &FOREST_RGB,
        Palette::Neon => &NEON_RGB,
        Palette::Warm => &WARM_RGB,
        Palette::Vibrant => &VIBRANT_RGB,
        Palette::LegibleMono => &LEGIBLEMONO_RGB,
        Palette::Slime => &SLIME_RGB,
        Palette::Mold => &MOLD_RGB,
        Palette::Fungus => &FUNGUS_RGB,
        Palette::Swamp => &SWAMP_RGB,
        Palette::Moss => &MOSS_RGB,
        Palette::Cosmic => &COSMIC_RGB,
        Palette::Ethereal => &ETHEREAL_RGB,
        Palette::Jade => &JADE_RGB,
        Palette::Amber => &AMBER_RGB,
        Palette::Slate => &SLATE_RGB,
        Palette::Pastel => &PASTEL_RGB,
        Palette::Ink => &INK_RGB,
        Palette::Copper => &COPPER_RGB,
        Palette::Custom(_) => panic!("Custom palette requires special handling"),
    }
}

// =============================================================================
// OKLch gradient definitions (11 control points per palette)
//
// Values derived from the original RGB palettes' measured OKLch coordinates,
// then smoothed for perceptually uniform interpolation. Chroma values are
// deliberately modest — terminal screensavers look best with muted, organic
// tones that sit comfortably against dark backgrounds.
//
// Each stop: { position, l (lightness 0–1), c (chroma 0–0.35), h (hue °0–360) }
// =============================================================================

// Organic: dark neutral → warm reddish-brown → olive → green → light aqua
const ORGANIC_OKLCH: [OklchStop; 11] = [
    OklchStop {
        position: 0.0,
        l: 0.182,
        c: 0.000,
        h: 0.0,
    },
    OklchStop {
        position: 0.1,
        l: 0.277,
        c: 0.000,
        h: 0.0,
    },
    OklchStop {
        position: 0.2,
        l: 0.274,
        c: 0.077,
        h: 24.0,
    },
    OklchStop {
        position: 0.3,
        l: 0.363,
        c: 0.087,
        h: 22.0,
    },
    OklchStop {
        position: 0.4,
        l: 0.429,
        c: 0.113,
        h: 30.0,
    },
    OklchStop {
        position: 0.5,
        l: 0.481,
        c: 0.146,
        h: 24.0,
    },
    OklchStop {
        position: 0.6,
        l: 0.511,
        c: 0.073,
        h: 88.0,
    },
    OklchStop {
        position: 0.7,
        l: 0.567,
        c: 0.103,
        h: 129.0,
    },
    OklchStop {
        position: 0.8,
        l: 0.636,
        c: 0.140,
        h: 144.0,
    },
    OklchStop {
        position: 0.9,
        l: 0.730,
        c: 0.124,
        h: 153.0,
    },
    OklchStop {
        position: 1.0,
        l: 0.842,
        c: 0.076,
        h: 175.0,
    },
];

// Heat: dark brownish-red → vivid red → orange → warm yellow → near-white
const HEAT_OKLCH: [OklchStop; 11] = [
    OklchStop {
        position: 0.0,
        l: 0.220,
        c: 0.033,
        h: 20.0,
    },
    OklchStop {
        position: 0.1,
        l: 0.220,
        c: 0.033,
        h: 20.0,
    },
    OklchStop {
        position: 0.2,
        l: 0.274,
        c: 0.077,
        h: 24.0,
    },
    OklchStop {
        position: 0.3,
        l: 0.353,
        c: 0.124,
        h: 26.0,
    },
    OklchStop {
        position: 0.4,
        l: 0.433,
        c: 0.164,
        h: 28.0,
    },
    OklchStop {
        position: 0.5,
        l: 0.523,
        c: 0.187,
        h: 29.0,
    },
    OklchStop {
        position: 0.6,
        l: 0.571,
        c: 0.171,
        h: 34.0,
    },
    OklchStop {
        position: 0.7,
        l: 0.628,
        c: 0.153,
        h: 43.0,
    },
    OklchStop {
        position: 0.8,
        l: 0.709,
        c: 0.135,
        h: 62.0,
    },
    OklchStop {
        position: 0.9,
        l: 0.797,
        c: 0.130,
        h: 82.0,
    },
    OklchStop {
        position: 1.0,
        l: 0.901,
        c: 0.057,
        h: 85.0,
    },
];

// Ocean: near-black → deep blue → rich blue → light blue
const OCEAN_OKLCH: [OklchStop; 11] = [
    OklchStop {
        position: 0.0,
        l: 0.182,
        c: 0.000,
        h: 264.0,
    },
    OklchStop {
        position: 0.1,
        l: 0.210,
        c: 0.058,
        h: 280.0,
    },
    OklchStop {
        position: 0.2,
        l: 0.231,
        c: 0.067,
        h: 274.0,
    },
    OklchStop {
        position: 0.3,
        l: 0.252,
        c: 0.076,
        h: 270.0,
    },
    OklchStop {
        position: 0.4,
        l: 0.285,
        c: 0.078,
        h: 263.0,
    },
    OklchStop {
        position: 0.5,
        l: 0.327,
        c: 0.094,
        h: 263.0,
    },
    OklchStop {
        position: 0.6,
        l: 0.400,
        c: 0.111,
        h: 259.0,
    },
    OklchStop {
        position: 0.7,
        l: 0.471,
        c: 0.126,
        h: 257.0,
    },
    OklchStop {
        position: 0.8,
        l: 0.540,
        c: 0.140,
        h: 257.0,
    },
    OklchStop {
        position: 0.9,
        l: 0.629,
        c: 0.144,
        h: 251.0,
    },
    OklchStop {
        position: 1.0,
        l: 0.713,
        c: 0.134,
        h: 245.0,
    },
];

// Mono: pure achromatic ramp
const MONO_OKLCH: [OklchStop; 11] = [
    OklchStop {
        position: 0.0,
        l: 0.182,
        c: 0.000,
        h: 0.0,
    },
    OklchStop {
        position: 0.1,
        l: 0.256,
        c: 0.000,
        h: 0.0,
    },
    OklchStop {
        position: 0.2,
        l: 0.337,
        c: 0.000,
        h: 0.0,
    },
    OklchStop {
        position: 0.3,
        l: 0.413,
        c: 0.000,
        h: 0.0,
    },
    OklchStop {
        position: 0.4,
        l: 0.485,
        c: 0.000,
        h: 0.0,
    },
    OklchStop {
        position: 0.5,
        l: 0.556,
        c: 0.000,
        h: 0.0,
    },
    OklchStop {
        position: 0.6,
        l: 0.623,
        c: 0.000,
        h: 0.0,
    },
    OklchStop {
        position: 0.7,
        l: 0.689,
        c: 0.000,
        h: 0.0,
    },
    OklchStop {
        position: 0.8,
        l: 0.754,
        c: 0.000,
        h: 0.0,
    },
    OklchStop {
        position: 0.9,
        l: 0.817,
        c: 0.000,
        h: 0.0,
    },
    OklchStop {
        position: 1.0,
        l: 0.879,
        c: 0.000,
        h: 0.0,
    },
];

// Forest: subdued dark greens → medium greens → gentle light green
const FOREST_OKLCH: [OklchStop; 11] = [
    OklchStop {
        position: 0.0,
        l: 0.253,
        c: 0.045,
        h: 144.0,
    },
    OklchStop {
        position: 0.1,
        l: 0.292,
        c: 0.051,
        h: 140.0,
    },
    OklchStop {
        position: 0.2,
        l: 0.330,
        c: 0.056,
        h: 136.0,
    },
    OklchStop {
        position: 0.3,
        l: 0.396,
        c: 0.079,
        h: 136.0,
    },
    OklchStop {
        position: 0.4,
        l: 0.459,
        c: 0.100,
        h: 136.0,
    },
    OklchStop {
        position: 0.5,
        l: 0.520,
        c: 0.115,
        h: 138.0,
    },
    OklchStop {
        position: 0.6,
        l: 0.580,
        c: 0.130,
        h: 139.0,
    },
    OklchStop {
        position: 0.7,
        l: 0.646,
        c: 0.129,
        h: 138.0,
    },
    OklchStop {
        position: 0.8,
        l: 0.710,
        c: 0.127,
        h: 138.0,
    },
    OklchStop {
        position: 0.9,
        l: 0.780,
        c: 0.109,
        h: 137.0,
    },
    OklchStop {
        position: 1.0,
        l: 0.853,
        c: 0.080,
        h: 139.0,
    },
];

// Neon: muted deep purple → gentle blue-lavender → vivid purple flash at peak
const NEON_OKLCH: [OklchStop; 11] = [
    OklchStop {
        position: 0.0,
        l: 0.185,
        c: 0.095,
        h: 307.0,
    },
    OklchStop {
        position: 0.1,
        l: 0.227,
        c: 0.092,
        h: 308.0,
    },
    OklchStop {
        position: 0.2,
        l: 0.272,
        c: 0.105,
        h: 303.0,
    },
    OklchStop {
        position: 0.3,
        l: 0.334,
        c: 0.102,
        h: 296.0,
    },
    OklchStop {
        position: 0.4,
        l: 0.433,
        c: 0.097,
        h: 289.0,
    },
    OklchStop {
        position: 0.5,
        l: 0.529,
        c: 0.093,
        h: 283.0,
    },
    OklchStop {
        position: 0.6,
        l: 0.623,
        c: 0.091,
        h: 276.0,
    },
    OklchStop {
        position: 0.7,
        l: 0.713,
        c: 0.091,
        h: 270.0,
    },
    OklchStop {
        position: 0.8,
        l: 0.804,
        c: 0.075,
        h: 268.0,
    },
    OklchStop {
        position: 0.9,
        l: 0.891,
        c: 0.053,
        h: 262.0,
    },
    OklchStop {
        position: 1.0,
        l: 0.544,
        c: 0.211,
        h: 311.0,
    }, // vivid purple flash
];

// Warm: dark warm brown → amber → soft warm near-white
const WARM_OKLCH: [OklchStop; 11] = [
    OklchStop {
        position: 0.0,
        l: 0.220,
        c: 0.033,
        h: 20.0,
    },
    OklchStop {
        position: 0.1,
        l: 0.274,
        c: 0.050,
        h: 39.0,
    },
    OklchStop {
        position: 0.2,
        l: 0.327,
        c: 0.065,
        h: 41.0,
    },
    OklchStop {
        position: 0.3,
        l: 0.403,
        c: 0.086,
        h: 44.0,
    },
    OklchStop {
        position: 0.4,
        l: 0.475,
        c: 0.107,
        h: 45.0,
    },
    OklchStop {
        position: 0.5,
        l: 0.554,
        c: 0.120,
        h: 48.0,
    },
    OklchStop {
        position: 0.6,
        l: 0.631,
        c: 0.131,
        h: 48.0,
    },
    OklchStop {
        position: 0.7,
        l: 0.699,
        c: 0.115,
        h: 60.0,
    },
    OklchStop {
        position: 0.8,
        l: 0.769,
        c: 0.106,
        h: 74.0,
    },
    OklchStop {
        position: 0.9,
        l: 0.845,
        c: 0.084,
        h: 84.0,
    },
    OklchStop {
        position: 1.0,
        l: 0.925,
        c: 0.041,
        h: 92.0,
    },
];

// Vibrant: muted purple → neutral → olive → warm gold → orange
const VIBRANT_OKLCH: [OklchStop; 11] = [
    OklchStop {
        position: 0.0,
        l: 0.257,
        c: 0.080,
        h: 317.0,
    },
    OklchStop {
        position: 0.1,
        l: 0.320,
        c: 0.073,
        h: 306.0,
    },
    OklchStop {
        position: 0.2,
        l: 0.395,
        c: 0.070,
        h: 306.0,
    },
    OklchStop {
        position: 0.3,
        l: 0.452,
        c: 0.027,
        h: 18.0,
    },
    OklchStop {
        position: 0.4,
        l: 0.513,
        c: 0.062,
        h: 84.0,
    },
    OklchStop {
        position: 0.5,
        l: 0.576,
        c: 0.101,
        h: 95.0,
    },
    OklchStop {
        position: 0.6,
        l: 0.640,
        c: 0.123,
        h: 98.0,
    },
    OklchStop {
        position: 0.7,
        l: 0.702,
        c: 0.138,
        h: 100.0,
    },
    OklchStop {
        position: 0.8,
        l: 0.703,
        c: 0.133,
        h: 82.0,
    },
    OklchStop {
        position: 0.9,
        l: 0.709,
        c: 0.135,
        h: 62.0,
    },
    OklchStop {
        position: 1.0,
        l: 0.720,
        c: 0.150,
        h: 44.0,
    },
];

// LegibleMono: high-contrast achromatic ramp
const LEGIBLEMONO_OKLCH: [OklchStop; 11] = [
    OklchStop {
        position: 0.0,
        l: 0.235,
        c: 0.000,
        h: 0.0,
    },
    OklchStop {
        position: 0.1,
        l: 0.317,
        c: 0.000,
        h: 0.0,
    },
    OklchStop {
        position: 0.2,
        l: 0.394,
        c: 0.000,
        h: 0.0,
    },
    OklchStop {
        position: 0.3,
        l: 0.468,
        c: 0.000,
        h: 0.0,
    },
    OklchStop {
        position: 0.4,
        l: 0.538,
        c: 0.000,
        h: 0.0,
    },
    OklchStop {
        position: 0.5,
        l: 0.607,
        c: 0.000,
        h: 0.0,
    },
    OklchStop {
        position: 0.6,
        l: 0.673,
        c: 0.000,
        h: 0.0,
    },
    OklchStop {
        position: 0.7,
        l: 0.738,
        c: 0.000,
        h: 0.0,
    },
    OklchStop {
        position: 0.8,
        l: 0.802,
        c: 0.000,
        h: 0.0,
    },
    OklchStop {
        position: 0.9,
        l: 0.864,
        c: 0.000,
        h: 0.0,
    },
    OklchStop {
        position: 1.0,
        l: 0.925,
        c: 0.000,
        h: 0.0,
    },
];

// Slime: intentionally vivid — dark green → radioactive green → white
const SLIME_OKLCH: [OklchStop; 11] = [
    OklchStop {
        position: 0.0,
        l: 0.253,
        c: 0.045,
        h: 144.0,
    },
    OklchStop {
        position: 0.1,
        l: 0.421,
        c: 0.143,
        h: 143.0,
    },
    OklchStop {
        position: 0.2,
        l: 0.542,
        c: 0.170,
        h: 145.0,
    },
    OklchStop {
        position: 0.3,
        l: 0.653,
        c: 0.222,
        h: 143.0,
    },
    OklchStop {
        position: 0.4,
        l: 0.729,
        c: 0.223,
        h: 143.0,
    },
    OklchStop {
        position: 0.5,
        l: 0.778,
        c: 0.238,
        h: 137.0,
    },
    OklchStop {
        position: 0.6,
        l: 0.840,
        c: 0.165,
        h: 144.0,
    },
    OklchStop {
        position: 0.7,
        l: 0.882,
        c: 0.144,
        h: 142.0,
    },
    OklchStop {
        position: 0.8,
        l: 0.927,
        c: 0.111,
        h: 141.0,
    },
    OklchStop {
        position: 0.9,
        l: 0.961,
        c: 0.081,
        h: 134.0,
    },
    OklchStop {
        position: 1.0,
        l: 1.000,
        c: 0.000,
        h: 0.0,
    },
];

// Mold: steep jump from dark to yellow-olive, then gentle drift to green
const MOLD_OKLCH: [OklchStop; 11] = [
    OklchStop {
        position: 0.0,
        l: 0.277,
        c: 0.000,
        h: 0.0,
    },
    OklchStop {
        position: 0.1,
        l: 0.603,
        c: 0.132,
        h: 110.0,
    }, // steep jump
    OklchStop {
        position: 0.2,
        l: 0.708,
        c: 0.150,
        h: 105.0,
    },
    OklchStop {
        position: 0.3,
        l: 0.770,
        c: 0.153,
        h: 105.0,
    },
    OklchStop {
        position: 0.4,
        l: 0.823,
        c: 0.140,
        h: 102.0,
    },
    OklchStop {
        position: 0.5,
        l: 0.857,
        c: 0.113,
        h: 100.0,
    },
    OklchStop {
        position: 0.6,
        l: 0.873,
        c: 0.107,
        h: 100.0,
    },
    OklchStop {
        position: 0.7,
        l: 0.890,
        c: 0.096,
        h: 98.0,
    },
    OklchStop {
        position: 0.8,
        l: 0.884,
        c: 0.102,
        h: 145.0,
    }, // hue shifts green
    OklchStop {
        position: 0.9,
        l: 0.899,
        c: 0.101,
        h: 145.0,
    },
    OklchStop {
        position: 1.0,
        l: 0.914,
        c: 0.101,
        h: 145.0,
    },
];

// Fungus: dark purple → vivid magenta → mauve → hue-shift to golden → cool white
const FUNGUS_OKLCH: [OklchStop; 11] = [
    OklchStop {
        position: 0.0,
        l: 0.200,
        c: 0.013,
        h: 326.0,
    },
    OklchStop {
        position: 0.1,
        l: 0.341,
        c: 0.157,
        h: 328.0,
    },
    OklchStop {
        position: 0.2,
        l: 0.437,
        c: 0.201,
        h: 328.0,
    },
    OklchStop {
        position: 0.3,
        l: 0.573,
        c: 0.264,
        h: 328.0,
    },
    OklchStop {
        position: 0.4,
        l: 0.595,
        c: 0.259,
        h: 339.0,
    },
    OklchStop {
        position: 0.5,
        l: 0.587,
        c: 0.084,
        h: 334.0,
    }, // chroma drops
    OklchStop {
        position: 0.6,
        l: 0.620,
        c: 0.083,
        h: 334.0,
    },
    OklchStop {
        position: 0.7,
        l: 0.652,
        c: 0.082,
        h: 334.0,
    },
    OklchStop {
        position: 0.8,
        l: 0.680,
        c: 0.099,
        h: 91.0,
    }, // hue-shift golden
    OklchStop {
        position: 0.9,
        l: 0.835,
        c: 0.127,
        h: 104.0,
    },
    OklchStop {
        position: 1.0,
        l: 0.952,
        c: 0.023,
        h: 258.0,
    }, // cool near-white
];

// Swamp: achromatic dark → barely tinted → subdued teal-green
const SWAMP_OKLCH: [OklchStop; 11] = [
    OklchStop {
        position: 0.0,
        l: 0.182,
        c: 0.000,
        h: 0.0,
    },
    OklchStop {
        position: 0.1,
        l: 0.256,
        c: 0.000,
        h: 0.0,
    },
    OklchStop {
        position: 0.2,
        l: 0.337,
        c: 0.000,
        h: 0.0,
    },
    OklchStop {
        position: 0.3,
        l: 0.442,
        c: 0.017,
        h: 129.0,
    },
    OklchStop {
        position: 0.4,
        l: 0.531,
        c: 0.017,
        h: 129.0,
    },
    OklchStop {
        position: 0.5,
        l: 0.627,
        c: 0.025,
        h: 135.0,
    },
    OklchStop {
        position: 0.6,
        l: 0.537,
        c: 0.116,
        h: 163.0,
    }, // jump to teal
    OklchStop {
        position: 0.7,
        l: 0.556,
        c: 0.106,
        h: 174.0,
    },
    OklchStop {
        position: 0.8,
        l: 0.600,
        c: 0.116,
        h: 172.0,
    },
    OklchStop {
        position: 0.9,
        l: 0.672,
        c: 0.131,
        h: 172.0,
    },
    OklchStop {
        position: 1.0,
        l: 0.713,
        c: 0.140,
        h: 171.0,
    },
];

// Moss: very gentle dark-to-light green, hue drifts toward yellow-green
const MOSS_OKLCH: [OklchStop; 11] = [
    OklchStop {
        position: 0.0,
        l: 0.237,
        c: 0.035,
        h: 144.0,
    },
    OklchStop {
        position: 0.1,
        l: 0.273,
        c: 0.049,
        h: 141.0,
    },
    OklchStop {
        position: 0.2,
        l: 0.327,
        c: 0.062,
        h: 139.0,
    },
    OklchStop {
        position: 0.3,
        l: 0.393,
        c: 0.082,
        h: 139.0,
    },
    OklchStop {
        position: 0.4,
        l: 0.459,
        c: 0.100,
        h: 136.0,
    },
    OklchStop {
        position: 0.5,
        l: 0.526,
        c: 0.109,
        h: 134.0,
    },
    OklchStop {
        position: 0.6,
        l: 0.593,
        c: 0.114,
        h: 132.0,
    },
    OklchStop {
        position: 0.7,
        l: 0.658,
        c: 0.118,
        h: 131.0,
    },
    OklchStop {
        position: 0.8,
        l: 0.713,
        c: 0.112,
        h: 126.0,
    },
    OklchStop {
        position: 0.9,
        l: 0.769,
        c: 0.102,
        h: 121.0,
    },
    OklchStop {
        position: 1.0,
        l: 0.836,
        c: 0.093,
        h: 118.0,
    },
];

// Cosmic: deep space purple → rich purple → gentle lavender fade
const COSMIC_OKLCH: [OklchStop; 11] = [
    OklchStop {
        position: 0.0,
        l: 0.179,
        c: 0.059,
        h: 295.0,
    },
    OklchStop {
        position: 0.1,
        l: 0.220,
        c: 0.082,
        h: 294.0,
    },
    OklchStop {
        position: 0.2,
        l: 0.281,
        c: 0.117,
        h: 298.0,
    },
    OklchStop {
        position: 0.3,
        l: 0.347,
        c: 0.144,
        h: 299.0,
    },
    OklchStop {
        position: 0.4,
        l: 0.423,
        c: 0.156,
        h: 298.0,
    },
    OklchStop {
        position: 0.5,
        l: 0.523,
        c: 0.154,
        h: 300.0,
    },
    OklchStop {
        position: 0.6,
        l: 0.616,
        c: 0.137,
        h: 303.0,
    },
    OklchStop {
        position: 0.7,
        l: 0.708,
        c: 0.121,
        h: 307.0,
    },
    OklchStop {
        position: 0.8,
        l: 0.787,
        c: 0.096,
        h: 305.0,
    },
    OklchStop {
        position: 0.9,
        l: 0.864,
        c: 0.065,
        h: 305.0,
    },
    OklchStop {
        position: 1.0,
        l: 0.940,
        c: 0.035,
        h: 304.0,
    },
];

// Ethereal: extremely muted purple tint — the subtlest palette
const ETHEREAL_OKLCH: [OklchStop; 11] = [
    OklchStop {
        position: 0.0,
        l: 0.171,
        c: 0.010,
        h: 285.0,
    },
    OklchStop {
        position: 0.1,
        l: 0.341,
        c: 0.053,
        h: 299.0,
    },
    OklchStop {
        position: 0.2,
        l: 0.453,
        c: 0.050,
        h: 300.0,
    },
    OklchStop {
        position: 0.3,
        l: 0.558,
        c: 0.047,
        h: 300.0,
    },
    OklchStop {
        position: 0.4,
        l: 0.659,
        c: 0.045,
        h: 300.0,
    },
    OklchStop {
        position: 0.5,
        l: 0.756,
        c: 0.044,
        h: 301.0,
    },
    OklchStop {
        position: 0.6,
        l: 0.849,
        c: 0.036,
        h: 304.0,
    },
    OklchStop {
        position: 0.7,
        l: 0.908,
        c: 0.029,
        h: 308.0,
    },
    OklchStop {
        position: 0.8,
        l: 0.952,
        c: 0.023,
        h: 315.0,
    },
    OklchStop {
        position: 0.9,
        l: 0.967,
        c: 0.023,
        h: 315.0,
    },
    OklchStop {
        position: 1.0,
        l: 0.970,
        c: 0.021,
        h: 338.0,
    },
];

// =============================================================================
// Additional palettes: OKLch stops + RGB fallback + 256-color codes.
// The OKLch stops are hand-authored; the RGB and 256-color arrays are derived
// from them via oklch_to_srgb + rgb_to_256 to stay perceptually consistent.
// =============================================================================

// Jade: dark green → mid-saturation jade → pale green
const JADE_OKLCH: [OklchStop; 11] = [
    OklchStop {
        position: 0.0,
        l: 0.2,
        c: 0.03,
        h: 155.0,
    },
    OklchStop {
        position: 0.1,
        l: 0.26,
        c: 0.05,
        h: 153.0,
    },
    OklchStop {
        position: 0.2,
        l: 0.32,
        c: 0.07,
        h: 152.0,
    },
    OklchStop {
        position: 0.3,
        l: 0.38,
        c: 0.09,
        h: 151.0,
    },
    OklchStop {
        position: 0.4,
        l: 0.44,
        c: 0.11,
        h: 150.0,
    },
    OklchStop {
        position: 0.5,
        l: 0.5,
        c: 0.13,
        h: 150.0,
    },
    OklchStop {
        position: 0.6,
        l: 0.56,
        c: 0.14,
        h: 149.0,
    },
    OklchStop {
        position: 0.7,
        l: 0.63,
        c: 0.14,
        h: 150.0,
    },
    OklchStop {
        position: 0.8,
        l: 0.71,
        c: 0.12,
        h: 152.0,
    },
    OklchStop {
        position: 0.9,
        l: 0.8,
        c: 0.09,
        h: 155.0,
    },
    OklchStop {
        position: 1.0,
        l: 0.9,
        c: 0.05,
        h: 158.0,
    },
];
const JADE_RGB: [RgbColor; 11] = [
    RgbColor {
        r: 10,
        g: 26,
        b: 16,
    },
    RgbColor {
        r: 14,
        g: 43,
        b: 24,
    },
    RgbColor {
        r: 16,
        g: 61,
        b: 32,
    },
    RgbColor {
        r: 18,
        g: 79,
        b: 40,
    },
    RgbColor {
        r: 21,
        g: 99,
        b: 47,
    },
    RgbColor {
        r: 19,
        g: 119,
        b: 56,
    },
    RgbColor {
        r: 37,
        g: 138,
        b: 67,
    },
    RgbColor {
        r: 60,
        g: 160,
        b: 89,
    },
    RgbColor {
        r: 98,
        g: 183,
        b: 124,
    },
    RgbColor {
        r: 142,
        g: 207,
        b: 164,
    },
    RgbColor {
        r: 195,
        g: 233,
        b: 210,
    },
];
const JADE_GRADIENT: [u8; 11] = [0, 22, 23, 29, 29, 29, 71, 72, 114, 151, 194];

// Amber: dark honey → gold → pale warm
const AMBER_OKLCH: [OklchStop; 11] = [
    OklchStop {
        position: 0.0,
        l: 0.22,
        c: 0.04,
        h: 50.0,
    },
    OklchStop {
        position: 0.1,
        l: 0.28,
        c: 0.07,
        h: 52.0,
    },
    OklchStop {
        position: 0.2,
        l: 0.34,
        c: 0.1,
        h: 54.0,
    },
    OklchStop {
        position: 0.3,
        l: 0.4,
        c: 0.13,
        h: 56.0,
    },
    OklchStop {
        position: 0.4,
        l: 0.47,
        c: 0.15,
        h: 60.0,
    },
    OklchStop {
        position: 0.5,
        l: 0.54,
        c: 0.16,
        h: 66.0,
    },
    OklchStop {
        position: 0.6,
        l: 0.61,
        c: 0.15,
        h: 72.0,
    },
    OklchStop {
        position: 0.7,
        l: 0.69,
        c: 0.14,
        h: 78.0,
    },
    OklchStop {
        position: 0.8,
        l: 0.77,
        c: 0.12,
        h: 84.0,
    },
    OklchStop {
        position: 0.9,
        l: 0.85,
        c: 0.09,
        h: 90.0,
    },
    OklchStop {
        position: 1.0,
        l: 0.93,
        c: 0.05,
        h: 95.0,
    },
];
const AMBER_RGB: [RgbColor; 11] = [
    RgbColor { r: 41, g: 21, b: 8 },
    RgbColor { r: 67, g: 29, b: 1 },
    RgbColor { r: 93, g: 37, b: 0 },
    RgbColor {
        r: 121,
        g: 45,
        b: 0,
    },
    RgbColor {
        r: 148,
        g: 62,
        b: 0,
    },
    RgbColor {
        r: 170,
        g: 85,
        b: 0,
    },
    RgbColor {
        r: 185,
        g: 113,
        b: 0,
    },
    RgbColor {
        r: 202,
        g: 143,
        b: 18,
    },
    RgbColor {
        r: 216,
        g: 173,
        b: 82,
    },
    RgbColor {
        r: 229,
        g: 204,
        b: 137,
    },
    RgbColor {
        r: 242,
        g: 232,
        b: 195,
    },
];
const AMBER_GRADIENT: [u8; 11] = [52, 58, 94, 94, 130, 136, 172, 178, 180, 187, 230];

// Slate: cool stone grey ramp, very low chroma
const SLATE_OKLCH: [OklchStop; 11] = [
    OklchStop {
        position: 0.0,
        l: 0.2,
        c: 0.005,
        h: 250.0,
    },
    OklchStop {
        position: 0.1,
        l: 0.27,
        c: 0.01,
        h: 250.0,
    },
    OklchStop {
        position: 0.2,
        l: 0.34,
        c: 0.015,
        h: 248.0,
    },
    OklchStop {
        position: 0.3,
        l: 0.41,
        c: 0.02,
        h: 246.0,
    },
    OklchStop {
        position: 0.4,
        l: 0.48,
        c: 0.025,
        h: 245.0,
    },
    OklchStop {
        position: 0.5,
        l: 0.55,
        c: 0.028,
        h: 245.0,
    },
    OklchStop {
        position: 0.6,
        l: 0.62,
        c: 0.028,
        h: 246.0,
    },
    OklchStop {
        position: 0.7,
        l: 0.69,
        c: 0.025,
        h: 248.0,
    },
    OklchStop {
        position: 0.8,
        l: 0.76,
        c: 0.02,
        h: 250.0,
    },
    OklchStop {
        position: 0.9,
        l: 0.84,
        c: 0.014,
        h: 252.0,
    },
    OklchStop {
        position: 1.0,
        l: 0.92,
        c: 0.008,
        h: 254.0,
    },
];
const SLATE_RGB: [RgbColor; 11] = [
    RgbColor {
        r: 20,
        g: 22,
        b: 24,
    },
    RgbColor {
        r: 35,
        g: 39,
        b: 43,
    },
    RgbColor {
        r: 50,
        g: 57,
        b: 64,
    },
    RgbColor {
        r: 66,
        g: 76,
        b: 85,
    },
    RgbColor {
        r: 82,
        g: 95,
        b: 107,
    },
    RgbColor {
        r: 100,
        g: 116,
        b: 129,
    },
    RgbColor {
        r: 121,
        g: 136,
        b: 150,
    },
    RgbColor {
        r: 144,
        g: 157,
        b: 170,
    },
    RgbColor {
        r: 168,
        g: 178,
        b: 190,
    },
    RgbColor {
        r: 196,
        g: 203,
        b: 212,
    },
    RgbColor {
        r: 225,
        g: 229,
        b: 234,
    },
];
const SLATE_GRADIENT: [u8; 11] = [0, 59, 59, 60, 102, 8, 8, 145, 7, 7, 189];

// Pastel: high-key airy multi-hue drift, low chroma
const PASTEL_OKLCH: [OklchStop; 11] = [
    OklchStop {
        position: 0.0,
        l: 0.55,
        c: 0.04,
        h: 320.0,
    },
    OklchStop {
        position: 0.1,
        l: 0.6,
        c: 0.05,
        h: 340.0,
    },
    OklchStop {
        position: 0.2,
        l: 0.65,
        c: 0.06,
        h: 20.0,
    },
    OklchStop {
        position: 0.3,
        l: 0.7,
        c: 0.06,
        h: 60.0,
    },
    OklchStop {
        position: 0.4,
        l: 0.74,
        c: 0.07,
        h: 120.0,
    },
    OklchStop {
        position: 0.5,
        l: 0.78,
        c: 0.07,
        h: 160.0,
    },
    OklchStop {
        position: 0.6,
        l: 0.82,
        c: 0.07,
        h: 200.0,
    },
    OklchStop {
        position: 0.7,
        l: 0.86,
        c: 0.06,
        h: 240.0,
    },
    OklchStop {
        position: 0.8,
        l: 0.9,
        c: 0.05,
        h: 280.0,
    },
    OklchStop {
        position: 0.9,
        l: 0.94,
        c: 0.03,
        h: 300.0,
    },
    OklchStop {
        position: 1.0,
        l: 0.97,
        c: 0.015,
        h: 320.0,
    },
];
const PASTEL_RGB: [RgbColor; 11] = [
    RgbColor {
        r: 124,
        g: 106,
        b: 128,
    },
    RgbColor {
        r: 149,
        g: 117,
        b: 138,
    },
    RgbColor {
        r: 177,
        g: 129,
        b: 128,
    },
    RgbColor {
        r: 187,
        g: 150,
        b: 121,
    },
    RgbColor {
        r: 165,
        g: 178,
        b: 128,
    },
    RgbColor {
        r: 144,
        g: 198,
        b: 168,
    },
    RgbColor {
        r: 141,
        g: 210,
        b: 214,
    },
    RgbColor {
        r: 174,
        g: 215,
        b: 245,
    },
    RgbColor {
        r: 215,
        g: 219,
        b: 255,
    },
    RgbColor {
        r: 238,
        g: 231,
        b: 253,
    },
    RgbColor {
        r: 250,
        g: 242,
        b: 252,
    },
];
const PASTEL_GRADIENT: [u8; 11] = [8, 8, 145, 180, 145, 151, 152, 153, 189, 15, 15];

// Ink: duotone cool-dark ink → warm paper white
const INK_OKLCH: [OklchStop; 11] = [
    OklchStop {
        position: 0.0,
        l: 0.16,
        c: 0.012,
        h: 260.0,
    },
    OklchStop {
        position: 0.1,
        l: 0.18,
        c: 0.012,
        h: 260.0,
    },
    OklchStop {
        position: 0.2,
        l: 0.21,
        c: 0.01,
        h: 260.0,
    },
    OklchStop {
        position: 0.3,
        l: 0.25,
        c: 0.008,
        h: 262.0,
    },
    OklchStop {
        position: 0.4,
        l: 0.31,
        c: 0.006,
        h: 264.0,
    },
    OklchStop {
        position: 0.5,
        l: 0.4,
        c: 0.005,
        h: 266.0,
    },
    OklchStop {
        position: 0.6,
        l: 0.52,
        c: 0.004,
        h: 268.0,
    },
    OklchStop {
        position: 0.7,
        l: 0.65,
        c: 0.004,
        h: 270.0,
    },
    OklchStop {
        position: 0.8,
        l: 0.78,
        c: 0.005,
        h: 80.0,
    },
    OklchStop {
        position: 0.9,
        l: 0.88,
        c: 0.006,
        h: 85.0,
    },
    OklchStop {
        position: 1.0,
        l: 0.95,
        c: 0.004,
        h: 90.0,
    },
];
const INK_RGB: [RgbColor; 11] = [
    RgbColor {
        r: 10,
        g: 13,
        b: 18,
    },
    RgbColor {
        r: 14,
        g: 18,
        b: 23,
    },
    RgbColor {
        r: 22,
        g: 24,
        b: 29,
    },
    RgbColor {
        r: 32,
        g: 34,
        b: 38,
    },
    RgbColor {
        r: 47,
        g: 48,
        b: 51,
    },
    RgbColor {
        r: 70,
        g: 72,
        b: 74,
    },
    RgbColor {
        r: 104,
        g: 105,
        b: 107,
    },
    RgbColor {
        r: 142,
        g: 143,
        b: 146,
    },
    RgbColor {
        r: 185,
        g: 183,
        b: 180,
    },
    RgbColor {
        r: 217,
        g: 215,
        b: 211,
    },
    RgbColor {
        r: 239,
        g: 238,
        b: 235,
    },
];
const INK_GRADIENT: [u8; 11] = [0, 0, 0, 59, 59, 59, 8, 8, 7, 7, 15];

// Copper: oxidized rust → desaturated crossover → verdigris teal
const COPPER_OKLCH: [OklchStop; 11] = [
    OklchStop {
        position: 0.0,
        l: 0.22,
        c: 0.05,
        h: 32.0,
    },
    OklchStop {
        position: 0.1,
        l: 0.29,
        c: 0.08,
        h: 33.0,
    },
    OklchStop {
        position: 0.2,
        l: 0.36,
        c: 0.11,
        h: 35.0,
    },
    OklchStop {
        position: 0.3,
        l: 0.43,
        c: 0.12,
        h: 38.0,
    },
    OklchStop {
        position: 0.4,
        l: 0.49,
        c: 0.1,
        h: 50.0,
    },
    OklchStop {
        position: 0.5,
        l: 0.55,
        c: 0.06,
        h: 120.0,
    },
    OklchStop {
        position: 0.6,
        l: 0.6,
        c: 0.08,
        h: 165.0,
    },
    OklchStop {
        position: 0.7,
        l: 0.66,
        c: 0.11,
        h: 175.0,
    },
    OklchStop {
        position: 0.8,
        l: 0.72,
        c: 0.12,
        h: 180.0,
    },
    OklchStop {
        position: 0.9,
        l: 0.8,
        c: 0.1,
        h: 183.0,
    },
    OklchStop {
        position: 1.0,
        l: 0.88,
        c: 0.06,
        h: 185.0,
    },
];
const COPPER_RGB: [RgbColor; 11] = [
    RgbColor {
        r: 46,
        g: 16,
        b: 11,
    },
    RgbColor {
        r: 75,
        g: 24,
        b: 14,
    },
    RgbColor {
        r: 107,
        g: 32,
        b: 14,
    },
    RgbColor {
        r: 132,
        g: 50,
        b: 23,
    },
    RgbColor {
        r: 141,
        g: 77,
        b: 39,
    },
    RgbColor {
        r: 109,
        g: 119,
        b: 79,
    },
    RgbColor {
        r: 78,
        g: 144,
        b: 116,
    },
    RgbColor {
        r: 50,
        g: 168,
        b: 142,
    },
    RgbColor {
        r: 47,
        g: 189,
        b: 167,
    },
    RgbColor {
        r: 107,
        g: 211,
        b: 196,
    },
    RgbColor {
        r: 171,
        g: 229,
        b: 221,
    },
];
const COPPER_GRADIENT: [u8; 11] = [52, 52, 1, 130, 137, 102, 108, 73, 79, 116, 152];

/// Returns the OKLch gradient control points for a given built-in palette.
///
/// These stops define the palette in perceptual OKLch space. The caller is
/// responsible for interpolating between them and converting to sRGB.
///
/// # Panics
/// Panics for `Palette::Custom` — custom palettes have no OKLch definition.
pub fn get_oklch_gradient(palette: Palette) -> &'static [OklchStop] {
    match palette {
        Palette::Organic => &ORGANIC_OKLCH,
        Palette::Heat => &HEAT_OKLCH,
        Palette::Ocean => &OCEAN_OKLCH,
        Palette::Mono => &MONO_OKLCH,
        Palette::Forest => &FOREST_OKLCH,
        Palette::Neon => &NEON_OKLCH,
        Palette::Warm => &WARM_OKLCH,
        Palette::Vibrant => &VIBRANT_OKLCH,
        Palette::LegibleMono => &LEGIBLEMONO_OKLCH,
        Palette::Slime => &SLIME_OKLCH,
        Palette::Mold => &MOLD_OKLCH,
        Palette::Fungus => &FUNGUS_OKLCH,
        Palette::Swamp => &SWAMP_OKLCH,
        Palette::Moss => &MOSS_OKLCH,
        Palette::Cosmic => &COSMIC_OKLCH,
        Palette::Ethereal => &ETHEREAL_OKLCH,
        Palette::Jade => &JADE_OKLCH,
        Palette::Amber => &AMBER_OKLCH,
        Palette::Slate => &SLATE_OKLCH,
        Palette::Pastel => &PASTEL_OKLCH,
        Palette::Ink => &INK_OKLCH,
        Palette::Copper => &COPPER_OKLCH,
        Palette::Custom(_) => panic!("Custom palette has no OKLch definition"),
    }
}
