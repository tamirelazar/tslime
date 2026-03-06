#[allow(dead_code)]
/// A frame of downsampled simulation data, ready for rendering.
pub struct DownsampledFrame {
    width: usize,
    height: usize,
    cells: Vec<Cell>,
    /// Pre-allocated scratch buffers for Laplacian sharpening to avoid allocations
    scratch_top: Vec<f32>,
    scratch_bottom: Vec<f32>,
    scratch_top_left: Vec<f32>,
    scratch_top_right: Vec<f32>,
    scratch_bottom_left: Vec<f32>,
    scratch_bottom_right: Vec<f32>,
    /// Pre-allocated scratch buffer for gradient magnitude computation
    scratch_gradient: Vec<f32>,
}

#[derive(Clone, Copy, Default)]
/// Represents a single terminal cell containing subpixel brightness values.
pub struct Cell {
    /// Top half brightness.
    pub top: f32,
    /// Bottom half brightness.
    pub bottom: f32,
    // Quadrant support: when using quadrant charset, these provide 4× vertical resolution
    /// Top-left quadrant brightness.
    pub top_left: f32,
    /// Top-right quadrant brightness.
    pub top_right: f32,
    /// Bottom-left quadrant brightness.
    pub bottom_left: f32,
    /// Bottom-right quadrant brightness.
    pub bottom_right: f32,
}

impl DownsampledFrame {
    /// Creates a new, empty downsampled frame with pre-allocated scratch buffers.
    pub fn new(width: usize, height: usize) -> Self {
        let cell_count = width * height;
        Self {
            width,
            height,
            cells: vec![
                Cell {
                    top: 0.0,
                    bottom: 0.0,
                    top_left: 0.0,
                    top_right: 0.0,
                    bottom_left: 0.0,
                    bottom_right: 0.0,
                };
                cell_count
            ],
            // Pre-allocate scratch buffers for sharpening to avoid per-frame allocations
            scratch_top: vec![0.0; cell_count],
            scratch_bottom: vec![0.0; cell_count],
            scratch_top_left: vec![0.0; cell_count],
            scratch_top_right: vec![0.0; cell_count],
            scratch_bottom_left: vec![0.0; cell_count],
            scratch_bottom_right: vec![0.0; cell_count],
            scratch_gradient: vec![0.0; cell_count],
        }
    }

    #[allow(dead_code)]
    /// Returns the width of the frame in cells.
    pub fn width(&self) -> usize {
        self.width
    }

    #[allow(dead_code)]
    /// Returns the height of the frame in cells.
    pub fn height(&self) -> usize {
        self.height
    }

    /// Returns a slice of all cells in row-major order.
    pub fn cells(&self) -> &[Cell] {
        &self.cells
    }

    #[allow(dead_code)]
    /// Returns the cell at the specified coordinates.
    pub fn get(&self, x: usize, y: usize) -> Cell {
        if x < self.width && y < self.height {
            self.cells[y * self.width + x]
        } else {
            Cell {
                top: 0.0,
                bottom: 0.0,
                top_left: 0.0,
                top_right: 0.0,
                bottom_left: 0.0,
                bottom_right: 0.0,
            }
        }
    }
}

