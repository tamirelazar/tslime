use crate::cli::Palette;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RgbColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HsvColor {
    pub h: f32,
    pub s: f32,
    pub v: f32,
}

/// A gradient control point with position (0.0-1.0) and RGB color
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GradientStop {
    pub position: f32,
    pub color: RgbColor,
}

/// Interpolates smoothly between gradient stops
/// Supports any number of control points for continuous color mapping
pub fn interpolate_gradient(stops: &[GradientStop], t: f32) -> RgbColor {
    let t = t.clamp(0.0, 1.0);

    if stops.is_empty() {
        return RgbColor { r: 0, g: 0, b: 0 };
    }

    if stops.len() == 1 {
        return stops[0].color;
    }

    // Find the two stops to interpolate between
    let mut lower_idx = 0;
    let mut upper_idx = stops.len() - 1;

    for (i, stop) in stops.iter().enumerate() {
        if stop.position <= t {
            lower_idx = i;
        }
        if stop.position >= t && i < upper_idx {
            upper_idx = i;
            break;
        }
    }

    // If we're exactly at a stop, return that color
    if (stops[lower_idx].position - t).abs() < f32::EPSILON {
        return stops[lower_idx].color;
    }
    if (stops[upper_idx].position - t).abs() < f32::EPSILON {
        return stops[upper_idx].color;
    }

    // Interpolate between lower and upper
    let lower_stop = stops[lower_idx];
    let upper_stop = stops[upper_idx];

    let range = upper_stop.position - lower_stop.position;
    if range < f32::EPSILON {
        return lower_stop.color;
    }

    let local_t = (t - lower_stop.position) / range;

    RgbColor {
        r: (lower_stop.color.r as f32
            + (upper_stop.color.r as f32 - lower_stop.color.r as f32) * local_t) as u8,
        g: (lower_stop.color.g as f32
            + (upper_stop.color.g as f32 - lower_stop.color.g as f32) * local_t) as u8,
        b: (lower_stop.color.b as f32
            + (upper_stop.color.b as f32 - lower_stop.color.b as f32) * local_t) as u8,
    }
}

