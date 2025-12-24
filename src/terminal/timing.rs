use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub struct FrameTimer {
    target_fps: usize,
    frame_delay: Duration,
    last_frame_time: Instant,
    frame_count: u64,
}

impl FrameTimer {
    pub fn new(fps: usize, frame_delay_seconds: f32) -> Self {
        let _target_frame_time = Duration::from_secs_f64(1.0 / fps as f64);
        let frame_delay = Duration::from_secs_f32(frame_delay_seconds);

        Self {
            target_fps: fps,
            frame_delay,
            last_frame_time: Instant::now(),
            frame_count: 0,
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

    pub fn tick(&mut self) {
        let elapsed = self.last_frame_time.elapsed();
        let target_frame_time = Duration::from_secs_f64(1.0 / self.target_fps as f64);

        if elapsed < target_frame_time {
            let sleep_time = target_frame_time - elapsed;
            std::thread::sleep(sleep_time.min(self.frame_delay));
        }

        self.last_frame_time = Instant::now();
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
    fn test_current_fps() {
        let timer = FrameTimer::new(60, 0.016);
        std::thread::sleep(Duration::from_millis(10));
        let fps = timer.current_fps();
        assert!(fps > 0.0 && fps < 200.0);
    }
}
