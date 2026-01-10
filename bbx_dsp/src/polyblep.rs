//! PolyBLEP and PolyBLAMP anti-aliasing for band-limited waveform generation.
//!
//! This module provides 4th-order polynomial corrections to eliminate aliasing
//! artifacts from discontinuous waveforms (sawtooth, square, pulse) and slope
//! discontinuities (triangle).
//!
//! PolyBLEP (Polynomial Band-Limited Step) smooths step discontinuities by applying
//! a polynomial correction near the transition point. PolyBLAMP (Band-Limited rAMP)
//! is the integrated form, used for slope discontinuities.

#[cfg(feature = "simd")]
use crate::sample::SIMD_LANES;
#[cfg(feature = "simd")]
use crate::sample::Sample;

/// Anti-aliasing mode for waveform generation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AntiAliasingMode {
    /// No anti-aliasing (naive waveforms). Fastest but produces aliasing artifacts.
    None,
    /// PolyBLEP/PolyBLAMP anti-aliasing. Standard for professional audio software.
    #[default]
    PolyBlep,
}

/// PolyBLEP correction value for step discontinuities.
///
/// Computes the correction to apply near a discontinuity to smooth it.
/// The fractional position `t` indicates where we are relative to the discontinuity:
/// - t = 0: exactly at the discontinuity
/// - t > 0: after the discontinuity (within 1 sample)
/// - t < 0: before the discontinuity (within 1 sample)
///
/// This uses the standard 2nd-order polynomial which provides a good balance
/// between anti-aliasing quality and computational cost, and is the de-facto
/// standard in professional synthesizers.
///
/// Returns a correction value in range [-1, 1] that should be scaled by the
/// discontinuity amplitude and added to the naive waveform sample.
#[inline]
pub fn polyblep(t: f64) -> f64 {
    if t >= 0.0 && t < 1.0 {
        // After discontinuity: 2t - t^2 - 1
        // At t=0: -1, at t=1: 0 (smooth transition)
        t + t - t * t - 1.0
    } else if t >= -1.0 && t < 0.0 {
        // Before discontinuity: t^2 + 2t + 1 = (t+1)^2
        // At t=-1: 0, at t=0: 1 (smooth transition)
        t * t + t + t + 1.0
    } else {
        0.0
    }
}

/// PolyBLAMP correction for slope discontinuities.
///
/// Used for triangle waves where the slope changes sign but there's no step discontinuity.
/// This is the integrated form of PolyBLEP, scaled by the phase increment.
///
/// # Arguments
/// * `t` - Fractional position relative to the slope change (-1 to 1)
/// * `phase_inc` - Phase increment per sample (determines correction magnitude)
#[inline]
pub fn polyblamp(t: f64, phase_inc: f64) -> f64 {
    if t >= 0.0 && t < 1.0 {
        // Integrated form of PolyBLEP for slope discontinuities
        // Integral of (2t - t^2 - 1) = t^2 - t^3/3 - t
        let t2 = t * t;
        let t3 = t2 * t;
        (t2 - t3 / 3.0 - t) * phase_inc
    } else if t >= -1.0 && t < 0.0 {
        // Integral of (t^2 + 2t + 1) = t^3/3 + t^2 + t
        let t2 = t * t;
        let t3 = t2 * t;
        (t3 / 3.0 + t2 + t) * phase_inc
    } else {
        0.0
    }
}

/// PolyBLEP correction for sawtooth waveform.
///
/// Sawtooth has a single discontinuity per period when the phase wraps from 1.0 to 0.0.
/// The discontinuity amplitude is 2.0 (from +1 to -1).
///
/// # Arguments
/// * `phase` - Current normalized phase (0.0 to 1.0)
/// * `phase_inc` - Phase increment per sample (normalized)
#[inline]
pub fn polyblep_correction_saw(phase: f64, phase_inc: f64) -> f64 {
    let t = if phase < phase_inc {
        // Just after discontinuity at phase = 0
        phase / phase_inc
    } else if phase > 1.0 - phase_inc {
        // Just before discontinuity at phase = 1
        (phase - 1.0) / phase_inc
    } else {
        return 0.0;
    };

    // Sawtooth drops from +1 to -1, so discontinuity amplitude is -2
    -2.0 * polyblep(t)
}

