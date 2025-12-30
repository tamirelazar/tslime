# Diffusion Benchmark

This benchmark compares the performance of the 3x3 mean filter (box blur) vs 5x5 Gaussian blur diffusion kernels.

## Running the benchmark

```bash
cargo bench --bench diffusion_benchmark
```

## Results

The benchmark will generate a report in `target/criterion` directory comparing:

- `diffuse_mean3x3`: 3x3 mean filter (box blur) - O(9) operations per pixel
- `diffuse_gaussian`: 5x5 Gaussian blur - O(25) operations per pixel

Expected performance:
- Mean3x3 should be faster due to fewer kernel operations (9 vs 25)
- Gaussian provides smoother, more natural pheromone spreading
