//! Parameter smoothing utilities for click-free parameter changes.
//!
//! Provides [`SmoothedValue`] for interpolating between parameter values over time,
//! avoiding audible clicks when parameters change abruptly. Supports both
//! [`Linear`] and [`Multiplicative`] (exponential) smoothing strategies.

use std::marker::PhantomData;

const INV_1000: f32 = 1.0 / 1000.0;

/// Check if two f32 values are approximately equal.
#[inline]
fn is_approximately_equal(a: f32, b: f32) -> bool {
    let epsilon = f32::EPSILON * a.abs().max(b.abs()).max(1.0);
    (a - b).abs() < epsilon
}

/// Marker type for linear smoothing.
///
/// Uses additive interpolation: `current + increment`.
/// Best for parameters with linear perception (e.g., pan position).
#[derive(Debug, Clone, Copy, Default)]
pub struct Linear;

/// Marker type for multiplicative (exponential) smoothing.
///
/// Uses exponential interpolation: `current * e^increment`.
/// Best for parameters with logarithmic perception (e.g., gain, frequency).
#[derive(Debug, Clone, Copy, Default)]
pub struct Multiplicative;

/// Trait defining smoothing behavior for different interpolation strategies.
pub trait SmoothingStrategy: Clone + Default {
    /// Calculate the increment value for smoothing.
    fn update_increment(current: f32, target: f32, num_samples: f32) -> f32;

    /// Apply the increment to the current value.
    fn apply_increment(current: f32, increment: f32) -> f32;

    /// Apply the increment multiple times (for skip).
    fn apply_increment_n(current: f32, increment: f32, n: i32) -> f32;

    /// Default initial value for this smoothing type.
    fn default_value() -> f32;
}

impl SmoothingStrategy for Linear {
    #[inline]
    fn update_increment(current: f32, target: f32, num_samples: f32) -> f32 {
        (target - current) / num_samples
    }

    #[inline]
    fn apply_increment(current: f32, increment: f32) -> f32 {
        current + increment
    }

    #[inline]
    fn apply_increment_n(current: f32, increment: f32, n: i32) -> f32 {
        current + increment * n as f32
    }

    #[inline]
    fn default_value() -> f32 {
        0.0
    }
}

impl SmoothingStrategy for Multiplicative {
    #[inline]
    fn update_increment(current: f32, target: f32, num_samples: f32) -> f32 {
        if current > 0.0 && target > 0.0 {
            (target / current).ln() / num_samples
        } else {
            0.0
        }
    }

    #[inline]
    fn apply_increment(current: f32, increment: f32) -> f32 {
        current * increment.exp()
    }

    #[inline]
    fn apply_increment_n(current: f32, increment: f32, n: i32) -> f32 {
        current * (increment * n as f32).exp()
    }

    #[inline]
    fn default_value() -> f32 {
        1.0
    }
}

/// A value that smoothly transitions to a target over time.
///
/// Uses the specified `SmoothingStrategy` to interpolate between values,
/// avoiding clicks and pops when parameters change.
#[derive(Debug, Clone)]
pub struct SmoothedValue<T: SmoothingStrategy> {
    sample_rate: f64,
    ramp_length_millis: f64,
    current_value: f32,
    target_value: f32,
    increment: f32,
    _marker: PhantomData<T>,
}

impl<T: SmoothingStrategy> SmoothedValue<T> {
    /// Create a new `SmoothedValue` with the given initial value.
    ///
    /// Uses default sample rate (44100.0) and ramp length (50ms).
    pub fn new(initial_value: f32) -> Self {
        Self {
            sample_rate: 44100.0,
            ramp_length_millis: 50.0,
            current_value: initial_value,
            target_value: initial_value,
            increment: 0.0,
            _marker: PhantomData,
        }
    }

    /// Reset the sample rate and ramp length.
    ///
    /// This recalculates the increment based on the new timing parameters.
    pub fn reset(&mut self, sample_rate: f64, ramp_length_millis: f64) {
        if sample_rate > 0.0 && ramp_length_millis >= 0.0 {
            self.sample_rate = sample_rate;
            self.ramp_length_millis = ramp_length_millis;
            self.update_increment();
        }
    }

    /// Set a new target value to smooth towards.
    #[inline]
    pub fn set_target_value(&mut self, value: f32) {
        self.target_value = value;
        self.update_increment();
    }

    /// Get the next smoothed value (call once per sample).
    #[inline]
    pub fn get_next_value(&mut self) -> f32 {
        if is_approximately_equal(self.current_value, self.target_value) {
            self.current_value = self.target_value;
            return self.current_value;
        }

        self.current_value = T::apply_increment(self.current_value, self.increment);

        // Prevent overshoot
        if (self.increment > 0.0 && self.current_value > self.target_value)
            || (self.increment < 0.0 && self.current_value < self.target_value)
        {
            self.current_value = self.target_value;
        }

        self.current_value
    }

    /// Skip ahead by the specified number of samples.
    #[inline]
    pub fn skip(&mut self, num_samples: i32) {
        if is_approximately_equal(self.current_value, self.target_value) {
            self.current_value = self.target_value;
            return;
        }

        let new_value = T::apply_increment_n(self.current_value, self.increment, num_samples);

        // Prevent overshoot
        if (self.increment > 0.0 && new_value > self.target_value)
            || (self.increment < 0.0 && new_value < self.target_value)
        {
            self.current_value = self.target_value;
        } else {
            self.current_value = new_value;
        }
    }

