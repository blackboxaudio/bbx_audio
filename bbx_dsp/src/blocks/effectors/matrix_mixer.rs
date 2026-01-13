//! Matrix mixer block for flexible NxM channel routing.

use crate::{
    block::Block,
    channel::ChannelConfig,
    context::DspContext,
    graph::{MAX_BLOCK_INPUTS, MAX_BLOCK_OUTPUTS},
    parameter::ModulationOutput,
    sample::Sample,
};

/// An NxM mixing matrix for flexible channel routing.
///
/// Each output is a weighted sum of all inputs, allowing arbitrary
/// channel mixing configurations. The gain matrix determines how
/// much of each input contributes to each output.
///
/// # Example
/// A 4x2 matrix mixer can down-mix 4 channels to stereo by setting
/// appropriate gains for left/right output combinations.
pub struct MatrixMixerBlock<S: Sample> {
    num_inputs: usize,
    num_outputs: usize,
    gains: [[S; MAX_BLOCK_INPUTS]; MAX_BLOCK_OUTPUTS],
}

impl<S: Sample> MatrixMixerBlock<S> {
    /// Create a new matrix mixer with the given input and output counts.
    ///
    /// All gains are initialized to zero.
    ///
    /// # Panics
    /// Panics if `inputs` or `outputs` is 0 or greater than 16.
    pub fn new(inputs: usize, outputs: usize) -> Self {
        assert!(inputs > 0 && inputs <= MAX_BLOCK_INPUTS);
        assert!(outputs > 0 && outputs <= MAX_BLOCK_OUTPUTS);
        Self {
            num_inputs: inputs,
            num_outputs: outputs,
            gains: [[S::ZERO; MAX_BLOCK_INPUTS]; MAX_BLOCK_OUTPUTS],
        }
    }

    /// Create a matrix mixer initialized as an identity matrix (passthrough).
    ///
    /// Each input channel maps directly to the corresponding output channel
    /// with unity gain. Requires `inputs == outputs`.
    ///
    /// # Panics
    /// Panics if `channels` is 0 or greater than 16.
    pub fn identity(channels: usize) -> Self {
        let mut mixer = Self::new(channels, channels);
        for ch in 0..channels {
            mixer.gains[ch][ch] = S::ONE;
        }
        mixer
    }

    /// Set the gain for a specific input-to-output routing.
    ///
    /// # Arguments
    /// * `input` - The input channel index (0-based)
    /// * `output` - The output channel index (0-based)
    /// * `gain` - The gain to apply (typically 0.0 to 1.0)
    ///
    /// # Panics
    /// Panics if `input >= num_inputs` or `output >= num_outputs`.
    pub fn set_gain(&mut self, input: usize, output: usize, gain: S) {
        assert!(input < self.num_inputs);
        assert!(output < self.num_outputs);
        self.gains[output][input] = gain;
    }

    /// Get the gain for a specific input-to-output routing.
    ///
    /// # Panics
    /// Panics if `input >= num_inputs` or `output >= num_outputs`.
    pub fn get_gain(&self, input: usize, output: usize) -> S {
        assert!(input < self.num_inputs);
        assert!(output < self.num_outputs);
        self.gains[output][input]
    }

    /// Returns the number of input channels.
    pub fn num_inputs(&self) -> usize {
        self.num_inputs
    }

    /// Returns the number of output channels.
    pub fn num_outputs(&self) -> usize {
        self.num_outputs
    }
}

impl<S: Sample> Block<S> for MatrixMixerBlock<S> {
    fn process(&mut self, inputs: &[&[S]], outputs: &mut [&mut [S]], _modulation_values: &[S], _context: &DspContext) {
        let num_inputs = self.num_inputs.min(inputs.len());
        let num_outputs = self.num_outputs.min(outputs.len());

        if num_inputs == 0 || num_outputs == 0 {
            return;
        }

        let num_samples = inputs[0].len().min(outputs[0].len());

        for (out_ch, output) in outputs.iter_mut().enumerate().take(num_outputs) {
            for i in 0..num_samples {
                let mut sum = S::ZERO;
                for (in_ch, input) in inputs.iter().enumerate().take(num_inputs) {
                    sum += input[i] * self.gains[out_ch][in_ch];
                }
                output[i] = sum;
            }
        }
    }

