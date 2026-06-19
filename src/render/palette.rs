use crate::render::gradients;
use std::collections::HashMap;
use std::sync::OnceLock;

/// Static cache of pre-computed gradient stops for built-in palettes.
/// This avoids allocating a new Vec on every call to get_gradient_stops.
fn get_gradient_stops_cache() -> &'static HashMap<Palette, Vec<GradientStop>> {
    static CACHE: OnceLock<HashMap<Palette, Vec<GradientStop>>> = OnceLock::new();
    CACHE.get_or_init(|| {
        let mut cache = HashMap::new();
        let built_in_palettes = [
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
            Palette::Cosmic,
            Palette::Ethereal,
        ];

        for palette in built_in_palettes {
            let oklch_stops = gradients::get_oklch_gradient(palette.clone());
            let stops = oklch_stops_to_gradient(oklch_stops, 64);
            cache.insert(palette, stops);
        }

        cache
    })
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
/// Color palette for rendering trails.
pub enum Palette {
    /// Organic green/brown tones.
    Organic,
    /// Thermal camera style (black-red-yellow-white).
    Heat,
    /// Deep ocean blues and teals.
    Ocean,
    /// Monochrome grayscale.
    Mono,
    /// Deep forest greens.
    Forest,
    /// Cyberpunk neon colors.
    Neon,
    /// Warm earth tones.
    Warm,
    /// High saturation vibrant colors.
    Vibrant,
    /// High contrast monochrome for readability.
    LegibleMono,
    /// Radioactive green slime.
    Slime,
    /// Dark moldy colors.
    Mold,
    /// Fungal growth colors.
    Fungus,
    /// Murky swamp colors.
    Swamp,
    /// Soft mossy greens.
    Moss,
    /// Deep space purples and blues.
    Cosmic,
    /// Ghostly pale colors.
    Ethereal,
    /// Custom user-defined palette.
    Custom(Vec<RgbColor>),
}

impl Palette {
    /// Returns the string representation of the palette name.
    pub fn name(&self) -> &str {
        match self {
            Palette::Organic => "Organic",
            Palette::Heat => "Heat",
            Palette::Ocean => "Ocean",
            Palette::Mono => "Mono",
            Palette::Forest => "Forest",
            Palette::Neon => "Neon",
            Palette::Warm => "Warm",
            Palette::Vibrant => "Vibrant",
            Palette::LegibleMono => "LegibleMono",
            Palette::Slime => "Slime",
            Palette::Mold => "Mold",
            Palette::Fungus => "Fungus",
            Palette::Swamp => "Swamp",
            Palette::Moss => "Moss",
            Palette::Cosmic => "Cosmic",
            Palette::Ethereal => "Ethereal",
            Palette::Custom(_) => "Custom",
        }
    }
}

/// List of all available color palettes for cycling.
/// This is the single source of truth for palette enumeration.
pub const ALL_PALETTES: [Palette; 16] = [
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
    Palette::Cosmic,
    Palette::Ethereal,
];

/// The number of palettes in ALL_PALETTES.
pub const NUM_PALETTES: usize = ALL_PALETTES.len();

