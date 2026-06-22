//! Simulation configuration types and presets.
//!
//! This module defines all the configuration parameters for the Physarum simulation,
//! including presets, diffusion kernels, initialization modes, and environmental effects.

use image::io::Reader as ImageReader;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::path::Path;

use super::agent::normalize_angle;
use crate::config_defaults::{
    agent as agent_consts, environment, environment as env_consts, food as food_img_consts,
    population, population as pop_consts, time as time_consts, trail as trail_consts,
};
use crate::render::palette::RgbColor;

/// Algorithm used for pheromone diffusion (spreading).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DiffusionKernel {
    /// Simple 3×3 box blur averaging. Fast with sharp patterns.
    Mean3x3,
    /// 5×5 Gaussian blur. Slower but produces smoother, more organic patterns.
    Gaussian,
}

/// Nonlinear curve applied to per-frame accumulated deposit before folding
/// into the trail. `Linear` (with scale 1, cap 0) is byte-identical to the
/// historical per-agent deposit.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DepositCurve {
    /// Identity: `x`. Default; preserves historical behavior.
    #[default]
    Linear,
    /// `sqrt(x)` — compresses density spikes into filaments.
    Sqrt,
    /// `ln(1 + x)` — log compression (0 at 0, guards log(0)).
    Log,
    /// `x^gamma` — `deposit_gamma` is the exponent (γ<1 compresses, γ>1 expands).
    Pow,
}

impl DepositCurve {
    /// Apply the curve to a (non-negative) accumulated deposit value.
    /// `gamma` is used only by `Pow`.
    #[inline]
    pub fn apply(self, x: f32, gamma: f32) -> f32 {
        match self {
            DepositCurve::Linear => x,
            DepositCurve::Sqrt => x.sqrt(),
            DepositCurve::Log => (1.0 + x).ln(),
            DepositCurve::Pow => x.powf(gamma),
        }
    }
}

/// Named parameter presets for different visual styles.
///
/// Each preset combines multiple parameters optimized for a specific aesthetic.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Preset {
    /// Dense, interconnected networks with rapid branching.
    Network,
    /// Wide, searching tentacles with exploratory behavior.
    Exploratory,
    /// Long branching arms stretching across the terminal.
    Tendrils,
    /// Balanced, natural-looking growth (default).
    Organic,
    /// Aggressive, fast-moving flame-like patterns.
    Fire,
    /// Flowing, water-like patterns.
    River,
    /// Petri dish simulation: starts center, slow growth, persistent trails.
    #[serde(rename = "petridish")]
    PetriDish,
    /// Spinning vortex patterns (rotation_angle > sensor_angle).
    Vortex,
    /// Fast dendritic branching like lightning.
    Lightning,
    /// Edge-of-chaos sensitive patterns (sensor_angle ≈ rotation_angle).
    #[serde(rename = "chaosedge")]
    ChaosEdge,
    /// Aggregating blob clusters.
    Blob,
    /// Slime-mold surface tension with trail-based flow modulation.
    Slime,
    /// Creeping vine tendrils with trail-modulated cohesion.
    Vines,
    /// ASCII-rendered cohesive flocking (vines pattern).
    Vinescii,
    /// Drifting smoke columns with wrapping boundary.
    Smoke,
    /// Enhanced vortex with trail modulation.
    #[serde(rename = "vortex36")]
    Vortex36,
    /// Dynamic tendrils with trail-based sensor modulation.
    #[serde(rename = "dynamictendrils")]
    DynamicTendrils,
    /// Bleuje-style front-lit veins: temporal-accent recolor of growing fronts.
    Mold,
    /// Directional filament linework via Sobel glyph selection (Braille, TUI-only).
    Etching,
    /// Color that shifts with motion direction (temporal Hue mode).
    Drift,
    /// Sparse star-map scatter (Points charset).
    Constellation,
    /// Constellation that holds its figure crisp via continuous template re-stamp.
    ConstellationStatic,
    /// Posterized color bands (Quantize mapping + Wrap palette cycles).
    Mosaic,
    /// Veined stone via heavy Gaussian + Perlin intensity mapping.
    Marble,
    /// Maximum color resolution (HalfBlockDual + SquareRoot mapping).
    Prism,
    /// Soft parchment density (Shade charset + Log deposit curve).
    Vellum,
    /// Grainy molten thermal (Exponential mapping + afterglow).
    Forge,
    /// Slow ghosting decay via low decay-gamma + Pow deposit curve.
    Wane,
    /// Delicate threads (Braille + brightness glyphs + Power mapping).
    Gossamer,
    /// Typographic engraving (custom ASCII + Sigmoid mapping).
    Codex,
    /// Living water with animated hue-shift over time.
    Tide,
}

/// Static identity of one preset: the enum variant, its display name, extra
/// parse aliases, and an optional number-row quick-select key.
///
/// [`PRESETS`] is the single source of truth for preset *identity*: CLI parsing,
/// display names, the live quick-select keys, and the validation test all derive
/// from it. Per-preset simulation parameters live in [`Preset::apply`]; per-preset
/// render defaults live in `RenderArtDefaults`. This table deliberately does not
/// duplicate either payload — only identity.
pub struct PresetSpec {
    /// The preset this entry describes.
    pub preset: Preset,
    /// Display name; also the canonical case-insensitive CLI parse key.
    pub name: &'static str,
    /// Additional case-insensitive names accepted on the CLI (hyphen/underscore
    /// variants, short forms).
    pub aliases: &'static [&'static str],
    /// Number-row key (`1`–`7`) that live-switches to this preset, if any. The
    /// shifted form selects it for A/B comparison.
    pub quick_key: Option<char>,
}

/// Identity table for every preset — the single list to edit when adding or
/// removing one (alongside the [`Preset`] variant and its [`Preset::apply`] arm).
pub const PRESETS: &[PresetSpec] = &[
    PresetSpec {
        preset: Preset::Network,
        name: "Network",
        aliases: &[],
        quick_key: None,
    },
    PresetSpec {
        preset: Preset::Exploratory,
        name: "Exploratory",
        aliases: &[],
        quick_key: None,
    },
    PresetSpec {
        preset: Preset::Tendrils,
        name: "Tendrils",
        aliases: &[],
        quick_key: None,
    },
    PresetSpec {
        preset: Preset::Organic,
        name: "Organic",
        aliases: &[],
        quick_key: Some('1'),
    },
    PresetSpec {
        preset: Preset::Fire,
        name: "Fire",
        aliases: &[],
        quick_key: None,
    },
    PresetSpec {
        preset: Preset::River,
        name: "River",
        aliases: &[],
        quick_key: None,
    },
    PresetSpec {
        preset: Preset::PetriDish,
        name: "PetriDish",
        aliases: &["petri"],
        quick_key: None,
    },
    PresetSpec {
        preset: Preset::Vortex,
        name: "Vortex",
        aliases: &[],
        quick_key: None,
    },
    PresetSpec {
        preset: Preset::Lightning,
        name: "Lightning",
        aliases: &[],
        quick_key: None,
    },
    PresetSpec {
        preset: Preset::ChaosEdge,
        name: "ChaosEdge",
        aliases: &["chaos-edge", "chaos_edge"],
        quick_key: None,
    },
    PresetSpec {
        preset: Preset::Blob,
        name: "Blob",
        aliases: &[],
        quick_key: None,
    },
    PresetSpec {
        preset: Preset::Slime,
        name: "Slime",
        aliases: &["pulse"],
        quick_key: None,
    },
    PresetSpec {
        preset: Preset::Vines,
        name: "Vines",
        aliases: &["flocking"],
        quick_key: None,
    },
    PresetSpec {
        preset: Preset::Vinescii,
        name: "Vinescii",
        aliases: &["vines-ascii"],
        quick_key: Some('3'),
    },
    PresetSpec {
        preset: Preset::Smoke,
        name: "Smoke",
        aliases: &["ripple"],
        quick_key: None,
    },
    PresetSpec {
        preset: Preset::Vortex36,
        name: "Vortex36",
        aliases: &["vortex-36", "vortex_36"],
        quick_key: None,
    },
    PresetSpec {
        preset: Preset::DynamicTendrils,
        name: "DynamicTendrils",
        aliases: &["dynamic-tendrils", "dynamic_tendrils"],
        quick_key: None,
    },
    PresetSpec {
        preset: Preset::Mold,
        name: "Mold",
        aliases: &["lumen"],
        quick_key: None,
    },
    PresetSpec {
        preset: Preset::Etching,
        name: "Etching",
        aliases: &[],
        quick_key: None,
    },
    PresetSpec {
        preset: Preset::Drift,
        name: "Drift",
        aliases: &[],
        quick_key: None,
    },
    PresetSpec {
        preset: Preset::Constellation,
        name: "Constellation",
        aliases: &[],
        quick_key: Some('2'),
    },
    PresetSpec {
        preset: Preset::ConstellationStatic,
        name: "ConstellationStatic",
        aliases: &["conststatic", "atlas"],
        quick_key: None,
    },
    PresetSpec {
        preset: Preset::Mosaic,
        name: "Mosaic",
        aliases: &[],
        quick_key: None,
    },
    PresetSpec {
        preset: Preset::Marble,
        name: "Marble",
        aliases: &[],
        quick_key: None,
    },
    PresetSpec {
        preset: Preset::Prism,
        name: "Prism",
        aliases: &[],
        quick_key: None,
    },
    PresetSpec {
        preset: Preset::Vellum,
        name: "Vellum",
        aliases: &[],
        quick_key: None,
    },
    PresetSpec {
        preset: Preset::Forge,
        name: "Forge",
        aliases: &[],
        quick_key: None,
    },
    PresetSpec {
        preset: Preset::Wane,
        name: "Wane",
        aliases: &[],
        quick_key: None,
    },
    PresetSpec {
        preset: Preset::Gossamer,
        name: "Gossamer",
        aliases: &[],
        quick_key: None,
    },
    PresetSpec {
        preset: Preset::Codex,
        name: "Codex",
        aliases: &[],
        quick_key: None,
    },
    PresetSpec {
        preset: Preset::Tide,
        name: "Tide",
        aliases: &[],
        quick_key: None,
    },
];