pub const ANSI_256_TO_RGB: [RgbColor; 256] = {
    // Colors 0-15: Standard ANSI system colors
    // Colors 16-231: 6×6×6 RGB cube with values [0, 95, 135, 175, 215, 255]
    // Colors 232-255: 24-step grayscale ramp (8, 18, 28, ... 248)
    [
        // 0-15: ANSI system colors
        RgbColor { r: 0, g: 0, b: 0 },   // 0: Black
        RgbColor { r: 128, g: 0, b: 0 }, // 1: Maroon
        RgbColor { r: 0, g: 128, b: 0 }, // 2: Green
        RgbColor {
            r: 128,
            g: 128,
            b: 0,
        }, // 3: Olive
        RgbColor { r: 0, g: 0, b: 128 }, // 4: Navy
        RgbColor {
            r: 128,
            g: 0,
            b: 128,
        }, // 5: Purple
        RgbColor {
            r: 0,
            g: 128,
            b: 128,
        }, // 6: Teal
        RgbColor {
            r: 192,
            g: 192,
            b: 192,
        }, // 7: Silver
        RgbColor {
            r: 128,
            g: 128,
            b: 128,
        }, // 8: Grey
        RgbColor { r: 255, g: 0, b: 0 }, // 9: Red
        RgbColor { r: 0, g: 255, b: 0 }, // 10: Lime
        RgbColor {
            r: 255,
            g: 255,
            b: 0,
        }, // 11: Yellow
        RgbColor { r: 0, g: 0, b: 255 }, // 12: Blue
        RgbColor {
            r: 255,
            g: 0,
            b: 255,
        }, // 13: Fuchsia
        RgbColor {
            r: 0,
            g: 255,
            b: 255,
        }, // 14: Aqua
        RgbColor {
            r: 255,
            g: 255,
            b: 255,
        }, // 15: White
        // 16-231: 6×6×6 RGB cube (r=0, g=0, b=0 to b=5)
        RgbColor { r: 0, g: 0, b: 0 },
        RgbColor { r: 0, g: 0, b: 95 },
        RgbColor { r: 0, g: 0, b: 135 },
        RgbColor { r: 0, g: 0, b: 175 },
        RgbColor { r: 0, g: 0, b: 215 },
        RgbColor { r: 0, g: 0, b: 255 },
        RgbColor { r: 0, g: 95, b: 0 },
        RgbColor { r: 0, g: 95, b: 95 },
        RgbColor {
            r: 0,
            g: 95,
            b: 135,
        },
        RgbColor {
            r: 0,
            g: 95,
            b: 175,
        },
        RgbColor {
            r: 0,
            g: 95,
            b: 215,
        },
        RgbColor {
            r: 0,
            g: 95,
            b: 255,
        },
        RgbColor { r: 0, g: 135, b: 0 },
        RgbColor {
            r: 0,
            g: 135,
            b: 95,
        },
        RgbColor {
            r: 0,
            g: 135,
            b: 135,
        },
        RgbColor {
            r: 0,
            g: 135,
            b: 175,
        },
        RgbColor {
            r: 0,
            g: 135,
            b: 215,
        },
        RgbColor {
            r: 0,
            g: 135,
            b: 255,
        },
        RgbColor { r: 0, g: 175, b: 0 },
        RgbColor {
            r: 0,
            g: 175,
            b: 95,
        },
        RgbColor {
            r: 0,
            g: 175,
            b: 135,
        },
        RgbColor {
            r: 0,
            g: 175,
            b: 175,
        },
        RgbColor {
            r: 0,
            g: 175,
            b: 215,
        },
        RgbColor {
            r: 0,
            g: 175,
            b: 255,
        },
        RgbColor { r: 0, g: 215, b: 0 },
        RgbColor {
            r: 0,
            g: 215,
            b: 95,
        },
        RgbColor {
            r: 0,
            g: 215,
            b: 135,
        },
        RgbColor {
            r: 0,
            g: 215,
            b: 175,
        },
        RgbColor {
            r: 0,
            g: 215,
            b: 215,
        },
        RgbColor {
            r: 0,
            g: 215,
            b: 255,
        },
        RgbColor { r: 0, g: 255, b: 0 },
        RgbColor {
            r: 0,
            g: 255,
            b: 95,
        },
        RgbColor {
            r: 0,
            g: 255,
            b: 135,
        },
        RgbColor {
            r: 0,
            g: 255,
            b: 175,
        },
        RgbColor {
            r: 0,
            g: 255,
            b: 215,
        },
        RgbColor {
            r: 0,
            g: 255,
            b: 255,
        },
        // r=1 (95)
        RgbColor { r: 95, g: 0, b: 0 },
        RgbColor { r: 95, g: 0, b: 95 },
        RgbColor {
            r: 95,
            g: 0,
            b: 135,
        },
        RgbColor {
            r: 95,
            g: 0,
            b: 175,
        },
        RgbColor {
            r: 95,
            g: 0,
            b: 215,
        },
        RgbColor {
            r: 95,
            g: 0,
            b: 255,
        },
        RgbColor { r: 95, g: 95, b: 0 },
        RgbColor {
            r: 95,
            g: 95,
            b: 95,
        },
        RgbColor {
            r: 95,
            g: 95,
            b: 135,
        },
        RgbColor {
            r: 95,
            g: 95,
            b: 175,
        },
        RgbColor {
            r: 95,
            g: 95,
            b: 215,
        },
        RgbColor {
            r: 95,
            g: 95,
            b: 255,
        },
        RgbColor {
            r: 95,
            g: 135,
            b: 0,
        },
        RgbColor {
            r: 95,
            g: 135,
            b: 95,
        },
        RgbColor {
            r: 95,
            g: 135,
            b: 135,
        },
        RgbColor {
            r: 95,
            g: 135,
            b: 175,
        },
        RgbColor {
            r: 95,
            g: 135,
            b: 215,
        },
        RgbColor {
            r: 95,
            g: 135,
            b: 255,
        },
        RgbColor {
            r: 95,
            g: 175,
            b: 0,
        },
        RgbColor {
            r: 95,
            g: 175,
            b: 95,
        },
        RgbColor {
            r: 95,
            g: 175,
            b: 135,
        },
        RgbColor {
            r: 95,
            g: 175,
            b: 175,
        },
        RgbColor {
            r: 95,
            g: 175,
            b: 215,
        },
        RgbColor {
            r: 95,
            g: 175,
            b: 255,
        },
        RgbColor {
            r: 95,
            g: 215,
            b: 0,
        },
        RgbColor {
            r: 95,
            g: 215,
            b: 95,
        },
        RgbColor {
            r: 95,
            g: 215,
            b: 135,
        },
        RgbColor {
            r: 95,
            g: 215,
            b: 175,
        },
        RgbColor {
            r: 95,
            g: 215,
            b: 215,
        },
        RgbColor {
            r: 95,
            g: 215,
            b: 255,
        },
        RgbColor {
            r: 95,
            g: 255,
            b: 0,
        },
        RgbColor {
            r: 95,
            g: 255,
            b: 95,
        },
        RgbColor {
            r: 95,
            g: 255,
            b: 135,
        },
        RgbColor {
            r: 95,
            g: 255,
            b: 175,
        },
        RgbColor {
            r: 95,
            g: 255,
            b: 215,
        },
        RgbColor {
            r: 95,
            g: 255,
            b: 255,
        },
        // r=2 (135)
        RgbColor { r: 135, g: 0, b: 0 },
        RgbColor {
            r: 135,
            g: 0,
            b: 95,
        },
        RgbColor {
            r: 135,
            g: 0,
            b: 135,
        },
        RgbColor {
            r: 135,
            g: 0,
            b: 175,
        },
        RgbColor {
            r: 135,
            g: 0,
            b: 215,
        },
        RgbColor {
            r: 135,
            g: 0,
            b: 255,
        },
        RgbColor {
            r: 135,
            g: 95,
            b: 0,
        },
        RgbColor {
            r: 135,
            g: 95,
            b: 95,
        },
        RgbColor {
            r: 135,
            g: 95,
            b: 135,
        },
        RgbColor {
            r: 135,
            g: 95,
            b: 175,
        },
        RgbColor {
            r: 135,
            g: 95,
            b: 215,
        },
        RgbColor {
            r: 135,
            g: 95,
            b: 255,
        },
        RgbColor {
            r: 135,
            g: 135,
            b: 0,
        },
        RgbColor {
            r: 135,
            g: 135,
            b: 95,
        },
        RgbColor {
            r: 135,
            g: 135,
            b: 135,
        },
        RgbColor {
            r: 135,
            g: 135,
            b: 175,
        },
        RgbColor {
            r: 135,
            g: 135,
            b: 215,
        },
        RgbColor {
            r: 135,
            g: 135,
            b: 255,
        },
        RgbColor {
            r: 135,
            g: 175,
            b: 0,
        },
        RgbColor {
            r: 135,
            g: 175,
            b: 95,
        },
        RgbColor {
            r: 135,
            g: 175,
            b: 135,
        },
        RgbColor {
            r: 135,
            g: 175,
            b: 175,
        },
        RgbColor {
            r: 135,
            g: 175,
            b: 215,
        },
        RgbColor {
            r: 135,
            g: 175,
            b: 255,
        },
        RgbColor {
            r: 135,
            g: 215,
            b: 0,
        },
        RgbColor {
            r: 135,
            g: 215,
            b: 95,
        },
        RgbColor {
            r: 135,
            g: 215,
            b: 135,
        },
        RgbColor {
            r: 135,
            g: 215,
            b: 175,
        },
        RgbColor {
            r: 135,
            g: 215,
            b: 215,
        },
        RgbColor {
            r: 135,
            g: 215,
            b: 255,
        },
        RgbColor {
            r: 135,
            g: 255,
            b: 0,
        },
        RgbColor {
            r: 135,
            g: 255,
            b: 95,
        },
        RgbColor {
            r: 135,
            g: 255,
            b: 135,
        },
        RgbColor {
            r: 135,
            g: 255,
            b: 175,
        },
        RgbColor {
            r: 135,
            g: 255,
            b: 215,
        },
        RgbColor {
            r: 135,
            g: 255,
            b: 255,
        },
        // r=3 (175)
        RgbColor { r: 175, g: 0, b: 0 },
        RgbColor {
            r: 175,
            g: 0,
            b: 95,
        },
        RgbColor {
            r: 175,
            g: 0,
            b: 135,
        },
        RgbColor {
            r: 175,
            g: 0,
            b: 175,
        },
        RgbColor {
            r: 175,
            g: 0,
            b: 215,
        },
        RgbColor {
            r: 175,
            g: 0,
            b: 255,
        },
        RgbColor {
            r: 175,
            g: 95,
            b: 0,
        },
        RgbColor {
            r: 175,
            g: 95,
            b: 95,
        },
        RgbColor {
            r: 175,
            g: 95,
            b: 135,
        },
        RgbColor {
            r: 175,
            g: 95,
            b: 175,
        },
        RgbColor {
            r: 175,
            g: 95,
            b: 215,
        },
        RgbColor {
            r: 175,
            g: 95,
            b: 255,
        },
        RgbColor {
            r: 175,
            g: 135,
            b: 0,
        },
        RgbColor {
            r: 175,
            g: 135,
            b: 95,
        },
        RgbColor {
            r: 175,
            g: 135,
            b: 135,
        },
        RgbColor {
            r: 175,
            g: 135,
            b: 175,
        },
        RgbColor {
            r: 175,
            g: 135,
            b: 215,
        },
        RgbColor {
            r: 175,
            g: 135,
            b: 255,
        },
        RgbColor {
            r: 175,
            g: 175,
            b: 0,
        },
        RgbColor {
            r: 175,
            g: 175,
            b: 95,
        },
        RgbColor {
            r: 175,
            g: 175,
            b: 135,
        },
        RgbColor {
            r: 175,
            g: 175,
            b: 175,
        },
        RgbColor {
            r: 175,
            g: 175,
            b: 215,
        },
        RgbColor {
            r: 175,
            g: 175,
            b: 255,
        },
        RgbColor {
            r: 175,
            g: 215,
            b: 0,
        },
        RgbColor {
            r: 175,
            g: 215,
            b: 95,
        },
        RgbColor {
            r: 175,
            g: 215,
            b: 135,
        },
        RgbColor {
            r: 175,
            g: 215,
            b: 175,
        },
        RgbColor {
            r: 175,
            g: 215,
            b: 215,
        },
        RgbColor {
            r: 175,
            g: 215,
            b: 255,
        },
        RgbColor {
            r: 175,
            g: 255,
            b: 0,
        },
        RgbColor {
            r: 175,
            g: 255,
            b: 95,
        },
        RgbColor {
            r: 175,
            g: 255,
            b: 135,
        },
        RgbColor {
            r: 175,
            g: 255,
            b: 175,
        },
        RgbColor {
            r: 175,
            g: 255,
            b: 215,
        },
        RgbColor {
            r: 175,
            g: 255,
            b: 255,
        },
        // r=4 (215)
        RgbColor { r: 215, g: 0, b: 0 },
        RgbColor {
            r: 215,
            g: 0,
            b: 95,
        },
        RgbColor {
            r: 215,
            g: 0,
            b: 135,
        },
        RgbColor {
            r: 215,
            g: 0,
            b: 175,
        },
        RgbColor {
            r: 215,
            g: 0,
            b: 215,
        },
        RgbColor {
            r: 215,
            g: 0,
            b: 255,
        },
        RgbColor {
            r: 215,
            g: 95,
            b: 0,
        },
        RgbColor {
            r: 215,
            g: 95,
            b: 95,
        },
        RgbColor {
            r: 215,
            g: 95,
            b: 135,
        },
        RgbColor {
            r: 215,
            g: 95,
            b: 175,
        },
        RgbColor {
            r: 215,
            g: 95,
            b: 215,
        },
        RgbColor {
            r: 215,
            g: 95,
            b: 255,
        },
        RgbColor {
            r: 215,
            g: 135,
            b: 0,
        },
        RgbColor {
            r: 215,
            g: 135,
            b: 95,
        },
        RgbColor {
            r: 215,
            g: 135,
            b: 135,
        },
        RgbColor {
            r: 215,
            g: 135,
            b: 175,
        },
        RgbColor {
            r: 215,
            g: 135,
            b: 215,
        },
        RgbColor {
            r: 215,
            g: 135,
            b: 255,
        },
        RgbColor {
            r: 215,
            g: 175,
            b: 0,
        },
        RgbColor {
            r: 215,
            g: 175,
            b: 95,
        },
        RgbColor {
            r: 215,
            g: 175,
            b: 135,
        },
        RgbColor {
            r: 215,
            g: 175,
            b: 175,
        },
        RgbColor {
            r: 215,
            g: 175,
            b: 215,
        },
        RgbColor {
            r: 215,
            g: 175,
            b: 255,
        },
        RgbColor {
            r: 215,
            g: 215,
            b: 0,
        },
        RgbColor {
            r: 215,
            g: 215,
            b: 95,
        },
        RgbColor {
            r: 215,
            g: 215,
            b: 135,
        },
        RgbColor {
            r: 215,
            g: 215,
            b: 175,
        },
        RgbColor {
            r: 215,
            g: 215,
            b: 215,
        },
        RgbColor {
            r: 215,
            g: 215,
            b: 255,
        },
        RgbColor {
            r: 215,
            g: 255,
            b: 0,
        },
        RgbColor {
            r: 215,
            g: 255,
            b: 95,
        },
        RgbColor {
            r: 215,
            g: 255,
            b: 135,
        },
        RgbColor {
            r: 215,
            g: 255,
            b: 175,
        },
        RgbColor {
            r: 215,
            g: 255,
            b: 215,
        },
        RgbColor {
            r: 215,
            g: 255,
            b: 255,
        },
        // r=5 (255)
        RgbColor { r: 255, g: 0, b: 0 },
        RgbColor {
            r: 255,
            g: 0,
            b: 95,
        },
        RgbColor {
            r: 255,
            g: 0,
            b: 135,
        },
        RgbColor {
            r: 255,
            g: 0,
            b: 175,
        },
        RgbColor {
            r: 255,
            g: 0,
            b: 215,
        },
        RgbColor {
            r: 255,
            g: 0,
            b: 255,
        },
        RgbColor {
            r: 255,
            g: 95,
            b: 0,
        },
        RgbColor {
            r: 255,
            g: 95,
            b: 95,
        },
        RgbColor {
            r: 255,
            g: 95,
            b: 135,
        },
        RgbColor {
            r: 255,
            g: 95,
            b: 175,
        },
        RgbColor {
            r: 255,
            g: 95,
            b: 215,
        },
        RgbColor {
            r: 255,
            g: 95,
            b: 255,
        },
        RgbColor {
            r: 255,
            g: 135,
            b: 0,
        },
        RgbColor {
            r: 255,
            g: 135,
            b: 95,
        },
        RgbColor {
            r: 255,
            g: 135,
            b: 135,
        },
        RgbColor {
            r: 255,
            g: 135,
            b: 175,
        },
        RgbColor {
            r: 255,
            g: 135,
            b: 215,
        },
        RgbColor {
            r: 255,
            g: 135,
            b: 255,
        },
        RgbColor {
            r: 255,
            g: 175,
            b: 0,
        },
        RgbColor {
            r: 255,
            g: 175,
            b: 95,
        },
        RgbColor {
            r: 255,
            g: 175,
            b: 135,
        },
        RgbColor {
            r: 255,
            g: 175,
            b: 175,
        },
        RgbColor {
            r: 255,
            g: 175,
            b: 215,
        },
        RgbColor {
            r: 255,
            g: 175,
            b: 255,
        },
        RgbColor {
            r: 255,
            g: 215,
            b: 0,
        },
        RgbColor {
            r: 255,
            g: 215,
            b: 95,
        },
        RgbColor {
            r: 255,
            g: 215,
            b: 135,
        },
        RgbColor {
            r: 255,
            g: 215,
            b: 175,
        },
        RgbColor {
            r: 255,
            g: 215,
            b: 215,
        },
        RgbColor {
            r: 255,
            g: 215,
            b: 255,
        },
        RgbColor {
            r: 255,
            g: 255,
            b: 0,
        },
        RgbColor {
            r: 255,
            g: 255,
            b: 95,
        },
        RgbColor {
            r: 255,
            g: 255,
            b: 135,
        },
        RgbColor {
            r: 255,
            g: 255,
            b: 175,
        },
        RgbColor {
            r: 255,
            g: 255,
            b: 215,
        },
        RgbColor {
            r: 255,
            g: 255,
            b: 255,
        },
        // 232-255: Grayscale
        RgbColor { r: 8, g: 8, b: 8 },
        RgbColor {
            r: 18,
            g: 18,
            b: 18,
        },
        RgbColor {
            r: 28,
            g: 28,
            b: 28,
        },
        RgbColor {
            r: 38,
            g: 38,
            b: 38,
        },
        RgbColor {
            r: 48,
            g: 48,
            b: 48,
        },
        RgbColor {
            r: 58,
            g: 58,
            b: 58,
        },
        RgbColor {
            r: 68,
            g: 68,
            b: 68,
        },
        RgbColor {
            r: 78,
            g: 78,
            b: 78,
        },
        RgbColor {
            r: 88,
            g: 88,
            b: 88,
        },
        RgbColor {
            r: 98,
            g: 98,
            b: 98,
        },
        RgbColor {
            r: 108,
            g: 108,
            b: 108,
        },
        RgbColor {
            r: 118,
            g: 118,
            b: 118,
        },
        RgbColor {
            r: 128,
            g: 128,
            b: 128,
        },
        RgbColor {
            r: 138,
            g: 138,
            b: 138,
        },
        RgbColor {
            r: 148,
            g: 148,
            b: 148,
        },
        RgbColor {
            r: 158,
            g: 158,
            b: 158,
        },
        RgbColor {
            r: 168,
            g: 168,
            b: 168,
        },
        RgbColor {
            r: 178,
            g: 178,
            b: 178,
        },
        RgbColor {
            r: 188,
            g: 188,
            b: 188,
        },
        RgbColor {
            r: 198,
            g: 198,
            b: 198,
        },
        RgbColor {
            r: 208,
            g: 208,
            b: 208,
        },
        RgbColor {
            r: 218,
            g: 218,
            b: 218,
        },
        RgbColor {
            r: 228,
            g: 228,
            b: 228,
        },
        RgbColor {
            r: 238,
            g: 238,
            b: 238,
        },
    ]
};

