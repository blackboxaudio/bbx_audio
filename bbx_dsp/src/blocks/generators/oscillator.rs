//! Waveform oscillator block.

use bbx_core::random::XorShiftRng;

#[cfg(feature = "simd")]
use crate::sample::SIMD_LANES;
#[cfg(feature = "simd")]
use crate::waveform::generate_waveform_samples_simd;
use crate::{
    block::{Block, DEFAULT_GENERATOR_INPUT_COUNT, DEFAULT_GENERATOR_OUTPUT_COUNT},
    context::DspContext,
    parameter::{ModulationOutput, Parameter, ParameterSource},
    sample::Sample,
    waveform::{Waveform, process_waveform_scalar},
};

/// A waveform oscillator for generating audio signals.
///
/// Supports standard waveforms (sine, square, sawtooth, triangle, pulse, noise).
/// Frequency can be controlled via parameter modulation or MIDI note messages.
///
/// Uses PolyBLEP/PolyBLAMP for band-limited output, reducing aliasing artifacts.
pub struct OscillatorBlock<S: Sample> {
    /// Base frequency in Hz (can be modulated).
    pub frequency: Parameter<S>,

    /// Pitch offset in semitones (for pitch bend/modulation).
    pub pitch_offset: Parameter<S>,

    base_frequency: S,
    midi_frequency: Option<S>,
    phase: f64,
    waveform: Waveform,
    rng: XorShiftRng,
}

impl<S: Sample> OscillatorBlock<S> {
    /// Create a new oscillator with the given frequency and waveform.
    pub fn new(frequency: S, waveform: Waveform, seed: Option<u64>) -> Self {
        Self {
            frequency: Parameter::constant(frequency),
            pitch_offset: Parameter::constant(S::ZERO),
            base_frequency: frequency,
            midi_frequency: None,
            phase: 0.0,
            waveform,
            rng: XorShiftRng::new(seed.unwrap_or_default()),
        }
    }

    /// Set the MIDI-controlled frequency (called by voice manager on note-on).
    pub fn set_midi_frequency(&mut self, frequency: S) {
        self.midi_frequency = Some(frequency);
    }

    /// Clear the MIDI frequency (called on note-off or when returning to parameter control).
    pub fn clear_midi_frequency(&mut self) {
        self.midi_frequency = None;
    }

    /// Set the waveform type.
    pub fn set_waveform(&mut self, waveform: Waveform) {
        self.waveform = waveform;
    }
}

impl<S: Sample> Block<S> for OscillatorBlock<S> {
    fn process(&mut self, _inputs: &[&[S]], outputs: &mut [&mut [S]], modulation_values: &[S], context: &DspContext) {
        let base = self.midi_frequency.unwrap_or(self.base_frequency);

        let freq_hz = match self.frequency.source() {
            ParameterSource::Constant(f) => self.midi_frequency.unwrap_or(*f),
            ParameterSource::Modulated(block_id) => {
                let mod_value = modulation_values.get(block_id.0).copied().unwrap_or(S::ZERO);
                base + mod_value
            }
        };

        let pitch_offset_semitones = match self.pitch_offset.source() {
            ParameterSource::Constant(offset) => *offset,
            ParameterSource::Modulated(block_id) => modulation_values.get(block_id.0).copied().unwrap_or(S::ZERO),
        };

        let freq = if pitch_offset_semitones != S::ZERO {
            let multiplier = S::from_f64(2.0f64.powf(pitch_offset_semitones.to_f64() / 12.0));
            freq_hz * multiplier
        } else {
            freq_hz
        };

        let phase_increment = freq.to_f64() / context.sample_rate * S::TAU.to_f64();

        #[cfg(feature = "simd")]
        {
            use crate::waveform::DEFAULT_DUTY_CYCLE;

            if !matches!(self.waveform, Waveform::Noise) {
                let buffer_size = context.buffer_size;
                let chunks = buffer_size / SIMD_LANES;
                let remainder_start = chunks * SIMD_LANES;
                let chunk_phase_step = phase_increment * SIMD_LANES as f64;

                let base_phase = S::simd_splat(S::from_f64(self.phase));
                let sample_inc_simd = S::simd_splat(S::from_f64(phase_increment));
                let mut phases = base_phase + S::simd_lane_offsets() * sample_inc_simd;
                let chunk_inc_simd = S::simd_splat(S::from_f64(chunk_phase_step));
                let duty = S::from_f64(DEFAULT_DUTY_CYCLE);
                let two_pi = S::simd_splat(S::TAU);
                let inv_two_pi = S::simd_splat(S::INV_TAU);
                let phase_inc_normalized = S::from_f64(phase_increment * S::INV_TAU.to_f64());
                let tau = S::TAU.to_f64();
                let inv_tau = 1.0 / tau;

                for chunk_idx in 0..chunks {
                    let phases_array = S::simd_to_array(phases);
                    let phases_normalized: [S; SIMD_LANES] = [
                        S::from_f64(phases_array[0].to_f64().rem_euclid(tau) * inv_tau),
                        S::from_f64(phases_array[1].to_f64().rem_euclid(tau) * inv_tau),
                        S::from_f64(phases_array[2].to_f64().rem_euclid(tau) * inv_tau),
                        S::from_f64(phases_array[3].to_f64().rem_euclid(tau) * inv_tau),
                    ];

                    if let Some(samples) = generate_waveform_samples_simd::<S>(
                        self.waveform,
                        phases,
                        phases_normalized,
                        phase_inc_normalized,
                        duty,
                        two_pi,
                        inv_two_pi,
                    ) {
                        let base = chunk_idx * SIMD_LANES;
                        outputs[0][base..base + SIMD_LANES].copy_from_slice(&samples);
                    }

                    phases = phases + chunk_inc_simd;
                }

                self.phase += chunk_phase_step * chunks as f64;
                self.phase = self.phase.rem_euclid(S::TAU.to_f64());

                process_waveform_scalar(
                    &mut outputs[0][remainder_start..],
                    self.waveform,
                    &mut self.phase,
                    phase_increment,
                    &mut self.rng,
                    1.0,
                );
            } else {
                process_waveform_scalar(
                    outputs[0],
                    self.waveform,
                    &mut self.phase,
                    phase_increment,
                    &mut self.rng,
                    1.0,
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
                1.0,
            );
        }
    }

    #[inline]
    fn input_count(&self) -> usize {
        DEFAULT_GENERATOR_INPUT_COUNT
    }

    #[inline]
    fn output_count(&self) -> usize {
        DEFAULT_GENERATOR_OUTPUT_COUNT
    }

    #[inline]
    fn modulation_outputs(&self) -> &[ModulationOutput] {
        &[]
    }
}
