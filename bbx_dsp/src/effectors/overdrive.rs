use crate::{
    buffer::{AudioBuffer, Buffer},
    process::{AudioInput, Process},
    utils::sum_audio_inputs,
};

pub struct OverdriveEffector;

impl Process for OverdriveEffector {
    fn process(&mut self, inputs: &[AudioInput], output: &mut [AudioBuffer<f32>]) {
        for output_buffer in output.iter_mut() {
            output_buffer.clear();
        }

        sum_audio_inputs(inputs, output);

        for channel_buffer in output.iter_mut() {
            channel_buffer.apply(|s| s - (s.powi(3) / 3.0))
        }
    }
}
