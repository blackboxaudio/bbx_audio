//! Audio frame type for inter-thread communication.

use bbx_core::StackVec;

/// Maximum number of interleaved samples per frame.
/// Supports up to 512 samples per channel for stereo (1024 total).
pub const MAX_FRAME_SAMPLES: usize = 1024;

/// A frame of audio data for visualization or inter-thread communication.
///
/// Uses stack-allocated storage for realtime-safe operation on audio threads.
#[derive(Clone)]
pub struct Frame {
    /// Interleaved audio samples (stack-allocated).
    pub samples: StackVec<f32, MAX_FRAME_SAMPLES>,
    /// Sample rate in Hz.
    pub sample_rate: u32,
    /// Number of audio channels.
    pub num_channels: usize,
}

impl Frame {
    /// Create a new frame from a slice of samples.
    ///
    /// # Panics
    ///
    /// Panics if `samples.len() > MAX_FRAME_SAMPLES`.
    pub fn new(samples: &[f32], sample_rate: u32, num_channels: usize) -> Self {
        assert!(
            samples.len() <= MAX_FRAME_SAMPLES,
            "Frame exceeds maximum size of {MAX_FRAME_SAMPLES} samples"
        );

        let mut stack_samples = StackVec::new();
        for &sample in samples {
            stack_samples.push_unchecked(sample);
        }

        Self {
            samples: stack_samples,
            sample_rate,
            num_channels,
        }
    }

    /// Returns an iterator over samples for a specific channel (de-interleaved).
    ///
    /// Returns `None` if the channel index is out of bounds.
    pub fn channel_samples(&self, channel: usize) -> Option<impl Iterator<Item = f32> + '_> {
        if channel >= self.num_channels || self.num_channels == 0 {
            return None;
        }
        Some(
            self.samples
                .as_slice()
                .iter()
                .skip(channel)
                .step_by(self.num_channels)
                .copied(),
        )
    }

    /// Get the number of samples per channel.
    pub fn samples_per_channel(&self) -> usize {
        if self.num_channels == 0 {
            0
        } else {
            self.samples.len() / self.num_channels
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_frame_new() {
        let samples = [0.5f32, -0.5, 0.25, -0.25];
        let frame = Frame::new(&samples, 44100, 2);

        assert_eq!(frame.samples.len(), 4);
        assert_eq!(frame.sample_rate, 44100);
        assert_eq!(frame.num_channels, 2);
    }

    #[test]
    fn test_channel_samples() {
        let samples = [1.0f32, 2.0, 3.0, 4.0, 5.0, 6.0];
        let frame = Frame::new(&samples, 44100, 2);

        let left: Vec<f32> = frame.channel_samples(0).unwrap().collect();
        let right: Vec<f32> = frame.channel_samples(1).unwrap().collect();

        assert_eq!(left, vec![1.0, 3.0, 5.0]);
        assert_eq!(right, vec![2.0, 4.0, 6.0]);
    }

    #[test]
    fn test_channel_samples_invalid() {
        let samples = [1.0f32, 2.0];
        let frame = Frame::new(&samples, 44100, 2);

        assert!(frame.channel_samples(2).is_none());
        assert!(frame.channel_samples(10).is_none());
    }

    #[test]
    fn test_samples_per_channel() {
        let samples = [1.0f32, 2.0, 3.0, 4.0, 5.0, 6.0];
        let frame = Frame::new(&samples, 44100, 2);

        assert_eq!(frame.samples_per_channel(), 3);
    }

    #[test]
    fn test_samples_per_channel_zero_channels() {
        let frame = Frame {
            samples: {
                let mut s = StackVec::new();
                s.push_unchecked(1.0);
                s.push_unchecked(2.0);
                s
            },
            sample_rate: 44100,
            num_channels: 0,
        };

        assert_eq!(frame.samples_per_channel(), 0);
    }
}
