//! Binaural decoder block for converting multi-channel audio to stereo headphone output.
//!
//! Supports both ambisonic (FOA/SOA/TOA) and surround (5.1, 7.1) inputs.
//! Two decoding strategies are available:
//!
//! - [`BinauralStrategy::Matrix`] - Lightweight ILD-based approximation (low CPU)
//! - [`BinauralStrategy::Hrtf`] - Full HRTF convolution for accurate binaural rendering (default)

mod hrir_data;
mod hrtf;
mod matrix;
mod virtual_speaker;

use std::marker::PhantomData;

use hrtf::HrtfConvolver;

use crate::{
    block::Block, channel::ChannelConfig, context::DspContext, graph::MAX_BLOCK_INPUTS, parameter::ModulationOutput,
    sample::Sample,
};

/// Binaural decoding strategy.
///
/// Determines how multi-channel audio is converted to binaural stereo.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinauralStrategy {
    /// Lightweight matrix-based decoder using ILD (Interaural Level Difference).
    ///
    /// Uses psychoacoustically-informed coefficients to approximate binaural cues.
    /// Low CPU usage but limited spatial accuracy.
    Matrix,

    /// Full HRTF (Head-Related Transfer Function) convolution.
    ///
    /// Uses measured impulse responses to accurately model how sounds arrive
    /// at each ear from different directions. Higher CPU usage but superior
    /// spatial rendering with proper externalization.
    Hrtf,
}

impl Default for BinauralStrategy {
    fn default() -> Self {
        Self::Hrtf
    }
}

/// Decodes multi-channel audio to stereo for headphone listening.
///
/// Supports ambisonic B-format (1st, 2nd, 3rd order) and can be configured
/// to use either lightweight matrix decoding or full HRTF convolution.
///
/// # Supported Input Formats
/// - **Ambisonics**: FOA (4 ch), SOA (9 ch), TOA (16 ch)
/// - **Surround**: 5.1 (6 ch), 7.1 (8 ch)
///
/// # Output
/// Always stereo (2 channels): Left, Right
///
/// # Example
/// ```ignore
/// use bbx_dsp::blocks::effectors::binaural_decoder::{BinauralDecoderBlock, BinauralStrategy};
///
/// // Create HRTF decoder (default)
/// let hrtf_decoder = BinauralDecoderBlock::<f32>::new(1);
///
/// // Create lightweight matrix decoder
/// let matrix_decoder = BinauralDecoderBlock::<f32>::with_strategy(1, BinauralStrategy::Matrix);
/// ```
pub struct BinauralDecoderBlock<S: Sample> {
    input_count: usize,
    strategy: BinauralStrategy,
    decoder_matrix: [[f64; MAX_BLOCK_INPUTS]; 2],
    hrtf_convolver: Option<Box<HrtfConvolver>>,
    _phantom: PhantomData<S>,
}

impl<S: Sample> BinauralDecoderBlock<S> {
    /// Create a new binaural decoder for ambisonics with the default strategy (HRTF).
    ///
    /// # Arguments
    /// * `order` - Ambisonic order (1, 2, or 3)
    ///
    /// # Panics
    /// Panics if order is not 1, 2, or 3.
    pub fn new(order: usize) -> Self {
        Self::with_strategy(order, BinauralStrategy::default())
    }

    /// Create a new binaural decoder for ambisonics with a specific strategy.
    ///
    /// # Arguments
    /// * `order` - Ambisonic order (1, 2, or 3)
    /// * `strategy` - The decoding strategy to use
    ///
    /// # Panics
    /// Panics if order is not 1, 2, or 3.
    pub fn with_strategy(order: usize, strategy: BinauralStrategy) -> Self {
        assert!((1..=3).contains(&order), "Ambisonic order must be 1, 2, or 3");

        let input_count = (order + 1) * (order + 1);
        let decoder_matrix = matrix::compute_matrix(order);

        let hrtf_convolver = match strategy {
            BinauralStrategy::Matrix => None,
            BinauralStrategy::Hrtf => Some(Box::new(HrtfConvolver::new_ambisonic(order))),
        };

        Self {
            input_count,
            strategy,
            decoder_matrix,
            hrtf_convolver,
            _phantom: PhantomData,
        }
    }

    /// Create a new binaural decoder for surround sound.
    ///
    /// # Arguments
    /// * `channel_count` - Number of input channels (6 for 5.1, 8 for 7.1)
    /// * `strategy` - The decoding strategy to use
    ///
    /// # Panics
    /// Panics if channel_count is not 6 or 8.
    pub fn new_surround(channel_count: usize, strategy: BinauralStrategy) -> Self {
        assert!(
            channel_count == 6 || channel_count == 8,
            "Surround channel count must be 6 (5.1) or 8 (7.1)"
        );

        let decoder_matrix = [[0.0; MAX_BLOCK_INPUTS]; 2];

        let hrtf_convolver = match strategy {
            BinauralStrategy::Matrix => None,
            BinauralStrategy::Hrtf => Some(Box::new(HrtfConvolver::new_surround(channel_count))),
        };

        Self {
            input_count: channel_count,
            strategy,
            decoder_matrix,
            hrtf_convolver,
            _phantom: PhantomData,
        }
    }

