use criterion::{black_box, criterion_group, criterion_main, Criterion};

pub struct TrailHistory {
    history: Vec<Vec<f32>>,
    capacity: usize,
    current_index: usize,
    count: usize,
    frame_size: usize,
    blended_buffer: Vec<f32>,
}

impl TrailHistory {
    pub fn new(capacity: usize, frame_size: usize) -> Self {
        let mut history = Vec::with_capacity(capacity);
        for _ in 0..capacity {
            history.push(vec![0.0f32; frame_size]);
        }

        Self {
            history,
            capacity,
            current_index: 0,
            count: 0,
            frame_size,
            blended_buffer: vec![0.0f32; frame_size],
        }
    }

    pub fn push(&mut self, trail_map: &[f32]) {
        if self.capacity == 0 {
            return;
        }

        self.history[self.current_index].copy_from_slice(trail_map);
        self.current_index = (self.current_index + 1) % self.capacity;

        if self.count < self.capacity {
            self.count += 1;
        }
    }

    pub fn blended(&mut self) -> Option<&[f32]> {
        if self.count == 0 {
            return None;
        }

        self.blended_buffer.fill(0.0);

        for frame in &self.history[..self.count] {
            for (i, &val) in frame.iter().enumerate() {
                self.blended_buffer[i] += val;
            }
        }

        let weight = 1.0 / self.count as f32;
        for val in &mut self.blended_buffer {
            *val *= weight;
        }

        Some(&self.blended_buffer)
    }
}

fn bench_trail_history_push(c: &mut Criterion) {
    let trail_data: Vec<f32> = (0..400 * 400).map(|i| (i % 100) as f32 / 10.0).collect();
    let mut history = TrailHistory::new(10, trail_data.len());

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
                let trail_data: Vec<f32> =
                    (0..400 * 400).map(|i| (i % 100) as f32 / 10.0).collect();
                let mut history = TrailHistory::new(size, trail_data.len());

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
                let mut history = TrailHistory::new(history_size, trail_data.len());

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
            let mut history = TrailHistory::new(10, trail_data.len());

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