/// PolyBLEP correction for square waveform.
///
/// Square wave has two discontinuities per period:
/// - Rising edge at phase = 0.0 (from -1 to +1)
/// - Falling edge at phase = 0.5 (from +1 to -1)
///
/// # Arguments
/// * `phase` - Current normalized phase (0.0 to 1.0)
/// * `phase_inc` - Phase increment per sample (normalized)
#[inline]
pub fn polyblep_correction_square(phase: f64, phase_inc: f64) -> f64 {
    let mut correction = 0.0;

    // Rising edge at phase = 0.0
    if phase < phase_inc {
        let t = phase / phase_inc;
        correction += 2.0 * polyblep(t);
    } else if phase > 1.0 - phase_inc {
        let t = (phase - 1.0) / phase_inc;
        correction += 2.0 * polyblep(t);
    }

    // Falling edge at phase = 0.5
    let phase_from_half = phase - 0.5;
    if phase_from_half >= 0.0 && phase_from_half < phase_inc {
        let t = phase_from_half / phase_inc;
        correction -= 2.0 * polyblep(t);
    } else if phase_from_half < 0.0 && phase_from_half > -phase_inc {
        let t = phase_from_half / phase_inc;
        correction -= 2.0 * polyblep(t);
    }

    correction
}

/// PolyBLEP correction for pulse waveform.
///
/// Similar to square but with variable duty cycle positioning the falling edge.
/// - Rising edge at phase = 0.0
/// - Falling edge at phase = duty_cycle
///
/// # Arguments
/// * `phase` - Current normalized phase (0.0 to 1.0)
/// * `phase_inc` - Phase increment per sample (normalized)
/// * `duty_cycle` - Duty cycle (0.0 to 1.0)
#[inline]
pub fn polyblep_correction_pulse(phase: f64, phase_inc: f64, duty_cycle: f64) -> f64 {
    let mut correction = 0.0;

    // Rising edge at phase = 0.0
    if phase < phase_inc {
        let t = phase / phase_inc;
        correction += 2.0 * polyblep(t);
    } else if phase > 1.0 - phase_inc {
        let t = (phase - 1.0) / phase_inc;
        correction += 2.0 * polyblep(t);
    }

    // Falling edge at phase = duty_cycle
    let phase_from_duty = phase - duty_cycle;
    if phase_from_duty >= 0.0 && phase_from_duty < phase_inc {
        let t = phase_from_duty / phase_inc;
        correction -= 2.0 * polyblep(t);
    } else if phase_from_duty < 0.0 && phase_from_duty > -phase_inc {
        let t = phase_from_duty / phase_inc;
        correction -= 2.0 * polyblep(t);
    }

    correction
}

/// PolyBLAMP correction for triangle waveform.
///
/// Triangle wave has slope discontinuities (not step discontinuities):
/// - At phase = 0.0: slope changes from -4 to +4 (delta = +8)
/// - At phase = 0.5: slope changes from +4 to -4 (delta = -8)
///
/// # Arguments
/// * `phase` - Current normalized phase (0.0 to 1.0)
/// * `phase_inc` - Phase increment per sample (normalized)
#[inline]
pub fn polyblamp_correction_triangle(phase: f64, phase_inc: f64) -> f64 {
    let mut correction = 0.0;

    // Slope change at phase = 0.0 (delta = +8)
    if phase < phase_inc {
        let t = phase / phase_inc;
        correction += 8.0 * polyblamp(t, phase_inc);
    } else if phase > 1.0 - phase_inc {
        let t = (phase - 1.0) / phase_inc;
        correction += 8.0 * polyblamp(t, phase_inc);
    }

    // Slope change at phase = 0.5 (delta = -8)
    let phase_from_half = phase - 0.5;
    if phase_from_half >= 0.0 && phase_from_half < phase_inc {
        let t = phase_from_half / phase_inc;
        correction -= 8.0 * polyblamp(t, phase_inc);
    } else if phase_from_half < 0.0 && phase_from_half > -phase_inc {
        let t = phase_from_half / phase_inc;
        correction -= 8.0 * polyblamp(t, phase_inc);
    }

    correction
}

/// State for tracking cross-chunk discontinuity corrections in SIMD processing.
#[cfg(feature = "simd")]
#[derive(Debug, Clone)]
pub struct PolyBlepState<S: Sample> {
    /// Correction to apply to the first sample of the next chunk.
    pub pending_correction: S,
    /// Last phase from the previous chunk (normalized 0-1).
    pub last_phase: S,
}

