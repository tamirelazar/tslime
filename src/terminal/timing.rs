use std::time::{Duration, Instant};

const FPS_SAMPLE_COUNT: usize = 30;

/// FrameTimer provides timing control for the simulation loop.
///
/// # Timing Model
///
/// FrameTimer tracks actual elapsed wall-clock time between frames and
/// returns a delta_time value scaled by time_scale. The caller (main.rs)
/// is responsible for applying reference timestep normalization.
///
/// delta_time() returns: `actual_elapsed_wall_time * time_scale`
///
/// The caller (main.rs) divides by REFERENCE_TIME_STEP (1/30) to normalize
/// to the reference 30-fps simulation rate. This ensures:
/// - time_scale 2.0 doubles simulation speed
/// - --fps 30 and --fps 60 produce identical simulation progression
///   at the same wall-clock elapsed time
///
/// The target_fps setting only controls the maximum frame rate (via sleep timing).
/// It does not affect the delta_time value.
///
/// # Usage
///
/// ```ignore
/// const REFERENCE_TIME_STEP: f32 = 1.0 / 30.0;
///
/// let mut timer = FrameTimer::with_time_scale(fps, frame_delay, time_scale);
///
/// loop {
///     let dt = timer.delta_time();  // Returns elapsed * time_scale
///     sim.update(dt / REFERENCE_TIME_STEP);  // Apply normalization
///     // ... render ...
///     timer.tick();  // Sleeps to maintain target FPS, increments frame count
/// }
/// ```

#[derive(Debug, Clone)]
pub struct FrameTimer {
    target_fps: usize,
    frame_delay: Duration,
    last_frame_time: Instant,
    frame_count: u64,
    time_scale: f32,
    fps_samples: [f32; FPS_SAMPLE_COUNT],
    fps_sample_index: usize,
    fps_count: usize,
    sim_start: Instant,
    sim_duration: Duration,
    render_start: Instant,
    render_duration: Duration,
}

impl FrameTimer {
    #[allow(dead_code)]
    pub fn new(fps: usize, frame_delay_seconds: f32) -> Self {
        Self::with_time_scale(fps, frame_delay_seconds, 1.0)
    }

    pub fn with_time_scale(fps: usize, frame_delay_seconds: f32, time_scale: f32) -> Self {
        let _target_frame_time = Duration::from_secs_f64(1.0 / fps as f64);
        let frame_delay = Duration::from_secs_f32(frame_delay_seconds);

        Self {
            target_fps: fps,
            frame_delay,
            last_frame_time: Instant::now(),
            frame_count: 0,
            time_scale,
            fps_samples: [0.0; FPS_SAMPLE_COUNT],
            fps_sample_index: 0,
            fps_count: 0,
            sim_start: Instant::now(),
            sim_duration: Duration::ZERO,
            render_start: Instant::now(),
            render_duration: Duration::ZERO,
        }
    }

    #[allow(dead_code)]
    pub fn target_fps(&self) -> usize {
        self.target_fps
    }

    #[allow(dead_code)]
    pub fn frame_delay(&self) -> Duration {
        self.frame_delay
    }

    #[allow(dead_code)]
    pub fn set_target_fps(&mut self, fps: usize) {
        self.target_fps = fps;
    }

    #[allow(dead_code)]
    pub fn set_frame_delay(&mut self, frame_delay_seconds: f32) {
        self.frame_delay = Duration::from_secs_f32(frame_delay_seconds);
    }

    pub fn frame_count(&self) -> u64 {
        self.frame_count
    }

    pub fn elapsed(&self) -> Duration {
        self.last_frame_time.elapsed()
    }

    pub fn current_fps(&self) -> f64 {
        let elapsed = self.elapsed().as_secs_f64();
        if elapsed > 0.0 {
            1.0 / elapsed
        } else {
            0.0
        }
    }

    pub fn average_fps(&self) -> f64 {
        if self.fps_count == 0 {
            0.0
        } else {
            let sum: f32 = if self.fps_count < FPS_SAMPLE_COUNT {
                self.fps_samples[..self.fps_count].iter().sum()
            } else {
                self.fps_samples.iter().sum()
            };
            sum as f64 / self.fps_count.min(FPS_SAMPLE_COUNT) as f64
        }
    }

    pub fn sim_duration(&self) -> Duration {
        self.sim_duration
    }

    pub fn render_duration(&self) -> Duration {
        self.render_duration
    }

    pub fn start_sim(&mut self) {
        self.sim_start = Instant::now();
    }

    pub fn end_sim_start_render(&mut self) {
        self.sim_duration = self.sim_start.elapsed();
        self.render_start = Instant::now();
    }

