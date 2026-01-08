//! Parameter smoothing utilities for click-free parameter changes.
//!
//! Provides [`SmoothedValue`] for interpolating between parameter values over time,
//! avoiding audible clicks when parameters change abruptly. Supports both
//! [`Linear`] and [`Multiplicative`] (exponential) smoothing strategies.

use std::marker::PhantomData;

use crate::sample::Sample;

const INV_1000: f64 = 1.0 / 1000.0;

/// Minimum ramp length in milliseconds to prevent instant value jumps (clicks).
/// 0.01ms is fast enough to sound near-instantaneous while still providing
/// a brief transition to avoid discontinuities.
const MIN_RAMP_LENGTH_MS: f64 = 0.01;

/// Check if two Sample values are approximately equal.
#[inline]
fn is_approximately_equal<S: Sample>(a: S, b: S) -> bool {
    let a_f64 = a.to_f64();
    let b_f64 = b.to_f64();
    let epsilon = f64::EPSILON * a_f64.abs().max(b_f64.abs()).max(1.0);
    (a_f64 - b_f64).abs() < epsilon
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
    /// Returns f64 for precision in smoothing calculations.
    fn update_increment<S: Sample>(current: S, target: S, num_samples: f64) -> f64;

    /// Apply the increment to the current value.
    fn apply_increment<S: Sample>(current: S, increment: f64) -> S;

    /// Apply the increment multiple times (for skip).
    fn apply_increment_n<S: Sample>(current: S, increment: f64, n: i32) -> S;

    /// Default initial value for this smoothing type.
    fn default_value<S: Sample>() -> S;
}

impl SmoothingStrategy for Linear {
    #[inline]
    fn update_increment<S: Sample>(current: S, target: S, num_samples: f64) -> f64 {
        (target.to_f64() - current.to_f64()) / num_samples
    }

    #[inline]
    fn apply_increment<S: Sample>(current: S, increment: f64) -> S {
        S::from_f64(current.to_f64() + increment)
    }

    #[inline]
    fn apply_increment_n<S: Sample>(current: S, increment: f64, n: i32) -> S {
        S::from_f64(current.to_f64() + increment * n as f64)
    }

    #[inline]
    fn default_value<S: Sample>() -> S {
        S::ZERO
    }
}

impl SmoothingStrategy for Multiplicative {
    #[inline]
    fn update_increment<S: Sample>(current: S, target: S, num_samples: f64) -> f64 {
        let current_f64 = current.to_f64();
        let target_f64 = target.to_f64();
        if current_f64 > 0.0 && target_f64 > 0.0 {
            (target_f64 / current_f64).ln() / num_samples
        } else {
            0.0
        }
    }

    #[inline]
    fn apply_increment<S: Sample>(current: S, increment: f64) -> S {
        S::from_f64(current.to_f64() * increment.exp())
    }

    #[inline]
    fn apply_increment_n<S: Sample>(current: S, increment: f64, n: i32) -> S {
        S::from_f64(current.to_f64() * (increment * n as f64).exp())
    }

    #[inline]
    fn default_value<S: Sample>() -> S {
        S::ONE
    }
}

/// A value that smoothly transitions to a target over time.
///
/// Uses the specified `SmoothingStrategy` to interpolate between values,
/// avoiding clicks and pops when parameters change.
///
/// Generic over `S: Sample` for the value type and `T: SmoothingStrategy`
/// for the interpolation method.
#[derive(Debug, Clone)]
pub struct SmoothedValue<S: Sample, T: SmoothingStrategy> {
    sample_rate: f64,
    ramp_length_millis: f64,
    current_value: S,
    target_value: S,
    increment: f64, // Keep as f64 for precision
    _marker: PhantomData<T>,
}

