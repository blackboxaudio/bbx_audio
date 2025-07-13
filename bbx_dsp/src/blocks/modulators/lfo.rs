use crate::{
    block::Block,
    context::DspContext,
    parameter::{ModulationOutput, Parameter},
    sample::Sample,
    waveform::Waveform,
};

pub struct LfoBlock<S: Sample> {
    pub frequency: Parameter<S>,
    pub depth: Parameter<S>,

    phase: f64,
    waveform: Waveform,
}

impl<S: Sample> LfoBlock<S> {
    const MODULATION_OUTPUTS: &'static [ModulationOutput] = &[ModulationOutput {
        name: "LFO",
        min_value: -1.0,
        max_value: 1.0,
    }];

    pub fn new(frequency: S, depth: S, waveform: Waveform) -> Self {
        Self {
            frequency: Parameter::Constant(frequency),
            depth: Parameter::Constant(depth),
            phase: 0.0,
            waveform,
        }
    }

    pub fn frequency(&self) -> &Parameter<S> {
        &self.frequency
    }
}

impl<S: Sample> Block<S> for LfoBlock<S> {
    fn process(&mut self, _inputs: &[&[S]], outputs: &mut [&mut [S]], modulation_values: &[S], context: &DspContext) {
        let frequency = self.frequency.get_value(modulation_values);
        let depth = self.depth.get_value(modulation_values);
        let phase_increment = frequency.to_f64() / context.sample_rate * 2.0 * std::f64::consts::PI;

        // Calculate LFO value at the start of the buffer
        let lfo_value = match self.waveform {
            Waveform::Sine => self.phase.sin(),
        } * depth.to_f64();

        let sample_value = S::from_f64(lfo_value);

        // Fill the entire buffer with this value
        // (For audio-rate modulation, you'd calculate per-sample)
        for sample_index in 0..context.buffer_size {
            outputs[0][sample_index] = sample_value;
        }

        // Advance phase by the entire buffer duration
        self.phase += phase_increment * context.buffer_size as f64;

        // Wrap phase
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
        Self::MODULATION_OUTPUTS
    }
}
