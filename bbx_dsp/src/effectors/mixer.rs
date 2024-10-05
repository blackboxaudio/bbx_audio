use bbx_buffer::buffer::AudioBuffer;

use crate::{
    process::{AudioInput, ModulationInput, Process},
    utils::{clear_output, sum_audio_inputs},
};

pub struct MixerEffector;

impl Process for MixerEffector {
    fn process(
        &mut self,
        inputs: &[AudioInput],
        output: &mut [AudioBuffer<f32>],
        _mod_inputs: &[ModulationInput],
        _mod_output: &mut Vec<f32>,
    ) {
        clear_output(output);
        sum_audio_inputs(inputs, output);
    }
}
