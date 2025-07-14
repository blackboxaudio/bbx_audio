use crate::sample::Sample;

pub trait Writer<S: Sample>: Send + Sync {
    fn sample_rate(&self) -> f64;
    fn num_channels(&self) -> usize;

    fn can_write(&self) -> bool;
    fn write_channel(&mut self, channel_index: usize, samples: &[S]) -> Result<(), Box<dyn std::error::Error>>;

    fn finalize(&mut self) -> Result<(), Box<dyn std::error::Error>>;
}
