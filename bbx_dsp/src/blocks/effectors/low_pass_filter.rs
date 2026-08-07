//! State Variable Filter (SVF) based low-pass filter block.

use bbx_core::flush_denormal_f64;

use crate::{
    block::{Block, DEFAULT_EFFECTOR_INPUT_COUNT, DEFAULT_EFFECTOR_OUTPUT_COUNT, MAX_BLOCK_OUTPUTS},
    context::DspContext,
    math,
    parameter::{ModulationOutput, Parameter},
    sample::Sample,
};

/// SVF-based low-pass filter for efficient, stable filtering.
///
/// Uses the TPT (Topology Preserving Transform) SVF algorithm which is:
/// - Stable at all cutoff frequencies
/// - Has no delay-free loops
/// - Maintains consistent behavior regardless of sample rate
///
/// Output is scaled by a compensation factor based on Q and cutoff frequency
/// to preserve passband gain while limiting the resonance peak (target ≤ 2.0).
pub struct LowPassFilterBlock<S: Sample> {
    /// Cutoff frequency in Hz (20-20000).
    pub cutoff: Parameter<S>,
    /// Resonance (Q factor, 0.5-10.0, default 0.707 = Butterworth).
    pub resonance: Parameter<S>,

    ic1eq: [f64; MAX_BLOCK_OUTPUTS],
    ic2eq: [f64; MAX_BLOCK_OUTPUTS],
}

impl<S: Sample> LowPassFilterBlock<S> {
    const MIN_CUTOFF: f64 = 20.0;
    const MAX_CUTOFF: f64 = 20000.0;
    const MIN_Q: f64 = 0.5;
    const MAX_Q: f64 = 10.0;

    /// Create a new low-pass filter with the given cutoff and resonance.
    pub fn new(cutoff: f64, resonance: f64) -> Self {
        Self {
            cutoff: Parameter::Constant(S::from_f64(cutoff)),
            resonance: Parameter::Constant(S::from_f64(resonance)),
            ic1eq: [0.0; MAX_BLOCK_OUTPUTS],
            ic2eq: [0.0; MAX_BLOCK_OUTPUTS],
        }
    }
}