impl<S: Sample, T: SmoothingStrategy> SmoothedValue<S, T> {
    /// Create a new `SmoothedValue` with the given initial value.
    ///
    /// Uses default sample rate (44100.0) and ramp length (50ms).
    pub fn new(initial_value: S) -> Self {
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
    /// A minimum ramp length of 0.01ms is enforced to prevent clicks.
    pub fn reset(&mut self, sample_rate: f64, ramp_length_millis: f64) {
        if sample_rate > 0.0 && ramp_length_millis >= 0.0 {
            self.sample_rate = sample_rate;
            self.ramp_length_millis = ramp_length_millis.max(MIN_RAMP_LENGTH_MS);
            self.update_increment();
        }
    }

    /// Set a new target value to smooth towards.
    #[inline]
    pub fn set_target_value(&mut self, value: S) {
        self.target_value = value;
        self.update_increment();
    }

    /// Get the next smoothed value (call once per sample).
    #[inline]
    pub fn get_next_value(&mut self) -> S {
        if is_approximately_equal(self.current_value, self.target_value) {
            self.current_value = self.target_value;
            return self.current_value;
        }

        self.current_value = T::apply_increment::<S>(self.current_value, self.increment);

        // Prevent overshoot
        let current_f64 = self.current_value.to_f64();
        let target_f64 = self.target_value.to_f64();
        if (self.increment > 0.0 && current_f64 > target_f64) || (self.increment < 0.0 && current_f64 < target_f64) {
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

        let new_value = T::apply_increment_n::<S>(self.current_value, self.increment, num_samples);

        // Prevent overshoot
        let new_f64 = new_value.to_f64();
        let target_f64 = self.target_value.to_f64();
        if (self.increment > 0.0 && new_f64 > target_f64) || (self.increment < 0.0 && new_f64 < target_f64) {
            self.current_value = self.target_value;
        } else {
            self.current_value = new_value;
        }
    }

    /// Get the current value without advancing.
    #[inline]
    pub fn current(&self) -> S {
        self.current_value
    }

    /// Get the target value.
    #[inline]
    pub fn target(&self) -> S {
        self.target_value
    }

    /// Check if the value is still smoothing towards the target.
    #[inline]
    pub fn is_smoothing(&self) -> bool {
        !is_approximately_equal(self.current_value, self.target_value)
    }

    /// Immediately set both current and target to a value (no smoothing).
    #[inline]
    pub fn set_immediate(&mut self, value: S) {
        self.current_value = value;
        self.target_value = value;
        self.increment = 0.0;
    }

    /// Recalculate the increment based on current timing parameters.
    #[inline]
    fn update_increment(&mut self) {
        // Enforce minimum ramp length to prevent clicks
        let ramp_ms = self.ramp_length_millis.max(MIN_RAMP_LENGTH_MS);
        let num_samples = ramp_ms * self.sample_rate * INV_1000;
        self.increment = T::update_increment::<S>(self.current_value, self.target_value, num_samples);
    }
}

impl<S: Sample, T: SmoothingStrategy> Default for SmoothedValue<S, T> {
    fn default() -> Self {
        Self::new(T::default_value::<S>())
    }
}

/// Linear smoothed value - uses additive interpolation.
pub type LinearSmoothedValue<S> = SmoothedValue<S, Linear>;

/// Multiplicative smoothed value - uses exponential interpolation.
/// Better for parameters like gain where equal ratios should feel equal.
pub type MultiplicativeSmoothedValue<S> = SmoothedValue<S, Multiplicative>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_linear_immediate_value() {
        let mut sv = LinearSmoothedValue::<f32>::new(1.0);
        assert_eq!(sv.current(), 1.0);
        assert_eq!(sv.get_next_value(), 1.0);
        assert!(!sv.is_smoothing());
    }

    #[test]
    fn test_linear_smoothing_f32() {
        let mut sv = LinearSmoothedValue::<f32>::new(0.0);
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
    fn test_linear_smoothing_f64() {
        let mut sv = LinearSmoothedValue::<f64>::new(0.0);
        sv.reset(1000.0, 4.0);
        sv.set_target_value(1.0);

        assert!(sv.is_smoothing());
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
        // With minimum ramp enforcement (0.01ms), zero ramp is treated as 0.01ms
        // At 44100 Hz, that's about 0.44 samples, so it reaches target quickly
        let mut sv = LinearSmoothedValue::<f32>::new(0.0);
        sv.reset(44100.0, 0.0);
        sv.set_target_value(1.0);

        // Should still be smoothing (minimum ramp applied)
        assert!(sv.is_smoothing());

        // But should reach target after a few samples
        for _ in 0..5 {
            sv.get_next_value();
        }
        assert_eq!(sv.current(), 1.0);
        assert!(!sv.is_smoothing());
    }

    #[test]
    fn test_skip() {
        let mut sv = LinearSmoothedValue::<f32>::new(0.0);
        sv.reset(1000.0, 10.0); // 10 samples
        sv.set_target_value(1.0);

        sv.skip(5);
        assert!((sv.current() - 0.5).abs() < 0.01);

        sv.skip(10); // Overshoot protection
        assert_eq!(sv.current(), 1.0);
    }

    #[test]
    fn test_retarget_during_smoothing() {
        let mut sv = LinearSmoothedValue::<f32>::new(0.0);
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
        let sv = MultiplicativeSmoothedValue::<f32>::default();
        assert_eq!(sv.current(), 1.0);
    }

    #[test]
    fn test_multiplicative_smoothing() {
        let mut sv = MultiplicativeSmoothedValue::<f32>::new(1.0);
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
        let mut sv = LinearSmoothedValue::<f32>::new(0.0);
        sv.reset(44100.0, 50.0);
        sv.set_target_value(1.0);
        assert!(sv.is_smoothing());

        sv.set_immediate(0.5);
        assert_eq!(sv.current(), 0.5);
        assert_eq!(sv.target(), 0.5);
        assert!(!sv.is_smoothing());
    }
}
