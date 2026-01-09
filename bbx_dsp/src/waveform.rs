//! Waveform types and generation.
//!
//! This module defines standard waveform shapes used by oscillators and LFOs.

#[cfg(feature = "simd")]
use std::simd::StdFloat;

use bbx_core::random::XorShiftRng;

#[cfg(feature = "simd")]
use crate::sample::SIMD_LANES;
use crate::sample::Sample;

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
#[cfg(feature = "simd")]
pub(crate) fn generate_waveform_samples_simd_generic<S: Sample>(
    waveform: Waveform,
    phases: S::Simd,
    duty_cycle: S,
) -> Option<[S; SIMD_LANES]> {
    let two_pi = S::simd_splat(S::from_f64(TWO_PI));
    let inv_two_pi = S::simd_splat(S::from_f64(INV_TWO_PI));

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
