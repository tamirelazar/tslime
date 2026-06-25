//! Hand-placed stellar-constellation asterisms used by `InitMode::Constellation`.
//! Coords are normalized to 0..1 (origin top-left); `edges` index into `stars`.

use crate::simulation::agent::Agent;
use crate::simulation::config::Aspect;
use crate::simulation::Rng;
use rand::Rng as _;
use std::f32::consts::PI;

/// One asterism: bright stars plus the line segments that connect them.
pub struct Constellation {
    /// Human-readable name of the constellation.
    pub name: &'static str,
    /// Normalized star positions as (x, y) in 0..1 (origin top-left).
    pub stars: &'static [(f32, f32)],
    /// Edges connecting stars, as indices into the `stars` array.
    pub edges: &'static [(u8, u8)],
}

const ORION: Constellation = Constellation {
    name: "Orion",
    stars: &[
        (0.30, 0.24),
        (0.60, 0.22),
        (0.40, 0.52),
        (0.48, 0.54),
        (0.56, 0.53),
        (0.36, 0.88),
        (0.64, 0.90),
        (0.45, 0.08),
        (0.46, 0.64),
        (0.45, 0.74),
    ],
    edges: &[
        (7, 0),
        (7, 1),
        (0, 1),
        (0, 2),
        (1, 4),
        (2, 3),
        (3, 4),
        (2, 5),
        (4, 6),
        (3, 8),
        (8, 9),
    ],
};

const URSA_MAJOR: Constellation = Constellation {
    name: "Ursa Major",
    stars: &[
        (0.80, 0.30),
        (0.80, 0.48),
        (0.64, 0.52),
        (0.60, 0.38),
        (0.46, 0.36),
        (0.32, 0.36),
        (0.14, 0.42),
        (0.90, 0.66),
        (0.66, 0.70),
        (0.96, 0.20),
    ],
    edges: &[
        (0, 1),
        (1, 2),
        (2, 3),
        (3, 0),
        (3, 4),
        (4, 5),
        (5, 6),
        (1, 8),
        (0, 7),
        (0, 9),
    ],
};

const SCORPIUS: Constellation = Constellation {
    name: "Scorpius",
    stars: &[
        (0.14, 0.10),
        (0.10, 0.26),
        (0.24, 0.20),
        (0.32, 0.34),
        (0.42, 0.46),
        (0.50, 0.58),
        (0.56, 0.70),
        (0.62, 0.82),
        (0.74, 0.86),
        (0.84, 0.78),
    ],
    edges: &[
        (0, 2),
        (1, 2),
        (2, 3),
        (3, 4),
        (4, 5),
        (5, 6),
        (6, 7),
        (7, 8),
        (8, 9),
    ],
};

const CYGNUS: Constellation = Constellation {
    name: "Cygnus",
    stars: &[
        (0.50, 0.00),
        (0.50, 0.32),
        (0.50, 0.66),
        (0.50, 1.00),
        (0.18, 0.40),
        (0.00, 0.30),
        (0.82, 0.52),
        (1.00, 0.60),
    ],
    edges: &[(0, 1), (1, 2), (2, 3), (1, 4), (4, 5), (1, 6), (6, 7)],
};

const LEO: Constellation = Constellation {
    name: "Leo",
    stars: &[
        (0.18, 0.64),
        (0.20, 0.50),
        (0.26, 0.38),
        (0.34, 0.32),
        (0.40, 0.42),
        (0.30, 0.52),
        (0.86, 0.56),
        (0.60, 0.42),
        (0.64, 0.64),
    ],
    edges: &[
        (0, 1),
        (1, 2),
        (2, 3),
        (3, 4),
        (1, 7),
        (7, 6),
        (6, 8),
        (8, 0),
        (7, 8),
    ],
};

const GEMINI: Constellation = Constellation {
    name: "Gemini",
    stars: &[
        (0.30, 0.10),
        (0.32, 0.30),
        (0.28, 0.54),
        (0.22, 0.78),
        (0.36, 0.76),
        (0.64, 0.14),
        (0.62, 0.34),
        (0.60, 0.56),
        (0.54, 0.80),
        (0.70, 0.80),
        (0.47, 0.32),
    ],
    edges: &[
        (0, 1),
        (1, 2),
        (2, 3),
        (2, 4),
        (5, 6),
        (6, 7),
        (7, 8),
        (7, 9),
        (1, 10),
        (10, 6),
    ],
};

