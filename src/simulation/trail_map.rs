//! Trail map for storing and diffusing pheromone values.
//!
//! The trail map is a 2D grid where agents deposit pheromones. The map
//! undergoes diffusion (spreading) and decay each frame to create organic
//! patterns.
//!
//! The 3x3 mean filter is the trail diffusion step from Jones (2010) — see
//! the [`crate::simulation`] module docs for the full citation. The 5x5
//! Gaussian is a standard image-processing alternative for smoother spread.

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
    /// The Gaussian sigma the current `gaussian_kernel` was generated from.
    sigma: f32,
    trail_sum: f32,
    boundary_mode: super::config::BoundaryMode,
    /// Pre-allocated snapshot buffer for the Lague diffuse-weight blend.
    /// Only populated (and only used) when `diffuse_weight < 1.0`.
    blend_src: Vec<f32>,
    /// Per-frame deposit-accumulation scratch buffer (lever 4). Empty until the
    /// nonlinear-deposit path is active; reused across frames (hot-path rule).
    accum: Vec<f32>,
}

const GAUSSIAN_KERNEL_SIZE: usize = 5;
const GAUSSIAN_RADIUS: i32 = 2;

fn generate_gaussian_kernel(sigma: f32) -> [f32; 25] {
    let mut kernel = [0.0f32; 25];
    let two_sigma_sq = 2.0 * sigma * sigma;
    let mut sum = 0.0f32;

    for y in -GAUSSIAN_RADIUS..=GAUSSIAN_RADIUS {
        for x in -GAUSSIAN_RADIUS..=GAUSSIAN_RADIUS {
            let idx = ((y + GAUSSIAN_RADIUS) * GAUSSIAN_KERNEL_SIZE as i32 + (x + GAUSSIAN_RADIUS))
                as usize;
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
            sigma: 1.0,
            trail_sum: 0.0,
            boundary_mode: super::config::BoundaryMode::Bounce,
            blend_src: Vec::new(),
            accum: Vec::new(),
        }
    }

    /// Create a new trail map with custom Gaussian sigma.
    pub fn new_with_sigma(width: usize, height: usize, sigma: f32) -> Self {
        let size = width * height;
        let gaussian_kernel = generate_gaussian_kernel(sigma);
        Self {
            width,
            height,
            current: vec![0.0; size],
            scratch: vec![0.0; size],
            gaussian_kernel,
            sigma,
            trail_sum: 0.0,
            boundary_mode: super::config::BoundaryMode::Bounce,
            blend_src: Vec::new(),
            accum: Vec::new(),
        }
    }

    /// Create a new trail map with custom Gaussian sigma and boundary mode.
    pub fn new_with_sigma_and_boundary(
        width: usize,
        height: usize,
        sigma: f32,
        boundary_mode: super::config::BoundaryMode,
    ) -> Self {
        let size = width * height;
        let gaussian_kernel = generate_gaussian_kernel(sigma);
        Self {
            width,
            height,
            current: vec![0.0; size],
            scratch: vec![0.0; size],
            gaussian_kernel,
            sigma,
            trail_sum: 0.0,
            boundary_mode,
            blend_src: Vec::new(),
            accum: Vec::new(),
        }
    }

    /// Update the Gaussian kernel sigma for diffusion.
    pub fn set_gaussian_sigma(&mut self, sigma: f32) {
        self.gaussian_kernel = generate_gaussian_kernel(sigma);
        self.sigma = sigma;
    }

    /// Returns the Gaussian sigma the diffusion kernel is currently built from.
    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) fn gaussian_sigma(&self) -> f32 {
        self.sigma
    }

    /// Get the width of the map.
    pub fn width(&self) -> usize {
        self.width
    }

    /// Get the height of the map.
    pub fn height(&self) -> usize {
        self.height
    }

    /// Get the current trail buffer.
    pub fn current(&self) -> &[f32] {
        &self.current
    }

    /// Get mutable access to the current trail buffer.
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

    /// Swap the current and scratch buffers.
    pub fn swap_buffers(&mut self) {
        std::mem::swap(&mut self.current, &mut self.scratch);
    }

    /// Get the pheromone value at (x, y). Returns 0.0 if out of bounds.
    #[inline]
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

    /// Apply 3x3 mean-filter diffusion (the trail diffusion from Jones 2010).
    pub fn diffuse(&mut self) {
        let width = self.width;
        let height = self.height;
        let current = &self.current;
        let scratch = &mut self.scratch;
        let is_wrap = matches!(self.boundary_mode, super::config::BoundaryMode::Wrap);

        scratch.copy_from_slice(current);

        if is_wrap {
            // Toroidal: neighbors wrap across edges, all pixels processed
            let w = width as i32;
            let h = height as i32;
            for y in 0..height {
                for x in 0..width {
                    let idx = y * width + x;
                    let mut sum = 0.0f32;

                    for dy in -1..=1 {
                        for dx in -1..=1 {
                            let mut nx = x as i32 + dx;
                            let mut ny = y as i32 + dy;
                            if nx < 0 {
                                nx += w;
                            } else if nx >= w {
                                nx -= w;
                            }
                            if ny < 0 {
                                ny += h;
                            } else if ny >= h {
                                ny -= h;
                            }
                            sum += current[ny as usize * width + nx as usize];
                        }
                    }

                    scratch[idx] = sum / 9.0;
                }
            }
        } else {
            // Bounce: leave the 1-pixel border undiffused
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

                    scratch[idx] = if count > 0 { sum / count as f32 } else { 0.0 };
                }
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

    /// AVX-optimized 3x3 mean filter.
    ///
    /// Processes 8 pixels per iteration in 256-bit registers: load the three
    /// neighboring rows, sum the nine taps, divide by 9. Columns past the
    /// last full chunk fall through to a scalar tail. Unaligned loads
    /// (`_mm256_loadu_ps`) are deliberate — the starting column (1) is not
    /// 32-byte aligned, and the penalty is negligible on modern x86_64.
    ///
    /// # Safety
    ///
    /// The caller must ensure that `current` and `scratch` slices have a
    /// length of at least `width * height` and that AVX is available on the
    /// target CPU. Widths too small for a full SIMD chunk are handled
    /// entirely by the scalar tail.
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

            let mut x = limit.max(1);
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

    /// NEON port of [`Self::diffuse_avx_impl`] (4-wide instead of 8-wide).
    ///
    /// # Safety
    ///
    /// The caller must ensure that `current` and `scratch` slices have a
    /// length of at least `width * height` and that NEON is available.
    /// Widths too small for a full SIMD chunk are handled by the scalar tail.
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

            let mut x = limit.max(1);
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

    /// Apply SIMD-optimized mean diffusion.
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
            // SAFETY: Neon feature detected, buffers sized at width*height.
            unsafe {
                Self::diffuse_neon_impl(current, scratch, width, height);
            }
            #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
            // SAFETY: AVX feature detected, buffers sized at width*height.
            unsafe {
                Self::diffuse_avx_impl(current, scratch, width, height);
            }
        } else {
            Self::diffuse_scalar_impl(current, scratch, width, height);
        }

        self.swap_buffers();
    }

    /// Apply 5x5 Gaussian diffusion.
    pub fn diffuse_gaussian(&mut self) {
        let width = self.width;
        let height = self.height;
        let scratch = &mut self.scratch;
        let kernel = &self.gaussian_kernel;
        let radius: i32 = 2;
        let is_wrap = matches!(self.boundary_mode, super::config::BoundaryMode::Wrap);

        scratch.copy_from_slice(&self.current);

        if is_wrap {
            // Toroidal: neighbors wrap across edges, all pixels processed
            let w = width as i32;
            let h = height as i32;
            let current = &self.current;
            for y in 0..height {
                for x in 0..width {
                    let idx = y * width + x;
                    let mut sum = 0.0f32;

                    for ky in -radius..=radius {
                        for kx in -radius..=radius {
                            let kernel_idx = ((ky + radius) * GAUSSIAN_KERNEL_SIZE as i32
                                + (kx + radius))
                                as usize;
                            let nx = ((x as i32 + kx % w) + w) % w;
                            let ny = ((y as i32 + ky % h) + h) % h;
                            sum += current[ny as usize * width + nx as usize] * kernel[kernel_idx];
                        }
                    }

                    scratch[idx] = sum;
                }
            }
        } else {
            // Bounce: leave the 2-pixel border undiffused
            let current = &self.current;
            for y in 2..height - 2 {
                let row_offset = y * width;
                for x in 2..width - 2 {
                    let idx = row_offset + x;

                    let mut sum = 0.0f32;

                    for ky in -radius..=radius {
                        for kx in -radius..=radius {
                            let nx = x as i32 + kx;
                            let ny = y as i32 + ky;
                            let kernel_idx = ((ky + radius) * GAUSSIAN_KERNEL_SIZE as i32
                                + (kx + radius))
                                as usize;
                            sum +=
                                current[(ny as usize) * width + (nx as usize)] * kernel[kernel_idx];
                        }
                    }

                    scratch[idx] = sum;
                }
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

    /// AVX-optimized 5x5 Gaussian blur.
    ///
    /// Same structure as [`Self::diffuse_avx_impl`], but with 25 weighted
    /// taps: the kernel weights are broadcast into registers up front, then
    /// each chunk of 8 pixels accumulates multiply-adds over the 5x5
    /// neighborhood. Remaining columns fall through to a scalar tail.
    ///
    /// # Safety
    ///
    /// The caller must ensure that:
    /// - `current` and `scratch` slices have length >= width * height
    /// - `width` >= 10 to accommodate AVX register width (8) + kernel boundary (2)
    /// - AVX is available on the target CPU (checked via is_x86_feature_detected!("avx"))
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    #[target_feature(enable = "avx")]
    unsafe fn diffuse_gaussian_avx_impl(
        current: &[f32],
        scratch: &mut [f32],
        width: usize,
        height: usize,
        kernel: &[f32; 25],
    ) {
        use std::arch::x86_64::*;

        // Broadcast all 25 kernel weights into registers up front
        let k0 = _mm256_set1_ps(kernel[0]);
        let k1 = _mm256_set1_ps(kernel[1]);
        let k2 = _mm256_set1_ps(kernel[2]);
        let k3 = _mm256_set1_ps(kernel[3]);
        let k4 = _mm256_set1_ps(kernel[4]);
        let k5 = _mm256_set1_ps(kernel[5]);
        let k6 = _mm256_set1_ps(kernel[6]);
        let k7 = _mm256_set1_ps(kernel[7]);
        let k8 = _mm256_set1_ps(kernel[8]);
        let k9 = _mm256_set1_ps(kernel[9]);
        let k10 = _mm256_set1_ps(kernel[10]);
        let k11 = _mm256_set1_ps(kernel[11]);
        let k12 = _mm256_set1_ps(kernel[12]);
        let k13 = _mm256_set1_ps(kernel[13]);
        let k14 = _mm256_set1_ps(kernel[14]);
        let k15 = _mm256_set1_ps(kernel[15]);
        let k16 = _mm256_set1_ps(kernel[16]);
        let k17 = _mm256_set1_ps(kernel[17]);
        let k18 = _mm256_set1_ps(kernel[18]);
        let k19 = _mm256_set1_ps(kernel[19]);
        let k20 = _mm256_set1_ps(kernel[20]);
        let k21 = _mm256_set1_ps(kernel[21]);
        let k22 = _mm256_set1_ps(kernel[22]);
        let k23 = _mm256_set1_ps(kernel[23]);
        let k24 = _mm256_set1_ps(kernel[24]);

        let simd_width = 8;
        // Leave room for kernel boundary on right side
        let limit = width.saturating_sub(simd_width + GAUSSIAN_RADIUS as usize);

        let mut y = GAUSSIAN_RADIUS as usize;

        while y < height - GAUSSIAN_RADIUS as usize {
            let current_row = y * width;
            let row_minus_2 = (y - GAUSSIAN_RADIUS as usize) * width;
            let row_minus_1 = (y - 1) * width;
            let row_plus_1 = (y + 1) * width;
            let row_plus_2 = (y + GAUSSIAN_RADIUS as usize) * width;

            let mut x = GAUSSIAN_RADIUS as usize;

            // Main SIMD loop - process 8 pixels at a time
            while x < limit {
                let idx = current_row + x;

                // Load 5x5 neighborhood around the 8-pixel chunk
                // Row y-2: x-2, x-1, x, x+1, x+2
                let m2_l = _mm256_loadu_ps(current.as_ptr().add(row_minus_2 + x - 2));
                let m2_m = _mm256_loadu_ps(current.as_ptr().add(row_minus_2 + x - 1));
                let m2_r = _mm256_loadu_ps(current.as_ptr().add(row_minus_2 + x));

                // Row y-1: x-2, x-1, x, x+1, x+2
                let m1_l = _mm256_loadu_ps(current.as_ptr().add(row_minus_1 + x - 2));
                let m1_m = _mm256_loadu_ps(current.as_ptr().add(row_minus_1 + x - 1));
                let m1_r = _mm256_loadu_ps(current.as_ptr().add(row_minus_1 + x));

                // Row y: x-2, x-1, x, x+1, x+2
                let p0_l = _mm256_loadu_ps(current.as_ptr().add(current_row + x - 2));
                let p0_m = _mm256_loadu_ps(current.as_ptr().add(current_row + x - 1));
                let p0_r = _mm256_loadu_ps(current.as_ptr().add(current_row + x));

                // Row y+1: x-2, x-1, x, x+1, x+2
                let p1_l = _mm256_loadu_ps(current.as_ptr().add(row_plus_1 + x - 2));
                let p1_m = _mm256_loadu_ps(current.as_ptr().add(row_plus_1 + x - 1));
                let p1_r = _mm256_loadu_ps(current.as_ptr().add(row_plus_1 + x));

                // Row y+2: x-2, x-1, x, x+1, x+2
                let p2_l = _mm256_loadu_ps(current.as_ptr().add(row_plus_2 + x - 2));
                let p2_m = _mm256_loadu_ps(current.as_ptr().add(row_plus_2 + x - 1));
                let p2_r = _mm256_loadu_ps(current.as_ptr().add(row_plus_2 + x));

                // Weighted sum, row by row
                // Row y-2: kernels 0-4
                let mut sum = _mm256_mul_ps(m2_l, k0);
                sum = _mm256_add_ps(sum, _mm256_mul_ps(m2_m, k1));
                sum = _mm256_add_ps(sum, _mm256_mul_ps(m2_r, k2));
                sum = _mm256_add_ps(
                    sum,
                    _mm256_mul_ps(
                        _mm256_loadu_ps(current.as_ptr().add(row_minus_2 + x + 1)),
                        k3,
                    ),
                );
                sum = _mm256_add_ps(
                    sum,
                    _mm256_mul_ps(
                        _mm256_loadu_ps(current.as_ptr().add(row_minus_2 + x + 2)),
                        k4,
                    ),
                );

                // Row y-1: kernels 5-9
                sum = _mm256_add_ps(sum, _mm256_mul_ps(m1_l, k5));
                sum = _mm256_add_ps(sum, _mm256_mul_ps(m1_m, k6));
                sum = _mm256_add_ps(sum, _mm256_mul_ps(m1_r, k7));
                sum = _mm256_add_ps(
                    sum,
                    _mm256_mul_ps(
                        _mm256_loadu_ps(current.as_ptr().add(row_minus_1 + x + 1)),
                        k8,
                    ),
                );
                sum = _mm256_add_ps(
                    sum,
                    _mm256_mul_ps(
                        _mm256_loadu_ps(current.as_ptr().add(row_minus_1 + x + 2)),
                        k9,
                    ),
                );

                // Row y: kernels 10-14
                sum = _mm256_add_ps(sum, _mm256_mul_ps(p0_l, k10));
                sum = _mm256_add_ps(sum, _mm256_mul_ps(p0_m, k11));
                sum = _mm256_add_ps(sum, _mm256_mul_ps(p0_r, k12));
                sum = _mm256_add_ps(
                    sum,
                    _mm256_mul_ps(
                        _mm256_loadu_ps(current.as_ptr().add(current_row + x + 1)),
                        k13,
                    ),
                );
                sum = _mm256_add_ps(
                    sum,
                    _mm256_mul_ps(
                        _mm256_loadu_ps(current.as_ptr().add(current_row + x + 2)),
                        k14,
                    ),
                );

                // Row y+1: kernels 15-19
                sum = _mm256_add_ps(sum, _mm256_mul_ps(p1_l, k15));
                sum = _mm256_add_ps(sum, _mm256_mul_ps(p1_m, k16));
                sum = _mm256_add_ps(sum, _mm256_mul_ps(p1_r, k17));
                sum = _mm256_add_ps(
                    sum,
                    _mm256_mul_ps(
                        _mm256_loadu_ps(current.as_ptr().add(row_plus_1 + x + 1)),
                        k18,
                    ),
                );
                sum = _mm256_add_ps(
                    sum,
                    _mm256_mul_ps(
                        _mm256_loadu_ps(current.as_ptr().add(row_plus_1 + x + 2)),
                        k19,
                    ),
                );

                // Row y+2: kernels 20-24
                sum = _mm256_add_ps(sum, _mm256_mul_ps(p2_l, k20));
                sum = _mm256_add_ps(sum, _mm256_mul_ps(p2_m, k21));
                sum = _mm256_add_ps(sum, _mm256_mul_ps(p2_r, k22));
                sum = _mm256_add_ps(
                    sum,
                    _mm256_mul_ps(
                        _mm256_loadu_ps(current.as_ptr().add(row_plus_2 + x + 1)),
                        k23,
                    ),
                );
                sum = _mm256_add_ps(
                    sum,
                    _mm256_mul_ps(
                        _mm256_loadu_ps(current.as_ptr().add(row_plus_2 + x + 2)),
                        k24,
                    ),
                );

                _mm256_storeu_ps(scratch.as_mut_ptr().add(idx), sum);

                x += simd_width;
            }

            // Scalar fallback for remaining columns
            let mut x = limit.max(GAUSSIAN_RADIUS as usize);
            while x < width - GAUSSIAN_RADIUS as usize {
                let idx = current_row + x;
                let mut sum = 0.0f32;

                for ky in -GAUSSIAN_RADIUS..=GAUSSIAN_RADIUS {
                    for kx in -GAUSSIAN_RADIUS..=GAUSSIAN_RADIUS {
                        let nx = x as i32 + kx;
                        let ny = y as i32 + ky;
                        let kernel_idx = ((ky + GAUSSIAN_RADIUS) * GAUSSIAN_KERNEL_SIZE as i32
                            + (kx + GAUSSIAN_RADIUS))
                            as usize;
                        sum += current[(ny as usize) * width + (nx as usize)] * kernel[kernel_idx];
                    }
                }

                scratch[idx] = sum;

                x += 1;
            }

            y += 1;
        }
    }

    /// NEON port of [`Self::diffuse_gaussian_avx_impl`] (4-wide instead of 8-wide).
    ///
    /// # Safety
    ///
    /// The caller must ensure that `current` and `scratch` slices have a
    /// length of at least `width * height` and that NEON is available.
    /// Widths too small for a full SIMD chunk are handled by the scalar tail.
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
        let limit = width.saturating_sub(simd_width + GAUSSIAN_RADIUS as usize);

        let mut y = GAUSSIAN_RADIUS as usize;

        while y < height - GAUSSIAN_RADIUS as usize {
            let current_row = y * width;
            let row_minus_1 = (y - 1) * width;
            let row_minus_2 = (y - GAUSSIAN_RADIUS as usize) * width;
            let row_plus_1 = (y + 1) * width;
            let row_plus_2 = (y + GAUSSIAN_RADIUS as usize) * width;

            let mut x = GAUSSIAN_RADIUS as usize;

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

            let mut x = limit.max(GAUSSIAN_RADIUS as usize);
            while x < width - GAUSSIAN_RADIUS as usize {
                let idx = current_row + x;
                let mut sum = 0.0f32;

                for ky in -GAUSSIAN_RADIUS..=GAUSSIAN_RADIUS {
                    for kx in -GAUSSIAN_RADIUS..=GAUSSIAN_RADIUS {
                        let nx = x as i32 + kx;
                        let ny = y as i32 + ky;
                        let kernel_idx = ((ky + GAUSSIAN_RADIUS) * GAUSSIAN_KERNEL_SIZE as i32
                            + (kx + GAUSSIAN_RADIUS))
                            as usize;
                        sum += current[(ny as usize) * width + (nx as usize)] * kernel[kernel_idx];
                    }
                }

                scratch[idx] = sum;

                x += 1;
            }

            y += 1;
        }
    }

    /// Apply SIMD-optimized Gaussian diffusion.
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

        if has_simd && width >= 10 {
            #[cfg(target_arch = "aarch64")]
            // SAFETY: Neon feature detected, buffers sized at width*height.
            unsafe {
                Self::diffuse_gaussian_neon_impl(current, scratch, width, height, kernel);
            }
            #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
            // SAFETY: AVX feature detected, buffers sized at width*height, width >= 10.
            unsafe {
                Self::diffuse_gaussian_avx_impl(current, scratch, width, height, kernel);
            }
        } else {
            Self::diffuse_gaussian_scalar_impl(current, scratch, width, height, kernel);
        }

        self.swap_buffers();
    }

    /// Separable Gaussian blur (two 1D passes) for the wider sigma range (sigma > 2.0).
    /// `O(2r)` per cell vs `O(r²)`. Uses bounce/wrap boundary consistent with the
    /// other diffusion paths. radius = ceil(3·sigma).
    pub fn diffuse_gaussian_separable(&mut self, sigma: f32) {
        let width = self.width;
        let height = self.height;
        let sigma = sigma.max(0.1);
        let radius = (3.0 * sigma).ceil() as i32;
        // Build normalized 1D kernel.
        let kernel_len = (2 * radius + 1) as usize;
        let mut kernel = Vec::with_capacity(kernel_len);
        let two_sigma2 = 2.0 * sigma * sigma;
        let mut ksum = 0.0f32;
        for k in -radius..=radius {
            let w = (-(k as f32 * k as f32) / two_sigma2).exp();
            kernel.push(w);
            ksum += w;
        }
        for w in &mut kernel {
            *w /= ksum;
        }
        let is_wrap = matches!(self.boundary_mode, super::config::BoundaryMode::Wrap);
        let clampi = |v: i32, n: i32| -> usize {
            if is_wrap {
                (((v % n) + n) % n) as usize
            } else {
                v.clamp(0, n - 1) as usize
            }
        };
        // Horizontal pass: current -> scratch.
        for y in 0..height {
            for x in 0..width {
                let mut sum = 0.0f32;
                for (ki, k) in (-radius..=radius).enumerate() {
                    let nx = clampi(x as i32 + k, width as i32);
                    sum += self.current[y * width + nx] * kernel[ki];
                }
                self.scratch[y * width + x] = sum;
            }
        }
        // Vertical pass: scratch -> current.
        for y in 0..height {
            for x in 0..width {
                let mut sum = 0.0f32;
                for (ki, k) in (-radius..=radius).enumerate() {
                    let ny = clampi(y as i32 + k, height as i32);
                    sum += self.scratch[ny * width + x] * kernel[ki];
                }
                self.current[y * width + x] = sum;
            }
        }
    }

    /// Dispatch diffusion, then apply the Lague diffuse-weight blend.
    ///
    /// `diffuse_weight == 1.0` ⇒ full blur (byte-identical to the pre-blend behavior,
    /// no snapshot taken).  `diffuse_weight == 0.0` ⇒ no-op.
    /// The blend `new = old·(1−w) + blur·w` is applied only when `0 < w < 1`.
    ///
    /// Routes to the separable path when `use_gaussian && sigma > 2.0`.
    pub fn diffuse_with_kernel(
        &mut self,
        use_simd: bool,
        use_gaussian: bool,
        diffuse_weight: f32,
        sigma: f32,
    ) {
        let w = diffuse_weight.clamp(0.0, 1.0);
        if w <= 0.0 {
            return; // no diffusion at all
        }

        // Snapshot the pre-blur trail for the blend only when needed (no alloc at w==1).
        if w < 1.0 {
            let size = self.width * self.height;
            if self.blend_src.len() != size {
                self.blend_src.resize(size, 0.0);
            }
            self.blend_src.copy_from_slice(&self.current);
        }

        // SIMD paths don't implement wrap-around, so wrap mode always takes the scalar path.
        let use_scalar =
            !use_simd || matches!(self.boundary_mode, super::config::BoundaryMode::Wrap);

        if use_gaussian && sigma > 2.0 {
            self.diffuse_gaussian_separable(sigma);
        } else if use_scalar {
            if use_gaussian {
                self.diffuse_gaussian();
            } else {
                self.diffuse();
            }
        } else if use_gaussian {
            self.diffuse_gaussian_simd();
        } else {
            self.diffuse_simd();
        }

        // Apply blend only for partial weight (w==1.0 fast path: no blend math).
        if w < 1.0 {
            for (cur, &src) in self.current.iter_mut().zip(self.blend_src.iter()) {
                *cur = src * (1.0 - w) + *cur * w;
            }
        }
    }

    /// Apply global exponential decay to all trail values.
    pub fn decay(&mut self, factor: f32) {
        for value in &mut self.current {
            *value *= factor;
        }
        self.trail_sum *= factor;
    }

    /// Value-dependent decay (lever 7). With `gamma == 1.0` this is identical to
    /// `decay(factor)`. With `gamma < 1.0`, faint cells decay less than bright cells
    /// (longer faint tails) by scaling the removed fraction by `(value/peak)^(1−γ)`.
    pub fn decay_gamma(&mut self, factor: f32, gamma: f32) {
        if (gamma - 1.0).abs() < f32::EPSILON {
            self.decay(factor);
            return;
        }
        let peak = self
            .current
            .iter()
            .copied()
            .fold(0.0f32, f32::max)
            .max(1e-6);
        let s = 1.0 - factor; // removed fraction at gamma=1
        let exp = 1.0 - gamma;
        let mut new_sum = 0.0f32;
        for value in &mut self.current {
            if *value > 0.0 {
                let norm = (*value / peak).clamp(0.0, 1.0);
                let removed_frac = (s * norm.powf(exp)).clamp(0.0, 1.0);
                *value *= 1.0 - removed_frac;
            }
            new_sum += *value;
        }
        self.trail_sum = new_sum;
    }

    /// Mutable access to the deposit-accumulation buffer, sized to the grid
    /// (zero-filled on first use / resize). Agents accumulate into this when
    /// the nonlinear-deposit path is active.
    pub fn accum_mut(&mut self) -> &mut [f32] {
        let size = self.width * self.height;
        if self.accum.len() != size {
            self.accum.resize(size, 0.0);
        }
        &mut self.accum
    }

    /// Fold the accumulated deposits into the current trail using the curve:
    /// `current[i] += min(curve(accum[i])·scale, cap)`; then zero the buffer.
    /// `cap <= 0.0` disables clamping. No-op if the buffer is unsized.
    pub fn fold_deposits(
        &mut self,
        curve: super::config::DepositCurve,
        scale: f32,
        gamma: f32,
        cap: f32,
    ) {
        let size = self.width * self.height;
        if self.accum.len() != size {
            return;
        }
        for (cur, acc) in self.current.iter_mut().zip(self.accum.iter_mut()) {
            if *acc != 0.0 {
                let mut v = curve.apply(*acc, gamma) * scale;
                if cap > 0.0 {
                    v = v.min(cap);
                }
                *cur += v;
                *acc = 0.0;
            }
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
    fn decay_gamma_one_matches_plain_decay() {
        let mut a = TrailMap::new(8, 8);
        let mut b = TrailMap::new(8, 8);
        for i in 0..64 {
            a.current_mut()[i] = (i as f32) * 0.1;
            b.current_mut()[i] = (i as f32) * 0.1;
        }
        a.decay(0.9);
        b.decay_gamma(0.9, 1.0);
        for (x, y) in a.current().iter().zip(b.current().iter()) {
            assert!((x - y).abs() < 1e-6, "gamma=1 must equal plain decay");
        }
    }

    #[test]
    fn decay_gamma_below_one_preserves_faint_more_than_baseline() {
        let mut base = TrailMap::new(8, 8);
        let mut g = TrailMap::new(8, 8);
        // a faint cell and a bright (peak) cell
        base.current_mut()[0] = 0.1;
        base.current_mut()[1] = 1.0;
        g.current_mut()[0] = 0.1;
        g.current_mut()[1] = 1.0;
        base.decay(0.9);
        g.decay_gamma(0.9, 0.5);
        // faint cell retains MORE with gamma<1; peak cell ~unchanged.
        assert!(g.current()[0] > base.current()[0], "faint persists longer");
        assert!(
            (g.current()[1] - base.current()[1]).abs() < 1e-3,
            "peak ~unchanged"
        );
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

        // weight=1.0, sigma<=2.0 → must be byte-identical to diffuse()
        trail1.diffuse_with_kernel(false, false, 1.0, 1.0);
        trail2.diffuse();

        assert_eq!(trail1.current(), trail2.current());

        trail1.clear();
        trail2.clear();
        trail1.set(5, 5, 9.0);
        trail2.set(5, 5, 9.0);
        // weight=1.0, sigma<=2.0, gaussian → byte-identical to diffuse_gaussian()
        trail1.diffuse_with_kernel(false, true, 1.0, 1.0);
        trail2.diffuse_gaussian();
        assert_eq!(trail1.current(), trail2.current());
    }

    #[test]
    fn test_set_gaussian_sigma() {
        let mut trail = TrailMap::new(10, 10);
        let k1 = trail.gaussian_kernel;
        trail.set_gaussian_sigma(2.0);
        let k2 = trail.gaussian_kernel;
        assert_ne!(k1, k2);
    }

    #[test]
    fn test_new_with_sigma() {
        let trail = TrailMap::new_with_sigma(10, 10, 2.0);
        assert_eq!(trail.width(), 10);
        let default_kernel = generate_gaussian_kernel(1.0);
        assert_ne!(trail.gaussian_kernel, default_kernel);
    }

    #[test]
    fn test_current_mut() {
        let mut trail = TrailMap::new(10, 10);
        trail.current_mut()[0] = 1.0;
        assert_eq!(trail.get(0, 0), 1.0);
    }

    #[test]
    fn test_scratch_access() {
        let mut trail = TrailMap::new(10, 10);
        trail.scratch_mut()[0] = 2.0;
        assert_eq!(trail.scratch()[0], 2.0);
    }

    #[test]
    fn test_trail_sum() {
        let mut trail = TrailMap::new(10, 10);
        trail.add(0, 0, 1.0);
        trail.add(1, 1, 2.0);
        assert_eq!(trail.trail_sum(), 3.0);
        trail.decay(0.5);
        assert_eq!(trail.trail_sum(), 1.5);
        trail.clear();
        assert_eq!(trail.trail_sum(), 0.0);
    }

    #[test]
    fn test_diffuse_simd_fallback() {
        let mut trail = TrailMap::new(20, 20);
        trail.set(10, 10, 9.0);
        trail.diffuse_simd();
        assert!(trail.get(10, 10) < 9.0);
    }

    #[test]
    fn test_diffuse_gaussian_simd_fallback() {
        let mut trail = TrailMap::new(20, 20);
        trail.set(10, 10, 9.0);
        trail.diffuse_gaussian_simd();
        assert!(trail.get(10, 10) < 9.0);
    }
    #[test]
    fn test_diffuse_simd_small_width() {
        let mut trail = TrailMap::new(4, 10);
        trail.set(1, 1, 9.0);
        trail.diffuse_simd();
        // Just ensure it doesn't panic
        assert!(trail.get(1, 1) < 9.0);
    }

    #[test]
    fn test_diffuse_gaussian_simd_small_width() {
        let mut trail = TrailMap::new(4, 10);
        trail.set(1, 1, 9.0);
        trail.diffuse_gaussian_simd();
        // Just ensure it doesn't panic.
        // With width 4 and 5x5 kernel (radius 2), no pixels are processed as "inner" pixels.
        // So the value should remain unchanged.
        assert_eq!(trail.get(1, 1), 9.0);
    }

    #[test]
    fn fold_deposits_applies_curve_scale_cap_and_clears() {
        use crate::simulation::config::DepositCurve;
        let mut tm = TrailMap::new(2, 1);
        {
            let accum = tm.accum_mut();
            accum[0] = 9.0;
            accum[1] = 100.0;
        }
        // sqrt(9)=3 *2 = 6 ; sqrt(100)=10 *2 = 20 -> capped at 8
        tm.fold_deposits(DepositCurve::Sqrt, 2.0, 1.0, 8.0);
        let cur = tm.current();
        assert!((cur[0] - 6.0).abs() < 1e-5);
        assert!((cur[1] - 8.0).abs() < 1e-5);
        // buffer cleared after fold
        assert_eq!(tm.accum_mut(), &[0.0, 0.0]);
    }

    #[test]
    fn accum_mut_initializes_zeroed() {
        let mut tm = TrailMap::new(3, 2);
        assert_eq!(tm.accum_mut(), &[0.0; 6]);
    }

    #[test]
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    fn test_diffuse_gaussian_avx_matches_scalar() {
        use std::arch::is_x86_feature_detected;

        // Skip test if AVX not available
        if !is_x86_feature_detected!("avx") {
            return;
        }

        let width = 64;
        let height = 64;

        // Create two trail maps with identical initial data
        let mut trail_avx = TrailMap::new(width, height);
        let mut trail_scalar = TrailMap::new(width, height);

        // Initialize with test pattern
        for y in 0..height {
            for x in 0..width {
                let value = ((x + y * width) % 100) as f32 / 100.0;
                trail_avx.set(x, y, value);
                trail_scalar.set(x, y, value);
            }
        }

        // Apply AVX Gaussian diffusion directly
        let kernel = trail_avx.gaussian_kernel;
        unsafe {
            TrailMap::diffuse_gaussian_avx_impl(
                &trail_avx.current,
                &mut trail_avx.scratch,
                width,
                height,
                &kernel,
            );
        }
        trail_avx.swap_buffers();

        // Apply scalar Gaussian diffusion
        TrailMap::diffuse_gaussian_scalar_impl(
            &trail_scalar.current,
            &mut trail_scalar.scratch,
            width,
            height,
            &kernel,
        );
        trail_scalar.swap_buffers();

        // Compare results (allow small floating point differences)
        let avx_data = trail_avx.current();
        let scalar_data = trail_scalar.current();

        for i in 0..width * height {
            let diff = (avx_data[i] - scalar_data[i]).abs();
            assert!(
                diff < 1e-4,
                "Mismatch at index {}: AVX={}, Scalar={}, diff={}",
                i,
                avx_data[i],
                scalar_data[i],
                diff
            );
        }
    }
}

#[cfg(test)]
mod prop_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn test_diffuse_always_finite_and_non_negative(
            values in proptest::collection::vec(0.0..100.0f32, 100)
        ) {
            let mut trail = TrailMap::new(10, 10);
            for (i, &v) in values.iter().enumerate() {
                trail.current_mut()[i] = v;
            }
            trail.diffuse();
            for &v in trail.current() {
                prop_assert!(v.is_finite());
                prop_assert!(v >= 0.0);
            }
        }

        #[test]
        fn test_diffuse_gaussian_always_finite_and_non_negative(
            values in proptest::collection::vec(0.0..100.0f32, 100)
        ) {
            let mut trail = TrailMap::new(10, 10);
            for (i, &v) in values.iter().enumerate() {
                trail.current_mut()[i] = v;
            }
            trail.diffuse_gaussian();
            for &v in trail.current() {
                prop_assert!(v.is_finite());
                prop_assert!(v >= 0.0);
            }
        }
    }

    #[test]
    fn diffuse_weight_zero_is_noop() {
        let mut tm = TrailMap::new(16, 16);
        tm.current_mut()[8 * 16 + 8] = 1.0;
        let before = tm.current().to_vec();
        tm.diffuse_with_kernel(false, false, 0.0, 1.0); // weight=0 ⇒ no change
        assert_eq!(
            tm.current(),
            &before[..],
            "weight=0 must leave the trail unchanged"
        );
    }

    #[test]
    fn separable_gaussian_conserves_mass_and_spreads() {
        let mut tm = TrailMap::new(32, 32);
        tm.current_mut()[16 * 32 + 16] = 9.0;
        let sum_before: f32 = tm.current().iter().sum();
        tm.diffuse_gaussian_separable(3.0);
        let sum_after: f32 = tm.current().iter().sum();
        assert!((sum_before - sum_after).abs() < 0.5, "blur ~conserves mass");
        assert!(
            tm.current()[16 * 32 + 16] < 9.0,
            "energy spread out of the center"
        );
        assert!(tm.current()[16 * 32 + 17] > 0.0, "neighbor received energy");
    }
}
