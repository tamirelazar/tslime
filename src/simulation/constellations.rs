//! Hand-placed stellar-constellation asterisms used by `InitMode::Constellation`.
//! Coords are normalized to 0..1 (origin top-left); `edges` index into `stars`.

use crate::simulation::config::Aspect;
use crate::simulation::Rng;
use rand::Rng as _;

/// One asterism: bright stars plus the line segments that connect them.
pub struct Constellation {
    /// Human-readable name of the constellation.
    pub name: &'static str,
    /// Normalized star positions as (x, y) in 0..1 (origin top-left).
    pub stars: &'static [(f32, f32)],
    /// Edges connecting stars, as indices into the `stars` array.
    pub edges: &'static [(u8, u8)],
}

// Orion: belt + shoulders + feet. Indices: 0 Betelgeuse, 1 Bellatrix, 2-4 belt,
// 5 Saiph, 6 Rigel, 7 Meissa (head).
const ORION: Constellation = Constellation {
    name: "Orion",
    stars: &[
        (0.30, 0.18),
        (0.66, 0.20),
        (0.40, 0.50),
        (0.50, 0.52),
        (0.60, 0.54),
        (0.34, 0.86),
        (0.70, 0.84),
        (0.48, 0.04),
    ],
    edges: &[
        (0, 2),
        (1, 4),
        (2, 3),
        (3, 4),
        (2, 5),
        (4, 6),
        (0, 7),
        (1, 7),
    ],
};

const URSA_MAJOR: Constellation = Constellation {
    name: "Big Dipper",
    stars: &[
        (0.08, 0.40),
        (0.26, 0.46),
        (0.44, 0.52),
        (0.60, 0.46),
        (0.74, 0.30),
        (0.90, 0.40),
        (0.70, 0.58),
    ],
    edges: &[(0, 1), (1, 2), (2, 3), (3, 4), (4, 5), (5, 6), (6, 3)],
};

const CASSIOPEIA: Constellation = Constellation {
    name: "Cassiopeia",
    stars: &[
        (0.08, 0.40),
        (0.30, 0.62),
        (0.50, 0.36),
        (0.70, 0.64),
        (0.92, 0.40),
    ],
    edges: &[(0, 1), (1, 2), (2, 3), (3, 4)],
};

const CYGNUS: Constellation = Constellation {
    name: "Cygnus",
    stars: &[
        (0.50, 0.06),
        (0.50, 0.40),
        (0.50, 0.70),
        (0.50, 0.94),
        (0.18, 0.30),
        (0.82, 0.30),
    ],
    edges: &[(0, 1), (1, 2), (2, 3), (4, 1), (1, 5)],
};

const SCORPIUS: Constellation = Constellation {
    name: "Scorpius",
    stars: &[
        (0.12, 0.14),
        (0.22, 0.26),
        (0.34, 0.36),
        (0.46, 0.50),
        (0.56, 0.66),
        (0.62, 0.82),
        (0.76, 0.84),
        (0.86, 0.72),
    ],
    edges: &[(0, 1), (1, 2), (2, 3), (3, 4), (4, 5), (5, 6), (6, 7)],
};

const CRUX: Constellation = Constellation {
    name: "Southern Cross",
    stars: &[(0.50, 0.06), (0.50, 0.94), (0.16, 0.50), (0.84, 0.46)],
    edges: &[(0, 1), (2, 3)],
};

/// All shipped figures, in display order.
pub const ALL: &[Constellation] = &[ORION, URSA_MAJOR, CASSIOPEIA, CYGNUS, SCORPIUS, CRUX];

/// Pick one figure using the seeded simulation RNG (deterministic under `--seed`).
pub fn pick(rng: &mut Rng) -> &'static Constellation {
    let i = rng.gen_range(0..ALL.len());
    &ALL[i]
}

/// Map normalized 0..1 figure coords into grid pixels, centered with a 10%
/// margin, scaled so the figure's **visual** aspect matches `aspect`. Halfblock
/// packs 2 vertical px per cell, so a visual ratio W:H needs pixel ratio W:(H*2).
pub fn fit_to_grid(
    stars: &[(f32, f32)],
    width: usize,
    height: usize,
    aspect: Aspect,
) -> Vec<(f32, f32)> {
    const MARGIN: f32 = 0.10;
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

    stars
        .iter()
        .map(|&(nx, ny)| (off_x + nx * box_w, off_y + ny * box_h))
        .collect()
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
}
