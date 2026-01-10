//! Waveform types and generation.
//!
//! This module defines standard waveform shapes used by oscillators and LFOs.

#[cfg(feature = "simd")]
use std::simd::StdFloat;

use bbx_core::random::XorShiftRng;

#[cfg(feature = "simd")]
use crate::polyblep::{apply_polyblamp_triangle, apply_polyblep_pulse, apply_polyblep_saw, apply_polyblep_square};
#[cfg(feature = "simd")]
use crate::sample::SIMD_LANES;
use crate::{
    polyblep::{polyblamp_triangle, polyblep_pulse, polyblep_saw, polyblep_square},
    sample::Sample,
};

/// Standard waveform shapes for oscillators and LFOs.
#[derive(Debug, Clone, Copy)]
pub enum Waveform {
    /// Pure tone with no harmonics.
    Sine,
    /// Rich in odd harmonics, bright and buzzy.
    Square,
    /// Contains all harmonics, bright and cutting.
    Sawtooth,
    /// Soft, flute-like tone with odd harmonics.
    Triangle,
    /// Variable duty cycle square wave.
    Pulse,
    /// Random values for each sample.
    Noise,
}

/// Default value for the duty cycle or the inflection point
/// at which the values of a waveform change sign. This effectively
/// determines the width of the waveform within its periodic cycle.
pub(crate) const DEFAULT_DUTY_CYCLE: f64 = 0.5;

const TWO_PI: f64 = 2.0 * std::f64::consts::PI;
const INV_TWO_PI: f64 = 1.0 / TWO_PI;

/// Generate a sample of a particular waveform, given its position (phase) and duty cycle.
pub(crate) fn generate_waveform_sample(waveform: Waveform, phase: f64, duty_cycle: f64, rng: &mut XorShiftRng) -> f64 {
    match waveform {
        Waveform::Sine => phase.sin(),
        Waveform::Square => {
            if phase.sin() > 0.0 {
                1.0
            } else {
                -1.0
            }
        }
        Waveform::Sawtooth => {
            let normalized_phase = (phase % TWO_PI) * INV_TWO_PI;
            2.0 * normalized_phase - 1.0
        }
        Waveform::Triangle => {
            let normalized_phase = (phase % TWO_PI) * INV_TWO_PI;
            if normalized_phase < 0.5 {
                4.0 * normalized_phase - 1.0
            } else {
                3.0 - 4.0 * normalized_phase
            }
        }
        Waveform::Pulse => {
            let normalized_phase = (phase % TWO_PI) * INV_TWO_PI;
            if normalized_phase < duty_cycle { 1.0 } else { -1.0 }
        }
        Waveform::Noise => rng.next_noise_sample(),
    }
}

/// Process waveform samples using scalar (non-SIMD) operations.
///
/// Writes samples to `output`, advances `phase` by `phase_increment` per sample,
/// and normalizes phase at the end.
pub(crate) fn process_waveform_scalar<S: Sample>(
    output: &mut [S],
    waveform: Waveform,
    phase: &mut f64,
    phase_increment: f64,
    rng: &mut XorShiftRng,
    scale: f64,
) {
    for sample in output.iter_mut() {
        let value = generate_waveform_sample(waveform, *phase, DEFAULT_DUTY_CYCLE, rng);
        *sample = S::from_f64(value * scale);
        *phase += phase_increment;
    }
    *phase = phase.rem_euclid(TWO_PI);
}

/// Generate 4 samples of a waveform at consecutive phases using SIMD (generic version).
///
/// This version works with any `Sample` type, using f32x4 for f32 and f64x4 for f64.
/// Returns `None` for Noise waveform (requires sequential RNG).
///
/// The `two_pi` and `inv_two_pi` parameters should be pre-computed once per process()
/// call to avoid redundant SIMD splat operations in the hot loop.
#[cfg(feature = "simd")]
pub(crate) fn generate_waveform_samples_simd_generic<S: Sample>(
    waveform: Waveform,
    phases: S::Simd,
    duty_cycle: S,
    two_pi: S::Simd,
    inv_two_pi: S::Simd,
) -> Option<[S; SIMD_LANES]> {
    match waveform {
        Waveform::Sine => Some(S::simd_to_array(phases.sin())),

        Waveform::Square => {
            let sin_phases = phases.sin();
            let zero = S::simd_splat(S::ZERO);
            let one = S::simd_splat(S::ONE);
            let neg_one = S::simd_splat(-S::ONE);
            Some(S::simd_to_array(S::simd_select_gt(sin_phases, zero, one, neg_one)))
        }

        Waveform::Sawtooth => {
            let two = S::simd_splat(S::from_f64(2.0));
            let one = S::simd_splat(S::ONE);
            let normalized = (phases % two_pi) * inv_two_pi;
            Some(S::simd_to_array(two * normalized - one))
        }

        Waveform::Triangle => {
            let half = S::simd_splat(S::from_f64(0.5));
            let four = S::simd_splat(S::from_f64(4.0));
            let one = S::simd_splat(S::ONE);
            let three = S::simd_splat(S::from_f64(3.0));

            let normalized = (phases % two_pi) * inv_two_pi;
            let rising = four * normalized - one;
            let falling = three - four * normalized;
            Some(S::simd_to_array(S::simd_select_lt(normalized, half, rising, falling)))
        }

        Waveform::Pulse => {
            let duty = S::simd_splat(duty_cycle);
            let one = S::simd_splat(S::ONE);
            let neg_one = S::simd_splat(-S::ONE);

            let normalized = (phases % two_pi) * inv_two_pi;
            Some(S::simd_to_array(S::simd_select_lt(normalized, duty, one, neg_one)))
        }

        Waveform::Noise => None,
    }
}

