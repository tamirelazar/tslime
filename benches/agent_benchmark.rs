use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_agent_sense_rotate_move_deposit(c: &mut Criterion) {
    #[derive(Clone, Copy)]
    struct Agent {
        x: f32,
        y: f32,
        heading: f32,
    }

    impl Agent {
        fn new(x: f32, y: f32, heading: f32) -> Self {
            Self { x, y, heading }
        }

        fn sense(
            &self,
            trail: &[f32],
            width: usize,
            height: usize,
            sensor_angle: f32,
            sensor_distance: f32,
        ) -> (f32, f32, f32) {
            let sensor_offset = sensor_distance;
            let left_angle = self.heading - sensor_angle;
            let right_angle = self.heading + sensor_angle;

            let left_x = self.x + left_angle.cos() * sensor_offset;
            let left_y = self.y + left_angle.sin() * sensor_offset;
            let center_x = self.x + self.heading.cos() * sensor_offset;
            let center_y = self.y + self.heading.sin() * sensor_offset;
            let right_x = self.x + right_angle.cos() * sensor_offset;
            let right_y = self.y + right_angle.sin() * sensor_offset;

            let left_idx = (left_y as usize).min(height - 1) * width + (left_x as usize).min(width - 1);
            let center_idx = (center_y as usize).min(height - 1) * width + (center_x as usize).min(width - 1);
            let right_idx = (right_y as usize).min(height - 1) * width + (right_x as usize).min(width - 1);

            (
                trail.get(left_idx).copied().unwrap_or(0.0),
                trail.get(center_idx).copied().unwrap_or(0.0),
                trail.get(right_idx).copied().unwrap_or(0.0),
            )
        }

        fn rotate(&mut self, left: f32, center: f32, right: f32, rotation_angle: f32, _rng: &mut impl rand::Rng) {
            if center > left && center > right {
            } else if center < left && center < right {
                self.heading += if _rng.gen_bool(0.5) { rotation_angle } else { -rotation_angle };
            } else if left > right {
                self.heading -= rotation_angle;
            } else if right > left {
                self.heading += rotation_angle;
            }
        }

        fn move_forward(&mut self, step_size: f32, width: usize, height: usize) {
            self.x = (self.x + step_size * self.heading.cos()).abs();
            self.y = (self.y + step_size * self.heading.sin()).abs();
            if self.x >= width as f32 { self.x -= width as f32; }
            if self.y >= height as f32 { self.y -= height as f32; }
        }

        fn deposit(&self, trail: &mut [f32], width: usize, height: usize, amount: f32) {
            let idx = (self.y as usize).min(height - 1) * width + (self.x as usize).min(width - 1);
            if let Some(cell) = trail.get_mut(idx) {
                *cell = (*cell + amount).min(1000.0);
            }
        }
    }

    let trail_map = {
        let mut d = vec![0.0f32; 400 * 400];
        for i in 0..400 {
            for j in 0..400 {
                d[i * 400 + j] = ((i * j) % 100) as f32 / 10.0;
            }
        }
        d
    };

    let agents: Vec<Agent> = (0..50000)
        .map(|i| {
            let angle = (i as f32 / 50000.0) * std::f32::consts::PI * 2.0;
            Agent::new(200.0 + angle.cos() * 100.0, 200.0 + angle.sin() * 100.0, angle)
        })
        .collect();

    let sensor_angle = 0.7853981633974483;
    let sensor_distance = 9.0;
    let rotation_angle = 0.39269908169872414;
    let step_size = 1.0;
    let deposit_amount = 5.0;
    let width = 400;
    let height = 400;

    use rand::Rng;

    c.bench_function("agent_sense_rotate_move_deposit_50k", |b| {
        b.iter(|| {
            let mut rng = rand::thread_rng();
            let mut trail = trail_map.clone();
            let mut agents = agents.clone();
            for agent in &mut agents {
                let (left, center, right) = agent.sense(
                    &trail,
                    width,
                    height,
                    sensor_angle,
                    sensor_distance,
                );

                agent.rotate(left, center, right, rotation_angle, &mut rng);
                agent.move_forward(step_size, width, height);
                agent.deposit(&mut trail, width, height, deposit_amount);
            }
            black_box(agents);
        });
    });
}