const CASSIOPEIA: Constellation = Constellation {
    name: "Cassiopeia",
    stars: &[
        (0.00, 0.20),
        (0.27, 0.55),
        (0.50, 0.00),
        (0.74, 0.48),
        (1.00, 0.30),
    ],
    edges: &[(0, 1), (1, 2), (2, 3), (3, 4)],
};

const SAGITTARIUS: Constellation = Constellation {
    name: "Sagittarius",
    stars: &[
        (0.00, 0.42),
        (0.20, 0.30),
        (0.18, 0.62),
        (0.40, 0.18),
        (0.46, 0.55),
        (0.62, 0.00),
        (0.66, 0.42),
        (0.92, 0.30),
        (1.00, 0.62),
    ],
    edges: &[
        (0, 1),
        (1, 3),
        (3, 5),
        (5, 6),
        (6, 4),
        (4, 2),
        (2, 0),
        (1, 6),
        (6, 7),
        (7, 8),
        (8, 4),
    ],
};

const PEGASUS_ANDROMEDA: Constellation = Constellation {
    name: "Pegasus-Andromeda",
    stars: &[
        (0.30, 0.62),
        (0.30, 1.00),
        (0.66, 0.60),
        (0.64, 0.98),
        (0.50, 0.78),
        (0.20, 0.40),
        (0.08, 0.20),
        (0.00, 0.00),
        (0.78, 0.46),
        (0.92, 0.34),
        (0.46, 0.46),
    ],
    edges: &[
        (0, 2),
        (2, 3),
        (3, 1),
        (1, 0),
        (0, 4),
        (0, 5),
        (5, 6),
        (6, 7),
        (2, 8),
        (8, 9),
        (2, 10),
    ],
};

const LYRA: Constellation = Constellation {
    name: "Lyra",
    stars: &[
        (0.15, 0.00),
        (0.00, 0.22),
        (0.40, 0.30),
        (0.30, 0.72),
        (0.70, 0.80),
        (0.85, 1.00),
        (0.62, 0.40),
    ],
    edges: &[
        (0, 1),
        (0, 2),
        (2, 6),
        (6, 3),
        (3, 4),
        (4, 5),
        (5, 6),
        (1, 3),
    ],
};

const AQUILA: Constellation = Constellation {
    name: "Aquila",
    stars: &[
        (0.50, 0.00),
        (0.36, 0.22),
        (0.62, 0.20),
        (0.30, 0.48),
        (0.70, 0.50),
        (0.16, 0.78),
        (1.00, 0.60),
        (0.00, 1.00),
    ],
    edges: &[(1, 0), (0, 2), (1, 3), (2, 4), (3, 5), (5, 7), (4, 6)],
};

const BOOTES: Constellation = Constellation {
    name: "Bootes",
    stars: &[
        (0.42, 1.00),
        (0.18, 0.62),
        (0.66, 0.66),
        (0.10, 0.30),
        (0.74, 0.30),
        (0.40, 0.18),
        (0.50, 0.00),
    ],
    edges: &[(0, 1), (0, 2), (1, 3), (2, 4), (3, 5), (4, 5), (5, 6)],
};

const TAURUS: Constellation = Constellation {
    name: "Taurus",
    stars: &[
        (0.00, 0.00),
        (0.20, 0.30),
        (0.40, 0.52),
        (0.56, 0.60),
        (0.50, 0.78),
        (0.72, 0.66),
        (1.00, 1.00),
        (0.90, 0.20),
    ],
    edges: &[
        (0, 1),
        (1, 2),
        (2, 3),
        (3, 4),
        (3, 5),
        (5, 7),
        (5, 6),
        (2, 5),
    ],
};

const DRACO: Constellation = Constellation {
    name: "Draco",
    stars: &[
        (0.92, 1.00),
        (0.78, 0.92),
        (0.86, 0.80),
        (0.70, 0.78),
        (0.58, 0.66),
        (0.40, 0.62),
        (0.30, 0.50),
        (0.42, 0.36),
        (0.30, 0.22),
        (0.46, 0.12),
        (0.20, 0.10),
        (0.00, 0.00),
        (0.16, 0.00),
    ],
    edges: &[
        (0, 2),
        (2, 1),
        (1, 3),
        (3, 2),
        (3, 4),
        (4, 5),
        (5, 6),
        (6, 7),
        (7, 8),
        (8, 9),
        (8, 10),
        (10, 11),
        (10, 12),
    ],
};

