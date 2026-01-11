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

/// Generate 4 naive samples of a waveform using SIMD (internal helper).
///
/// Returns `None` for Noise waveform (requires sequential RNG).
#[cfg(feature = "simd")]
fn generate_naive_samples_simd<S: Sample>(
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

/// Generate a band-limited waveform sample using PolyBLEP/PolyBLAMP.
///
/// Uses polynomial corrections near discontinuities to reduce aliasing.
/// Sine and noise waveforms pass through without correction.
pub(crate) fn generate_waveform_sample(
    waveform: Waveform,
    phase: f64,
    phase_increment: f64,
    duty_cycle: f64,
    rng: &mut XorShiftRng,
) -> f64 {
    let normalized_phase = (phase % <f64 as Sample>::TAU) * <f64 as Sample>::INV_TAU;
    let normalized_inc = phase_increment * <f64 as Sample>::INV_TAU;

    match waveform {
        Waveform::Sine => phase.sin(),
        Waveform::Sawtooth => polyblep_saw(normalized_phase, normalized_inc),
        Waveform::Square => polyblep_square(normalized_phase, normalized_inc),
        Waveform::Pulse => polyblep_pulse(normalized_phase, normalized_inc, duty_cycle),
        Waveform::Triangle => polyblamp_triangle(normalized_phase, normalized_inc),
        Waveform::Noise => rng.next_noise_sample(),
    }
}

/// Process waveform samples using scalar operations with band-limiting.
///
/// Writes band-limited samples to `output`, advances `phase` by `phase_increment`
/// per sample, and applies PolyBLEP/PolyBLAMP corrections.
pub(crate) fn process_waveform_scalar<S: Sample>(
    output: &mut [S],
    waveform: Waveform,
    phase: &mut f64,
    phase_increment: f64,
    rng: &mut XorShiftRng,
    scale: f64,
) {
    for sample in output.iter_mut() {
        let value = generate_waveform_sample(waveform, *phase, phase_increment, DEFAULT_DUTY_CYCLE, rng);
        *sample = S::from_f64(value * scale);
        *phase += phase_increment;
    }
    *phase = phase.rem_euclid(<f64 as Sample>::TAU);
}

/// Generate 4 band-limited waveform samples using SIMD with PolyBLEP corrections.
///
/// Generates naive samples via SIMD, then applies PolyBLEP/PolyBLAMP corrections.
#[cfg(feature = "simd")]
pub(crate) fn generate_waveform_samples_simd<S: Sample>(
    waveform: Waveform,
    phases: S::Simd,
    phases_normalized: [S; SIMD_LANES],
    phase_inc_normalized: S,
    duty_cycle: S,
    two_pi: S::Simd,
    inv_two_pi: S::Simd,
) -> Option<[S; SIMD_LANES]> {
    let mut samples = generate_naive_samples_simd(waveform, phases, duty_cycle, two_pi, inv_two_pi)?;

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