    /// Returns the ambisonic order (for ambisonic inputs).
    ///
    /// Returns 0 for surround inputs.
    pub fn order(&self) -> usize {
        match self.input_count {
            4 => 1,
            9 => 2,
            16 => 3,
            _ => 0,
        }
    }

    /// Returns the current decoding strategy.
    pub fn strategy(&self) -> BinauralStrategy {
        self.strategy
    }

    /// Reset the HRTF convolver state (clears convolution buffers).
    pub fn reset(&mut self) {
        if let Some(ref mut convolver) = self.hrtf_convolver {
            convolver.reset();
        }
    }

    fn process_matrix(&self, inputs: &[&[S]], outputs: &mut [&mut [S]]) {
        let num_inputs = self.input_count.min(inputs.len());
        let num_outputs = 2.min(outputs.len());

        if num_inputs == 0 || num_outputs == 0 || inputs[0].is_empty() {
            return;
        }

        let num_samples = inputs[0].len().min(outputs[0].len());

        for (out_ch, output) in outputs.iter_mut().enumerate().take(num_outputs) {
            for i in 0..num_samples {
                let mut sum = 0.0f64;
                for (in_ch, input) in inputs.iter().enumerate().take(num_inputs) {
                    sum += input[i].to_f64() * self.decoder_matrix[out_ch][in_ch];
                }
                output[i] = S::from_f64(sum);
            }
        }
    }

    fn process_hrtf(&mut self, inputs: &[&[S]], outputs: &mut [&mut [S]]) {
        if outputs.len() < 2 {
            return;
        }

        let num_inputs = self.input_count.min(inputs.len());

        if let Some(ref mut convolver) = self.hrtf_convolver {
            let (left_output, rest) = outputs.split_at_mut(1);
            let right_output = &mut rest[0];
            convolver.process(inputs, left_output[0], right_output, num_inputs);
        }
    }
}

impl<S: Sample> Block<S> for BinauralDecoderBlock<S> {
    fn process(&mut self, inputs: &[&[S]], outputs: &mut [&mut [S]], _modulation_values: &[S], _context: &DspContext) {
        match self.strategy {
            BinauralStrategy::Matrix => self.process_matrix(inputs, outputs),
            BinauralStrategy::Hrtf => self.process_hrtf(inputs, outputs),
        }
    }

    #[inline]
    fn input_count(&self) -> usize {
        self.input_count
    }