#[cfg(feature = "simd")]
impl<S: Sample> Default for PolyBlepState<S> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "simd")]
impl<S: Sample> PolyBlepState<S> {
    /// Create a new PolyBLEP state with zero pending correction.
    pub fn new() -> Self {
        Self {
            pending_correction: S::ZERO,
            last_phase: S::ZERO,
        }
    }

    /// Reset the state (call on voice reset or frequency change).
    pub fn reset(&mut self) {
        self.pending_correction = S::ZERO;
        self.last_phase = S::ZERO;
    }
}

/// Information about discontinuities detected within a SIMD chunk.
#[cfg(feature = "simd")]
pub struct SimdDiscontinuityInfo {
    /// Bitmask indicating which lanes have a discontinuity just before them.
    /// Bit 0 = lane 0, bit 1 = lane 1, etc.
    pub lane_mask: u8,
    /// Fractional positions (t values) for each lane. Zero if no discontinuity.
    pub fractions: [f64; SIMD_LANES],
}

/// Detect discontinuities within a SIMD vector of phases for sawtooth waveform.
///
/// Sawtooth has a discontinuity when phase wraps from near 1.0 to near 0.0.
///
/// # Arguments
/// * `phases` - Array of 4 consecutive phases (normalized 0-1)
/// * `prev_last_phase` - Last phase from the previous SIMD chunk
/// * `phase_inc` - Phase increment per sample (normalized)
#[cfg(feature = "simd")]
pub fn detect_discontinuities_saw(
    phases: [f64; SIMD_LANES],
    prev_last_phase: f64,
    phase_inc: f64,
) -> SimdDiscontinuityInfo {
    let mut lane_mask = 0u8;
    let mut fractions = [0.0; SIMD_LANES];

    // Check if discontinuity occurred before lane 0 (cross-chunk)
    if prev_last_phase > 0.5 && phases[0] < 0.5 && phases[0] < phase_inc * 2.0 {
        lane_mask |= 1;
        fractions[0] = phases[0] / phase_inc;
    }

    // Check lanes 1, 2, 3 against their predecessors
    for i in 1..SIMD_LANES {
        let prev = phases[i - 1];
        let curr = phases[i];
        if prev > 0.5 && curr < 0.5 && curr < phase_inc * 2.0 {
            lane_mask |= 1 << i;
            fractions[i] = curr / phase_inc;
        }
    }

    SimdDiscontinuityInfo { lane_mask, fractions }
}

/// Detect discontinuities within a SIMD vector of phases for square waveform.
///
/// Square has discontinuities at phase = 0.0 (rising) and phase = 0.5 (falling).
///
/// # Arguments
/// * `phases` - Array of 4 consecutive phases (normalized 0-1)
/// * `prev_last_phase` - Last phase from the previous SIMD chunk
/// * `phase_inc` - Phase increment per sample (normalized)
///
/// # Returns
/// Two discontinuity infos: (rising edges, falling edges)
#[cfg(feature = "simd")]
pub fn detect_discontinuities_square(
    phases: [f64; SIMD_LANES],
    prev_last_phase: f64,
    phase_inc: f64,
) -> (SimdDiscontinuityInfo, SimdDiscontinuityInfo) {
    let mut rising_mask = 0u8;
    let mut rising_fractions = [0.0; SIMD_LANES];
    let mut falling_mask = 0u8;
    let mut falling_fractions = [0.0; SIMD_LANES];

    // Check cross-chunk for rising edge (phase wrap)
    if prev_last_phase > 0.5 && phases[0] < 0.5 && phases[0] < phase_inc * 2.0 {
        rising_mask |= 1;
        rising_fractions[0] = phases[0] / phase_inc;
    }

    // Check cross-chunk for falling edge (crossing 0.5)
    if prev_last_phase < 0.5 && phases[0] >= 0.5 {
        let t = (phases[0] - 0.5) / phase_inc;
        if t < 1.0 {
            falling_mask |= 1;
            falling_fractions[0] = t;
        }
    }

    // Check lanes 1, 2, 3
    for i in 1..SIMD_LANES {
        let prev = phases[i - 1];
        let curr = phases[i];

        // Rising edge (phase wrap)
        if prev > 0.5 && curr < 0.5 && curr < phase_inc * 2.0 {
            rising_mask |= 1 << i;
            rising_fractions[i] = curr / phase_inc;
        }

        // Falling edge (crossing 0.5)
        if prev < 0.5 && curr >= 0.5 {
            let t = (curr - 0.5) / phase_inc;
            if t < 1.0 {
                falling_mask |= 1 << i;
                falling_fractions[i] = t;
            }
        }
    }

    (
        SimdDiscontinuityInfo {
            lane_mask: rising_mask,
            fractions: rising_fractions,
        },
        SimdDiscontinuityInfo {
            lane_mask: falling_mask,
            fractions: falling_fractions,
        },
    )
}