    #[inline]
    fn input_count(&self) -> usize {
        self.num_inputs
    }

    #[inline]
    fn output_count(&self) -> usize {
        self.num_outputs
    }

    #[inline]
    fn modulation_outputs(&self) -> &[ModulationOutput] {
        &[]
    }

    #[inline]
    fn channel_config(&self) -> ChannelConfig {
        ChannelConfig::Explicit
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::channel::ChannelLayout;

    fn test_context() -> DspContext {
        DspContext {
            sample_rate: 44100.0,
            num_channels: 2,
            buffer_size: 4,
            current_sample: 0,
            channel_layout: ChannelLayout::Stereo,
        }
    }

    #[test]
    fn test_matrix_mixer_identity() {
        let mut mixer = MatrixMixerBlock::<f32>::identity(2);
        let context = test_context();

        let left_in = [1.0f32, 2.0, 3.0, 4.0];
        let right_in = [5.0f32, 6.0, 7.0, 8.0];
        let mut left_out = [0.0f32; 4];
        let mut right_out = [0.0f32; 4];

        let inputs: [&[f32]; 2] = [&left_in, &right_in];
        let mut outputs: [&mut [f32]; 2] = [&mut left_out, &mut right_out];

        mixer.process(&inputs, &mut outputs, &[], &context);

        assert_eq!(left_out, left_in);
        assert_eq!(right_out, right_in);
    }

    #[test]
    fn test_matrix_mixer_mono_sum() {
        let mut mixer = MatrixMixerBlock::<f32>::new(2, 1);
        mixer.set_gain(0, 0, 0.5);
        mixer.set_gain(1, 0, 0.5);
        let context = test_context();

        let left_in = [1.0f32, 2.0, 3.0, 4.0];
        let right_in = [1.0f32, 2.0, 3.0, 4.0];
        let mut mono_out = [0.0f32; 4];

        let inputs: [&[f32]; 2] = [&left_in, &right_in];
        let mut outputs: [&mut [f32]; 1] = [&mut mono_out];

        mixer.process(&inputs, &mut outputs, &[], &context);

        assert_eq!(mono_out, [1.0, 2.0, 3.0, 4.0]);
    }

    #[test]
    fn test_matrix_mixer_swap_channels() {
        let mut mixer = MatrixMixerBlock::<f32>::new(2, 2);
        mixer.set_gain(0, 1, 1.0); // left input -> right output
        mixer.set_gain(1, 0, 1.0); // right input -> left output
        let context = test_context();

        let left_in = [1.0f32, 2.0, 3.0, 4.0];
        let right_in = [5.0f32, 6.0, 7.0, 8.0];
        let mut left_out = [0.0f32; 4];
        let mut right_out = [0.0f32; 4];

        let inputs: [&[f32]; 2] = [&left_in, &right_in];
        let mut outputs: [&mut [f32]; 2] = [&mut left_out, &mut right_out];

        mixer.process(&inputs, &mut outputs, &[], &context);

        assert_eq!(left_out, right_in);
        assert_eq!(right_out, left_in);
    }

    #[test]
    fn test_matrix_mixer_counts() {
        let mixer = MatrixMixerBlock::<f32>::new(4, 2);
        assert_eq!(mixer.input_count(), 4);
        assert_eq!(mixer.output_count(), 2);
        assert_eq!(mixer.channel_config(), ChannelConfig::Explicit);
    }

    #[test]
    #[should_panic]
    fn test_matrix_mixer_zero_inputs_panics() {
        let _ = MatrixMixerBlock::<f32>::new(0, 2);
    }

    #[test]
    #[should_panic]
    fn test_matrix_mixer_zero_outputs_panics() {
        let _ = MatrixMixerBlock::<f32>::new(2, 0);
    }
}
