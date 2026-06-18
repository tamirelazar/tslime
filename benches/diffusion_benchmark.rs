use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_diffuse_mean3x3(c: &mut Criterion) {
    let width = 400;
    let height = 400;
    let size = width * height;

    let mut data = vec![0.0f32; size];
    for i in 0..width {
        for j in 0..height {
            data[i * width + j] = (i * j % 100) as f32 / 10.0;
        }
    }

    c.bench_function("diffuse_mean3x3", |b| {
        b.iter(|| {
            let mut scratch = data.clone();

            for y in 1..height - 1 {
                let row_offset = y * width;
                for x in 1..width - 1 {
                    let idx = row_offset + x;
                    let mut sum = 0.0f32;
                    let mut count = 0;

                    for dy in -1i32..=1 {
                        for dx in -1i32..=1 {
                            let nx = (x as i32 + dx) as usize;
                            let ny = (y as i32 + dy) as usize;
                            if nx < width && ny < height {
                                sum += data[ny * width + nx];
                                count += 1;
                            }
                        }
                    }
                    scratch[idx] = sum / count as f32;
                }
            }
            black_box(&scratch);
        });
    });
}

fn bench_diffuse_gaussian(c: &mut Criterion) {
    let width = 400;
    let height = 400;
    let size = width * height;

    let mut data = vec![0.0f32; size];
    for i in 0..width {
        for j in 0..height {
            data[i * width + j] = (i * j % 100) as f32 / 10.0;
        }
    }

    let kernel_size = 5usize;
    let radius = 2i32;
    let sigma = 1.0f32;
    let two_sigma_sq = 2.0 * sigma * sigma;

    let mut kernel = [0.0f32; 25];
    let mut sum = 0.0f32;

    for y in -radius..=radius {
        for x in -radius..=radius {
            let idx = ((y + radius) * kernel_size as i32 + (x + radius)) as usize;
            let dist_sq = (x * x + y * y) as f32;
            kernel[idx] = (-dist_sq / two_sigma_sq).exp();
            sum += kernel[idx];
        }
    }

    for k_val in kernel.iter_mut() {
        *k_val /= sum;
    }

    c.bench_function("diffuse_gaussian", |b| {
        b.iter(|| {
            let mut scratch = data.clone();

            for y in 2..height - 2 {
                let row_offset = y * width;
                for x in 2..width - 2 {
                    let idx = row_offset + x;
                    let mut conv_sum = 0.0f32;

                    for ky in -radius..=radius {
                        for kx in -radius..=radius {
                            let nx = (x as i32 + kx) as usize;
                            let ny = (y as i32 + ky) as usize;
                            let kernel_idx =
                                ((ky + radius) * kernel_size as i32 + (kx + radius)) as usize;
                            conv_sum += data[ny * width + nx] * kernel[kernel_idx];
                        }
                    }
                    scratch[idx] = conv_sum;
                }
            }
            black_box(&scratch);
        });
    });
}