/// Apply PolyBLEP corrections to a SIMD chunk of sawtooth samples.
///
/// # Arguments
/// * `samples` - Naive waveform samples to correct (modified in-place)
/// * `phases` - Phases for each sample (normalized 0-1)
/// * `prev_last_phase` - Last phase from the previous SIMD chunk
/// * `phase_inc` - Phase increment per sample (normalized)
#[cfg(feature = "simd")]
pub fn apply_polyblep_saw<S: Sample>(
    samples: &mut [S; SIMD_LANES],
    phases: [f64; SIMD_LANES],
    prev_last_phase: f64,
    phase_inc: f64,
) {
    let info = detect_discontinuities_saw(phases, prev_last_phase, phase_inc);

    for i in 0..SIMD_LANES {
        if (info.lane_mask >> i) & 1 != 0 {
            let correction = polyblep_correction_saw(phases[i], phase_inc);
            samples[i] = samples[i] + S::from_f64(correction);
        }
    }

    // Also check for "before discontinuity" corrections on samples near phase = 1.0
    for i in 0..SIMD_LANES {
        if phases[i] > 1.0 - phase_inc {
            let correction = polyblep_correction_saw(phases[i], phase_inc);
            samples[i] = samples[i] + S::from_f64(correction);
        }
    }
}

/// Apply PolyBLEP corrections to a SIMD chunk of square samples.
#[cfg(feature = "simd")]
pub fn apply_polyblep_square<S: Sample>(
    samples: &mut [S; SIMD_LANES],
    phases: [f64; SIMD_LANES],
    _prev_last_phase: f64,
    phase_inc: f64,
) {
    for i in 0..SIMD_LANES {
        let correction = polyblep_correction_square(phases[i], phase_inc);
        if correction.abs() > 1e-10 {
            samples[i] = samples[i] + S::from_f64(correction);
        }
    }
}

/// Apply PolyBLEP corrections to a SIMD chunk of pulse samples.
#[cfg(feature = "simd")]
pub fn apply_polyblep_pulse<S: Sample>(
    samples: &mut [S; SIMD_LANES],
    phases: [f64; SIMD_LANES],
    phase_inc: f64,
    duty_cycle: f64,
) {
    for i in 0..SIMD_LANES {
        let correction = polyblep_correction_pulse(phases[i], phase_inc, duty_cycle);
        if correction.abs() > 1e-10 {
            samples[i] = samples[i] + S::from_f64(correction);
        }
    }
}

