//! HRTF convolution engine for binaural rendering.
//!
//! Performs time-domain convolution of virtual speaker signals with HRIRs
//! to produce binaural stereo output.

use super::{
    hrir_data::{HRIR_LENGTH, get_hrir_for_azimuth},
    virtual_speaker::{MAX_HRIR_LENGTH, MAX_VIRTUAL_SPEAKERS, VirtualSpeaker, layouts},
};
use crate::{block::MAX_BLOCK_INPUTS, math, sample::Sample};

/// HRTF convolution engine with pre-allocated buffers.
///
/// Maintains circular buffers for each virtual speaker's signal history
/// and performs time-domain convolution with HRIRs.
pub struct HrtfConvolver {
    /// Circular buffers for each virtual speaker's signal history.
    signal_buffers: [[f32; MAX_HRIR_LENGTH]; MAX_VIRTUAL_SPEAKERS],

    /// Current write position in circular buffers.
    buffer_pos: usize,

    /// Virtual speaker configurations.
    speakers: [Option<VirtualSpeakerConfig>; MAX_VIRTUAL_SPEAKERS],

    /// Number of active virtual speakers.
    num_speakers: usize,

    /// Actual HRIR length being used.
    hrir_length: usize,
}

/// Internal speaker configuration with owned HRIR references.
#[derive(Clone)]
struct VirtualSpeakerConfig {
    /// Spherical harmonic weights for decoding.
    sh_weights: [f64; MAX_BLOCK_INPUTS],

    /// Left ear HRIR (borrowed from static data).
    left_hrir: &'static [f32],

    /// Right ear HRIR (borrowed from static data).
    right_hrir: &'static [f32],
}

impl HrtfConvolver {
    /// Create a new HRTF convolver for ambisonic decoding.
    ///
    /// # Arguments
    /// * `ambisonic_order` - Ambisonic order (1, 2, or 3) for SH coefficient calculation
    pub fn new_ambisonic(ambisonic_order: usize) -> Self {
        let positions = layouts::FOA_POSITIONS;
        let num_speakers = positions.len();

        let mut speakers: [Option<VirtualSpeakerConfig>; MAX_VIRTUAL_SPEAKERS] = [const { None }; MAX_VIRTUAL_SPEAKERS];

        for (i, &(azimuth, elevation)) in positions.iter().enumerate() {
            let hrir = get_hrir_for_azimuth(azimuth);
            let speaker = VirtualSpeaker::new(azimuth, elevation, ambisonic_order, hrir.left, hrir.right);

            speakers[i] = Some(VirtualSpeakerConfig {
                sh_weights: speaker.sh_weights,
                left_hrir: speaker.left_hrir,
                right_hrir: speaker.right_hrir,
            });
        }

        Self {
            signal_buffers: [[0.0; MAX_HRIR_LENGTH]; MAX_VIRTUAL_SPEAKERS],
            buffer_pos: 0,
            speakers,
            num_speakers,
            hrir_length: HRIR_LENGTH,
        }
    }

    /// Create a new HRTF convolver for surround sound decoding.
    ///
    /// # Arguments
    /// * `channel_count` - Number of input channels (6 for 5.1, 8 for 7.1)
    pub fn new_surround(channel_count: usize) -> Self {
        let positions = match channel_count {
            6 => &layouts::SURROUND_51_POSITIONS[..],
            8 => &layouts::SURROUND_71_POSITIONS[..],
            _ => panic!("Unsupported surround channel count: {channel_count}"),
        };

        let mut speakers: [Option<VirtualSpeakerConfig>; MAX_VIRTUAL_SPEAKERS] = [const { None }; MAX_VIRTUAL_SPEAKERS];

        for (i, &(azimuth, _elevation)) in positions.iter().enumerate() {
            let hrir = get_hrir_for_azimuth(azimuth);

            // For surround, each channel maps directly to a speaker (no SH decoding)
            let mut sh_weights = [0.0; MAX_BLOCK_INPUTS];
            sh_weights[i] = 1.0;

            speakers[i] = Some(VirtualSpeakerConfig {
                sh_weights,
                left_hrir: hrir.left,
                right_hrir: hrir.right,
            });
        }

        Self {
            signal_buffers: [[0.0; MAX_HRIR_LENGTH]; MAX_VIRTUAL_SPEAKERS],
            buffer_pos: 0,
            speakers,
            num_speakers: channel_count,
            hrir_length: HRIR_LENGTH,
        }
    }

