//! PolyBLEP and PolyBLAMP anti-aliasing for band-limited waveform generation.
//!
//! This module provides polynomial corrections to eliminate aliasing artifacts from
//! discontinuous waveforms (sawtooth, square, pulse) and slope discontinuities (triangle).
//!
//! PolyBLEP (Polynomial Band-Limited Step) smooths step discontinuities by applying
//! a polynomial correction near the transition point. PolyBLAMP (Band-Limited rAMP)
//! is the integrated form, used for slope discontinuities.
//!
//! All functions are generic over the `Sample` trait for efficient f32/f64 processing.

#[cfg(feature = "simd")]
use std::simd::StdFloat;

#[cfg(feature = "simd")]
use crate::sample::SIMD_LANES;
use crate::sample::Sample;

/// Fractional part of a Sample value.
#[inline]
fn fract<S: Sample>(x: S) -> S {
    x - S::from_f64(x.to_f64().floor())
}

/// PolyBLEP correction for step discontinuities.
///
/// Takes raw phase and phase increment (both normalized 0-1), handles normalization
/// internally. Returns a correction value to SUBTRACT from the naive waveform.
///
/// # Arguments
/// * `t` - Current phase (0.0 to 1.0, normalized)
/// * `dt` - Phase increment per sample (normalized)
#[inline]
pub fn poly_blep<S: Sample>(t: S, dt: S) -> S {
    let two = S::from_f64(2.0);
    if t < dt {
        let t_norm = t / dt;
        two * t_norm - t_norm * t_norm - S::ONE
    } else if t > S::ONE - dt {
        let t_norm = (t - S::ONE) / dt;
        t_norm * t_norm + two * t_norm + S::ONE
    } else {
        S::ZERO
    }
}

/// PolyBLAMP correction for slope discontinuities.
///
/// Used for triangle waves where the slope changes sign but there's no step discontinuity.
/// This is the integrated form of PolyBLEP.
///
/// # Arguments
/// * `t` - Current phase (0.0 to 1.0, normalized)
/// * `dt` - Phase increment per sample (normalized)
#[inline]
pub fn poly_blamp<S: Sample>(t: S, dt: S) -> S {
    let third = S::from_f64(1.0 / 3.0);
    if t < dt {
        let t_norm = t / dt;
        let t2 = t_norm * t_norm;
        let t3 = t2 * t_norm;
        (t2 - t3 * third - t_norm) * dt
    } else if t > S::ONE - dt {
        let t_norm = (t - S::ONE) / dt;
        let t2 = t_norm * t_norm;
        let t3 = t2 * t_norm;
        (t3 * third + t2 + t_norm) * dt
    } else {
        S::ZERO
    }
}

/// Branchless SIMD PolyBLEP correction for 4 phase values.
///
/// Computes all branches and selects using masks for efficient vectorization.
/// The branches are mutually exclusive when `dt < 0.5` (valid for all practical
/// oscillator frequencies below Nyquist/2).
#[cfg(feature = "simd")]
#[inline]
pub fn poly_blep_simd<S: Sample>(t: S::Simd, dt: S::Simd) -> S::Simd {
    let zero = S::simd_splat(S::ZERO);
    let one = S::simd_splat(S::ONE);
    let two = S::simd_splat(S::from_f64(2.0));
    let one_minus_dt = one - dt;

    // Branch 1: t < dt (just after discontinuity)
    let t_norm_after = t / dt;
    let result_after = two * t_norm_after - t_norm_after * t_norm_after - one;

    // Branch 2: t > 1-dt (just before discontinuity)
    let t_norm_before = (t - one) / dt;
    let result_before = t_norm_before * t_norm_before + two * t_norm_before + one;

    // Select with masks: after_or_zero where t < dt, then override with before where t > 1-dt
    let after_or_zero = S::simd_select_lt(t, dt, result_after, zero);
    S::simd_select_gt(t, one_minus_dt, result_before, after_or_zero)
}