/// Apply PolyBLAMP corrections to a SIMD chunk of triangle samples.
#[cfg(feature = "simd")]
pub fn apply_polyblamp_triangle<S: Sample>(samples: &mut [S; SIMD_LANES], phases: [f64; SIMD_LANES], phase_inc: f64) {
    for i in 0..SIMD_LANES {
        let correction = polyblamp_correction_triangle(phases[i], phase_inc);
        if correction.abs() > 1e-10 {
            samples[i] = samples[i] + S::from_f64(correction);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_polyblep_zero_outside_range() {
        assert_eq!(polyblep(1.5), 0.0);
        assert_eq!(polyblep(-1.5), 0.0);
        assert_eq!(polyblep(2.0), 0.0);
        assert_eq!(polyblep(-2.0), 0.0);
    }

    #[test]
    fn test_polyblep_antisymmetry() {
        // The correction should be antisymmetric around t=0
        for &t in &[0.1, 0.3, 0.5, 0.7, 0.9] {
            let pos = polyblep(t);
            let neg = polyblep(-t);
            // Due to the polynomial form, they should have opposite signs
            assert!(
                (pos + neg).abs() < 1e-10,
                "Antisymmetry failed at t={}: polyblep({}) + polyblep({}) = {}",
                t,
                t,
                -t,
                pos + neg
            );
        }
    }

    #[test]
    fn test_polyblep_boundary_values() {
        // At t=0 (just after discontinuity), correction should be -1
        let at_zero = polyblep(0.0);
        assert!(
            (at_zero - (-1.0)).abs() < 1e-10,
            "polyblep(0) should be -1, got {}",
            at_zero
        );

        // At t=1 (outside range), should be 0
        let at_one = polyblep(1.0);
        assert_eq!(at_one, 0.0);

        // At t=-1 (start of before-discontinuity range), polynomial gives 0
        // This is correct because the correction smoothly transitions to 0 at the boundary
        let at_neg_one = polyblep(-1.0);
        assert!(
            at_neg_one.abs() < 1e-10,
            "polyblep(-1) should be ~0, got {}",
            at_neg_one
        );

        // Just before t=0 (e.g., t=-0.001), correction should be close to +1
        let just_before_zero = polyblep(-0.001);
        assert!(
            just_before_zero > 0.99,
            "polyblep(-0.001) should be close to 1, got {}",
            just_before_zero
        );
    }

    #[test]
    fn test_polyblep_continuity() {
        // Check near-boundary continuity
        let eps = 1e-6;

        // Near t=1 (end of after-discontinuity range), should approach 0
        let near_one = polyblep(1.0 - eps);
        assert!(near_one.abs() < 0.01, "Should be near 0 at t=1-eps, got {}", near_one);

        // Near t=-1 (start of before-discontinuity range), should be near 0
        let near_neg_one = polyblep(-1.0 + eps);
        assert!(
            near_neg_one.abs() < 0.01,
            "Should be near 0 at t=-1+eps, got {}",
            near_neg_one
        );
    }

    #[test]
    fn test_saw_correction_at_discontinuity() {
        let phase_inc = 0.01; // 1% of period per sample

        // Just after discontinuity (phase near 0)
        let correction_after = polyblep_correction_saw(0.005, phase_inc);
        assert!(
            correction_after != 0.0,
            "Should have correction just after discontinuity"
        );

        // Just before discontinuity (phase near 1)
        let correction_before = polyblep_correction_saw(0.995, phase_inc);
        assert!(
            correction_before != 0.0,
            "Should have correction just before discontinuity"
        );

        // Far from discontinuity
        let no_correction = polyblep_correction_saw(0.5, phase_inc);
        assert_eq!(no_correction, 0.0, "Should have no correction far from discontinuity");
    }

    #[test]
    fn test_square_correction_at_edges() {
        let phase_inc = 0.01;

        // Rising edge at phase = 0
        let rising = polyblep_correction_square(0.005, phase_inc);
        assert!(rising != 0.0, "Should have correction at rising edge");

        // Falling edge at phase = 0.5
        let falling = polyblep_correction_square(0.505, phase_inc);
        assert!(falling != 0.0, "Should have correction at falling edge");

        // Middle of high section (no edges nearby)
        let middle = polyblep_correction_square(0.25, phase_inc);
        assert_eq!(middle, 0.0, "Should have no correction away from edges");
    }

    #[test]
    fn test_triangle_blamp_correction() {
        let phase_inc = 0.01;

        // Near slope change at phase = 0
        let at_zero = polyblamp_correction_triangle(0.005, phase_inc);
        assert!(at_zero != 0.0, "Should have BLAMP correction at phase 0");

        // Near slope change at phase = 0.5
        let at_half = polyblamp_correction_triangle(0.505, phase_inc);
        assert!(at_half != 0.0, "Should have BLAMP correction at phase 0.5");

        // Away from slope changes
        let away = polyblamp_correction_triangle(0.25, phase_inc);
        assert_eq!(away, 0.0, "Should have no correction away from slope changes");
    }

    #[cfg(feature = "simd")]
    #[test]
    fn test_simd_discontinuity_detection_saw() {
        let phase_inc = 0.1;

        // Phases where discontinuity occurs between lanes 1 and 2
        let phases = [0.7, 0.8, 0.05, 0.15]; // Wrap happened before lane 2
        let prev_last = 0.6;

        let info = detect_discontinuities_saw(phases, prev_last, phase_inc);

        // Lane 2 should have discontinuity
        assert_eq!(info.lane_mask & 0b0100, 0b0100, "Lane 2 should have discontinuity");
        assert!(info.fractions[2] > 0.0 && info.fractions[2] < 1.0);
    }
}