    /// Get the current value without advancing.
    #[inline]
    pub fn current(&self) -> f32 {
        self.current_value
    }

    /// Get the target value.
    #[inline]
    pub fn target(&self) -> f32 {
        self.target_value
    }

    /// Check if the value is still smoothing towards the target.
    #[inline]
    pub fn is_smoothing(&self) -> bool {
        !is_approximately_equal(self.current_value, self.target_value)
    }

    /// Immediately set both current and target to a value (no smoothing).
    #[inline]
    pub fn set_immediate(&mut self, value: f32) {
        self.current_value = value;
        self.target_value = value;
        self.increment = 0.0;
    }

    /// Recalculate the increment based on current timing parameters.
    #[inline]
    fn update_increment(&mut self) {
        if self.ramp_length_millis <= 0.0 {
            self.current_value = self.target_value;
            self.increment = 0.0;
            return;
        }

        let num_samples = (self.ramp_length_millis * self.sample_rate) as f32 * INV_1000;
        self.increment = T::update_increment(self.current_value, self.target_value, num_samples);
    }
}

impl<T: SmoothingStrategy> Default for SmoothedValue<T> {
    fn default() -> Self {
        Self::new(T::default_value())
    }
}

/// Linear smoothed value - uses additive interpolation.
pub type LinearSmoothedValue = SmoothedValue<Linear>;

/// Multiplicative smoothed value - uses exponential interpolation.
/// Better for parameters like gain where equal ratios should feel equal.
pub type MultiplicativeSmoothedValue = SmoothedValue<Multiplicative>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_linear_immediate_value() {
        let mut sv = LinearSmoothedValue::new(1.0);
        assert_eq!(sv.current(), 1.0);
        assert_eq!(sv.get_next_value(), 1.0);
        assert!(!sv.is_smoothing());
    }

    #[test]
    fn test_linear_smoothing() {
        let mut sv = LinearSmoothedValue::new(0.0);
        // Set up for exactly 4 samples of smoothing
        sv.reset(1000.0, 4.0); // 1000 Hz, 4ms = 4 samples
        sv.set_target_value(1.0);

        assert!(sv.is_smoothing());
        // Each step should advance by 0.25
        let v1 = sv.get_next_value();
        assert!((v1 - 0.25).abs() < 0.01, "Expected ~0.25, got {}", v1);

        let v2 = sv.get_next_value();
        assert!((v2 - 0.5).abs() < 0.01, "Expected ~0.5, got {}", v2);

        let v3 = sv.get_next_value();
        assert!((v3 - 0.75).abs() < 0.01, "Expected ~0.75, got {}", v3);

        let v4 = sv.get_next_value();
        assert!((v4 - 1.0).abs() < 0.01, "Expected ~1.0, got {}", v4);
    }

    #[test]
    fn test_zero_ramp_length() {
        let mut sv = LinearSmoothedValue::new(0.0);
        sv.reset(44100.0, 0.0); // Zero ramp = immediate
        sv.set_target_value(1.0);
        assert_eq!(sv.current(), 1.0);
        assert!(!sv.is_smoothing());
    }

    #[test]
    fn test_skip() {
        let mut sv = LinearSmoothedValue::new(0.0);
        sv.reset(1000.0, 10.0); // 10 samples
        sv.set_target_value(1.0);

        sv.skip(5);
        assert!((sv.current() - 0.5).abs() < 0.01);

        sv.skip(10); // Overshoot protection
        assert_eq!(sv.current(), 1.0);
    }

    #[test]
    fn test_retarget_during_smoothing() {
        let mut sv = LinearSmoothedValue::new(0.0);
        sv.reset(1000.0, 4.0);
        sv.set_target_value(1.0);

        sv.get_next_value(); // 0.25
        sv.get_next_value(); // 0.5

        // Retarget back to 0 while at 0.5
        sv.set_target_value(0.0);
        // Should now smooth from ~0.5 to 0.0 over 4 samples
        assert!(sv.is_smoothing());
    }

    #[test]
    fn test_multiplicative_default() {
        let sv = MultiplicativeSmoothedValue::default();
        assert_eq!(sv.current(), 1.0);
    }

    #[test]
    fn test_multiplicative_smoothing() {
        let mut sv = MultiplicativeSmoothedValue::new(1.0);
        sv.reset(1000.0, 10.0);
        sv.set_target_value(2.0);

        assert!(sv.is_smoothing());

        // Should exponentially approach 2.0
        let mut prev = sv.current();
        for _ in 0..10 {
            let curr = sv.get_next_value();
            assert!(curr > prev, "Value should increase");
            prev = curr;
        }
    }

    #[test]
    fn test_set_immediate() {
        let mut sv = LinearSmoothedValue::new(0.0);
        sv.reset(44100.0, 50.0);
        sv.set_target_value(1.0);
        assert!(sv.is_smoothing());

        sv.set_immediate(0.5);
        assert_eq!(sv.current(), 0.5);
        assert_eq!(sv.target(), 0.5);
        assert!(!sv.is_smoothing());
    }
}