/// Branchless SIMD PolyBLAMP correction for 4 phase values.
///
/// Used for slope discontinuities (triangle waves). Computes all branches
/// and selects using masks for efficient vectorization.
#[cfg(feature = "simd")]
#[inline]
pub fn poly_blamp_simd<S: Sample>(t: S::Simd, dt: S::Simd) -> S::Simd {
    let zero = S::simd_splat(S::ZERO);
    let one = S::simd_splat(S::ONE);
    let third = S::simd_splat(S::from_f64(1.0 / 3.0));
    let one_minus_dt = one - dt;

    // Branch 1: t < dt
    let t_norm_after = t / dt;
    let t2_after = t_norm_after * t_norm_after;
    let t3_after = t2_after * t_norm_after;
    let result_after = (t2_after - t3_after * third - t_norm_after) * dt;

    // Branch 2: t > 1-dt
    let t_norm_before = (t - one) / dt;
    let t2_before = t_norm_before * t_norm_before;
    let t3_before = t2_before * t_norm_before;
    let result_before = (t3_before * third + t2_before + t_norm_before) * dt;

    let after_or_zero = S::simd_select_lt(t, dt, result_after, zero);
    S::simd_select_gt(t, one_minus_dt, result_before, after_or_zero)
}

/// Generate a band-limited sawtooth sample using PolyBLEP.
///
/// # Arguments
/// * `phase` - Current normalized phase (0.0 to 1.0)
/// * `phase_inc` - Phase increment per sample (normalized)
#[inline]
pub fn polyblep_saw<S: Sample>(phase: S, phase_inc: S) -> S {
    let two = S::from_f64(2.0);
    let naive = two * phase - S::ONE;
    naive - poly_blep(phase, phase_inc)
}

/// Generate a band-limited square wave sample using PolyBLEP.
///
/// # Arguments
/// * `phase` - Current normalized phase (0.0 to 1.0)
/// * `phase_inc` - Phase increment per sample (normalized)
#[inline]
pub fn polyblep_square<S: Sample>(phase: S, phase_inc: S) -> S {
    let half = S::from_f64(0.5);
    let naive = if phase < half { S::ONE } else { -S::ONE };
    let mut out = naive;
    out += poly_blep(phase, phase_inc);
    let falling_phase = fract(phase + half);
    out -= poly_blep(falling_phase, phase_inc);
    out
}

/// Generate a band-limited pulse wave sample using PolyBLEP.
///
/// # Arguments
/// * `phase` - Current normalized phase (0.0 to 1.0)
/// * `phase_inc` - Phase increment per sample (normalized)
/// * `duty_cycle` - Duty cycle (0.0 to 1.0)
#[inline]
pub fn polyblep_pulse<S: Sample>(phase: S, phase_inc: S, duty_cycle: S) -> S {
    let naive = if phase < duty_cycle { S::ONE } else { -S::ONE };
    let mut out = naive;
    out += poly_blep(phase, phase_inc);
    let falling_phase = fract(phase - duty_cycle + S::ONE);
    out -= poly_blep(falling_phase, phase_inc);
    out
}

/// Generate a band-limited triangle wave sample using PolyBLAMP.
///
/// # Arguments
/// * `phase` - Current normalized phase (0.0 to 1.0)
/// * `phase_inc` - Phase increment per sample (normalized)
#[inline]
pub fn polyblamp_triangle<S: Sample>(phase: S, phase_inc: S) -> S {
    let half = S::from_f64(0.5);
    let four = S::from_f64(4.0);
    let three = S::from_f64(3.0);
    let eight = S::from_f64(8.0);

    let naive = if phase < half {
        four * phase - S::ONE
    } else {
        three - four * phase
    };

    let mut out = naive;
    out += eight * poly_blamp(phase, phase_inc);
    out -= eight * poly_blamp(fract(phase + half), phase_inc);
    out
}

/// Apply PolyBLEP corrections to a SIMD chunk of sawtooth samples.
///
/// Uses branchless SIMD for efficient vectorized correction.
#[cfg(feature = "simd")]
pub fn apply_polyblep_saw<S: Sample>(samples: &mut [S; SIMD_LANES], phases: [S; SIMD_LANES], phase_inc: S) {
    let samples_simd = S::simd_from_slice(samples);
    let phases_simd = S::simd_from_slice(&phases);
    let phase_inc_simd = S::simd_splat(phase_inc);

    let correction = poly_blep_simd::<S>(phases_simd, phase_inc_simd);
    *samples = S::simd_to_array(samples_simd - correction);
}

