use bbx_core::random::XorShiftRng;

use crate::{
    block::{Block, DEFAULT_MODULATOR_INPUT_COUNT, DEFAULT_MODULATOR_OUTPUT_COUNT},
    context::DspContext,
    parameter::{ModulationOutput, Parameter},
    sample::Sample,
    waveform::{DEFAULT_DUTY_CYCLE, Waveform, generate_waveform_sample},
};

// TODO: Make sure modulation can be applied to more than one destination.
/// Used for applying waveform modulation to one or more `Parameter`s of one or more blocks.
pub struct LfoBlock<S: Sample> {
    pub frequency: Parameter<S>,
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

        // Process per-sample for smooth modulation (same pattern as oscillator)
        for sample_index in 0..context.buffer_size {
            let lfo_value = generate_waveform_sample(self.waveform, self.phase, DEFAULT_DUTY_CYCLE, &mut self.rng);
            outputs[0][sample_index] = S::from_f64(lfo_value * depth);

            self.phase += phase_increment;
        }

        // Wrap phase using modulo for efficiency (avoids while loop with extreme frequencies)
        self.phase = self.phase.rem_euclid(std::f64::consts::TAU);
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
