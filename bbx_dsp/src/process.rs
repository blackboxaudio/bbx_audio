/// Implemented by `Block` objects used in a DSP `Graph` to process or generate signals.
pub trait Process<T> {
    fn process(&mut self, sample: Option<T>) -> T;
}