/// Apply PolyBLEP corrections to a SIMD chunk of square samples.
///
/// Uses branchless SIMD for both rising (phase=0) and falling (phase=0.5) edges.
#[cfg(feature = "simd")]
pub fn apply_polyblep_square<S: Sample>(samples: &mut [S; SIMD_LANES], phases: [S; SIMD_LANES], phase_inc: S) {
    let samples_simd = S::simd_from_slice(samples);
    let phases_simd = S::simd_from_slice(&phases);
    let phase_inc_simd = S::simd_splat(phase_inc);
    let half = S::simd_splat(S::from_f64(0.5));

    // Rising edge correction at phase = 0
    let rising = poly_blep_simd::<S>(phases_simd, phase_inc_simd);

    // Falling edge correction at phase = 0.5 (SIMD fract)
    let falling_phase_raw = phases_simd + half;
    let falling_phase = falling_phase_raw - falling_phase_raw.floor();
    let falling = poly_blep_simd::<S>(falling_phase, phase_inc_simd);

    *samples = S::simd_to_array(samples_simd + rising - falling);
}

/// Apply PolyBLEP corrections to a SIMD chunk of pulse samples.
///
/// Uses branchless SIMD for both rising (phase=0) and falling (phase=duty_cycle) edges.
#[cfg(feature = "simd")]
pub fn apply_polyblep_pulse<S: Sample>(
    samples: &mut [S; SIMD_LANES],
    phases: [S; SIMD_LANES],
    phase_inc: S,
    duty_cycle: S,
) {
    let samples_simd = S::simd_from_slice(samples);
    let phases_simd = S::simd_from_slice(&phases);
    let phase_inc_simd = S::simd_splat(phase_inc);
    let duty = S::simd_splat(duty_cycle);
    let one = S::simd_splat(S::ONE);

    // Rising edge correction at phase = 0
    let rising = poly_blep_simd::<S>(phases_simd, phase_inc_simd);

    // Falling edge correction at phase = duty_cycle (SIMD fract)
    let falling_phase_raw = phases_simd - duty + one;
    let falling_phase = falling_phase_raw - falling_phase_raw.floor();
    let falling = poly_blep_simd::<S>(falling_phase, phase_inc_simd);

    *samples = S::simd_to_array(samples_simd + rising - falling);
}

