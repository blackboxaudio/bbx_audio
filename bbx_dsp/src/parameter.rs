use crate::{block::BlockId, sample::Sample};

#[derive(Debug, Clone)]
pub enum Parameter<S: Sample> {
    Constant(S),
    Modulated(BlockId),
}

impl<S: Sample> Parameter<S> {
    pub fn get_value(&self, modulation_values: &[S]) -> S {
        match self {
            Parameter::Constant(value) => *value,
            Parameter::Modulated(block_id) => modulation_values[block_id.0],
        }
    }
}

#[derive(Debug, Clone)]
pub struct ModulationOutput {
    pub name: &'static str,
    pub min_value: f64,
    pub max_value: f64,
}
