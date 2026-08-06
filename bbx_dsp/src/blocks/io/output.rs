//! Terminal output block for DSP graphs.

use core::marker::PhantomData;

use crate::{block::Block, context::DspContext, parameter::ModulationOutput, sample::Sample};

/// The terminal output block for a DSP graph.
///
/// Collects final audio from upstream blocks and makes it available
/// for playback or further processing outside the graph.
pub struct OutputBlock<S: Sample> {
    num_channels: usize,
    _phantom: PhantomData<S>,
}

impl<S: Sample> OutputBlock<S> {
    /// Create an `OutputBlock` with a given number of channels.
    pub fn new(num_channels: usize) -> Self {
        Self {
            num_channels,
            _phantom: PhantomData,
        }
    }
}

impl<S: Sample> Block<S> for OutputBlock<S> {
    fn process(&mut self, inputs: &[&[S]], outputs: &mut [&mut [S]], _modulation_values: &[S], _context: &DspContext) {
        for (input, output) in inputs.iter().zip(outputs.iter_mut()) {
            output.copy_from_slice(input);
        }
    }

    #[inline]
    fn input_count(&self) -> usize {
        self.num_channels
    }

    #[inline]
    fn output_count(&self) -> usize {
        self.num_channels
    }

    #[inline]
    fn modulation_outputs(&self) -> &[ModulationOutput] {
        &[]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::channel::ChannelLayout;

    fn test_context(buffer_size: usize) -> DspContext {
        DspContext {
            sample_rate: 44100.0,
            num_channels: 2,
            buffer_size,
            current_sample: 0,
            channel_layout: ChannelLayout::Stereo,
        }
    }

    #[test]
    fn test_output_block_input_output_counts_f32() {
        let block = OutputBlock::<f32>::new(2);
        assert_eq!(block.input_count(), 2);
        assert_eq!(block.output_count(), 2);
    }

    #[test]
    fn test_output_block_input_output_counts_f64() {
        let block = OutputBlock::<f64>::new(2);
        assert_eq!(block.input_count(), 2);
        assert_eq!(block.output_count(), 2);
    }

    #[test]
    fn test_output_block_mono_f32() {
        let block = OutputBlock::<f32>::new(1);
        assert_eq!(block.input_count(), 1);
        assert_eq!(block.output_count(), 1);
    }

    #[test]
    fn test_output_block_multichannel_f32() {
        let block = OutputBlock::<f32>::new(6);
        assert_eq!(block.input_count(), 6);
        assert_eq!(block.output_count(), 6);
    }

    #[test]
    fn test_output_block_passthrough_f32() {
        let mut block = OutputBlock::<f32>::new(2);
        let context = test_context(4);

        let left_in = [1.0f32, 2.0, 3.0, 4.0];
        let right_in = [5.0f32, 6.0, 7.0, 8.0];
        let mut left_out = [0.0f32; 4];
        let mut right_out = [0.0f32; 4];

        let inputs: [&[f32]; 2] = [&left_in, &right_in];
        let mut outputs: [&mut [f32]; 2] = [&mut left_out, &mut right_out];

        block.process(&inputs, &mut outputs, &[], &context);

        assert_eq!(left_out, left_in);
        assert_eq!(right_out, right_in);
    }

    #[test]
    fn test_output_block_passthrough_f64() {
        let mut block = OutputBlock::<f64>::new(2);
        let context = test_context(4);

        let left_in = [1.0f64, 2.0, 3.0, 4.0];
        let right_in = [5.0f64, 6.0, 7.0, 8.0];
        let mut left_out = [0.0f64; 4];
        let mut right_out = [0.0f64; 4];

        let inputs: [&[f64]; 2] = [&left_in, &right_in];
        let mut outputs: [&mut [f64]; 2] = [&mut left_out, &mut right_out];

        block.process(&inputs, &mut outputs, &[], &context);

        assert_eq!(left_out, left_in);
        assert_eq!(right_out, right_in);
    }

    #[test]
    fn test_output_block_modulation_outputs_empty() {
        let block = OutputBlock::<f32>::new(2);
        assert!(block.modulation_outputs().is_empty());
    }
}
