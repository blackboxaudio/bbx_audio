use bbx_buffer::buffer::{AudioBuffer, Buffer};

use crate::{
    context::Context,
    process::{AudioInput, Process},
    utils::{clear_output, sum_audio_inputs},
};

pub struct FilterEffector {
    context: Context,
    cutoff: f32,
    resonance: f32,
    stages: [f32; 4],
}

impl FilterEffector {
    pub fn new(context: Context, cutoff: f32, resonance: f32) -> Self {
        FilterEffector {
            context,
            cutoff,
            resonance,
            stages: [0.0; 4],
        }
    }
}

impl Process for FilterEffector {
    fn process(&mut self, inputs: &[AudioInput], output: &mut [AudioBuffer<f32>]) {
        clear_output(output);
        sum_audio_inputs(inputs, output);

        for sample_idx in 0..output[0].len() {
            let g = (std::f32::consts::PI * self.cutoff / self.context.sample_rate as f32).sin();

            for channel_buffer in output.iter_mut() {
                let input_sample = channel_buffer[sample_idx];
                let feedback = self.resonance * self.stages[3];

                self.stages[0] = self.stages[0] + g * (input_sample - feedback - self.stages[0]);
                self.stages[1] = self.stages[1] + g * (self.stages[0] - self.stages[1]);
                self.stages[2] = self.stages[2] + g * (self.stages[1] - self.stages[2]);
                self.stages[3] = self.stages[3] + g * (self.stages[2] - self.stages[3]);

                channel_buffer[sample_idx] = self.stages[3];
            }
        }
    }
}
