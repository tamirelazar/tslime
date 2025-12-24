#[allow(dead_code)]
pub struct DownsampledFrame {
    width: usize,
    height: usize,
    cells: Vec<Cell>,
}

#[derive(Clone, Copy)]
pub struct Cell {
    pub top: f32,
    pub bottom: f32,
}

impl DownsampledFrame {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            cells: vec![
                Cell {
                    top: 0.0,
                    bottom: 0.0
                };
                width * height
            ],
        }
    }

    #[allow(dead_code)]
    pub fn width(&self) -> usize {
        self.width
    }

    #[allow(dead_code)]
    pub fn height(&self) -> usize {
        self.height
    }

    pub fn cells(&self) -> &[Cell] {
        &self.cells
    }

    #[allow(dead_code)]
    pub fn get(&self, x: usize, y: usize) -> Cell {
        if x < self.width && y < self.height {
            self.cells[y * self.width + x]
        } else {
            Cell {
                top: 0.0,
                bottom: 0.0,
            }
        }
    }
}

pub fn downsample(
    trail_map: &[f32],
    sim_width: usize,
    sim_height: usize,
    term_width: usize,
    term_height: usize,
) -> DownsampledFrame {
    let mut frame = DownsampledFrame::new(term_width, term_height);

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

            frame.cells[cy * term_width + cx] = Cell {
                top: top_brightness,
                bottom: bottom_brightness,
            };
        }
    }

    frame
}

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
        let frame = downsample(&trail_map, 100, 100, 100, 50);

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

        let frame = downsample(&trail_map, 4, 4, 2, 2);

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

        let frame = downsample(&modified, 100, 100, 100, 50);

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
        let frame = downsample(&trail_map, 400, 400, 80, 24);

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
    fn test_downsample_reproducibility() {
        let trail_map: Vec<f32> = (0..40000).map(|i| (i % 100) as f32 / 100.0).collect();
        let frame1 = downsample(&trail_map, 200, 200, 100, 50);
        let frame2 = downsample(&trail_map, 200, 200, 100, 50);

        assert_eq!(frame1.cells().len(), frame2.cells().len());
        for (c1, c2) in frame1.cells().iter().zip(frame2.cells().iter()) {
            assert_eq!(c1.top, c2.top);
            assert_eq!(c1.bottom, c2.bottom);
        }
    }
}
