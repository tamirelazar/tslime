pub struct ErrorDiffusion {
    width: usize,
    height: usize,
    error_buffer: Vec<f32>,
    frame_counter: usize,
    reset_interval: usize,
}

impl ErrorDiffusion {
    pub fn new(width: usize, height: usize, reset_interval: usize) -> Self {
        Self {
            width,
            height,
            error_buffer: vec![0.0; width * height],
            frame_counter: 0,
            reset_interval,
        }
    }

    pub fn reset(&mut self) {
        self.error_buffer.fill(0.0);
        self.frame_counter = 0;
    }

    pub fn tick(&mut self) {
        self.frame_counter += 1;
        if self.frame_counter >= self.reset_interval {
            self.reset();
        }
    }

    pub fn apply_and_distribute(&mut self, idx: usize, brightness: f32, quantized: f32) -> f32 {
        let accumulated = self.error_buffer[idx];
        let adjusted = brightness + accumulated;
        let error = adjusted - quantized;
        self.error_buffer[idx] = 0.0;

        let x = idx % self.width;
        let y = idx / self.width;

        if x + 1 < self.width {
            self.error_buffer[idx + 1] += error * 7.0 / 16.0;
        }
        if x > 0 && y + 1 < self.height {
            self.error_buffer[idx + self.width - 1] += error * 3.0 / 16.0;
        }
        if y + 1 < self.height {
            self.error_buffer[idx + self.width] += error * 5.0 / 16.0;
        }
        if x + 1 < self.width && y + 1 < self.height {
            self.error_buffer[idx + self.width + 1] += error * 1.0 / 16.0;
        }

        adjusted.clamp(0.0, 1.0)
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_diffusion_new() {
        let ed = ErrorDiffusion::new(10, 20, 60);
        assert_eq!(ed.width(), 10);
        assert_eq!(ed.height(), 20);
        assert_eq!(ed.error_buffer.len(), 200);
        assert!(ed.error_buffer.iter().all(|&e| e == 0.0));
    }

    #[test]
    fn test_error_diffusion_reset() {
        let mut ed = ErrorDiffusion::new(5, 5, 60);
        ed.error_buffer[12] = 0.5;
        ed.error_buffer[13] = -0.3;
        ed.reset();
        assert!(ed.error_buffer.iter().all(|&e| e == 0.0));
        assert_eq!(ed.frame_counter, 0);
    }

    #[test]
    fn test_error_diffusion_tick_resets() {
        let mut ed = ErrorDiffusion::new(5, 5, 10);
        ed.error_buffer[12] = 0.5;
        for _ in 0..9 {
            ed.tick();
        }
        assert!(ed.error_buffer[12] != 0.0);
        ed.tick();
        assert!(ed.error_buffer.iter().all(|&e| e == 0.0));
    }

    #[test]
    fn test_error_diffusion_apply_and_distribute() {
        let mut ed = ErrorDiffusion::new(4, 2, 60);
        let adjusted = ed.apply_and_distribute(0, 0.7, 0.625);
        assert_eq!(adjusted, 0.7);
        assert_eq!(ed.error_buffer[0], 0.0);
        assert!((ed.error_buffer[1] - 0.075 / 16.0 * 7.0).abs() < 0.0001);
        assert!((ed.error_buffer[4] - 0.075 / 16.0 * 5.0).abs() < 0.0001);
        assert!((ed.error_buffer[5] - 0.075 / 16.0 * 1.0).abs() < 0.0001);
    }

    #[test]
    fn test_error_diffusion_right_boundary() {
        let mut ed = ErrorDiffusion::new(4, 2, 60);
        ed.apply_and_distribute(3, 0.7, 0.625);
        assert_eq!(ed.error_buffer[3], 0.0);
        assert!((ed.error_buffer[6] - 0.075 / 16.0 * 3.0).abs() < 0.0001);
        assert!((ed.error_buffer[7] - 0.075 / 16.0 * 5.0).abs() < 0.0001);
    }

    #[test]
    fn test_error_diffusion_bottom_boundary() {
        let mut ed = ErrorDiffusion::new(4, 2, 60);
        ed.apply_and_distribute(4, 0.7, 0.625);
        assert_eq!(ed.error_buffer[4], 0.0);
        assert!((ed.error_buffer[5] - 0.075 / 16.0 * 7.0).abs() < 0.0001);
    }

    #[test]
    fn test_error_diffusion_clamping() {
        let mut ed = ErrorDiffusion::new(4, 2, 60);
        let adjusted = ed.apply_and_distribute(0, 1.5, 1.0);
        assert_eq!(adjusted, 1.0);
        let adjusted = ed.apply_and_distribute(1, -0.5, 0.0);
        assert_eq!(adjusted, 0.0);
    }
}