/// Looks up a preset by display name or alias (case-insensitive).
#[must_use]
pub fn preset_from_name(name: &str) -> Option<Preset> {
    PRESETS
        .iter()
        .find(|spec| {
            spec.name.eq_ignore_ascii_case(name)
                || spec.aliases.iter().any(|a| a.eq_ignore_ascii_case(name))
        })
        .map(|spec| spec.preset)
}

/// Comma-separated list of canonical preset names, for CLI error messages.
#[must_use]
pub fn preset_name_list() -> String {
    PRESETS
        .iter()
        .map(|spec| spec.name.to_lowercase())
        .collect::<Vec<_>>()
        .join(", ")
}

/// Preset bound to a number-row key (`1`–`7`) for live switching, if any.
#[must_use]
pub fn preset_for_set_key(key: char) -> Option<Preset> {
    PRESETS
        .iter()
        .find(|spec| spec.quick_key == Some(key))
        .map(|spec| spec.preset)
}

/// Preset bound to a shifted number key (`!@#$%^&`) for A/B comparison, if any.
#[must_use]
pub fn preset_for_compare_key(key: char) -> Option<Preset> {
    shifted_digit(key).and_then(preset_for_set_key)
}

/// Public mapping from a shifted number key (`!@#$%^&`) to its base digit (`1`-`7`).
#[must_use]
pub fn compare_key_digit(key: char) -> Option<char> {
    shifted_digit(key)
}

/// Maps a shifted number key to its base digit (`!`→`1` … `&`→`7`).
fn shifted_digit(key: char) -> Option<char> {
    match key {
        '!' => Some('1'),
        '@' => Some('2'),
        '#' => Some('3'),
        '$' => Some('4'),
        '%' => Some('5'),
        '^' => Some('6'),
        '&' => Some('7'),
        _ => None,
    }
}

impl Preset {
    /// Display name of this preset (from [`PRESETS`]).
    #[must_use]
    pub fn name(&self) -> &'static str {
        PRESETS
            .iter()
            .find(|spec| spec.preset == *self)
            .map_or("Unknown", |spec| spec.name)
    }
}

/// How agents are initially distributed in the simulation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum InitMode {
    /// Agents randomly distributed across the entire canvas.
    Random,
    /// Agents start from the center and burst outward.
    CentralBurst,
    /// Agents arranged in a circle.
    Circle,
    /// Agents distributed in a gradient pattern.
    Gradient,
    /// Agents start as a wave front.
    WaveFront,
    /// Agents arranged in a spiral pattern.
    Spiral,
    /// Agents in random clusters.
    RandomClusters,
    /// Agents distributed based on a loaded image (food source).
    Food,
    /// Agents distributed in a Gaussian blob at the center (Petri dish style).
    Petri,
    /// Agents seeded as a real star constellation (stars + asterism edges).
    Constellation,
}

