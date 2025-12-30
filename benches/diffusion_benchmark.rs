use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_diffuse_mean3x3(c: &mut Criterion) {
    let data = {
        let mut d = vec![0.0f32; 400 * 400];
        for i in 0..400 {
            for j in 0..400 {
                d[i * 400 + j] = (i * j % 100) as f32 / 10.0;
            }
        }
        d
    };

    c.bench_function("diffuse_mean3x3", |b| {
        b.iter(|| {
            let mut scratch = data.clone();
            let width = 400;
            let height = 400;

            for y in 1..height - 1 {
                let row_offset = y * width;
                for x in 1..width - 1 {
                    let idx = row_offset + x;
                    let mut sum = 0.0f32;
                    let mut count = 0;

                    for dy in -1i32..=1 {
                        for dx in -1i32..=1 {
                            let nx = x as i32 + dx;
                            let ny = y as i32 + dy;
                            if nx >= 0 && nx < width as i32 && ny >= 0 && ny < height as i32 {
                                sum += data[(ny as usize) * width + (nx as usize)];
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
    let data = {
        let mut d = vec![0.0f32; 400 * 400];
        for i in 0..400 {
            for j in 0..400 {
                d[i * 400 + j] = (i * j % 100) as f32 / 10.0;
            }
        }
        d
    };

    let kernel_size = 5;
    let radius: i32 = 2;
    let sigma = 1.0f32;
    let two_sigma_sq = 2.0 * sigma * sigma;

    let kernel = {
        let mut k = [0.0f32; 25];
        let mut sum = 0.0f32;

        for y in -radius..=radius {
            for x in -radius..=radius {
                let idx = ((y + radius) * kernel_size + (x + radius)) as usize;
                let dist_sq = (x * x + y * y) as f32;
                k[idx] = (-dist_sq / two_sigma_sq).exp();
                sum += k[idx];
            }
        }

        for k_val in k.iter_mut() {
            *k_val /= sum;
        }
        k
    };

    c.bench_function("diffuse_gaussian", |b| {
        b.iter(|| {
            let mut scratch = data.clone();
            let width = 400;
            let height = 400;

            for y in 2..height - 2 {
                let row_offset = y * width;
                for x in 2..width - 2 {
                    let idx = row_offset + x;
                    let mut conv_sum = 0.0f32;

                    for ky in -radius..=radius {
                        for kx in -radius..=radius {
                            let nx = x as i32 + kx;
                            let ny = y as i32 + ky;
                            let kernel_idx = ((ky + radius) * kernel_size + (kx + radius)) as usize;
                            conv_sum +=
                                data[(ny as usize) * width + (nx as usize)] * kernel[kernel_idx];
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
    let data = {
        let mut d = vec![0.0f32; 400 * 400];
        for i in 0..400 {
            for j in 0..400 {
                d[i * 400 + j] = (i * j % 100) as f32 / 10.0;
            }
        }
        d
    };

    let kernel_size = 5;
    let radius: i32 = 2;
    let sigma = 1.0f32;
    let two_sigma_sq = 2.0 * sigma * sigma;

    let kernel = {
        let mut k = [0.0f32; 25];
        let mut sum = 0.0f32;

        for y in -radius..=radius {
            for x in -radius..=radius {
                let idx = ((y + radius) * kernel_size + (x + radius)) as usize;
                let dist_sq = (x * x + y * y) as f32;
                k[idx] = (-dist_sq / two_sigma_sq).exp();
                sum += k[idx];
            }
        }

        for k_val in k.iter_mut() {
            *k_val /= sum;
        }
        k
    };

    let mut group = c.benchmark_group("diffusion_comparison");

    group.bench_function("mean3x3", |b| {
        b.iter(|| {
            let mut scratch = data.clone();
            let width = 400;
            let height = 400;

            for y in 1..height - 1 {
                let row_offset = y * width;
                for x in 1..width - 1 {
                    let idx = row_offset + x;
                    let mut sum = 0.0f32;
                    let mut count = 0;

                    for dy in -1i32..=1 {
                        for dx in -1i32..=1 {
                            let nx = x as i32 + dx;
                            let ny = y as i32 + dy;
                            if nx >= 0 && nx < width as i32 && ny >= 0 && ny < height as i32 {
                                sum += data[(ny as usize) * width + (nx as usize)];
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

    group.bench_function("gaussian", |b| {
        b.iter(|| {
            let mut scratch = data.clone();
            let width = 400;
            let height = 400;

            for y in 2..height - 2 {
                let row_offset = y * width;
                for x in 2..width - 2 {
                    let idx = row_offset + x;
                    let mut conv_sum = 0.0f32;

                    for ky in -radius..=radius {
                        for kx in -radius..=radius {
                            let nx = x as i32 + kx;
                            let ny = y as i32 + ky;
                            let kernel_idx = ((ky + radius) * kernel_size + (kx + radius)) as usize;
                            conv_sum +=
                                data[(ny as usize) * width + (nx as usize)] * kernel[kernel_idx];
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

criterion_group!(
    benches,
    bench_diffuse_mean3x3,
    bench_diffuse_gaussian,
    bench_diffuse_comparison
);
criterion_main!(benches);
