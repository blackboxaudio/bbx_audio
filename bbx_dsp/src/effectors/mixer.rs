use crate::{
    buffer::AudioBuffer,
    process::{AudioInput, Process},
    utils::{clear_output, sum_audio_inputs},
};

pub struct MixerEffector;

impl Process for MixerEffector {
    fn process(&mut self, inputs: &[AudioInput], output: &mut [AudioBuffer<f32>]) {
        clear_output(output);
        sum_audio_inputs(inputs, output);
    }
}