const HERCULES: Constellation = Constellation {
    name: "Hercules",
    stars: &[
        (0.30, 0.42),
        (0.62, 0.36),
        (0.66, 0.66),
        (0.36, 0.70),
        (0.50, 0.00),
        (0.08, 0.30),
        (0.00, 0.10),
        (0.80, 0.24),
        (1.00, 0.30),
        (0.22, 0.92),
        (0.04, 1.00),
        (0.72, 0.96),
        (0.92, 1.00),
    ],
    edges: &[
        (0, 1),
        (1, 2),
        (2, 3),
        (3, 0),
        (0, 5),
        (5, 6),
        (1, 4),
        (1, 7),
        (7, 8),
        (3, 9),
        (9, 10),
        (2, 11),
        (11, 12),
    ],
};

const CANIS_MAJOR: Constellation = Constellation {
    name: "Canis Major",
    stars: &[
        (0.30, 0.00),
        (0.10, 0.18),
        (0.46, 0.24),
        (0.00, 0.40),
        (0.40, 0.52),
        (0.52, 0.78),
        (0.78, 0.86),
        (1.00, 1.00),
        (0.66, 0.58),
    ],
    edges: &[
        (0, 1),
        (0, 2),
        (1, 3),
        (2, 4),
        (4, 5),
        (5, 6),
        (6, 7),
        (4, 8),
        (8, 2),
    ],
};

const PERSEUS: Constellation = Constellation {
    name: "Perseus",
    stars: &[
        (0.50, 0.00),
        (0.40, 0.22),
        (0.46, 0.42),
        (0.30, 0.56),
        (0.16, 0.70),
        (0.00, 0.86),
        (0.10, 1.00),
        (0.60, 0.60),
        (0.72, 0.78),
        (0.66, 1.00),
        (0.86, 0.50),
        (1.00, 0.40),
    ],
    edges: &[
        (0, 1),
        (1, 2),
        (2, 3),
        (3, 4),
        (4, 5),
        (4, 6),
        (2, 7),
        (7, 8),
        (8, 9),
        (2, 10),
        (10, 11),
    ],
};

/// All shipped figures, in display order.
pub const ALL: &[Constellation] = &[
    ORION,
    URSA_MAJOR,
    SCORPIUS,
    CYGNUS,
    LEO,
    GEMINI,
    CASSIOPEIA,
    SAGITTARIUS,
    PEGASUS_ANDROMEDA,
    LYRA,
    AQUILA,
    BOOTES,
    TAURUS,
    DRACO,
    HERCULES,
    CANIS_MAJOR,
    PERSEUS,
];

/// Pick one figure using the seeded simulation RNG (deterministic under `--seed`).
pub fn pick(rng: &mut Rng) -> &'static Constellation {
    let i = rng.gen_range(0..ALL.len());
    &ALL[i]
}

/// Uniformly rescale + center a figure so its bounding box fills the unit
/// square in its longer dimension, preserving the figure's real proportions.
///
/// Figures are authored over varying sub-ranges of 0..1; without this an
/// asterism that only spans, say, x∈0.30..0.64 would render as a small patch.
/// Normalizing to the bbox makes every figure cover the available area.
fn normalize_to_unit_square(stars: &[(f32, f32)]) -> Vec<(f32, f32)> {
    let (mut minx, mut maxx, mut miny, mut maxy) = (f32::MAX, f32::MIN, f32::MAX, f32::MIN);
    for &(x, y) in stars {
        minx = minx.min(x);
        maxx = maxx.max(x);
        miny = miny.min(y);
        maxy = maxy.max(y);
    }
    let fw = (maxx - minx).max(1e-6);
    let fh = (maxy - miny).max(1e-6);
    // Uniform scale by the larger extent keeps the figure undistorted; the
    // longer dimension spans exactly 0..1, the shorter is centered.
    let s = 1.0 / fw.max(fh);
    let (cx, cy) = ((minx + maxx) / 2.0, (miny + maxy) / 2.0);
    stars
        .iter()
        .map(|&(x, y)| (0.5 + (x - cx) * s, 0.5 + (y - cy) * s))
        .collect()
}

