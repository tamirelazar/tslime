//! Metrics for quantifying emergent pattern characteristics.
//!
//! These metrics are used to mathematically characterize the visual patterns
//! produced by different parameter combinations, enabling automated discovery
//! of interesting presets.

use crate::simulation::agent::Agent;

/// Collection of metrics describing a simulation state.
#[derive(Debug, Clone, Default)]
pub struct PatternMetrics {
    /// Collective angular momentum (positive = counterclockwise rotation).
    /// High values indicate vortex/swirling behavior.
    pub angular_momentum: f32,

    /// Variance of agent headings (0 = all same direction, high = chaotic).
    pub heading_variance: f32,

    /// Number of disconnected trail regions (high = blobby/fragmented).
    pub trail_fragmentation: u32,

    /// Average elongation ratio of trail clusters (high = worm-like).
    pub trail_elongation: f32,

    /// Spatial entropy of trail distribution (high = uniform, low = concentrated).
    pub spatial_entropy: f32,

    /// Frame-to-frame correlation (high = stable/crystal-like).
    pub temporal_stability: f32,

    /// Variance of trail density across grid (high = high contrast networks).
    pub density_variance: f32,

    /// Average trail intensity (brightness).
    pub mean_intensity: f32,

    /// Fraction of grid cells with non-zero trail.
    pub coverage: f32,

    /// Estimated branching factor of trail network.
    pub branching_factor: f32,
}

impl PatternMetrics {
    /// Compute all metrics from current simulation state.
    pub fn compute(
        trail_map: &[f32],
        width: usize,
        height: usize,
        agents: &[Agent],
        previous_trail: Option<&[f32]>,
    ) -> Self {
        let angular_momentum = compute_angular_momentum(agents, width, height);
        let heading_variance = compute_heading_variance(agents);
        let trail_fragmentation = compute_fragmentation(trail_map, width, height);
        let trail_elongation = compute_elongation(trail_map, width, height);
        let spatial_entropy = compute_spatial_entropy(trail_map);
        let temporal_stability = previous_trail
            .map(|prev| compute_temporal_stability(trail_map, prev))
            .unwrap_or(1.0);
        let density_variance = compute_density_variance(trail_map);
        let mean_intensity = compute_mean_intensity(trail_map);
        let coverage = compute_coverage(trail_map);
        let branching_factor = compute_branching_factor(trail_map, width, height);

        Self {
            angular_momentum,
            heading_variance,
            trail_fragmentation,
            trail_elongation,
            spatial_entropy,
            temporal_stability,
            density_variance,
            mean_intensity,
            coverage,
            branching_factor,
        }
    }

    /// Score for vortex-like behavior (swirling).
    pub fn vortex_score(&self) -> f32 {
        // Want high angular momentum, low heading variance (coherent rotation)
        self.angular_momentum.abs() * (1.0 / (1.0 + self.heading_variance))
    }

    /// Score for lightning-like behavior (branching dendrites).
    pub fn lightning_score(&self) -> f32 {
        // Want high branching, low coverage (sparse), high contrast
        self.branching_factor * (1.0 - self.coverage) * self.density_variance.sqrt()
    }

    /// Score for crystal-like behavior (stable structures).
    pub fn crystal_score(&self) -> f32 {
        // Want high temporal stability, moderate coverage
        self.temporal_stability * (1.0 - (self.coverage - 0.3).abs())
    }

    /// Score for blob-like behavior (isolated clusters).
    pub fn blob_score(&self) -> f32 {
        // Want high fragmentation, low elongation, moderate coverage
        (self.trail_fragmentation as f32).sqrt() * (1.0 / (1.0 + self.trail_elongation))
    }

    /// Score for worm-like behavior (long snaking trails).
    pub fn worm_score(&self) -> f32 {
        // Want high elongation, low fragmentation, low coverage
        self.trail_elongation * (1.0 / (1.0 + self.trail_fragmentation as f32)) * (1.0 - self.coverage)
    }

