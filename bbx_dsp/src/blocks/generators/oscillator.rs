use bbx_core::random::XorShiftRng;

use crate::{
    block::Block,
    context::DspContext,
    parameter::{ModulationOutput, Parameter},
    sample::Sample,
    waveform::{DEFAULT_DUTY_CYCLE, Waveform, generate_waveform_sample},
};

pub struct OscillatorBlock<S: Sample> {
    pub frequency: Parameter<S>,

    base_frequency: S,
    phase: f64,
    waveform: Waveform,
    rng: XorShiftRng,
}

impl<S: Sample> OscillatorBlock<S> {
    pub fn new(frequency: S, waveform: Waveform, seed: Option<u64>) -> Self {
        Self {
            base_frequency: frequency,
            frequency: Parameter::Constant(frequency),
            phase: 0.0,
            waveform,
            rng: XorShiftRng::new(seed.unwrap_or_default()),
        }
    }
}

impl<S: Sample> Block<S> for OscillatorBlock<S> {
    fn process(&mut self, _inputs: &[&[S]], outputs: &mut [&mut [S]], modulation_values: &[S], context: &DspContext) {
        let freq = match &self.frequency {
            Parameter::Constant(freq) => *freq,
            Parameter::Modulated(block_id) => self.base_frequency + modulation_values[block_id.0],
        };

        let phase_increment = freq.to_f64() / context.sample_rate * 2.0 * std::f64::consts::PI;

        for sample_index in 0..context.buffer_size {
            let sample_value = generate_waveform_sample(self.waveform, self.phase, DEFAULT_DUTY_CYCLE, &mut self.rng);
            let sample = S::from_f64(sample_value);
            outputs[0][sample_index] = sample;
            self.phase += phase_increment;
        }

        while self.phase >= 2.0 * std::f64::consts::PI {
            self.phase -= 2.0 * std::f64::consts::PI;
        }
    }

    fn input_count(&self) -> usize {
        0
    }
    fn output_count(&self) -> usize {
        1
    }
    fn modulation_outputs(&self) -> &[ModulationOutput] {
        &[]
    }
}
