//! Timer and FPS management for the simulation loop.
//!
//! This module provides a high-precision timer that manages the update/render loop,
//! tracks performance metrics (FPS, frame times), and implements adaptive FPS scaling
//! when performance drops.

use std::time::{Duration, Instant};

const FPS_SAMPLE_COUNT: usize = 30;
const ADAPTIVE_CHECK_INTERVAL: usize = 60;
const FPS_DROP_THRESHOLD: f64 = 0.85;
const MIN_ADAPTIVE_FPS: usize = 15;
const FPS_STEPS: [usize; 6] = [60, 45, 30, 25, 20, 15];

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
    adaptive_fps_enabled: bool,
    adaptive_check_counter: usize,
    last_adjusted_fps: usize,
    /// Flag indicating if the FPS was automatically adjusted in the last frame.
    pub fps_adjusted_notification: bool,
}

impl FrameTimer {
    #[cfg(test)]
    /// Create a new frame timer with the specified FPS and frame delay.
    pub fn new(fps: usize, frame_delay_seconds: f32) -> Self {
        Self::with_time_scale(fps, frame_delay_seconds, 1.0)
    }

    /// Create a new frame timer with specified FPS, delay, and time scale.
    pub fn with_time_scale(fps: usize, frame_delay_seconds: f32, time_scale: f32) -> Self {
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
            adaptive_fps_enabled: true,
            adaptive_check_counter: 0,
            last_adjusted_fps: fps,
            fps_adjusted_notification: false,
        }
    }

    /// Enable or disable adaptive FPS scaling.
    pub fn set_adaptive_fps(&mut self, enabled: bool) {
        self.adaptive_fps_enabled = enabled;
    }

    /// Check if the frame rate should be lowered based on recent performance.
    pub fn should_adjust_fps(&mut self) -> bool {
        if !self.adaptive_fps_enabled {
            return false;
        }

        self.adaptive_check_counter += 1;
        if self.adaptive_check_counter < ADAPTIVE_CHECK_INTERVAL {
            return false;
        }
        self.adaptive_check_counter = 0;

        let avg_fps = self.average_fps();
        let target_fps_f64 = self.target_fps as f64;
        let threshold = target_fps_f64 * FPS_DROP_THRESHOLD;

        avg_fps < threshold && self.target_fps > MIN_ADAPTIVE_FPS
    }

    /// Calculate the next lower safe FPS step.
    pub fn get_adjusted_fps(&self) -> Option<usize> {
        if !self.adaptive_fps_enabled || self.target_fps <= MIN_ADAPTIVE_FPS {
            return None;
        }

        let avg_fps = self.average_fps();

        FPS_STEPS
            .iter()
            .find(|&&fps| fps < self.target_fps && avg_fps < (fps as f64 * 1.1))
            .copied()
    }

    /// Apply a new target FPS limit.
    pub fn apply_fps_adjustment(&mut self, new_fps: usize) {
        if new_fps < self.target_fps && new_fps >= MIN_ADAPTIVE_FPS {
            self.target_fps = new_fps;
            self.last_adjusted_fps = new_fps;
            self.fps_adjusted_notification = true;
        }
    }

    #[cfg(test)]
    /// Get the target FPS setting.
    pub fn target_fps(&self) -> usize {
        self.target_fps
    }

    #[cfg(test)]
    /// Get the frame delay duration.
    pub fn frame_delay(&self) -> Duration {
        self.frame_delay
    }

    #[cfg(test)]
    /// Set the target FPS.
    pub fn set_target_fps(&mut self, fps: usize) {
        self.target_fps = fps;
    }

    #[cfg(test)]
    /// Set the frame delay in seconds.
    pub fn set_frame_delay(&mut self, frame_delay_seconds: f32) {
        self.frame_delay = Duration::from_secs_f32(frame_delay_seconds);
    }

    /// Get the total number of frames processed.
    pub fn frame_count(&self) -> u64 {
        self.frame_count
    }

    /// Get the time elapsed since the last frame.
    pub fn elapsed(&self) -> Duration {
        self.last_frame_time.elapsed()
    }

    /// Get the time elapsed since the last frame in milliseconds.
    pub fn last_frame_ms(&self) -> f32 {
        self.last_frame_time.elapsed().as_secs_f32() * 1000.0
    }

    /// Calculate the instantaneous FPS based on the last frame time.
    pub fn current_fps(&self) -> f64 {
        let elapsed = self.elapsed().as_secs_f64();
        if elapsed > 0.0 {
            1.0 / elapsed
        } else {
            0.0
        }
    }

    /// Calculate the average FPS over the last `FPS_SAMPLE_COUNT` frames.
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

    /// Get the duration of the last simulation step.
    pub fn sim_duration(&self) -> Duration {
        self.sim_duration
    }

    /// Get the duration of the last render step.
    pub fn render_duration(&self) -> Duration {
        self.render_duration
    }

    /// Mark the start of the simulation update.
    pub fn start_sim(&mut self) {
        self.sim_start = Instant::now();
    }

    /// Mark the end of simulation and start of rendering.
    pub fn end_sim_start_render(&mut self) {
        self.sim_duration = self.sim_start.elapsed();
        self.render_start = Instant::now();
    }

    /// Mark the end of rendering.
    pub fn end_render(&mut self) {
        self.render_duration = self.render_start.elapsed();
    }

    /// Calculate delta time for the current frame.
    ///
    /// This updates the internal timer state and returns the elapsed time
    /// multiplied by the time scale.
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

    /// Fixed simulation timestep for the current frame, scaled by time_scale.
    ///
    /// Unlike [`delta_time`](Self::delta_time), this is derived from the target
    /// frame interval (`1 / target_fps`) rather than measured wall-clock elapsed.
    /// Feeding the simulation a fixed step decouples it from frame-write jitter:
    /// when a blocked `write` inflates a frame's wall time, the sim still advances
    /// by a constant amount instead of lurching forward, which is what was causing
    /// flicker under terminal back-pressure (e.g. while holding a key).
    ///
    /// Trade-off: under sustained real slowdown (actual FPS below target) the sim
    /// runs slightly slow-motion rather than catching up — preferable to lurching
    /// for a screensaver, and consistent with the existing `dt.clamp(0.1)` policy.
    pub fn fixed_delta(&self) -> f32 {
        let target_fps = self.target_fps.max(1) as f32;
        (1.0 / target_fps) * self.time_scale
    }

    /// Set the simulation time scale.
    pub fn set_time_scale(&mut self, time_scale: f32) {
        self.time_scale = time_scale;
    }

    /// Sleep to maintain the target frame rate.
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
            adaptive_fps_enabled: true,
            adaptive_check_counter: 0,
            last_adjusted_fps: 30,
            fps_adjusted_notification: false,
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
    fn test_fixed_delta_is_target_interval() {
        // 30 FPS, time_scale 1.0 → fixed step of 1/30s, independent of wall time.
        let timer = FrameTimer::with_time_scale(30, 0.033, 1.0);
        assert!((timer.fixed_delta() - 1.0 / 30.0).abs() < 1e-6);
    }

    #[test]
    fn test_fixed_delta_scales_with_time_scale_and_fps() {
        // time_scale doubles the step; higher target_fps shrinks it.
        let timer = FrameTimer::with_time_scale(60, 0.016, 2.0);
        assert!((timer.fixed_delta() - 2.0 / 60.0).abs() < 1e-6);
    }

    #[test]
    fn test_fixed_delta_is_deterministic_regardless_of_wall_time() {
        // Unlike delta_time(), fixed_delta() must not change with elapsed wall time.
        let timer = FrameTimer::with_time_scale(30, 0.033, 1.0);
        let a = timer.fixed_delta();
        std::thread::sleep(Duration::from_millis(20));
        let b = timer.fixed_delta();
        assert_eq!(a, b);
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
    #[ignore]
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
    #[ignore]
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
    #[ignore]
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
    #[ignore]
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
    #[ignore]
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
    #[ignore]
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
    #[ignore]
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
    #[ignore]
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
        for _ in 0..FPS_SAMPLE_COUNT + 5 {
            timer.fps_samples[timer.fps_sample_index] = 60.0;
            timer.fps_sample_index = (timer.fps_sample_index + 1) % FPS_SAMPLE_COUNT;
            timer.fps_count += 1;
        }
        assert_eq!(timer.average_fps(), 60.0);
    }

    #[test]
    fn test_fps_adaptive_logic() {
        let mut timer = FrameTimer::new(60, 0.033);
        timer.set_adaptive_fps(true);
        // Force some low FPS samples
        for _ in 0..FPS_SAMPLE_COUNT {
            timer.fps_samples[timer.fps_sample_index] = 10.0;
            timer.fps_sample_index = (timer.fps_sample_index + 1) % FPS_SAMPLE_COUNT;
            timer.fps_count += 1;
        }

        // Advance counter to trigger check
        timer.adaptive_check_counter = ADAPTIVE_CHECK_INTERVAL - 1;
        assert!(timer.should_adjust_fps());

        let adjusted = timer.get_adjusted_fps();
        assert!(adjusted.is_some());
        assert!(adjusted.unwrap() < 60);

        timer.apply_fps_adjustment(30);
        assert_eq!(timer.target_fps, 30);
        assert!(timer.fps_adjusted_notification);
    }

    #[test]
    fn test_last_frame_ms() {
        let timer = FrameTimer::new(30, 0.033);
        let ms = timer.last_frame_ms();
        assert!(ms >= 0.0);
    }
}
