use crate::sample::Sample;

pub trait Reader<S: Sample>: Send + Sync {
    fn sample_rate(&self) -> f64;
    fn num_channels(&self) -> usize;
    fn num_samples(&self) -> usize;

    fn read_channel(&self, channel_index: usize) -> &[S];
}
