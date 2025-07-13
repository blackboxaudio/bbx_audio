use crate::{
    block::Block,
    context::DspContext,
    parameter::{ModulationOutput, Parameter},
    sample::Sample,
    waveform::Waveform,
};

pub struct OscillatorBlock<S: Sample> {
    pub frequency: Parameter<S>,

    base_frequency: S,
    phase: f64,
    waveform: Waveform,
}

impl<S: Sample> OscillatorBlock<S> {
    pub fn new(frequency: S, waveform: Waveform) -> Self {
        Self {
            base_frequency: frequency,
            frequency: Parameter::Constant(frequency),
            phase: 0.0,
            waveform,
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
            let sample = match self.waveform {
                Waveform::Sine => self.phase.sin(),
                // Waveform::Square => if self.phase.sin() > 0.0 { 1.0 } else { -1.0 },
            };
            let sample_value = S::from_f64(sample);
            outputs[0][sample_index] = sample_value;
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
