use std::time::{Duration, Instant};

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
}

impl FrameTimer {
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

    pub fn delta_time(&mut self) -> f32 {
        let elapsed = self.last_frame_time.elapsed();
        self.last_frame_time = Instant::now();
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
        Self::new(30, 0.033)
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
        assert!(dt >= 0.08 && dt < 0.15, "dt should be ~0.1s (100ms * 1.0), got {}", dt);
    }

    #[test]
    fn test_delta_time_with_time_scale() {
        let mut timer = FrameTimer::with_time_scale(60, 0.016, 2.0);
        std::thread::sleep(Duration::from_millis(100));
        let dt = timer.delta_time();
        assert!(dt >= 0.15 && dt < 0.25, "dt should be ~0.2s (100ms * 2.0), got {}", dt);
    }

    #[test]
    fn test_set_time_scale() {
        let mut timer = FrameTimer::new(30, 0.033);
        std::thread::sleep(Duration::from_millis(100));
        timer.set_time_scale(0.5);
        let dt = timer.delta_time();
        assert!(dt >= 0.04 && dt < 0.07, "dt should be ~0.05s (100ms * 0.5), got {}", dt);
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
        assert!(ratio > 1.8 && ratio < 2.2, "ratio should be ~2.0, got {}", ratio);
    }

    #[test]
    fn test_delta_time_fps_invariant() {
        let mut timer_30fps = FrameTimer::new(30, 0.033);

        std::thread::sleep(Duration::from_millis(100));
        let dt_30 = timer_30fps.delta_time();

        std::thread::sleep(Duration::from_millis(100));
        let dt_30_again = timer_30fps.delta_time();

        assert!(dt_30 >= 0.08 && dt_30 < 0.15, "First dt should be ~0.1s, got {}", dt_30);
        assert!(dt_30_again >= 0.08 && dt_30_again < 0.15, "Second dt should be ~0.1s, got {}", dt_30_again);
        assert!((dt_30 - dt_30_again).abs() < 0.05, "Both dt calls should return similar values, got {} vs {}", dt_30, dt_30_again);
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

        assert!(dt_30 >= 0.04 && dt_30 < 0.07, "30fps dt should be ~0.05s, got {}", dt_30);
        assert!(dt_60 >= 0.04 && dt_60 < 0.07, "60fps dt should be ~0.05s, got {}", dt_60);
        assert!(dt_144 >= 0.04 && dt_144 < 0.07, "144fps dt should be ~0.05s, got {}", dt_144);
        assert!((dt_30 - dt_60).abs() < 0.02, "30fps and 60fps dt should be similar, got {} vs {}", dt_30, dt_60);
        assert!((dt_60 - dt_144).abs() < 0.02, "60fps and 144fps dt should be similar, got {} vs {}", dt_60, dt_144);
    }

    #[test]
    fn test_current_fps() {
        let timer = FrameTimer::new(60, 0.016);
        std::thread::sleep(Duration::from_millis(10));
        let fps = timer.current_fps();
        assert!(fps > 0.0 && fps < 200.0);
    }
}
