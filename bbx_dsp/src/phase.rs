//! Phase accumulation for oscillators and other cyclic generators.
//!
//! Provides [`PhaseAccumulator`], a minimal stateful counter that produces
//! phase in cycles (`[0, 1)`) and wraps every sample.

use crate::math;

/// A phase counter in cycles, advanced once per sample.
///
/// Phase is stored in cycles rather than radians so that waveform readers
/// (tables, breakpoint curves, PolyBLEP) can consume it without a `1/τ` scale,
/// and wrapped every sample with `floor` so accumulated error never grows past
/// one cycle. With per-sample wrapping, `f32` holds relative pitch error near
/// `1e-7` (about 0.0002 cents), so `f64` is not needed in the hot path.
///
/// The increment is stored pre-divided (`frequency / sample_rate`) so the
/// per-sample cost is one add and one subtract.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct PhaseAccumulator {
    phase: f32,
    increment: f32,
}

impl PhaseAccumulator {
    /// Create an accumulator at phase zero with no motion.
    #[inline]
    pub const fn new() -> Self {
        Self {
            phase: 0.0,
            increment: 0.0,
        }
    }

    /// Set the per-sample increment from a frequency in Hz and a sample rate in Hz.
    ///
    /// Negative frequencies are allowed and run the phase backwards.
    #[inline]
    pub fn set_frequency(&mut self, frequency: f32, sample_rate: f32) {
        debug_assert!(sample_rate > 0.0, "sample rate must be positive");
        self.increment = frequency / sample_rate;
    }

    /// Set the per-sample increment directly, in cycles per sample.
    #[inline]
    pub fn set_increment(&mut self, increment: f32) {
        self.increment = increment;
    }

    /// The per-sample increment in cycles per sample.
    #[inline]
    pub fn increment(&self) -> f32 {
        self.increment
    }

    /// Return the current phase and advance by one sample.
    ///
    /// The returned value is the phase *before* the advance, so the first call
    /// after [`reset`](Self::reset) yields exactly `0.0`.
    ///
    /// Not an `Iterator`: the sequence never ends, so wrapping every sample in
    /// `Some` would only add an unwrap to the hot path.
    #[allow(clippy::should_implement_trait)]
    #[inline]
    pub fn next(&mut self) -> f32 {
        let current = self.phase;
        self.phase = wrap(current + self.increment);
        current
    }

    /// The current phase in cycles, always in `[0, 1)`.
    #[inline]
    pub fn phase(&self) -> f32 {
        self.phase
    }

    /// Jump to a phase in cycles. Values outside `[0, 1)` are wrapped.
    #[inline]
    pub fn set_phase(&mut self, phase: f32) {
        self.phase = wrap(phase);
    }

    /// Return the phase to zero, keeping the increment.
    ///
    /// The increment is a parameter, not state: a voice retriggering a note
    /// resets phase without having to re-derive its frequency.
    #[inline]
    pub fn reset(&mut self) {
        self.phase = 0.0;
    }
}