/// Returns the number of available palettes.
pub fn num_palettes() -> usize {
    NUM_PALETTES
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
/// An RGB color value.
pub struct RgbColor {
    /// Red component (0-255).
    pub r: u8,
    /// Green component (0-255).
    pub g: u8,
    /// Blue component (0-255).
    pub b: u8,
}

impl RgbColor {
    /// Creates a new RGB color from individual components.
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    /// Creates a new RGB color from a 24-bit hex value.
    ///
    /// # Example
    /// ```
    /// use tslime::render::palette::RgbColor;
    /// let color = RgbColor::from_hex(0xFF5733); // Orange-red
    /// ```
    pub const fn from_hex(hex: u32) -> Self {
        Self {
            r: ((hex >> 16) & 0xFF) as u8,
            g: ((hex >> 8) & 0xFF) as u8,
            b: (hex & 0xFF) as u8,
        }
    }

    /// Returns a new color with the given alpha transparency applied.
    ///
    /// Alpha of 1.0 returns the original color, 0.0 returns black.
    pub fn with_alpha(&self, alpha: f32) -> Self {
        let alpha = alpha.clamp(0.0, 1.0);
        Self {
            r: (self.r as f32 * alpha) as u8,
            g: (self.g as f32 * alpha) as u8,
            b: (self.b as f32 * alpha) as u8,
        }
    }

    /// Blends this color with another color using the given factor.
    ///
    /// Factor of 0.0 returns self, 1.0 returns other, 0.5 returns average.
    pub fn blend(&self, other: &Self, factor: f32) -> Self {
        let factor = factor.clamp(0.0, 1.0);
        let inv = 1.0 - factor;
        Self {
            r: (self.r as f32 * inv + other.r as f32 * factor) as u8,
            g: (self.g as f32 * inv + other.g as f32 * factor) as u8,
            b: (self.b as f32 * inv + other.b as f32 * factor) as u8,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
/// An HSV color value.
pub struct HsvColor {
    /// Hue (0.0-360.0).
    pub h: f32,
    /// Saturation (0.0-1.0).
    pub s: f32,
    /// Value/Brightness (0.0-1.0).
    pub v: f32,
}

/// An OKLch perceptual color value.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct OklchColor {
    /// Perceptual lightness (0.0 = black, 1.0 = white).
    pub l: f32,
    /// Chroma / colorfulness (0.0 = gray, ~0.35 = max sRGB saturation).
    pub c: f32,
    /// Hue angle in degrees (0–360). NaN indicates "powerless" hue when chroma ≈ 0.
    pub h: f32,
}

/// Epsilon value for determining when chroma is effectively zero.
/// When chroma is below this threshold, hue is considered "powerless" (undefined).
/// This follows CSS Color Module Level 4 specification for missing/powerless components.
pub const OKLCH_EPSILON: f32 = 0.0001;

/// A gradient control point with position (0.0-1.0) and RGB color.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GradientStop {
    /// Position along the gradient (0.0 to 1.0).
    pub position: f32,
    /// Color at this position.
    pub color: RgbColor,
}

//==============================================================================
// OKLch Color Space — Perceptually Uniform Gradient System
//==============================================================================

/// A gradient control point defined in OKLch perceptual color space.
///
/// OKLch axes:
/// - `l`: Lightness  (0.0 = black, 1.0 = white)
/// - `c`: Chroma     (0.0 = gray, ~0.35 = maximum sRGB saturation)
/// - `h`: Hue angle  (degrees, 0–360: red≈30°, green≈142°, blue≈264°, purple≈285°)
#[derive(Clone, Copy, Debug)]
pub struct OklchStop {
    /// Position along the gradient (0.0 to 1.0).
    pub position: f32,
    /// Perceptual lightness (0.0–1.0).
    pub l: f32,
    /// Chroma / saturation (0.0–~0.35 for sRGB gamut).
    pub c: f32,
    /// Hue angle in degrees (0–360).
    pub h: f32,
}

/// Converts an OKLch color to sRGB `RgbColor`.
///
/// Performs the full OKLch → OKLab → LMS → linear-sRGB → gamma-sRGB chain.
/// Out-of-gamut values are clamped to [0, 255].
///
/// Note: If `h_deg` is NaN (indicating powerless hue when chroma ≈ 0), it is treated as 0
/// since the hue value doesn't affect the result for near-grayscale colors.
pub fn oklch_to_srgb(l: f32, c: f32, h_deg: f32) -> RgbColor {
    // Step 1: OKLch → OKLab
    // Handle NaN hue (powerless/undefined when chroma ≈ 0) - any value works since c ≈ 0
    let h_rad = if h_deg.is_nan() {
        0.0
    } else {
        h_deg.to_radians()
    };
    let a = c * h_rad.cos();
    let b = c * h_rad.sin();

    // Step 2: OKLab → LMS (cube roots of cone responses)
    // Coefficients from Björn Ottosson, "A perceptual color space for image
    // processing" (2020), https://bottosson.github.io/posts/oklab/
    let l_ = l + 0.396_337_8 * a + 0.215_803_76 * b;
    let m_ = l - 0.105_561_35 * a - 0.063_854_17 * b;
    let s_ = l - 0.089_484_18 * a - 1.291_485_5 * b;

    let lms_l = l_ * l_ * l_;
    let lms_m = m_ * m_ * m_;
    let lms_s = s_ * s_ * s_;

    // Step 3: LMS → linear sRGB
    // Red-row M/S constants predate Ottosson's 2021 matrix update (deltas ~1e-5,
    // visually negligible); green/blue rows and srgb_to_oklch match the updated post.
    let r_lin = 4.076_741_7 * lms_l - 3.307_736_3 * lms_m + 0.230_910_13 * lms_s;
    let g_lin = -1.268_438 * lms_l + 2.609_757_4 * lms_m - 0.341_319_4 * lms_s;
    let b_lin = -0.004_196_077 * lms_l - 0.703_418_6 * lms_m + 1.707_614_7 * lms_s;

    // Step 4: Linear sRGB → gamma-corrected sRGB (IEC 61966-2-1)
    let gamma = |x: f32| -> f32 {
        if x <= 0.0031308 {
            12.92 * x
        } else {
            1.055 * x.powf(1.0 / 2.4) - 0.055
        }
    };

    RgbColor {
        r: (gamma(r_lin).clamp(0.0, 1.0) * 255.0).round() as u8,
        g: (gamma(g_lin).clamp(0.0, 1.0) * 255.0).round() as u8,
        b: (gamma(b_lin).clamp(0.0, 1.0) * 255.0).round() as u8,
    }
}

/// Converts sRGB `RgbColor` to OKLch perceptual color space.
///
/// Performs the full gamma-sRGB → linear-sRGB → LMS → OKLab → OKLch chain.
pub fn srgb_to_oklch(rgb: RgbColor) -> OklchColor {
    // Step 1: gamma-corrected sRGB → linear sRGB
    let inv_gamma = |x: f32| -> f32 {
        if x <= 0.04045 {
            x / 12.92
        } else {
            ((x + 0.055) / 1.055).powf(2.4)
        }
    };
    let r = inv_gamma(rgb.r as f32 / 255.0);
    let g = inv_gamma(rgb.g as f32 / 255.0);
    let b = inv_gamma(rgb.b as f32 / 255.0);

    // Step 2: linear sRGB → LMS
    let l = 0.412_221_47 * r + 0.536_332_54 * g + 0.051_445_99 * b;
    let m = 0.211_903_5 * r + 0.680_699_5 * g + 0.107_396_96 * b;
    let s = 0.088_302_46 * r + 0.281_718_84 * g + 0.629_978_7 * b;

    // Step 3: cube root (LMS → LMS cube-root space)
    let l_ = l.cbrt();
    let m_ = m.cbrt();
    let s_ = s.cbrt();

    // Step 4: LMS cube roots → OKLab
    let lab_l = 0.210_454_26 * l_ + 0.793_617_8 * m_ - 0.004_072_047 * s_;
    let lab_a = 1.977_998_5 * l_ - 2.428_592_2 * m_ + 0.450_593_7 * s_;
    let lab_b = 0.025_904_037 * l_ + 0.782_771_77 * m_ - 0.808_675_77 * s_;

    // Step 5: OKLab → OKLch
    let c = (lab_a * lab_a + lab_b * lab_b).sqrt();
    // When chroma is effectively zero, hue is "powerless" (undefined).
    // Per CSS Color Module Level 4, return NaN to indicate missing component.
    let h = if c < OKLCH_EPSILON {
        f32::NAN
    } else {
        let h = lab_b.atan2(lab_a).to_degrees();
        if h < 0.0 {
            h + 360.0
        } else {
            h
        }
    };

    OklchColor { l: lab_l, c, h }
}

/// Converts an `OklchColor` to sRGB `RgbColor`.
pub fn oklch_to_rgb(oklch: OklchColor) -> RgbColor {
    oklch_to_srgb(oklch.l, oklch.c, oklch.h)
}

/// Interpolates between OKLch stops at parameter `t` using short-arc hue interpolation.
fn interpolate_oklch_stops(stops: &[OklchStop], t: f32) -> RgbColor {
    let t = t.clamp(0.0, 1.0);

    if stops.is_empty() {
        return RgbColor { r: 0, g: 0, b: 0 };
    }
    if stops.len() == 1 {
        return oklch_to_srgb(stops[0].l, stops[0].c, stops[0].h);
    }

    // Binary search for the two bracketing stops
    let mut lo = 0;
    let mut hi = stops.len() - 1;
    while lo < hi {
        let mid = (lo + hi + 1) / 2;
        if stops[mid].position <= t {
            lo = mid;
        } else {
            hi = mid - 1;
        }
    }
    // lo is now the index of the largest stop position <= t
    let lo = lo.min(stops.len() - 1);
    let hi = (lo + 1).min(stops.len() - 1);

    if lo == hi {
        let s = &stops[lo];
        return oklch_to_srgb(s.l, s.c, s.h);
    }

    let range = stops[hi].position - stops[lo].position;
    if range < f32::EPSILON {
        let s = &stops[lo];
        return oklch_to_srgb(s.l, s.c, s.h);
    }

    let local_t = (t - stops[lo].position) / range;

    // Interpolate L and C linearly
    let l = stops[lo].l + (stops[hi].l - stops[lo].l) * local_t;
    let c = stops[lo].c + (stops[hi].c - stops[lo].c) * local_t;

    // Hue: short-arc interpolation (always takes the shorter path around the wheel)
    let h0 = stops[lo].h;
    let h1 = stops[hi].h;
    let dh = {
        let diff = (h1 - h0).rem_euclid(360.0);
        if diff > 180.0 {
            diff - 360.0
        } else {
            diff
        }
    };
    let h = (h0 + dh * local_t).rem_euclid(360.0);

    oklch_to_srgb(l, c, h)
}

/// Builds a dense `Vec<GradientStop>` from OKLch stops by sampling `n` evenly-spaced
/// points and converting each through OKLch → sRGB.
///
/// Using 64 samples means the linear-sRGB interpolation done by `interpolate_gradient`
/// operates on ≤1.6%-wide intervals — perceptually indistinguishable from true OKLch
/// interpolation.
fn oklch_stops_to_gradient(stops: &[OklchStop], n: usize) -> Vec<GradientStop> {
    let n = n.max(2);
    (0..n)
        .map(|i| {
            let t = i as f32 / (n - 1) as f32;
            GradientStop {
                position: t,
                color: interpolate_oklch_stops(stops, t),
            }
        })
        .collect()
}

//==============================================================================
// Intensity Mapping - Non-linear transformation of intensity values
//==============================================================================

/// Mapping function that transforms intensity values before palette lookup.
/// All functions guarantee: f(0) = 0, f(1) = 1, and monotonically increasing output.
#[derive(Clone, Debug, PartialEq, Default)]
pub enum MappingFunction {
    /// Linear: f(x) = x
    #[default]
    Linear,

    /// Logarithmic: compresses high values, expands low values.
    /// f(x) = log(1 + x * base) / log(1 + base)
    Logarithmic {
        /// Base of the logarithm. Higher values = stronger compression of brights.
        base: f32,
    },

    /// Exponential: expands high values, compresses low values.
    /// f(x) = (base^x - 1) / (base - 1)
    Exponential {
        /// Base of the exponential. Higher values = stronger expansion of brights.
        base: f32,
    },

    /// Power/Gamma: f(x) = x^gamma.
    /// gamma < 1: more darks visible, gamma > 1: more brights visible
    Power {
        /// Gamma exponent. Values < 1 expand darks, > 1 expand brights.
        gamma: f32,
    },

    /// Square root: f(x) = sqrt(x) - gentler than log
    SquareRoot,

    /// Square: f(x) = x² - gentler than exponential
    Square,

    /// Sigmoid/S-curve: smooth compression at extremes, expansion in middle.
    /// Normalized to guarantee f(0)=0, f(1)=1
    Sigmoid {
        /// Steepness of the curve. Higher values = sharper transition.
        steepness: f32,
    },

    /// Smoothstep: cubic Hermite interpolation (common in graphics).
    /// f(x) = 3x² - 2x³
    Smoothstep,

    /// Quantize: discrete steps (posterization effect).
    Quantize {
        /// Number of discrete levels (2-255).
        levels: u8,
    },

    /// Perlin distortion: adds organic noise-based variation.
    /// Uses endpoint anchoring to ensure f(0)≈0, f(1)≈1
    Perlin {
        /// Strength of distortion (0.0-0.3 recommended).
        amplitude: f32,
        /// How "fast" the noise changes (1.0-10.0 recommended).
        frequency: f32,
        /// Seed for reproducible noise.
        seed: u64,
    },
}

impl MappingFunction {
    /// Applies the mapping function to a value in [0, 1].
    ///
    /// # Guarantees
    /// - Output is always in [0, 1]
    /// - For non-Perlin functions: f(0)=0 and f(1)=1 exactly
    /// - For Perlin: f(0)≈0 and f(1)≈1 with organic variation in between
    #[inline]
    pub fn apply(&self, x: f32) -> f32 {
        let x = x.clamp(0.0, 1.0);

        match self {
            MappingFunction::Linear => x,

            MappingFunction::Logarithmic { base } => {
                // f(x) = log(1 + x*base) / log(1 + base)
                // f(0) = log(1)/log(1+base) = 0 ✓
                // f(1) = log(1+base)/log(1+base) = 1 ✓
                let base = base.max(0.001);
                (1.0 + x * base).ln() / (1.0 + base).ln()
            }

            MappingFunction::Exponential { base } => {
                // f(x) = (base^x - 1) / (base - 1)
                // f(0) = (1 - 1)/(base - 1) = 0 ✓
                // f(1) = (base - 1)/(base - 1) = 1 ✓
                let base = base.max(1.001);
                (base.powf(x) - 1.0) / (base - 1.0)
            }

            MappingFunction::Power { gamma } => {
                // f(x) = x^gamma
                // f(0) = 0 ✓, f(1) = 1 ✓
                let gamma = gamma.max(0.001);
                x.powf(gamma)
            }

            MappingFunction::SquareRoot => x.sqrt(), // 0→0, 1→1 ✓

            MappingFunction::Square => x * x, // 0→0, 1→1 ✓

            MappingFunction::Sigmoid { steepness } => {
                // Normalized sigmoid to ensure f(0)=0, f(1)=1
                let k = steepness.max(0.1);
                let sigmoid = |t: f32| 1.0 / (1.0 + (-t).exp());
                let s_min = sigmoid(-k * 0.5);
                let s_max = sigmoid(k * 0.5);
                let raw = sigmoid(k * (x - 0.5));
                (raw - s_min) / (s_max - s_min)
            }

            MappingFunction::Smoothstep => {
                // f(x) = 3x² - 2x³
                // f(0) = 0 ✓, f(1) = 3 - 2 = 1 ✓
                x * x * (3.0 - 2.0 * x)
            }

            MappingFunction::Quantize { levels } => {
                let levels = (*levels).max(2) as f32;
                let quantized = (x * levels).floor() / (levels - 1.0);
                quantized.clamp(0.0, 1.0)
            }

            MappingFunction::Perlin {
                amplitude,
                frequency,
                seed,
            } => Self::apply_perlin(x, *amplitude, *frequency, *seed),
        }
    }

    /// Applies Perlin-based distortion while maintaining endpoint anchoring.
    fn apply_perlin(x: f32, amplitude: f32, frequency: f32, seed: u64) -> f32 {
        // Generate deterministic noise value for this x position
        let noise_val = Self::perlin_1d(x * frequency, seed);

        // Endpoint anchoring: reduce distortion near 0 and 1
        // w(x) = 4x(1-x), which is 0 at endpoints, 1 at x=0.5
        let endpoint_weight = 4.0 * x * (1.0 - x);

        // Apply weighted distortion
        let distorted = x + noise_val * amplitude * endpoint_weight;

        distorted.clamp(0.0, 1.0)
    }

    /// Simple 1D Perlin-like noise using hash function.
    /// Returns value in [-1, 1].
    fn perlin_1d(x: f32, seed: u64) -> f32 {
        let x0 = x.floor() as i32;
        let x1 = x0 + 1;

        // Fractional part with smoothstep interpolation
        let t = x - x.floor();
        let t_smooth = t * t * (3.0 - 2.0 * t);

        // Hash-based gradient at each integer point
        let g0 = Self::hash_gradient(x0, seed);
        let g1 = Self::hash_gradient(x1, seed);

        // Dot product with distance vectors and interpolate
        let d0 = t * g0;
        let d1 = (t - 1.0) * g1;

        d0 + t_smooth * (d1 - d0)
    }

    /// Hash function to generate pseudo-random gradient at integer position.
    fn hash_gradient(x: i32, seed: u64) -> f32 {
        let mut h = (x as u64).wrapping_mul(0x9E3779B97F4A7C15);
        h ^= seed;
        h = h.wrapping_mul(0x517CC1B727220A95);
        h ^= h >> 32;

        // Convert to [-1, 1]
        (h as f32 / u64::MAX as f32) * 2.0 - 1.0
    }
}

/// A segment of the intensity domain with its own mapping function.
#[derive(Clone, Debug, PartialEq)]
pub struct MappingSegment {
    /// Start of this segment in input domain [0, 1]
    pub start: f32,
    /// End of this segment in input domain [0, 1]
    pub end: f32,
    /// Mapping function applied within this segment
    pub function: MappingFunction,
}

/// Error type for invalid mapping configurations.
#[derive(Debug, Clone, PartialEq)]
pub enum MappingError {
    /// No segments provided.
    NoSegments,
    /// Domain [0, 1] not fully covered.
    DomainNotCovered {
        /// Description of why the domain is not covered.
        reason: String,
    },
    /// Gap between segments.
    SegmentGap {
        /// Index of the segment before the gap.
        segment_index: usize,
        /// End of the previous segment.
        gap_start: f32,
        /// Start of the next segment.
        gap_end: f32,
    },
    /// Invalid segment definition.
    InvalidSegment {
        /// Index of the invalid segment.
        segment_index: usize,
        /// Description of why the segment is invalid.
        reason: String,
    },
}

impl std::fmt::Display for MappingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MappingError::NoSegments => write!(f, "No mapping segments provided"),
            MappingError::DomainNotCovered { reason } => {
                write!(f, "Domain [0,1] not fully covered: {}", reason)
            }
            MappingError::SegmentGap {
                segment_index,
                gap_start,
                gap_end,
            } => write!(
                f,
                "Gap between segments {} and {}: [{}, {}]",
                segment_index,
                segment_index + 1,
                gap_start,
                gap_end
            ),
            MappingError::InvalidSegment {
                segment_index,
                reason,
            } => write!(f, "Invalid segment {}: {}", segment_index, reason),
        }
    }
}