impl InitMode {
    /// Uniformly pick any init mode. Used by presets (e.g. Constellation) that
    /// re-roll their starting layout on each reset.
    ///
    /// `ALL` is hand-maintained; the exhaustive match below is a compile-time
    /// guard — adding an `InitMode` variant fails to compile here until it is
    /// also added to `ALL`, so the picker can never silently exclude a variant.
    pub fn random(rng: &mut impl rand::Rng) -> Self {
        use InitMode::*;
        const ALL: [InitMode; 9] = [
            Random,
            CentralBurst,
            Circle,
            Gradient,
            WaveFront,
            Spiral,
            RandomClusters,
            Food,
            Petri,
        ];
        // Exhaustiveness guard: keep ALL in sync with the enum. Adding a variant
        // breaks this match until it is also added to ALL above.
        #[allow(dead_code)]
        const fn _guard(m: InitMode) {
            match m {
                InitMode::Random
                | InitMode::CentralBurst
                | InitMode::Circle
                | InitMode::Gradient
                | InitMode::WaveFront
                | InitMode::Spiral
                | InitMode::RandomClusters
                | InitMode::Food
                | InitMode::Petri
                | InitMode::Constellation => {}
            }
        }
        ALL[rng.gen_range(0..ALL.len())]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
/// Types of terrain-based steering bias.
pub enum TerrainType {
    /// No terrain effect.
    #[default]
    None,
    /// Smooth, flowing patterns based on Perlin noise.
    Smooth,
    /// Chaotic, turbulent patterns.
    Turbulent,
    /// Combination of smooth and turbulent layers.
    Mixed,
}

impl std::str::FromStr for TerrainType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "none" | "off" | "disabled" => Ok(TerrainType::None),
            "smooth" => Ok(TerrainType::Smooth),
            "turbulent" => Ok(TerrainType::Turbulent),
            "mixed" => Ok(TerrainType::Mixed),
            _ => Err(format!(
                "Invalid terrain type: {}. Must be one of: none, smooth, turbulent, mixed",
                s
            )),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
/// Global wind force configuration.
pub struct Wind {
    /// Horizontal wind strength (-1.0 to 1.0).
    pub dx: f32,
    /// Vertical wind strength (-1.0 to 1.0).
    pub dy: f32,
}

impl Wind {
    /// Creates a new wind vector.
    pub fn new(dx: f32, dy: f32) -> Self {
        Self { dx, dy }
    }
}

impl Default for Wind {
    fn default() -> Self {
        Self { dx: 0.0, dy: 0.0 }
    }
}

impl Validatable for Wind {
    fn validate(&self) -> Result<(), ValidationError> {
        if self.dx < -1.0 || self.dx > 1.0 {
            return Err(ValidationError::out_of_range("wind.dx", -1.0, 1.0, self.dx));
        }
        if self.dy < -1.0 || self.dy > 1.0 {
            return Err(ValidationError::out_of_range("wind.dy", -1.0, 1.0, self.dy));
        }
        if self.dx.abs() < 0.001 && self.dy.abs() < 0.001 {
            return Err(ValidationError::custom("wind cannot be zero vector"));
        }
        Ok(())
    }
}

impl std::str::FromStr for Wind {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use crate::validation::Validatable;

        let parts: Vec<&str> = s.split(',').collect();
        if parts.len() != 2 {
            return Err(format!("Wind must be in dx,dy format, got: {}", s));
        }

        let dx = parts[0]
            .parse::<f32>()
            .map_err(|e| format!("Invalid dx: {}", e))?;
        let dy = parts[1]
            .parse::<f32>()
            .map_err(|e| format!("Invalid dy: {}", e))?;

        let wind = Wind::new(dx, dy);
        Validatable::validate(&wind).map_err(|e| e.to_string())?;
        Ok(wind)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
/// A point attractor or repeller.
pub struct Attractor {
    /// X coordinate.
    pub x: f32,
    /// Y coordinate.
    pub y: f32,
    /// Strength of attraction (negative for repulsion).
    pub strength: f32,
}

impl Attractor {
    /// Creates a new attractor.
    pub fn new(x: f32, y: f32, strength: f32) -> Self {
        Self { x, y, strength }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
/// A temporary attractor created by mouse interaction.
pub struct MouseAttractor {
    /// X coordinate.
    pub x: f32,
    /// Y coordinate.
    pub y: f32,
    /// Strength of attraction/repulsion.
    pub strength: f32,
    /// Time of creation.
    pub created_at: std::time::Instant,
    /// Duration in seconds before expiration.
    pub timeout_seconds: f32,
}

impl MouseAttractor {
    /// Creates a new mouse attractor.
    pub fn new(x: f32, y: f32, strength: f32, timeout_seconds: f32) -> Self {
        Self {
            x,
            y,
            strength,
            created_at: std::time::Instant::now(),
            timeout_seconds,
        }
    }

    /// Checks if the attractor has expired.
    pub fn is_expired(&self) -> bool {
        self.created_at.elapsed().as_secs_f32() >= self.timeout_seconds
    }
}

#[derive(Debug, Clone, PartialEq)]
/// Mask data for image-based obstacles.
pub struct ObstacleMask {
    /// Flattened pixel data (normalized brightness).
    pub pixels: Vec<f32>,
    /// Width of the mask.
    pub width: usize,
    /// Height of the mask.
    pub height: usize,
}

impl ObstacleMask {
    /// Creates a mask from an image file.
    ///
    /// Resizes the image to target dimensions.
    pub fn from_image(
        image_path: &str,
        target_width: usize,
        target_height: usize,
        invert: bool,
    ) -> Result<Self, String> {
        let path = Path::new(image_path);

        if !path.exists() {
            return Err(format!("Image file not found: {}", image_path));
        }

        let img = ImageReader::open(path)
            .map_err(|e| format!("Failed to open image: {}", e))?
            .decode()
            .map_err(|e| format!("Failed to decode image: {}", e))?;

        let resized = img.resize_exact(
            target_width as u32,
            target_height as u32,
            image::imageops::FilterType::Nearest,
        );

        let pixels: Vec<f32> = resized
            .to_luma8()
            .pixels()
            .map(|p| {
                let brightness = p[0] as f32 / 255.0;
                if invert {
                    1.0 - brightness
                } else {
                    brightness
                }
            })
            .collect();

        Ok(Self {
            pixels,
            width: target_width,
            height: target_height,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
/// Geometric shape or image obstacle definition.
pub enum Obstacle {
    /// Circular obstacle.
    Circle {
        /// Center X.
        x: f32,
        /// Center Y.
        y: f32,
        /// Radius.
        radius: f32,
    },
    /// Rectangular obstacle.
    Rect {
        /// Top-left X.
        x: f32,
        /// Top-left Y.
        y: f32,
        /// Width.
        width: f32,
        /// Height.
        height: f32,
    },
    /// Image-based obstacle mask.
    Image {
        /// Path to image file.
        path: String,
        /// Top-left X.
        x: f32,
        /// Top-left Y.
        y: f32,
        /// Width.
        width: usize,
        /// Height.
        height: usize,
        /// Whether to invert the image mask.
        invert: bool,
        /// Brightness threshold for collision.
        threshold: f32,
    },
}

impl Obstacle {
    /// Checks if a point is contained within the obstacle.
    pub fn contains(&self, px: f32, py: f32, mask: Option<&ObstacleMask>) -> bool {
        match self {
            Obstacle::Circle { x, y, radius } => {
                let dx = px - x;
                let dy = py - y;
                dx * dx + dy * dy <= radius * radius
            }
            Obstacle::Rect {
                x,
                y,
                width,
                height,
            } => px >= *x && px <= *x + *width && py >= *y && py <= *y + *height,
            Obstacle::Image {
                path: _,
                x,
                y,
                width,
                height,
                invert: _,
                threshold,
            } => {
                let lx = px - x;
                let ly = py - y;
                if lx < 0.0 || lx >= *width as f32 || ly < 0.0 || ly >= *height as f32 {
                    return false;
                }
                if let Some(m) = mask {
                    let ix = lx as usize;
                    let iy = ly as usize;
                    let idx = iy * m.width + ix;
                    if idx >= m.pixels.len() {
                        return false;
                    }
                    m.pixels[idx] >= *threshold
                } else {
                    false
                }
            }
        }
    }

    /// Calculates new heading after bouncing off the obstacle.
    pub fn bounce(&self, px: f32, py: f32, heading: f32, _mask: Option<&ObstacleMask>) -> f32 {
        match self {
            Obstacle::Circle { x, y, radius: _ } => {
                let dx = px - x;
                let dy = py - y;
                let normal_angle = dy.atan2(dx);
                let new_heading = 2.0 * normal_angle - heading + std::f32::consts::PI;
                normalize_angle(new_heading)
            }
            Obstacle::Rect {
                x,
                y,
                width,
                height,
            } => {
                let nearest_x = px.clamp(*x, *x + *width);
                let nearest_y = py.clamp(*y, *y + *height);
                let dx = px - nearest_x;
                let dy = py - nearest_y;
                if dx.abs() > dy.abs() {
                    -heading + std::f32::consts::PI
                } else {
                    -heading
                }
            }
            Obstacle::Image {
                path: _,
                x: _,
                y: _,
                width: _,
                height: _,
                invert: _,
                threshold: _,
            } => -heading,
        }
    }
}

/// Boundary handling mode for agent movement.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BoundaryMode {
    /// Agents bounce/reflect at boundaries (default).
    #[default]
    Bounce,
    /// Agents wrap around to opposite side (toroidal).
    Wrap,
}

/// Window frame display mode for terminal visualization.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum WindowFrame {
    /// No frame - full terminal used for simulation.
    None,
    /// Solid block border using accent color.
    Accented,
    /// Gradient border fading from accent color inward.
    Glow,
    /// Thin-line frame (default).
    #[default]
    Frame,
}

impl WindowFrame {
    /// Returns true if this mode reduces simulation display area.
    ///
    /// The windowed layout reserves a frame ring for every mode uniformly, so
    /// no single mode specially reduces the area; retained for API stability.
    pub fn reduces_display_area(&self) -> bool {
        false
    }

    /// Returns the window frame thickness in cells.
    pub fn thickness(&self) -> usize {
        match self {
            WindowFrame::None => 0,
            WindowFrame::Frame => 2,
            WindowFrame::Accented => 1,
            WindowFrame::Glow => 3,
        }
    }

    /// Returns true if window frame has visual rendering.
    pub fn is_visible(&self) -> bool {
        !matches!(self, WindowFrame::None)
    }
}

impl std::str::FromStr for WindowFrame {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "none" => Ok(WindowFrame::None),
            "accented" => Ok(WindowFrame::Accented),
            "glow" => Ok(WindowFrame::Glow),
            "frame" => Ok(WindowFrame::Frame),
            _ => Err(format!(
                "Invalid window frame: {}. Must be one of: none, accented, glow, frame",
                s
            )),
        }
    }
}

/// Chrome display level for window mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ChromeStyle {
    /// Frame only, no title or footer (default).
    #[default]
    Minimal,
    /// Always-visible title block + footer (sticky expanded).
    Expanded,
    /// No window; sim fills terminal edge-to-edge.
    Fullscreen,
}

impl std::str::FromStr for ChromeStyle {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "minimal" => Ok(ChromeStyle::Minimal),
            "expanded" => Ok(ChromeStyle::Expanded),
            "fullscreen" => Ok(ChromeStyle::Fullscreen),
            _ => Err(format!(
                "Invalid chrome style: '{}'. Must be one of: minimal, expanded, fullscreen",
                s
            )),
        }
    }
}

/// Visual aspect ratio for the simulation window.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct Aspect {
    /// Horizontal units of the aspect ratio.
    pub width: u32,
    /// Vertical units of the aspect ratio.
    pub height: u32,
}

impl Default for Aspect {
    fn default() -> Self {
        Self {
            width: 3,
            height: 2,
        }
    }
}

impl Aspect {
    /// Terminal cell ratio (cells_w : cells_h) for halfblock rendering.
    ///
    /// With halfblock, each terminal cell packs 2 vertical sim pixels and is
    /// ~2:1 tall:wide, making halfblock pixels visually square. For a visual
    /// aspect of W:H, the required terminal cell ratio is W : (H/2).
    pub fn cell_ratio(&self) -> f32 {
        self.width as f32 / (self.height as f32 / 2.0)
    }
}

impl std::str::FromStr for Aspect {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "square" => {
                return Ok(Self {
                    width: 1,
                    height: 1,
                })
            }
            "4:3" => {
                return Ok(Self {
                    width: 4,
                    height: 3,
                })
            }
            "3:2" => {
                return Ok(Self {
                    width: 3,
                    height: 2,
                })
            }
            "16:10" => {
                return Ok(Self {
                    width: 16,
                    height: 10,
                })
            }
            "16:9" => {
                return Ok(Self {
                    width: 16,
                    height: 9,
                })
            }
            _ => {}
        }
        let parts: Vec<&str> = s.split(':').collect();
        if parts.len() != 2 {
            return Err(format!(
                "Invalid aspect '{}'. Use W:H or preset (square, 4:3, 3:2, 16:10, 16:9)",
                s
            ));
        }
        let w = parts[0]
            .parse::<u32>()
            .map_err(|_| format!("Invalid aspect width in '{}'", s))?;
        let h = parts[1]
            .parse::<u32>()
            .map_err(|_| format!("Invalid aspect height in '{}'", s))?;
        if w == 0 || h == 0 {
            return Err(format!("Aspect W and H must be non-zero, got '{}'", s));
        }
        Ok(Self {
            width: w,
            height: h,
        })
    }
}

/// Window outer padding — auto (5% of min terminal dimension, ≥ 2) or fixed cells.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub enum WindowPadding {
    /// Automatically compute padding (5% of smallest terminal dimension, minimum 2 cells).
    #[default]
    Auto,
    /// Fixed padding in terminal cells.
    Fixed(usize),
}

impl std::str::FromStr for WindowPadding {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.to_lowercase() == "auto" {
            return Ok(Self::Auto);
        }
        let n = s
            .parse::<usize>()
            .map_err(|_| format!("Invalid window padding '{}'. Use 'auto' or an integer.", s))?;
        Ok(Self::Fixed(n))
    }
}

/// Minimum terminal size threshold for fallback logic (WxH format).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct TerminalSizeThreshold {
    /// Minimum terminal width in columns.
    pub width: usize,
    /// Minimum terminal height in rows.
    pub height: usize,
}

impl Default for TerminalSizeThreshold {
    fn default() -> Self {
        Self {
            width: 20,
            height: 10,
        }
    }
}

impl std::str::FromStr for TerminalSizeThreshold {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split('x').collect();
        if parts.len() != 2 {
            return Err(format!(
                "Invalid size '{}'. Use WxH format, e.g. '20x10'",
                s
            ));
        }
        let w = parts[0]
            .parse::<usize>()
            .map_err(|_| format!("Invalid width in size '{}'", s))?;
        let h = parts[1]
            .parse::<usize>()
            .map_err(|_| format!("Invalid height in size '{}'", s))?;
        if w == 0 || h == 0 {
            return Err(format!("Size W and H must be non-zero, got '{}'", s));
        }
        Ok(Self {
            width: w,
            height: h,
        })
    }
}

/// Trail sampling method for agent sensing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SamplingMode {
    /// Fast nearest-pixel sampling (default).
    #[default]
    Nearest,
    /// Smooth bilinear interpolation.
    Bilinear,
}

impl std::str::FromStr for BoundaryMode {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "bounce" => Ok(BoundaryMode::Bounce),
            "wrap" | "toroidal" => Ok(BoundaryMode::Wrap),
            _ => Err(format!(
                "Invalid boundary mode: {}. Must be one of: bounce, wrap",
                s
            )),
        }
    }
}

