use criterion::{black_box, criterion_group, criterion_main, Criterion};

pub struct TrailHistory {
    history: Vec<Vec<f32>>,
    capacity: usize,
    current_index: usize,
    count: usize,
}

impl TrailHistory {
    pub fn new(capacity: usize) -> Self {
        Self {
            history: Vec::with_capacity(capacity),
            capacity,
            current_index: 0,
            count: 0,
        }
    }

    pub fn push(&mut self, trail_map: &[f32]) {
        if self.capacity == 0 {
            return;
        }

        if self.history.len() < self.capacity {
            self.history.push(trail_map.to_vec());
            self.count = self.history.len();
        } else {
            self.history[self.current_index].copy_from_slice(trail_map);
        }

        self.current_index = (self.current_index + 1) % self.capacity;
    }

    pub fn blended(&self) -> Option<Vec<f32>> {
        if self.count == 0 {
            return None;
        }

        let mut result = vec![0.0f32; self.history[0].len()];
        for frame in &self.history[..self.count] {
            for (i, &val) in frame.iter().enumerate() {
                result[i] += val;
            }
        }

        let weight = 1.0 / self.count as f32;
        for val in &mut result {
            *val *= weight;
        }

        Some(result)
    }
}

fn bench_trail_history_push(c: &mut Criterion) {
    let mut history = TrailHistory::new(10);

    let trail_data: Vec<f32> = (0..400 * 400).map(|i| (i % 100) as f32 / 10.0).collect();

    c.bench_function("trail_history_push_10_frames", |b| {
        b.iter(|| {
            history.push(black_box(&trail_data));
            black_box(&history);
        });
    });
}

fn bench_trail_history_blend_various_sizes(c: &mut Criterion) {
    let mut group = c.benchmark_group("trail_history_blend");

    for size in [3, 5, 10] {
        group.bench_function(format!("blend_{}_frames", size), |b| {
            b.iter(|| {
                let mut history = TrailHistory::new(size);
                let trail_data: Vec<f32> =
                    (0..400 * 400).map(|i| (i % 100) as f32 / 10.0).collect();

                for _ in 0..size {
                    history.push(&trail_data);
                }

                let blended = history.blended().unwrap();
                black_box(blended);
            });
        });
    }

    group.finish();
}

fn bench_trail_history_overhead_comparison(c: &mut Criterion) {
    let trail_data: Vec<f32> = (0..400 * 400).map(|i| (i % 100) as f32 / 10.0).collect();

    let mut group = c.benchmark_group("trail_history_overhead");

    group.bench_function("no_history", |b| {
        b.iter(|| {
            let sum: f32 = trail_data.iter().copied().sum();
            black_box(sum);
        });
    });

    for history_size in [3, 5, 10] {
        group.bench_function(format!("with_history_{}", history_size), |b| {
            b.iter(|| {
                let mut history = TrailHistory::new(history_size);

                for _ in 0..history_size {
                    history.push(&trail_data);
                }

                let blended = history.blended().unwrap();
                black_box(blended);
            });
        });
    }

    group.finish();
}

fn bench_trail_history_blend_performance(c: &mut Criterion) {
    let trail_data: Vec<f32> = (0..400 * 400).map(|i| (i % 100) as f32 / 10.0).collect();

    c.bench_function("trail_history_blend_400x400", |b| {
        b.iter(|| {
            let mut history = TrailHistory::new(10);

            for _ in 0..10 {
                history.push(&trail_data);
            }

            let blended = history.blended().unwrap();
            black_box(blended);
        });
    });
}

criterion_group!(
    benches,
    bench_trail_history_push,
    bench_trail_history_blend_various_sizes,
    bench_trail_history_overhead_comparison,
    bench_trail_history_blend_performance
);
criterion_main!(benches);
