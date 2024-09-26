use crate::{
    buffer::{AudioBuffer, Buffer},
    process::{AudioInput, Process},
    utils::sum_audio_inputs,
};
use crate::utils::clear_output;

pub struct OverdriveEffector;

impl Process for OverdriveEffector {
    fn process(&mut self, inputs: &[AudioInput], output: &mut [AudioBuffer<f32>]) {
        clear_output(output);
        sum_audio_inputs(inputs, output);

        for channel_buffer in output.iter_mut() {
            channel_buffer.apply(|s| s - (s.powi(3) / 3.0))
        }
    }
}