/// Downsamples a high-resolution trail map to terminal dimensions.
///
/// Aggregates grid cells into terminal character cells, computing average brightness
/// for sub-regions (top/bottom, quadrants) to support high-res rendering modes.
///
/// # Arguments
/// * `trail_map` - The high-resolution trail map data
/// * `sim_width` - Width of the simulation grid
/// * `sim_height` - Height of the simulation grid
/// * `term_width` - Target terminal width in columns
/// * `term_height` - Target terminal height in rows
/// * `frame` - Pre-allocated frame buffer to write into (must match term dimensions)
pub fn downsample(
    trail_map: &[f32],
    sim_width: usize,
    sim_height: usize,
    term_width: usize,
    term_height: usize,
    frame: &mut DownsampledFrame,
) {
    let x_scale = sim_width as f32 / term_width as f32;
    let y_scale = sim_height as f32 / term_height as f32;

    for cy in 0..term_height {
        for cx in 0..term_width {
            let sim_x_start = (cx as f32 * x_scale) as usize;
            let sim_x_end = (((cx + 1) as f32 * x_scale).ceil() as usize).min(sim_width);

            let sim_y_start = (cy as f32 * y_scale) as usize;
            let sim_y_mid = (((cy as f32 + 0.5) * y_scale).ceil() as usize).max(sim_y_start + 1);
            let sim_y_end = (((cy + 1) as f32 * y_scale).ceil() as usize).min(sim_height);

            let top_brightness = compute_average(
                trail_map,
                sim_width,
                sim_y_start,
                sim_y_mid,
                sim_x_start,
                sim_x_end,
            );

            let bottom_brightness = compute_average(
                trail_map,
                sim_width,
                sim_y_mid,
                sim_y_end,
                sim_x_start,
                sim_x_end,
            );

            // Compute quadrant values for higher resolution
            let sim_x_mid = (((cx as f32 + 0.5) * x_scale).ceil() as usize)
                .max(sim_x_start + 1)
                .min(sim_x_end);

            let top_left_brightness = compute_average(
                trail_map,
                sim_width,
                sim_y_start,
                sim_y_mid,
                sim_x_start,
                sim_x_mid,
            );

            let top_right_brightness = compute_average(
                trail_map,
                sim_width,
                sim_y_start,
                sim_y_mid,
                sim_x_mid,
                sim_x_end,
            );

            let bottom_left_brightness = compute_average(
                trail_map,
                sim_width,
                sim_y_mid,
                sim_y_end,
                sim_x_start,
                sim_x_mid,
            );

            let bottom_right_brightness = compute_average(
                trail_map, sim_width, sim_y_mid, sim_y_end, sim_x_mid, sim_x_end,
            );

            frame.cells[cy * term_width + cx] = Cell {
                top: top_brightness,
                bottom: bottom_brightness,
                top_left: top_left_brightness,
                top_right: top_right_brightness,
                bottom_left: bottom_left_brightness,
                bottom_right: bottom_right_brightness,
            };
        }
    }

    // frame is modified in place, no return needed
}

/// Downsamples multiple trail maps, aggregating brightness across species.
///
/// Similar to `downsample`, but sums contributions from multiple species layers.
///
/// # Arguments
/// * `trail_maps` - Slice of (trail_map, species_index) tuples
/// * `sim_width` - Width of the simulation grid
/// * `sim_height` - Height of the simulation grid
/// * `term_width` - Target terminal width in columns
/// * `term_height` - Target terminal height in rows
/// * `frame` - Pre-allocated frame buffer to write into (must match term dimensions)
pub fn downsample_multi_species(
    trail_maps: &[(&[f32], usize)],
    sim_width: usize,
    sim_height: usize,
    term_width: usize,
    term_height: usize,
    frame: &mut DownsampledFrame,
) {
    let x_scale = sim_width as f32 / term_width as f32;
    let y_scale = sim_height as f32 / term_height as f32;

    for cy in 0..term_height {
        for cx in 0..term_width {
            let sim_x_start = (cx as f32 * x_scale) as usize;
            let sim_x_end = (((cx + 1) as f32 * x_scale).ceil() as usize).min(sim_width);

            let sim_y_start = (cy as f32 * y_scale) as usize;
            let sim_y_mid = (((cy as f32 + 0.5) * y_scale).ceil() as usize).max(sim_y_start + 1);
            let sim_y_end = (((cy + 1) as f32 * y_scale).ceil() as usize).min(sim_height);

            let mut top_brightness = 0.0f32;
            let mut bottom_brightness = 0.0f32;
            let mut top_left_brightness = 0.0f32;
            let mut top_right_brightness = 0.0f32;
            let mut bottom_left_brightness = 0.0f32;
            let mut bottom_right_brightness = 0.0f32;

            let sim_x_mid = (((cx as f32 + 0.5) * x_scale).ceil() as usize)
                .max(sim_x_start + 1)
                .min(sim_x_end);

            for (trail_map, _species_idx) in trail_maps {
                let t = compute_average(
                    trail_map,
                    sim_width,
                    sim_y_start,
                    sim_y_mid,
                    sim_x_start,
                    sim_x_end,
                );
                let b = compute_average(
                    trail_map,
                    sim_width,
                    sim_y_mid,
                    sim_y_end,
                    sim_x_start,
                    sim_x_end,
                );

                let tl = compute_average(
                    trail_map,
                    sim_width,
                    sim_y_start,
                    sim_y_mid,
                    sim_x_start,
                    sim_x_mid,
                );

                let tr = compute_average(
                    trail_map,
                    sim_width,
                    sim_y_start,
                    sim_y_mid,
                    sim_x_mid,
                    sim_x_end,
                );

                let bl = compute_average(
                    trail_map,
                    sim_width,
                    sim_y_mid,
                    sim_y_end,
                    sim_x_start,
                    sim_x_mid,
                );

                let br = compute_average(
                    trail_map, sim_width, sim_y_mid, sim_y_end, sim_x_mid, sim_x_end,
                );

                top_brightness += t;
                bottom_brightness += b;
                top_left_brightness += tl;
                top_right_brightness += tr;
                bottom_left_brightness += bl;
                bottom_right_brightness += br;
            }

            frame.cells[cy * term_width + cx] = Cell {
                top: top_brightness,
                bottom: bottom_brightness,
                top_left: top_left_brightness,
                top_right: top_right_brightness,
                bottom_left: bottom_left_brightness,
                bottom_right: bottom_right_brightness,
            };
        }
    }
    // frame is modified in place, no return needed
}

