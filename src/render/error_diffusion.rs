//! Implements Floyd-Steinberg error diffusion dithering.
//!
//! Maintains error buffers for the current and next row to propagate quantization errors.

/// Floyd-Steinberg error diffusion coefficients (left-to-right scan).
/// The 16 divisor is implicit: right = 7/16, down_left = 3/16, down = 5/16, down_right = 1/16
mod fs_coeff {
    pub const RIGHT: f32 = 7.0 / 16.0;
    pub const DOWN_LEFT: f32 = 3.0 / 16.0;
    pub const DOWN: f32 = 5.0 / 16.0;
    pub const DOWN_RIGHT: f32 = 1.0 / 16.0;

    /// Coefficients for serpentine (right-to-left) scanning.
    /// Vertical coefficients (down, down_right) are shared with the parent module.
    pub mod serpentine {
        pub const RIGHT: f32 = 3.0 / 16.0;
        pub const DOWN_LEFT: f32 = 7.0 / 16.0;
    }
}

/// Error diffusion state for dithering quantization errors across pixels.
pub struct ErrorDiffusion {
    width: usize,
    height: usize,
    current_row_errors_top: Vec<f32>,
    current_row_errors_bottom: Vec<f32>,
    next_row_errors_top: Vec<f32>,
    next_row_errors_bottom: Vec<f32>,
    last_row_errors_top: Vec<f32>,
    last_row_errors_bottom: Vec<f32>,
    current_y: usize,
}

