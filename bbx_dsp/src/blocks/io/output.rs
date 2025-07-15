use std::marker::PhantomData;

use crate::{block::Block, context::DspContext, parameter::ModulationOutput, sample::Sample};

/// Used for collecting audio output from all the relevant blocks within a DSP `Graph`.
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

    fn input_count(&self) -> usize {
        self.num_channels
    }
    fn output_count(&self) -> usize {
        self.num_channels
    }
    fn modulation_outputs(&self) -> &[ModulationOutput] {
        &[]
    }
}