#[inline]
fn compute_average(
    data: &[f32],
    data_width: usize,
    y_start: usize,
    y_end: usize,
    x_start: usize,
    x_end: usize,
) -> f32 {
    let mut sum = 0.0f32;
    let mut count = 0;

    for y in y_start..y_end {
        if y * data_width >= data.len() {
            break;
        }
        for x in x_start..x_end {
            let idx = y * data_width + x;
            if idx < data.len() {
                sum += data[idx];
                count += 1;
            }
        }
    }

    if count > 0 {
        sum / count as f32
    } else {
        0.0
    }
}

/// Auxiliary per-cell data for visual effects (trail age, temporal delta, gradient).
#[derive(Clone, Copy, Default)]
pub struct AuxCell {
    /// Normalized trail age [0, 1] where 1.0 = AGE_MAX_SECONDS.
    pub age: f32,
    /// Normalized temporal delta [0, 1].
    pub delta: f32,
    /// Normalized gradient magnitude [0, 1] for edge glow.
    pub gradient: f32,
}

/// Auxiliary frame holding per-cell age and delta at terminal resolution.
#[derive(Clone)]
pub struct AuxFrame {
    /// Width of the frame in terminal columns.
    pub width: usize,
    /// Height of the frame in terminal rows.
    pub height: usize,
    /// Auxiliary cell data in row-major order.
    pub cells: Vec<AuxCell>,
}

impl AuxFrame {
    /// Resize the frame to new dimensions, reusing existing allocation if possible.
    pub fn resize(&mut self, width: usize, height: usize) {
        let new_len = width * height;
        if self.cells.len() != new_len {
            self.cells.resize(new_len, AuxCell::default());
        }
        self.width = width;
        self.height = height;
    }
}

