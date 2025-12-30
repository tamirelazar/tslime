pub struct TrailMap {
    width: usize,
    height: usize,
    current: Vec<f32>,
    scratch: Vec<f32>,
    gaussian_kernel: [f32; 25],
}

const GAUSSIAN_KERNEL_SIZE: usize = 5;

fn generate_gaussian_kernel(sigma: f32) -> [f32; 25] {
    let mut kernel = [0.0f32; 25];
    let radius: i32 = 2;
    let two_sigma_sq = 2.0 * sigma * sigma;
    let mut sum = 0.0f32;

    for y in -radius..=radius {
        for x in -radius..=radius {
            let idx = ((y + radius) * GAUSSIAN_KERNEL_SIZE as i32 + (x + radius)) as usize;
            let dist_sq = (x * x + y * y) as f32;
            kernel[idx] = (-dist_sq / two_sigma_sq).exp();
            sum += kernel[idx];
        }
    }

    for kernel_val in kernel.iter_mut() {
        *kernel_val /= sum;
    }

    kernel
}

impl TrailMap {
    #[allow(dead_code)]
    pub fn new(width: usize, height: usize) -> Self {
        let size = width * height;
        let gaussian_kernel = generate_gaussian_kernel(1.0);
        Self {
            width,
            height,
            current: vec![0.0; size],
            scratch: vec![0.0; size],
            gaussian_kernel,
        }
    }

    pub fn new_with_sigma(width: usize, height: usize, sigma: f32) -> Self {
        let size = width * height;
        let gaussian_kernel = generate_gaussian_kernel(sigma);
        Self {
            width,
            height,
            current: vec![0.0; size],
            scratch: vec![0.0; size],
            gaussian_kernel,
        }
    }

