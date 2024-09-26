use crate::sample::Sample;

/// Implemented by `Block` objects used in a DSP `Graph` to process or generate signals.
pub trait Process: std::fmt::Display {
    type Sample: Sample;

    fn process(&mut self, inputs: &Vec<Self::Sample>) -> Self::Sample;
}