fn bench_agent_update_full_frame(c: &mut Criterion) {
    #[derive(Clone, Copy)]
    struct Agent {
        x: f32,
        y: f32,
        heading: f32,
    }

    impl Agent {
        fn new(x: f32, y: f32, heading: f32) -> Self {
            Self { x, y, heading }
        }

        fn sense(&self, trail: &[f32], width: usize, height: usize, sensor_angle: f32, sensor_distance: f32) -> (f32, f32, f32) {
            let sensor_offset = sensor_distance;
            let left_angle = self.heading - sensor_angle;
            let right_angle = self.heading + sensor_angle;

            let left_x = (self.x + left_angle.cos() * sensor_offset) as usize;
            let left_y = (self.y + left_angle.sin() * sensor_offset) as usize;
            let center_x = (self.x + self.heading.cos() * sensor_offset) as usize;
            let center_y = (self.y + self.heading.sin() * sensor_offset) as usize;
            let right_x = (self.x + right_angle.cos() * sensor_offset) as usize;
            let right_y = (self.y + right_angle.sin() * sensor_offset) as usize;

            let left_idx = left_y.min(height - 1) * width + left_x.min(width - 1);
            let center_idx = center_y.min(height - 1) * width + center_x.min(width - 1);
            let right_idx = right_y.min(height - 1) * width + right_x.min(width - 1);

            (trail[left_idx], trail[center_idx], trail[right_idx])
        }

        fn rotate(&mut self, left: f32, center: f32, right: f32, rotation_angle: f32, rng: &mut impl rand::Rng) {
            if center > left && center > right {
            } else if center < left && center < right {
                self.heading += if rng.gen_bool(0.5) { rotation_angle } else { -rotation_angle };
            } else if left > right {
                self.heading -= rotation_angle;
            } else if right > left {
                self.heading += rotation_angle;
            }
        }

        fn move_forward(&mut self, step_size: f32, width: usize, height: usize) {
            self.x = (self.x + step_size * self.heading.cos()).abs();
            self.y = (self.y + step_size * self.heading.sin()).abs();
            if self.x >= width as f32 { self.x -= width as f32; }
            if self.y >= height as f32 { self.y -= height as f32; }
        }

        fn deposit(&self, trail: &mut [f32], width: usize, height: usize, amount: f32) {
            let idx = (self.y as usize).min(height - 1) * width + (self.x as usize).min(width - 1);
            trail[idx] = (trail[idx] + amount).min(1000.0);
        }
    }

    let mut agents: Vec<Agent> = (0..50000)
        .map(|i| {
            let angle = (i as f32 / 50000.0) * std::f32::consts::PI * 2.0;
            Agent::new(200.0 + angle.cos() * 100.0, 200.0 + angle.sin() * 100.0, angle)
        })
        .collect();

    let sensor_angle = 0.7853981633974483;
    let sensor_distance = 9.0;
    let rotation_angle = 0.39269908169872414;
    let step_size = 1.0;
    let deposit_amount = 5.0;
    let width = 400;
    let height = 400;

    c.bench_function("full_frame_50k_agents", |b| {
        b.iter(|| {
            let mut rng = rand::thread_rng();
            let mut trail = vec![0.0f32; 400 * 400];

            for agent in &mut agents {
                let (left, center, right) = agent.sense(&trail, width, height, sensor_angle, sensor_distance);
                agent.rotate(left, center, right, rotation_angle, &mut rng);
                agent.move_forward(step_size, width, height);
                agent.deposit(&mut trail, width, height, deposit_amount);
            }

            black_box(&trail);
        });
    });
}

