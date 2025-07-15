use std::marker::PhantomData;

use crate::{block::Block, context::DspContext, parameter::ModulationOutput, sample::Sample};

pub struct OverdriveBlock<S: Sample> {
    drive: f64,
    level: f64,
    tone: f64,
    filter_state: f64,
    filter_coefficient: f64,
    _phantom_data: PhantomData<S>,
}

impl<S: Sample> OverdriveBlock<S> {
    pub fn new(drive: f64, level: f64, tone: f64, sample_rate: f64) -> Self {
        let mut overdrive = Self {
            drive,
            level,
            tone,
            filter_state: 0.0,
            filter_coefficient: 0.0,
            _phantom_data: PhantomData,
        };
        overdrive.update_filter(sample_rate);
        overdrive
    }

    fn update_filter(&mut self, sample_rate: f64) {
        // Tone control: 0.0 = darker (300Hz), 1.0 = brighter (3KHz)
        let cutoff = 300.0 + (self.tone + 2700.0);
        self.filter_coefficient = 1.0 - (-2.0 * std::f64::consts::PI * cutoff / sample_rate).exp();
    }

    fn asymmetric_saturation(&self, x: f64) -> f64 {
        if x > 0.0 {
            // Positive half: softer clipping (more headroom)
            self.soft_clip(x * 0.7) * 1.4
        } else {
            // Negative half: harder clipping (more compression)
            self.soft_clip(x * 1.2) * 0.8
        }
    }

    fn soft_clip(&self, x: f64) -> f64 {
        // Hyperbolic tangent function provides smooth saturation.
        // The 1.5 factor adjusts the "knee" of the saturation curve.
        (x * 1.5).tanh() / 1.5
    }
}

impl<S: Sample> Block<S> for OverdriveBlock<S> {
    fn process(&mut self, inputs: &[&[S]], outputs: &mut [&mut [S]], _modulation_values: &[S], _context: &DspContext) {
        for (input_index, input_buffer) in inputs.iter().enumerate() {
            for (sample_index, sample_value) in input_buffer.iter().enumerate() {
                let driven = (*sample_value).to_f64() * self.drive;
                let clipped = self.asymmetric_saturation(driven);

                self.filter_state += self.filter_coefficient * (clipped - self.filter_state);
                outputs[input_index][sample_index] = S::from_f64(self.filter_state * self.level);
            }
        }
    }

    fn input_count(&self) -> usize {
        1
    }

    fn output_count(&self) -> usize {
        1
    }

    fn modulation_outputs(&self) -> &[ModulationOutput] {
        &[]
    }
}