impl ErrorDiffusion {
    /// Creates a new error diffusion state.
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            current_row_errors_top: vec![0.0; width],
            current_row_errors_bottom: vec![0.0; width],
            next_row_errors_top: vec![0.0; width],
            next_row_errors_bottom: vec![0.0; width],
            last_row_errors_top: vec![0.0; width],
            last_row_errors_bottom: vec![0.0; width],
            current_y: 0,
        }
    }

    /// Resets all error buffers to zero.
    pub fn reset(&mut self) {
        self.current_row_errors_top.fill(0.0);
        self.current_row_errors_bottom.fill(0.0);
        self.next_row_errors_top.fill(0.0);
        self.next_row_errors_bottom.fill(0.0);
        self.last_row_errors_top.fill(0.0);
        self.last_row_errors_bottom.fill(0.0);
        self.current_y = 0;
    }

    /// Resizes the error buffers to match new dimensions.
    pub fn resize(&mut self, width: usize, height: usize) {
        self.width = width;
        self.height = height;
        self.current_row_errors_top.resize(width, 0.0);
        self.current_row_errors_bottom.resize(width, 0.0);
        self.next_row_errors_top.resize(width, 0.0);
        self.next_row_errors_bottom.resize(width, 0.0);
        self.last_row_errors_top.resize(width, 0.0);
        self.last_row_errors_bottom.resize(width, 0.0);
        self.reset();
    }

    /// Prepares for processing a new row.
    ///
    /// Swaps the "next" row errors into "current", and clears "next".
    pub fn start_row(&mut self, y: usize) {
        self.current_y = y;
        std::mem::swap(
            &mut self.current_row_errors_top,
            &mut self.next_row_errors_top,
        );
        std::mem::swap(
            &mut self.current_row_errors_bottom,
            &mut self.next_row_errors_bottom,
        );
        self.next_row_errors_top.fill(0.0);
        self.next_row_errors_bottom.fill(0.0);
    }

    /// Backs up the current row errors to the "last" buffer.
    /// Used for wrapping errors in some modes.
    pub fn transfer_boundary_errors(&mut self) {
        for x in 0..self.width {
            self.last_row_errors_top[x] = self.current_row_errors_top[x];
            self.last_row_errors_bottom[x] = self.current_row_errors_bottom[x];
        }
    }

    /// Injects errors from the previous frame/row into the current one.
    pub fn inject_boundary_errors(&mut self) {
        for x in 0..self.width {
            self.current_row_errors_top[x] += self.last_row_errors_top[x];
            self.current_row_errors_bottom[x] += self.last_row_errors_bottom[x];
        }
    }

    /// Applies error diffusion to a single pixel.
    ///
    /// # Arguments
    /// * `x`, `y` - Pixel coordinates.
    /// * `brightness` - Original brightness value.
    /// * `quantized` - The chosen quantized value.
    /// * `is_top` - Whether this is the top or bottom subpixel (for half-block charsets).
    /// * `serpentine` - If true, alternates error distribution direction.
    ///
    /// Returns the quantized value (passed through).
    pub fn apply_and_distribute(
        &mut self,
        x: usize,
        y: usize,
        brightness: f32,
        quantized: f32,
        is_top: bool,
        serpentine: bool,
    ) -> f32 {
        let accumulated = if is_top {
            self.current_row_errors_top[x]
        } else {
            self.current_row_errors_bottom[x]
        };
        let adjusted = brightness + accumulated;
        let error = adjusted - quantized;

        let current_errors = if is_top {
            &mut self.current_row_errors_top
        } else {
            &mut self.current_row_errors_bottom
        };
        let next_errors = if is_top {
            &mut self.next_row_errors_top
        } else {
            &mut self.next_row_errors_bottom
        };

        current_errors[x] = 0.0;

        let (right, down_left, down, down_right) = if serpentine && y % 2 == 1 {
            (
                fs_coeff::serpentine::RIGHT,
                fs_coeff::serpentine::DOWN_LEFT,
                fs_coeff::DOWN,
                fs_coeff::DOWN_RIGHT,
            )
        } else {
            (
                fs_coeff::RIGHT,
                fs_coeff::DOWN_LEFT,
                fs_coeff::DOWN,
                fs_coeff::DOWN_RIGHT,
            )
        };

        if x + 1 < self.width {
            current_errors[x + 1] += error * right;
        }
        if x > 0 && y + 1 < self.height {
            next_errors[x - 1] += error * down_left;
        }
        if y + 1 < self.height {
            next_errors[x] += error * down;
        }
        if x + 1 < self.width && y + 1 < self.height {
            next_errors[x + 1] += error * down_right;
        }

        quantized
    }

    /// Returns the width of the error buffer.
    pub fn width(&self) -> usize {
        self.width
    }

    /// Returns the height of the error buffer.
    pub fn height(&self) -> usize {
        self.height
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_diffusion_new() {
        let ed = ErrorDiffusion::new(10, 20);
        assert_eq!(ed.width(), 10);
        assert_eq!(ed.height(), 20);
        assert_eq!(ed.current_row_errors_top.len(), 10);
        assert_eq!(ed.current_row_errors_bottom.len(), 10);
        assert!(ed.current_row_errors_top.iter().all(|&e| e == 0.0));
        assert!(ed.current_row_errors_bottom.iter().all(|&e| e == 0.0));
    }

    #[test]
    fn test_error_diffusion_reset() {
        let mut ed = ErrorDiffusion::new(5, 5);
        ed.current_row_errors_top[2] = 0.5;
        ed.current_row_errors_top[3] = -0.3;
        ed.current_row_errors_bottom[2] = 0.2;
        ed.reset();
        assert!(ed.current_row_errors_top.iter().all(|&e| e == 0.0));
        assert!(ed.current_row_errors_bottom.iter().all(|&e| e == 0.0));
    }

    #[test]
    fn test_error_diffusion_apply_and_distribute_top() {
        let mut ed = ErrorDiffusion::new(4, 2);
        ed.start_row(0);
        let quantized = ed.apply_and_distribute(0, 0, 0.7, 0.625, true, false);
        assert_eq!(quantized, 0.625);
        assert_eq!(ed.current_row_errors_top[0], 0.0);
        assert!((ed.current_row_errors_top[1] - 0.075 / 16.0 * 7.0).abs() < 0.0001);
        assert!((ed.next_row_errors_top[0] - 0.075 / 16.0 * 5.0).abs() < 0.0001);
        assert!((ed.next_row_errors_top[1] - 0.075 / 16.0 * 1.0).abs() < 0.0001);
    }

    #[test]
    fn test_error_diffusion_apply_and_distribute_bottom() {
        let mut ed = ErrorDiffusion::new(4, 2);
        ed.start_row(0);
        let quantized = ed.apply_and_distribute(0, 0, 0.7, 0.625, false, false);
        assert_eq!(quantized, 0.625);
        assert_eq!(ed.current_row_errors_bottom[0], 0.0);
        assert!((ed.current_row_errors_bottom[1] - 0.075 / 16.0 * 7.0).abs() < 0.0001);
        assert!((ed.next_row_errors_bottom[0] - 0.075 / 16.0 * 5.0).abs() < 0.0001);
        assert!((ed.next_row_errors_bottom[1] - 0.075 / 16.0 * 1.0).abs() < 0.0001);
    }

    #[test]
    fn test_error_diffusion_separate_buffers() {
        let mut ed = ErrorDiffusion::new(4, 2);
        ed.start_row(0);
        ed.apply_and_distribute(0, 0, 0.7, 0.625, true, false);
        ed.apply_and_distribute(0, 0, 0.3, 0.25, false, false);
        assert_eq!(ed.current_row_errors_top[0], 0.0);
        assert_eq!(ed.current_row_errors_bottom[0], 0.0);
        assert!(ed.current_row_errors_top[1] != 0.0);
        assert!(ed.current_row_errors_bottom[1] != 0.0);
    }

    #[test]
    fn test_error_diffusion_right_boundary() {
        let mut ed = ErrorDiffusion::new(4, 2);
        ed.start_row(0);
        ed.apply_and_distribute(3, 0, 0.7, 0.625, true, false);
        assert_eq!(ed.current_row_errors_top[3], 0.0);
        assert!((ed.next_row_errors_top[2] - 0.075 / 16.0 * 3.0).abs() < 0.0001);
        assert!((ed.next_row_errors_top[3] - 0.075 / 16.0 * 5.0).abs() < 0.0001);
    }

    #[test]
    fn test_error_diffusion_serpentine_right_to_left() {
        let mut ed = ErrorDiffusion::new(4, 3);
        ed.start_row(0);
        ed.apply_and_distribute(3, 0, 0.7, 0.625, true, true);
        assert_eq!(ed.current_row_errors_top[3], 0.0);
        assert!((ed.next_row_errors_top[2] - 0.075 / 16.0 * 3.0).abs() < 0.0001);
        assert!((ed.next_row_errors_top[3] - 0.075 / 16.0 * 5.0).abs() < 0.0001);
    }

    #[test]
    fn test_error_diffusion_quantized_return() {
        let mut ed = ErrorDiffusion::new(4, 2);
        ed.start_row(0);
        let quantized = ed.apply_and_distribute(0, 0, 1.5, 1.0, true, false);
        assert_eq!(quantized, 1.0);
        let quantized = ed.apply_and_distribute(1, 0, -0.5, 0.0, true, false);
        assert_eq!(quantized, 0.0);
    }

    #[test]
    fn test_error_diffusion_start_row() {
        let mut ed = ErrorDiffusion::new(4, 2);
        ed.current_row_errors_top[0] = 1.0;
        ed.next_row_errors_top[0] = 2.0;
        ed.start_row(0);
        assert_eq!(ed.current_row_errors_top[0], 2.0);
        assert_eq!(ed.next_row_errors_top[0], 0.0);
    }

    #[test]
    fn test_error_diffusion_boundary_transfer() {
        let mut ed = ErrorDiffusion::new(4, 2);
        ed.start_row(0);
        ed.apply_and_distribute(0, 0, 0.7, 0.625, true, false);
        ed.transfer_boundary_errors();
        assert_eq!(ed.last_row_errors_top[0], 0.0);
        assert!((ed.last_row_errors_top[1] - 0.075 / 16.0 * 7.0).abs() < 0.0001);
    }

    #[test]
    fn test_error_diffusion_boundary_inject() {
        let mut ed = ErrorDiffusion::new(4, 2);
        ed.last_row_errors_top[0] = 0.5;
        ed.inject_boundary_errors();
        assert_eq!(ed.current_row_errors_top[0], 0.5);
    }

    #[test]
    fn test_error_diffusion_row_swap() {
        let mut ed = ErrorDiffusion::new(4, 2);
        ed.next_row_errors_top[2] = 0.3;
        ed.start_row(0);
        assert_eq!(ed.current_row_errors_top[2], 0.3);
        assert_eq!(ed.next_row_errors_top[2], 0.0);
    }
}