impl std::error::Error for MappingError {}

/// Complete intensity mapping configuration.
///
/// Maps intensity values [0, 1] through one or more segments, each with its own
/// mapping function. Segments must be contiguous and cover the entire domain.
///
/// # Guarantees
/// - Input 0.0 maps to output 0.0 (first palette color)
/// - Input 1.0 maps to output 1.0 (last palette color)
/// - All intermediate palette colors are reachable (monotonic functions)
#[derive(Clone, Debug, PartialEq)]
pub struct IntensityMapping {
    segments: Vec<MappingSegment>,
}

impl IntensityMapping {
    /// Creates a new IntensityMapping with validation.
    ///
    /// # Errors
    /// Returns `MappingError` if segments don't fully cover [0, 1].
    pub fn new(segments: Vec<MappingSegment>) -> Result<Self, MappingError> {
        Self::validate_segments(&segments)?;
        Ok(Self { segments })
    }

    /// Validates that segments form a valid partition of [0, 1].
    fn validate_segments(segments: &[MappingSegment]) -> Result<(), MappingError> {
        use crate::config_defaults::math::EPSILON;

        if segments.is_empty() {
            return Err(MappingError::NoSegments);
        }

        // Check first segment starts at 0
        if (segments[0].start - 0.0).abs() > EPSILON {
            return Err(MappingError::DomainNotCovered {
                reason: format!("First segment starts at {}, not 0.0", segments[0].start),
            });
        }

        // Check last segment ends at 1
        let last = segments.last().unwrap();
        if (last.end - 1.0).abs() > EPSILON {
            return Err(MappingError::DomainNotCovered {
                reason: format!("Last segment ends at {}, not 1.0", last.end),
            });
        }

        // Check segments are contiguous (no gaps or overlaps)
        for i in 0..segments.len() - 1 {
            let current_end = segments[i].end;
            let next_start = segments[i + 1].start;

            if (current_end - next_start).abs() > EPSILON {
                return Err(MappingError::SegmentGap {
                    segment_index: i,
                    gap_start: current_end,
                    gap_end: next_start,
                });
            }
        }

        // Check each segment has positive width
        for (i, seg) in segments.iter().enumerate() {
            if seg.end <= seg.start {
                return Err(MappingError::InvalidSegment {
                    segment_index: i,
                    reason: format!("Segment end ({}) <= start ({})", seg.end, seg.start),
                });
            }
        }

        Ok(())
    }

    /// Applies the mapping to an intensity value.
    ///
    /// # Guarantees
    /// - Input 0.0 maps to output 0.0
    /// - Input 1.0 maps to output 1.0
    /// - Output is always in [0, 1]
    #[inline]
    pub fn apply(&self, intensity: f32) -> f32 {
        let intensity = intensity.clamp(0.0, 1.0);

        for segment in &self.segments {
            if intensity >= segment.start && intensity <= segment.end {
                let segment_width = segment.end - segment.start;

                // Normalize to [0, 1] within this segment
                let local_t = if segment_width > 0.0 {
                    (intensity - segment.start) / segment_width
                } else {
                    0.0
                };

                // Apply the mapping function (guarantees [0,1] → [0,1])
                let mapped_local = segment.function.apply(local_t);

                // Scale back to segment's output range
                return segment.start + mapped_local * segment_width;
            }
        }

        // Fallback (should never reach due to validation)
        intensity
    }

    /// Returns the segments for inspection.
    pub fn segments(&self) -> &[MappingSegment] {
        &self.segments
    }
}

// Convenience constructors (pre-validated, cannot fail)
impl IntensityMapping {
    /// Linear mapping.
    pub fn linear() -> Self {
        Self {
            segments: vec![MappingSegment {
                start: 0.0,
                end: 1.0,
                function: MappingFunction::Linear,
            }],
        }
    }

    /// Uniform logarithmic mapping across entire range.
    pub fn logarithmic(base: f32) -> Self {
        Self {
            segments: vec![MappingSegment {
                start: 0.0,
                end: 1.0,
                function: MappingFunction::Logarithmic { base },
            }],
        }
    }

    /// Uniform exponential mapping across entire range.
    pub fn exponential(base: f32) -> Self {
        Self {
            segments: vec![MappingSegment {
                start: 0.0,
                end: 1.0,
                function: MappingFunction::Exponential { base },
            }],
        }
    }

    /// Uniform power/gamma mapping across entire range.
    pub fn power(gamma: f32) -> Self {
        Self {
            segments: vec![MappingSegment {
                start: 0.0,
                end: 1.0,
                function: MappingFunction::Power { gamma },
            }],
        }
    }

    /// Lower colors mapped linearly, upper colors mapped logarithmically.
    /// For an 11-color palette: [0, 6/11] linear, [6/11, 1] logarithmic.
    pub fn linear_log_split(log_base: f32) -> Self {
        use crate::config_defaults::palette::{DEFAULT_PALETTE_STEPS, LINEAR_COLOR_COUNT};
        let split_point = LINEAR_COLOR_COUNT / DEFAULT_PALETTE_STEPS as f32; // ≈ 0.545454...
        Self {
            segments: vec![
                MappingSegment {
                    start: 0.0,
                    end: split_point,
                    function: MappingFunction::Linear,
                },
                MappingSegment {
                    start: split_point,
                    end: 1.0,
                    function: MappingFunction::Logarithmic { base: log_base },
                },
            ],
        }
    }

    /// Split at arbitrary point with configurable functions.
    pub fn split_at(
        split_point: f32,
        lower_fn: MappingFunction,
        upper_fn: MappingFunction,
    ) -> Result<Self, MappingError> {
        let split_point = split_point.clamp(0.001, 0.999);
        Self::new(vec![
            MappingSegment {
                start: 0.0,
                end: split_point,
                function: lower_fn,
            },
            MappingSegment {
                start: split_point,
                end: 1.0,
                function: upper_fn,
            },
        ])
    }