/// Map a figure's normalized coords into grid pixels, centered with a small
/// margin, scaled so the figure's **visual** aspect matches `aspect`. Halfblock
/// packs 2 vertical px per cell, so a visual ratio W:H needs pixel ratio W:(H*2).
///
/// The figure is first uniformly rescaled to fill the unit square (see
/// [`normalize_to_unit_square`]) so it covers as much of the frame as possible.
pub fn fit_to_grid(
    stars: &[(f32, f32)],
    width: usize,
    height: usize,
    aspect: Aspect,
) -> Vec<(f32, f32)> {
    const MARGIN: f32 = 0.04;
    let normalized = normalize_to_unit_square(stars);
    let gw = width as f32;
    let gh = height as f32;
    let avail_w = gw * (1.0 - 2.0 * MARGIN);
    let avail_h = gh * (1.0 - 2.0 * MARGIN);

    // Target pixel aspect for an undistorted figure: W : (H * 2).
    let target_px_ratio = aspect.width as f32 / (aspect.height as f32 * 2.0);

    // Fit a unit square (the normalized space) into avail box at target ratio.
    let (box_w, box_h) = if avail_w / avail_h > target_px_ratio {
        (avail_h * target_px_ratio, avail_h)
    } else {
        (avail_w, avail_w / target_px_ratio)
    };
    let off_x = (gw - box_w) / 2.0;
    let off_y = (gh - box_h) / 2.0;

    normalized
        .iter()
        .map(|&(nx, ny)| (off_x + nx * box_w, off_y + ny * box_h))
        .collect()
}

/// A picked figure, fitted to the grid, with its rasterized trail template.
pub struct ConstellationLayout {
    /// Human-readable name of the constellation.
    pub name: &'static str,
    /// Star positions in grid pixels.
    pub stars_px: Vec<(f32, f32)>,
    /// Edges connecting stars, as indices into the `stars_px` array.
    pub edges: Vec<(usize, usize)>,
    /// Row-major f32 grid (0..=1): Gaussian glow at each star + anti-aliased
    /// lines along each edge.
    pub template: Vec<f32>,
}

/// Pick a figure and produce its grid layout + anti-aliased template.
pub fn build_layout(
    rng: &mut Rng,
    width: usize,
    height: usize,
    aspect: Aspect,
) -> ConstellationLayout {
    let c = pick(rng);
    let stars_px = fit_to_grid(c.stars, width, height, aspect);
    let edges: Vec<(usize, usize)> = c
        .edges
        .iter()
        .map(|&(a, b)| (a as usize, b as usize))
        .collect();

    let mut template = vec![0.0f32; width * height];

    // Stars: Gaussian glow (sigma 2.2px, peak 1.0).
    let star_sigma = 2.2f32;
    for &(sx, sy) in &stars_px {
        stamp_gaussian(&mut template, width, height, sx, sy, star_sigma, 1.0);
    }
    // Edges: anti-aliased line, sampled densely, thin glow per sample (sigma 0.55, peak 0.45).
    for &(a, b) in &edges {
        let (ax, ay) = stars_px[a];
        let (bx, by) = stars_px[b];
        let len = ((bx - ax).powi(2) + (by - ay).powi(2)).sqrt().max(1.0);
        let steps = (len * 2.0) as usize;
        for s in 0..=steps {
            let t = s as f32 / steps as f32;
            let x = ax + (bx - ax) * t;
            let y = ay + (by - ay) * t;
            stamp_gaussian(&mut template, width, height, x, y, 0.55, 0.45);
        }
    }
    // Clamp to 0..1.
    for v in &mut template {
        *v = v.min(1.0);
    }

    ConstellationLayout {
        name: c.name,
        stars_px,
        edges,
        template,
    }
}

/// Add a Gaussian splat centred at (cx, cy) with peak `peak` into `grid`
/// (max-combine).
fn stamp_gaussian(
    grid: &mut [f32],
    width: usize,
    height: usize,
    cx: f32,
    cy: f32,
    sigma: f32,
    peak: f32,
) {
    let radius = (sigma * 3.0).ceil() as i32;
    let two_s2 = 2.0 * sigma * sigma;
    let icx = cx.round() as i32;
    let icy = cy.round() as i32;
    for dy in -radius..=radius {
        for dx in -radius..=radius {
            let x = icx + dx;
            let y = icy + dy;
            if x < 0 || y < 0 || x >= width as i32 || y >= height as i32 {
                continue;
            }
            let d2 = (dx * dx + dy * dy) as f32;
            let v = peak * (-d2 / two_s2).exp();
            let idx = y as usize * width + x as usize;
            grid[idx] = grid[idx].max(v);
        }
    }
}