    /// Reset all convolution buffers to zero.
    pub fn reset(&mut self) {
        for buffer in &mut self.signal_buffers {
            buffer.fill(0.0);
        }
        self.buffer_pos = 0;
    }

    /// Process a buffer of samples through HRTF convolution.
    ///
    /// # Arguments
    /// * `inputs` - Input sample buffers (one per ambisonic/surround channel)
    /// * `left_output` - Left ear output buffer
    /// * `right_output` - Right ear output buffer
    /// * `num_input_channels` - Number of valid input channels
    pub fn process<S: Sample>(
        &mut self,
        inputs: &[&[S]],
        left_output: &mut [S],
        right_output: &mut [S],
        num_input_channels: usize,
    ) {
        if inputs.is_empty() || left_output.is_empty() || right_output.is_empty() {
            return;
        }

        let num_samples = inputs[0].len().min(left_output.len()).min(right_output.len());

        for sample_idx in 0..num_samples {
            // Decode input channels to f64 for processing
            let mut input_samples = [0.0f64; MAX_BLOCK_INPUTS];
            for (ch, input) in inputs.iter().enumerate().take(num_input_channels) {
                input_samples[ch] = input[sample_idx].to_f64();
            }

            let (left, right) = self.process_sample(&input_samples, num_input_channels);

            left_output[sample_idx] = S::from_f64(left);
            right_output[sample_idx] = S::from_f64(right);
        }
    }

    /// Process a single sample through all virtual speakers.
    ///
    /// Returns (left, right) output samples.
    #[inline]
    fn process_sample(&mut self, inputs: &[f64; MAX_BLOCK_INPUTS], num_channels: usize) -> (f64, f64) {
        let mut left_sum = 0.0f64;
        let mut right_sum = 0.0f64;

        for speaker_idx in 0..self.num_speakers {
            if let Some(ref speaker) = self.speakers[speaker_idx] {
                // Decode input to this speaker's position
                let signal = self.decode_to_speaker(inputs, num_channels, &speaker.sh_weights);

                // Store in circular buffer
                self.signal_buffers[speaker_idx][self.buffer_pos] = signal as f32;

                // Convolve with left HRIR
                left_sum += self.convolve(speaker_idx, speaker.left_hrir);

                // Convolve with right HRIR
                right_sum += self.convolve(speaker_idx, speaker.right_hrir);
            }
        }

        // Advance buffer position
        self.buffer_pos = (self.buffer_pos + 1) % self.hrir_length;

        // Normalize by number of speakers to prevent clipping
        let normalization = 1.0 / math::sqrt(self.num_speakers as f64);
        (left_sum * normalization, right_sum * normalization)
    }

    /// Decode input channels to a speaker position using SH weights.
    #[inline]
    fn decode_to_speaker(
        &self,
        inputs: &[f64; MAX_BLOCK_INPUTS],
        num_channels: usize,
        sh_weights: &[f64; MAX_BLOCK_INPUTS],
    ) -> f64 {
        let mut sum = 0.0;
        for ch in 0..num_channels {
            sum += inputs[ch] * sh_weights[ch];
        }
        sum
    }