    #[allow(dead_code)]
    pub fn set_gaussian_sigma(&mut self, sigma: f32) {
        self.gaussian_kernel = generate_gaussian_kernel(sigma);
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn current(&self) -> &[f32] {
        &self.current
    }

    pub fn current_mut(&mut self) -> &mut [f32] {
        &mut self.current
    }

    #[allow(dead_code)]
    pub fn scratch(&self) -> &[f32] {
        &self.scratch
    }

    #[allow(dead_code)]
    pub fn scratch_mut(&mut self) -> &mut [f32] {
        &mut self.scratch
    }

    pub fn swap_buffers(&mut self) {
        std::mem::swap(&mut self.current, &mut self.scratch);
    }

    #[allow(dead_code)]
    pub fn get(&self, x: usize, y: usize) -> f32 {
        if x < self.width && y < self.height {
            self.current[y * self.width + x]
        } else {
            0.0
        }
    }

    #[allow(dead_code)]
    pub fn set(&mut self, x: usize, y: usize, value: f32) {
        if x < self.width && y < self.height {
            self.current[y * self.width + x] = value;
        }
    }

    #[allow(dead_code)]
    pub fn add(&mut self, x: usize, y: usize, value: f32) {
        if x < self.width && y < self.height {
            self.current[y * self.width + x] += value;
        }
    }

    #[allow(dead_code)]
    pub fn index(&self, x: usize, y: usize) -> Option<usize> {
        if x < self.width && y < self.height {
            Some(y * self.width + x)
        } else {
            None
        }
    }

    #[allow(dead_code)]
    pub fn clear(&mut self) {
        self.current.fill(0.0);
        self.scratch.fill(0.0);
    }

    #[allow(dead_code)]
    pub fn size(&self) -> usize {
        self.width * self.height
    }

    pub fn diffuse(&mut self) {
        let width = self.width;
        let height = self.height;
        let current = &self.current;
        let scratch = &mut self.scratch;

        scratch.copy_from_slice(current);

        for y in 1..height - 1 {
            let row_offset = y * width;
            for x in 1..width - 1 {
                let idx = row_offset + x;

                let mut sum = 0.0f32;
                let mut count = 0;

                for dy in -1..=1 {
                    for dx in -1..=1 {
                        let nx = x as i32 + dx;
                        let ny = y as i32 + dy;
                        if nx >= 0 && nx < width as i32 && ny >= 0 && ny < height as i32 {
                            sum += current[(ny as usize) * width + (nx as usize)];
                            count += 1;
                        }
                    }
                }

                scratch[idx] = sum / count as f32;
            }
        }

        self.swap_buffers();
    }

    pub fn diffuse_gaussian(&mut self) {
        let width = self.width;
        let height = self.height;
        let current = &self.current;
        let scratch = &mut self.scratch;
        let kernel = &self.gaussian_kernel;
        let radius: i32 = 2;

        scratch.copy_from_slice(current);

        for y in 2..height - 2 {
            let row_offset = y * width;
            for x in 2..width - 2 {
                let idx = row_offset + x;

                let mut sum = 0.0f32;

                for ky in -radius..=radius {
                    for kx in -radius..=radius {
                        let nx = x as i32 + kx;
                        let ny = y as i32 + ky;
                        let kernel_idx =
                            ((ky + radius) * GAUSSIAN_KERNEL_SIZE as i32 + (kx + radius)) as usize;
                        sum += current[(ny as usize) * width + (nx as usize)] * kernel[kernel_idx];
                    }
                }

                scratch[idx] = sum;
            }
        }

        self.swap_buffers();
    }

    pub fn diffuse_with_kernel(&mut self, use_gaussian: bool) {
        if use_gaussian {
            self.diffuse_gaussian();
        } else {
            self.diffuse();
        }
    }

    pub fn decay(&mut self, factor: f32) {
        for value in &mut self.current {
            *value *= factor;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trail_map_creation() {
        let trail = TrailMap::new(400, 400);
        assert_eq!(trail.width(), 400);
        assert_eq!(trail.height(), 400);
        assert_eq!(trail.size(), 160000);
    }

    #[test]
    fn test_set_and_get() {
        let mut trail = TrailMap::new(400, 400);
        trail.set(100, 100, 5.0);
        assert_eq!(trail.get(100, 100), 5.0);
    }

    #[test]
    fn test_add() {
        let mut trail = TrailMap::new(400, 400);
        trail.add(100, 100, 3.0);
        trail.add(100, 100, 2.0);
        assert_eq!(trail.get(100, 100), 5.0);
    }

    #[test]
    fn test_boundary_checking() {
        let mut trail = TrailMap::new(400, 400);
        trail.set(400, 100, 5.0);
        assert_eq!(trail.get(400, 100), 0.0);

        trail.set(-1_isize as usize, 100, 5.0);
        assert_eq!(trail.get(-1_isize as usize, 100), 0.0);
    }

    #[test]
    fn test_index() {
        let trail = TrailMap::new(400, 400);
        assert_eq!(trail.index(100, 100), Some(100 * 400 + 100));
        assert_eq!(trail.index(400, 100), None);
        assert_eq!(trail.index(100, 400), None);
    }

    #[test]
    fn test_clear() {
        let mut trail = TrailMap::new(400, 400);
        trail.set(100, 100, 5.0);
        trail.clear();
        assert_eq!(trail.get(100, 100), 0.0);
    }

    #[test]
    fn test_swap_buffers() {
        let mut trail = TrailMap::new(400, 400);
        trail.set(100, 100, 5.0);
        trail.swap_buffers();
        assert_eq!(trail.get(100, 100), 0.0);
    }

    #[test]
    fn test_diffuse_single_pixel() {
        let mut trail = TrailMap::new(10, 10);
        trail.set(5, 5, 9.0);
        trail.diffuse();
        let value = trail.get(5, 5);
        assert!(value > 0.0 && value < 9.0);
    }

    #[test]
    fn test_diffuse_preserves_edges() {
        let mut trail = TrailMap::new(10, 10);
        trail.set(0, 0, 5.0);
        trail.set(9, 9, 5.0);
        trail.diffuse();
        assert!(trail.get(0, 0) > 0.0);
        assert!(trail.get(9, 9) > 0.0);
    }

    #[test]
    fn test_diffuse_no_values() {
        let mut trail = TrailMap::new(10, 10);
        trail.diffuse();
        assert_eq!(trail.get(5, 5), 0.0);
    }

    #[test]
    fn test_decay() {
        let mut trail = TrailMap::new(10, 10);
        trail.set(5, 5, 10.0);
        trail.decay(0.5);
        assert!((trail.get(5, 5) - 5.0).abs() < 0.001);
    }

    #[test]
    fn test_decay_multiple() {
        let mut trail = TrailMap::new(10, 10);
        trail.set(5, 5, 100.0);
        trail.decay(0.9);
        trail.decay(0.9);
        assert!((trail.get(5, 5) - 81.0).abs() < 0.001);
    }

    #[test]
    fn test_gaussian_kernel_normalization() {
        let kernel = generate_gaussian_kernel(1.0);
        let sum: f32 = kernel.iter().sum();
        assert!((sum - 1.0).abs() < 0.0001);
    }

    #[test]
    fn test_gaussian_kernel_center_weight() {
        let kernel = generate_gaussian_kernel(1.0);
        let center_idx = 12;
        let corner_idx = 0;
        assert!(kernel[center_idx] > kernel[corner_idx]);
    }

    #[test]
    fn test_gaussian_kernel_sigma_effect() {
        let kernel_small = generate_gaussian_kernel(0.5);
        let kernel_large = generate_gaussian_kernel(1.5);
        let center_idx = 12;
        assert!(kernel_small[center_idx] > kernel_large[center_idx]);
    }

    #[test]
    fn test_diffuse_gaussian_single_pixel() {
        let mut trail = TrailMap::new(10, 10);
        trail.set(5, 5, 9.0);
        trail.diffuse_gaussian();
        let value = trail.get(5, 5);
        assert!(value > 0.0 && value < 9.0);
    }

    #[test]
    fn test_diffuse_gaussian_spreads_more_than_mean() {
        let mut trail1 = TrailMap::new(20, 20);
        let mut trail2 = TrailMap::new(20, 20);

        trail1.set(10, 10, 10.0);
        trail2.set(10, 10, 10.0);

        trail1.diffuse();
        trail2.diffuse_gaussian();

        let center1 = trail1.get(10, 10);
        let center2 = trail2.get(10, 10);

        assert!(
            center2 > center1,
            "Gaussian should preserve more center value than mean"
        );
    }

    #[test]
    fn test_diffuse_with_kernel_dispatch() {
        let mut trail1 = TrailMap::new(10, 10);
        let mut trail2 = TrailMap::new(10, 10);

        trail1.set(5, 5, 9.0);
        trail2.set(5, 5, 9.0);

        trail1.diffuse_with_kernel(false);
        trail2.diffuse();

        assert_eq!(trail1.current(), trail2.current());
    }
}
