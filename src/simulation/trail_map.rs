//! Trail map for storing and diffusing pheromone values.
//!
//! The trail map is a 2D grid where agents deposit pheromones. The map
//! undergoes diffusion (spreading) and decay each frame to create organic
//! patterns.

// These methods are part of the public library API even if unused by the CLI binary
#![allow(dead_code)]

/// A 2D grid storing pheromone trail values.
///
/// Uses double-buffering for efficient diffusion operations.
/// The grid is stored as a 1D vector in row-major order for cache efficiency.
pub struct TrailMap {
    width: usize,
    height: usize,
    current: Vec<f32>,
    scratch: Vec<f32>,
    gaussian_kernel: [f32; 25],
    trail_sum: f32,
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
    /// Create a new trail map with default Gaussian sigma of 1.0.
    pub fn new(width: usize, height: usize) -> Self {
        let size = width * height;
        let gaussian_kernel = generate_gaussian_kernel(1.0);
        Self {
            width,
            height,
            current: vec![0.0; size],
            scratch: vec![0.0; size],
            gaussian_kernel,
            trail_sum: 0.0,
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
            trail_sum: 0.0,
        }
    }

    /// Update the Gaussian kernel sigma for diffusion.
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

    /// Get the scratch buffer (used during diffusion).
    pub fn scratch(&self) -> &[f32] {
        &self.scratch
    }

    /// Get mutable access to the scratch buffer.
    pub fn scratch_mut(&mut self) -> &mut [f32] {
        &mut self.scratch
    }

    pub fn swap_buffers(&mut self) {
        std::mem::swap(&mut self.current, &mut self.scratch);
    }

    /// Get the pheromone value at (x, y). Returns 0.0 if out of bounds.
    pub fn get(&self, x: usize, y: usize) -> f32 {
        if x < self.width && y < self.height {
            self.current[y * self.width + x]
        } else {
            0.0
        }
    }

    /// Set the pheromone value at (x, y). No-op if out of bounds.
    pub fn set(&mut self, x: usize, y: usize, value: f32) {
        if x < self.width && y < self.height {
            self.current[y * self.width + x] = value;
        }
    }

    /// Add to the pheromone value at (x, y). No-op if out of bounds.
    pub fn add(&mut self, x: usize, y: usize, value: f32) {
        if x < self.width && y < self.height {
            self.current[y * self.width + x] += value;
            self.trail_sum += value;
        }
    }

    /// Get the linear index for (x, y), or None if out of bounds.
    pub fn index(&self, x: usize, y: usize) -> Option<usize> {
        if x < self.width && y < self.height {
            Some(y * self.width + x)
        } else {
            None
        }
    }

    /// Clear all pheromone values to zero.
    pub fn clear(&mut self) {
        self.current.fill(0.0);
        self.scratch.fill(0.0);
        self.trail_sum = 0.0;
    }

    /// Get the total number of cells (width * height).
    pub fn size(&self) -> usize {
        self.width * self.height
    }