    /// Perlin distortion across entire range.
    pub fn perlin(amplitude: f32, frequency: f32, seed: u64) -> Self {
        Self {
            segments: vec![MappingSegment {
                start: 0.0,
                end: 1.0,
                function: MappingFunction::Perlin {
                    amplitude,
                    frequency,
                    seed,
                },
            }],
        }
    }

    /// Smoothstep mapping for natural-looking gradients.
    pub fn smoothstep() -> Self {
        Self {
            segments: vec![MappingSegment {
                start: 0.0,
                end: 1.0,
                function: MappingFunction::Smoothstep,
            }],
        }
    }

    /// Quantized mapping for posterization effect.
    pub fn quantize(levels: u8) -> Self {
        Self {
            segments: vec![MappingSegment {
                start: 0.0,
                end: 1.0,
                function: MappingFunction::Quantize { levels },
            }],
        }
    }
}

impl Default for IntensityMapping {
    fn default() -> Self {
        Self::logarithmic(10.0)
    }
}

/// Interpolates linearly between gradient stops; supports any number of control points.
#[inline]
pub fn interpolate_gradient(stops: &[GradientStop], t: f32) -> RgbColor {
    let t = t.clamp(0.0, 1.0);

    if stops.is_empty() {
        return RgbColor { r: 0, g: 0, b: 0 };
    }

    if stops.len() == 1 {
        return stops[0].color;
    }

    // Binary search for the two stops to interpolate between
    let mut lo = 0;
    let mut hi = stops.len() - 1;
    while lo < hi {
        let mid = (lo + hi + 1) / 2;
        if stops[mid].position <= t {
            lo = mid;
        } else {
            hi = mid - 1;
        }
    }
    let lower_idx = lo.min(stops.len() - 1);
    let upper_idx = (lo + 1).min(stops.len() - 1);

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

/// Mapping from ANSI 256 color codes to RGB values.
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

/// Converts an RGB color to HSV.
#[inline]
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

/// Converts an HSV color to RGB.
#[inline]
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

/// Rotates the hue of an HSV color.
pub fn rotate_hue(hsv: HsvColor, degrees: f32) -> HsvColor {
    use crate::config_defaults::math::DEGREES_IN_CIRCLE;
    HsvColor {
        h: (hsv.h + degrees) % DEGREES_IN_CIRCLE,
        s: hsv.s,
        v: hsv.v,
    }
}

/// Approximates the nearest ANSI 256 color code for an RGB color.
///
/// Near-gray colors snap to the grayscale ramp (232-255), close matches to the
/// 16 system colors are taken directly, and everything else rounds into the
/// 6×6×6 color cube — a fast heuristic, not an exhaustive nearest search.
#[inline]
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

/// Inverts an ANSI 256 color code (hue rotation of 180 degrees).
pub fn invert_256_color(color_code: u8) -> u8 {
    use crate::config_defaults::math::DEGREES_HALF_CIRCLE;
    let rgb = ANSI_256_TO_RGB[color_code as usize];
    let hsv = rgb_to_hsv(rgb);
    let rotated = rotate_hue(hsv, DEGREES_HALF_CIRCLE);
    let new_rgb = hsv_to_rgb(rotated);
    rgb_to_256(new_rgb)
}

/// Parses a hex color string (e.g., "#FF0000" or "FF0000") into `RgbColor`.
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

/// Internal helper to compute HSV color from brightness and base color.
/// Shared logic between map_species_brightness and map_species_brightness_rgb.
fn compute_species_hsv(brightness: f32, base_color: RgbColor, reverse: bool) -> HsvColor {
    use crate::config_defaults::hsv;

    let hsv_color = rgb_to_hsv(base_color);
    let brightness = if reverse {
        1.0 - brightness
    } else {
        brightness
    };

    let t = brightness.clamp(0.0, 1.0);

    let min_s = hsv::MIN_SATURATION;
    let max_s = hsv_color.s.max(hsv::MAX_SATURATION_FLOOR);
    let min_v = hsv::MIN_VALUE;
    let max_v =
        (hsv_color.v * hsv::MAX_VALUE_SCALE + hsv::MAX_VALUE_OFFSET).min(hsv::MAX_VALUE_CAP);

    let s = min_s + (max_s - min_s) * t;
    let v = min_v + (max_v - min_v) * t;

    HsvColor {
        h: hsv_color.h,
        s,
        v,
    }
}

/// Maps brightness to a color based on a base species color.
///
/// This generates a gradient from dark to the base color (modulated by brightness).
pub fn map_species_brightness(brightness: f32, base_color: RgbColor, reverse: bool) -> u8 {
    let final_hsv = compute_species_hsv(brightness, base_color, reverse);
    let final_rgb = hsv_to_rgb(final_hsv);
    rgb_to_256(final_rgb)
}

/// Maps brightness to an RGB color based on a base species color.
pub fn map_species_brightness_rgb(
    brightness: f32,
    base_color: RgbColor,
    reverse: bool,
) -> RgbColor {
    let final_hsv = compute_species_hsv(brightness, base_color, reverse);
    hsv_to_rgb(final_hsv)
}

// Thread-local storage for custom palette gradient stops to avoid repeated allocations.
// This is used only for Palette::Custom variants.
thread_local! {
    static CUSTOM_STOPS_CACHE: std::cell::RefCell<Option<(Vec<RgbColor>, Vec<GradientStop>)>> = const { std::cell::RefCell::new(None) };
}

/// Returns gradient stops for a palette (always a cloned `Vec`).
///
/// Built-in palettes clone from the global static cache; custom palettes are
/// cached in thread-local storage keyed on the color list, so repeated calls
/// with the same palette avoid recomputing the stops.
pub(crate) fn get_gradient_stops(palette: &Palette) -> Vec<GradientStop> {
    match palette {
        Palette::Custom(colors) => {
            // Check if we have this custom palette cached in thread-local storage
            CUSTOM_STOPS_CACHE.with(|cache| {
                let mut cache = cache.borrow_mut();

                // Check if the cached palette matches the current one
                if let Some((cached_colors, _)) = cache.as_ref() {
                    if cached_colors == colors {
                        // Return a clone of the cached stops (necessary because we return Vec)
                        return cache.as_ref().unwrap().1.clone();
                    }
                }

                // Compute new stops for this custom palette
                let stops: Vec<GradientStop> = colors
                    .iter()
                    .enumerate()
                    .map(|(i, &color)| GradientStop {
                        position: i as f32 / (colors.len() - 1).max(1) as f32,
                        color,
                    })
                    .collect();

                // Store in cache
                *cache = Some((colors.clone(), stops.clone()));
                stops
            })
        }
        built_in => {
            // For built-in palettes, use the global static cache
            // Clone is necessary because we return Vec, but this only allocates once per palette
            get_gradient_stops_cache()
                .get(built_in)
                .cloned()
                .unwrap_or_default()
        }
    }
}

fn invert_color(color_code: u8) -> u8 {
    invert_256_color(color_code)
}

/// Linearly interpolates between custom palette colors for a brightness in [0, 1].
fn interpolate_custom_color(colors: &[RgbColor], brightness: f32) -> RgbColor {
    let num_colors = colors.len();
    if num_colors == 0 {
        return RgbColor { r: 0, g: 0, b: 0 };
    }
    if num_colors == 1 {
        return colors[0];
    }

    // Map brightness (0.0-1.0) to position in color array
    let t = brightness.clamp(0.0, 1.0) * (num_colors - 1) as f32;
    let segment = t.floor() as usize;
    let segment_t = t.fract();

    let start_idx = segment.min(num_colors - 1);
    let end_idx = (segment + 1).min(num_colors - 1);

    let start_color = colors[start_idx];
    let end_color = colors[end_idx];

    RgbColor {
        r: ((start_color.r as f32 * (1.0 - segment_t) + end_color.r as f32 * segment_t) as u8),
        g: ((start_color.g as f32 * (1.0 - segment_t) + end_color.g as f32 * segment_t) as u8),
        b: ((start_color.b as f32 * (1.0 - segment_t) + end_color.b as f32 * segment_t) as u8),
    }
}

/// Maps brightness (0.0-1.0) to an ANSI 256 color code for the selected palette.
///
/// `reverse` flips the intensity before lookup, `invert` hue-rotates the final
/// color by 180°, and `mapping` applies an optional non-linear intensity curve.
///
/// This is a thin wrapper around [`map_brightness_cycled`] with an identity
/// [`PaletteCycle`]. All existing callers remain byte-identical.
pub fn map_brightness(
    brightness: f32,
    palette: Palette,
    reverse: bool,
    invert: bool,
    mapping: Option<&IntensityMapping>,
) -> u8 {
    map_brightness_cycled(
        brightness,
        palette,
        reverse,
        invert,
        mapping,
        PaletteCycle::default(),
    )
}

/// 256-color counterpart to [`map_brightness_rgb_cycled`].
///
/// Applies a [`PaletteCycle`] remap at pass-order step 5 (between the tone
/// curve and the gradient lookup). The non-cycled wrapper delegates here with
/// an identity cycle, so existing callers stay byte-identical.
pub fn map_brightness_cycled(
    brightness: f32,
    palette: Palette,
    reverse: bool,
    invert: bool,
    mapping: Option<&IntensityMapping>,
    cycle: PaletteCycle,
) -> u8 {
    let mut brightness = brightness.clamp(0.0, 1.0);

    if reverse {
        brightness = 1.0 - brightness;
    }

    // Apply non-linear intensity mapping if provided
    if let Some(mapping) = mapping {
        brightness = mapping.apply(brightness);
    }

    // Pass-order step 5: spatial palette repeat, pre-lookup.
    let t = cycle.map(brightness);

    // Get the color based on palette type
    let color = match &palette {
        Palette::Custom(colors) => {
            // Custom palettes: interpolate to RGB, then convert to 256-color
            let rgb = interpolate_custom_color(colors, t);
            rgb_to_256(rgb)
        }
        _ => {
            // For built-in palettes, use the pre-computed gradient
            let gradient = gradients::get_256_gradient(palette);
            let position = t * (gradient.len() - 1) as f32;
            let lower = position.floor() as usize;
            let upper = position.ceil() as usize;
            let fraction = position - lower as f32;

            if upper == lower || fraction < 0.5 {
                gradient[lower]
            } else {
                gradient[upper]
            }
        }
    };

    if invert {
        invert_color(color)
    } else {
        color
    }
}

fn invert_rgb(rgb: RgbColor) -> RgbColor {
    RgbColor {
        r: 255 - rgb.r,
        g: 255 - rgb.g,
        b: 255 - rgb.b,
    }
}

/// Maps brightness (0.0-1.0) to an RGB color for the selected palette.
///
/// `reverse` flips the intensity before lookup, `invert` complements the final
/// RGB color, `hue_shift` rotates the hue by that many degrees, and `mapping`
/// applies an optional non-linear intensity curve.
///
/// This is a thin wrapper around [`map_brightness_rgb_cycled`] with an identity
/// [`PaletteCycle`]. All existing callers remain byte-identical.
pub fn map_brightness_rgb(
    brightness: f32,
    palette: Palette,
    reverse: bool,
    invert: bool,
    hue_shift: f32,
    mapping: Option<&IntensityMapping>,
) -> RgbColor {
    map_brightness_rgb_cycled(
        brightness,
        palette,
        reverse,
        invert,
        hue_shift,
        mapping,
        PaletteCycle::default(),
    )
}

/// Same as [`map_brightness_rgb`] but applies a [`PaletteCycle`] remap at
/// pass-order step 5 (between the tone curve and the gradient lookup). The
/// non-cycled wrapper delegates here with an identity cycle, so existing
/// callers stay byte-identical.
#[allow(clippy::too_many_arguments)]
pub fn map_brightness_rgb_cycled(
    brightness: f32,
    palette: Palette,
    reverse: bool,
    invert: bool,
    hue_shift: f32,
    mapping: Option<&IntensityMapping>,
    cycle: PaletteCycle,
) -> RgbColor {
    let mut brightness = brightness.clamp(0.0, 1.0);

    if reverse {
        brightness = 1.0 - brightness;
    }

    // Apply non-linear intensity mapping if provided
    if let Some(mapping) = mapping {
        brightness = mapping.apply(brightness);
    }

    // Pass-order step 5: spatial palette repeat, pre-lookup.
    let t = cycle.map(brightness);

    // Use the new gradient interpolation system
    let stops = get_gradient_stops(&palette);
    let mut final_color = interpolate_gradient(&stops, t);

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

/// Returns a vivid accent color representative of the current palette configuration.
///
/// Samples the palette at brightness = 0.85 so the accent sits in the bright but not
/// fully-saturated region of each palette's gradient. Suitable for title badges,
/// palette swatches, and key-binding highlights in the TUI.
///
/// Arguments mirror `map_brightness_rgb`: `reverse`, `invert`, `hue_offset`, and
/// an optional `intensity_mapping`. Passing the same values used for simulation
/// rendering ensures the accent matches what the user sees on screen.
pub fn palette_accent_color(
    palette: &Palette,
    reverse: bool,
    invert: bool,
    hue_offset: f32,
    mapping: Option<&IntensityMapping>,
) -> RgbColor {
    map_brightness_rgb(0.85, palette.clone(), reverse, invert, hue_offset, mapping)
}

/// Generates an ANSI escape sequence for a truecolor foreground or background.
pub fn truecolor_ansi(r: u8, g: u8, b: u8, is_fg: bool) -> String {
    format!("\x1b[{};2;{};{};{}m", if is_fg { 38 } else { 48 }, r, g, b)
}

/// Generates an ANSI escape sequence for a truecolor foreground.
pub fn truecolor_ansi_fg(r: u8, g: u8, b: u8) -> String {
    truecolor_ansi(r, g, b, true)
}

/// Generates an ANSI escape sequence for a truecolor background.
pub fn truecolor_ansi_bg(r: u8, g: u8, b: u8) -> String {
    truecolor_ansi(r, g, b, false)
}

/// Color-modulation mode for temporal-difference coloring (lever 3).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TemporalMode {
    /// Rotate the base color's hue in OKLch by the (signed) temporal difference.
    Hue,
    /// Blend the base color toward a front-accent color (Bleuje-style).
    Accent,
}

/// Spatial palette-repeat mode (lever 6). Remaps the gradient index `t`
/// pre-lookup so the palette tiles across the brightness range.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PaletteCycleMode {
    /// Sawtooth `fract(t·n)` — seam unless the palette endpoints match.
    Wrap,
    /// Triangle `1 − |1 − fract(t·n)·2|` — ping-pong, seamless on any palette.
    #[default]
    Mirror,
}