    /// Score for chaos-edge behavior (high sensitivity).
    pub fn chaos_score(&self) -> f32 {
        // Want moderate values of everything with high variance
        // This is tricky - we'd need multiple runs to measure sensitivity
        // For now, use heading variance as proxy
        self.heading_variance * self.density_variance
    }
}

/// Compute collective angular momentum around grid center.
fn compute_angular_momentum(agents: &[Agent], width: usize, height: usize) -> f32 {
    let cx = width as f32 / 2.0;
    let cy = height as f32 / 2.0;

    let mut total_l = 0.0f64;
    for agent in agents {
        let rx = agent.x - cx;
        let ry = agent.y - cy;
        let vx = agent.heading.cos();
        let vy = agent.heading.sin();
        // L = r × v (cross product z-component)
        total_l += (rx * vy - ry * vx) as f64;
    }

    (total_l / agents.len().max(1) as f64) as f32
}

/// Compute circular variance of agent headings.
fn compute_heading_variance(agents: &[Agent]) -> f32 {
    if agents.is_empty() {
        return 0.0;
    }

    // Use circular statistics: R = |mean of unit vectors|
    let mut sum_cos = 0.0f64;
    let mut sum_sin = 0.0f64;
    for agent in agents {
        sum_cos += agent.heading.cos() as f64;
        sum_sin += agent.heading.sin() as f64;
    }
    let n = agents.len() as f64;
    let r = ((sum_cos / n).powi(2) + (sum_sin / n).powi(2)).sqrt();

    // Circular variance = 1 - R (0 = all same direction, 1 = uniform)
    (1.0 - r) as f32
}

/// Count disconnected trail regions using flood fill.
fn compute_fragmentation(trail_map: &[f32], width: usize, height: usize) -> u32 {
    let threshold = 0.1;
    let mut visited = vec![false; trail_map.len()];
    let mut count = 0;

    for start in 0..trail_map.len() {
        if visited[start] || trail_map[start] < threshold {
            continue;
        }

        // Flood fill from this point
        let mut stack = vec![start];
        while let Some(idx) = stack.pop() {
            if visited[idx] {
                continue;
            }
            visited[idx] = true;

            let x = idx % width;
            let y = idx / width;

            // Check 4-neighbors
            if x > 0 {
                let left = idx - 1;
                if !visited[left] && trail_map[left] >= threshold {
                    stack.push(left);
                }
            }
            if x < width - 1 {
                let right = idx + 1;
                if !visited[right] && trail_map[right] >= threshold {
                    stack.push(right);
                }
            }
            if y > 0 {
                let up = idx - width;
                if !visited[up] && trail_map[up] >= threshold {
                    stack.push(up);
                }
            }
            if y < height - 1 {
                let down = idx + width;
                if !visited[down] && trail_map[down] >= threshold {
                    stack.push(down);
                }
            }
        }
        count += 1;
    }

    count
}

/// Compute average elongation of trail clusters.
fn compute_elongation(trail_map: &[f32], width: usize, height: usize) -> f32 {
    let threshold = 0.1;
    let mut visited = vec![false; trail_map.len()];
    let mut total_elongation = 0.0f32;
    let mut cluster_count = 0u32;

    for start in 0..trail_map.len() {
        if visited[start] || trail_map[start] < threshold {
            continue;
        }

        // Collect cluster points
        let mut points: Vec<(usize, usize)> = Vec::new();
        let mut stack = vec![start];

        while let Some(idx) = stack.pop() {
            if visited[idx] {
                continue;
            }
            visited[idx] = true;
            let x = idx % width;
            let y = idx / width;
            points.push((x, y));

            if x > 0 && !visited[idx - 1] && trail_map[idx - 1] >= threshold {
                stack.push(idx - 1);
            }
            if x < width - 1 && !visited[idx + 1] && trail_map[idx + 1] >= threshold {
                stack.push(idx + 1);
            }
            if y > 0 && !visited[idx - width] && trail_map[idx - width] >= threshold {
                stack.push(idx - width);
            }
            if y < height - 1 && !visited[idx + width] && trail_map[idx + width] >= threshold {
                stack.push(idx + width);
            }
        }

        if points.len() < 10 {
            continue; // Skip tiny clusters
        }

        // Compute bounding box
        let min_x = points.iter().map(|p| p.0).min().unwrap_or(0);
        let max_x = points.iter().map(|p| p.0).max().unwrap_or(0);
        let min_y = points.iter().map(|p| p.1).min().unwrap_or(0);
        let max_y = points.iter().map(|p| p.1).max().unwrap_or(0);

        let dx = (max_x - min_x + 1) as f32;
        let dy = (max_y - min_y + 1) as f32;

        // Elongation = max(dx/dy, dy/dx)
        let elongation = (dx / dy).max(dy / dx);
        total_elongation += elongation;
        cluster_count += 1;
    }

    if cluster_count == 0 {
        1.0
    } else {
        total_elongation / cluster_count as f32
    }
}

