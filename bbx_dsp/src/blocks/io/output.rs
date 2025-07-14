use std::marker::PhantomData;

use crate::{block::Block, context::DspContext, parameter::ModulationOutput, sample::Sample};

pub struct OutputBlock<S: Sample> {
    channels: usize,
    _phantom: PhantomData<S>,
}

impl<S: Sample> OutputBlock<S> {
    pub fn new(channels: usize) -> Self {
        Self {
            channels,
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
        self.channels
    }
    fn output_count(&self) -> usize {
        self.channels
    }
    fn modulation_outputs(&self) -> &[ModulationOutput] {
        &[]
    }
}
