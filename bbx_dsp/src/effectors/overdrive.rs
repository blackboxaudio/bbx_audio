use bbx_buffer::buffer::{AudioBuffer, Buffer};

use crate::{
    process::{AudioInput, ModulationInput, Process},
    utils::{clear_output, sum_audio_inputs},
};

pub struct OverdriveEffector;

impl Process for OverdriveEffector {
    fn process(
        &mut self,
        inputs: &[AudioInput],
        output: &mut [AudioBuffer<f32>],
        _mod_inputs: &[ModulationInput],
        _mod_output: &mut Vec<f32>,
    ) {
        clear_output(output);
        sum_audio_inputs(inputs, output);

        for channel_buffer in output.iter_mut() {
            channel_buffer.apply(|s| s - (s.powi(3) / 3.0));
        }
    }
}
