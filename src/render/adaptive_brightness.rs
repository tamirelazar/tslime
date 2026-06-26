#[derive(Debug, Clone)]
/// Manages adaptive brightness normalization to prevent screen flickering.
///
/// Tracks a history of peak brightness values and smooths transitions between them
/// to avoid sudden jumps in exposure when the simulation intensity changes rapidly.
pub struct AdaptiveBrightness {
    window_size: usize,
    peak_history: Vec<f32>,
    current_max: f32,
    smoothing_factor: f32,
    enabled: bool,
}

impl AdaptiveBrightness {
    /// Creates a new tracker. `window_size` is the number of frames of peak
    /// history to keep (clamped to 1-200).
    pub fn new(window_size: usize, enabled: bool) -> Self {
        use crate::config_defaults::normalization;
        Self {
            window_size: window_size.clamp(1, normalization::LARGE_WINDOW),
            peak_history: Vec::with_capacity(normalization::DEFAULT_WINDOW_SIZE),
            current_max: 1.0,
            smoothing_factor: 0.1,
            enabled,
        }
    }

    /// Sets the smoothing factor for brightness transitions,
    /// clamped to 0.01 (slow) - 0.5 (fast).
    pub fn with_smoothing_factor(mut self, factor: f32) -> Self {
        self.smoothing_factor = factor.clamp(0.01, 0.5);
        self
    }

    /// Records the current frame's peak brightness and updates the smoothed maximum.
    pub fn update(&mut self, cells: &[crate::render::downsample::Cell]) {
        if !self.enabled {
            return;
        }

        let current_peak: f32 = cells
            .iter()
            .map(|c| c.top.max(c.bottom))
            .fold(0.0, |acc, v| acc.max(v));

        self.peak_history.push(current_peak);

        if self.peak_history.len() > self.window_size {
            self.peak_history.remove(0);
        }

        if self.peak_history.len() >= 3 {
            let avg_peak: f32 =
                self.peak_history.iter().sum::<f32>() / self.peak_history.len() as f32;
            self.current_max =
                self.current_max + (avg_peak - self.current_max) * self.smoothing_factor;
            self.current_max = self.current_max.max(0.1);
        } else if current_peak > self.current_max {
            self.current_max =
                self.current_max + (current_peak - self.current_max) * self.smoothing_factor;
        }
    }

    /// Returns the current smoothed maximum brightness (never below 1.0).
    /// Returns 1.0 when disabled.
    pub fn get_max_brightness(&self) -> f32 {
        if self.enabled {
            self.current_max.max(1.0)
        } else {
            1.0
        }
    }

    /// Resets the history and current max brightness.
    pub fn reset(&mut self) {
        self.peak_history.clear();
        self.current_max = 1.0;
    }

    /// Enables or disables adaptive brightness.
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        if !enabled {
            self.reset();
        }
    }

    /// Checks if adaptive brightness is enabled.
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::render::downsample::Cell;

    #[test]
    fn test_adaptive_brightness_disabled() {
        let mut ab = AdaptiveBrightness::new(10, false);
        let cells = vec![
            Cell {
                top: 10.0,
                bottom: 5.0,
                ..Default::default()
            },
            Cell {
                top: 8.0,
                bottom: 6.0,
                ..Default::default()
            },
        ];
        ab.update(&cells);
        assert_eq!(ab.get_max_brightness(), 1.0);
    }

    #[test]
    fn test_adaptive_brightness_tracks_increasing_peak() {
        let mut ab = AdaptiveBrightness::new(10, true).with_smoothing_factor(0.5);

        let cells1 = vec![Cell {
            top: 5.0,
            bottom: 3.0,
            ..Default::default()
        }];
        ab.update(&cells1);

        let cells2 = vec![Cell {
            top: 10.0,
            bottom: 8.0,
            ..Default::default()
        }];
        ab.update(&cells2);

        let max = ab.get_max_brightness();
        assert!(max > 1.0, "Expected max > 1.0, got {}", max);
        assert!(max <= 10.0, "Expected max <= 10.0, got {}", max);
    }

    #[test]
    fn test_adaptive_brightness_window_size() {
        let mut ab = AdaptiveBrightness::new(3, true).with_smoothing_factor(0.5);

        for i in 1..=5 {
            let cells = vec![Cell {
                top: i as f32 * 10.0,
                bottom: 0.0,
                ..Default::default()
            }];
            ab.update(&cells);
        }

        assert_eq!(ab.peak_history.len(), 3);
        let max = ab.get_max_brightness();
        assert!(
            (30.0..=50.0).contains(&max),
            "Expected max in [30, 50], got {}",
            max
        );
    }

    #[test]
    fn test_adaptive_brightness_reset() {
        let mut ab = AdaptiveBrightness::new(10, true);
        let cells = vec![Cell {
            top: 25.0,
            bottom: 20.0,
            ..Default::default()
        }];
        ab.update(&cells);

        ab.reset();
        assert_eq!(ab.get_max_brightness(), 1.0);
        assert!(ab.peak_history.is_empty());
    }

    #[test]
    fn test_adaptive_brightness_set_enabled() {
        let mut ab = AdaptiveBrightness::new(10, false);
        let cells = vec![Cell {
            top: 15.0,
            bottom: 10.0,
            ..Default::default()
        }];
        ab.update(&cells);

        ab.set_enabled(true);
        assert!(ab.is_enabled());

        ab.set_enabled(false);
        assert!(!ab.is_enabled());
    }

    #[test]
    fn test_adaptive_brightness_minimum() {
        let mut ab = AdaptiveBrightness::new(10, true);
        let cells = vec![Cell {
            top: 0.1,
            bottom: 0.0,
            ..Default::default()
        }];
        ab.update(&cells);
        assert_eq!(ab.get_max_brightness(), 1.0);
    }

    #[test]
    fn test_adaptive_brightness_smoothing() {
        let mut ab = AdaptiveBrightness::new(10, true).with_smoothing_factor(0.1);

        for _ in 0..10 {
            let cells = vec![Cell {
                top: 20.0,
                bottom: 15.0,
                ..Default::default()
            }];
            ab.update(&cells);
        }

        let max = ab.get_max_brightness();
        assert!(
            max > 10.0 && max < 20.0,
            "Expected smoothed value, got {}",
            max
        );
    }
}