impl std::fmt::Display for PaletteCycleMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PaletteCycleMode::Wrap => write!(f, "wrap"),
            PaletteCycleMode::Mirror => write!(f, "mirror"),
        }
    }
}

/// Palette-cycle configuration: how many times the palette repeats across the
/// brightness range and the repeat mode. `cycles = 1` (the default) is identity.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PaletteCycle {
    /// Number of palette repeats across `[0,1]`. `1` = no repeat (identity).
    pub cycles: u32,
    /// Repeat mode (wrap/sawtooth or mirror/triangle).
    pub mode: PaletteCycleMode,
}

impl Default for PaletteCycle {
    fn default() -> Self {
        Self {
            cycles: 1,
            mode: PaletteCycleMode::Mirror,
        }
    }
}

impl PaletteCycle {
    /// True when the remap is a no-op (`cycles ≤ 1`).
    #[inline]
    pub fn is_identity(&self) -> bool {
        self.cycles <= 1
    }

    /// Remap a tone-mapped brightness `t` (0..1) to the cycled gradient index
    /// (spec §6 step 5). Identity when `cycles ≤ 1`.
    #[inline]
    pub fn map(&self, t: f32) -> f32 {
        if self.is_identity() {
            return t;
        }
        let n = self.cycles.max(1) as f32;
        let saw = (t * n).fract();
        match self.mode {
            PaletteCycleMode::Wrap => saw,
            PaletteCycleMode::Mirror => 1.0 - (1.0 - saw * 2.0).abs(),
        }
    }
}

/// Shared per-subpixel colorizer: the single colorize pass for ALL outputs —
/// the TUI render path (`FrameBuffer::from_downsampled`, which also feeds GIF,
/// WebM, headless print, and frame capture) and PNG export (`save_frame_as_png`).
///
/// `brightness` is the normalized 0..1 subpixel value. `mapping` applies the
/// intensity tone curve. `diff_norm` is the white-point-normalized signed
/// temporal difference (lever 3); `temporal_strength` 0.0 disables temporal
/// modulation, making this byte-identical to `map_brightness_rgb`.
/// `cycle` applies a [`PaletteCycle`] remap at pass-order step 5 (between the
/// tone curve and the gradient lookup). Pass [`PaletteCycle::default()`] for
/// identity behavior (byte-identical to the pre-cycle path).
#[allow(clippy::too_many_arguments)]
pub fn colorize_subpixel(
    brightness: f32,
    palette: Palette,
    reverse: bool,
    invert: bool,
    hue_shift: f32,
    mapping: Option<&IntensityMapping>,
    diff_norm: f32,
    temporal_strength: f32,
    temporal_mode: TemporalMode,
    cycle: PaletteCycle,
    temporal_accent: Option<RgbColor>,
) -> RgbColor {
    let base = map_brightness_rgb_cycled(
        brightness,
        palette.clone(),
        reverse,
        invert,
        hue_shift,
        mapping,
        cycle,
    );
    if temporal_strength <= 0.0 {
        return base;
    }
    temporal_modulate(
        base,
        diff_norm,
        temporal_mode,
        temporal_strength,
        palette,
        temporal_accent,
    )
}

/// Maximum hue rotation (degrees) at |blend| = 1, before strength scaling.
const TEMPORAL_MAX_HUE_DEG: f32 = 60.0;
/// tanh steepness for the white-point-normalized diff.
const TEMPORAL_TANH_K: f32 = 6.0;

fn temporal_modulate(
    base: RgbColor,
    diff_norm: f32,
    mode: TemporalMode,
    strength: f32,
    palette: Palette,
    accent: Option<RgbColor>,
) -> RgbColor {
    let blend = (TEMPORAL_TANH_K * diff_norm).tanh(); // -1..1; growing front ⇒ +, decaying ⇒ -
    if blend == 0.0 || strength <= 0.0 {
        return base;
    }
    match mode {
        TemporalMode::Hue => {
            let oklch = srgb_to_oklch(base);
            let h = oklch.h + blend * TEMPORAL_MAX_HUE_DEG * strength;
            oklch_to_srgb(oklch.l, oklch.c, h)
        }
        TemporalMode::Accent => {
            // Front accent = the palette's hot end (brightness 1.0). Blend toward it
            // for the growing front (blend > 0). Mix in OKLch for perceptual evenness.
            let accent =
                accent.unwrap_or_else(|| map_brightness_rgb(1.0, palette, false, false, 0.0, None));
            let t = blend.max(0.0) * strength;
            mix_oklch(base, accent, t)
        }
    }
}

