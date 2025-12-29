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
    ///
    /// For modulated parameters, safely looks up the modulation value
    /// and returns zero if the BlockId is out of bounds.
    #[inline]
    pub fn get_value(&self, modulation_values: &[S]) -> S {
        match self {
            Parameter::Constant(value) => *value,
            Parameter::Modulated(block_id) => {
                // Safe lookup to prevent panic in audio thread
                modulation_values.get(block_id.0).copied().unwrap_or(S::ZERO)
            }
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