impl<S: Sample> Block<S> for LowPassFilterBlock<S> {
    fn process(&mut self, inputs: &[&[S]], outputs: &mut [&mut [S]], modulation_values: &[S], context: &DspContext) {
        let cutoff_hz = self
            .cutoff
            .get_value(modulation_values)
            .to_f64()
            .clamp(Self::MIN_CUTOFF, Self::MAX_CUTOFF);

        let q = self
            .resonance
            .get_value(modulation_values)
            .to_f64()
            .clamp(Self::MIN_Q, Self::MAX_Q);

        let g = math::tan(S::PI.to_f64() * cutoff_hz / context.sample_rate);
        let k = 1.0 / q;
        let a1 = 1.0 / (1.0 + g * (g + k));
        let a2 = g * a1;
        let a3 = g * a2;

        let compensation = {
            let q_factor = if q <= 1.0 {
                1.0
            } else {
                let target = 2.0 / q;
                let blend = (q - 1.0).min(1.0);
                1.0 - blend * (1.0 - target)
            };

            let g_factor = if g > 1.0 {
                1.0 / (1.0 + 0.1 * (g - 1.0).min(5.0))
            } else {
                1.0
            };

            (q_factor * g_factor).clamp(0.1, 1.0)
        };

        let num_channels = inputs.len().min(outputs.len()).min(MAX_BLOCK_OUTPUTS);

        for ch in 0..num_channels {
            let input = inputs[ch];
            let output = &mut outputs[ch];

            let mut ic1 = self.ic1eq[ch];
            let mut ic2 = self.ic2eq[ch];

            for i in 0..context.buffer_size.min(input.len()).min(output.len()) {
                let v0 = input[i].to_f64();

                let v3 = v0 - ic2;
                let v1 = a1 * ic1 + a2 * v3;
                let v2 = ic2 + a2 * ic1 + a3 * v3;

                ic1 = 2.0 * v1 - ic1;
                ic2 = 2.0 * v2 - ic2;

                output[i] = S::from_f64(v2 * compensation);
            }

            self.ic1eq[ch] = flush_denormal_f64(ic1);
            self.ic2eq[ch] = flush_denormal_f64(ic2);
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

    fn prepare(&mut self, _context: &DspContext) {
        self.reset();
    }

    fn reset(&mut self) {
        self.ic1eq = [0.0; MAX_BLOCK_OUTPUTS];
        self.ic2eq = [0.0; MAX_BLOCK_OUTPUTS];
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
    fn test_low_pass_filter_6_channels() {
        let mut filter = LowPassFilterBlock::<f32>::new(1000.0, 0.707);
        let context = test_context(4);

        let input: [[f32; 4]; 6] = [[1.0; 4]; 6];
        let mut outputs: [[f32; 4]; 6] = [[0.0; 4]; 6];

        let input_refs: Vec<&[f32]> = input.iter().map(|ch| ch.as_slice()).collect();
        let mut output_refs: Vec<&mut [f32]> = outputs.iter_mut().map(|ch| ch.as_mut_slice()).collect();

        filter.process(&input_refs, &mut output_refs, &[], &context);

        for ch in 0..6 {
            assert!(outputs[ch][3].abs() > 0.0, "Channel {ch} should have output");
        }
    }

    #[test]
    fn test_low_pass_filter_independent_channels() {
        let mut filter = LowPassFilterBlock::<f32>::new(5000.0, 0.707);
        let context = test_context(64);

        let mut input: [[f32; 64]; 4] = [[0.0; 64]; 4];
        input[0] = [1.0; 64];
        input[1] = [0.0; 64];
        input[2] = [0.5; 64];
        input[3] = [-0.5; 64];

        let mut outputs: [[f32; 64]; 4] = [[0.0; 64]; 4];

        let input_refs: Vec<&[f32]> = input.iter().map(|ch| ch.as_slice()).collect();
        let mut output_refs: Vec<&mut [f32]> = outputs.iter_mut().map(|ch| ch.as_mut_slice()).collect();

        filter.process(&input_refs, &mut output_refs, &[], &context);

        assert!(outputs[0][63].abs() > outputs[1][63].abs());
        assert!(outputs[2][63].abs() < outputs[0][63].abs());
        assert!(outputs[3][63] < 0.0);
    }

    #[test]
    fn test_low_pass_filter_input_output_counts_f32() {
        let filter = LowPassFilterBlock::<f32>::new(1000.0, 0.707);
        assert_eq!(filter.input_count(), DEFAULT_EFFECTOR_INPUT_COUNT);
        assert_eq!(filter.output_count(), DEFAULT_EFFECTOR_OUTPUT_COUNT);
    }

    #[test]
    fn test_low_pass_filter_input_output_counts_f64() {
        let filter = LowPassFilterBlock::<f64>::new(1000.0, 0.707);
        assert_eq!(filter.input_count(), DEFAULT_EFFECTOR_INPUT_COUNT);
        assert_eq!(filter.output_count(), DEFAULT_EFFECTOR_OUTPUT_COUNT);
    }

    #[test]
    fn test_low_pass_filter_basic_f64() {
        let mut filter = LowPassFilterBlock::<f64>::new(1000.0, 0.707);
        let context = test_context(64);

        let input: [f64; 64] = [1.0; 64];
        let mut output: [f64; 64] = [0.0; 64];

        let inputs: [&[f64]; 1] = [&input];
        let mut outputs: [&mut [f64]; 1] = [&mut output];

        filter.process(&inputs, &mut outputs, &[], &context);

        assert!(output[63].abs() > 0.0, "Filter should produce output");
    }

    #[test]
    fn test_low_pass_filter_modulation_outputs_empty() {
        let filter = LowPassFilterBlock::<f32>::new(1000.0, 0.707);
        assert!(filter.modulation_outputs().is_empty());
    }

    #[test]
    fn test_low_pass_filter_reset() {
        let mut filter = LowPassFilterBlock::<f32>::new(1000.0, 0.707);
        let context = test_context(64);

        let input: [f32; 64] = [1.0; 64];
        let mut output: [f32; 64] = [0.0; 64];

        let inputs: [&[f32]; 1] = [&input];
        let mut outputs: [&mut [f32]; 1] = [&mut output];

        filter.process(&inputs, &mut outputs, &[], &context);
        filter.reset();

        let mut output2: [f32; 64] = [0.0; 64];
        let mut outputs2: [&mut [f32]; 1] = [&mut output2];
        filter.process(&inputs, &mut outputs2, &[], &context);

        assert!((output[0] - output2[0]).abs() < 1e-6, "Reset should clear state");
    }

    #[test]
    fn test_low_pass_filter_prepare_resets_state() {
        let mut filter = LowPassFilterBlock::<f32>::new(1000.0, 0.707);
        let context = test_context(64);

        let input: [f32; 64] = [1.0; 64];
        let mut output: [f32; 64] = [0.0; 64];

        let inputs: [&[f32]; 1] = [&input];
        let mut outputs: [&mut [f32]; 1] = [&mut output];

        filter.process(&inputs, &mut outputs, &[], &context);

        let new_context = DspContext {
            sample_rate: 48000.0,
            num_channels: 6,
            buffer_size: 64,
            current_sample: 0,
            channel_layout: ChannelLayout::Surround51,
        };
        filter.prepare(&new_context);

        let mut fresh_filter = LowPassFilterBlock::<f32>::new(1000.0, 0.707);
        let mut output_used: [f32; 64] = [0.0; 64];
        let mut output_fresh: [f32; 64] = [0.0; 64];

        let mut outputs_used: [&mut [f32]; 1] = [&mut output_used];
        let mut outputs_fresh: [&mut [f32]; 1] = [&mut output_fresh];

        filter.process(&inputs, &mut outputs_used, &[], &new_context);
        fresh_filter.process(&inputs, &mut outputs_fresh, &[], &new_context);

        assert!(
            (output_used[0] - output_fresh[0]).abs() < 1e-6,
            "After prepare(), filter should behave like a fresh filter"
        );
    }
}
