//! Channel splitter block for separating multi-channel audio.

use std::marker::PhantomData;

use crate::{
    block::Block,
    channel::ChannelConfig,
    context::DspContext,
    graph::MAX_BLOCK_OUTPUTS,
    parameter::ModulationOutput,
    sample::Sample,
};

/// Splits multi-channel input into individual mono outputs.
///
/// Each input channel is copied to the corresponding output port,
/// allowing downstream blocks to process channels independently.
///
/// # Example
/// A 4-channel splitter takes 4 input channels and outputs them
/// as 4 separate mono signals that can be routed to different blocks.
pub struct ChannelSplitterBlock<S: Sample> {
    channel_count: usize,
    _phantom: PhantomData<S>,
}

impl<S: Sample> ChannelSplitterBlock<S> {
    /// Create a new channel splitter for the given number of channels.
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

    /// Returns the number of channels this splitter handles.
    pub fn channel_count(&self) -> usize {
        self.channel_count
    }
}

impl<S: Sample> Block<S> for ChannelSplitterBlock<S> {
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
    fn test_channel_splitter_stereo() {
        let mut splitter = ChannelSplitterBlock::<f32>::new(2);
        let context = test_context();

        let left_in = [1.0f32, 2.0, 3.0, 4.0];
        let right_in = [5.0f32, 6.0, 7.0, 8.0];
        let mut left_out = [0.0f32; 4];
        let mut right_out = [0.0f32; 4];

        let inputs: [&[f32]; 2] = [&left_in, &right_in];
        let mut outputs: [&mut [f32]; 2] = [&mut left_out, &mut right_out];

        splitter.process(&inputs, &mut outputs, &[], &context);

        assert_eq!(left_out, left_in);
        assert_eq!(right_out, right_in);
    }

    #[test]
    fn test_channel_splitter_quad() {
        let splitter = ChannelSplitterBlock::<f32>::new(4);
        assert_eq!(splitter.input_count(), 4);
        assert_eq!(splitter.output_count(), 4);
        assert_eq!(splitter.channel_config(), ChannelConfig::Explicit);
    }

    #[test]
    #[should_panic]
    fn test_channel_splitter_zero_channels_panics() {
        let _ = ChannelSplitterBlock::<f32>::new(0);
    }

    #[test]
    #[should_panic]
    fn test_channel_splitter_too_many_channels_panics() {
        let _ = ChannelSplitterBlock::<f32>::new(17);
    }
}