pub fn rgb_to_hsv(rgb: RgbColor) -> HsvColor {
    let r = rgb.r as f32 / 255.0;
    let g = rgb.g as f32 / 255.0;
    let b = rgb.b as f32 / 255.0;

    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let delta = max - min;

    let h = if delta == 0.0 {
        0.0
    } else if max == r {
        60.0 * (((g - b) / delta) % 6.0)
    } else if max == g {
        60.0 * (((b - r) / delta) + 2.0)
    } else {
        60.0 * (((r - g) / delta) + 4.0)
    };

    let s = if max == 0.0 { 0.0 } else { delta / max };
    let v = max;

    HsvColor {
        h: if h < 0.0 { h + 360.0 } else { h },
        s,
        v,
    }
}

pub fn hsv_to_rgb(hsv: HsvColor) -> RgbColor {
    let h = hsv.h;
    let s = hsv.s;
    let v = hsv.v;

    let c = v * s;
    let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
    let m = v - c;

    let (r, g, b) = if h < 60.0 {
        (c, x, 0.0)
    } else if h < 120.0 {
        (x, c, 0.0)
    } else if h < 180.0 {
        (0.0, c, x)
    } else if h < 240.0 {
        (0.0, x, c)
    } else if h < 300.0 {
        (x, 0.0, c)
    } else {
        (c, 0.0, x)
    };

    RgbColor {
        r: ((r + m) * 255.0).clamp(0.0, 255.0) as u8,
        g: ((g + m) * 255.0).clamp(0.0, 255.0) as u8,
        b: ((b + m) * 255.0).clamp(0.0, 255.0) as u8,
    }
}