    pub fn end_render(&mut self) {
        self.render_duration = self.render_start.elapsed();
    }

    pub fn delta_time(&mut self) -> f32 {
        let elapsed = self.last_frame_time.elapsed();
        self.last_frame_time = Instant::now();

        let fps_sample = if elapsed.as_secs_f64() > 0.0 {
            1.0 / elapsed.as_secs_f64() as f32
        } else {
            0.0
        };

        self.fps_samples[self.fps_sample_index] = fps_sample;
        self.fps_sample_index = (self.fps_sample_index + 1) % FPS_SAMPLE_COUNT;
        if self.fps_count < FPS_SAMPLE_COUNT {
            self.fps_count += 1;
        }

        elapsed.as_secs_f32() * self.time_scale
    }

    #[allow(dead_code)]
    pub fn set_time_scale(&mut self, time_scale: f32) {
        self.time_scale = time_scale;
    }

    pub fn tick(&mut self) {
        let elapsed = self.last_frame_time.elapsed();
        let target_frame_time = Duration::from_secs_f64(1.0 / self.target_fps as f64);

        if elapsed < target_frame_time {
            let sleep_time = target_frame_time - elapsed;
            std::thread::sleep(sleep_time.min(self.frame_delay));
        }

        self.frame_count += 1;
    }
}

