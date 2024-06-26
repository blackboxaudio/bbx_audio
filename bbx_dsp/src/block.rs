use crate::{effector::Effector, generator::Generator, process::Process};

/// The representation of a DSP operation within a `Graph`.
pub enum Block {
    Effector(Effector),
    Generator(Generator),
}

pub struct Blockk {
    id: usize,
    inputs: Vec<usize>,
    outputs: Vec<usize>,
    operation: Box<dyn Process>,
}
