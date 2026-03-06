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

    // === NEW METRICS ===
    /// Local flow coherence - how aligned are nearby agents?
    /// High = locally coherent movement (worm-like), Low = chaotic/turbulent.
    pub flow_coherence: f32,

    /// Spatial concentration - how clustered is the trail distribution?
    /// High = concentrated (blob-like), Low = spread out.
    pub spatial_concentration: f32,

    /// Path continuity - average length of unbroken trail paths.
    /// High = long continuous trails (worm), Low = fragmented/scattered.
    pub path_continuity: f32,
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

        // New metrics
        let flow_coherence = compute_flow_coherence(agents, width, height);
        let spatial_concentration = compute_spatial_concentration(trail_map, width, height);
        let path_continuity = compute_path_continuity(trail_map, width, height);

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
            flow_coherence,
            spatial_concentration,
            path_continuity,
        }
    }

    /// Score for vortex-like behavior (swirling).
    /// Fixed: rotation should ALLOW heading variance (agents at different positions rotate)
    pub fn vortex_score(&self) -> f32 {
        // High angular momentum is key; heading variance is actually expected in vortices
        // since agents at different radial positions face different directions
        let am = self.angular_momentum.abs();
        // Boost score when there's moderate heading variance (0.3-0.7 is ideal for vortex)
        let variance_factor = 0.5 + self.heading_variance * 0.5;
        // Prefer moderate coverage (not too sparse, not too full)
        let coverage_factor = 1.0 - (self.coverage - 0.4).abs();
        am * variance_factor * coverage_factor.max(0.3)
    }

    /// Score for lightning-like behavior (branching dendrites).
    pub fn lightning_score(&self) -> f32 {
        // Want high branching, low coverage (sparse), high contrast
        let branching = self.branching_factor;
        let sparsity = (1.0 - self.coverage).powf(0.7); // Not too aggressive on sparsity
        let contrast = self.density_variance.sqrt();
        // Add elongation factor - lightning should have elongated structures
        let elongation_bonus = self.trail_elongation.sqrt().min(2.0);
        branching * sparsity * contrast * (0.5 + elongation_bonus * 0.25)
    }

    /// Score for crystal-like behavior (stable structures).
    /// Fixed: removed arbitrary 0.3 target, use stability + structure
    pub fn crystal_score(&self) -> f32 {
        let stability = self.temporal_stability.max(0.0);
        // Accept wider coverage range (0.1-0.6)
        let coverage_ok = if self.coverage > 0.1 && self.coverage < 0.6 {
            1.0
        } else {
            0.5
        };
        // Reward high density variance (structured patterns, not uniform)
        let structure = 1.0 + self.density_variance.sqrt();
        // Low fragmentation preferred (cohesive structure)
        let cohesion = 1.0 / (1.0 + (self.trail_fragmentation as f32) * 0.1);
        stability * coverage_ok * structure * cohesion
    }

    /// Score for blob-like behavior (isolated clusters).
    /// Fixed: stronger fragmentation weight, proper coverage term, uses new metrics
    pub fn blob_score(&self) -> f32 {
        // Key: multiple disconnected regions (high fragmentation)
        let frag = (self.trail_fragmentation as f32).powf(0.7);
        // Blobs should be round-ish, not elongated
        let anti_elongation = 1.0 / (1.0 + self.trail_elongation * 0.5);
        // Prefer moderate coverage (0.15-0.35) - enough to see blobs but not merged
        let coverage_deviation = (self.coverage - 0.25).abs();
        let coverage_term = (1.0 - coverage_deviation * 2.5).max(0.1);
        // High spatial concentration indicates clustered (blob) behavior
        let concentration_bonus = 0.5 + self.spatial_concentration * 0.5;
        // Low flow coherence is expected (agents in different blobs don't coordinate)
        let coherence_penalty = 1.0 - self.flow_coherence * 0.3;
        frag * anti_elongation * coverage_term * concentration_bonus * coherence_penalty.max(0.5)
    }

    /// Score for worm-like behavior (long snaking trails).
    /// Fixed: focus on elongation + low fragmentation + sparse, uses new metrics
    pub fn worm_score(&self) -> f32 {
        // Key: highly elongated structures
        let elongation_boost = self.trail_elongation.powf(1.3);
        // Few fragments (ideally 1-5 long worms, not many pieces)
        let anti_frag = 1.0 / (1.0 + self.trail_fragmentation as f32 * 0.3);
        // Should be sparse - worms are thin trails
        let sparse = (1.0 - self.coverage).powf(0.5);
        // High flow coherence - agents follow each other along trails
        let coherence_bonus = 0.7 + self.flow_coherence * 0.5;
        // High path continuity - long unbroken trails
        let continuity_bonus = 0.5 + self.path_continuity * 1.5;
        elongation_boost * anti_frag * sparse * coherence_bonus * continuity_bonus
    }

    /// Score for chaos-edge behavior (high sensitivity, dynamic patterns).
    pub fn chaos_score(&self) -> f32 {
        // High heading variance indicates chaotic/turbulent behavior
        let variance_term = self.heading_variance;
        // High density variance means interesting contrast/structure
        let contrast = self.density_variance;
        // Moderate temporal stability (not frozen, not completely random)
        let dynamism = 1.0 - (self.temporal_stability - 0.5).abs();
        // Moderate coverage
        let coverage_factor = 1.0 - (self.coverage - 0.4).abs();
        variance_term * contrast * dynamism.max(0.3) * coverage_factor.max(0.3)
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

/// Compute local flow coherence - how aligned are nearby agents?
/// Uses grid-based spatial binning to find local neighborhoods.
fn compute_flow_coherence(agents: &[Agent], width: usize, height: usize) -> f32 {
    if agents.len() < 10 {
        return 0.0;
    }

    // Divide grid into cells for local neighborhood detection
    let cell_size = 20.0f32; // 20x20 pixel cells
    let cells_x = (width as f32 / cell_size).ceil() as usize;
    let cells_y = (height as f32 / cell_size).ceil() as usize;

    // Build spatial hash map: cell -> list of agent indices
    let mut cell_agents: Vec<Vec<usize>> = vec![Vec::new(); cells_x * cells_y];
    for (i, agent) in agents.iter().enumerate() {
        let cx = ((agent.x / cell_size) as usize).min(cells_x - 1);
        let cy = ((agent.y / cell_size) as usize).min(cells_y - 1);
        cell_agents[cy * cells_x + cx].push(i);
    }

    // Compute local coherence for each cell with agents
    let mut total_coherence = 0.0f64;
    let mut cell_count = 0u32;

    for cell in &cell_agents {
        if cell.len() < 3 {
            continue; // Need at least 3 agents for meaningful coherence
        }

        // Compute mean direction vector for this cell
        let mut sum_cos = 0.0f64;
        let mut sum_sin = 0.0f64;
        for &agent_idx in cell {
            let h = agents[agent_idx].heading;
            sum_cos += h.cos() as f64;
            sum_sin += h.sin() as f64;
        }

        let n = cell.len() as f64;
        let r = ((sum_cos / n).powi(2) + (sum_sin / n).powi(2)).sqrt();

        // r is in [0, 1]: 0 = random directions, 1 = all same direction
        total_coherence += r;
        cell_count += 1;
    }

    if cell_count == 0 {
        0.0
    } else {
        (total_coherence / cell_count as f64) as f32
    }
}

/// Compute spatial concentration - how clustered is the trail distribution?
/// Uses variance of distance from center of mass.
fn compute_spatial_concentration(trail_map: &[f32], width: usize, height: usize) -> f32 {
    let threshold = 0.05;

    // Compute center of mass of trail
    let mut sum_x = 0.0f64;
    let mut sum_y = 0.0f64;
    let mut total_weight = 0.0f64;

    for y in 0..height {
        for x in 0..width {
            let val = trail_map[y * width + x];
            if val > threshold {
                sum_x += x as f64 * val as f64;
                sum_y += y as f64 * val as f64;
                total_weight += val as f64;
            }
        }
    }

    if total_weight < 1e-10 {
        return 0.0;
    }

    let cx = sum_x / total_weight;
    let cy = sum_y / total_weight;

    // Compute variance of weighted distances from center
    let mut variance_sum = 0.0f64;
    for y in 0..height {
        for x in 0..width {
            let val = trail_map[y * width + x];
            if val > threshold {
                let dx = x as f64 - cx;
                let dy = y as f64 - cy;
                let dist_sq = dx * dx + dy * dy;
                variance_sum += dist_sq * val as f64;
            }
        }
    }

    // Normalize by grid diagonal squared to get [0, 1] range
    let max_dist_sq = (width * width + height * height) as f64;
    let concentration = 1.0 - (variance_sum / total_weight / max_dist_sq).sqrt().min(1.0);

    concentration as f32
}

/// Compute path continuity - average length of unbroken trail paths.
/// Traces paths through the trail map.
fn compute_path_continuity(trail_map: &[f32], width: usize, height: usize) -> f32 {
    let threshold = 0.1;
    let mut visited = vec![false; trail_map.len()];
    let mut total_length = 0u32;
    let mut path_count = 0u32;

    // Find endpoints (cells with exactly 1 neighbor) and trace paths from them
    for start in 0..trail_map.len() {
        if visited[start] || trail_map[start] < threshold {
            continue;
        }

        let x = start % width;
        let y = start / width;

        // Count neighbors
        let neighbors = count_neighbors(trail_map, x, y, width, height, threshold);

        // Start from endpoints (1 neighbor) or junction points (3+ neighbors)
        // to trace distinct paths
        if neighbors != 2 && neighbors > 0 {
            // Trace path from this point
            let length = trace_path_length(trail_map, &mut visited, x, y, width, height, threshold);
            if length > 1 {
                total_length += length;
                path_count += 1;
            }
        }
    }

    // Also count isolated cells that weren't visited (single points or small clusters)
    for start in 0..trail_map.len() {
        if !visited[start] && trail_map[start] >= threshold {
            // Trace this remaining component
            let x = start % width;
            let y = start / width;
            let length = trace_path_length(trail_map, &mut visited, x, y, width, height, threshold);
            if length > 0 {
                total_length += length;
                path_count += 1;
            }
        }
    }

    if path_count == 0 {
        0.0
    } else {
        (total_length as f32 / path_count as f32).min(100.0) / 100.0
    }
}

/// Count 4-neighbors above threshold.
fn count_neighbors(
    trail_map: &[f32],
    x: usize,
    y: usize,
    width: usize,
    height: usize,
    threshold: f32,
) -> u8 {
    let mut count = 0u8;
    if x > 0 && trail_map[y * width + (x - 1)] >= threshold {
        count += 1;
    }
    if x < width - 1 && trail_map[y * width + (x + 1)] >= threshold {
        count += 1;
    }
    if y > 0 && trail_map[(y - 1) * width + x] >= threshold {
        count += 1;
    }
    if y < height - 1 && trail_map[(y + 1) * width + x] >= threshold {
        count += 1;
    }
    count
}

/// Trace path length from a starting point using DFS.
fn trace_path_length(
    trail_map: &[f32],
    visited: &mut [bool],
    start_x: usize,
    start_y: usize,
    width: usize,
    height: usize,
    threshold: f32,
) -> u32 {
    let mut stack = vec![(start_x, start_y)];
    let mut length = 0u32;

    while let Some((x, y)) = stack.pop() {
        let idx = y * width + x;
        if visited[idx] || trail_map[idx] < threshold {
            continue;
        }
        visited[idx] = true;
        length += 1;

        // Push unvisited neighbors
        if x > 0 {
            let neighbor_idx = y * width + (x - 1);
            if !visited[neighbor_idx] && trail_map[neighbor_idx] >= threshold {
                stack.push((x - 1, y));
            }
        }
        if x < width - 1 {
            let neighbor_idx = y * width + (x + 1);
            if !visited[neighbor_idx] && trail_map[neighbor_idx] >= threshold {
                stack.push((x + 1, y));
            }
        }
        if y > 0 {
            let neighbor_idx = (y - 1) * width + x;
            if !visited[neighbor_idx] && trail_map[neighbor_idx] >= threshold {
                stack.push((x, y - 1));
            }
        }
        if y < height - 1 {
            let neighbor_idx = (y + 1) * width + x;
            if !visited[neighbor_idx] && trail_map[neighbor_idx] >= threshold {
                stack.push((x, y + 1));
            }
        }
    }

    length
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
                progress: 0,
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
                progress: 0,
            })
            .collect();
        let variance = compute_heading_variance(&agents);
        assert!(variance > 0.9, "Expected high variance, got {}", variance);
    }

    #[test]
    fn test_fragmentation_single_blob() {
        let mut trail = vec![0.0; 100];
        // Create one connected region
        for v in trail.iter_mut().take(60).skip(40) {
            *v = 1.0;
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
        for v in trail.iter_mut().take(25) {
            *v = 1.0;
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
                    progress: 0,
                }
            })
            .collect();
        let am = compute_angular_momentum(&agents, 100, 100);
        assert!(am > 0.0, "Expected positive angular momentum, got {}", am);
    }
}
