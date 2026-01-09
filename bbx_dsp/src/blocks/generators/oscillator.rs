//! Waveform oscillator block.

use bbx_core::random::XorShiftRng;

#[cfg(feature = "simd")]
use crate::sample::SIMD_LANES;
#[cfg(feature = "simd")]
use crate::waveform::generate_waveform_samples_simd_generic;
use crate::{
    block::{Block, DEFAULT_GENERATOR_INPUT_COUNT, DEFAULT_GENERATOR_OUTPUT_COUNT},
    context::DspContext,
    parameter::{ModulationOutput, Parameter},
    sample::Sample,
    waveform::{Waveform, process_waveform_scalar},
};

/// A waveform oscillator for generating audio signals.
///
/// Supports standard waveforms (sine, square, sawtooth, triangle, pulse, noise).
/// Frequency can be controlled via parameter modulation or MIDI note messages.
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
    ///
    /// # Arguments
    ///
    /// * `frequency` - Base frequency in Hz
    /// * `waveform` - The waveform shape to generate
    /// * `seed` - Optional RNG seed for noise waveform
    pub fn new(frequency: S, waveform: Waveform, seed: Option<u64>) -> Self {
        Self {
            frequency: Parameter::Constant(frequency),
            pitch_offset: Parameter::Constant(S::ZERO),
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

        let freq_hz = match &self.frequency {
            Parameter::Constant(f) => self.midi_frequency.unwrap_or(*f),
            Parameter::Modulated(block_id) => {
                // Add modulation to base (allows vibrato on top of MIDI note)
                // Safe lookup to prevent panic in audio thread
                let mod_value = modulation_values.get(block_id.0).copied().unwrap_or(S::ZERO);
                base + mod_value
            }
        };

        let pitch_offset_semitones = match &self.pitch_offset {
            Parameter::Constant(offset) => *offset,
            Parameter::Modulated(block_id) => {
                // Safe lookup to prevent panic in audio thread
                modulation_values.get(block_id.0).copied().unwrap_or(S::ZERO)
            }
        };

        // Convert semitones to frequency multiplier: 2^(semitones/12)
        let freq = if pitch_offset_semitones != S::ZERO {
            let multiplier = S::from_f64(2.0f64.powf(pitch_offset_semitones.to_f64() / 12.0));
            freq_hz * multiplier
        } else {
            freq_hz
        };

        let phase_increment = freq.to_f64() / context.sample_rate * std::f64::consts::TAU;

        #[cfg(feature = "simd")]
        {
            use crate::waveform::DEFAULT_DUTY_CYCLE;

            if !matches!(self.waveform, Waveform::Noise) {
                let buffer_size = context.buffer_size;
                let chunks = buffer_size / SIMD_LANES;
                let remainder_start = chunks * SIMD_LANES;
                let inc_lanes = phase_increment * SIMD_LANES as f64;

                for chunk_idx in 0..chunks {
                    // Build phase array in f64, then convert to S::Simd
                    let phase_arr: [S; SIMD_LANES] = [
                        S::from_f64(self.phase),
                        S::from_f64(self.phase + phase_increment),
                        S::from_f64(self.phase + phase_increment * 2.0),
                        S::from_f64(self.phase + phase_increment * 3.0),
                    ];
                    let phases = S::simd_from_slice(&phase_arr);
                    let duty = S::from_f64(DEFAULT_DUTY_CYCLE);

                    if let Some(samples) = generate_waveform_samples_simd_generic::<S>(self.waveform, phases, duty) {
                        let base = chunk_idx * SIMD_LANES;
                        outputs[0][base..base + SIMD_LANES].copy_from_slice(&samples);
                    }

                    self.phase += inc_lanes;
                }

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
