//! Gradient data for color palettes.
//!
//! This module contains all the predefined color gradient data used by the palette system.
//! Gradients are defined as arrays of 11 color stops for both 256-color and RGB modes.

use crate::cli::Palette;
use crate::render::palette::RgbColor;
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
        Palette::Custom(_) => panic!("Custom palette requires special handling"),
    }
}