pub fn rotate_hue(hsv: HsvColor, degrees: f32) -> HsvColor {
    HsvColor {
        h: (hsv.h + degrees) % 360.0,
        s: hsv.s,
        v: hsv.v,
    }
}

pub fn rgb_to_256(rgb: RgbColor) -> u8 {
    let gray_diff = (rgb.r as i16 - rgb.g as i16).abs()
        + (rgb.g as i16 - rgb.b as i16).abs()
        + (rgb.b as i16 - rgb.r as i16).abs();
    if gray_diff < 3 {
        if rgb.r < 8 {
            return 16;
        } else if rgb.r > 248 {
            return 231;
        } else {
            let gray_level = (rgb.r - 8) / 10;
            return 232 + gray_level;
        }
    }

    for (i, c) in ANSI_256_TO_RGB.iter().enumerate().take(16) {
        let dist = ((rgb.r as i32 - c.r as i32).pow(2)
            + (rgb.g as i32 - c.g as i32).pow(2)
            + (rgb.b as i32 - c.b as i32).pow(2)) as u32;
        if dist < 2000 {
            return i as u8;
        }
    }

    let r_idx = ((rgb.r as f32 / 255.0) * 5.0).round() as u8;
    let g_idx = ((rgb.g as f32 / 255.0) * 5.0).round() as u8;
    let b_idx = ((rgb.b as f32 / 255.0) * 5.0).round() as u8;

    let r_idx = r_idx.clamp(0, 5);
    let g_idx = g_idx.clamp(0, 5);
    let b_idx = b_idx.clamp(0, 5);

    16 + (r_idx * 36 + g_idx * 6 + b_idx)
}

pub fn invert_256_color(color_code: u8) -> u8 {
    let rgb = ANSI_256_TO_RGB[color_code as usize];
    let hsv = rgb_to_hsv(rgb);
    let rotated = rotate_hue(hsv, 180.0);
    let new_rgb = hsv_to_rgb(rotated);
    rgb_to_256(new_rgb)
}

pub fn hex_to_rgb(hex: &str) -> Option<RgbColor> {
    let hex = hex.trim_start_matches('#');
    if hex.len() != 6 {
        return None;
    }
    let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
    let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
    let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
    Some(RgbColor { r, g, b })
}

pub fn map_species_brightness(brightness: f32, base_color: RgbColor, reverse: bool) -> u8 {
    let hsv = rgb_to_hsv(base_color);
    let brightness = if reverse {
        1.0 - brightness
    } else {
        brightness
    };

    let t = brightness.clamp(0.0, 1.0);

    let min_s = 0.05;
    let max_s = hsv.s.max(0.1);
    let min_v = 0.08;
    let max_v = (hsv.v * 0.9 + 0.1).min(0.95);

    let s = min_s + (max_s - min_s) * t;
    let v = min_v + (max_v - min_v) * t;

    let final_hsv = HsvColor { h: hsv.h, s, v };
    let final_rgb = hsv_to_rgb(final_hsv);
    rgb_to_256(final_rgb)
}

pub fn map_species_brightness_rgb(
    brightness: f32,
    base_color: RgbColor,
    reverse: bool,
) -> RgbColor {
    let hsv = rgb_to_hsv(base_color);
    let brightness = if reverse {
        1.0 - brightness
    } else {
        brightness
    };

    let t = brightness.clamp(0.0, 1.0);

    let min_s = 0.05;
    let max_s = hsv.s.max(0.1);
    let min_v = 0.08;
    let max_v = (hsv.v * 0.9 + 0.1).min(0.95);

    let s = min_s + (max_s - min_s) * t;
    let v = min_v + (max_v - min_v) * t;

    let final_hsv = HsvColor { h: hsv.h, s, v };
    hsv_to_rgb(final_hsv)
}

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

fn get_256_gradient(palette: Palette) -> &'static [u8; 11] {
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

fn get_rgb_gradient(palette: Palette) -> &'static [RgbColor; 11] {
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

/// Convert an array of RGB colors to evenly-spaced gradient stops
fn rgb_array_to_gradient_stops<const N: usize>(colors: &[RgbColor; N]) -> Vec<GradientStop> {
    colors
        .iter()
        .enumerate()
        .map(|(i, &color)| GradientStop {
            position: i as f32 / (N - 1).max(1) as f32,
            color,
        })
        .collect()
}