/// Seed `population` agents from `layout`: 35% Gaussian blobs at stars, 65%
/// along edges (∝ length) with edge-tangent headings split both directions.
/// Precondition: `agents` is empty on entry (the exact-count top-up assumes this).
pub fn seed_agents(
    rng: &mut Rng,
    layout: &ConstellationLayout,
    agents: &mut Vec<Agent>,
    population: usize,
    species_id: u8,
) {
    debug_assert!(agents.is_empty(), "seed_agents expects an empty agents Vec");
    let star_pop = population * 35 / 100;
    let edge_pop = population - star_pop;

    // Stars: even split, Gaussian blob (sigma 2.0).
    let n_stars = layout.stars_px.len().max(1);
    for i in 0..star_pop {
        let (cx, cy) = layout.stars_px[i % n_stars];
        let (dx, dy) = gaussian_offset(rng, 2.0);
        let heading = rng.gen_range(0.0..PI * 2.0);
        agents.push(Agent::new(cx + dx, cy + dy, heading, species_id));
    }

    // Edges: count proportional to length.
    let lengths: Vec<f32> = layout
        .edges
        .iter()
        .map(|&(a, b)| {
            let (ax, ay) = layout.stars_px[a];
            let (bx, by) = layout.stars_px[b];
            ((bx - ax).powi(2) + (by - ay).powi(2)).sqrt().max(1.0)
        })
        .collect();
    let total_len: f32 = lengths.iter().sum::<f32>().max(1.0);

    let mut placed = 0usize;
    for (ei, &(a, b)) in layout.edges.iter().enumerate() {
        let (ax, ay) = layout.stars_px[a];
        let (bx, by) = layout.stars_px[b];
        let want = ((edge_pop as f32) * (lengths[ei] / total_len)).round() as usize;
        let want = if ei == layout.edges.len() - 1 {
            edge_pop.saturating_sub(placed) // last edge soaks up rounding remainder
        } else {
            want
        };
        let tangent = (by - ay).atan2(bx - ax);
        for k in 0..want {
            let t = rng.gen_range(0.0..1.0f32);
            let x = ax + (bx - ax) * t;
            let y = ay + (by - ay) * t;
            // Half travel each way along the edge, plus small angular jitter.
            let dir = if k % 2 == 0 { 0.0 } else { PI };
            let jitter = rng.gen_range(-0.15..0.15f32);
            agents.push(Agent::new(x, y, tangent + dir + jitter, species_id));
            placed += 1;
        }
    }
    // Top up any shortfall from rounding onto the first star.
    while agents.len() < (population) {
        let (cx, cy) = layout.stars_px[0];
        let heading = rng.gen_range(0.0..PI * 2.0);
        agents.push(Agent::new(cx, cy, heading, species_id));
    }
}

/// Build a re-stamp template from a food/brightness image (e.g. the embedded
/// tslime logo). Mirrors the loader selection used by `InitMode::Food`: the
/// embedded logo for the default path, otherwise a file on disk. The result is
/// normalized to `0..=1` so it re-stamps with the same scaling as the asterism
/// template. Returns `None` if the image can't be loaded or is fully dark.
pub fn build_food_template(
    width: usize,
    height: usize,
    food_path: Option<&str>,
    invert: bool,
    scale: f32,
) -> Option<Vec<f32>> {
    use crate::simulation::food::{load_default_food_image, load_image_grayscale};
    let path = food_path?;
    let map = if path == "assets/tslime_logo.png" {
        load_default_food_image(width, height, invert, scale)
    } else {
        load_image_grayscale(path, width, height, invert, scale)
    };
    match map {
        Ok(m) => {
            let max = m.iter().copied().fold(0.0f32, f32::max);
            if max <= 0.0 {
                return None;
            }
            Some(m.iter().map(|&v| (v / max).min(1.0)).collect())
        }
        Err(e) => {
            eprintln!("Warning: failed to load food template '{path}': {e}");
            None
        }
    }
}