    #[inline]
    fn output_count(&self) -> usize {
        2
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
    fn test_default_strategy_is_hrtf() {
        assert_eq!(BinauralStrategy::default(), BinauralStrategy::Hrtf);
    }

    #[test]
    fn test_new_uses_default_strategy() {
        let decoder = BinauralDecoderBlock::<f32>::new(1);
        assert_eq!(decoder.strategy(), BinauralStrategy::Hrtf);
    }

    #[test]
    fn test_with_strategy_matrix() {
        let decoder = BinauralDecoderBlock::<f32>::with_strategy(1, BinauralStrategy::Matrix);
        assert_eq!(decoder.strategy(), BinauralStrategy::Matrix);
    }

    #[test]
    fn test_binaural_foa_channel_counts() {
        let decoder = BinauralDecoderBlock::<f32>::new(1);
        assert_eq!(decoder.input_count(), 4);
        assert_eq!(decoder.output_count(), 2);
        assert_eq!(decoder.channel_config(), ChannelConfig::Explicit);
    }

    #[test]
    fn test_binaural_soa_channel_counts() {
        let decoder = BinauralDecoderBlock::<f32>::new(2);
        assert_eq!(decoder.input_count(), 9);
        assert_eq!(decoder.output_count(), 2);
    }

    #[test]
    fn test_binaural_toa_channel_counts() {
        let decoder = BinauralDecoderBlock::<f32>::new(3);
        assert_eq!(decoder.input_count(), 16);
        assert_eq!(decoder.output_count(), 2);
    }

    #[test]
    fn test_binaural_front_signal_balanced() {
        let mut decoder = BinauralDecoderBlock::<f32>::with_strategy(1, BinauralStrategy::Matrix);
        let context = test_context();

        // Front signal: W=1, Y=0, Z=0, X=1
        let w = [1.0f32; 4];
        let y = [0.0f32; 4];
        let z = [0.0f32; 4];
        let x = [1.0f32; 4];
        let mut left_out = [0.0f32; 4];
        let mut right_out = [0.0f32; 4];

        let inputs: [&[f32]; 4] = [&w, &y, &z, &x];
        let mut outputs: [&mut [f32]; 2] = [&mut left_out, &mut right_out];

        decoder.process(&inputs, &mut outputs, &[], &context);

        let diff = (left_out[0] - right_out[0]).abs();
        assert!(diff < 0.01, "Front signal should be balanced, diff={}", diff);
    }

    #[test]
    fn test_binaural_left_signal_louder_in_left() {
        let mut decoder = BinauralDecoderBlock::<f32>::with_strategy(1, BinauralStrategy::Matrix);
        let context = test_context();

        // Left signal: W=1, Y=1, Z=0, X=0
        let w = [1.0f32; 4];
        let y = [1.0f32; 4];
        let z = [0.0f32; 4];
        let x = [0.0f32; 4];
        let mut left_out = [0.0f32; 4];
        let mut right_out = [0.0f32; 4];

        let inputs: [&[f32]; 4] = [&w, &y, &z, &x];
        let mut outputs: [&mut [f32]; 2] = [&mut left_out, &mut right_out];

        decoder.process(&inputs, &mut outputs, &[], &context);

        assert!(
            left_out[0] > right_out[0],
            "Left signal should be louder in left channel: L={}, R={}",
            left_out[0],
            right_out[0]
        );
    }

    #[test]
    fn test_binaural_right_signal_louder_in_right() {
        let mut decoder = BinauralDecoderBlock::<f32>::with_strategy(1, BinauralStrategy::Matrix);
        let context = test_context();

        // Right signal: W=1, Y=-1, Z=0, X=0
        let w = [1.0f32; 4];
        let y = [-1.0f32; 4];
        let z = [0.0f32; 4];
        let x = [0.0f32; 4];
        let mut left_out = [0.0f32; 4];
        let mut right_out = [0.0f32; 4];

        let inputs: [&[f32]; 4] = [&w, &y, &z, &x];
        let mut outputs: [&mut [f32]; 2] = [&mut left_out, &mut right_out];

        decoder.process(&inputs, &mut outputs, &[], &context);

        assert!(
            right_out[0] > left_out[0],
            "Right signal should be louder in right channel: L={}, R={}",
            left_out[0],
            right_out[0]
        );
    }

    #[test]
    fn test_binaural_rear_signal_balanced() {
        let mut decoder = BinauralDecoderBlock::<f32>::with_strategy(1, BinauralStrategy::Matrix);
        let context = test_context();

        // Rear signal: W=1, Y=0, Z=0, X=-1
        let w = [1.0f32; 4];
        let y = [0.0f32; 4];
        let z = [0.0f32; 4];
        let x = [-1.0f32; 4];
        let mut left_out = [0.0f32; 4];
        let mut right_out = [0.0f32; 4];

        let inputs: [&[f32]; 4] = [&w, &y, &z, &x];
        let mut outputs: [&mut [f32]; 2] = [&mut left_out, &mut right_out];

        decoder.process(&inputs, &mut outputs, &[], &context);

        let diff = (left_out[0] - right_out[0]).abs();
        assert!(diff < 0.01, "Rear signal should be balanced, diff={}", diff);
    }

    #[test]
    fn test_binaural_silence_produces_silence() {
        let mut decoder = BinauralDecoderBlock::<f32>::with_strategy(1, BinauralStrategy::Matrix);
        let context = test_context();

        let w = [0.0f32; 4];
        let y = [0.0f32; 4];
        let z = [0.0f32; 4];
        let x = [0.0f32; 4];
        let mut left_out = [1.0f32; 4];
        let mut right_out = [1.0f32; 4];

        let inputs: [&[f32]; 4] = [&w, &y, &z, &x];
        let mut outputs: [&mut [f32]; 2] = [&mut left_out, &mut right_out];

        decoder.process(&inputs, &mut outputs, &[], &context);

        for i in 0..4 {
            assert!(left_out[i].abs() < 1e-10, "Left output should be silence");
            assert!(right_out[i].abs() < 1e-10, "Right output should be silence");
        }
    }

    #[test]
    fn test_binaural_order_accessor() {
        assert_eq!(BinauralDecoderBlock::<f32>::new(1).order(), 1);
        assert_eq!(BinauralDecoderBlock::<f32>::new(2).order(), 2);
        assert_eq!(BinauralDecoderBlock::<f32>::new(3).order(), 3);
    }

    #[test]
    #[should_panic]
    fn test_binaural_invalid_order_zero_panics() {
        let _ = BinauralDecoderBlock::<f32>::new(0);
    }

    #[test]
    #[should_panic]
    fn test_binaural_invalid_order_four_panics() {
        let _ = BinauralDecoderBlock::<f32>::new(4);
    }

    #[test]
    fn test_binaural_soa_lateral_differentiation() {
        let mut decoder = BinauralDecoderBlock::<f32>::with_strategy(2, BinauralStrategy::Matrix);
        let context = test_context();

        // Only V channel active (ACN index 4)
        let mut inputs_data: [[f32; 4]; 9] = [[0.0; 4]; 9];
        inputs_data[4] = [1.0; 4];

        let inputs: Vec<&[f32]> = inputs_data.iter().map(|a| a.as_slice()).collect();
        let mut left_out = [0.0f32; 4];
        let mut right_out = [0.0f32; 4];
        let mut outputs: [&mut [f32]; 2] = [&mut left_out, &mut right_out];

        decoder.process(&inputs, &mut outputs, &[], &context);

        // V channel should create opposite signs for L/R
        assert!(
            left_out[0] * right_out[0] < 0.0,
            "V channel should create opposite polarity: L={}, R={}",
            left_out[0],
            right_out[0]
        );
    }
}
