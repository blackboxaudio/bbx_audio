use crate::sample::Sample;

/// Describes a component that can read audio files.
pub trait Reader<S: Sample>: Send + Sync {
    /// Get the sample rate of the audio file being read.
    fn sample_rate(&self) -> f64;

    /// Get the number of channels in the audio file being read.
    fn num_channels(&self) -> usize;

    /// Get the number of samples in the audio file being read.
    fn num_samples(&self) -> usize;

    /// Read samples from a specific channel of the audio file.
    fn read_channel(&self, channel_index: usize) -> &[S];
}