/// Get gradient stops for a palette (supports continuous interpolation)
fn get_gradient_stops(palette: &Palette) -> Vec<GradientStop> {
    match palette {
        Palette::Custom(colors) => {
            // For custom palettes, create evenly spaced stops
            colors
                .iter()
                .enumerate()
                .map(|(i, &color)| GradientStop {
                    position: i as f32 / (colors.len() - 1).max(1) as f32,
                    color,
                })
                .collect()
        }
        _ => {
            // For built-in palettes, convert the 11-step arrays to gradient stops
            let rgb_gradient = get_rgb_gradient(palette.clone());
            rgb_array_to_gradient_stops(rgb_gradient)
        }
    }
}

fn interpolate_custom_palette(colors: &[RgbColor]) -> [RgbColor; 11] {
    let num_colors = colors.len();
    if num_colors == 2 {
        let mut result = [colors[0]; 11];
        for (i, slot) in result.iter_mut().enumerate() {
            let t = i as f32 / 10.0;
            *slot = RgbColor {
                r: ((colors[0].r as f32 * (1.0 - t) + colors[1].r as f32 * t) as u8),
                g: ((colors[0].g as f32 * (1.0 - t) + colors[1].g as f32 * t) as u8),
                b: ((colors[0].b as f32 * (1.0 - t) + colors[1].b as f32 * t) as u8),
            };
        }
        return result;
    }

    let mut result = [RgbColor { r: 0, g: 0, b: 0 }; 11];
    for (i, slot) in result.iter_mut().enumerate() {
        let t = if num_colors > 1 {
            (i as f32 / 10.0) * (num_colors - 1) as f32
        } else {
            0.0
        };
        let segment = t.floor() as usize;
        let segment_t = t.fract();

        let start_idx = segment.min(num_colors - 1);
        let end_idx = (segment + 1).min(num_colors - 1);

        let start_color = colors[start_idx];
        let end_color = colors[end_idx];

        *slot = RgbColor {
            r: ((start_color.r as f32 * (1.0 - segment_t) + end_color.r as f32 * segment_t) as u8),
            g: ((start_color.g as f32 * (1.0 - segment_t) + end_color.g as f32 * segment_t) as u8),
            b: ((start_color.b as f32 * (1.0 - segment_t) + end_color.b as f32 * segment_t) as u8),
        };
    }
    result
}

fn invert_color(color_code: u8) -> u8 {
    invert_256_color(color_code)
}

pub fn map_brightness(brightness: f32, palette: Palette, reverse: bool, invert: bool) -> u8 {
    let mut brightness = brightness.clamp(0.0, 1.0);

    let gradient: &[u8] = match &palette {
        Palette::Custom(colors) => {
            let interpolated = interpolate_custom_palette(colors);
            let gradient_256: Vec<u8> = interpolated.iter().map(|c| rgb_to_256(*c)).collect();
            let boxed = gradient_256.into_boxed_slice();
            Box::leak(boxed)
        }
        _ => get_256_gradient(palette),
    };

    if reverse {
        brightness = 1.0 - brightness;
    }

    let position = brightness * (gradient.len() - 1) as f32;
    let lower = position.floor() as usize;
    let upper = position.ceil() as usize;
    let fraction = position - lower as f32;

    let color = if upper == lower || fraction < 0.5 {
        gradient[lower]
    } else {
        gradient[upper]
    };

    let mut final_color = color;

    if invert {
        final_color = invert_color(final_color);
    }

    final_color
}

fn invert_rgb(rgb: RgbColor) -> RgbColor {
    RgbColor {
        r: 255 - rgb.r,
        g: 255 - rgb.g,
        b: 255 - rgb.b,
    }
}

pub fn map_brightness_rgb(
    brightness: f32,
    palette: Palette,
    reverse: bool,
    invert: bool,
    hue_shift: f32,
) -> RgbColor {
    let mut brightness = brightness.clamp(0.0, 1.0);

    if reverse {
        brightness = 1.0 - brightness;
    }

    // Use the new gradient interpolation system
    let stops = get_gradient_stops(&palette);
    let mut final_color = interpolate_gradient(&stops, brightness);

    if invert {
        final_color = invert_rgb(final_color);
    }

    if hue_shift == 0.0 {
        return final_color;
    }

    let hsv = rgb_to_hsv(final_color);
    let rotated = rotate_hue(hsv, hue_shift);
    hsv_to_rgb(rotated)
}

#[allow(dead_code)]
pub fn truecolor_ansi(r: u8, g: u8, b: u8, is_fg: bool) -> String {
    if is_fg {
        format!("\x1b[38;2;{};{};{}m", r, g, b)
    } else {
        format!("\x1b[48;2;{};{};{}m", r, g, b)
    }
}

#[allow(dead_code)]
pub fn truecolor_ansi_fg(r: u8, g: u8, b: u8) -> String {
    format!("\x1b[38;2;{};{};{}m", r, g, b)
}