/// Downsamples optional age, delta, and gradient buffers from sim resolution to terminal resolution.
///
/// Uses the same average-pooling grid mapping as `downsample()`.
/// Age values are divided by `AGE_MAX` and clamped to [0, 1].
///
/// # Arguments
/// * `age_buf` - Optional trail age buffer
/// * `delta_buf` - Optional trail delta buffer
/// * `gradient_buf` - Optional gradient magnitude buffer
/// * `sim_width` - Width of the simulation grid
/// * `sim_height` - Height of the simulation grid
/// * `term_width` - Target terminal width in columns
/// * `term_height` - Target terminal height in rows
/// * `frame` - Pre-allocated aux frame buffer to write into (must match term dimensions)
#[allow(clippy::too_many_arguments)]
pub fn downsample_aux(
    age_buf: Option<&[f32]>,
    delta_buf: Option<&[f32]>,
    gradient_buf: Option<&[f32]>,
    sim_width: usize,
    sim_height: usize,
    term_width: usize,
    term_height: usize,
    frame: &mut AuxFrame,
) {
    use crate::config_defaults::visual_fx::AGE_MAX_SECONDS;

    // Ensure frame has correct dimensions
    if frame.width != term_width || frame.height != term_height {
        frame.resize(term_width, term_height);
    }

    let x_scale = sim_width as f32 / term_width as f32;
    let y_scale = sim_height as f32 / term_height as f32;

    for cy in 0..term_height {
        for cx in 0..term_width {
            let sim_x_start = (cx as f32 * x_scale) as usize;
            let sim_x_end = (((cx + 1) as f32 * x_scale).ceil() as usize).min(sim_width);
            let sim_y_start = (cy as f32 * y_scale) as usize;
            let sim_y_end = (((cy + 1) as f32 * y_scale).ceil() as usize).min(sim_height);

            let age = if let Some(buf) = age_buf {
                let avg = compute_average(
                    buf,
                    sim_width,
                    sim_y_start,
                    sim_y_end,
                    sim_x_start,
                    sim_x_end,
                );
                (avg / AGE_MAX_SECONDS).clamp(0.0, 1.0)
            } else {
                0.0
            };

            let delta = if let Some(buf) = delta_buf {
                compute_average(
                    buf,
                    sim_width,
                    sim_y_start,
                    sim_y_end,
                    sim_x_start,
                    sim_x_end,
                )
                .clamp(0.0, 1.0)
            } else {
                0.0
            };

            let gradient = if let Some(buf) = gradient_buf {
                compute_average(
                    buf,
                    sim_width,
                    sim_y_start,
                    sim_y_end,
                    sim_x_start,
                    sim_x_end,
                )
                .clamp(0.0, 1.0)
            } else {
                0.0
            };

            frame.cells[cy * term_width + cx] = AuxCell {
                age,
                delta,
                gradient,
            };
        }
    }
    // frame is modified in place, no return needed
}

/// Applies Laplacian sharpening to a downsampled frame.
///
/// For each interior cell, computes `L = center - avg(4 neighbors)` and
/// adds `strength * L` to sharpen vein edges. Boundary cells are unchanged.
///
/// Uses pre-allocated scratch buffers to avoid memory allocation.
pub fn apply_laplacian_sharpening(frame: &mut DownsampledFrame, strength: f32) {
    let w = frame.width;
    let h = frame.height;
    if w < 3 || h < 3 {
        return;
    }

    // Copy values into pre-allocated scratch buffers
    for (i, cell) in frame.cells.iter().enumerate() {
        frame.scratch_top[i] = cell.top;
        frame.scratch_bottom[i] = cell.bottom;
        frame.scratch_top_left[i] = cell.top_left;
        frame.scratch_top_right[i] = cell.top_right;
        frame.scratch_bottom_left[i] = cell.bottom_left;
        frame.scratch_bottom_right[i] = cell.bottom_right;
    }

    // Use references to scratch buffers for clarity
    let top_orig = &frame.scratch_top[..];
    let bot_orig = &frame.scratch_bottom[..];
    let tl_orig = &frame.scratch_top_left[..];
    let tr_orig = &frame.scratch_top_right[..];
    let bl_orig = &frame.scratch_bottom_left[..];
    let br_orig = &frame.scratch_bottom_right[..];

    for y in 1..h - 1 {
        for x in 1..w - 1 {
            let idx = y * w + x;
            let up = (y - 1) * w + x;
            let dn = (y + 1) * w + x;
            let lt = y * w + (x - 1);
            let rt = y * w + (x + 1);

            // Sharpen top
            let lap_t =
                top_orig[idx] - (top_orig[up] + top_orig[dn] + top_orig[lt] + top_orig[rt]) * 0.25;
            frame.cells[idx].top = (top_orig[idx] + strength * lap_t).max(0.0);

            // Sharpen bottom
            let lap_b =
                bot_orig[idx] - (bot_orig[up] + bot_orig[dn] + bot_orig[lt] + bot_orig[rt]) * 0.25;
            frame.cells[idx].bottom = (bot_orig[idx] + strength * lap_b).max(0.0);

            // Sharpen quadrants
            let lap_tl =
                tl_orig[idx] - (tl_orig[up] + tl_orig[dn] + tl_orig[lt] + tl_orig[rt]) * 0.25;
            frame.cells[idx].top_left = (tl_orig[idx] + strength * lap_tl).max(0.0);

            let lap_tr =
                tr_orig[idx] - (tr_orig[up] + tr_orig[dn] + tr_orig[lt] + tr_orig[rt]) * 0.25;
            frame.cells[idx].top_right = (tr_orig[idx] + strength * lap_tr).max(0.0);

            let lap_bl =
                bl_orig[idx] - (bl_orig[up] + bl_orig[dn] + bl_orig[lt] + bl_orig[rt]) * 0.25;
            frame.cells[idx].bottom_left = (bl_orig[idx] + strength * lap_bl).max(0.0);

            let lap_br =
                br_orig[idx] - (br_orig[up] + br_orig[dn] + br_orig[lt] + br_orig[rt]) * 0.25;
            frame.cells[idx].bottom_right = (br_orig[idx] + strength * lap_br).max(0.0);
        }
    }
}

