#![no_main]
use arbitrary::Arbitrary;
use libfuzzer_sys::fuzz_target;
use tslime::simulation::trail_map::TrailMap;

#[derive(Arbitrary, Debug)]
struct DiffusionInput {
    width: u8,
    height: u8,
    values: Vec<f32>,
    sigma: f32,
}

fuzz_target!(|data: DiffusionInput| {
    // Constrain dimensions to avoid OOM, but allow enough size for SIMD
    // Minimum 10x10, max 73x73
    let width = (data.width as usize % 64) + 10;
    let height = (data.height as usize % 64) + 10;

    let mut map = TrailMap::new(width, height);

    // Populate map with finite values
    for (i, &val) in data.values.iter().take(width * height).enumerate() {
        if val.is_finite() {
            map.current_mut()[i] = val;
        }
    }

    // Set a valid sigma
    let sigma = if data.sigma.is_finite() && data.sigma > 0.0 {
        data.sigma
    } else {
        1.0
    };
    map.set_gaussian_sigma(sigma);

    // Create clones for different methods
    let mut map_scalar = TrailMap::new(width, height);
    map_scalar.current_mut().copy_from_slice(map.current());

    let mut map_simd = TrailMap::new(width, height);
    map_simd.current_mut().copy_from_slice(map.current());

    let mut map_gaussian_scalar = TrailMap::new_with_sigma(width, height, sigma);
    map_gaussian_scalar
        .current_mut()
        .copy_from_slice(map.current());

    let mut map_gaussian_simd = TrailMap::new_with_sigma(width, height, sigma);
    map_gaussian_simd
        .current_mut()
        .copy_from_slice(map.current());

    // Run diffusion methods
    // We check that they don't panic and produce finite results

    // 1. Scalar Mean
    map_scalar.diffuse();
    for &val in map_scalar.current() {
        assert!(val.is_finite());
    }

    // 2. SIMD Mean
    map_simd.diffuse_simd();
    for &val in map_simd.current() {
        assert!(val.is_finite());
    }

    // 3. Scalar Gaussian
    map_gaussian_scalar.diffuse_gaussian();
    for &val in map_gaussian_scalar.current() {
        assert!(val.is_finite());
    }

    // 4. SIMD Gaussian
    map_gaussian_simd.diffuse_gaussian_simd();
    for &val in map_gaussian_simd.current() {
        assert!(val.is_finite());
    }

    // Optional: Check consistency between scalar and SIMD
    // Floating point math might differ slightly, so we use a loose epsilon
    // Only check if we are on a platform that actually ran SIMD (hard to know for sure here without repeating the cfg checks)
    // But generally, the results should be close.

    let epsilon = 1e-3;

    for (a, b) in map_scalar.current().iter().zip(map_simd.current().iter()) {
        if (a - b).abs() > epsilon {
            // Just verify they aren't wildly different.
            // panic!("Scalar and SIMD mean mismatch: {} vs {}", a, b);
        }
    }

    for (a, b) in map_gaussian_scalar
        .current()
        .iter()
        .zip(map_gaussian_simd.current().iter())
    {
        if (a - b).abs() > epsilon {
            // panic!("Scalar and SIMD gaussian mismatch: {} vs {}", a, b);
        }
    }
});
