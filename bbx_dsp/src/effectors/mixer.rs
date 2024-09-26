use crate::{
    buffer::AudioBuffer,
    process::{AudioInput, Process},
    utils::sum_audio_inputs,
};
use crate::utils::clear_output;

pub struct MixerEffector;

impl Process for MixerEffector {
    fn process(&mut self, inputs: &[AudioInput], output: &mut [AudioBuffer<f32>]) {
        clear_output(output);
        sum_audio_inputs(inputs, output);
    }
}