/// Wrap into `[0, 1)` without a branch so the caller can be vectorised later.
///
/// `libm`'s floor is used rather than the inherent method because the inherent
/// `f32::floor` is not available in `no_std` builds; both are exact.
#[inline]
fn wrap(phase: f32) -> f32 {
    phase - math::floor(phase)
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_RATE: f32 = 48_000.0;

    /// Distance between two phases on the unit circle, so `0.99999` and `0.00001` are close.
    fn wrapped_distance(a: f64, b: f64) -> f64 {
        let d = (a - b).rem_euclid(1.0);
        d.min(1.0 - d)
    }

    fn accumulator(frequency: f32) -> PhaseAccumulator {
        let mut accumulator = PhaseAccumulator::new();
        accumulator.set_frequency(frequency, SAMPLE_RATE);
        accumulator
    }

    #[test]
    fn test_new_is_stationary_at_zero() {
        let mut accumulator = PhaseAccumulator::new();
        assert_eq!(accumulator.phase(), 0.0);
        assert_eq!(accumulator.increment(), 0.0);
        for _ in 0..16 {
            assert_eq!(accumulator.next(), 0.0);
        }
        assert_eq!(PhaseAccumulator::default(), PhaseAccumulator::new());
    }

    #[test]
    fn test_set_frequency_divides_by_sample_rate() {
        let accumulator = accumulator(480.0);
        assert!((accumulator.increment() - 0.01).abs() < 1e-7);
    }

    #[test]
    fn test_next_returns_phase_before_advance() {
        let mut accumulator = PhaseAccumulator::new();
        accumulator.set_increment(0.25);
        assert_eq!(accumulator.next(), 0.0);
        assert_eq!(accumulator.next(), 0.25);
        assert_eq!(accumulator.next(), 0.5);
        assert_eq!(accumulator.next(), 0.75);
        assert_eq!(accumulator.next(), 0.0);
        assert_eq!(accumulator.phase(), 0.25);
    }

    #[test]
    fn test_returns_to_start_after_one_period() {
        for frequency in [100.0f32, 440.0, 1000.0] {
            let period = (SAMPLE_RATE / frequency).round() as usize;
            let mut accumulator = accumulator(frequency);
            let start = accumulator.phase() as f64;
            for _ in 0..period {
                accumulator.next();
            }
            // A rounded period differs from the true period, so measure against where the
            // ideal accumulator would be, not against the start phase.
            let expected = (period as f64 * frequency as f64 / SAMPLE_RATE as f64).fract();
            let error = wrapped_distance(accumulator.phase() as f64, start + expected);
            assert!(
                error < 1e-5,
                "{frequency} Hz: phase off by {error} after {period} samples"
            );
        }
    }

    #[test]
    fn test_tracks_f64_reference_within_one_period() {
        for frequency in [100.0f32, 440.0, 1000.0, 7_000.0] {
            let period = (SAMPLE_RATE / frequency).round() as usize;
            let mut accumulator = accumulator(frequency);
            let increment = frequency as f64 / SAMPLE_RATE as f64;
            for n in 0..period {
                let expected = (n as f64 * increment).fract();
                let error = wrapped_distance(accumulator.next() as f64, expected);
                assert!(error < 1e-5, "{frequency} Hz: sample {n} off by {error}");
            }
        }
    }

    #[test]
    fn test_phase_always_in_unit_interval() {
        for frequency in [0.0f32, 1.0, 440.0, 12_345.6, 23_999.0, -440.0] {
            let mut accumulator = accumulator(frequency);
            for n in 0..(SAMPLE_RATE as usize) {
                let phase = accumulator.next();
                assert!(
                    (0.0..1.0).contains(&phase),
                    "{frequency} Hz: phase {phase} out of range at sample {n}"
                );
            }
        }
    }

    #[test]
    fn test_wrap_when_sum_rounds_to_exactly_one() {
        let mut accumulator = PhaseAccumulator::new();
        accumulator.set_phase(1.0 - f32::EPSILON);
        accumulator.set_increment(f32::EPSILON);
        accumulator.next();
        assert_eq!(accumulator.phase(), 0.0);
    }

    #[test]
    fn test_negative_frequency_runs_backwards() {
        let mut accumulator = PhaseAccumulator::new();
        accumulator.set_increment(-0.25);
        assert_eq!(accumulator.next(), 0.0);
        assert_eq!(accumulator.next(), 0.75);
        assert_eq!(accumulator.next(), 0.5);
        assert_eq!(accumulator.next(), 0.25);
        assert_eq!(accumulator.next(), 0.0);
    }

    #[test]
    fn test_frequency_change_is_phase_continuous() {
        let mut accumulator = accumulator(440.0);
        for _ in 0..1000 {
            accumulator.next();
        }
        let before = accumulator.phase();

        accumulator.set_frequency(2_000.0, SAMPLE_RATE);
        assert_eq!(accumulator.next(), before, "changing frequency moved the phase");

        let step = accumulator.next() - before;
        let expected_step = 2_000.0 / SAMPLE_RATE;
        assert!(
            (step - expected_step).abs() < 1e-6,
            "first step after change was {step}, expected {expected_step}"
        );
    }

    #[test]
    fn test_split_render_matches_single_render() {
        let mut whole = accumulator(440.0);
        let mut split = accumulator(440.0);

        let expected: [f32; 500] = core::array::from_fn(|_| whole.next());

        let mut actual = [0.0f32; 500];
        for sample in &mut actual[..250] {
            *sample = split.next();
        }
        // A block re-applies its frequency at every process call, which must not perturb the phase.
        split.set_frequency(440.0, SAMPLE_RATE);
        for sample in &mut actual[250..] {
            *sample = split.next();
        }

        assert_eq!(expected, actual);
    }

    #[test]
    fn test_set_phase_wraps_into_unit_interval() {
        let mut accumulator = PhaseAccumulator::new();
        accumulator.set_phase(0.3);
        assert_eq!(accumulator.phase(), 0.3);
        accumulator.set_phase(1.75);
        assert!((accumulator.phase() - 0.75).abs() < 1e-7);
        accumulator.set_phase(-0.25);
        assert!((accumulator.phase() - 0.75).abs() < 1e-7);
        accumulator.set_phase(1.0);
        assert_eq!(accumulator.phase(), 0.0);
    }

    #[test]
    fn test_reset_clears_phase_and_keeps_increment() {
        let mut accumulator = accumulator(440.0);
        for _ in 0..100 {
            accumulator.next();
        }
        assert_ne!(accumulator.phase(), 0.0);

        accumulator.reset();
        assert_eq!(accumulator.phase(), 0.0);
        assert!((accumulator.increment() - 440.0 / SAMPLE_RATE).abs() < 1e-9);
        assert_eq!(accumulator.next(), 0.0);
        assert_ne!(accumulator.next(), 0.0);
    }
}
