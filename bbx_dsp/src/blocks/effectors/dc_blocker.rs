//! DC offset removal filter using a simple one-pole high-pass design.

use core::marker::PhantomData;

use bbx_core::flush_denormal_f64;

use crate::{
    block::{Block, DEFAULT_EFFECTOR_INPUT_COUNT, DEFAULT_EFFECTOR_OUTPUT_COUNT, MAX_BLOCK_OUTPUTS},
    context::DspContext,
    parameter::ModulationOutput,
    sample::Sample,
};

/// A DC blocking filter that removes DC offset from audio signals.
///
/// Uses a first-order high-pass filter with approximately 5Hz cutoff.
pub struct DcBlockerBlock<S: Sample> {
    /// Whether the DC blocker is enabled.
    pub enabled: bool,

    x_prev: [f64; MAX_BLOCK_OUTPUTS],
    y_prev: [f64; MAX_BLOCK_OUTPUTS],

    // Filter coefficient (~0.995 for 5Hz at 44.1kHz)
    coeff: f64,

    _phantom: PhantomData<S>,
}

impl<S: Sample> DcBlockerBlock<S> {
    /// Create a new `DcBlockerBlock`.
    pub fn new(enabled: bool) -> Self {
        Self {
            enabled,
            x_prev: [0.0; MAX_BLOCK_OUTPUTS],
            y_prev: [0.0; MAX_BLOCK_OUTPUTS],
            coeff: 0.995, // Will be recalculated on prepare
            _phantom: PhantomData,
        }
    }

    /// Recalculate filter coefficient for the given sample rate.
    /// Targets approximately 5Hz cutoff frequency.
    pub fn set_sample_rate(&mut self, sample_rate: f64) {
        // DC blocker coefficient: R = 1 - (2 * PI * fc / fs)
        // For fc = 5Hz, this gives approximately 0.9993 at 44.1kHz
        let cutoff_hz = 5.0;
        self.coeff = 1.0 - (2.0 * S::PI.to_f64() * cutoff_hz / sample_rate);
        self.coeff = self.coeff.clamp(0.9, 0.9999);
    }

    /// Reset the filter state.
    pub fn reset(&mut self) {
        self.x_prev = [0.0; MAX_BLOCK_OUTPUTS];
        self.y_prev = [0.0; MAX_BLOCK_OUTPUTS];
    }
}

impl<S: Sample> Block<S> for DcBlockerBlock<S> {
    fn process(&mut self, inputs: &[&[S]], outputs: &mut [&mut [S]], _modulation_values: &[S], _context: &DspContext) {
        if !self.enabled {
            // Pass through unchanged
            for (ch, input) in inputs.iter().enumerate() {
                if ch < outputs.len() {
                    outputs[ch].copy_from_slice(input);
                }
            }
            return;
        }

        // Process each channel
        for (ch, input) in inputs.iter().enumerate() {
            if ch >= outputs.len() || ch >= MAX_BLOCK_OUTPUTS {
                break;
            }

            for (i, &sample) in input.iter().enumerate() {
                let x = sample.to_f64();

                // y[n] = x[n] - x[n-1] + R * y[n-1]
                let y = x - self.x_prev[ch] + self.coeff * self.y_prev[ch];

                self.x_prev[ch] = x;
                // Flush denormals to prevent CPU slowdown during quiet passages
                self.y_prev[ch] = flush_denormal_f64(y);

                outputs[ch][i] = S::from_f64(y);
            }
        }
    }

    #[inline]
    fn input_count(&self) -> usize {
        DEFAULT_EFFECTOR_INPUT_COUNT
    }

    #[inline]
    fn output_count(&self) -> usize {
        DEFAULT_EFFECTOR_OUTPUT_COUNT
    }

    #[inline]
    fn modulation_outputs(&self) -> &[ModulationOutput] {
        &[]
    }

    fn prepare(&mut self, context: &DspContext) {
        self.set_sample_rate(context.sample_rate);
        self.reset();
    }

