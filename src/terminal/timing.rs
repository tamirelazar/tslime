use std::time::{Duration, Instant};

const SIMULATION_FPS: f32 = 30.0;

/// FrameTimer provides timing control for the simulation loop.
///
/// # Timing Model
///
/// The FrameTimer tracks actual elapsed wall-clock time between frames and
/// returns a delta_time value scaled for simulation consistency.
/// The delta_time calculation is:
///
/// `dt = actual_elapsed_time * time_scale * SIMULATION_FPS`
///
/// This ensures that:
/// - Simulation speed is consistent regardless of FPS setting
/// - `--time-scale 2.0` doubles simulation speed (agents move twice as far per second)
/// - `--fps 60` and `--fps 30` produce identical simulation progression at same wall-clock time
///
/// The target_fps setting only controls the maximum frame rate (via sleep timing).
/// It does not affect the simulation delta_time value.
///
/// # Usage
///
/// ```ignore
/// let mut timer = FrameTimer::with_time_scale(fps, frame_delay, time_scale);
///
/// loop {
///     let dt = timer.delta_time();  // Returns scaled delta time for simulation
///     sim.update(dt);
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
        elapsed.as_secs_f32() * self.time_scale * SIMULATION_FPS
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
        assert!(dt >= 2.0 && dt < 4.0, "dt should be ~3.0 (0.1s * 30), got {}", dt);
    }

    #[test]
    fn test_delta_time_with_time_scale() {
        let mut timer = FrameTimer::with_time_scale(60, 0.016, 2.0);
        std::thread::sleep(Duration::from_millis(100));
        let dt = timer.delta_time();
        assert!(dt >= 4.0 && dt < 8.0, "dt should be ~6.0 (0.1s * 30 * 2), got {}", dt);
    }

    #[test]
    fn test_set_time_scale() {
        let mut timer = FrameTimer::new(30, 0.033);
        std::thread::sleep(Duration::from_millis(100));
        timer.set_time_scale(0.5);
        let dt = timer.delta_time();
        assert!(dt >= 1.0 && dt < 2.5, "dt should be ~1.5 (0.1s * 30 * 0.5), got {}", dt);
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
        let mut timer_60fps = FrameTimer::new(60, 0.016);

        std::thread::sleep(Duration::from_millis(100));
        let dt_30 = timer_30fps.delta_time();

        std::thread::sleep(Duration::from_millis(100));
        let dt_60 = timer_60fps.delta_time();

        assert!(dt_30 >= 2.0 && dt_30 < 5.0, "30fps dt should be ~3.0, got {}", dt_30);
        assert!(dt_60 >= 2.0 && dt_60 < 8.0, "60fps dt should be ~3.0, got {}", dt_60);
    }

    #[test]
    fn test_current_fps() {
        let timer = FrameTimer::new(60, 0.016);
        std::thread::sleep(Duration::from_millis(10));
        let fps = timer.current_fps();
        assert!(fps > 0.0 && fps < 200.0);
    }
}
