//! Channel-wise audio mixer block.

use core::marker::PhantomData;

use crate::{
    block::{Block, MAX_BLOCK_INPUTS},
    channel::ChannelConfig,
    context::DspContext,
    math,
    parameter::ModulationOutput,
    sample::Sample,
};

/// Normalization strategy for summed signals.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum NormalizationStrategy {
    /// Divide by number of sources (average).
    Average,
    /// Divide by sqrt(N) for constant power summing.
    #[default]
    ConstantPower,
}

/// A channel-wise audio mixer that sums multiple sources per output channel.
///
/// Unlike [`MatrixMixerBlock`](super::matrix_mixer::MatrixMixerBlock) which requires explicit
/// gain setup, `MixerBlock` automatically groups inputs by channel and sums them. This is
/// useful for combining multiple audio sources into a single stereo (or N-channel) output.
///
/// # Input Organization
///
/// Inputs are organized in groups, where each group represents one source's contribution to
/// all output channels. For stereo output with 3 sources:
/// - Inputs 0, 1: Source A (L, R)
/// - Inputs 2, 3: Source B (L, R)
/// - Inputs 4, 5: Source C (L, R)
///
/// The mixer sums: Output L = A.L + B.L + C.L, Output R = A.R + B.R + C.R
pub struct MixerBlock<S: Sample> {
    num_sources: usize,
    num_channels: usize,
    normalization: NormalizationStrategy,
    _phantom: PhantomData<S>,
}

impl<S: Sample> MixerBlock<S> {
    /// Create a new mixer for the given number of sources and output channels.
    ///
    /// # Arguments
    /// * `num_sources` - Number of sources to mix (each provides all channels)
    /// * `num_channels` - Number of output channels (e.g., 2 for stereo)
    ///
    /// # Panics
    /// Panics if the total input count exceeds MAX_BLOCK_INPUTS (16).
    pub fn new(num_sources: usize, num_channels: usize) -> Self {
        assert!(num_sources > 0, "Must have at least one source");
        assert!(num_channels > 0, "Must have at least one channel");
        assert!(
            num_sources * num_channels <= MAX_BLOCK_INPUTS,
            "Total inputs {} exceeds MAX_BLOCK_INPUTS {}",
            num_sources * num_channels,
            MAX_BLOCK_INPUTS
        );
        Self {
            num_sources,
            num_channels,
            normalization: NormalizationStrategy::ConstantPower,
            _phantom: PhantomData,
        }
    }

    /// Create a stereo mixer for the given number of sources.
    pub fn stereo(num_sources: usize) -> Self {
        Self::new(num_sources, 2)
    }

    /// Set the normalization strategy.
    pub fn with_normalization(mut self, normalization: NormalizationStrategy) -> Self {
        self.normalization = normalization;
        self
    }

    /// Returns the number of sources being mixed.
    pub fn num_sources(&self) -> usize {
        self.num_sources
    }

    /// Returns the number of output channels.
    pub fn num_channels(&self) -> usize {
        self.num_channels
    }
}

impl<S: Sample> Block<S> for MixerBlock<S> {
    fn process(&mut self, inputs: &[&[S]], outputs: &mut [&mut [S]], _modulation_values: &[S], _context: &DspContext) {
        let num_channels = self.num_channels.min(outputs.len());
        if num_channels == 0 || inputs.is_empty() {
            return;
        }

        let num_samples = outputs[0].len();
        let normalization_factor = match self.normalization {
            NormalizationStrategy::Average => S::from_f64(1.0 / self.num_sources as f64),
            NormalizationStrategy::ConstantPower => S::from_f64(1.0 / math::sqrt(self.num_sources as f64)),
        };

        for (ch, output) in outputs.iter_mut().enumerate().take(num_channels) {
            for sample in output.iter_mut().take(num_samples) {
                *sample = S::ZERO;
            }

            for source_idx in 0..self.num_sources {
                let input_idx = source_idx * self.num_channels + ch;
                if let Some(input) = inputs.get(input_idx) {
                    let len = num_samples.min(input.len());
                    for i in 0..len {
                        output[i] += input[i];
                    }
                }
            }

            for sample in output.iter_mut().take(num_samples) {
                *sample *= normalization_factor;
            }
        }
    }

    #[inline]
    fn input_count(&self) -> usize {
        self.num_sources * self.num_channels
    }

    #[inline]
    fn output_count(&self) -> usize {
        self.num_channels
    }

    #[inline]
    fn modulation_outputs(&self) -> &[ModulationOutput] {
        &[]
    }

