use bbx_core::random::XorShiftRng;

use crate::{
    block::{Block, DEFAULT_GENERATOR_INPUT_COUNT, DEFAULT_GENERATOR_OUTPUT_COUNT},
    context::DspContext,
    parameter::{ModulationOutput, Parameter},
    sample::Sample,
    waveform::{DEFAULT_DUTY_CYCLE, Waveform, generate_waveform_sample},
};

/// Used for generating the sound of a simple waveform.
pub struct OscillatorBlock<S: Sample> {
    pub frequency: Parameter<S>,
    pub pitch_offset: Parameter<S>,

    base_frequency: S,
    midi_frequency: Option<S>,
    phase: f64,
    waveform: Waveform,
    rng: XorShiftRng,
}

impl<S: Sample> OscillatorBlock<S> {
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
}

impl<S: Sample> Block<S> for OscillatorBlock<S> {
    fn process(&mut self, _inputs: &[&[S]], outputs: &mut [&mut [S]], modulation_values: &[S], context: &DspContext) {
        // Determine base frequency (MIDI takes priority if set)
        let base = self.midi_frequency.unwrap_or(self.base_frequency);

        // Apply Hz-based frequency modulation
        let freq_hz = match &self.frequency {
            Parameter::Constant(f) => {
                // Use MIDI frequency if set, otherwise use the constant
                self.midi_frequency.unwrap_or(*f)
            }
            Parameter::Modulated(block_id) => {
                // Add modulation to base (allows vibrato on top of MIDI note)
                // Safe lookup to prevent panic in audio thread
                let mod_value = modulation_values.get(block_id.0).copied().unwrap_or(S::ZERO);
                base + mod_value
            }
        };

        // Apply pitch offset (semitones) - for pitch bend
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

        for sample_index in 0..context.buffer_size {
            let sample_value = generate_waveform_sample(self.waveform, self.phase, DEFAULT_DUTY_CYCLE, &mut self.rng);
            outputs[0][sample_index] = S::from_f64(sample_value);
            self.phase += phase_increment;
        }

        // Wrap phase using modulo for efficiency (avoids while loop with extreme frequencies)
        self.phase = self.phase.rem_euclid(std::f64::consts::TAU);
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
