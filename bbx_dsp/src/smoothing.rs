//! Parameter smoothing utilities for click-free parameter changes.

/// A value that smoothly transitions to a target over time.
///
/// Uses linear interpolation to avoid clicks and pops when parameters change.
/// Call `set_target()` when the parameter changes, then `next()` for each sample.
#[derive(Debug, Clone)]
pub struct SmoothedValue {
    current: f64,
    target: f64,
    step: f64,
    samples_remaining: usize,
}

impl SmoothedValue {
    /// Default smoothing time in samples (~5ms at 44.1kHz).
    pub const DEFAULT_SMOOTHING_SAMPLES: usize = 220;

    /// Create a new `SmoothedValue` starting at the given value.
    pub fn new(initial_value: f64) -> Self {
        Self {
            current: initial_value,
            target: initial_value,
            step: 0.0,
            samples_remaining: 0,
        }
    }

    /// Set a new target value to smooth towards over the given number of samples.
    #[inline]
    pub fn set_target(&mut self, target: f64, samples: usize) {
        if samples == 0 {
            self.current = target;
            self.target = target;
            self.step = 0.0;
            self.samples_remaining = 0;
        } else {
            self.target = target;
            self.step = (target - self.current) / samples as f64;
            self.samples_remaining = samples;
        }
    }

    /// Set target with default smoothing time.
    #[inline]
    pub fn set_target_default(&mut self, target: f64) {
        self.set_target(target, Self::DEFAULT_SMOOTHING_SAMPLES);
    }

    /// Get the next smoothed value (call once per sample).
    #[inline]
    pub fn next_value(&mut self) -> f64 {
        if self.samples_remaining > 0 {
            self.current += self.step;
            self.samples_remaining -= 1;

            // Snap to target when done to avoid floating-point drift
            if self.samples_remaining == 0 {
                self.current = self.target;
            }
        }
        self.current
    }

    /// Get the current value without advancing.
    #[inline]
    pub fn current(&self) -> f64 {
        self.current
    }

    /// Get the target value.
    #[inline]
    pub fn target(&self) -> f64 {
        self.target
    }

    /// Check if the value is still smoothing.
    #[inline]
    pub fn is_smoothing(&self) -> bool {
        self.samples_remaining > 0
    }

    /// Immediately set both current and target to a value (no smoothing).
    #[inline]
    pub fn set_immediate(&mut self, value: f64) {
        self.current = value;
        self.target = value;
        self.step = 0.0;
        self.samples_remaining = 0;
    }
}

impl Default for SmoothedValue {
    fn default() -> Self {
        Self::new(0.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_immediate_value() {
        let mut sv = SmoothedValue::new(1.0);
        assert_eq!(sv.current(), 1.0);
        assert_eq!(sv.next_value(), 1.0);
    }

    #[test]
    fn test_smoothing() {
        let mut sv = SmoothedValue::new(0.0);
        sv.set_target(1.0, 4);

        assert!(sv.is_smoothing());
        assert_eq!(sv.next_value(), 0.25);
        assert_eq!(sv.next_value(), 0.5);
        assert_eq!(sv.next_value(), 0.75);
        assert_eq!(sv.next_value(), 1.0);
        assert!(!sv.is_smoothing());
    }

    #[test]
    fn test_zero_samples() {
        let mut sv = SmoothedValue::new(0.0);
        sv.set_target(1.0, 0);
        assert_eq!(sv.current(), 1.0);
        assert!(!sv.is_smoothing());
    }

    #[test]
    fn test_retarget_during_smoothing() {
        let mut sv = SmoothedValue::new(0.0);
        sv.set_target(1.0, 4);
        sv.next_value(); // 0.25
        sv.next_value(); // 0.5

        // Retarget while smoothing
        sv.set_target(0.0, 2);
        assert_eq!(sv.next_value(), 0.25);
        assert_eq!(sv.next_value(), 0.0);
    }
}
