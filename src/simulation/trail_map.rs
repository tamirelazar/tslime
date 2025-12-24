pub struct TrailMap {
    width: usize,
    height: usize,
    current: Vec<f32>,
    scratch: Vec<f32>,
}

impl TrailMap {
    pub fn new(width: usize, height: usize) -> Self {
        let size = width * height;
        Self {
            width,
            height,
            current: vec![0.0; size],
            scratch: vec![0.0; size],
        }
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
}