#[allow(dead_code)]
pub fn truecolor_ansi_bg(r: u8, g: u8, b: u8) -> String {
    format!("\x1b[48;2;{};{};{}m", r, g, b)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_map_brightness_min() {
        assert_eq!(map_brightness(0.0, Palette::Organic, false, false), 232);
        assert_eq!(map_brightness(0.0, Palette::Heat, false, false), 232);
        assert_eq!(map_brightness(0.0, Palette::Ocean, false, false), 232);
        assert_eq!(map_brightness(0.0, Palette::Mono, false, false), 232);
        assert_eq!(map_brightness(0.0, Palette::Forest, false, false), 22);
        assert_eq!(map_brightness(0.0, Palette::Neon, false, false), 17);
        assert_eq!(map_brightness(0.0, Palette::Warm, false, false), 52);
        assert_eq!(map_brightness(0.0, Palette::Vibrant, false, false), 197);
        assert_eq!(map_brightness(0.0, Palette::LegibleMono, false, false), 236);
    }

    #[test]
    fn test_map_brightness_max() {
        assert_eq!(map_brightness(1.0, Palette::Organic, false, false), 226);
        assert_eq!(map_brightness(1.0, Palette::Heat, false, false), 226);
        assert_eq!(map_brightness(1.0, Palette::Ocean, false, false), 51);
        assert_eq!(map_brightness(1.0, Palette::Mono, false, false), 252);
        assert_eq!(map_brightness(1.0, Palette::Forest, false, false), 40);
        assert_eq!(map_brightness(1.0, Palette::Neon, false, false), 195);
        assert_eq!(map_brightness(1.0, Palette::Warm, false, false), 226);
        assert_eq!(map_brightness(1.0, Palette::Vibrant, false, false), 231);
        assert_eq!(map_brightness(1.0, Palette::LegibleMono, false, false), 255);
    }

    #[test]
    fn test_map_brightness_mid() {
        let color = map_brightness(0.5, Palette::Organic, false, false);
        assert_eq!(color, 46);

        let color = map_brightness(0.5, Palette::Heat, false, false);
        assert_eq!(color, 196);

        let color = map_brightness(0.5, Palette::Ocean, false, false);
        assert_eq!(color, 21);

        let color = map_brightness(0.5, Palette::Mono, false, false);
        assert_eq!(color, 242);

        let color = map_brightness(0.5, Palette::Forest, false, false);
        assert_eq!(color, 40);

        let color = map_brightness(0.5, Palette::Neon, false, false);
        assert_eq!(color, 123);

        let color = map_brightness(0.5, Palette::Warm, false, false);
        assert_eq!(color, 208);

        let color = map_brightness(0.5, Palette::Vibrant, false, false);
        assert_eq!(color, 121);

        let color = map_brightness(0.5, Palette::LegibleMono, false, false);
        assert_eq!(color, 251);
    }

    #[test]
    fn test_map_brightness_clamped() {
        assert_eq!(map_brightness(-0.5, Palette::Organic, false, false), 232);
        assert_eq!(map_brightness(1.5, Palette::Organic, false, false), 226);
        assert_eq!(map_brightness(-0.5, Palette::Forest, false, false), 22);
        assert_eq!(map_brightness(1.5, Palette::Forest, false, false), 40);
    }

    #[test]
    fn test_map_brightness_quarter() {
        let color = map_brightness(0.25, Palette::Organic, false, false);
        assert_eq!(color, 34);

        let color = map_brightness(0.25, Palette::Heat, false, false);
        assert_eq!(color, 124);

        let color = map_brightness(0.25, Palette::Forest, false, false);
        assert_eq!(color, 34);

        let color = map_brightness(0.25, Palette::Neon, false, false);
        assert_eq!(color, 51);

        let color = map_brightness(0.25, Palette::Warm, false, false);
        assert_eq!(color, 166);
    }

    #[test]
    fn test_map_brightness_three_quarter() {
        let color = map_brightness(0.75, Palette::Organic, false, false);
        assert_eq!(color, 154);

        let color = map_brightness(0.75, Palette::Heat, false, false);
        assert_eq!(color, 214);

        let color = map_brightness(0.75, Palette::Forest, false, false);
        assert_eq!(color, 154);

        let color = map_brightness(0.75, Palette::Neon, false, false);
        assert_eq!(color, 201);

        let color = map_brightness(0.75, Palette::Warm, false, false);
        assert_eq!(color, 226);
    }

    #[test]
    fn test_reverse_palette() {
        assert_eq!(map_brightness(0.0, Palette::Organic, true, false), 226);
        assert_eq!(map_brightness(1.0, Palette::Organic, true, false), 232);
    }

    #[test]
    fn test_invert_palette() {
        let normal = map_brightness(0.5, Palette::Organic, false, false);
        let inverted = map_brightness(0.5, Palette::Organic, false, true);
        let normal_rgb = ANSI_256_TO_RGB[normal as usize];
        let inverted_rgb = ANSI_256_TO_RGB[inverted as usize];
        assert_ne!(inverted, normal);
        assert_ne!(inverted, 255 - normal);
        let hsv_normal = rgb_to_hsv(normal_rgb);
        let hsv_inverted = rgb_to_hsv(inverted_rgb);
        let hue_diff = (hsv_inverted.h - hsv_normal.h).abs();
        assert!(hue_diff > 170.0 && hue_diff < 190.0);
    }

    #[test]
    fn test_reverse_and_invert_palette() {
        let reversed = map_brightness(0.0, Palette::Organic, true, false);
        let reversed_and_inverted = map_brightness(0.0, Palette::Organic, true, true);
        let inverted = invert_256_color(reversed);
        assert_eq!(reversed_and_inverted, inverted);
    }

    #[test]
    fn test_map_brightness_rgb_min() {
        let color = map_brightness_rgb(0.0, Palette::Organic, false, false, 0.0);
        assert_eq!(color.r, 18);
        assert_eq!(color.g, 18);
        assert_eq!(color.b, 18);
    }

    #[test]
    fn test_map_brightness_rgb_max() {
        let color = map_brightness_rgb(1.0, Palette::Organic, false, false, 0.0);
        assert_eq!(color.r, 150);
        assert_eq!(color.g, 220);
        assert_eq!(color.b, 200);
    }

    #[test]
    fn test_map_brightness_rgb_interpolation() {
        let color = map_brightness_rgb(0.5, Palette::Organic, false, false, 0.0);
        assert!(color.r >= 18 && color.r <= 160);
        assert!(color.g >= 18 && color.g <= 220);
        assert!(color.b >= 18 && color.b <= 200);

        let color = map_brightness_rgb(0.5, Palette::Ocean, false, false, 0.0);
        assert!(color.r >= 18 && color.r <= 80);
        assert!(color.g >= 18 && color.g <= 170);
        assert!(color.b >= 18 && color.b <= 240);
    }

    #[test]
    fn test_map_brightness_rgb_heat() {
        let min_color = map_brightness_rgb(0.0, Palette::Heat, false, false, 0.0);
        let max_color = map_brightness_rgb(1.0, Palette::Heat, false, false, 0.0);
        assert_eq!(min_color.r, 40);
        assert_eq!(min_color.g, 20);
        assert_eq!(min_color.b, 20);
        assert_eq!(max_color.r, 240);
        assert_eq!(max_color.g, 220);
        assert_eq!(max_color.b, 180);
    }

    #[test]
    fn test_map_brightness_rgb_ocean() {
        let min_color = map_brightness_rgb(0.0, Palette::Ocean, false, false, 0.0);
        let max_color = map_brightness_rgb(1.0, Palette::Ocean, false, false, 0.0);
        assert_eq!(min_color.r, 18);
        assert_eq!(min_color.g, 18);
        assert_eq!(min_color.b, 18);
        assert_eq!(max_color.r, 80);
        assert_eq!(max_color.g, 170);
        assert_eq!(max_color.b, 240);
    }

    #[test]
    fn test_map_brightness_rgb_forest() {
        let min_color = map_brightness_rgb(0.0, Palette::Forest, false, false, 0.0);
        let max_color = map_brightness_rgb(1.0, Palette::Forest, false, false, 0.0);
        assert_eq!(min_color.r, 20);
        assert_eq!(min_color.g, 40);
        assert_eq!(min_color.b, 20);
        assert_eq!(max_color.r, 180);
        assert_eq!(max_color.g, 220);
        assert_eq!(max_color.b, 170);
    }

    #[test]
    fn test_map_brightness_rgb_reverse() {
        let normal = map_brightness_rgb(0.0, Palette::Organic, false, false, 0.0);
        let reversed = map_brightness_rgb(1.0, Palette::Organic, true, false, 0.0);
        assert_eq!(normal.r, reversed.r);
        assert_eq!(normal.g, reversed.g);
        assert_eq!(normal.b, reversed.b);
    }

    #[test]
    fn test_map_brightness_rgb_invert() {
        let normal = map_brightness_rgb(0.5, Palette::Organic, false, false, 0.0);
        let inverted = map_brightness_rgb(0.5, Palette::Organic, false, true, 0.0);
        assert_eq!(inverted.r, 255 - normal.r);
        assert_eq!(inverted.g, 255 - normal.g);
        assert_eq!(inverted.b, 255 - normal.b);
    }

    #[test]
    fn test_map_brightness_rgb_all_palettes() {
        let palettes = [
            Palette::Organic,
            Palette::Heat,
            Palette::Ocean,
            Palette::Mono,
            Palette::Forest,
            Palette::Neon,
            Palette::Warm,
            Palette::Vibrant,
            Palette::LegibleMono,
            Palette::Slime,
            Palette::Mold,
            Palette::Fungus,
            Palette::Swamp,
            Palette::Moss,
        ];

        for palette in palettes {
            let _color = map_brightness_rgb(0.5, palette, false, false, 0.0);
        }
    }

    #[test]
    fn test_map_brightness_rgb_clamped() {
        let min = map_brightness_rgb(-0.5, Palette::Heat, false, false, 0.0);
        let max = map_brightness_rgb(1.5, Palette::Heat, false, false, 0.0);
        let normal = map_brightness_rgb(0.5, Palette::Heat, false, false, 0.0);
        assert_eq!(min.r, 40);
        assert_eq!(max.r, 240);
        assert!(min.r <= normal.r && normal.r <= max.r);
    }

    #[test]
    fn test_truecolor_ansi_fg() {
        let code = truecolor_ansi(255, 128, 64, true);
        assert_eq!(code, "\x1b[38;2;255;128;64m");
    }

    #[test]
    fn test_truecolor_ansi_bg() {
        let code = truecolor_ansi(255, 128, 64, false);
        assert_eq!(code, "\x1b[48;2;255;128;64m");
    }

    #[test]
    fn test_truecolor_ansi_fg_specific() {
        let code = truecolor_ansi_fg(42, 42, 42);
        assert_eq!(code, "\x1b[38;2;42;42;42m");
    }

    #[test]
    fn test_truecolor_ansi_bg_specific() {
        let code = truecolor_ansi_bg(42, 42, 42);
        assert_eq!(code, "\x1b[48;2;42;42;42m");
    }

    #[test]
    fn test_truecolor_ansi_zeros() {
        let code = truecolor_ansi(0, 0, 0, true);
        assert_eq!(code, "\x1b[38;2;0;0;0m");
    }

    #[test]
    fn test_truecolor_ansi_max_values() {
        let code = truecolor_ansi(255, 255, 255, true);
        assert_eq!(code, "\x1b[38;2;255;255;255m");
    }

    #[test]
    fn test_rgb_to_hsv_red() {
        let hsv = rgb_to_hsv(RgbColor { r: 255, g: 0, b: 0 });
        assert!((hsv.h - 0.0).abs() < 1.0);
        assert!((hsv.s - 1.0).abs() < 0.01);
        assert!((hsv.v - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_rgb_to_hsv_green() {
        let hsv = rgb_to_hsv(RgbColor { r: 0, g: 255, b: 0 });
        assert!((hsv.h - 120.0).abs() < 1.0);
        assert!((hsv.s - 1.0).abs() < 0.01);
        assert!((hsv.v - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_rgb_to_hsv_blue() {
        let hsv = rgb_to_hsv(RgbColor { r: 0, g: 0, b: 255 });
        assert!((hsv.h - 240.0).abs() < 1.0);
        assert!((hsv.s - 1.0).abs() < 0.01);
        assert!((hsv.v - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_rgb_to_hsv_cyan_complementary_to_red() {
        let red_hsv = rgb_to_hsv(RgbColor { r: 255, g: 0, b: 0 });
        let cyan_hsv = rgb_to_hsv(RgbColor {
            r: 0,
            g: 255,
            b: 255,
        });
        let hue_diff = (cyan_hsv.h - red_hsv.h).abs();
        assert!(hue_diff > 178.0 && hue_diff < 182.0);
    }

    #[test]
    fn test_hsv_to_rgb_roundtrip() {
        let original = RgbColor {
            r: 128,
            g: 64,
            b: 255,
        };
        let hsv = rgb_to_hsv(original);
        let result = hsv_to_rgb(hsv);
        assert!((result.r as i16 - original.r as i16).abs() <= 1);
        assert!((result.g as i16 - original.g as i16).abs() <= 1);
        assert!((result.b as i16 - original.b as i16).abs() <= 1);
    }

    #[test]
    fn test_rotate_hue_180_degrees() {
        let hsv = HsvColor {
            h: 0.0,
            s: 1.0,
            v: 1.0,
        };
        let rotated = rotate_hue(hsv, 180.0);
        assert!((rotated.h - 180.0).abs() < 0.01);
    }

    #[test]
    fn test_rotate_hue_wraps_around() {
        let hsv = HsvColor {
            h: 300.0,
            s: 1.0,
            v: 1.0,
        };
        let rotated = rotate_hue(hsv, 100.0);
        assert!((rotated.h - 40.0).abs() < 0.01);
    }

    #[test]
    fn test_invert_256_red_to_cyan() {
        let red_code = 9;
        let inverted = invert_256_color(red_code);
        let inverted_rgb = ANSI_256_TO_RGB[inverted as usize];
        assert!(inverted_rgb.r < 100 && inverted_rgb.g > 100 && inverted_rgb.b > 100);
    }

    #[test]
    fn test_invert_256_green_to_magenta() {
        let green_code = 10;
        let inverted = invert_256_color(green_code);
        let inverted_rgb = ANSI_256_TO_RGB[inverted as usize];
        assert!(inverted_rgb.r > 100 && inverted_rgb.g < 100 && inverted_rgb.b > 100);
    }

    #[test]
    fn test_invert_256_blue_to_yellow() {
        let blue_code = 12;
        let inverted = invert_256_color(blue_code);
        let inverted_rgb = ANSI_256_TO_RGB[inverted as usize];
        assert!(inverted_rgb.r > 100 && inverted_rgb.g > 100 && inverted_rgb.b < 100);
    }

    #[test]
    fn test_invert_256_grayscale_unchanged() {
        for code in 232..=255 {
            let inverted = invert_256_color(code);
            assert_eq!(
                inverted, code,
                "Grayscale color {} should remain unchanged when inverted",
                code
            );
        }
    }

    #[test]
    fn test_rgb_to_256_roundtrip() {
        for (code, rgb) in ANSI_256_TO_RGB.iter().enumerate() {
            let back = rgb_to_256(*rgb);
            let back_rgb = ANSI_256_TO_RGB[back as usize];
            let dist_orig = ((rgb.r as i32 - back_rgb.r as i32).pow(2)
                + (rgb.g as i32 - back_rgb.g as i32).pow(2)
                + (rgb.b as i32 - back_rgb.b as i32).pow(2)) as f32;
            assert!(
                dist_orig < 5000.0,
                "Color {} should round-trip close to itself, got dist {}",
                code,
                dist_orig
            );
        }
    }

    #[test]
    fn test_new_palettes_exist() {
        let _ = Palette::Slime;
        let _ = Palette::Mold;
        let _ = Palette::Fungus;
        let _ = Palette::Swamp;
        let _ = Palette::Moss;
    }

    #[test]
    fn test_slime_palette_gradient() {
        let min_color = map_brightness(0.0, Palette::Slime, false, false);
        let max_color = map_brightness(1.0, Palette::Slime, false, false);
        assert_eq!(min_color, 22);
        assert_eq!(max_color, 231);
    }

    #[test]
    fn test_mold_palette_gradient() {
        let min_color = map_brightness(0.0, Palette::Mold, false, false);
        let max_color = map_brightness(1.0, Palette::Mold, false, false);
        assert_eq!(min_color, 236);
        assert_eq!(max_color, 193);
    }

    #[test]
    fn test_fungus_palette_gradient() {
        let min_color = map_brightness(0.0, Palette::Fungus, false, false);
        let max_color = map_brightness(1.0, Palette::Fungus, false, false);
        assert_eq!(min_color, 232);
        assert_eq!(max_color, 223);
    }

    #[test]
    fn test_swamp_palette_gradient() {
        let min_color = map_brightness(0.0, Palette::Swamp, false, false);
        let max_color = map_brightness(1.0, Palette::Swamp, false, false);
        assert_eq!(min_color, 232);
        assert_eq!(max_color, 79);
    }

    #[test]
    fn test_moss_palette_gradient() {
        let min_color = map_brightness(0.0, Palette::Moss, false, false);
        let max_color = map_brightness(1.0, Palette::Moss, false, false);
        assert_eq!(min_color, 22);
        assert_eq!(max_color, 220);
    }

    #[test]
    fn test_slime_palette_rgb_values() {
        let color = map_brightness_rgb(0.5, Palette::Slime, false, false, 0.0);
        assert!(color.g > color.r && color.g > color.b);
    }

    #[test]
    fn test_fungus_palette_has_purple_tones() {
        let color = map_brightness_rgb(0.3, Palette::Fungus, false, false, 0.0);
        assert!(color.r > 50 && color.b > 50);
    }

    #[test]
    fn test_all_new_palettes_in_all_palettes_test() {
        let palettes = [
            Palette::Slime,
            Palette::Mold,
            Palette::Fungus,
            Palette::Swamp,
            Palette::Moss,
        ];
        for _ in palettes {
            let _color = map_brightness_rgb(0.5, Palette::Slime, false, false, 0.0);
        }
    }

    #[test]
    fn test_moss_palette_has_green_tones() {
        let color = map_brightness_rgb(0.5, Palette::Moss, false, false, 0.0);
        assert!(color.g > color.r && color.g > color.b);
    }

    #[test]
    fn test_map_brightness_rgb_hue_shift_with_invert() {
        let _color_shifted = map_brightness_rgb(0.5, Palette::Organic, false, true, 90.0);
    }

    #[test]
    fn test_map_brightness_rgb_hue_shift_with_reverse() {
        let _color_shifted = map_brightness_rgb(0.5, Palette::Organic, true, false, 90.0);
    }

    #[test]
    fn test_hex_to_rgb_valid() {
        let rgb = hex_to_rgb("ff0000").unwrap();
        assert_eq!(rgb.r, 255);
        assert_eq!(rgb.g, 0);
        assert_eq!(rgb.b, 0);
    }

    #[test]
    fn test_hex_to_rgb_with_hash() {
        let rgb = hex_to_rgb("#00ff00").unwrap();
        assert_eq!(rgb.r, 0);
        assert_eq!(rgb.g, 255);
        assert_eq!(rgb.b, 0);
    }

    #[test]
    fn test_hex_to_rgb_invalid() {
        assert!(hex_to_rgb("invalid").is_none());
        assert!(hex_to_rgb("fff").is_none());
    }

    #[test]
    fn test_map_species_brightness() {
        let base_color = RgbColor { r: 255, g: 0, b: 0 };
        let dark = map_species_brightness(0.0, base_color, false);
        let light = map_species_brightness(1.0, base_color, false);
        assert_ne!(dark, light, "Dark and light colors should be different");
    }

    #[test]
    fn test_map_species_brightness_reverse() {
        let base_color = RgbColor { r: 0, g: 0, b: 255 };
        let _dark = map_species_brightness(0.0, base_color, false);
        let _light = map_species_brightness(1.0, base_color, false);
        let dark_rev = map_species_brightness(0.0, base_color, true);
        let light_rev = map_species_brightness(1.0, base_color, true);
        assert_ne!(
            dark_rev, light_rev,
            "Reversed dark and light should be different"
        );
    }

    #[test]
    fn test_map_species_brightness_rgb() {
        let base_color = RgbColor {
            r: 255,
            g: 128,
            b: 0,
        };
        let dark = map_species_brightness_rgb(0.0, base_color, false);
        let light = map_species_brightness_rgb(1.0, base_color, false);
        assert_ne!(dark.r, light.r, "Red channel should differ");
    }

    #[test]
    fn test_gradient_stop_interpolation() {
        // Test with 2 stops - simple linear interpolation
        let stops = vec![
            GradientStop {
                position: 0.0,
                color: RgbColor { r: 0, g: 0, b: 0 },
            },
            GradientStop {
                position: 1.0,
                color: RgbColor {
                    r: 100,
                    g: 100,
                    b: 100,
                },
            },
        ];

        let color = interpolate_gradient(&stops, 0.5);
        assert_eq!(color.r, 50);
        assert_eq!(color.g, 50);
        assert_eq!(color.b, 50);
    }

    #[test]
    fn test_gradient_stop_interpolation_multiple_stops() {
        // Test with 3 stops
        let stops = vec![
            GradientStop {
                position: 0.0,
                color: RgbColor { r: 0, g: 0, b: 0 },
            },
            GradientStop {
                position: 0.5,
                color: RgbColor { r: 100, g: 0, b: 0 },
            },
            GradientStop {
                position: 1.0,
                color: RgbColor {
                    r: 100,
                    g: 100,
                    b: 100,
                },
            },
        ];

        // At 0.25, should be halfway between first and second stop
        let color = interpolate_gradient(&stops, 0.25);
        assert_eq!(color.r, 50);
        assert_eq!(color.g, 0);
        assert_eq!(color.b, 0);

        // At 0.75, should be halfway between second and third stop
        let color = interpolate_gradient(&stops, 0.75);
        assert_eq!(color.r, 100);
        assert_eq!(color.g, 50);
        assert_eq!(color.b, 50);
    }

    #[test]
    fn test_gradient_stop_edge_cases() {
        let stops = vec![
            GradientStop {
                position: 0.0,
                color: RgbColor {
                    r: 50,
                    g: 50,
                    b: 50,
                },
            },
            GradientStop {
                position: 1.0,
                color: RgbColor {
                    r: 200,
                    g: 200,
                    b: 200,
                },
            },
        ];

        // Exactly at start
        let color = interpolate_gradient(&stops, 0.0);
        assert_eq!(color.r, 50);

        // Exactly at end
        let color = interpolate_gradient(&stops, 1.0);
        assert_eq!(color.r, 200);

        // Clamping below 0
        let color = interpolate_gradient(&stops, -0.5);
        assert_eq!(color.r, 50);

        // Clamping above 1
        let color = interpolate_gradient(&stops, 1.5);
        assert_eq!(color.r, 200);
    }

    #[test]
    fn test_continuous_interpolation_vs_old_system() {
        // Verify that the new system produces smooth gradients
        // by checking that intermediate values between control points are different
        let color1 = map_brightness_rgb(0.45, Palette::Heat, false, false, 0.0);
        let color2 = map_brightness_rgb(0.50, Palette::Heat, false, false, 0.0);
        let color3 = map_brightness_rgb(0.55, Palette::Heat, false, false, 0.0);

        // These should all be different (continuous gradient)
        assert!(
            color1.r != color2.r || color1.g != color2.g || color1.b != color2.b,
            "Colors at 0.45 and 0.50 should differ"
        );
        assert!(
            color2.r != color3.r || color2.g != color3.g || color2.b != color3.b,
            "Colors at 0.50 and 0.55 should differ"
        );
    }
}