    fn reset(&mut self) {
        self.x_prev = [0.0; MAX_BLOCK_OUTPUTS];
        self.y_prev = [0.0; MAX_BLOCK_OUTPUTS];
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::channel::ChannelLayout;

    fn test_context(buffer_size: usize) -> DspContext {
        DspContext {
            sample_rate: 44100.0,
            num_channels: 6,
            buffer_size,
            current_sample: 0,
            channel_layout: ChannelLayout::Surround51,
        }
    }

    #[test]
    fn test_dc_blocker_6_channels() {
        let mut blocker = DcBlockerBlock::<f32>::new(true);
        blocker.set_sample_rate(44100.0);
        let context = test_context(4);

        let input: [[f32; 4]; 6] = [[0.5; 4]; 6];
        let mut outputs: [[f32; 4]; 6] = [[0.0; 4]; 6];

        let input_refs: Vec<&[f32]> = input.iter().map(|ch| ch.as_slice()).collect();
        let mut output_refs: Vec<&mut [f32]> = outputs.iter_mut().map(|ch| ch.as_mut_slice()).collect();

        blocker.process(&input_refs, &mut output_refs, &[], &context);

        for ch in 0..6 {
            assert!(outputs[ch][3].abs() > 0.0, "Channel {ch} should have output");
        }
    }

    #[test]
    fn test_dc_blocker_removes_dc_offset() {
        let mut blocker = DcBlockerBlock::<f32>::new(true);
        blocker.set_sample_rate(44100.0);
        let context = test_context(1024);

        let input: [f32; 1024] = [0.5; 1024];
        let mut output: [f32; 1024] = [0.0; 1024];

        let inputs: [&[f32]; 1] = [&input];
        let mut outputs: [&mut [f32]; 1] = [&mut output];

        for _ in 0..100 {
            blocker.process(&inputs, &mut outputs, &[], &context);
        }

        let final_val = output[1023].abs();
        assert!(final_val < 0.1, "DC should be mostly removed, got {final_val}");
    }

    #[test]
    fn test_dc_blocker_input_output_counts_f32() {
        let blocker = DcBlockerBlock::<f32>::new(true);
        assert_eq!(blocker.input_count(), DEFAULT_EFFECTOR_INPUT_COUNT);
        assert_eq!(blocker.output_count(), DEFAULT_EFFECTOR_OUTPUT_COUNT);
    }

    #[test]
    fn test_dc_blocker_input_output_counts_f64() {
        let blocker = DcBlockerBlock::<f64>::new(true);
        assert_eq!(blocker.input_count(), DEFAULT_EFFECTOR_INPUT_COUNT);
        assert_eq!(blocker.output_count(), DEFAULT_EFFECTOR_OUTPUT_COUNT);
    }

    #[test]
    fn test_dc_blocker_basic_f64() {
        let mut blocker = DcBlockerBlock::<f64>::new(true);
        blocker.set_sample_rate(44100.0);
        let context = test_context(64);

        let input: [f64; 64] = [0.5; 64];
        let mut output: [f64; 64] = [0.0; 64];

        let inputs: [&[f64]; 1] = [&input];
        let mut outputs: [&mut [f64]; 1] = [&mut output];

        blocker.process(&inputs, &mut outputs, &[], &context);

        assert!(output[63].abs() > 0.0, "DC blocker should produce output");
    }

    #[test]
    fn test_dc_blocker_modulation_outputs_empty() {
        let blocker = DcBlockerBlock::<f32>::new(true);
        assert!(blocker.modulation_outputs().is_empty());
    }

    #[test]
    fn test_dc_blocker_disabled_passthrough() {
        let mut blocker = DcBlockerBlock::<f32>::new(false);
        let context = test_context(4);

        let input: [f32; 4] = [0.5, 0.6, 0.7, 0.8];
        let mut output: [f32; 4] = [0.0; 4];

        let inputs: [&[f32]; 1] = [&input];
        let mut outputs: [&mut [f32]; 1] = [&mut output];

        blocker.process(&inputs, &mut outputs, &[], &context);

        assert_eq!(output, input, "Disabled DC blocker should pass through unchanged");
    }

    #[test]
    fn test_dc_blocker_reset() {
        let mut blocker = DcBlockerBlock::<f32>::new(true);
        blocker.set_sample_rate(44100.0);
        let context = test_context(64);

        let input: [f32; 64] = [0.5; 64];
        let mut output: [f32; 64] = [0.0; 64];

        let inputs: [&[f32]; 1] = [&input];
        let mut outputs: [&mut [f32]; 1] = [&mut output];

        blocker.process(&inputs, &mut outputs, &[], &context);
        blocker.reset();

        let mut output2: [f32; 64] = [0.0; 64];
        let mut outputs2: [&mut [f32]; 1] = [&mut output2];
        blocker.process(&inputs, &mut outputs2, &[], &context);

        assert!((output[0] - output2[0]).abs() < 1e-6, "Reset should clear state");
    }
}