    /// Perform time-domain convolution with HRIR.
    ///
    /// Uses the circular buffer to efficiently convolve the signal history
    /// with the HRIR coefficients.
    #[inline]
    fn convolve(&self, speaker_idx: usize, hrir: &[f32]) -> f64 {
        let buffer = &self.signal_buffers[speaker_idx];
        let len = self.hrir_length.min(hrir.len());

        let mut sum = 0.0f64;

        // Convolve: output[n] = sum(buffer[n-k] * hrir[k])
        // buffer_pos points to the most recent sample
        for (k, &h) in hrir.iter().enumerate().take(len) {
            // Calculate buffer index going backwards from current position
            let buf_idx = (self.buffer_pos + self.hrir_length - k) % self.hrir_length;
            sum += buffer[buf_idx] as f64 * h as f64;
        }

        sum
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_ambisonic_foa() {
        let convolver = HrtfConvolver::new_ambisonic(1);
        assert_eq!(convolver.num_speakers, 8);
        assert_eq!(convolver.hrir_length, HRIR_LENGTH);
    }

    #[test]
    fn test_new_surround_51() {
        let convolver = HrtfConvolver::new_surround(6);
        assert_eq!(convolver.num_speakers, 6);
    }

    #[test]
    fn test_new_surround_71() {
        let convolver = HrtfConvolver::new_surround(8);
        assert_eq!(convolver.num_speakers, 8);
    }

    #[test]
    fn test_reset() {
        let mut convolver = HrtfConvolver::new_ambisonic(1);

        // Process some samples
        let input = [1.0f32; 16];
        let inputs: [&[f32]; 4] = [&input, &input, &input, &input];
        let mut left = [0.0f32; 16];
        let mut right = [0.0f32; 16];
        convolver.process(&inputs, &mut left, &mut right, 4);

        // Reset
        convolver.reset();

        // Verify buffers are cleared
        for buffer in &convolver.signal_buffers {
            for &sample in buffer.iter() {
                assert_eq!(sample, 0.0);
            }
        }
        assert_eq!(convolver.buffer_pos, 0);
    }

    #[test]
    fn test_process_silence() {
        let mut convolver = HrtfConvolver::new_ambisonic(1);

        let input = [0.0f32; 16];
        let inputs: [&[f32]; 4] = [&input, &input, &input, &input];
        let mut left = [1.0f32; 16];
        let mut right = [1.0f32; 16];

        convolver.process(&inputs, &mut left, &mut right, 4);

        // Output should be near silence
        for &sample in &left {
            assert!(sample.abs() < 1e-6, "Left should be silence");
        }
        for &sample in &right {
            assert!(sample.abs() < 1e-6, "Right should be silence");
        }
    }

    #[test]
    fn test_process_produces_output() {
        let mut convolver = HrtfConvolver::new_ambisonic(1);

        // Input with W channel active (omnidirectional)
        let w_input = [1.0f32; 16];
        let zero_input = [0.0f32; 16];
        let inputs: [&[f32]; 4] = [&w_input, &zero_input, &zero_input, &zero_input];
        let mut left = [0.0f32; 16];
        let mut right = [0.0f32; 16];

        convolver.process(&inputs, &mut left, &mut right, 4);

        // Should produce some output
        let left_energy: f32 = left.iter().map(|x| x * x).sum();
        let right_energy: f32 = right.iter().map(|x| x * x).sum();

        assert!(left_energy > 0.0, "Left should have output");
        assert!(right_energy > 0.0, "Right should have output");
    }

    #[test]
    fn test_left_signal_louder_in_left() {
        let mut convolver = HrtfConvolver::new_ambisonic(1);
        convolver.reset();

        // Left-biased ambisonic signal: W for omnidirectional, Y for lateral
        // Use a longer buffer to capture full HRIR response
        const LEN: usize = 512;
        let w_input = [0.5f32; LEN];
        let y_input = [1.0f32; LEN]; // Positive Y = left (stronger than W for clear bias)
        let zero_input = [0.0f32; LEN];
        let inputs: [&[f32]; 4] = [&w_input, &y_input, &zero_input, &zero_input];
        let mut left = [0.0f32; LEN];
        let mut right = [0.0f32; LEN];

        convolver.process(&inputs, &mut left, &mut right, 4);

        // Skip HRIR length to get steady-state response
        let left_energy: f32 = left[256..].iter().map(|x| x * x).sum();
        let right_energy: f32 = right[256..].iter().map(|x| x * x).sum();

        assert!(
            left_energy > right_energy,
            "Left signal should be louder in left ear: L={}, R={}",
            left_energy,
            right_energy
        );
    }
}
