use crate::{
    buffer::AudioBuffer,
    process::{AudioInput, Process},
    utils::sum_audio_inputs,
};

pub struct MixerEffector;

impl Process for MixerEffector {
    fn process(&mut self, inputs: &[AudioInput], output: &mut [AudioBuffer<f32>]) {
        sum_audio_inputs(inputs, output);
    }
}
