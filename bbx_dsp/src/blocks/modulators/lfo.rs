//! Low-frequency oscillator (LFO) block for parameter modulation.

use bbx_core::random::XorShiftRng;

#[cfg(feature = "simd")]
use crate::sample::SIMD_LANES;
#[cfg(feature = "simd")]
use crate::waveform::generate_waveform_samples_simd_generic;
use crate::{
    block::{Block, DEFAULT_MODULATOR_INPUT_COUNT, DEFAULT_MODULATOR_OUTPUT_COUNT},
    context::DspContext,
    parameter::{ModulationOutput, Parameter},
    sample::Sample,
    waveform::{Waveform, process_waveform_scalar},
};

/// A low-frequency oscillator for modulating block parameters.
///
/// Generates control signals (typically < 20 Hz) using standard waveforms.
/// Output range is -depth to +depth, centered at zero.
pub struct LfoBlock<S: Sample> {
    /// LFO frequency in Hz (typically 0.01-20 Hz).
    pub frequency: Parameter<S>,

    /// Modulation depth (output amplitude).
    pub depth: Parameter<S>,

    phase: f64,
    waveform: Waveform,
    rng: XorShiftRng,
}

impl<S: Sample> LfoBlock<S> {
    const MODULATION_OUTPUTS: &'static [ModulationOutput] = &[ModulationOutput {
        name: "LFO",
        min_value: -1.0,
        max_value: 1.0,
    }];

    /// Create an `LfoBlock` with a given frequency, depth, waveform, and optional seed (used for noise waveforms).
    pub fn new(frequency: S, depth: S, waveform: Waveform, seed: Option<u64>) -> Self {
        Self {
            frequency: Parameter::Constant(frequency),
            depth: Parameter::Constant(depth),
            phase: 0.0,
            waveform,
            rng: XorShiftRng::new(seed.unwrap_or_default()),
        }
    }
}

impl<S: Sample> Block<S> for LfoBlock<S> {
    fn process(&mut self, _inputs: &[&[S]], outputs: &mut [&mut [S]], modulation_values: &[S], context: &DspContext) {
        let frequency = self.frequency.get_value(modulation_values);
        let depth = self.depth.get_value(modulation_values).to_f64();
        let phase_increment = frequency.to_f64() / context.sample_rate * std::f64::consts::TAU;

        #[cfg(feature = "simd")]
        {
            use crate::waveform::DEFAULT_DUTY_CYCLE;

            if !matches!(self.waveform, Waveform::Noise) {
                let buffer_size = context.buffer_size;
                let chunks = buffer_size / SIMD_LANES;
                let remainder_start = chunks * SIMD_LANES;
                let chunk_phase_step = phase_increment * SIMD_LANES as f64;
                let depth_s = S::from_f64(depth);
                let depth_vec = S::simd_splat(depth_s);

                let base_phase = S::simd_splat(S::from_f64(self.phase));
                let sample_inc_simd = S::simd_splat(S::from_f64(phase_increment));
                let mut phases = base_phase + S::simd_lane_offsets() * sample_inc_simd;
                let chunk_inc_simd = S::simd_splat(S::from_f64(chunk_phase_step));
                let duty = S::from_f64(DEFAULT_DUTY_CYCLE);
                let two_pi = S::simd_splat(S::from_f64(std::f64::consts::TAU));
                let inv_two_pi = S::simd_splat(S::from_f64(1.0 / std::f64::consts::TAU));

                for chunk_idx in 0..chunks {
                    if let Some(samples) =
                        generate_waveform_samples_simd_generic::<S>(self.waveform, phases, duty, two_pi, inv_two_pi)
                    {
                        let samples_vec = S::simd_from_slice(&samples);
                        let scaled = samples_vec * depth_vec;
                        let base = chunk_idx * SIMD_LANES;
                        outputs[0][base..base + SIMD_LANES].copy_from_slice(&S::simd_to_array(scaled));
                    }

                    phases = phases + chunk_inc_simd;
                }

                self.phase += chunk_phase_step * chunks as f64;
                self.phase = self.phase.rem_euclid(std::f64::consts::TAU);

                process_waveform_scalar(
                    &mut outputs[0][remainder_start..],
                    self.waveform,
                    &mut self.phase,
                    phase_increment,
                    &mut self.rng,
                    depth,
                );
            } else {
                process_waveform_scalar(
                    outputs[0],
                    self.waveform,
                    &mut self.phase,
                    phase_increment,
                    &mut self.rng,
                    depth,
                );
            }
        }

        #[cfg(not(feature = "simd"))]
        {
            process_waveform_scalar(
                outputs[0],
                self.waveform,
                &mut self.phase,
                phase_increment,
                &mut self.rng,
                depth,
            );
        }
    }

    #[inline]
    fn input_count(&self) -> usize {
        DEFAULT_MODULATOR_INPUT_COUNT
    }

    #[inline]
    fn output_count(&self) -> usize {
        DEFAULT_MODULATOR_OUTPUT_COUNT
    }

    #[inline]
    fn modulation_outputs(&self) -> &[ModulationOutput] {
        Self::MODULATION_OUTPUTS
    }
}
