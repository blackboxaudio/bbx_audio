//! Channel merger block for combining individual channels into multi-channel audio.

use core::marker::PhantomData;

use crate::{
    block::{Block, MAX_BLOCK_OUTPUTS},
    channel::ChannelConfig,
    context::DspContext,
    parameter::ModulationOutput,
    sample::Sample,
};

/// Merges individual mono inputs into a multi-channel output.
///
/// Each input channel is copied to the corresponding output port,
/// combining separate mono signals into a single multi-channel stream.
///
/// # Example
/// A 4-channel merger takes 4 separate mono signals and combines them
/// into a 4-channel output that can be processed by blocks expecting
/// multi-channel input.
pub struct ChannelMergerBlock<S: Sample> {
    channel_count: usize,
    _phantom: PhantomData<S>,
}

impl<S: Sample> ChannelMergerBlock<S> {
    /// Create a new channel merger for the given number of channels.
    ///
    /// # Panics
    /// Panics if `channels` is 0 or greater than `MAX_BLOCK_OUTPUTS` (16).
    pub fn new(channels: usize) -> Self {
        assert!(channels > 0 && channels <= MAX_BLOCK_OUTPUTS);
        Self {
            channel_count: channels,
            _phantom: PhantomData,
        }
    }

    /// Returns the number of channels this merger handles.
    pub fn channel_count(&self) -> usize {
        self.channel_count
    }
}

impl<S: Sample> Block<S> for ChannelMergerBlock<S> {
    fn process(&mut self, inputs: &[&[S]], outputs: &mut [&mut [S]], _modulation_values: &[S], _context: &DspContext) {
        let num_channels = self.channel_count.min(inputs.len()).min(outputs.len());

        for ch in 0..num_channels {
            let input = inputs[ch];
            let output = &mut outputs[ch];
            let num_samples = input.len().min(output.len());

            output[..num_samples].copy_from_slice(&input[..num_samples]);
        }
    }

    #[inline]
    fn input_count(&self) -> usize {
        self.channel_count
    }

    #[inline]
    fn output_count(&self) -> usize {
        self.channel_count
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
    fn test_channel_merger_stereo() {
        let mut merger = ChannelMergerBlock::<f32>::new(2);
        let context = test_context();

        let left_in = [1.0f32, 2.0, 3.0, 4.0];
        let right_in = [5.0f32, 6.0, 7.0, 8.0];
        let mut left_out = [0.0f32; 4];
        let mut right_out = [0.0f32; 4];

        let inputs: [&[f32]; 2] = [&left_in, &right_in];
        let mut outputs: [&mut [f32]; 2] = [&mut left_out, &mut right_out];

        merger.process(&inputs, &mut outputs, &[], &context);

        assert_eq!(left_out, left_in);
        assert_eq!(right_out, right_in);
    }

    #[test]
    fn test_channel_merger_quad() {
        let merger = ChannelMergerBlock::<f32>::new(4);
        assert_eq!(merger.input_count(), 4);
        assert_eq!(merger.output_count(), 4);
        assert_eq!(merger.channel_config(), ChannelConfig::Explicit);
    }

    #[test]
    #[should_panic]
    fn test_channel_merger_zero_channels_panics() {
        let _ = ChannelMergerBlock::<f32>::new(0);
    }

    #[test]
    #[should_panic]
    fn test_channel_merger_too_many_channels_panics() {
        let _ = ChannelMergerBlock::<f32>::new(17);
    }

    #[test]
    fn test_channel_merger_input_output_counts_f64() {
        let merger = ChannelMergerBlock::<f64>::new(4);
        assert_eq!(merger.input_count(), 4);
        assert_eq!(merger.output_count(), 4);
        assert_eq!(merger.channel_config(), ChannelConfig::Explicit);
    }

    #[test]
    fn test_channel_merger_stereo_f64() {
        let mut merger = ChannelMergerBlock::<f64>::new(2);
        let context = test_context();

        let left_in = [1.0f64, 2.0, 3.0, 4.0];
        let right_in = [5.0f64, 6.0, 7.0, 8.0];
        let mut left_out = [0.0f64; 4];
        let mut right_out = [0.0f64; 4];

        let inputs: [&[f64]; 2] = [&left_in, &right_in];
        let mut outputs: [&mut [f64]; 2] = [&mut left_out, &mut right_out];

        merger.process(&inputs, &mut outputs, &[], &context);

        assert_eq!(left_out, left_in);
        assert_eq!(right_out, right_in);
    }

    #[test]
    fn test_channel_merger_modulation_outputs_empty() {
        let merger = ChannelMergerBlock::<f32>::new(2);
        assert!(merger.modulation_outputs().is_empty());
    }
}
