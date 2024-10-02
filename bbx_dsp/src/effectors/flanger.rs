use bbx_buffer::buffer::{AudioBuffer, Buffer};

use crate::{
    context::Context,
    process::{AudioInput, Process},
    utils::{clear_output, sum_audio_inputs},
};

pub struct FlangerEffector {
    context: Context,
    depth: f32,
    feedback: f32,
    rate: f32,
    delay_time: f32,
    delay_buffer: Vec<AudioBuffer<f32>>,
    delay_idx: usize,
    delay_phase: f32,
}

impl FlangerEffector {
    pub fn new(context: Context, depth: f32, feedback: f32, rate: f32, delay_time: f32) -> Self {
        let max_num_delay_samples = (delay_time * context.sample_rate as f32).round() as usize;
        FlangerEffector {
            context,
            depth,
            feedback,
            rate,
            delay_time,
            delay_buffer: vec![AudioBuffer::new(max_num_delay_samples); context.num_channels],
            delay_idx: 0,
            delay_phase: 0.0,
        }
    }

    pub fn get_delay_time(&self) -> f32 {
        self.delay_time
    }

    pub fn set_delay_time(&mut self, delay_time: f32) {
        self.delay_time = delay_time;
    }
}

impl Process for FlangerEffector {
    fn process(&mut self, inputs: &[AudioInput], output: &mut [AudioBuffer<f32>]) {
        clear_output(output);
        sum_audio_inputs(inputs, output);

        let max_delay_samples = (self.delay_time * self.context.sample_rate as f32).round() as usize;

        for sample_idx in 0..self.context.buffer_size {
            let lfo = (self.delay_phase * 2.0 * std::f32::consts::PI).sin() * self.depth;
            let modulated_delay = (self.delay_time + lfo) * self.context.sample_rate as f32;
            let delay_samples = modulated_delay.round() as usize;

            for (channel_idx, channel_buffer) in output.iter_mut().enumerate() {
                let delayed_sample = self.delay_buffer[channel_idx]
                    [(self.delay_idx + max_delay_samples - delay_samples) % max_delay_samples];
                let wet_sample = channel_buffer[sample_idx] + self.feedback * delayed_sample;

                channel_buffer[sample_idx] = wet_sample;
                self.delay_buffer[channel_idx][self.delay_idx] = wet_sample;
            }

            self.delay_idx = (self.delay_idx + 1) % max_delay_samples;
            self.delay_phase += self.rate / self.context.sample_rate as f32;
            if self.delay_phase >= 1.0 {
                self.delay_phase -= 1.0;
            }
        }
    }
}
