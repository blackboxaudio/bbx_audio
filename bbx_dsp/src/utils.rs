use crate::{
    buffer::{AudioBuffer, Buffer},
    process::AudioInput,
};
use crate::sample::Sample;

/// Zeroes the sample data inside a slice of `AudioBuffer`s.
pub fn clear_output<S: Sample>(output: &mut [AudioBuffer<S>]) {
    for channel_buffer in output {
        channel_buffer.clear();
    }
}

/// Sums a slice of audio inputs, each containing a number of audio buffers,
/// to a single output slice of output buffers.
pub fn sum_audio_inputs(inputs: &[AudioInput], output: &mut [AudioBuffer<f32>]) {
    for (channel_idx, channel_buffer) in output.iter_mut().enumerate() {
        for sample_idx in 0..channel_buffer.len() {
            let input_sum: f32 = inputs
                .iter()
                .map(|i| {
                    let input_buffers = i.as_slice();
                    &input_buffers[channel_idx][sample_idx]
                })
                .sum();
            let input_val = input_sum / inputs.len() as f32;
            let output_val = input_val.clamp(-1.0, 1.0);
            channel_buffer[sample_idx] = output_val;
        }
    }
}