fn bench_diffuse_comparison(c: &mut Criterion) {
    let width = 400;
    let height = 400;
    let size = width * height;

    let mut data_vec = vec![0.0f32; size];
    for i in 0..width {
        for j in 0..height {
            data_vec[i * width + j] = (i * j % 100) as f32 / 10.0;
        }
    }

    let kernel_size = 5usize;
    let radius = 2i32;
    let sigma = 1.0f32;
    let two_sigma_sq = 2.0 * sigma * sigma;

    let mut kernel = [0.0f32; 25];
    let mut sum = 0.0f32;

    for y in -radius..=radius {
        for x in -radius..=radius {
            let idx = ((y + radius) * kernel_size as i32 + (x + radius)) as usize;
            let dist_sq = (x * x + y * y) as f32;
            kernel[idx] = (-dist_sq / two_sigma_sq).exp();
            sum += kernel[idx];
        }
    }

    for k_val in kernel.iter_mut() {
        *k_val /= sum;
    }

    let mut group = c.benchmark_group("diffusion_comparison");

    group.bench_function("mean3x3_scalar", |b| {
        b.iter(|| {
            let mut scratch = data_vec.clone();

            for y in 1..height - 1 {
                let row_offset = y * width;
                for x in 1..width - 1 {
                    let idx = row_offset + x;
                    let mut sum = 0.0f32;
                    let mut count = 0;

                    for dy in -1i32..=1 {
                        for dx in -1i32..=1 {
                            let nx = (x as i32 + dx) as usize;
                            let ny = (y as i32 + dy) as usize;
                            if nx < width && ny < height {
                                sum += data_vec[ny * width + nx];
                                count += 1;
                            }
                        }
                    }
                    scratch[idx] = sum / count as f32;
                }
            }
            black_box(&scratch);
        });
    });

    group.bench_function("gaussian_scalar", |b| {
        b.iter(|| {
            let mut scratch = data_vec.clone();

            for y in 2..height - 2 {
                let row_offset = y * width;
                for x in 2..width - 2 {
                    let idx = row_offset + x;
                    let mut conv_sum = 0.0f32;

                    for ky in -radius..=radius {
                        for kx in -radius..=radius {
                            let nx = (x as i32 + kx) as usize;
                            let ny = (y as i32 + ky) as usize;
                            let kernel_idx =
                                ((ky + radius) * kernel_size as i32 + (kx + radius)) as usize;
                            conv_sum += data_vec[ny * width + nx] * kernel[kernel_idx];
                        }
                    }
                    scratch[idx] = conv_sum;
                }
            }
            black_box(&scratch);
        });
    });

    group.finish();
}

fn bench_diffuse_gaussian_separable_sigma3(c: &mut Criterion) {
    use tslime::simulation::trail_map::TrailMap;

    let width = 400;
    let height = 400;
    let mut tm = TrailMap::new(width, height);
    let data = tm.current_mut();
    for (i, v) in data.iter_mut().enumerate() {
        *v = (i * 7 % 100) as f32 / 10.0;
    }

    c.bench_function("diffuse_gaussian_separable_sigma3", |b| {
        b.iter(|| {
            tm.diffuse_gaussian_separable(black_box(3.0));
        });
    });
}

fn bench_decay_gamma(c: &mut Criterion) {
    use tslime::simulation::trail_map::TrailMap;

    let width = 400;
    let height = 400;
    let mut tm = TrailMap::new(width, height);
    let data = tm.current_mut();
    for (i, v) in data.iter_mut().enumerate() {
        *v = (i * 7 % 100) as f32 / 10.0;
    }

    c.bench_function("decay_gamma_0.9_0.5", |b| {
        b.iter(|| {
            tm.decay_gamma(black_box(0.9), black_box(0.5));
        });
    });
}

fn bench_afterglow_ema_pass(c: &mut Criterion) {
    use tslime::simulation::config::{InitMode, SimConfig, SpeciesConfig};
    use tslime::Simulation;

    let config = SimConfig {
        afterglow: 0.4,
        afterglow_rate: 0.05,
        species_configs: vec![SpeciesConfig {
            count: 100,
            ..Default::default()
        }],
        ..Default::default()
    };
    let mut sim = Simulation::new(400, 400, config, 42, InitMode::Random, 0);
    // Enable afterglow EMA buffer (normally done by app/runner when afterglow > 0).
    sim.set_compute_afterglow(true, 0.05);
    // Warm up the trail map so the EMA has non-zero values to process.
    for _ in 0..10 {
        sim.update(1.0);
    }

    c.bench_function("afterglow_ema_pass", |b| {
        b.iter(|| {
            sim.update(black_box(1.0));
        });
    });
}

criterion_group!(
    benches,
    bench_diffuse_mean3x3,
    bench_diffuse_gaussian,
    bench_diffuse_comparison,
    bench_diffuse_gaussian_separable_sigma3,
    bench_decay_gamma,
    bench_afterglow_ema_pass
);
criterion_main!(benches);
