use crate::{effector::Effector, generator::Generator};

/// The representation of a DSP operation within a `Graph`.
pub enum Block {
    Effector(Effector),
    Generator(Generator),
}