    #[inline]
    fn channel_config(&self) -> ChannelConfig {
        ChannelConfig::Explicit
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::channel::ChannelLayout;

    fn test_context() -> DspContext {
        DspContext {
            sample_rate: 44100.0,
            num_channels: 2,
            buffer_size: 4,
            current_sample: 0,
            channel_layout: ChannelLayout::Stereo,
        }
    }

    #[test]
    fn test_mixer_two_stereo_sources() {
        let mut mixer = MixerBlock::<f32>::stereo(2);
        let context = test_context();

        // Source 0: L=[1,1,1,1], R=[2,2,2,2]
        // Source 1: L=[3,3,3,3], R=[4,4,4,4]
        let src0_l = [1.0f32; 4];
        let src0_r = [2.0f32; 4];
        let src1_l = [3.0f32; 4];
        let src1_r = [4.0f32; 4];
        let mut out_l = [0.0f32; 4];
        let mut out_r = [0.0f32; 4];

        let inputs: [&[f32]; 4] = [&src0_l, &src0_r, &src1_l, &src1_r];
        let mut outputs: [&mut [f32]; 2] = [&mut out_l, &mut out_r];

        mixer.process(&inputs, &mut outputs, &[], &context);

        // Default is ConstantPower: L = (1+3)/sqrt(2), R = (2+4)/sqrt(2)
        let sqrt2 = 2.0_f32.sqrt();
        for &sample in &out_l {
            assert!((sample - 4.0 / sqrt2).abs() < 1e-6);
        }
        for &sample in &out_r {
            assert!((sample - 6.0 / sqrt2).abs() < 1e-6);
        }
    }

    #[test]
    fn test_mixer_average_normalization() {
        let mut mixer = MixerBlock::<f32>::stereo(2).with_normalization(NormalizationStrategy::Average);
        let context = test_context();

        let src0_l = [2.0f32; 4];
        let src0_r = [4.0f32; 4];
        let src1_l = [2.0f32; 4];
        let src1_r = [4.0f32; 4];
        let mut out_l = [0.0f32; 4];
        let mut out_r = [0.0f32; 4];

        let inputs: [&[f32]; 4] = [&src0_l, &src0_r, &src1_l, &src1_r];
        let mut outputs: [&mut [f32]; 2] = [&mut out_l, &mut out_r];

        mixer.process(&inputs, &mut outputs, &[], &context);

        // Expected: L = (2+2)/2 = 2, R = (4+4)/2 = 4
        for &sample in &out_l {
            assert!((sample - 2.0).abs() < 1e-6);
        }
        for &sample in &out_r {
            assert!((sample - 4.0).abs() < 1e-6);
        }
    }

    #[test]
    fn test_mixer_constant_power_normalization() {
        let mut mixer = MixerBlock::<f32>::stereo(4).with_normalization(NormalizationStrategy::ConstantPower);
        let context = test_context();

        // 4 sources, all with value 1.0
        let inputs: Vec<[f32; 4]> = vec![[1.0f32; 4]; 8]; // 4 sources * 2 channels
        let input_refs: Vec<&[f32]> = inputs.iter().map(|a| a.as_slice()).collect();
        let mut out_l = [0.0f32; 4];
        let mut out_r = [0.0f32; 4];
        let mut outputs: [&mut [f32]; 2] = [&mut out_l, &mut out_r];

        mixer.process(&input_refs, &mut outputs, &[], &context);

        // Expected: (1+1+1+1) / sqrt(4) = 4/2 = 2
        for &sample in &out_l {
            assert!((sample - 2.0).abs() < 1e-6);
        }
    }

    #[test]
    fn test_mixer_mono_three_sources() {
        let mut mixer = MixerBlock::<f32>::new(3, 1);
        let context = test_context();

        let src0 = [1.0f32; 4];
        let src1 = [2.0f32; 4];
        let src2 = [3.0f32; 4];
        let mut output = [0.0f32; 4];

        let inputs: [&[f32]; 3] = [&src0, &src1, &src2];
        let mut outputs: [&mut [f32]; 1] = [&mut output];

        mixer.process(&inputs, &mut outputs, &[], &context);

        // Default is ConstantPower: (1+2+3) / sqrt(3)
        let expected = 6.0 / 3.0_f32.sqrt();
        for &sample in &output {
            assert!((sample - expected).abs() < 1e-6);
        }
    }

    #[test]
    fn test_mixer_input_output_counts() {
        let mixer = MixerBlock::<f32>::new(3, 2);
        assert_eq!(mixer.input_count(), 6); // 3 sources * 2 channels
        assert_eq!(mixer.output_count(), 2);
        assert_eq!(mixer.channel_config(), ChannelConfig::Explicit);
    }

    #[test]
    fn test_mixer_f64() {
        let mut mixer = MixerBlock::<f64>::stereo(2);
        let context = test_context();

        let src0_l = [0.5f64; 4];
        let src0_r = [0.25f64; 4];
        let src1_l = [0.5f64; 4];
        let src1_r = [0.25f64; 4];
        let mut out_l = [0.0f64; 4];
        let mut out_r = [0.0f64; 4];

        let inputs: [&[f64]; 4] = [&src0_l, &src0_r, &src1_l, &src1_r];
        let mut outputs: [&mut [f64]; 2] = [&mut out_l, &mut out_r];

        mixer.process(&inputs, &mut outputs, &[], &context);

        // Default is ConstantPower: (0.5+0.5)/sqrt(2) and (0.25+0.25)/sqrt(2)
        let sqrt2 = 2.0_f64.sqrt();
        for &sample in &out_l {
            assert!((sample - 1.0 / sqrt2).abs() < 1e-12);
        }
        for &sample in &out_r {
            assert!((sample - 0.5 / sqrt2).abs() < 1e-12);
        }
    }

    #[test]
    #[should_panic(expected = "Must have at least one source")]
    fn test_mixer_zero_sources_panics() {
        let _ = MixerBlock::<f32>::new(0, 2);
    }

    #[test]
    #[should_panic(expected = "Must have at least one channel")]
    fn test_mixer_zero_channels_panics() {
        let _ = MixerBlock::<f32>::new(2, 0);
    }

    #[test]
    #[should_panic(expected = "exceeds MAX_BLOCK_INPUTS")]
    fn test_mixer_exceeds_max_inputs_panics() {
        let _ = MixerBlock::<f32>::new(9, 2); // 9*2 = 18 > 16
    }
}