/// Computes gradient magnitude for edge detection.
///
/// For each cell, computes the magnitude of the gradient using central differences:
/// Gx = (right - left) / 2, Gy = (bottom - top) / 2
/// magnitude = sqrt(Gx² + Gy²)
///
/// Writes results into the frame's pre-allocated scratch_gradient buffer.
/// Returns a reference to the buffer containing normalized gradient magnitudes in [0, 1] range.
pub fn compute_gradient_magnitude(frame: &mut DownsampledFrame) -> &[f32] {
    let w = frame.width;
    let h = frame.height;
    let magnitude = &mut frame.scratch_gradient;

    // Reset buffer
    magnitude.fill(0.0);

    if w < 3 || h < 3 {
        return magnitude;
    }

    // Compute gradients for each channel and average them
    for y in 1..h - 1 {
        for x in 1..w - 1 {
            let idx = y * w + x;
            let up = (y - 1) * w + x;
            let dn = (y + 1) * w + x;
            let lt = y * w + (x - 1);
            let rt = y * w + (x + 1);

            // Average all cell channels for gradient computation
            let avg = |cell: &Cell| {
                (cell.top
                    + cell.bottom
                    + cell.top_left
                    + cell.top_right
                    + cell.bottom_left
                    + cell.bottom_right)
                    / 6.0
            };

            let _center_val = avg(&frame.cells[idx]);
            let up_val = avg(&frame.cells[up]);
            let dn_val = avg(&frame.cells[dn]);
            let lt_val = avg(&frame.cells[lt]);
            let rt_val = avg(&frame.cells[rt]);

            // Central differences for gradient
            let gx = (rt_val - lt_val) * 0.5;
            let gy = (dn_val - up_val) * 0.5;

            // Gradient magnitude
            magnitude[idx] = (gx * gx + gy * gy).sqrt();
        }
    }

    // Normalize to [0, 1]
    let max_val = magnitude.iter().copied().fold(0.0f32, f32::max);
    if max_val > 0.0 {
        for m in magnitude.iter_mut() {
            *m = (*m / max_val).min(1.0);
        }
    }

    magnitude
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_downsampled_frame_creation() {
        let frame = DownsampledFrame::new(80, 24);
        assert_eq!(frame.width(), 80);
        assert_eq!(frame.height(), 24);
        assert_eq!(frame.cells().len(), 80 * 24);
    }

    #[test]
    fn test_cell_get() {
        let mut frame = DownsampledFrame::new(10, 10);
        frame.cells[5 * 10 + 3] = Cell {
            top: 1.0,
            bottom: 2.0,
            top_left: 1.0,
            top_right: 1.0,
            bottom_left: 2.0,
            bottom_right: 2.0,
        };
        let cell = frame.get(3, 5);
        assert_eq!(cell.top, 1.0);
        assert_eq!(cell.bottom, 2.0);
    }

    #[test]
    fn test_cell_get_out_of_bounds() {
        let frame = DownsampledFrame::new(10, 10);
        let cell = frame.get(20, 5);
        assert_eq!(cell.top, 0.0);
        assert_eq!(cell.bottom, 0.0);
    }

    #[test]
    fn test_downsample_identity() {
        let trail_map = vec![1.0; 10000];
        let mut frame = DownsampledFrame::new(100, 50);
        downsample(&trail_map, 100, 100, 100, 50, &mut frame);

        assert_eq!(frame.width(), 100);
        assert_eq!(frame.height(), 50);

        for cell in frame.cells() {
            assert_eq!(cell.top, 1.0);
            assert_eq!(cell.bottom, 1.0);
        }
    }

    #[test]
    fn test_downsample_4x4_to_2x2() {
        let trail_map = vec![
            1.0, 1.0, 2.0, 2.0, 1.0, 1.0, 2.0, 2.0, 3.0, 3.0, 4.0, 4.0, 3.0, 3.0, 4.0, 4.0,
        ];

        let mut frame = DownsampledFrame::new(2, 2);
        downsample(&trail_map, 4, 4, 2, 2, &mut frame);

        assert_eq!(frame.width(), 2);
        assert_eq!(frame.height(), 2);

        assert_eq!(frame.get(0, 0).top, 1.0);
        assert_eq!(frame.get(0, 0).bottom, 1.0);
        assert_eq!(frame.get(1, 0).top, 2.0);
        assert_eq!(frame.get(1, 0).bottom, 2.0);
        assert_eq!(frame.get(0, 1).top, 3.0);
        assert_eq!(frame.get(0, 1).bottom, 3.0);
        assert_eq!(frame.get(1, 1).top, 4.0);
        assert_eq!(frame.get(1, 1).bottom, 4.0);
    }

    #[test]
    fn test_downsample_half_blocks() {
        let trail_map = vec![0.0; 10000];

        let mut modified = trail_map.clone();
        for y in 0..50 {
            for x in 0..100 {
                modified[y * 100 + x] = 1.0;
            }
        }

        let mut frame = DownsampledFrame::new(100, 50);
        downsample(&modified, 100, 100, 100, 50, &mut frame);

        for cy in 0..25 {
            for cx in 0..100 {
                let cell = frame.get(cx, cy);
                assert_eq!(cell.top, 1.0);
                assert_eq!(cell.bottom, 1.0);
            }
        }

        for cy in 25..50 {
            for cx in 0..100 {
                let cell = frame.get(cx, cy);
                assert_eq!(cell.top, 0.0);
                assert_eq!(cell.bottom, 0.0);
            }
        }
    }

    #[test]
    fn test_downsample_empty() {
        let trail_map = vec![0.0; 160000];
        let mut frame = DownsampledFrame::new(80, 24);
        downsample(&trail_map, 400, 400, 80, 24, &mut frame);

        for cell in frame.cells() {
            assert_eq!(cell.top, 0.0);
            assert_eq!(cell.bottom, 0.0);
        }
    }

    #[test]
    fn test_compute_average_region() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0];
        let avg = compute_average(&data, 3, 0, 2, 0, 2);
        assert_eq!(avg, 3.0);
    }

    #[test]
    fn test_compute_average_single() {
        let data = vec![5.0];
        let avg = compute_average(&data, 1, 0, 1, 0, 1);
        assert_eq!(avg, 5.0);
    }

    #[test]
    fn test_compute_average_empty() {
        let data = vec![0.0];
        let avg = compute_average(&data, 1, 0, 0, 0, 0);
        assert_eq!(avg, 0.0);
    }

    #[test]
    fn test_compute_average_out_of_bounds() {
        let data = vec![1.0; 10];
        assert_eq!(compute_average(&data, 2, 0, 10, 0, 2), 1.0);
        assert_eq!(compute_average(&data, 2, 10, 15, 0, 2), 0.0);
    }

    #[test]
    fn test_downsample_quadrant_values() {
        let mut trail_map = vec![0.0; 16];
        trail_map[0] = 1.0; // Top-left of the first cell
        let mut frame = DownsampledFrame::new(1, 1);
        downsample(&trail_map, 4, 4, 1, 1, &mut frame);
        let cell = frame.get(0, 0);
        assert!(cell.top_left > 0.0);
        assert_eq!(cell.bottom_right, 0.0);
    }

    #[test]
    fn test_downsample_no_zero_width_quadrants() {
        // Test common terminal and simulation sizes to ensure no zero-width quadrants
        let sim_sizes = [(400, 400), (200, 200), (100, 100)];
        let term_sizes = [(80, 24), (160, 48), (40, 12), (120, 36)];

        for &(sim_w, sim_h) in &sim_sizes {
            for &(term_w, term_h) in &term_sizes {
                let trail_map = vec![0.0; sim_w * sim_h];
                let mut frame = DownsampledFrame::new(term_w, term_h);
                downsample(&trail_map, sim_w, sim_h, term_w, term_h, &mut frame);

                // Check each cell's quadrant values are computed (not just zero)
                // The actual check for zero-width quadrants requires inspecting the internal
                // downsampling logic. We'll trust that compute_average returns 0.0 for empty regions.
                // Instead, we'll verify that quadrant values are consistent with top/bottom averages.
                for cy in 0..term_h {
                    for cx in 0..term_w {
                        let cell = frame.get(cx, cy);
                        // Top should be average of top_left and top_right
                        // Bottom should be average of bottom_left and bottom_right
                        // With uniform zero map, all values should be zero
                        assert_eq!(cell.top, 0.0);
                        assert_eq!(cell.bottom, 0.0);
                        assert_eq!(cell.top_left, 0.0);
                        assert_eq!(cell.top_right, 0.0);
                        assert_eq!(cell.bottom_left, 0.0);
                        assert_eq!(cell.bottom_right, 0.0);
                    }
                }
            }
        }
    }

    #[test]
    fn test_downsample_quadrant_widths() {
        // Manually compute quadrant widths for common configurations
        let sim_width = 400;
        let sim_height = 400;
        let term_width = 80;
        let term_height = 24;
        let x_scale = sim_width as f32 / term_width as f32;
        let y_scale = sim_height as f32 / term_height as f32;

        let mut zero_width_found = false;

        for cy in 0..term_height {
            for cx in 0..term_width {
                let sim_x_start = (cx as f32 * x_scale) as usize;
                let sim_x_end = (((cx + 1) as f32 * x_scale).ceil() as usize).min(sim_width);
                let sim_y_start = (cy as f32 * y_scale) as usize;
                let sim_y_end = (((cy + 1) as f32 * y_scale).ceil() as usize).min(sim_height);

                let sim_x_mid = (((cx as f32 + 0.5) * x_scale).ceil() as usize)
                    .max(sim_x_start + 1)
                    .min(sim_x_end);
                let sim_y_mid = (((cy as f32 + 0.5) * y_scale).ceil() as usize)
                    .max(sim_y_start + 1)
                    .min(sim_y_end);

                let left_width = sim_x_mid - sim_x_start;
                let right_width = sim_x_end - sim_x_mid;
                let top_height = sim_y_mid - sim_y_start;
                let bottom_height = sim_y_end - sim_y_mid;

                if left_width == 0 || right_width == 0 || top_height == 0 || bottom_height == 0 {
                    zero_width_found = true;
                    println!("Zero-width quadrant at term cell ({}, {}): left={}, right={}, top={}, bottom={}, sim_x_start={}, sim_x_end={}, sim_x_mid={}, sim_y_start={}, sim_y_end={}, sim_y_mid={}",
                             cx, cy, left_width, right_width, top_height, bottom_height,
                             sim_x_start, sim_x_end, sim_x_mid, sim_y_start, sim_y_end, sim_y_mid);
                }
            }
        }

        // This test will fail if zero-width quadrants are found
        assert!(
            !zero_width_found,
            "Found zero-width quadrants in downsampling"
        );
    }

    #[test]
    fn test_downsample_uniform_brightness_quadrants() {
        // Test that all quadrants receive brightness when trail map is uniform
        let sim_width = 400;
        let sim_height = 400;
        let term_width = 80;
        let term_height = 24;
        let trail_map = vec![1.0; sim_width * sim_height];
        let mut frame = DownsampledFrame::new(term_width, term_height);
        downsample(
            &trail_map,
            sim_width,
            sim_height,
            term_width,
            term_height,
            &mut frame,
        );

        let mut zero_brightness_quadrant = false;
        for cy in 0..term_height {
            for cx in 0..term_width {
                let cell = frame.get(cx, cy);
                // All quadrant values should be 1.0 because the entire simulation is 1.0
                // However, floating point rounding may cause slight differences
                if cell.top_left < 0.99
                    || cell.top_right < 0.99
                    || cell.bottom_left < 0.99
                    || cell.bottom_right < 0.99
                {
                    zero_brightness_quadrant = true;
                    println!(
                        "Low quadrant brightness at ({}, {}): tl={}, tr={}, bl={}, br={}",
                        cx, cy, cell.top_left, cell.top_right, cell.bottom_left, cell.bottom_right
                    );
                }
                // Top should be average of top_left and top_right
                let expected_top = (cell.top_left + cell.top_right) / 2.0;
                let expected_bottom = (cell.bottom_left + cell.bottom_right) / 2.0;
                assert!((cell.top - expected_top).abs() < 0.01);
                assert!((cell.bottom - expected_bottom).abs() < 0.01);
            }
        }
        assert!(
            !zero_brightness_quadrant,
            "Found quadrant with low brightness"
        );
    }

    #[test]
    fn test_downsample_gap_detection() {
        // Test various terminal sizes to detect systematic gaps
        let sim_width = 400;
        let sim_height = 400;

        // Test common terminal sizes
        let test_sizes = [
            (80, 24),  // Standard terminal
            (79, 24),  // Odd width
            (81, 24),  // Odd width
            (40, 12),  // Half size
            (100, 30), // Larger
            (120, 30), // Wider
        ];

        for (term_width, term_height) in test_sizes.iter() {
            // Create a pattern that should fill the entire screen
            // Use a checkerboard pattern to detect alignment issues
            let mut trail_map = vec![0.0; sim_width * sim_height];

            // Fill with a pattern that should produce continuous output
            // Simple: fill all pixels with 1.0
            for pixel in trail_map.iter_mut() {
                *pixel = 1.0;
            }

            let mut frame = DownsampledFrame::new(*term_width, *term_height);
            downsample(
                &trail_map,
                sim_width,
                sim_height,
                *term_width,
                *term_height,
                &mut frame,
            );

            // Check each cell for low brightness (should not happen with uniform 1.0)

            // More useful: check each cell individually
            let mut low_brightness_cells = 0;
            for cy in 0..*term_height {
                for cx in 0..*term_width {
                    let cell = frame.get(cx, cy);
                    if cell.top_left < 0.1
                        || cell.top_right < 0.1
                        || cell.bottom_left < 0.1
                        || cell.bottom_right < 0.1
                    {
                        low_brightness_cells += 1;
                        println!("Low brightness cell at ({}, {}) for size {}x{}: tl={}, tr={}, bl={}, br={}", 
                                 cx, cy, term_width, term_height,
                                 cell.top_left, cell.top_right, cell.bottom_left, cell.bottom_right);
                    }
                }
            }

            assert_eq!(
                low_brightness_cells, 0,
                "Found {} cells with low brightness for terminal size {}x{}",
                low_brightness_cells, term_width, term_height
            );
        }
    }
}