fn bench_agent_update_varying_population(c: &mut Criterion) {
    #[derive(Clone, Copy)]
    struct Agent {
        x: f32,
        y: f32,
        heading: f32,
    }

    impl Agent {
        fn new(x: f32, y: f32, heading: f32) -> Self {
            Self { x, y, heading }
        }
    }

    let mut group = c.benchmark_group("agent_update_population");

    for population in [10000, 25000, 50000, 100000] {
        let agents: Vec<Agent> = (0..population)
            .map(|i| {
                let angle = (i as f32 / population as f32) * std::f32::consts::PI * 2.0;
                Agent::new(200.0 + angle.cos() * 100.0, 200.0 + angle.sin() * 100.0, angle)
            })
            .collect();

        group.bench_function(format!("{}_agents", population), |b| {
            b.iter(|| {
                let mut rng = rand::thread_rng();
                let mut trail = vec![0.0f32; 400 * 400];
                let mut agents = agents.clone();

                for agent in &mut agents {
                use rand::Rng;
                let left_x = (agent.x + (agent.heading - 0.785).cos() * 9.0) as usize;
                    let left_y = (agent.y + (agent.heading - 0.785).sin() * 9.0) as usize;
                    let center_x = (agent.x + agent.heading.cos() * 9.0) as usize;
                    let center_y = (agent.y + agent.heading.sin() * 9.0) as usize;
                    let right_x = (agent.x + (agent.heading + 0.785).cos() * 9.0) as usize;
                    let right_y = (agent.y + (agent.heading + 0.785).sin() * 9.0) as usize;

                    let idx_l = left_y.min(399) * 400 + left_x.min(399);
                    let idx_c = center_y.min(399) * 400 + center_x.min(399);
                    let idx_r = right_y.min(399) * 400 + right_x.min(399);

                    let (left, center, right) = (trail[idx_l], trail[idx_c], trail[idx_r]);

                    if center > left && center > right {
                    } else if center < left && center < right {
                        agent.heading += if rng.gen_bool(0.5) { 0.3927 } else { -0.3927 };
                    } else if left > right {
                        agent.heading -= 0.3927;
                    } else if right > left {
                        agent.heading += 0.3927;
                    }

                    agent.x = (agent.x + agent.heading.cos()).abs();
                    agent.y = (agent.y + agent.heading.sin()).abs();
                    if agent.x >= 400.0 { agent.x -= 400.0; }
                    if agent.y >= 400.0 { agent.y -= 400.0; }

                    let idx = (agent.y as usize) * 400 + (agent.x as usize);
                    trail[idx] = (trail[idx] + 5.0).min(1000.0);
                }
                black_box(agents);
            });
        });
    }

    group.finish();
}

fn bench_downsampling(c: &mut Criterion) {
    let trail_map = {
        let mut d = vec![0.0f32; 400 * 400];
        for i in 0..400 {
            for j in 0..400 {
                d[i * 400 + j] = ((i * j) % 100) as f32 / 10.0;
            }
        }
        d
    };

    fn downsample(trail: &[f32], sim_w: usize, sim_h: usize, term_w: usize, term_h: usize) -> Vec<(f32, f32)> {
        let cell_w = sim_w as f32 / term_w as f32;
        let cell_h = sim_h as f32 / term_h as f32;
        let mut cells = Vec::with_capacity(term_w * term_h);

        for y in 0..term_h {
            for x in 0..term_w {
                let start_x = (x as f32 * cell_w) as usize;
                let start_y = (y as f32 * cell_h) as usize;
                let end_x = ((x + 1) as f32 * cell_w) as usize;
                let end_y = ((y + 1) as f32 * cell_h) as usize;

                let mut sum = 0.0f32;
                let mut count = 0;

                for py in start_y..end_y.min(sim_h) {
                    for px in start_x..end_x.min(sim_w) {
                        let val = trail[py * sim_w + px];
                        sum += val;
                        count += 1;
                    }
                }

                let avg = if count > 0 { sum / count as f32 } else { 0.0 };
                cells.push((avg, avg));
            }
        }

        cells
    }

    let mut group = c.benchmark_group("downsampling");

    for (term_w, term_h) in [(80, 24), (120, 40), (200, 60)] {
        group.bench_function(format!("{}x{}", term_w, term_h), |b| {
            b.iter(|| {
                let result = downsample(black_box(&trail_map), 400, 400, term_w, term_h);
                black_box(result);
            });
        });
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_agent_sense_rotate_move_deposit,
    bench_agent_update_full_frame,
    bench_agent_update_varying_population,
    bench_downsampling
);
criterion_main!(benches);
