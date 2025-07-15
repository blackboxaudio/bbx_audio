use crate::{block::BlockId, sample::Sample};

/// Types of parameters that DSP blocks can use, which is
/// useful when wanting a particular parameter to be modulated
/// by a `Modulator`.
#[derive(Debug, Clone)]
pub enum Parameter<S: Sample> {
    Constant(S),
    Modulated(BlockId),
}

impl<S: Sample> Parameter<S> {
    /// Get the appropriate value for a `Parameter`.
    pub fn get_value(&self, modulation_values: &[S]) -> S {
        match self {
            Parameter::Constant(value) => *value,
            Parameter::Modulated(block_id) => modulation_values[block_id.0],
        }
    }
}

/// Used for declaring outputs of a particular
/// `Modulator` block.
#[derive(Debug, Clone)]
pub struct ModulationOutput {
    pub name: &'static str,
    pub min_value: f64,
    pub max_value: f64,
}