/// Compute spatial entropy of trail distribution.
fn compute_spatial_entropy(trail_map: &[f32]) -> f32 {
    let total: f64 = trail_map.iter().map(|&v| v.max(0.0) as f64).sum();
    if total < 1e-10 {
        return 0.0;
    }

    let mut entropy = 0.0f64;
    for &value in trail_map {
        if value > 0.0 {
            let p = value as f64 / total;
            entropy -= p * p.ln();
        }
    }

    // Normalize by maximum entropy (uniform distribution)
    let max_entropy = (trail_map.len() as f64).ln();
    (entropy / max_entropy) as f32
}

/// Compute frame-to-frame correlation (temporal stability).
fn compute_temporal_stability(current: &[f32], previous: &[f32]) -> f32 {
    if current.len() != previous.len() || current.is_empty() {
        return 0.0;
    }

    // Pearson correlation coefficient
    let n = current.len() as f64;
    let sum_x: f64 = current.iter().map(|&v| v as f64).sum();
    let sum_y: f64 = previous.iter().map(|&v| v as f64).sum();
    let sum_xy: f64 = current
        .iter()
        .zip(previous.iter())
        .map(|(&x, &y)| x as f64 * y as f64)
        .sum();
    let sum_x2: f64 = current.iter().map(|&v| (v as f64).powi(2)).sum();
    let sum_y2: f64 = previous.iter().map(|&v| (v as f64).powi(2)).sum();

    let numerator = n * sum_xy - sum_x * sum_y;
    let denominator = ((n * sum_x2 - sum_x.powi(2)) * (n * sum_y2 - sum_y.powi(2))).sqrt();

    if denominator < 1e-10 {
        1.0
    } else {
        (numerator / denominator).clamp(-1.0, 1.0) as f32
    }
}

/// Compute variance of trail density.
fn compute_density_variance(trail_map: &[f32]) -> f32 {
    if trail_map.is_empty() {
        return 0.0;
    }

    let mean = trail_map.iter().map(|&v| v as f64).sum::<f64>() / trail_map.len() as f64;
    let variance = trail_map
        .iter()
        .map(|&v| (v as f64 - mean).powi(2))
        .sum::<f64>()
        / trail_map.len() as f64;

    variance.sqrt() as f32
}

/// Compute mean trail intensity.
fn compute_mean_intensity(trail_map: &[f32]) -> f32 {
    if trail_map.is_empty() {
        return 0.0;
    }
    trail_map.iter().sum::<f32>() / trail_map.len() as f32
}

/// Compute fraction of grid with non-zero trail.
fn compute_coverage(trail_map: &[f32]) -> f32 {
    if trail_map.is_empty() {
        return 0.0;
    }
    let threshold = 0.01;
    let count = trail_map.iter().filter(|&&v| v > threshold).count();
    count as f32 / trail_map.len() as f32
}