/// 36 Points trail-based parameter modulation configuration.
///
/// This enables dynamic parameter adjustment based on the trail value at each agent's position,
/// creating diverse emergent behaviors as described in Sage Jenson's "36 Points" work.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct PointConfig {
    /// Sensor distance base value (p1).
    pub sensor_distance_base: f32,
    /// Sensor distance multiplier (p2).
    pub sensor_distance_multiplier: f32,
    /// Sensor distance exponent (p3).
    pub sensor_distance_exponent: f32,

    /// Sensor angle base value in degrees (p4).
    pub sensor_angle_base: f32,
    /// Sensor angle multiplier (p5).
    pub sensor_angle_multiplier: f32,
    /// Sensor angle exponent (p6).
    pub sensor_angle_exponent: f32,

    /// Rotation angle base value in degrees (p7).
    pub rotation_angle_base: f32,
    /// Rotation angle multiplier (p8).
    pub rotation_angle_multiplier: f32,
    /// Rotation angle exponent (p9).
    pub rotation_angle_exponent: f32,

    /// Step size base value (p10).
    pub step_size_base: f32,
    /// Step size multiplier (p11).
    pub step_size_multiplier: f32,
    /// Step size exponent (p12).
    pub step_size_exponent: f32,

    /// Absolute vertical offset in pixels (p13).
    pub vertical_offset: f32,
    /// Heading-relative offset in pixels (p14).
    pub heading_offset: f32,
    /// Trail value rescaling factor (p15).
    pub trail_rescale: f32,
}

impl Default for PointConfig {
    fn default() -> Self {
        Self {
            // Default: no modulation (multipliers = 0, exponents = 1)
            sensor_distance_base: agent_consts::DEFAULT_SENSOR_DISTANCE,
            sensor_distance_multiplier: 0.0,
            sensor_distance_exponent: 1.0,
            sensor_angle_base: agent_consts::DEFAULT_SENSOR_ANGLE,
            sensor_angle_multiplier: 0.0,
            sensor_angle_exponent: 1.0,
            rotation_angle_base: agent_consts::DEFAULT_ROTATION_ANGLE,
            rotation_angle_multiplier: 0.0,
            rotation_angle_exponent: 1.0,
            step_size_base: agent_consts::DEFAULT_STEP_SIZE,
            step_size_multiplier: 0.0,
            step_size_exponent: 1.0,
            vertical_offset: 0.0,
            heading_offset: 0.0,
            trail_rescale: 1.0,
        }
    }
}

/// Computed modulated parameters for an agent.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ModulatedParams {
    /// Modulated sensor distance.
    pub sensor_distance: f32,
    /// Modulated sensor angle in degrees.
    pub sensor_angle: f32,
    /// Modulated rotation angle in degrees.
    pub rotation_angle: f32,
    /// Modulated step size.
    pub step_size: f32,
}

impl PointConfig {
    /// Compute modulated parameters based on trail value x.
    ///
    /// Formulas:
    /// - sensor_distance = p1 + p2 * x^p3
    /// - sensor_angle    = p4 + p5 * x^p6
    /// - rotation_angle  = p7 + p8 * x^p9
    /// - step_size       = p10 + p11 * x^p12
    ///
    /// # Arguments
    /// * `x` - Trail value at agent position (should be in [0, 1])
    ///
    /// # Returns
    /// A `ModulatedParams` struct containing:
    /// - `sensor_distance`: Modulated sensor distance in pixels
    /// - `sensor_angle`: Modulated sensor angle in degrees
    /// - `rotation_angle`: Modulated rotation angle in degrees
    /// - `step_size`: Modulated step size in pixels
    pub fn compute_params(&self, x: f32) -> ModulatedParams {
        // Apply rescale factor and clamp to [0, 1]
        let x = (x * self.trail_rescale).clamp(0.0, 1.0);

        // Helper to compute modulated value with formula: base + multiplier * x^exponent
        let compute = |base: f32, multiplier: f32, exponent: f32| -> f32 {
            if multiplier == 0.0 || x == 0.0 {
                base
            } else if exponent == 1.0 {
                base + multiplier * x
            } else {
                base + multiplier * x.powf(exponent)
            }
        };

        ModulatedParams {
            sensor_distance: compute(
                self.sensor_distance_base,
                self.sensor_distance_multiplier,
                self.sensor_distance_exponent,
            )
            .clamp(
                agent_consts::MIN_SENSOR_DISTANCE,
                agent_consts::MAX_SENSOR_DISTANCE,
            ),
            sensor_angle: compute(
                self.sensor_angle_base,
                self.sensor_angle_multiplier,
                self.sensor_angle_exponent,
            )
            .clamp(
                agent_consts::MIN_SENSOR_ANGLE,
                agent_consts::MAX_SENSOR_ANGLE,
            ),
            rotation_angle: compute(
                self.rotation_angle_base,
                self.rotation_angle_multiplier,
                self.rotation_angle_exponent,
            )
            .clamp(
                agent_consts::MIN_ROTATION_ANGLE,
                agent_consts::MAX_ROTATION_ANGLE,
            ),
            step_size: compute(
                self.step_size_base,
                self.step_size_multiplier,
                self.step_size_exponent,
            )
            .clamp(agent_consts::MIN_STEP_SIZE, agent_consts::MAX_STEP_SIZE),
        }
    }

    /// Returns true if this config has any modulation enabled.
    pub fn has_modulation(&self) -> bool {
        self.sensor_distance_multiplier != 0.0
            || self.sensor_angle_multiplier != 0.0
            || self.rotation_angle_multiplier != 0.0
            || self.step_size_multiplier != 0.0
    }
}

/// Particle respawn configuration.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct RespawnConfig {
    /// Interval in frames between respawn checks (0 = disabled).
    pub interval: u32,
    /// Base probability of respawn when interval is reached (0.0-1.0).
    pub base_probability: f32,
    /// Whether respawn probability depends on trail value.
    pub trail_dependent: bool,
    /// Maximum respawn probability multiplier, reached when the normalized trail
    /// value saturates to 1.0. Effective probability is
    /// `base_probability * (1 + x * (max_probability_multiplier - 1))`, where
    /// `x = (trail * trail_rescale).clamp(0, 1)`.
    pub max_probability_multiplier: f32,
    /// Scales the raw pheromone value into the normalized `[0, 1]` range before
    /// the multiplier is applied (mirrors `PointConfig::trail_rescale`). Pick it
    /// so healthy trail densities map well below 1.0 and only an abnormal
    /// accumulation (the wall-collapse line) saturates — otherwise the
    /// multiplier cap is meaningless because raw trail values are unbounded.
    pub trail_rescale: f32,
}