    /// Get the cumulative sum of all deposited pheromone.
    pub fn trail_sum(&self) -> f32 {
        self.trail_sum
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

    #[inline]
    fn diffuse_scalar_impl(current: &[f32], scratch: &mut [f32], width: usize, height: usize) {
        for y in 1..height - 1 {
            let row_offset = y * width;
            for x in 1..width - 1 {
                let idx = row_offset + x;
                let mut sum = 0.0f32;

                let above = (y - 1) * width;
                let current_row = y * width;
                let below = (y + 1) * width;

                sum += current[above + x - 1];
                sum += current[above + x];
                sum += current[above + x + 1];
                sum += current[current_row + x - 1];
                sum += current[current_row + x];
                sum += current[current_row + x + 1];
                sum += current[below + x - 1];
                sum += current[below + x];
                sum += current[below + x + 1];

                scratch[idx] = sum / 9.0;
            }
        }
    }

    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    #[target_feature(enable = "avx")]
    unsafe fn diffuse_avx_impl(current: &[f32], scratch: &mut [f32], width: usize, height: usize) {
        use std::arch::x86_64::*;

        let simd_width = 8;
        let limit = width.saturating_sub(simd_width + 1);

        let mut y = 1usize;
        while y < height - 1 {
            let current_row = y * width;
            let above = (y - 1) * width;
            let below = (y + 1) * width;
            let mut x = 1usize;

            while x < limit {
                let idx = current_row + x;

                let above_left = _mm256_loadu_ps(current.as_ptr().add(above + x - 1));
                let above_mid = _mm256_loadu_ps(current.as_ptr().add(above + x));
                let above_right = _mm256_loadu_ps(current.as_ptr().add(above + x + 1));

                let curr_left = _mm256_loadu_ps(current.as_ptr().add(current_row + x - 1));
                let curr_mid = _mm256_loadu_ps(current.as_ptr().add(current_row + x));
                let curr_right = _mm256_loadu_ps(current.as_ptr().add(current_row + x + 1));

                let below_left = _mm256_loadu_ps(current.as_ptr().add(below + x - 1));
                let below_mid = _mm256_loadu_ps(current.as_ptr().add(below + x));
                let below_right = _mm256_loadu_ps(current.as_ptr().add(below + x + 1));

                let sum = _mm256_add_ps(above_left, above_mid);
                let sum = _mm256_add_ps(sum, above_right);
                let sum = _mm256_add_ps(sum, curr_left);
                let sum = _mm256_add_ps(sum, curr_mid);
                let sum = _mm256_add_ps(sum, curr_right);
                let sum = _mm256_add_ps(sum, below_left);
                let sum = _mm256_add_ps(sum, below_mid);
                let sum = _mm256_add_ps(sum, below_right);

                let nine = _mm256_set1_ps(9.0);
                let result = _mm256_div_ps(sum, nine);

                _mm256_storeu_ps(scratch.as_mut_ptr().add(idx), result);

                x += simd_width;
            }

            let mut x = limit;
            while x < width - 1 {
                let idx = current_row + x;
                let mut sum = 0.0f32;

                sum += current[above + x - 1];
                sum += current[above + x];
                sum += current[above + x + 1];
                sum += current[current_row + x - 1];
                sum += current[current_row + x];
                sum += current[current_row + x + 1];
                sum += current[below + x - 1];
                sum += current[below + x];
                sum += current[below + x + 1];

                scratch[idx] = sum / 9.0;

                x += 1;
            }

            y += 1;
        }
    }

    #[cfg(target_arch = "aarch64")]
    #[target_feature(enable = "neon")]
    #[allow(unused_variables, dead_code)]
    unsafe fn diffuse_neon_impl(current: &[f32], scratch: &mut [f32], width: usize, height: usize) {
        use std::arch::aarch64::*;

        let simd_width = 4;
        let limit = width.saturating_sub(simd_width + 1);

        let mut y = 1usize;
        while y < height - 1 {
            let current_row = y * width;
            let above = (y - 1) * width;
            let below = (y + 1) * width;
            let mut x = 1usize;

            while x < limit {
                let idx = current_row + x;

                let above_left = vld1q_f32(current.as_ptr().add(above + x - 1));
                let above_mid = vld1q_f32(current.as_ptr().add(above + x));
                let above_right = vld1q_f32(current.as_ptr().add(above + x + 1));

                let curr_left = vld1q_f32(current.as_ptr().add(current_row + x - 1));
                let curr_mid = vld1q_f32(current.as_ptr().add(current_row + x));
                let curr_right = vld1q_f32(current.as_ptr().add(current_row + x + 1));

                let below_left = vld1q_f32(current.as_ptr().add(below + x - 1));
                let below_mid = vld1q_f32(current.as_ptr().add(below + x));
                let below_right = vld1q_f32(current.as_ptr().add(below + x + 1));

                let mut sum = vaddq_f32(above_left, above_mid);
                sum = vaddq_f32(sum, above_right);
                sum = vaddq_f32(sum, curr_left);
                sum = vaddq_f32(sum, curr_mid);
                sum = vaddq_f32(sum, curr_right);
                sum = vaddq_f32(sum, below_left);
                sum = vaddq_f32(sum, below_mid);
                sum = vaddq_f32(sum, below_right);

                let nine = vdupq_n_f32(9.0);
                let result = vdivq_f32(sum, nine);

                vst1q_f32(scratch.as_mut_ptr().add(idx), result);

                x += simd_width;
            }

            let mut x = limit;
            while x < width - 1 {
                let idx = current_row + x;
                let mut sum = 0.0f32;

                sum += current[above + x - 1];
                sum += current[above + x];
                sum += current[above + x + 1];
                sum += current[current_row + x - 1];
                sum += current[current_row + x];
                sum += current[current_row + x + 1];
                sum += current[below + x - 1];
                sum += current[below + x];
                sum += current[below + x + 1];

                scratch[idx] = sum / 9.0;

                x += 1;
            }

            y += 1;
        }
    }

    pub fn diffuse_simd(&mut self) {
        let width = self.width;
        let height = self.height;
        let current = &self.current;
        let scratch = &mut self.scratch;

        scratch.copy_from_slice(current);

        #[cfg(target_arch = "aarch64")]
        let has_simd = std::arch::is_aarch64_feature_detected!("neon");

        #[cfg(target_arch = "aarch64")]
        let _has_avx = false;

        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        let has_avx = std::arch::is_x86_feature_detected!("avx");

        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        let has_simd = has_avx;

        #[cfg(not(any(target_arch = "aarch64", target_arch = "x86", target_arch = "x86_64")))]
        let has_simd = false;

        if has_simd {
            #[cfg(target_arch = "aarch64")]
            unsafe {
                Self::diffuse_neon_impl(current, scratch, width, height);
            }
            #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
            unsafe {
                Self::diffuse_avx_impl(current, scratch, width, height);
            }
        } else {
            Self::diffuse_scalar_impl(current, scratch, width, height);
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

    #[inline]
    fn diffuse_gaussian_scalar_impl(
        current: &[f32],
        scratch: &mut [f32],
        width: usize,
        height: usize,
        kernel: &[f32; 25],
    ) {
        let radius: i32 = 2;
        let kernel_size = 5usize;

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
                            ((ky + radius) * kernel_size as i32 + (kx + radius)) as usize;
                        sum += current[(ny as usize) * width + (nx as usize)] * kernel[kernel_idx];
                    }
                }

                scratch[idx] = sum;
            }
        }
    }

    #[cfg(target_arch = "aarch64")]
    #[target_feature(enable = "neon")]
    #[allow(unused_variables, dead_code)]
    unsafe fn diffuse_gaussian_neon_impl(
        current: &[f32],
        scratch: &mut [f32],
        width: usize,
        height: usize,
        kernel: &[f32; 25],
    ) {
        use std::arch::aarch64::*;

        let k0 = vdupq_n_f32(kernel[0]);
        let k1 = vdupq_n_f32(kernel[1]);
        let k2 = vdupq_n_f32(kernel[2]);
        let k3 = vdupq_n_f32(kernel[3]);
        let k4 = vdupq_n_f32(kernel[4]);
        let k5 = vdupq_n_f32(kernel[5]);
        let k6 = vdupq_n_f32(kernel[6]);
        let k7 = vdupq_n_f32(kernel[7]);
        let k8 = vdupq_n_f32(kernel[8]);
        let k9 = vdupq_n_f32(kernel[9]);
        let k10 = vdupq_n_f32(kernel[10]);
        let k11 = vdupq_n_f32(kernel[11]);
        let k12 = vdupq_n_f32(kernel[12]);
        let k13 = vdupq_n_f32(kernel[13]);
        let k14 = vdupq_n_f32(kernel[14]);
        let k15 = vdupq_n_f32(kernel[15]);
        let k16 = vdupq_n_f32(kernel[16]);
        let k17 = vdupq_n_f32(kernel[17]);
        let k18 = vdupq_n_f32(kernel[18]);
        let k19 = vdupq_n_f32(kernel[19]);
        let k20 = vdupq_n_f32(kernel[20]);
        let k21 = vdupq_n_f32(kernel[21]);
        let k22 = vdupq_n_f32(kernel[22]);
        let k23 = vdupq_n_f32(kernel[23]);
        let k24 = vdupq_n_f32(kernel[24]);

        let simd_width = 4;
        let limit = width.saturating_sub(simd_width + 2);

        let mut y = 2usize;

        while y < height - 2 {
            let current_row = y * width;
            let row_minus_1 = (y - 1) * width;
            let row_minus_2 = (y - 2) * width;
            let row_plus_1 = (y + 1) * width;
            let row_plus_2 = (y + 2) * width;

            let mut x = 2usize;

            while x < limit {
                let idx = current_row + x;

                let m2_l = vld1q_f32(current.as_ptr().add(row_minus_2 + x - 2));
                let m2_m = vld1q_f32(current.as_ptr().add(row_minus_2 + x - 1));
                let m2_r = vld1q_f32(current.as_ptr().add(row_minus_2 + x));

                let m1_l = vld1q_f32(current.as_ptr().add(row_minus_1 + x - 2));
                let m1_m = vld1q_f32(current.as_ptr().add(row_minus_1 + x - 1));
                let m1_r = vld1q_f32(current.as_ptr().add(row_minus_1 + x));

                let p0_l = vld1q_f32(current.as_ptr().add(current_row + x - 2));
                let p0_m = vld1q_f32(current.as_ptr().add(current_row + x - 1));
                let p0_r = vld1q_f32(current.as_ptr().add(current_row + x));

                let p1_l = vld1q_f32(current.as_ptr().add(row_plus_1 + x - 2));
                let p1_m = vld1q_f32(current.as_ptr().add(row_plus_1 + x - 1));
                let p1_r = vld1q_f32(current.as_ptr().add(row_plus_1 + x));

                let p2_l = vld1q_f32(current.as_ptr().add(row_plus_2 + x - 2));
                let p2_m = vld1q_f32(current.as_ptr().add(row_plus_2 + x - 1));
                let p2_r = vld1q_f32(current.as_ptr().add(row_plus_2 + x));

                let mut sum = vmulq_f32(m2_l, k0);
                sum = vmlaq_f32(sum, m2_m, k1);
                sum = vmlaq_f32(sum, m2_r, k2);
                sum = vmlaq_f32(
                    sum,
                    vld1q_f32(current.as_ptr().add(row_minus_2 + x + 1)),
                    k3,
                );
                sum = vmlaq_f32(
                    sum,
                    vld1q_f32(current.as_ptr().add(row_minus_2 + x + 2)),
                    k4,
                );
                sum = vmlaq_f32(sum, m1_l, k5);
                sum = vmlaq_f32(sum, m1_m, k6);
                sum = vmlaq_f32(sum, m1_r, k7);
                sum = vmlaq_f32(
                    sum,
                    vld1q_f32(current.as_ptr().add(row_minus_1 + x + 1)),
                    k8,
                );
                sum = vmlaq_f32(
                    sum,
                    vld1q_f32(current.as_ptr().add(row_minus_1 + x + 2)),
                    k9,
                );
                sum = vmlaq_f32(sum, p0_l, k10);
                sum = vmlaq_f32(sum, p0_m, k11);
                sum = vmlaq_f32(sum, p0_r, k12);
                sum = vmlaq_f32(
                    sum,
                    vld1q_f32(current.as_ptr().add(current_row + x + 1)),
                    k13,
                );
                sum = vmlaq_f32(
                    sum,
                    vld1q_f32(current.as_ptr().add(current_row + x + 2)),
                    k14,
                );
                sum = vmlaq_f32(sum, p1_l, k15);
                sum = vmlaq_f32(sum, p1_m, k16);
                sum = vmlaq_f32(sum, p1_r, k17);
                sum = vmlaq_f32(
                    sum,
                    vld1q_f32(current.as_ptr().add(row_plus_1 + x + 1)),
                    k18,
                );
                sum = vmlaq_f32(
                    sum,
                    vld1q_f32(current.as_ptr().add(row_plus_1 + x + 2)),
                    k19,
                );
                sum = vmlaq_f32(sum, p2_l, k20);
                sum = vmlaq_f32(sum, p2_m, k21);
                sum = vmlaq_f32(sum, p2_r, k22);
                sum = vmlaq_f32(
                    sum,
                    vld1q_f32(current.as_ptr().add(row_plus_2 + x + 1)),
                    k23,
                );
                sum = vmlaq_f32(
                    sum,
                    vld1q_f32(current.as_ptr().add(row_plus_2 + x + 2)),
                    k24,
                );

                vst1q_f32(scratch.as_mut_ptr().add(idx), sum);

                x += simd_width;
            }

            let mut x = limit;
            while x < width - 2 {
                let idx = current_row + x;
                let mut sum = 0.0f32;

                for ky in -2..=2 {
                    for kx in -2..=2 {
                        let nx = x as i32 + kx;
                        let ny = y as i32 + ky;
                        let kernel_idx = ((ky + 2) * 5 + (kx + 2)) as usize;
                        sum += current[(ny as usize) * width + (nx as usize)] * kernel[kernel_idx];
                    }
                }

                scratch[idx] = sum;

                x += 1;
            }

            y += 1;
        }
    }

    pub fn diffuse_gaussian_simd(&mut self) {
        let width = self.width;
        let height = self.height;
        let current = &self.current;
        let scratch = &mut self.scratch;
        let kernel = &self.gaussian_kernel;

        scratch.copy_from_slice(current);

        #[cfg(target_arch = "aarch64")]
        let has_simd = std::arch::is_aarch64_feature_detected!("neon");

        #[cfg(target_arch = "aarch64")]
        let _has_avx = false;

        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        let has_avx = std::arch::is_x86_feature_detected!("avx");

        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        let has_simd = has_avx;

        #[cfg(not(any(target_arch = "aarch64", target_arch = "x86", target_arch = "x86_64")))]
        let has_simd = false;

        if has_simd {
            #[cfg(target_arch = "aarch64")]
            unsafe {
                Self::diffuse_gaussian_neon_impl(current, scratch, width, height, kernel);
            }
            #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
            {
                Self::diffuse_gaussian_scalar_impl(current, scratch, width, height, kernel);
            }
        } else {
            Self::diffuse_gaussian_scalar_impl(current, scratch, width, height, kernel);
        }

        self.swap_buffers();
    }

    pub fn diffuse_with_kernel(&mut self, use_simd: bool, use_gaussian: bool) {
        if use_simd {
            if use_gaussian {
                self.diffuse_gaussian_simd();
            } else {
                self.diffuse_simd();
            }
        } else if use_gaussian {
            self.diffuse_gaussian();
        } else {
            self.diffuse();
        }
    }

    pub fn decay(&mut self, factor: f32) {
        for value in &mut self.current {
            *value *= factor;
        }
        self.trail_sum *= factor;
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

        trail1.diffuse_with_kernel(false, false);
        trail2.diffuse();

        assert_eq!(trail1.current(), trail2.current());
    }
}