/// Linear interpolation of two sRGB colors through OKLch (t in 0..1).
fn mix_oklch(a: RgbColor, b: RgbColor, t: f32) -> RgbColor {
    let t = t.clamp(0.0, 1.0);
    let oa = srgb_to_oklch(a);
    let ob = srgb_to_oklch(b);
    let l = oa.l + (ob.l - oa.l) * t;
    let c = oa.c + (ob.c - oa.c) * t;
    let h = oa.h + (ob.h - oa.h) * t;
    oklch_to_srgb(l, c, h)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn temporal_modulate_zero_diff_is_identity() {
        let base = RgbColor {
            r: 120,
            g: 200,
            b: 80,
        };
        let out = temporal_modulate(base, 0.0, TemporalMode::Hue, 1.0, Palette::Organic, None);
        assert_eq!(out, base);
    }

    #[test]
    fn temporal_modulate_hue_shifts_with_positive_diff() {
        let base = RgbColor {
            r: 120,
            g: 200,
            b: 80,
        };
        let out = temporal_modulate(base, 0.5, TemporalMode::Hue, 1.0, Palette::Organic, None);
        assert_ne!(out, base, "a growing front should change hue");
    }

    #[test]
    fn temporal_accent_some_overrides_hot_end() {
        let base = RgbColor::new(10, 80, 40);
        let custom = RgbColor::new(255, 180, 60);
        // Accent mode, strong diff: with a custom accent the result blends toward it.
        let with_custom = temporal_modulate(
            base,
            0.9,
            TemporalMode::Accent,
            1.0,
            Palette::Slime,
            Some(custom),
        );
        let hot_end = temporal_modulate(base, 0.9, TemporalMode::Accent, 1.0, Palette::Slime, None);
        assert_ne!(
            with_custom, hot_end,
            "custom accent must differ from hot-end"
        );
        // None path must equal the pre-change behavior (hot-end accent).
        let manual_accent = map_brightness_rgb(1.0, Palette::Slime, false, false, 0.0, None);
        let t = (TEMPORAL_TANH_K * 0.9_f32).tanh() * 1.0;
        let expect_none = mix_oklch(base, manual_accent, t);
        assert_eq!(hot_end, expect_none, "None must reproduce hot-end accent");
    }

    #[test]
    fn test_map_brightness_min() {
        assert_eq!(
            map_brightness(0.0, Palette::Organic, false, false, None),
            232
        );
        assert_eq!(map_brightness(0.0, Palette::Heat, false, false, None), 232);
        assert_eq!(map_brightness(0.0, Palette::Ocean, false, false, None), 232);
        assert_eq!(map_brightness(0.0, Palette::Mono, false, false, None), 232);
        assert_eq!(map_brightness(0.0, Palette::Forest, false, false, None), 22);
        assert_eq!(map_brightness(0.0, Palette::Neon, false, false, None), 17);
        assert_eq!(map_brightness(0.0, Palette::Warm, false, false, None), 52);
        assert_eq!(
            map_brightness(0.0, Palette::Vibrant, false, false, None),
            197
        );
        assert_eq!(
            map_brightness(0.0, Palette::LegibleMono, false, false, None),
            236
        );
    }

    #[test]
    fn test_map_brightness_max() {
        assert_eq!(
            map_brightness(1.0, Palette::Organic, false, false, None),
            226
        );
        assert_eq!(map_brightness(1.0, Palette::Heat, false, false, None), 226);
        assert_eq!(map_brightness(1.0, Palette::Ocean, false, false, None), 51);
        assert_eq!(map_brightness(1.0, Palette::Mono, false, false, None), 252);
        assert_eq!(map_brightness(1.0, Palette::Forest, false, false, None), 40);
        assert_eq!(map_brightness(1.0, Palette::Neon, false, false, None), 195);
        assert_eq!(map_brightness(1.0, Palette::Warm, false, false, None), 226);
        assert_eq!(
            map_brightness(1.0, Palette::Vibrant, false, false, None),
            231
        );
        assert_eq!(
            map_brightness(1.0, Palette::LegibleMono, false, false, None),
            255
        );
    }

    #[test]
    fn test_map_brightness_mid() {
        let color = map_brightness(0.5, Palette::Organic, false, false, None);
        assert_eq!(color, 46);

        let color = map_brightness(0.5, Palette::Heat, false, false, None);
        assert_eq!(color, 196);

        let color = map_brightness(0.5, Palette::Ocean, false, false, None);
        assert_eq!(color, 21);

        let color = map_brightness(0.5, Palette::Mono, false, false, None);
        assert_eq!(color, 242);

        let color = map_brightness(0.5, Palette::Forest, false, false, None);
        assert_eq!(color, 40);

        let color = map_brightness(0.5, Palette::Neon, false, false, None);
        assert_eq!(color, 123);

        let color = map_brightness(0.5, Palette::Warm, false, false, None);
        assert_eq!(color, 208);

        let color = map_brightness(0.5, Palette::Vibrant, false, false, None);
        assert_eq!(color, 121);

        let color = map_brightness(0.5, Palette::LegibleMono, false, false, None);
        assert_eq!(color, 251);
    }

    #[test]
    fn test_map_brightness_clamped() {
        assert_eq!(
            map_brightness(-0.5, Palette::Organic, false, false, None),
            232
        );
        assert_eq!(
            map_brightness(1.5, Palette::Organic, false, false, None),
            226
        );
        assert_eq!(
            map_brightness(-0.5, Palette::Forest, false, false, None),
            22
        );
        assert_eq!(map_brightness(1.5, Palette::Forest, false, false, None), 40);
    }

    #[test]
    fn test_map_brightness_quarter() {
        let color = map_brightness(0.25, Palette::Organic, false, false, None);
        assert_eq!(color, 34);

        let color = map_brightness(0.25, Palette::Heat, false, false, None);
        assert_eq!(color, 124);

        let color = map_brightness(0.25, Palette::Forest, false, false, None);
        assert_eq!(color, 34);

        let color = map_brightness(0.25, Palette::Neon, false, false, None);
        assert_eq!(color, 51);

        let color = map_brightness(0.25, Palette::Warm, false, false, None);
        assert_eq!(color, 166);
    }

    #[test]
    fn test_map_brightness_three_quarter() {
        let color = map_brightness(0.75, Palette::Organic, false, false, None);
        assert_eq!(color, 154);

        let color = map_brightness(0.75, Palette::Heat, false, false, None);
        assert_eq!(color, 214);

        let color = map_brightness(0.75, Palette::Forest, false, false, None);
        assert_eq!(color, 154);

        let color = map_brightness(0.75, Palette::Neon, false, false, None);
        assert_eq!(color, 201);

        let color = map_brightness(0.75, Palette::Warm, false, false, None);
        assert_eq!(color, 226);
    }

    #[test]
    fn test_reverse_palette() {
        assert_eq!(
            map_brightness(0.0, Palette::Organic, true, false, None),
            226
        );
        assert_eq!(
            map_brightness(1.0, Palette::Organic, true, false, None),
            232
        );
    }

    #[test]
    fn test_invert_palette() {
        let normal = map_brightness(0.5, Palette::Organic, false, false, None);
        let inverted = map_brightness(0.5, Palette::Organic, false, true, None);
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
        let reversed = map_brightness(0.0, Palette::Organic, true, false, None);
        let reversed_and_inverted = map_brightness(0.0, Palette::Organic, true, true, None);
        let inverted = invert_256_color(reversed);
        assert_eq!(reversed_and_inverted, inverted);
    }

    #[test]
    fn test_map_brightness_rgb_min() {
        let color = map_brightness_rgb(0.0, Palette::Organic, false, false, 0.0, None);
        assert_eq!(color.r, 18);
        assert_eq!(color.g, 18);
        assert_eq!(color.b, 18);
    }

    #[test]
    fn test_map_brightness_rgb_max() {
        let color = map_brightness_rgb(1.0, Palette::Organic, false, false, 0.0, None);
        assert_eq!(color.r, 150);
        assert_eq!(color.g, 220);
        assert_eq!(color.b, 200);
    }

    #[test]
    fn test_map_brightness_rgb_interpolation() {
        let color = map_brightness_rgb(0.5, Palette::Organic, false, false, 0.0, None);
        assert!(color.r >= 18 && color.r <= 160);
        assert!(color.g >= 18 && color.g <= 220);
        assert!(color.b >= 18 && color.b <= 200);

        let color = map_brightness_rgb(0.5, Palette::Ocean, false, false, 0.0, None);
        assert!(color.r >= 18 && color.r <= 80);
        assert!(color.g >= 18 && color.g <= 170);
        assert!(color.b >= 18 && color.b <= 240);
    }

    #[test]
    fn test_map_brightness_rgb_heat() {
        let min_color = map_brightness_rgb(0.0, Palette::Heat, false, false, 0.0, None);
        let max_color = map_brightness_rgb(1.0, Palette::Heat, false, false, 0.0, None);
        assert_eq!(min_color.r, 40);
        assert_eq!(min_color.g, 20);
        assert_eq!(min_color.b, 20);
        assert_eq!(max_color.r, 240);
        assert_eq!(max_color.g, 220);
        assert_eq!(max_color.b, 180);
    }

    #[test]
    fn test_map_brightness_rgb_ocean() {
        let min_color = map_brightness_rgb(0.0, Palette::Ocean, false, false, 0.0, None);
        let max_color = map_brightness_rgb(1.0, Palette::Ocean, false, false, 0.0, None);
        assert_eq!(min_color.r, 18);
        assert_eq!(min_color.g, 18);
        assert_eq!(min_color.b, 18);
        assert_eq!(max_color.r, 80);
        assert_eq!(max_color.g, 170);
        assert_eq!(max_color.b, 240);
    }

    #[test]
    fn test_map_brightness_rgb_forest() {
        let min_color = map_brightness_rgb(0.0, Palette::Forest, false, false, 0.0, None);
        let max_color = map_brightness_rgb(1.0, Palette::Forest, false, false, 0.0, None);
        assert_eq!(min_color.r, 20);
        assert_eq!(min_color.g, 40);
        assert_eq!(min_color.b, 20);
        assert_eq!(max_color.r, 180);
        assert_eq!(max_color.g, 220);
        assert_eq!(max_color.b, 170);
    }

    #[test]
    fn test_map_brightness_rgb_reverse() {
        let normal = map_brightness_rgb(0.0, Palette::Organic, false, false, 0.0, None);
        let reversed = map_brightness_rgb(1.0, Palette::Organic, true, false, 0.0, None);
        assert_eq!(normal.r, reversed.r);
        assert_eq!(normal.g, reversed.g);
        assert_eq!(normal.b, reversed.b);
    }

    #[test]
    fn test_map_brightness_rgb_invert() {
        let normal = map_brightness_rgb(0.5, Palette::Organic, false, false, 0.0, None);
        let inverted = map_brightness_rgb(0.5, Palette::Organic, false, true, 0.0, None);
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
            let _color = map_brightness_rgb(0.5, palette, false, false, 0.0, None);
        }
    }

    #[test]
    fn test_map_brightness_rgb_clamped() {
        // Out-of-range inputs clamp to [0, 1] endpoints
        let min = map_brightness_rgb(-0.5, Palette::Heat, false, false, 0.0, None);
        let max = map_brightness_rgb(1.5, Palette::Heat, false, false, 0.0, None);
        let normal = map_brightness_rgb(0.5, Palette::Heat, false, false, 0.0, None);
        // Heat t=0: r=40; Heat t=1: r=240
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
        let min_color = map_brightness(0.0, Palette::Slime, false, false, None);
        let max_color = map_brightness(1.0, Palette::Slime, false, false, None);
        assert_eq!(min_color, 22);
        assert_eq!(max_color, 231);
    }

    #[test]
    fn test_mold_palette_gradient() {
        let min_color = map_brightness(0.0, Palette::Mold, false, false, None);
        let max_color = map_brightness(1.0, Palette::Mold, false, false, None);
        assert_eq!(min_color, 236);
        assert_eq!(max_color, 193);
    }

    #[test]
    fn test_fungus_palette_gradient() {
        let min_color = map_brightness(0.0, Palette::Fungus, false, false, None);
        let max_color = map_brightness(1.0, Palette::Fungus, false, false, None);
        assert_eq!(min_color, 232);
        assert_eq!(max_color, 223);
    }

    #[test]
    fn test_swamp_palette_gradient() {
        let min_color = map_brightness(0.0, Palette::Swamp, false, false, None);
        let max_color = map_brightness(1.0, Palette::Swamp, false, false, None);
        assert_eq!(min_color, 232);
        assert_eq!(max_color, 79);
    }

    #[test]
    fn test_moss_palette_gradient() {
        let min_color = map_brightness(0.0, Palette::Moss, false, false, None);
        let max_color = map_brightness(1.0, Palette::Moss, false, false, None);
        assert_eq!(min_color, 22);
        assert_eq!(max_color, 220);
    }

    #[test]
    fn test_slime_palette_rgb_values() {
        let color = map_brightness_rgb(0.5, Palette::Slime, false, false, 0.0, None);
        assert!(color.g > color.r && color.g > color.b);
    }

    #[test]
    fn test_fungus_palette_has_purple_tones() {
        let color = map_brightness_rgb(0.3, Palette::Fungus, false, false, 0.0, None);
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
            let _color = map_brightness_rgb(0.5, Palette::Slime, false, false, 0.0, None);
        }
    }

    #[test]
    fn test_moss_palette_has_green_tones() {
        let color = map_brightness_rgb(0.5, Palette::Moss, false, false, 0.0, None);
        assert!(color.g > color.r && color.g > color.b);
    }

    #[test]
    fn test_map_brightness_rgb_hue_shift_with_invert() {
        let _color_shifted = map_brightness_rgb(0.5, Palette::Organic, false, true, 90.0, None);
    }

    #[test]
    fn test_map_brightness_rgb_hue_shift_with_reverse() {
        let _color_shifted = map_brightness_rgb(0.5, Palette::Organic, true, false, 90.0, None);
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
        let color1 = map_brightness_rgb(0.45, Palette::Heat, false, false, 0.0, None);
        let color2 = map_brightness_rgb(0.50, Palette::Heat, false, false, 0.0, None);
        let color3 = map_brightness_rgb(0.55, Palette::Heat, false, false, 0.0, None);

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

    // =========================================================================
    // Intensity Mapping Tests
    // =========================================================================

    #[test]
    fn test_mapping_function_linear() {
        let f = MappingFunction::Linear;
        assert!((f.apply(0.0) - 0.0).abs() < f32::EPSILON);
        assert!((f.apply(0.5) - 0.5).abs() < f32::EPSILON);
        assert!((f.apply(1.0) - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_mapping_function_logarithmic() {
        let f = MappingFunction::Logarithmic { base: 10.0 };
        // f(0) = 0, f(1) = 1
        assert!((f.apply(0.0) - 0.0).abs() < 0.001);
        assert!((f.apply(1.0) - 1.0).abs() < 0.001);
        // Log should compress high values: f(0.5) > 0.5
        assert!(f.apply(0.5) > 0.5);
    }

    #[test]
    fn test_mapping_function_exponential() {
        let f = MappingFunction::Exponential { base: 10.0 };
        // f(0) = 0, f(1) = 1
        assert!((f.apply(0.0) - 0.0).abs() < 0.001);
        assert!((f.apply(1.0) - 1.0).abs() < 0.001);
        // Exp should expand high values: f(0.5) < 0.5
        assert!(f.apply(0.5) < 0.5);
    }

    #[test]
    fn test_mapping_function_power() {
        let f_dark = MappingFunction::Power { gamma: 0.5 };
        let f_bright = MappingFunction::Power { gamma: 2.0 };

        // Both should preserve endpoints
        assert!((f_dark.apply(0.0) - 0.0).abs() < 0.001);
        assert!((f_dark.apply(1.0) - 1.0).abs() < 0.001);
        assert!((f_bright.apply(0.0) - 0.0).abs() < 0.001);
        assert!((f_bright.apply(1.0) - 1.0).abs() < 0.001);

        // gamma < 1 expands darks: f(0.5) > 0.5
        assert!(f_dark.apply(0.5) > 0.5);
        // gamma > 1 compresses darks: f(0.5) < 0.5
        assert!(f_bright.apply(0.5) < 0.5);
    }

    #[test]
    fn test_mapping_function_smoothstep() {
        let f = MappingFunction::Smoothstep;
        // f(0) = 0, f(1) = 1
        assert!((f.apply(0.0) - 0.0).abs() < 0.001);
        assert!((f.apply(1.0) - 1.0).abs() < 0.001);
        // f(0.5) = 0.5 for smoothstep
        assert!((f.apply(0.5) - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_mapping_function_quantize() {
        let f = MappingFunction::Quantize { levels: 4 };
        // Should produce discrete steps
        assert!((f.apply(0.0) - 0.0).abs() < 0.001);
        assert!((f.apply(0.33) - 0.333).abs() < 0.1);
        assert!((f.apply(1.0) - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_mapping_function_perlin() {
        let f = MappingFunction::Perlin {
            amplitude: 0.2,
            frequency: 5.0,
            seed: 42,
        };
        // Endpoints should be close to 0 and 1 due to anchoring
        assert!(f.apply(0.0).abs() < 0.01);
        assert!((f.apply(1.0) - 1.0).abs() < 0.01);
        // Middle values should be distorted but bounded
        let mid = f.apply(0.5);
        assert!((0.0..=1.0).contains(&mid));
    }

    #[test]
    fn test_intensity_mapping_linear() {
        let mapping = IntensityMapping::linear();
        assert!((mapping.apply(0.0) - 0.0).abs() < f32::EPSILON);
        assert!((mapping.apply(0.5) - 0.5).abs() < f32::EPSILON);
        assert!((mapping.apply(1.0) - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_intensity_mapping_linear_log_split() {
        let mapping = IntensityMapping::linear_log_split(10.0);

        // Endpoints preserved
        assert!((mapping.apply(0.0) - 0.0).abs() < 0.001);
        assert!((mapping.apply(1.0) - 1.0).abs() < 0.001);

        // Split point preserved (6/11 ≈ 0.545)
        let split = 6.0 / 11.0;
        assert!((mapping.apply(split) - split).abs() < 0.001);
    }

    #[test]
    fn test_intensity_mapping_validation_empty() {
        let result = IntensityMapping::new(vec![]);
        assert!(matches!(result, Err(MappingError::NoSegments)));
    }

    #[test]
    fn test_intensity_mapping_validation_gap() {
        let result = IntensityMapping::new(vec![
            MappingSegment {
                start: 0.0,
                end: 0.4,
                function: MappingFunction::Linear,
            },
            MappingSegment {
                start: 0.6, // Gap!
                end: 1.0,
                function: MappingFunction::Linear,
            },
        ]);
        assert!(matches!(result, Err(MappingError::SegmentGap { .. })));
    }

    #[test]
    fn test_intensity_mapping_validation_not_starting_at_zero() {
        let result = IntensityMapping::new(vec![MappingSegment {
            start: 0.1,
            end: 1.0,
            function: MappingFunction::Linear,
        }]);
        assert!(matches!(result, Err(MappingError::DomainNotCovered { .. })));
    }

    #[test]
    fn test_intensity_mapping_validation_not_ending_at_one() {
        let result = IntensityMapping::new(vec![MappingSegment {
            start: 0.0,
            end: 0.9,
            function: MappingFunction::Linear,
        }]);
        assert!(matches!(result, Err(MappingError::DomainNotCovered { .. })));
    }

    #[test]
    fn test_map_brightness_with_logarithmic_mapping() {
        let mapping = IntensityMapping::logarithmic(10.0);

        // With log mapping, f(0.5) > 0.5, so should get a brighter color
        let without_mapping = map_brightness(0.5, Palette::Organic, false, false, None);
        let with_mapping = map_brightness(0.5, Palette::Organic, false, false, Some(&mapping));

        // These should be different
        assert_ne!(without_mapping, with_mapping);
    }

    #[test]
    fn test_map_brightness_rgb_with_linear_log_split() {
        let mapping = IntensityMapping::linear_log_split(10.0);

        // Color at 0.9 should be compressed (darker) with log mapping
        let without_mapping = map_brightness_rgb(0.9, Palette::Heat, false, false, 0.0, None);
        let with_mapping =
            map_brightness_rgb(0.9, Palette::Heat, false, false, 0.0, Some(&mapping));

        // These should be different
        assert!(
            without_mapping.r != with_mapping.r
                || without_mapping.g != with_mapping.g
                || without_mapping.b != with_mapping.b
        );
    }

    // =========================================================================
    // OKLch NaN Hue Handling Tests (CSS Color Module Level 4 compliance)
    // =========================================================================

    #[test]
    fn test_oklch_grayscale_returns_nan_hue() {
        // Grayscale colors should return NaN for hue (powerless component)
        let gray_colors = [
            RgbColor { r: 0, g: 0, b: 0 }, // Black
            RgbColor {
                r: 128,
                g: 128,
                b: 128,
            }, // Mid gray
            RgbColor {
                r: 255,
                g: 255,
                b: 255,
            }, // White
        ];

        for color in gray_colors {
            let oklch = srgb_to_oklch(color);
            assert!(
                oklch.c < OKLCH_EPSILON || oklch.h.is_nan(),
                "Grayscale color {:?} should have chroma < epsilon or NaN hue, got c={}, h={}",
                color,
                oklch.c,
                oklch.h
            );
        }
    }

    #[test]
    fn test_oklch_color_returns_valid_hue() {
        // Non-grayscale colors should return valid hue
        let color = RgbColor { r: 255, g: 0, b: 0 }; // Pure red
        let oklch = srgb_to_oklch(color);

        assert!(
            oklch.c >= OKLCH_EPSILON,
            "Red should have significant chroma, got {}",
            oklch.c
        );
        assert!(
            oklch.h.is_finite(),
            "Red should have finite hue, got {}",
            oklch.h
        );
        // OKLch red hue is approximately 30° (not 0° like HSL)
        assert!(
            oklch.h > 20.0 && oklch.h < 40.0,
            "Red hue should be around 30° in OKLch, got {}",
            oklch.h
        );
    }

    #[test]
    fn test_oklch_to_srgb_handles_nan_hue() {
        // Test that oklch_to_srgb handles NaN hue gracefully
        let gray = oklch_to_srgb(0.5, 0.0, f32::NAN);

        // Should produce grayscale (R=G=B)
        assert!(
            gray.r.abs_diff(gray.g) <= 1 && gray.g.abs_diff(gray.b) <= 1,
            "NaN hue with zero chroma should produce grayscale, got R={}, G={}, B={}",
            gray.r,
            gray.g,
            gray.b
        );
    }

    #[test]
    fn test_oklch_roundtrip_preserves_color() {
        // Test that RGB -> OKLch -> RGB roundtrip works correctly
        let original = RgbColor {
            r: 100,
            g: 150,
            b: 80,
        };

        let oklch = srgb_to_oklch(original);
        let restored = oklch_to_rgb(oklch);

        // Due to 8-bit quantization, allow small differences
        let tolerance = 2;
        assert!(
            original.r.abs_diff(restored.r) <= tolerance
                && original.g.abs_diff(restored.g) <= tolerance
                && original.b.abs_diff(restored.b) <= tolerance,
            "OKLch roundtrip failed: {:?} -> {:?}",
            original,
            restored
        );
    }

    #[test]
    fn test_oklch_epsilon_boundary() {
        // Test that colors very close to grayscale are handled correctly
        let near_gray = RgbColor {
            r: 100,
            g: 101,
            b: 100,
        };
        let oklch = srgb_to_oklch(near_gray);

        // Should detect near-zero chroma
        if oklch.c < OKLCH_EPSILON {
            assert!(oklch.h.is_nan(), "When chroma < epsilon, hue should be NaN");
        }
    }

    #[test]
    fn colorize_subpixel_matches_map_brightness_rgb_when_no_temporal() {
        let p = Palette::Organic;
        let expected = map_brightness_rgb(0.6, p.clone(), false, false, 0.0, None);
        // strength 0.0 and identity cycle ⇒ identical to the legacy mapping
        let got = colorize_subpixel(
            0.6,
            p,
            false,
            false,
            0.0,
            None,
            0.0,
            0.0,
            TemporalMode::Hue,
            PaletteCycle::default(),
            None,
        );
        assert_eq!(got, expected);
    }

    #[test]
    fn palette_cycle_identity_is_noop() {
        let id = PaletteCycle::default();
        assert!(id.is_identity());
        for &t in &[0.0_f32, 0.25, 0.5, 0.75, 1.0] {
            assert_eq!(id.map(t), t, "n=1 must be identity at t={t}");
        }
        // cycles=0 clamps to identity (defensive)
        let zero = PaletteCycle {
            cycles: 0,
            mode: PaletteCycleMode::Wrap,
        };
        assert!(zero.is_identity());
        assert_eq!(zero.map(0.4), 0.4);
    }

    #[test]
    fn palette_cycle_wrap_is_sawtooth() {
        let c = PaletteCycle {
            cycles: 2,
            mode: PaletteCycleMode::Wrap,
        };
        assert!((c.map(0.0) - 0.0).abs() < 1e-6);
        assert!((c.map(0.25) - 0.5).abs() < 1e-6);
        assert!((c.map(0.5) - 0.0).abs() < 1e-6); // seam: fract(1.0) = 0
        assert!((c.map(0.75) - 0.5).abs() < 1e-6);
    }

    #[test]
    fn palette_cycle_mirror_is_triangle() {
        let c = PaletteCycle {
            cycles: 2,
            mode: PaletteCycleMode::Mirror,
        };
        assert!((c.map(0.0) - 0.0).abs() < 1e-6);
        assert!((c.map(0.25) - 1.0).abs() < 1e-6); // peak at quarter
        assert!((c.map(0.5) - 0.0).abs() < 1e-6); // trough at half
        assert!((c.map(0.75) - 1.0).abs() < 1e-6);
        // output stays in 0..1 across a sweep
        for i in 0..=100 {
            let t = i as f32 / 100.0;
            let o = c.map(t);
            assert!(
                (0.0..=1.0).contains(&o),
                "mirror out of range at t={t}: {o}"
            );
        }
    }

    #[test]
    fn palette_cycle_mode_from_str() {
        use std::str::FromStr;
        assert_eq!(
            PaletteCycleMode::from_str("wrap").unwrap(),
            PaletteCycleMode::Wrap
        );
        assert_eq!(
            PaletteCycleMode::from_str("MIRROR").unwrap(),
            PaletteCycleMode::Mirror
        );
        assert!(PaletteCycleMode::from_str("bogus").is_err());
    }

    #[test]
    fn cycled_core_identity_matches_legacy() {
        // cycles=1 must be byte-identical to the legacy public fns (back-compat).
        let id = PaletteCycle::default();
        for &b in &[0.0_f32, 0.3, 0.6, 1.0] {
            assert_eq!(
                map_brightness_rgb_cycled(b, Palette::Organic, false, false, 0.0, None, id),
                map_brightness_rgb(b, Palette::Organic, false, false, 0.0, None),
            );
            assert_eq!(
                map_brightness_cycled(b, Palette::Organic, false, false, None, id),
                map_brightness(b, Palette::Organic, false, false, None),
            );
        }
    }

    #[test]
    fn cycled_core_changes_color_when_active() {
        // A mid brightness under cycles=2 mirror maps to a different gradient index,
        // so the resulting color must differ from the identity lookup.
        let active = PaletteCycle {
            cycles: 2,
            mode: PaletteCycleMode::Mirror,
        };
        let id = PaletteCycle::default();
        let on = map_brightness_rgb_cycled(0.25, Palette::Organic, false, false, 0.0, None, active);
        let off = map_brightness_rgb_cycled(0.25, Palette::Organic, false, false, 0.0, None, id);
        assert_ne!(on, off, "cycles=2 must remap the gradient index");
    }

    #[test]
    fn colorize_subpixel_identity_cycle_matches_map_brightness_rgb() {
        let id = PaletteCycle::default();
        let got = colorize_subpixel(
            0.6,
            Palette::Organic,
            false,
            false,
            0.0,
            None,
            0.0,
            0.0,
            TemporalMode::Hue,
            id,
            None,
        );
        let want = map_brightness_rgb(0.6, Palette::Organic, false, false, 0.0, None);
        assert_eq!(got, want);
    }
}
