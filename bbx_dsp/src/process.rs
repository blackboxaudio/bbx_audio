use crate::sample::Sample;

/// Implemented by `Block` objects used in a DSP `Graph` to process or generate signals.
pub trait Process: std::fmt::Display {
    fn process(&mut self, sample: Option<Sample>) -> Sample;
}
