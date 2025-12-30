//! Audio file reader trait.
//!
//! This module defines the [`Reader`] trait for reading audio file data.
//! Implementations are provided by the `bbx_file` crate (e.g., WAV reader).

use crate::sample::Sample;

/// Trait for reading audio data from files.
///
/// Implementations should load audio data on construction and provide
/// access to the decoded samples via [`read_channel`](Self::read_channel).
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