/// Apply PolyBLAMP corrections to a SIMD chunk of triangle samples.
///
/// Uses branchless SIMD for slope changes at phase=0 and phase=0.5.
#[cfg(feature = "simd")]
pub fn apply_polyblamp_triangle<S: Sample>(samples: &mut [S; SIMD_LANES], phases: [S; SIMD_LANES], phase_inc: S) {
    let samples_simd = S::simd_from_slice(samples);
    let phases_simd = S::simd_from_slice(&phases);
    let phase_inc_simd = S::simd_splat(phase_inc);
    let half = S::simd_splat(S::from_f64(0.5));
    let eight = S::simd_splat(S::from_f64(8.0));

    // Slope change at phase = 0
    let at_zero = poly_blamp_simd::<S>(phases_simd, phase_inc_simd);

    // Slope change at phase = 0.5 (SIMD fract)
    let at_half_phase_raw = phases_simd + half;
    let at_half_phase = at_half_phase_raw - at_half_phase_raw.floor();
    let at_half = poly_blamp_simd::<S>(at_half_phase, phase_inc_simd);

    *samples = S::simd_to_array(samples_simd + eight * at_zero - eight * at_half);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_poly_blep_zero_outside_range() {
        let dt = 0.1f64;
        assert_eq!(poly_blep(0.5, dt), 0.0);
        assert_eq!(poly_blep(0.3, dt), 0.0);
        assert_eq!(poly_blep(0.7, dt), 0.0);
    }

    #[test]
    fn test_poly_blep_near_discontinuity() {
        let dt = 0.1f64;

        // Just after discontinuity (phase near 0)
        let correction_after = poly_blep(0.05, dt);
        assert!(
            correction_after != 0.0,
            "Should have correction just after discontinuity"
        );
        assert!(correction_after < 0.0, "Correction should be negative just after");

        // Just before discontinuity (phase near 1)
        let correction_before = poly_blep(0.95, dt);
        assert!(
            correction_before != 0.0,
            "Should have correction just before discontinuity"
        );
        assert!(correction_before > 0.0, "Correction should be positive just before");
    }

    #[test]
    fn test_poly_blep_boundary_values() {
        let dt = 0.1f64;

        // At t=0 (just after discontinuity), correction should be -1
        let at_zero = poly_blep(0.0, dt);
        assert!(
            (at_zero - (-1.0)).abs() < 1e-10,
            "poly_blep(0, dt) should be -1, got {}",
            at_zero
        );

        // At t approaching dt, correction should approach 0
        let near_dt = poly_blep(dt - 0.001, dt);
        assert!(near_dt.abs() < 0.1, "Should approach 0 near dt boundary");
    }

    #[test]
    fn test_polyblep_saw_matches_reference() {
        // Reference implementation test case:
        // At phase=0.005, phase_inc=0.01:
        // naive = 2*0.005 - 1 = -0.99
        // poly_blep: t < dt, t_norm = 0.5, correction = 2*0.5 - 0.25 - 1 = -0.25
        // result = -0.99 - (-0.25) = -0.74
        let phase = 0.005f64;
        let phase_inc = 0.01f64;
        let result = polyblep_saw(phase, phase_inc);
        let expected = -0.74f64;
        assert!(
            (result - expected).abs() < 0.01,
            "polyblep_saw({}, {}) = {}, expected ~{}",
            phase,
            phase_inc,
            result,
            expected
        );
    }

    #[test]
    fn test_polyblep_saw_no_correction_mid_phase() {
        // Far from discontinuity, should be close to naive
        let phase = 0.5f64;
        let phase_inc = 0.01f64;
        let result = polyblep_saw(phase, phase_inc);
        let naive = 2.0 * phase - 1.0; // 0.0
        assert!(
            (result - naive).abs() < 1e-10,
            "Should equal naive far from discontinuity"
        );
    }

    #[test]
    fn test_polyblep_square_at_edges() {
        let phase_inc = 0.01f64;

        // Just after rising edge at phase = 0
        let at_rising = polyblep_square(0.005, phase_inc);
        // Should be close to 1.0 but with some correction
        assert!(at_rising > 0.5 && at_rising < 1.5, "Rising edge should be positive");

        // Just after falling edge at phase = 0.5
        let at_falling = polyblep_square(0.505, phase_inc);
        // Should be close to -1.0 but with some correction
        assert!(
            at_falling < -0.5 && at_falling > -1.5,
            "Falling edge should be negative"
        );

        // Middle of high section (no edges nearby)
        let middle_high = polyblep_square(0.25, phase_inc);
        assert!(
            (middle_high - 1.0).abs() < 1e-10,
            "Should be 1.0 in middle of high section"
        );

        // Middle of low section (no edges nearby)
        let middle_low = polyblep_square(0.75, phase_inc);
        assert!(
            (middle_low - (-1.0)).abs() < 1e-10,
            "Should be -1.0 in middle of low section"
        );
    }

    #[test]
    fn test_polyblep_pulse_variable_duty() {
        let phase_inc = 0.01f64;
        let duty_cycle = 0.25f64;

        // Middle of high section (phase < duty_cycle, away from edges)
        let high = polyblep_pulse(0.125, phase_inc, duty_cycle);
        assert!((high - 1.0).abs() < 1e-10, "Should be 1.0 in high section");

        // Middle of low section (phase > duty_cycle, away from edges)
        let low = polyblep_pulse(0.6, phase_inc, duty_cycle);
        assert!((low - (-1.0)).abs() < 1e-10, "Should be -1.0 in low section");
    }

    #[test]
    fn test_polyblamp_triangle() {
        let phase_inc = 0.01f64;

        // Middle of rising section
        let rising = polyblamp_triangle(0.25, phase_inc);
        // Naive: 4*0.25 - 1 = 0.0, correction should be minimal
        assert!(rising.abs() < 0.1, "Should be near 0 at phase 0.25");

        // Middle of falling section
        let falling = polyblamp_triangle(0.75, phase_inc);
        // Naive: 3 - 4*0.75 = 0.0, correction should be minimal
        assert!(falling.abs() < 0.1, "Should be near 0 at phase 0.75");
    }

    #[test]
    fn test_generic_f32() {
        // Verify functions work with f32
        // Use phase=0.75 to be in middle of section, away from discontinuities
        let phase = 0.75f32;
        let phase_inc = 0.01f32;

        let saw = polyblep_saw(phase, phase_inc);
        let square = polyblep_square(phase, phase_inc);
        let triangle = polyblamp_triangle(phase, phase_inc);

        // saw at 0.75: naive = 2*0.75 - 1 = 0.5
        assert!((saw - 0.5f32).abs() < 1e-6);
        // square at 0.75: in low section, away from edges
        assert!((square - (-1.0f32)).abs() < 1e-6);
        // triangle at 0.75: naive = 3 - 4*0.75 = 0
        assert!((triangle - 0.0f32).abs() < 0.1);
    }

    #[test]
    fn test_generic_f64() {
        // Verify functions work with f64
        // Use phase=0.75 to be in middle of section, away from discontinuities
        let phase = 0.75f64;
        let phase_inc = 0.01f64;

        let saw = polyblep_saw(phase, phase_inc);
        let square = polyblep_square(phase, phase_inc);
        let triangle = polyblamp_triangle(phase, phase_inc);

        // saw at 0.75: naive = 2*0.75 - 1 = 0.5
        assert!((saw - 0.5f64).abs() < 1e-10);
        // square at 0.75: in low section, away from edges
        assert!((square - (-1.0f64)).abs() < 1e-10);
        // triangle at 0.75: naive = 3 - 4*0.75 = 0
        assert!((triangle - 0.0f64).abs() < 0.1);
    }

    #[cfg(feature = "simd")]
    mod simd_tests {
        use super::*;
        use crate::sample::Sample;

        fn compare_scalar_simd_blep<S: Sample>(phases: [S; SIMD_LANES], phase_inc: S, tolerance: S) {
            let phases_simd = S::simd_from_slice(&phases);
            let phase_inc_simd = S::simd_splat(phase_inc);

            let simd_result = S::simd_to_array(poly_blep_simd::<S>(phases_simd, phase_inc_simd));

            for i in 0..SIMD_LANES {
                let scalar_result = poly_blep(phases[i], phase_inc);
                let diff = if simd_result[i] > scalar_result {
                    simd_result[i] - scalar_result
                } else {
                    scalar_result - simd_result[i]
                };
                assert!(
                    diff < tolerance,
                    "SIMD vs scalar mismatch at lane {}: simd={:?}, scalar={:?}, diff={:?}",
                    i,
                    simd_result[i],
                    scalar_result,
                    diff
                );
            }
        }

        fn compare_scalar_simd_blamp<S: Sample>(phases: [S; SIMD_LANES], phase_inc: S, tolerance: S) {
            let phases_simd = S::simd_from_slice(&phases);
            let phase_inc_simd = S::simd_splat(phase_inc);

            let simd_result = S::simd_to_array(poly_blamp_simd::<S>(phases_simd, phase_inc_simd));

            for i in 0..SIMD_LANES {
                let scalar_result = poly_blamp(phases[i], phase_inc);
                let diff = if simd_result[i] > scalar_result {
                    simd_result[i] - scalar_result
                } else {
                    scalar_result - simd_result[i]
                };
                assert!(
                    diff < tolerance,
                    "SIMD vs scalar mismatch at lane {}: simd={:?}, scalar={:?}, diff={:?}",
                    i,
                    simd_result[i],
                    scalar_result,
                    diff
                );
            }
        }

        #[test]
        fn test_poly_blep_simd_matches_scalar_f64() {
            let phase_inc = 0.01f64;
            let tolerance = 1e-10f64;

            // Test various phase combinations including near discontinuities
            let test_cases: [[f64; SIMD_LANES]; 5] = [
                [0.25, 0.5, 0.75, 0.9],       // Middle phases (no correction)
                [0.005, 0.5, 0.75, 0.995],    // Near discontinuities
                [0.0, 0.01, 0.99, 1.0],       // At boundaries
                [0.001, 0.002, 0.003, 0.004], // All near start
                [0.996, 0.997, 0.998, 0.999], // All near end
            ];

            for phases in test_cases {
                compare_scalar_simd_blep(phases, phase_inc, tolerance);
            }
        }

        #[test]
        fn test_poly_blep_simd_matches_scalar_f32() {
            let phase_inc = 0.01f32;
            let tolerance = 1e-5f32;

            let test_cases: [[f32; SIMD_LANES]; 5] = [
                [0.25, 0.5, 0.75, 0.9],
                [0.005, 0.5, 0.75, 0.995],
                [0.0, 0.01, 0.99, 1.0],
                [0.001, 0.002, 0.003, 0.004],
                [0.996, 0.997, 0.998, 0.999],
            ];

            for phases in test_cases {
                compare_scalar_simd_blep(phases, phase_inc, tolerance);
            }
        }

        #[test]
        fn test_poly_blamp_simd_matches_scalar_f64() {
            let phase_inc = 0.01f64;
            let tolerance = 1e-10f64;

            let test_cases: [[f64; SIMD_LANES]; 5] = [
                [0.25, 0.5, 0.75, 0.9],
                [0.005, 0.5, 0.75, 0.995],
                [0.0, 0.01, 0.99, 1.0],
                [0.001, 0.002, 0.003, 0.004],
                [0.996, 0.997, 0.998, 0.999],
            ];

            for phases in test_cases {
                compare_scalar_simd_blamp(phases, phase_inc, tolerance);
            }
        }

        #[test]
        fn test_poly_blamp_simd_matches_scalar_f32() {
            let phase_inc = 0.01f32;
            let tolerance = 1e-5f32;

            let test_cases: [[f32; SIMD_LANES]; 5] = [
                [0.25, 0.5, 0.75, 0.9],
                [0.005, 0.5, 0.75, 0.995],
                [0.0, 0.01, 0.99, 1.0],
                [0.001, 0.002, 0.003, 0.004],
                [0.996, 0.997, 0.998, 0.999],
            ];

            for phases in test_cases {
                compare_scalar_simd_blamp(phases, phase_inc, tolerance);
            }
        }

        #[test]
        fn test_apply_polyblep_saw_simd_correctness() {
            let phase_inc = 0.01f64;
            let phases: [f64; SIMD_LANES] = [0.005, 0.25, 0.75, 0.995];

            // Compute expected via scalar
            let mut expected = [0.0f64; SIMD_LANES];
            for i in 0..SIMD_LANES {
                let naive = 2.0 * phases[i] - 1.0;
                expected[i] = naive - poly_blep(phases[i], phase_inc);
            }

            // Generate naive samples and apply SIMD correction
            let mut samples: [f64; SIMD_LANES] = [
                2.0 * phases[0] - 1.0,
                2.0 * phases[1] - 1.0,
                2.0 * phases[2] - 1.0,
                2.0 * phases[3] - 1.0,
            ];
            apply_polyblep_saw(&mut samples, phases, phase_inc);

            for i in 0..SIMD_LANES {
                let diff = (samples[i] - expected[i]).abs();
                assert!(
                    diff < 1e-10,
                    "Saw mismatch at lane {}: got {:?}, expected {:?}",
                    i,
                    samples[i],
                    expected[i]
                );
            }
        }

        #[test]
        fn test_apply_polyblep_square_simd_correctness() {
            let phase_inc = 0.01f64;
            let phases: [f64; SIMD_LANES] = [0.005, 0.25, 0.505, 0.75];

            // Compute expected via scalar
            let mut expected = [0.0f64; SIMD_LANES];
            for i in 0..SIMD_LANES {
                expected[i] = polyblep_square(phases[i], phase_inc);
            }

            // Generate naive samples and apply SIMD correction
            let mut samples: [f64; SIMD_LANES] = [
                if phases[0] < 0.5 { 1.0 } else { -1.0 },
                if phases[1] < 0.5 { 1.0 } else { -1.0 },
                if phases[2] < 0.5 { 1.0 } else { -1.0 },
                if phases[3] < 0.5 { 1.0 } else { -1.0 },
            ];
            apply_polyblep_square(&mut samples, phases, phase_inc);

            for i in 0..SIMD_LANES {
                let diff = (samples[i] - expected[i]).abs();
                assert!(
                    diff < 1e-10,
                    "Square mismatch at lane {}: got {:?}, expected {:?}",
                    i,
                    samples[i],
                    expected[i]
                );
            }
        }

        #[test]
        fn test_apply_polyblamp_triangle_simd_correctness() {
            let phase_inc = 0.01f64;
            let phases: [f64; SIMD_LANES] = [0.005, 0.25, 0.505, 0.75];

            // Compute expected via scalar
            let mut expected = [0.0f64; SIMD_LANES];
            for i in 0..SIMD_LANES {
                expected[i] = polyblamp_triangle(phases[i], phase_inc);
            }

            // Generate naive samples and apply SIMD correction
            let mut samples: [f64; SIMD_LANES] = [
                if phases[0] < 0.5 {
                    4.0 * phases[0] - 1.0
                } else {
                    3.0 - 4.0 * phases[0]
                },
                if phases[1] < 0.5 {
                    4.0 * phases[1] - 1.0
                } else {
                    3.0 - 4.0 * phases[1]
                },
                if phases[2] < 0.5 {
                    4.0 * phases[2] - 1.0
                } else {
                    3.0 - 4.0 * phases[2]
                },
                if phases[3] < 0.5 {
                    4.0 * phases[3] - 1.0
                } else {
                    3.0 - 4.0 * phases[3]
                },
            ];
            apply_polyblamp_triangle(&mut samples, phases, phase_inc);

            for i in 0..SIMD_LANES {
                let diff = (samples[i] - expected[i]).abs();
                assert!(
                    diff < 1e-10,
                    "Triangle mismatch at lane {}: got {:?}, expected {:?}",
                    i,
                    samples[i],
                    expected[i]
                );
            }
        }
    }
}
