use bbx_buffer::buffer::{AudioBuffer, Buffer};

use crate::{
    process::{AudioInput, Process},
    utils::{clear_output, sum_audio_inputs},
};

pub struct AmplifierEffector {
    gain: f32,
}

impl AmplifierEffector {
    pub fn new(gain: f32) -> Self {
        AmplifierEffector {
            gain: gain.clamp(0.0, 1.0),
        }
    }

    pub fn set_gain(&mut self, gain: f32) {
        self.gain = gain.clamp(0.0, 1.0);
    }
}

impl Process for AmplifierEffector {
    fn process(&mut self, inputs: &[AudioInput], output: &mut [AudioBuffer<f32>]) {
        clear_output(output);
        sum_audio_inputs(inputs, output);

        for channel_buffer in output.iter_mut() {
            channel_buffer.apply(|s| s * self.gain);
        }
    }
}
