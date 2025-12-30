//! Audio file writer trait.
//!
//! This module defines the [`Writer`] trait for writing audio file data.
//! Implementations are provided by the `bbx_file` crate (e.g., WAV writer).

use crate::sample::Sample;

/// Trait for writing audio data to files.
///
/// Implementations handle encoding and file format specifics. Call
/// [`finalize`](Self::finalize) when done to ensure proper file closure.
pub trait Writer<S: Sample>: Send + Sync {
    /// Get the sample rate of the writer.
    fn sample_rate(&self) -> f64;

    /// Get the number of channels of the writer.
    fn num_channels(&self) -> usize;

    /// Check whether the writer can write to audio an audio file. This
    /// is useful for particular audio files that use closing byte signatures,
    /// unlike WAV files which can usually be appended with more audio data
    /// at any time.
    fn can_write(&self) -> bool;

    /// Write samples to a specified channel.
    fn write_channel(&mut self, channel_index: usize, samples: &[S]) -> Result<(), Box<dyn std::error::Error>>;

    /// Finalize the writing of an audio file.
    fn finalize(&mut self) -> Result<(), Box<dyn std::error::Error>>;
}