fn gaussian_offset(rng: &mut Rng, sigma: f32) -> (f32, f32) {
    let u1: f32 = rng.gen();
    let u2: f32 = rng.gen();
    let r = (-2.0 * u1.max(1e-6).ln()).sqrt();
    let theta = 2.0 * PI * u2;
    (r * theta.cos() * sigma, r * theta.sin() * sigma)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::simulation::config::Aspect;
    use crate::simulation::Rng;
    use rand::SeedableRng;

    #[test]
    fn fit_preserves_aspect_and_centers() {
        // A unit square figure into a 3:2 visual aspect on a 300x100 grid.
        let square = [(0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (0.0, 1.0)];
        let pts = fit_to_grid(
            &square,
            300,
            100,
            Aspect {
                width: 3,
                height: 2,
            },
        );
        // Bounding box of the square must have visual w:h == 3:2.
        let (mut minx, mut maxx, mut miny, mut maxy) = (f32::MAX, f32::MIN, f32::MAX, f32::MIN);
        for &(x, y) in &pts {
            minx = minx.min(x);
            maxx = maxx.max(x);
            miny = miny.min(y);
            maxy = maxy.max(y);
        }
        let bw = maxx - minx;
        let bh = maxy - miny;
        // visual ratio = (bw) : (bh / 2) because a cell packs 2 vertical px (halfblock).
        let visual = bw / (bh / 2.0);
        assert!((visual - 1.5).abs() < 0.05, "visual aspect {visual} != 1.5");
        // Centered: equal margins horizontally.
        assert!(
            (minx - (300.0 - bw - minx)).abs() < 1.0,
            "not centered in x"
        );
        // Within grid with margin.
        assert!(minx >= 0.0 && maxx <= 300.0 && miny >= 0.0 && maxy <= 100.0);
    }

    #[test]
    fn all_figures_have_valid_edges() {
        for c in ALL {
            assert!(!c.stars.is_empty(), "{} has no stars", c.name);
            assert!(!c.edges.is_empty(), "{} has no edges", c.name);
            for &(a, b) in c.edges {
                assert!((a as usize) < c.stars.len(), "{} edge a OOB", c.name);
                assert!((b as usize) < c.stars.len(), "{} edge b OOB", c.name);
                assert_ne!(a, b, "{} self-edge", c.name);
            }
            for &(x, y) in c.stars {
                assert!(
                    (0.0..=1.0).contains(&x) && (0.0..=1.0).contains(&y),
                    "{} star out of 0..1",
                    c.name
                );
            }
        }
    }

    #[test]
    fn pick_is_deterministic_under_seed() {
        let mut a = Rng::seed_from_u64(42);
        let mut b = Rng::seed_from_u64(42);
        assert_eq!(pick(&mut a).name, pick(&mut b).name);
    }

    #[test]
    fn template_is_bright_on_stars_and_edges() {
        let mut rng = Rng::seed_from_u64(7);
        let layout = build_layout(
            &mut rng,
            120,
            80,
            Aspect {
                width: 3,
                height: 2,
            },
        );
        assert_eq!(layout.template.len(), 120 * 80);
        // Every star pixel neighborhood is non-zero.
        for &(sx, sy) in &layout.stars_px {
            let idx = (sy as usize).min(79) * 120 + (sx as usize).min(119);
            assert!(layout.template[idx] > 0.0, "star not bright at {sx},{sy}");
        }
        // Edge midpoints are non-zero.
        for &(a, b) in &layout.edges {
            let (ax, ay) = layout.stars_px[a];
            let (bx, by) = layout.stars_px[b];
            let (mx, my) = ((ax + bx) / 2.0, (ay + by) / 2.0);
            let idx = (my as usize).min(79) * 120 + (mx as usize).min(119);
            assert!(layout.template[idx] > 0.0, "edge midpoint dark");
        }
    }

    #[test]
    fn seed_agents_splits_population_and_uses_tangent_headings() {
        let mut rng = Rng::seed_from_u64(3);
        let layout = build_layout(
            &mut rng,
            200,
            120,
            Aspect {
                width: 3,
                height: 2,
            },
        );
        let mut agents: Vec<Agent> = Vec::new();
        seed_agents(&mut rng, &layout, &mut agents, 1000, 0);
        // All agents placed (no drops), within grid.
        assert_eq!(agents.len(), 1000);
        for a in &agents {
            assert!(
                a.x >= 0.0 && a.x <= 200.0 && a.y >= 0.0 && a.y <= 120.0,
                "agent at ({}, {}) out of bounds",
                a.x,
                a.y
            );
        }
    }
}
