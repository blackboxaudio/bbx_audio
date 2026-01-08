//! Waveform types and generation.
//!
//! This module defines standard waveform shapes used by oscillators and LFOs.

#[cfg(feature = "simd")]
use std::simd::{StdFloat, cmp::SimdPartialOrd, f64x4};

use bbx_core::random::XorShiftRng;

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

/// Generate 4 samples of a waveform at consecutive phases using SIMD.
///
/// Returns `None` for Noise waveform (requires sequential RNG).
/// For all other waveforms, returns the 4 samples as an array.
#[cfg(feature = "simd")]
pub(crate) fn generate_waveform_samples_simd(waveform: Waveform, phases: f64x4, duty_cycle: f64) -> Option<[f64; 4]> {
    let two_pi = f64x4::splat(TWO_PI);
    let inv_two_pi = f64x4::splat(INV_TWO_PI);

    match waveform {
        Waveform::Sine => Some(phases.sin().to_array()),

        Waveform::Square => {
            let sin_phases = phases.sin();
            let zero = f64x4::splat(0.0);
            let one = f64x4::splat(1.0);
            let neg_one = f64x4::splat(-1.0);
            let mask = sin_phases.simd_gt(zero);
            Some(mask.select(one, neg_one).to_array())
        }

        Waveform::Sawtooth => {
            let two = f64x4::splat(2.0);
            let one = f64x4::splat(1.0);
            let normalized = (phases % two_pi) * inv_two_pi;
            Some((two * normalized - one).to_array())
        }

        Waveform::Triangle => {
            let half = f64x4::splat(0.5);
            let four = f64x4::splat(4.0);
            let one = f64x4::splat(1.0);
            let three = f64x4::splat(3.0);

            let normalized = (phases % two_pi) * inv_two_pi;
            let mask = normalized.simd_lt(half);
            let rising = four * normalized - one;
            let falling = three - four * normalized;
            Some(mask.select(rising, falling).to_array())
        }

        Waveform::Pulse => {
            let duty = f64x4::splat(duty_cycle);
            let one = f64x4::splat(1.0);
            let neg_one = f64x4::splat(-1.0);

            let normalized = (phases % two_pi) * inv_two_pi;
            let mask = normalized.simd_lt(duty);
            Some(mask.select(one, neg_one).to_array())
        }

        Waveform::Noise => None,
    }
}
