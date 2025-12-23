//! State Variable Filter (SVF) block with multiple filter modes.

use crate::{
    block::Block,
    context::DspContext,
    parameter::{ModulationOutput, ModulatableParam},
    sample::Sample,
};

/// Filter mode for the SVF.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilterMode {
    LowPass,
    HighPass,
    BandPass,
    Notch,
}

impl FilterMode {
    /// Parse filter mode from a string.
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "lowpass" | "lp" | "low" => Some(FilterMode::LowPass),
            "highpass" | "hp" | "high" => Some(FilterMode::HighPass),
            "bandpass" | "bp" | "band" => Some(FilterMode::BandPass),
            "notch" | "br" | "bandreject" => Some(FilterMode::Notch),
            _ => None,
        }
    }
}

/// State Variable Filter block.
///
/// Implements a 2-pole SVF with low-pass, high-pass, band-pass, and notch outputs.
/// Based on the Chamberlin SVF topology with Zavalishin's improvements for stability.
pub struct FilterBlock<S: Sample> {
    /// Cutoff frequency in Hz.
    pub cutoff: ModulatableParam<S>,
    /// Resonance (Q factor), typically 0.5 to 20.0.
    pub resonance: ModulatableParam<S>,
    /// Filter mode.
    pub mode: FilterMode,

    /// Per-channel state: [ic1eq, ic2eq] for each channel.
    state: Vec<[S; 2]>,
    /// Number of channels.
    num_channels: usize,
}

impl<S: Sample> FilterBlock<S> {
    /// Create a new filter block.
    ///
    /// # Arguments
    /// * `cutoff_hz` - Cutoff frequency in Hz
    /// * `resonance` - Q factor (0.5 = gentle, 10.0 = resonant)
    /// * `mode` - Filter mode
    /// * `num_channels` - Number of audio channels
    pub fn new(cutoff_hz: S, resonance: S, mode: FilterMode, num_channels: usize) -> Self {
        Self {
            cutoff: ModulatableParam::new(cutoff_hz),
            resonance: ModulatableParam::new(resonance),
            mode,
            state: vec![[S::ZERO; 2]; num_channels],
            num_channels,
        }
    }

    /// Create a low-pass filter at 1kHz.
    pub fn lowpass(cutoff_hz: S, resonance: S, num_channels: usize) -> Self {
        Self::new(cutoff_hz, resonance, FilterMode::LowPass, num_channels)
    }

    /// Create a high-pass filter.
    pub fn highpass(cutoff_hz: S, resonance: S, num_channels: usize) -> Self {
        Self::new(cutoff_hz, resonance, FilterMode::HighPass, num_channels)
    }

    /// Reset filter state.
    pub fn reset(&mut self) {
        for state in &mut self.state {
            *state = [S::ZERO; 2];
        }
    }

    /// Process a single sample through the SVF.
    ///
    /// Uses the trapezoidal integrated SVF (TPT/ZDF) for better stability.
    #[inline]
    fn process_sample(&mut self, input: S, channel: usize, g: S, k: S) -> S {
        let [ic1eq, ic2eq] = self.state[channel];

        // High-pass output
        let hp = (input - ic1eq * k - ic2eq) / (S::ONE + g * k + g * g);
        // Band-pass output (integrator 1)
        let bp = g * hp + ic1eq;
        // Low-pass output (integrator 2)
        let lp = g * bp + ic2eq;

        // Update state
        self.state[channel][0] = g * hp + bp;
        self.state[channel][1] = g * bp + lp;

        // Select output based on mode
        match self.mode {
            FilterMode::LowPass => lp,
            FilterMode::HighPass => hp,
            FilterMode::BandPass => bp,
            FilterMode::Notch => lp + hp,
        }
    }
}