impl Default for FrameTimer {
    fn default() -> Self {
        Self {
            target_fps: 30,
            frame_delay: Duration::from_secs_f32(0.033),
            last_frame_time: Instant::now(),
            frame_count: 0,
            time_scale: 1.0,
            fps_samples: [0.0; FPS_SAMPLE_COUNT],
            fps_sample_index: 0,
            fps_count: 0,
            sim_start: Instant::now(),
            sim_duration: Duration::ZERO,
            render_start: Instant::now(),
            render_duration: Duration::ZERO,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_frame_timer_creation() {
        let timer = FrameTimer::new(30, 0.033);
        assert_eq!(timer.target_fps(), 30);
        assert_eq!(timer.frame_count(), 0);
    }

    #[test]
    fn test_frame_timer_default() {
        let timer = FrameTimer::default();
        assert_eq!(timer.target_fps(), 30);
    }

    #[test]
    fn test_set_target_fps() {
        let mut timer = FrameTimer::new(30, 0.033);
        timer.set_target_fps(60);
        assert_eq!(timer.target_fps(), 60);
    }

    #[test]
    fn test_set_frame_delay() {
        let mut timer = FrameTimer::new(30, 0.033);
        timer.set_frame_delay(0.016);
        assert_eq!(timer.frame_delay(), Duration::from_secs_f32(0.016));
    }

    #[test]
    fn test_elapsed() {
        let timer = FrameTimer::new(30, 0.033);
        std::thread::sleep(Duration::from_millis(10));
        assert!(timer.elapsed() >= Duration::from_millis(10));
    }

    #[test]
    fn test_tick_increments_frame_count() {
        let mut timer = FrameTimer::new(60, 0.016);
        assert_eq!(timer.frame_count(), 0);
        timer.tick();
        assert_eq!(timer.frame_count(), 1);
        timer.tick();
        assert_eq!(timer.frame_count(), 2);
    }

    #[test]
    fn test_delta_time_returns_scaled_value() {
        let mut timer = FrameTimer::new(60, 0.016);
        std::thread::sleep(Duration::from_millis(100));
        let dt = timer.delta_time();
        assert!(
            (0.08..0.15).contains(&dt),
            "dt should be ~0.1s (100ms * 1.0), got {}",
            dt
        );
    }

    #[test]
    fn test_delta_time_with_time_scale() {
        let mut timer = FrameTimer::with_time_scale(60, 0.016, 2.0);
        std::thread::sleep(Duration::from_millis(100));
        let dt = timer.delta_time();
        assert!(
            (0.15..0.25).contains(&dt),
            "dt should be ~0.2s (100ms * 2.0), got {}",
            dt
        );
    }

    #[test]
    fn test_set_time_scale() {
        let mut timer = FrameTimer::new(30, 0.033);
        std::thread::sleep(Duration::from_millis(100));
        timer.set_time_scale(0.5);
        let dt = timer.delta_time();
        assert!(
            (0.04..0.07).contains(&dt),
            "dt should be ~0.05s (100ms * 0.5), got {}",
            dt
        );
    }

    #[test]
    fn test_time_scale_doubles_simulation_speed() {
        let mut timer_fast = FrameTimer::with_time_scale(60, 0.016, 2.0);

        std::thread::sleep(Duration::from_millis(100));
        let dt_normal = timer_fast.delta_time();

        timer_fast.set_time_scale(1.0);

        std::thread::sleep(Duration::from_millis(100));
        let dt_slower = timer_fast.delta_time();

        let ratio = dt_normal / dt_slower;
        assert!(
            ratio > 1.8 && ratio < 2.2,
            "ratio should be ~2.0, got {}",
            ratio
        );
    }

    #[test]
    fn test_delta_time_fps_invariant() {
        let mut timer_30fps = FrameTimer::new(30, 0.033);

        std::thread::sleep(Duration::from_millis(100));
        let dt_30 = timer_30fps.delta_time();

        std::thread::sleep(Duration::from_millis(100));
        let dt_30_again = timer_30fps.delta_time();

        assert!(
            (0.08..0.15).contains(&dt_30),
            "First dt should be ~0.1s, got {}",
            dt_30
        );
        assert!(
            (0.08..0.15).contains(&dt_30_again),
            "Second dt should be ~0.1s, got {}",
            dt_30_again
        );
        assert!(
            (dt_30 - dt_30_again).abs() < 0.05,
            "Both dt calls should return similar values, got {} vs {}",
            dt_30,
            dt_30_again
        );
    }

    #[test]
    fn test_delta_time_fps_setting_does_not_affect_value() {
        let mut timer_30 = FrameTimer::with_time_scale(30, 0.033, 1.0);

        std::thread::sleep(Duration::from_millis(50));
        let dt_30 = timer_30.delta_time();

        let mut timer_60 = FrameTimer::with_time_scale(60, 0.016, 1.0);

        std::thread::sleep(Duration::from_millis(50));
        let dt_60 = timer_60.delta_time();

        let mut timer_144 = FrameTimer::with_time_scale(144, 0.007, 1.0);

        std::thread::sleep(Duration::from_millis(50));
        let dt_144 = timer_144.delta_time();

        assert!(
            (0.04..0.07).contains(&dt_30),
            "30fps dt should be ~0.05s, got {}",
            dt_30
        );
        assert!(
            (0.04..0.07).contains(&dt_60),
            "60fps dt should be ~0.05s, got {}",
            dt_60
        );
        assert!(
            (0.04..0.07).contains(&dt_144),
            "144fps dt should be ~0.05s, got {}",
            dt_144
        );
        assert!(
            (dt_30 - dt_60).abs() < 0.02,
            "30fps and 60fps dt should be similar, got {} vs {}",
            dt_30,
            dt_60
        );
        assert!(
            (dt_60 - dt_144).abs() < 0.02,
            "60fps and 144fps dt should be similar, got {} vs {}",
            dt_60,
            dt_144
        );
    }

    #[test]
    fn test_current_fps() {
        let timer = FrameTimer::new(60, 0.016);
        std::thread::sleep(Duration::from_millis(10));
        let fps = timer.current_fps();
        assert!(fps > 0.0 && fps < 200.0);
    }

    #[test]
    fn test_average_fps_initial() {
        let timer = FrameTimer::new(30, 0.033);
        assert_eq!(timer.average_fps(), 0.0);
    }

    #[test]
    fn test_average_fps_after_samples() {
        let mut timer = FrameTimer::new(30, 0.033);

        for _ in 0..5 {
            std::thread::sleep(Duration::from_millis(50));
            timer.delta_time();
        }

        let avg = timer.average_fps();
        assert!(
            avg > 10.0 && avg < 30.0,
            "Average FPS should be around 20, got {}",
            avg
        );
    }

    #[test]
    fn test_average_fps_converges() {
        let mut timer = FrameTimer::new(30, 0.033);

        for _ in 0..35 {
            std::thread::sleep(Duration::from_millis(50));
            timer.delta_time();
        }

        let avg = timer.average_fps();
        assert!(
            avg > 15.0 && avg < 25.0,
            "Average should converge to ~20 FPS, got {}",
            avg
        );
    }

    #[test]
    fn test_sim_and_render_timing() {
        let mut timer = FrameTimer::new(30, 0.033);

        timer.start_sim();
        std::thread::sleep(Duration::from_millis(5));
        timer.end_sim_start_render();

        assert!(timer.sim_duration() >= Duration::from_millis(4));

        timer.end_render();
        assert!(timer.render_duration() > Duration::ZERO);
    }

    #[test]
    fn test_fps_samples_wrap_around() {
        let mut timer = FrameTimer::new(30, 0.033);

        for i in 0..35 {
            std::thread::sleep(Duration::from_millis(10 + i % 5));
            timer.delta_time();
        }

        let avg = timer.average_fps();
        assert!(
            avg > 30.0 && avg < 120.0,
            "Average should be valid after wrap, got {}",
            avg
        );
    }
}