/// Estimate branching factor by counting junction points.
fn compute_branching_factor(trail_map: &[f32], width: usize, height: usize) -> f32 {
    let threshold = 0.1;
    let mut junction_count = 0u32;
    let mut trail_count = 0u32;

    for y in 1..height - 1 {
        for x in 1..width - 1 {
            let idx = y * width + x;
            if trail_map[idx] < threshold {
                continue;
            }
            trail_count += 1;

            // Count neighbors above threshold
            let mut neighbor_count = 0u8;
            let neighbors = [
                (x - 1, y),     // left
                (x + 1, y),     // right
                (x, y - 1),     // up
                (x, y + 1),     // down
                (x - 1, y - 1), // top-left
                (x + 1, y - 1), // top-right
                (x - 1, y + 1), // bottom-left
                (x + 1, y + 1), // bottom-right
            ];

            for (nx, ny) in neighbors {
                if trail_map[ny * width + nx] >= threshold {
                    neighbor_count += 1;
                }
            }

            // Junction = point with 3+ distinct trail branches
            if neighbor_count >= 3 {
                junction_count += 1;
            }
        }
    }

    if trail_count == 0 {
        0.0
    } else {
        junction_count as f32 / trail_count as f32 * 100.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::PI;

    fn make_test_agents(n: usize, heading: f32) -> Vec<Agent> {
        (0..n)
            .map(|i| Agent {
                x: 200.0 + (i as f32 * 0.1),
                y: 200.0,
                heading,
                species_id: 0,
            })
            .collect()
    }

    #[test]
    fn test_heading_variance_uniform() {
        // All agents same direction -> variance near 0
        let agents = make_test_agents(100, 0.0);
        let variance = compute_heading_variance(&agents);
        assert!(variance < 0.01, "Expected low variance, got {}", variance);
    }

    #[test]
    fn test_heading_variance_random() {
        // Agents pointing in all directions -> variance near 1
        let agents: Vec<Agent> = (0..100)
            .map(|i| Agent {
                x: 200.0,
                y: 200.0,
                heading: (i as f32 / 100.0) * 2.0 * PI,
                species_id: 0,
            })
            .collect();
        let variance = compute_heading_variance(&agents);
        assert!(variance > 0.9, "Expected high variance, got {}", variance);
    }

    #[test]
    fn test_fragmentation_single_blob() {
        let mut trail = vec![0.0; 100];
        // Create one connected region
        for i in 40..60 {
            trail[i] = 1.0;
        }
        let frags = compute_fragmentation(&trail, 10, 10);
        assert_eq!(frags, 1);
    }

    #[test]
    fn test_fragmentation_multiple_blobs() {
        let mut trail = vec![0.0; 100];
        // Create two separate regions
        trail[11] = 1.0;
        trail[12] = 1.0;
        trail[88] = 1.0;
        trail[89] = 1.0;
        let frags = compute_fragmentation(&trail, 10, 10);
        assert_eq!(frags, 2);
    }

    #[test]
    fn test_coverage() {
        let mut trail = vec![0.0; 100];
        for i in 0..25 {
            trail[i] = 1.0;
        }
        let cov = compute_coverage(&trail);
        assert!((cov - 0.25).abs() < 0.01);
    }

    #[test]
    fn test_temporal_stability_identical() {
        let trail = vec![1.0; 100];
        let stability = compute_temporal_stability(&trail, &trail);
        assert!((stability - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_angular_momentum_rotating() {
        // Agents moving tangentially around center
        let agents: Vec<Agent> = (0..8)
            .map(|i| {
                let angle = (i as f32 / 8.0) * 2.0 * PI;
                let r = 50.0;
                Agent {
                    x: 50.0 + r * angle.cos(),
                    y: 50.0 + r * angle.sin(),
                    heading: angle + PI / 2.0, // tangent direction (counterclockwise)
                    species_id: 0,
                }
            })
            .collect();
        let am = compute_angular_momentum(&agents, 100, 100);
        assert!(am > 0.0, "Expected positive angular momentum, got {}", am);
    }
}