impl<S: Sample> Block<S> for FilterBlock<S> {
    fn process(
        &mut self,
        inputs: &[&[S]],
        outputs: &mut [&mut [S]],
        modulation_values: &[S],
        context: &DspContext,
    ) {
        // Get modulated parameters
        let cutoff_hz = self.cutoff.evaluate(modulation_values);
        let resonance = self.resonance.evaluate(modulation_values);

        // Clamp cutoff to valid range (20 Hz to Nyquist)
        let nyquist = context.sample_rate * 0.5;
        let cutoff_clamped = cutoff_hz.to_f64().clamp(20.0, nyquist - 100.0);

        // Clamp resonance (Q)
        let q = resonance.to_f64().clamp(0.5, 20.0);

        // Calculate filter coefficients
        // g = tan(π * fc / fs) - frequency warping for bilinear transform
        let g = S::from_f64((std::f64::consts::PI * cutoff_clamped / context.sample_rate).tan());
        // k = 1/Q - damping factor
        let k = S::from_f64(1.0 / q);

        // Process each channel
        let channel_count = inputs.len().min(outputs.len()).min(self.num_channels);
        for channel in 0..channel_count {
            let input = inputs[channel];
            let output = &mut outputs[channel];

            for i in 0..input.len().min(output.len()) {
                output[i] = self.process_sample(input[i], channel, g, k);
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
    fn test_filter_mode_from_str() {
        assert_eq!(FilterMode::from_str("lowpass"), Some(FilterMode::LowPass));
        assert_eq!(FilterMode::from_str("LP"), Some(FilterMode::LowPass));
        assert_eq!(FilterMode::from_str("highpass"), Some(FilterMode::HighPass));
        assert_eq!(FilterMode::from_str("bandpass"), Some(FilterMode::BandPass));
        assert_eq!(FilterMode::from_str("notch"), Some(FilterMode::Notch));
        assert_eq!(FilterMode::from_str("invalid"), None);
    }

    #[test]
    fn test_lowpass_attenuates_high_frequencies() {
        let mut filter = FilterBlock::<f32>::lowpass(1000.0, 0.707, 1);
        let context = make_context();

        // Generate a high-frequency sine wave (10kHz)
        let freq = 10000.0;
        let input: Vec<f32> = (0..1024)
            .map(|i| (2.0 * std::f32::consts::PI * freq * i as f32 / 44100.0).sin())
            .collect();
        let mut output = vec![0.0f32; 1024];

        filter.process(&[&input], &mut [&mut output], &[], &context);

        // The high frequency should be significantly attenuated
        let input_rms: f32 = (input.iter().map(|x| x * x).sum::<f32>() / input.len() as f32).sqrt();
        let output_rms: f32 = (output[512..].iter().map(|x| x * x).sum::<f32>() / 512.0).sqrt();

        // Output RMS should be much lower than input for frequencies above cutoff
        assert!(output_rms < input_rms * 0.2);
    }

    #[test]
    fn test_lowpass_passes_low_frequencies() {
        let mut filter = FilterBlock::<f32>::lowpass(5000.0, 0.707, 1);
        let context = make_context();

        // Generate a low-frequency sine wave (100Hz)
        let freq = 100.0;
        let input: Vec<f32> = (0..1024)
            .map(|i| (2.0 * std::f32::consts::PI * freq * i as f32 / 44100.0).sin())
            .collect();
        let mut output = vec![0.0f32; 1024];

        filter.process(&[&input], &mut [&mut output], &[], &context);

        // The low frequency should pass through with minimal attenuation
        let input_rms: f32 = (input[512..].iter().map(|x| x * x).sum::<f32>() / 512.0).sqrt();
        let output_rms: f32 = (output[512..].iter().map(|x| x * x).sum::<f32>() / 512.0).sqrt();

        // Output should be close to input for frequencies well below cutoff
        assert!((output_rms - input_rms).abs() < 0.1);
    }

    #[test]
    fn test_filter_reset() {
        let mut filter = FilterBlock::<f32>::lowpass(1000.0, 0.707, 2);

        // Process some samples to build up state
        let input = vec![1.0f32; 64];
        let mut output = vec![0.0f32; 64];
        let context = make_context();

        filter.process(&[&input], &mut [&mut output], &[], &context);

        // State should be non-zero
        assert!(filter.state[0][0] != 0.0 || filter.state[0][1] != 0.0);

        // Reset
        filter.reset();

        // State should be zero
        assert_eq!(filter.state[0], [0.0, 0.0]);
        assert_eq!(filter.state[1], [0.0, 0.0]);
    }
}