impl Default for RespawnConfig {
    fn default() -> Self {
        Self {
            interval: 0, // Disabled by default
            base_probability: 0.01,
            trail_dependent: false,
            max_probability_multiplier: 1.0,
            trail_rescale: 1.0,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
/// Configuration for a single agent species.
pub struct SpeciesConfig {
    /// Species name.
    pub name: String,
    /// Population count.
    pub count: usize,
    /// Sensor angle (degrees).
    pub sensor_angle: f32,
    /// Rotation angle (degrees).
    pub rotation_angle: f32,
    /// Step size (speed).
    pub step_size: f32,
    /// Amount of pheromone deposited.
    pub deposit_amount: f32,
    /// Color as RGB.
    pub color: RgbColor,
    /// Trail-based parameter modulation (36 Points).
    pub trail_modulation: Option<PointConfig>,
}

impl Default for SpeciesConfig {
    fn default() -> Self {
        Self {
            name: "default".to_string(),
            count: population::DEFAULT_POPULATION,
            sensor_angle: agent_consts::DEFAULT_SENSOR_ANGLE,
            rotation_angle: agent_consts::DEFAULT_ROTATION_ANGLE,
            step_size: agent_consts::DEFAULT_STEP_SIZE,
            deposit_amount: agent_consts::DEFAULT_DEPOSIT_AMOUNT,
            color: RgbColor::from_hex(0x228b22), // Forest green
            trail_modulation: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
/// Global simulation configuration.
pub struct SimConfig {
    /// Sensor angle (degrees).
    pub sensor_angle: f32,
    /// Sensor offset distance (pixels).
    pub sensor_distance: f32,
    /// Rotation angle (degrees).
    pub rotation_angle: f32,
    /// Agent speed (pixels/step).
    pub step_size: f32,
    /// Trail decay factor (0.5-0.9999).
    pub decay_factor: f32,
    /// Amount of trail deposited per step.
    pub deposit_amount: f32,
    /// Diffusion algorithm.
    pub diffusion_kernel: DiffusionKernel,
    /// Sigma for Gaussian diffusion.
    pub diffusion_sigma: f32,
    /// Diffusion blend weight (Lague): `new = old·(1−w) + blur·w`. 1.0 = full blur (today).
    pub diffuse_weight: f32,
    /// Nonlinear decay exponent γ. 1.0 = current multiplicative decay; γ<1 lengthens faint tails.
    pub decay_gamma: f32,
    /// Nonlinear deposit curve applied to the per-frame accumulation buffer.
    /// `Linear` + scale 1 + cap 0 = historical behavior (off path).
    pub deposit_curve: DepositCurve,
    /// Multiplier applied to `curve(accum)` before folding into the trail.
    pub deposit_scale: f32,
    /// Exponent for `DepositCurve::Pow` (ignored by other curves).
    pub deposit_gamma: f32,
    /// Clamp cap for the folded contribution; `0.0` = off.
    pub deposit_cap: f32,
    /// White-point divisor for brightness normalization (higher = darker).
    pub max_brightness: f32,
    /// Time scale multiplier (0.1-10.0).
    pub time_scale: f32,
    /// List of active attractors.
    pub attractors: Vec<Attractor>,
    /// Global attractor strength multiplier.
    pub attractor_strength: f32,
    /// Temporary mouse attractors.
    pub mouse_attractors: Vec<MouseAttractor>,
    /// Timeout for mouse attractors (seconds).
    pub mouse_timeout: f32,
    /// Configuration for each species.
    pub species_configs: Vec<SpeciesConfig>,
    /// Whether to use separate trail maps per species.
    pub separate_species_trails: bool,
    /// Whether to use SIMD acceleration.
    pub use_simd: bool,
    /// Path to food image for initialization.
    pub food_image_path: Option<String>,
    /// Whether to invert food image brightness.
    pub food_image_invert: bool,
    /// Scaling factor for food image.
    pub food_image_scale: f32,
    /// List of obstacles.
    pub obstacles: Vec<Obstacle>,
    /// Loaded masks for image obstacles.
    pub obstacle_masks: Vec<Option<ObstacleMask>>,
    /// Global wind force.
    pub wind: Option<Wind>,
    /// Active terrain effect.
    pub terrain: TerrainType,
    /// Strength of terrain effect.
    pub terrain_strength: f32,
    /// Background color hex code.
    pub background_color: Option<String>,
    /// Preferred initialization mode for this config (if any).
    pub preferred_init_mode: Option<InitMode>,
    /// Boundary handling mode (bounce or wrap).
    pub boundary_mode: BoundaryMode,
    /// Window frame display mode for terminal visualization.
    pub window_frame: WindowFrame,
    /// Background matte width in columns between the frame border and the sim
    /// (left/right). Wider than `frame_matte_rows` to offset terminal cell aspect.
    pub frame_matte_cols: usize,
    /// Background matte height in rows between the frame border and the sim
    /// (top/bottom).
    pub frame_matte_rows: usize,
    /// Chrome display style (minimal, expanded, fullscreen).
    pub chrome_style: ChromeStyle,
    /// Visual aspect ratio of the simulation window.
    pub aspect: Aspect,
    /// Outer padding between terminal edge and window frame.
    pub window_padding: WindowPadding,
    /// Show legacy status bar in windowed mode (default false).
    pub show_status_bar: bool,
    /// Fallback threshold: below this sim size, drop padding.
    pub min_sim_size: TerminalSizeThreshold,
    /// Fallback threshold: below this sim size, drop the frame.
    pub min_frame_size: TerminalSizeThreshold,
    /// Particle respawn configuration.
    pub respawn_config: RespawnConfig,
    /// Trail sampling method (nearest or bilinear).
    pub sampling_mode: SamplingMode,
    /// Constellation atlas re-stamp strength, applied each frame after
    /// diffusion/decay. 0.0 = no re-stamp (drift); > 0.0 = self-healing
    /// template source (static hold).
    pub constellation_restamp_floor: f32,
}

impl SimConfig {
    /// Returns the total population across all species.
    pub fn total_population(&self) -> usize {
        self.species_configs.iter().map(|s| s.count).sum()
    }

    /// True when the nonlinear-deposit accumulation path is engaged. When
    /// false, deposits go straight to the trail (byte-identical to history).
    #[inline]
    pub fn deposit_active(&self) -> bool {
        self.deposit_curve != DepositCurve::Linear
            || self.deposit_scale != 1.0
            || self.deposit_cap > 0.0
    }

    /// Loads mask data for all image-based obstacles.
    pub fn load_obstacle_masks(&mut self) -> Result<(), String> {
        self.obstacle_masks.clear();
        for obstacle in &self.obstacles {
            match obstacle {
                Obstacle::Image {
                    path,
                    width,
                    height,
                    invert,
                    ..
                } => {
                    let mask = ObstacleMask::from_image(path, *width, *height, *invert)?;
                    self.obstacle_masks.push(Some(mask));
                }
                _ => {
                    self.obstacle_masks.push(None);
                }
            }
        }
        Ok(())
    }

    /// Adds a new mouse-controlled attractor.
    pub fn add_mouse_attractor(&mut self, x: f32, y: f32, strength: f32) {
        self.mouse_attractors
            .push(MouseAttractor::new(x, y, strength, self.mouse_timeout));
    }

    /// Removes mouse attractors that have timed out.
    pub fn remove_expired_mouse_attractors(&mut self) {
        self.mouse_attractors.retain(|ma| !ma.is_expired());
    }

    /// Returns a combined list of all active attractors (static + mouse).
    ///
    /// # Performance
    /// If there are no mouse attractors, returns a reference to the static attractors
    /// without cloning. Otherwise, returns an owned vector with both static and mouse attractors.
    pub fn effective_attractors(&self) -> Cow<'_, [Attractor]> {
        if self.mouse_attractors.is_empty() {
            // Fast path: no mouse attractors, return borrowed reference
            Cow::Borrowed(&self.attractors)
        } else {
            // Slow path: need to combine static and mouse attractors
            let mut result = self.attractors.clone();
            for ma in &self.mouse_attractors {
                result.push(Attractor::new(ma.x, ma.y, ma.strength));
            }
            Cow::Owned(result)
        }
    }
}

impl Default for SimConfig {
    fn default() -> Self {
        Self {
            sensor_angle: agent_consts::DEFAULT_SENSOR_ANGLE,
            sensor_distance: agent_consts::DEFAULT_SENSOR_DISTANCE,
            rotation_angle: agent_consts::DEFAULT_ROTATION_ANGLE,
            step_size: agent_consts::DEFAULT_STEP_SIZE,
            decay_factor: trail_consts::DEFAULT_DECAY_FACTOR,
            deposit_amount: agent_consts::DEFAULT_DEPOSIT_AMOUNT,
            diffusion_kernel: DiffusionKernel::Gaussian,
            diffusion_sigma: trail_consts::DEFAULT_DIFFUSION_SIGMA,
            diffuse_weight: trail_consts::DEFAULT_DIFFUSE_WEIGHT,
            decay_gamma: trail_consts::DEFAULT_DECAY_GAMMA,
            deposit_curve: DepositCurve::default(),
            deposit_scale: trail_consts::DEFAULT_DEPOSIT_SCALE,
            deposit_gamma: trail_consts::DEFAULT_DEPOSIT_GAMMA,
            deposit_cap: trail_consts::DEFAULT_DEPOSIT_CAP,
            max_brightness: trail_consts::DEFAULT_MAX_BRIGHTNESS,
            time_scale: time_consts::DEFAULT_TIME_SCALE,
            attractors: Vec::new(),
            attractor_strength: env_consts::DEFAULT_ATTRACTOR_STRENGTH,
            mouse_attractors: Vec::new(),
            mouse_timeout: env_consts::DEFAULT_MOUSE_TIMEOUT,
            species_configs: vec![SpeciesConfig::default()],
            separate_species_trails: false,
            use_simd: true,
            food_image_path: Some(food_img_consts::DEFAULT_FOOD_PATH.to_string()),
            food_image_invert: food_img_consts::DEFAULT_FOOD_INVERT,
            food_image_scale: food_img_consts::DEFAULT_FOOD_SCALE,
            obstacles: Vec::new(),
            obstacle_masks: Vec::new(),
            wind: None,
            terrain: TerrainType::None,
            terrain_strength: env_consts::DEFAULT_TERRAIN_STRENGTH,
            background_color: None,
            preferred_init_mode: Some(InitMode::Food),
            boundary_mode: BoundaryMode::Bounce,
            window_frame: WindowFrame::Frame,
            frame_matte_cols: crate::config_defaults::frame_matte::DEFAULT_COLS,
            frame_matte_rows: crate::config_defaults::frame_matte::DEFAULT_ROWS,
            chrome_style: ChromeStyle::Minimal,
            aspect: Aspect::default(),
            window_padding: WindowPadding::Auto,
            show_status_bar: false,
            min_sim_size: TerminalSizeThreshold {
                width: 20,
                height: 10,
            },
            min_frame_size: TerminalSizeThreshold {
                width: 12,
                height: 6,
            },
            respawn_config: RespawnConfig::default(),
            sampling_mode: SamplingMode::Nearest,
            constellation_restamp_floor:
                crate::config_defaults::DEFAULT_CONSTELLATION_RESTAMP_FLOOR,
        }
    }
}

// Validation implementations using the Validatable trait
use crate::error::ValidationError;
use crate::validation::{rules, Validatable};

impl Validatable for SimConfig {
    fn validate(&self) -> Result<(), ValidationError> {
        // Check that at least one species is configured
        if self.species_configs.is_empty() {
            return Err(ValidationError::custom(
                "at least one species must be configured",
            ));
        }

        // Validate total population
        let total_pop: usize = self.species_configs.iter().map(|s| s.count).sum();
        if !(population::MIN_POPULATION..=population::MAX_POPULATION).contains(&total_pop) {
            return Err(ValidationError::custom(format!(
                "total population must be between {} and {}, got {}",
                population::MIN_POPULATION,
                population::MAX_POPULATION,
                total_pop
            )));
        }

        // Validate agent parameters
        rules::SENSOR_ANGLE.validate_f32(self.sensor_angle)?;
        rules::SENSOR_DISTANCE.validate_f32(self.sensor_distance)?;
        rules::ROTATION_ANGLE.validate_f32(self.rotation_angle)?;
        rules::STEP_SIZE.validate_f32(self.step_size)?;
        rules::DEPOSIT_AMOUNT.validate_f32(self.deposit_amount)?;

        // Validate trail parameters
        rules::DECAY_FACTOR.validate_f32(self.decay_factor)?;
        rules::MAX_BRIGHTNESS.validate_f32(self.max_brightness)?;
        rules::DIFFUSION_SIGMA.validate_f32(self.diffusion_sigma)?;
        rules::DECAY_GAMMA.validate_f32(self.decay_gamma)?;
        rules::DIFFUSE_WEIGHT.validate_f32(self.diffuse_weight)?;
        rules::DEPOSIT_SCALE.validate_f32(self.deposit_scale)?;
        rules::DEPOSIT_GAMMA.validate_f32(self.deposit_gamma)?;
        rules::DEPOSIT_CAP.validate_f32(self.deposit_cap)?;

        // Validate time and environment parameters
        rules::TIME_SCALE.validate_f32(self.time_scale)?;
        rules::ATTRACTOR_STRENGTH.validate_f32(self.attractor_strength)?;
        rules::TERRAIN_STRENGTH.validate_f32(self.terrain_strength)?;

        // Validate individual attractors
        for (i, attractor) in self.attractors.iter().enumerate() {
            if attractor.strength < environment::MIN_ATTRACTOR_STRENGTH
                || attractor.strength > environment::MAX_ATTRACTOR_STRENGTH
            {
                return Err(ValidationError::out_of_range(
                    format!("attractor[{}].strength", i),
                    environment::MIN_ATTRACTOR_STRENGTH,
                    environment::MAX_ATTRACTOR_STRENGTH,
                    attractor.strength,
                ));
            }
        }

        // Validate species configs
        for species in &self.species_configs {
            Validatable::validate(species)?;
        }

        // Validate wind if present
        if let Some(ref wind) = self.wind {
            Validatable::validate(wind)?;
        }

        Ok(())
    }
}

impl TryFrom<&crate::cli::Args> for SimConfig {
    type Error = crate::error::ValidationError;

    /// Builds a validated `SimConfig` from parsed CLI args.
    ///
    /// Assembles the config (preset merge + CLI overrides + species/wind/terrain/obstacles),
    /// then validates the final merged config once through [`Validatable::validate`].
    ///
    /// # Errors
    /// Returns [`ValidationError`] if assembly fails (e.g. invalid terrain string) or any
    /// merged parameter is out of range.
    fn try_from(args: &crate::cli::Args) -> Result<Self, Self::Error> {
        let profile = crate::profile::Profile::resolve_from_args(args)
            .map_err(crate::error::ValidationError::custom)?;
        Ok(profile.sim)
    }
}

impl Validatable for SpeciesConfig {
    fn validate(&self) -> Result<(), ValidationError> {
        // Validate count
        if self.count < pop_consts::MIN_SPECIES_COUNT || self.count > pop_consts::MAX_SPECIES_COUNT
        {
            return Err(ValidationError::out_of_range(
                format!("species '{}' count", self.name),
                pop_consts::MIN_SPECIES_COUNT,
                pop_consts::MAX_SPECIES_COUNT,
                self.count,
            ));
        }

        // Validate sensor angle
        if self.sensor_angle < agent_consts::MIN_SENSOR_ANGLE
            || self.sensor_angle > agent_consts::MAX_SENSOR_ANGLE
        {
            return Err(ValidationError::out_of_range(
                format!("species '{}' sensor_angle", self.name),
                agent_consts::MIN_SENSOR_ANGLE,
                agent_consts::MAX_SENSOR_ANGLE,
                self.sensor_angle,
            ));
        }

        // Validate rotation angle
        if self.rotation_angle < agent_consts::MIN_ROTATION_ANGLE
            || self.rotation_angle > agent_consts::MAX_ROTATION_ANGLE
        {
            return Err(ValidationError::out_of_range(
                format!("species '{}' rotation_angle", self.name),
                agent_consts::MIN_ROTATION_ANGLE,
                agent_consts::MAX_ROTATION_ANGLE,
                self.rotation_angle,
            ));
        }

        // Validate step size
        if self.step_size < agent_consts::MIN_STEP_SIZE
            || self.step_size > agent_consts::MAX_STEP_SIZE
        {
            return Err(ValidationError::out_of_range(
                format!("species '{}' step_size", self.name),
                agent_consts::MIN_STEP_SIZE,
                agent_consts::MAX_STEP_SIZE,
                self.step_size,
            ));
        }

        // Validate deposit amount
        if self.deposit_amount < agent_consts::MIN_DEPOSIT_AMOUNT
            || self.deposit_amount > agent_consts::MAX_DEPOSIT_AMOUNT
        {
            return Err(ValidationError::out_of_range(
                format!("species '{}' deposit_amount", self.name),
                agent_consts::MIN_DEPOSIT_AMOUNT,
                agent_consts::MAX_DEPOSIT_AMOUNT,
                self.deposit_amount,
            ));
        }

        Ok(())
    }
}

impl From<Preset> for SimConfig {
    fn from(preset: Preset) -> Self {
        let mut config = Self::default();
        crate::preset_sim_defaults::PresetSimDefaults::from(preset).apply_to(&mut config);
        config
    }
}

// ── serde string conversion impls (used by #[serde(try_from = "String", into = "String")]) ──

impl From<Aspect> for String {
    fn from(a: Aspect) -> Self {
        format!("{}:{}", a.width, a.height)
    }
}

impl TryFrom<String> for Aspect {
    type Error = String;
    fn try_from(s: String) -> Result<Self, Self::Error> {
        s.parse()
    }
}

impl From<WindowPadding> for String {
    fn from(p: WindowPadding) -> Self {
        match p {
            WindowPadding::Auto => "auto".to_string(),
            WindowPadding::Fixed(n) => n.to_string(),
        }
    }
}

impl TryFrom<String> for WindowPadding {
    type Error = String;
    fn try_from(s: String) -> Result<Self, Self::Error> {
        s.parse()
    }
}

impl From<TerminalSizeThreshold> for String {
    fn from(t: TerminalSizeThreshold) -> Self {
        format!("{}x{}", t.width, t.height)
    }
}

impl TryFrom<String> for TerminalSizeThreshold {
    type Error = String;
    fn try_from(s: String) -> Result<Self, Self::Error> {
        s.parse()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::PI;

    #[test]
    fn init_mode_random_covers_all_over_many_draws() {
        use rand::SeedableRng;
        let mut rng = rand_xoshiro::Xoshiro256PlusPlus::seed_from_u64(42);
        const ALL: [InitMode; 9] = [
            InitMode::Random,
            InitMode::CentralBurst,
            InitMode::Circle,
            InitMode::Gradient,
            InitMode::WaveFront,
            InitMode::Spiral,
            InitMode::RandomClusters,
            InitMode::Food,
            InitMode::Petri,
        ];
        let mut seen = [false; 9];
        for _ in 0..1000 {
            let m = InitMode::random(&mut rng);
            let i = ALL.iter().position(|x| *x == m).unwrap();
            seen[i] = true;
        }
        assert!(seen.iter().all(|&s| s), "every InitMode should appear");
    }

    #[test]
    fn test_default_config() {
        let config = SimConfig::default();
        assert_eq!(config.total_population(), 50_000);
        assert_eq!(config.sensor_angle, 22.5);
        assert_eq!(config.sensor_distance, 9.0);
        assert_eq!(config.rotation_angle, 45.0);
        assert_eq!(config.step_size, 1.0);
        assert_eq!(config.decay_factor, 0.5);
        assert_eq!(config.deposit_amount, 5.0);
        assert_eq!(config.max_brightness, 100.0);
    }

    #[test]
    fn test_validate_default() {
        let config = SimConfig::default();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_validate_population_too_low() {
        let config = SimConfig {
            species_configs: vec![SpeciesConfig {
                count: 500,
                ..Default::default()
            }],
            ..Default::default()
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_validate_population_too_high() {
        let config = SimConfig {
            species_configs: vec![SpeciesConfig {
                count: 300_000,
                ..Default::default()
            }],
            ..Default::default()
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_validate_sensor_angle() {
        let config = SimConfig {
            sensor_angle: 100.0,
            ..Default::default()
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_validate_decay_factor() {
        let config = SimConfig {
            decay_factor: 1.0,
            ..Default::default()
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_validate_max_brightness_too_low() {
        let config = SimConfig {
            max_brightness: 0.5,
            ..Default::default()
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_validate_max_brightness_too_high() {
        let config = SimConfig {
            max_brightness: 1500.0,
            ..Default::default()
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_validate_attractor_strength_too_low() {
        let config = SimConfig {
            attractor_strength: 0.05,
            ..Default::default()
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_validate_attractor_strength_too_high() {
        let config = SimConfig {
            attractor_strength: 15.0,
            ..Default::default()
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_validate_attractor_strength_valid() {
        let config = SimConfig {
            attractor_strength: 5.0,
            ..Default::default()
        };
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_attractor_creation() {
        let attractor = Attractor::new(200.0, 200.0, 1.0);
        assert_eq!(attractor.x, 200.0);
        assert_eq!(attractor.y, 200.0);
        assert_eq!(attractor.strength, 1.0);
    }

    #[test]
    fn test_negative_attractor_strength() {
        let attractor = Attractor::new(200.0, 200.0, -1.0);
        assert_eq!(attractor.strength, -1.0);
    }

    #[test]
    fn test_species_config_default() {
        let species = SpeciesConfig::default();
        assert_eq!(species.count, 50_000);
        assert_eq!(species.sensor_angle, 22.5);
        assert_eq!(species.rotation_angle, 45.0);
        assert_eq!(species.step_size, 1.0);
        assert_eq!(species.deposit_amount, 5.0);
    }

    #[test]
    fn test_species_config_validate_count_too_low() {
        let species = SpeciesConfig {
            count: 50,
            ..Default::default()
        };
        assert!(species.validate().is_err());
    }

    #[test]
    fn test_species_config_validate_count_too_high() {
        let species = SpeciesConfig {
            count: 300_000,
            ..Default::default()
        };
        assert!(species.validate().is_err());
    }

    #[test]
    fn test_total_population_single_species() {
        let config = SimConfig {
            species_configs: vec![SpeciesConfig {
                count: 10000,
                ..Default::default()
            }],
            ..Default::default()
        };
        assert_eq!(config.total_population(), 10000);
    }

    #[test]
    fn test_total_population_multiple_species() {
        let config = SimConfig {
            species_configs: vec![
                SpeciesConfig {
                    count: 10000,
                    ..Default::default()
                },
                SpeciesConfig {
                    count: 20000,
                    name: "second".to_string(),
                    color: RgbColor::from_hex(0xff0000),
                    ..Default::default()
                },
            ],
            ..Default::default()
        };
        assert_eq!(config.total_population(), 30000);
    }

    #[test]
    fn test_obstacle_circle_contains() {
        let circle = Obstacle::Circle {
            x: 100.0,
            y: 100.0,
            radius: 50.0,
        };
        assert!(circle.contains(100.0, 100.0, None));
        assert!(circle.contains(100.0, 150.0, None));
        assert!(circle.contains(150.0, 100.0, None));
        assert!(!circle.contains(200.0, 100.0, None));
        assert!(!circle.contains(100.0, 200.0, None));
    }

    #[test]
    fn test_obstacle_rect_contains() {
        let rect = Obstacle::Rect {
            x: 100.0,
            y: 100.0,
            width: 50.0,
            height: 50.0,
        };
        assert!(rect.contains(100.0, 100.0, None));
        assert!(rect.contains(150.0, 150.0, None));
        assert!(!rect.contains(99.0, 100.0, None));
        assert!(!rect.contains(100.0, 99.0, None));
        assert!(!rect.contains(151.0, 100.0, None));
        assert!(!rect.contains(100.0, 151.0, None));
    }

    #[test]
    fn test_obstacle_circle_bounce() {
        let circle = Obstacle::Circle {
            x: 100.0,
            y: 100.0,
            radius: 50.0,
        };
        let heading = circle.bounce(100.0, 60.0, 0.0, None);
        assert!(
            heading.is_finite(),
            "Bounce should return a valid heading, got {}",
            heading
        );
    }

    #[test]
    fn test_obstacle_rect_bounce() {
        let rect = Obstacle::Rect {
            x: 100.0,
            y: 100.0,
            width: 50.0,
            height: 50.0,
        };
        let heading = rect.bounce(120.0, 100.0, 0.0, None);
        assert!(
            heading.is_finite(),
            "Bounce should return a valid heading, got {}",
            heading
        );
    }

    #[test]
    fn test_obstacle_mask_from_image_nonexistent() {
        let result = ObstacleMask::from_image("nonexistent.png", 100, 100, false);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not found"));
    }

    #[test]
    fn test_sim_config_load_obstacle_masks() {
        let mut config = SimConfig {
            obstacles: vec![Obstacle::Circle {
                x: 100.0,
                y: 100.0,
                radius: 50.0,
            }],
            ..Default::default()
        };
        let result = config.load_obstacle_masks();
        assert!(result.is_ok());
        assert_eq!(config.obstacle_masks.len(), 1);
        assert!(config.obstacle_masks[0].is_none());
    }

    #[test]
    fn test_wind_creation() {
        let wind = Wind::new(0.5, 0.5);
        assert_eq!(wind.dx, 0.5);
        assert_eq!(wind.dy, 0.5);
    }

    #[test]
    fn test_wind_validate_valid() {
        let wind = Wind::new(1.0, 1.0);
        assert!(wind.validate().is_ok());

        let wind = Wind::new(-1.0, 0.0);
        assert!(wind.validate().is_ok());

        let wind = Wind::new(0.0, -1.0);
        assert!(wind.validate().is_ok());
    }

    #[test]
    fn test_wind_validate_invalid_dx() {
        let wind = Wind::new(1.5, 0.0);
        assert!(wind.validate().is_err());
    }

    #[test]
    fn test_wind_validate_invalid_dy() {
        let wind = Wind::new(0.0, 1.5);
        assert!(wind.validate().is_err());
    }

    #[test]
    fn test_wind_validate_zero() {
        let wind = Wind::new(0.0, 0.0);
        assert!(wind.validate().is_err());
    }

    #[test]
    fn test_wind_parse() {
        let wind: Wind = "0.5,0.5".parse().unwrap();
        assert_eq!(wind.dx, 0.5);
        assert_eq!(wind.dy, 0.5);

        let wind: Wind = "-0.3,0.7".parse().unwrap();
        assert_eq!(wind.dx, -0.3);
        assert_eq!(wind.dy, 0.7);
    }

    #[test]
    fn test_wind_parse_invalid() {
        assert!("0.5".parse::<Wind>().is_err());
        assert!("0.5,0.5,extra".parse::<Wind>().is_err());
        assert!("abc,def".parse::<Wind>().is_err());
    }

    #[test]
    fn test_terrain_type_parse() {
        assert_eq!("none".parse::<TerrainType>().unwrap(), TerrainType::None);
        assert_eq!("off".parse::<TerrainType>().unwrap(), TerrainType::None);
        assert_eq!(
            "smooth".parse::<TerrainType>().unwrap(),
            TerrainType::Smooth
        );
        assert_eq!(
            "turbulent".parse::<TerrainType>().unwrap(),
            TerrainType::Turbulent
        );
        assert_eq!("mixed".parse::<TerrainType>().unwrap(), TerrainType::Mixed);

        assert_eq!("NONE".parse::<TerrainType>().unwrap(), TerrainType::None);
        assert_eq!(
            "Smooth".parse::<TerrainType>().unwrap(),
            TerrainType::Smooth
        );
    }

    #[test]
    fn test_terrain_type_parse_invalid() {
        assert!("invalid".parse::<TerrainType>().is_err());
        assert!("chaos".parse::<TerrainType>().is_err());
    }

    #[test]
    fn test_sim_config_wind_field() {
        let config = SimConfig {
            wind: Some(Wind::new(0.5, 0.0)),
            ..Default::default()
        };
        assert!(config.wind.is_some());
        assert_eq!(config.wind.unwrap().dx, 0.5);
    }

    #[test]
    fn test_sim_config_terrain_field() {
        let config = SimConfig {
            terrain: TerrainType::Turbulent,
            terrain_strength: 2.0,
            ..Default::default()
        };
        assert_eq!(config.terrain, TerrainType::Turbulent);
        assert_eq!(config.terrain_strength, 2.0);
    }

    #[test]
    fn test_validate_terrain_strength_too_low() {
        let config = SimConfig {
            terrain_strength: 0.05,
            ..Default::default()
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_validate_terrain_strength_too_high() {
        let config = SimConfig {
            terrain_strength: 10.0,
            ..Default::default()
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_validate_wind_invalid() {
        let config = SimConfig {
            wind: Some(Wind::new(1.5, 0.0)),
            ..Default::default()
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_effective_attractors() {
        let mut config = SimConfig {
            attractors: vec![Attractor::new(10.0, 10.0, 1.0)],
            ..Default::default()
        };
        config.add_mouse_attractor(20.0, 20.0, 2.0);
        let effective = config.effective_attractors();
        assert_eq!(effective.len(), 2);
        assert_eq!(effective[0].strength, 1.0);
        assert_eq!(effective[1].strength, 2.0);
    }

    #[test]
    fn test_mouse_attractor_expiry() {
        let ma = MouseAttractor::new(10.0, 10.0, 1.0, 0.01);
        assert!(!ma.is_expired());
        std::thread::sleep(std::time::Duration::from_millis(20));
        assert!(ma.is_expired());
    }

    #[test]
    fn test_remove_expired_mouse_attractors() {
        let mut config = SimConfig {
            mouse_timeout: 0.01,
            ..Default::default()
        };
        config.add_mouse_attractor(10.0, 10.0, 1.0);
        assert_eq!(config.mouse_attractors.len(), 1);
        std::thread::sleep(std::time::Duration::from_millis(20));
        config.remove_expired_mouse_attractors();
        assert_eq!(config.mouse_attractors.len(), 0);
    }

    #[test]
    fn test_presets_valid() {
        for spec in PRESETS {
            let config: SimConfig = spec.preset.into();
            assert!(
                config.validate().is_ok(),
                "Preset {:?} failed validation: {:?}",
                spec.preset,
                config.validate()
            );
        }
    }

    #[test]
    fn preset_names_and_aliases_round_trip() {
        for spec in PRESETS {
            // Display name resolves back to this preset, both via the method and
            // the parser, case-insensitively.
            assert_eq!(spec.preset.name(), spec.name);
            assert_eq!(preset_from_name(spec.name), Some(spec.preset));
            assert_eq!(
                preset_from_name(&spec.name.to_lowercase()),
                Some(spec.preset)
            );
            for alias in spec.aliases {
                assert_eq!(
                    preset_from_name(alias),
                    Some(spec.preset),
                    "alias {alias} did not resolve to {:?}",
                    spec.preset
                );
            }
        }
        assert_eq!(preset_from_name("definitely-not-a-preset"), None);
    }

    #[test]
    fn preset_quick_keys_are_consistent() {
        let mut seen = Vec::new();
        for spec in PRESETS {
            if let Some(key) = spec.quick_key {
                assert!(
                    key.is_ascii_digit() && key != '0',
                    "quick_key {key} is not 1-9"
                );
                assert!(!seen.contains(&key), "duplicate quick_key {key}");
                seen.push(key);
                // The set key round-trips, and its shifted form selects the same
                // preset for comparison.
                assert_eq!(preset_for_set_key(key), Some(spec.preset));
                let shifted = match key {
                    '1' => '!',
                    '2' => '@',
                    '3' => '#',
                    '4' => '$',
                    '5' => '%',
                    '6' => '^',
                    '7' => '&',
                    _ => continue,
                };
                assert_eq!(preset_for_compare_key(shifted), Some(spec.preset));
            }
        }
    }

    #[test]
    fn launch_quick_keys_map_to_launch_presets() {
        assert_eq!(preset_for_set_key('1'), Some(Preset::Organic));
        assert_eq!(preset_for_set_key('2'), Some(Preset::Constellation));
        assert_eq!(preset_for_set_key('3'), Some(Preset::Vinescii));
        for c in ['4', '5', '6', '7'] {
            assert_eq!(
                preset_for_set_key(c),
                None,
                "key {c} must be unbound at launch"
            );
        }
        assert_eq!(preset_for_compare_key('@'), Some(Preset::Constellation));
    }

    #[test]
    fn test_try_from_args_valid() {
        use crate::cli::Args;
        use clap::Parser;
        let args = Args::parse_from(["tslime"]);
        let config = SimConfig::try_from(&args);
        assert!(
            config.is_ok(),
            "default args must convert: {:?}",
            config.err()
        );
    }

    #[test]
    fn test_try_from_args_rejects_out_of_range_sensor_angle() {
        use crate::cli::Args;
        use clap::Parser;
        let args = Args::parse_from(["tslime", "--sensor-angle", "200"]);
        let result = SimConfig::try_from(&args);
        assert!(result.is_err(), "sensor_angle 200 must be rejected");
    }

    #[test]
    #[cfg(feature = "multi-species")]
    fn test_try_from_args_rejects_bad_species_strict() {
        // NEW behavior: species params are now validated post-merge.
        use crate::cli::Args;
        use clap::Parser;
        let args = Args::parse_from(["tslime", "--species", "x:20000@999,45,1.0,5.0:ff0000"]);
        let result = SimConfig::try_from(&args);
        assert!(
            result.is_err(),
            "out-of-range species param must now error (was silently accepted)"
        );
    }

    #[test]
    #[cfg(not(feature = "multi-species"))]
    fn test_species_flag_rejected_without_feature() {
        use crate::cli::Args;
        use clap::Parser;
        assert!(
            Args::try_parse_from(["tslime", "--species", "x:1000"]).is_err(),
            "--species must be unknown without multi-species feature"
        );
    }

    #[test]
    fn test_obstacle_rect_bounce_sides() {
        let rect = Obstacle::Rect {
            x: 100.0,
            y: 100.0,
            width: 50.0,
            height: 50.0,
        };
        // Bounce off top/bottom (dy > dx)
        let h1 = rect.bounce(125.0, 99.9, 0.1, None);
        assert!((h1 - (-0.1)).abs() < 0.001);
        // Bounce off left/right (dx > dy)
        let h2 = rect.bounce(99.9, 125.0, 0.1, None);
        assert!((h2 - (PI - 0.1)).abs() < 0.001);
    }

    #[test]
    fn test_species_config_validate_all() {
        let s = SpeciesConfig {
            sensor_angle: 1.0,
            ..Default::default()
        };
        assert!(s.validate().is_err());
        let s = SpeciesConfig {
            rotation_angle: 1.0,
            ..Default::default()
        };
        assert!(s.validate().is_err());
        let s = SpeciesConfig {
            step_size: 0.005,
            ..Default::default()
        };
        assert!(s.validate().is_err());
        let s = SpeciesConfig {
            deposit_amount: 0.05,
            ..Default::default()
        };
        assert!(s.validate().is_err());
    }

    #[test]
    fn test_validatable_trait() {
        use crate::validation::Validatable;

        let valid_config = SimConfig::default();
        assert!(valid_config.validate().is_ok());

        let invalid_config = SimConfig {
            sensor_angle: 200.0, // Invalid
            ..Default::default()
        };
        assert!(invalid_config.validate().is_err());
    }

    #[test]
    fn test_species_validatable_trait() {
        use crate::validation::Validatable;

        let valid_species = SpeciesConfig::default();
        assert!(valid_species.validate().is_ok());

        let invalid_species = SpeciesConfig {
            count: 50, // Below minimum
            ..Default::default()
        };
        assert!(invalid_species.validate().is_err());
    }

    #[test]
    fn art_knob_defaults_are_backcompat_neutral() {
        let c = SimConfig::default();
        assert_eq!(
            c.diffuse_weight, 1.0,
            "diffuse_weight=1 == full blur == today"
        );
        assert_eq!(c.decay_gamma, 1.0, "decay_gamma=1 == current decay");
    }

    #[test]
    fn validate_decay_gamma_rejects_out_of_range() {
        let config = SimConfig {
            decay_gamma: 9999.0,
            ..SimConfig::default()
        };
        assert!(
            config.validate().is_err(),
            "decay_gamma=9999.0 must be rejected"
        );
    }

    #[test]
    fn validate_decay_gamma_accepts_valid() {
        let config = SimConfig {
            decay_gamma: 1.0,
            ..SimConfig::default()
        };
        assert!(
            config.validate().is_ok(),
            "decay_gamma=1.0 must be accepted"
        );
    }

    #[test]
    fn deposit_curve_apply_matches_definitions() {
        use crate::simulation::config::DepositCurve;
        // Linear is identity; gamma ignored.
        assert_eq!(DepositCurve::Linear.apply(3.0, 0.5), 3.0);
        // Sqrt.
        assert!((DepositCurve::Sqrt.apply(9.0, 1.0) - 3.0).abs() < 1e-6);
        assert_eq!(DepositCurve::Sqrt.apply(0.0, 1.0), 0.0);
        // Log is log1p: 0 at 0, monotonic.
        assert_eq!(DepositCurve::Log.apply(0.0, 1.0), 0.0);
        assert!((DepositCurve::Log.apply(std::f32::consts::E - 1.0, 1.0) - 1.0).abs() < 1e-6);
        // Pow uses gamma as exponent.
        assert!((DepositCurve::Pow.apply(4.0, 0.5) - 2.0).abs() < 1e-6);
        assert!((DepositCurve::Pow.apply(2.0, 2.0) - 4.0).abs() < 1e-6);
        // Default is Linear.
        assert_eq!(DepositCurve::default(), DepositCurve::Linear);
    }

    #[test]
    fn deposit_active_off_at_defaults() {
        let cfg = SimConfig::default();
        assert_eq!(cfg.deposit_curve, DepositCurve::Linear);
        assert_eq!(cfg.deposit_scale, 1.0);
        assert_eq!(cfg.deposit_gamma, 1.0);
        assert_eq!(cfg.deposit_cap, 0.0);
        assert!(!cfg.deposit_active(), "defaults must be the off path");

        let on = SimConfig {
            deposit_curve: DepositCurve::Sqrt,
            ..Default::default()
        };
        assert!(on.deposit_active());
        let scaled = SimConfig {
            deposit_scale: 2.0,
            ..Default::default()
        };
        assert!(scaled.deposit_active());
        let capped = SimConfig {
            deposit_cap: 5.0,
            ..Default::default()
        };
        assert!(capped.deposit_active());
    }

    #[test]
    fn deposit_validation_rejects_out_of_range() {
        let cfg = SimConfig {
            deposit_gamma: 0.0, // below MIN_DEPOSIT_GAMMA
            ..Default::default()
        };
        assert!(cfg.validate().is_err());
    }
}

#[cfg(test)]
mod window_type_tests {
    use super::*;

    #[test]
    fn test_aspect_from_str_presets() {
        assert_eq!(
            "3:2".parse::<Aspect>().unwrap(),
            Aspect {
                width: 3,
                height: 2
            }
        );
        assert_eq!(
            "square".parse::<Aspect>().unwrap(),
            Aspect {
                width: 1,
                height: 1
            }
        );
        assert_eq!(
            "4:3".parse::<Aspect>().unwrap(),
            Aspect {
                width: 4,
                height: 3
            }
        );
        assert_eq!(
            "16:10".parse::<Aspect>().unwrap(),
            Aspect {
                width: 16,
                height: 10
            }
        );
        assert_eq!(
            "16:9".parse::<Aspect>().unwrap(),
            Aspect {
                width: 16,
                height: 9
            }
        );
    }

    #[test]
    fn test_aspect_from_str_custom() {
        assert_eq!(
            "5:3".parse::<Aspect>().unwrap(),
            Aspect {
                width: 5,
                height: 3
            }
        );
    }

    #[test]
    fn test_aspect_from_str_errors() {
        assert!("0:1".parse::<Aspect>().is_err());
        assert!("bad".parse::<Aspect>().is_err());
        assert!("1:2:3".parse::<Aspect>().is_err());
    }

    #[test]
    fn test_aspect_cell_ratio_3_2() {
        let aspect = Aspect {
            width: 3,
            height: 2,
        };
        // For 3:2 visual with halfblock: cell_ratio = 3 / (2/2) = 3.0
        assert!((aspect.cell_ratio() - 3.0).abs() < 0.001);
    }

    #[test]
    fn test_chrome_style_from_str() {
        assert_eq!(
            "minimal".parse::<ChromeStyle>().unwrap(),
            ChromeStyle::Minimal
        );
        assert_eq!(
            "expanded".parse::<ChromeStyle>().unwrap(),
            ChromeStyle::Expanded
        );
        assert_eq!(
            "fullscreen".parse::<ChromeStyle>().unwrap(),
            ChromeStyle::Fullscreen
        );
        assert!("invalid".parse::<ChromeStyle>().is_err());
    }

    #[test]
    fn test_window_padding_from_str() {
        assert_eq!(
            "auto".parse::<WindowPadding>().unwrap(),
            WindowPadding::Auto
        );
        assert_eq!(
            "4".parse::<WindowPadding>().unwrap(),
            WindowPadding::Fixed(4)
        );
        assert!("bad".parse::<WindowPadding>().is_err());
    }

    #[test]
    fn test_terminal_size_threshold_from_str() {
        let t = "20x10".parse::<TerminalSizeThreshold>().unwrap();
        assert_eq!(t.width, 20);
        assert_eq!(t.height, 10);
        assert!("bad".parse::<TerminalSizeThreshold>().is_err());
        assert!("0x10".parse::<TerminalSizeThreshold>().is_err());
    }
}
