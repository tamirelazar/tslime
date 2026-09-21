//! Coarse spatial index over obstacles so the per-agent collision test in
//! [`Agent::move_forward`](super::agent::Agent::move_forward) only visits the
//! obstacles that can possibly contain the agent's position.
//!
//! Without this the collision loop is O(agents × obstacles) per step. That is
//! fine for the handful of obstacles presets ship, but a `--border-ring` adds
//! 40–70 circles and the loop dominated the step (2.5× in release, 4.5× in
//! debug). With the index an interior agent costs one cell lookup and zero
//! `contains` tests.
//!
//! Layout is CSR: `offsets[cell]..offsets[cell + 1]` slices `indices`, so the
//! hot path allocates nothing.

use super::config::Obstacle;

/// Side length of one index cell in sim pixels.
const CELL_SIZE: f32 = 16.0;

/// Cell → obstacle-id lists for a `width`×`height` grid.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ObstacleIndex {
    cols: usize,
    rows: usize,
    inv_cell: f32,
    /// `cols * rows + 1` entries; slice bounds into `indices`.
    offsets: Vec<u32>,
    /// Obstacle ids, grouped by cell, ascending within a cell.
    indices: Vec<u16>,
}

impl ObstacleIndex {
    /// Builds the index. Every obstacle is registered in every cell its
    /// bounding box touches; boxes are clamped into the grid so obstacles
    /// hanging past an edge still resolve for agents that step out of bounds.
    pub fn build(obstacles: &[Obstacle], width: usize, height: usize) -> Self {
        let cols = (width as f32 / CELL_SIZE).ceil().max(1.0) as usize;
        let rows = (height as f32 / CELL_SIZE).ceil().max(1.0) as usize;
        let n_cells = cols * rows;
        if obstacles.is_empty() {
            return Self {
                cols,
                rows,
                inv_cell: 1.0 / CELL_SIZE,
                offsets: vec![0; n_cells + 1],
                indices: Vec::new(),
            };
        }

        // Pass 1: count per cell. Pass 2: fill. Same cell walk both times.
        let mut counts = vec![0u32; n_cells];
        let mut spans = Vec::with_capacity(obstacles.len());
        for obstacle in obstacles {
            let (x0, y0, x1, y1) = bounding_box(obstacle);
            let c0 = cell_coord(x0, cols, 1.0 / CELL_SIZE);
            let c1 = cell_coord(x1, cols, 1.0 / CELL_SIZE);
            let r0 = cell_coord(y0, rows, 1.0 / CELL_SIZE);
            let r1 = cell_coord(y1, rows, 1.0 / CELL_SIZE);
            spans.push((c0, c1, r0, r1));
            for r in r0..=r1 {
                for c in c0..=c1 {
                    counts[r * cols + c] += 1;
                }
            }
        }
        let mut offsets = Vec::with_capacity(n_cells + 1);
        let mut acc = 0u32;
        offsets.push(0);
        for &n in &counts {
            acc += n;
            offsets.push(acc);
        }
        let mut fill: Vec<u32> = offsets[..n_cells].to_vec();
        let mut indices = vec![0u16; acc as usize];
        for (id, &(c0, c1, r0, r1)) in spans.iter().enumerate() {
            for r in r0..=r1 {
                for c in c0..=c1 {
                    let cell = r * cols + c;
                    indices[fill[cell] as usize] = id as u16;
                    fill[cell] += 1;
                }
            }
        }
        Self {
            cols,
            rows,
            inv_cell: 1.0 / CELL_SIZE,
            offsets,
            indices,
        }
    }

    /// Obstacle ids that may contain `(x, y)`. Out-of-grid points map to the
    /// nearest edge cell. Empty for the (common) interior case.
    #[inline]
    pub fn candidates(&self, x: f32, y: f32) -> &[u16] {
        if self.indices.is_empty() {
            return &[];
        }
        let c = cell_coord(x, self.cols, self.inv_cell);
        let r = cell_coord(y, self.rows, self.inv_cell);
        let cell = r * self.cols + c;
        &self.indices[self.offsets[cell] as usize..self.offsets[cell + 1] as usize]
    }

    /// True when no obstacle is registered anywhere.
    pub fn is_empty(&self) -> bool {
        self.indices.is_empty()
    }
}

/// Cell coordinate for a sim-pixel coordinate, clamped into `[0, n)`.
#[inline]
fn cell_coord(v: f32, n: usize, inv_cell: f32) -> usize {
    if v.is_nan() || v <= 0.0 {
        // Negative, zero, or NaN all land in the first cell.
        return 0;
    }
    ((v * inv_cell) as usize).min(n - 1)
}

/// Axis-aligned bounding box `(x0, y0, x1, y1)` of an obstacle.
fn bounding_box(obstacle: &Obstacle) -> (f32, f32, f32, f32) {
    match obstacle {
        Obstacle::Circle { x, y, radius } => (x - radius, y - radius, x + radius, y + radius),
        Obstacle::Rect {
            x,
            y,
            width,
            height,
        } => (*x, *y, x + width, y + height),
        Obstacle::Image {
            x,
            y,
            width,
            height,
            ..
        } => (*x, *y, x + *width as f32, y + *height as f32),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::simulation::config::{BorderRing, SimConfig};

    fn circle(x: f32, y: f32, radius: f32) -> Obstacle {
        Obstacle::Circle { x, y, radius }
    }

    /// Every obstacle that contains a point must be among that point's candidates.
    #[test]
    fn candidates_are_a_superset_of_containing_obstacles() {
        let obstacles = vec![
            circle(200.0, 100.0, 50.0),
            Obstacle::Rect {
                x: 10.0,
                y: 10.0,
                width: 30.0,
                height: 30.0,
            },
            circle(395.0, 5.0, 8.0),  // hangs off the corner
            circle(-3.0, 100.0, 5.0), // mostly outside the grid
        ];
        let index = ObstacleIndex::build(&obstacles, 400, 200);
        let mut probes = Vec::new();
        for yi in -2..=202 {
            for xi in -2..=402 {
                probes.push((xi as f32 * 1.0 + 0.5, yi as f32 * 1.0 + 0.5));
            }
        }
        for (x, y) in probes {
            let cands = index.candidates(x, y);
            for (i, o) in obstacles.iter().enumerate() {
                if o.contains(x, y, None) {
                    assert!(
                        cands.contains(&(i as u16)),
                        "obstacle {i} contains ({x},{y}) but is not a candidate: {cands:?}"
                    );
                }
            }
        }
    }

    #[test]
    fn interior_has_no_candidates_with_a_border_ring() {
        let mut cfg = SimConfig {
            border_ring: Some(BorderRing::default()),
            ..SimConfig::default()
        };
        cfg.expand_border_ring(400, 200);
        assert!(cfg.obstacles.len() > 40);
        let index = ObstacleIndex::build(&cfg.obstacles, 400, 200);
        assert!(index.candidates(200.0, 100.0).is_empty());
        assert!(index.candidates(100.0, 60.0).is_empty());
        // At the wall there are candidates.
        assert!(!index.candidates(1.0, 100.0).is_empty());
    }

    #[test]
    fn empty_index_is_cheap_and_empty() {
        let index = ObstacleIndex::build(&[], 400, 200);
        assert!(index.is_empty());
        assert!(index.candidates(-5.0, f32::NAN).is_empty());
    }
}