/// Generate an anti-aliased sample of a waveform using PolyBLEP/PolyBLAMP.
///
/// This function generates band-limited waveforms using polynomial corrections
/// near discontinuities. For sine and noise waveforms, no correction is needed.
///
/// # Arguments
/// * `waveform` - The waveform type to generate
/// * `phase` - Current phase in radians (0 to 2π)
/// * `phase_increment` - Phase increment per sample in radians
/// * `duty_cycle` - Duty cycle for pulse waveform (0.0 to 1.0)
/// * `rng` - Random number generator for noise waveform
pub(crate) fn generate_waveform_sample_antialiased(
    waveform: Waveform,
    phase: f64,
    phase_increment: f64,
    duty_cycle: f64,
    rng: &mut XorShiftRng,
) -> f64 {
    let normalized_phase = (phase % TWO_PI) * INV_TWO_PI;
    let normalized_inc = phase_increment * INV_TWO_PI;

    match waveform {
        Waveform::Sine => phase.sin(),
        Waveform::Sawtooth => polyblep_saw(normalized_phase, normalized_inc),
        Waveform::Square => polyblep_square(normalized_phase, normalized_inc),
        Waveform::Pulse => polyblep_pulse(normalized_phase, normalized_inc, duty_cycle),
        Waveform::Triangle => polyblamp_triangle(normalized_phase, normalized_inc),
        Waveform::Noise => rng.next_noise_sample(),
    }
}

/// Process waveform samples with anti-aliasing using scalar operations.
///
/// Writes samples to `output`, advances `phase` by `phase_increment` per sample,
/// and applies PolyBLEP/PolyBLAMP corrections for band-limited output.
pub(crate) fn process_waveform_scalar_antialiased<S: Sample>(
    output: &mut [S],
    waveform: Waveform,
    phase: &mut f64,
    phase_increment: f64,
    rng: &mut XorShiftRng,
    scale: f64,
) {
    for sample in output.iter_mut() {
        let value = generate_waveform_sample_antialiased(waveform, *phase, phase_increment, DEFAULT_DUTY_CYCLE, rng);
        *sample = S::from_f64(value * scale);
        *phase += phase_increment;
    }
    *phase = phase.rem_euclid(TWO_PI);
}

/// Generate 4 anti-aliased samples of a waveform using SIMD with PolyBLEP corrections.
///
/// This function first generates naive samples using fast SIMD operations, then applies
/// scalar PolyBLEP/PolyBLAMP corrections near discontinuities.
///
/// # Arguments
/// * `waveform` - The waveform type to generate
/// * `phases` - SIMD vector of 4 phases (in radians)
/// * `phases_normalized` - Array of 4 normalized phases (0-1) for polyblep calculation
/// * `phase_inc_normalized` - Phase increment per sample (normalized 0-1)
/// * `duty_cycle` - Duty cycle for pulse waveform
/// * `two_pi` - Pre-computed SIMD 2π constant
/// * `inv_two_pi` - Pre-computed SIMD 1/(2π) constant
#[cfg(feature = "simd")]
pub(crate) fn generate_waveform_samples_simd_antialiased<S: Sample>(
    waveform: Waveform,
    phases: S::Simd,
    phases_normalized: [S; SIMD_LANES],
    phase_inc_normalized: S,
    duty_cycle: S,
    two_pi: S::Simd,
    inv_two_pi: S::Simd,
) -> Option<[S; SIMD_LANES]> {
    // First generate naive samples using fast SIMD
    let mut samples = generate_waveform_samples_simd_generic(waveform, phases, duty_cycle, two_pi, inv_two_pi)?;

    // Apply scalar corrections near discontinuities
    match waveform {
        Waveform::Sine | Waveform::Noise => {}
        Waveform::Sawtooth => {
            apply_polyblep_saw(&mut samples, phases_normalized, phase_inc_normalized);
        }
        Waveform::Square => {
            apply_polyblep_square(&mut samples, phases_normalized, phase_inc_normalized);
        }
        Waveform::Pulse => {
            apply_polyblep_pulse(&mut samples, phases_normalized, phase_inc_normalized, duty_cycle);
        }
        Waveform::Triangle => {
            apply_polyblamp_triangle(&mut samples, phases_normalized, phase_inc_normalized);
        }
    }

    Some(samples)
}
