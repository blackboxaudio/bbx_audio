//! DC blocker for removing DC offset from audio signals.

use crate::{
    block::Block,
    context::DspContext,
    parameter::ModulationOutput,
    sample::Sample,
};

/// DC blocker using a simple one-pole high-pass filter.
///
/// Implements the difference equation:
/// y[n] = x[n] - x[n-1] + R * y[n-1]
///
/// where R is typically 0.995-0.999 for audio applications.
/// Higher R values give a lower cutoff frequency.
pub struct DcBlockerBlock<S: Sample> {
    /// Filter coefficient (R). Typically 0.995 for ~14Hz cutoff at 44.1kHz.
    coefficient: S,
    /// Previous input sample per channel.
    x_prev: Vec<S>,
    /// Previous output sample per channel.
    y_prev: Vec<S>,
    /// Number of channels.
    num_channels: usize,
}

impl<S: Sample> DcBlockerBlock<S> {
    /// Create a new DC blocker with the given coefficient.
    ///
    /// # Arguments
    /// * `coefficient` - Filter coefficient R (typically 0.995-0.999)
    /// * `num_channels` - Number of audio channels
    pub fn new(coefficient: S, num_channels: usize) -> Self {
        Self {
            coefficient,
            x_prev: vec![S::ZERO; num_channels],
            y_prev: vec![S::ZERO; num_channels],
            num_channels,
        }
    }

    /// Create a DC blocker with the default coefficient (0.995).
    pub fn default_coeff(num_channels: usize) -> Self {
        Self::new(S::from_f64(0.995), num_channels)
    }

    /// Create a DC blocker with a specific cutoff frequency.
    ///
    /// # Arguments
    /// * `cutoff_hz` - Cutoff frequency in Hz (typically 5-30 Hz)
    /// * `sample_rate` - Sample rate in Hz
    /// * `num_channels` - Number of audio channels
    pub fn with_cutoff(cutoff_hz: f64, sample_rate: f64, num_channels: usize) -> Self {
        // R = 1 - (2π * fc / fs)
        // This is an approximation that works well for low frequencies
        let coefficient = 1.0 - (2.0 * std::f64::consts::PI * cutoff_hz / sample_rate);
        let coefficient = coefficient.clamp(0.9, 0.9999);
        Self::new(S::from_f64(coefficient), num_channels)
    }

    /// Reset the filter state.
    pub fn reset(&mut self) {
        for i in 0..self.num_channels {
            self.x_prev[i] = S::ZERO;
            self.y_prev[i] = S::ZERO;
        }
    }
}

impl<S: Sample> Block<S> for DcBlockerBlock<S> {
    fn process(
        &mut self,
        inputs: &[&[S]],
        outputs: &mut [&mut [S]],
        _modulation_values: &[S],
        _context: &DspContext,
    ) {
        let channel_count = inputs.len().min(outputs.len()).min(self.num_channels);

        for channel in 0..channel_count {
            let input = inputs[channel];
            let output = &mut outputs[channel];

            for i in 0..input.len().min(output.len()) {
                let x = input[i];
                // y[n] = x[n] - x[n-1] + R * y[n-1]
                let y = x - self.x_prev[channel] + self.coefficient * self.y_prev[channel];

                self.x_prev[channel] = x;
                self.y_prev[channel] = y;
                output[i] = y;
            }
        }
    }

    #[inline]
    fn input_count(&self) -> usize {
        self.num_channels
    }

    #[inline]
    fn output_count(&self) -> usize {
        self.num_channels
    }

    #[inline]
    fn modulation_outputs(&self) -> &[ModulationOutput] {
        &[]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_context() -> DspContext {
        DspContext {
            sample_rate: 44100.0,
            buffer_size: 64,
            num_channels: 2,
            current_sample: 0,
        }
    }

    #[test]
    fn test_removes_dc_offset() {
        let mut dc_blocker = DcBlockerBlock::<f32>::default_coeff(1);
        let context = make_context();

        // Signal with DC offset
        let dc_offset = 0.5;
        let input: Vec<f32> = (0..4096).map(|_| dc_offset).collect();
        let mut output = vec![0.0f32; 4096];

        dc_blocker.process(&[&input], &mut [&mut output], &[], &context);

        // After settling, output should approach zero
        let final_samples = &output[3000..];
        let avg: f32 = final_samples.iter().sum::<f32>() / final_samples.len() as f32;

        // Should be very close to zero after the filter settles
        assert!(avg.abs() < 0.01, "DC offset not removed: avg = {}", avg);
    }

    #[test]
    fn test_passes_ac_signal() {
        let mut dc_blocker = DcBlockerBlock::<f32>::default_coeff(1);
        let context = make_context();

        // Generate a 1kHz sine wave (well above the DC blocker cutoff)
        let freq = 1000.0;
        let input: Vec<f32> = (0..4096)
            .map(|i| (2.0 * std::f32::consts::PI * freq * i as f32 / 44100.0).sin())
            .collect();
        let mut output = vec![0.0f32; 4096];

        dc_blocker.process(&[&input], &mut [&mut output], &[], &context);

        // Compare RMS of input and output (after settling)
        let input_rms: f32 = (input[2000..].iter().map(|x| x * x).sum::<f32>() / 2096.0).sqrt();
        let output_rms: f32 = (output[2000..].iter().map(|x| x * x).sum::<f32>() / 2096.0).sqrt();

        // Output should be very close to input for AC signals
        let ratio = output_rms / input_rms;
        assert!((ratio - 1.0).abs() < 0.01, "AC signal attenuated: ratio = {}", ratio);
    }

    #[test]
    fn test_reset() {
        let mut dc_blocker = DcBlockerBlock::<f32>::default_coeff(2);
        let context = make_context();

        // Process some samples
        let input = vec![1.0f32; 64];
        let mut output_left = vec![0.0f32; 64];
        let mut output_right = vec![0.0f32; 64];
        dc_blocker.process(&[&input, &input], &mut [&mut output_left, &mut output_right], &[], &context);

        // State should be non-zero
        assert!(dc_blocker.x_prev[0] != 0.0 || dc_blocker.y_prev[0] != 0.0);

        // Reset
        dc_blocker.reset();

        // State should be zero
        assert_eq!(dc_blocker.x_prev[0], 0.0);
        assert_eq!(dc_blocker.y_prev[0], 0.0);
        assert_eq!(dc_blocker.x_prev[1], 0.0);
        assert_eq!(dc_blocker.y_prev[1], 0.0);
    }

    #[test]
    fn test_with_cutoff() {
        // 20 Hz cutoff at 44.1kHz
        let dc_blocker = DcBlockerBlock::<f32>::with_cutoff(20.0, 44100.0, 1);

        // Coefficient should be close to 0.997
        let coeff = dc_blocker.coefficient.to_f64();
        assert!(coeff > 0.99 && coeff < 1.0);
    }
}
