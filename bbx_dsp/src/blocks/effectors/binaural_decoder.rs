//! Binaural decoder block for converting B-format to stereo headphone output.

use std::marker::PhantomData;

use crate::{
    block::Block, channel::ChannelConfig, context::DspContext, graph::MAX_BLOCK_INPUTS, parameter::ModulationOutput,
    sample::Sample,
};

/// Decodes ambisonics B-format to stereo for headphone listening.
///
/// Uses psychoacoustically-informed matrix coefficients to approximate
/// binaural cues (primarily ILD - Interaural Level Difference) without
/// HRTF convolution. This provides a lightweight, realtime-safe decoder
/// suitable for basic spatial audio reproduction on headphones.
///
/// # Supported Input Orders
/// - 1st order (FOA): 4 channels (W, Y, Z, X)
/// - 2nd order (SOA): 9 channels
/// - 3rd order (TOA): 16 channels
///
/// # Output
/// Always stereo (2 channels): Left, Right
pub struct BinauralDecoderBlock<S: Sample> {
    input_order: usize,
    decoder_matrix: [[f64; MAX_BLOCK_INPUTS]; 2],
    _phantom: PhantomData<S>,
}

impl<S: Sample> BinauralDecoderBlock<S> {
    /// Create a new binaural decoder for the given ambisonic order.
    ///
    /// # Arguments
    /// * `order` - Ambisonic order (1, 2, or 3)
    ///
    /// # Panics
    /// Panics if order is not 1, 2, or 3.
    pub fn new(order: usize) -> Self {
        assert!((1..=3).contains(&order), "Ambisonic order must be 1, 2, or 3");

        let mut decoder = Self {
            input_order: order,
            decoder_matrix: [[0.0; MAX_BLOCK_INPUTS]; 2],
            _phantom: PhantomData,
        };
        decoder.compute_binaural_matrix();
        decoder
    }

    /// Returns the ambisonic order.
    pub fn order(&self) -> usize {
        self.input_order
    }

    fn input_channel_count(&self) -> usize {
        (self.input_order + 1) * (self.input_order + 1)
    }

    fn compute_binaural_matrix(&mut self) {
        match self.input_order {
            1 => self.compute_foa_matrix(),
            2 => self.compute_soa_matrix(),
            3 => self.compute_toa_matrix(),
            _ => unreachable!(),
        }
        self.normalize_matrix();
    }

    fn compute_foa_matrix(&mut self) {
        // ACN ordering: W(0), Y(1), Z(2), X(3)
        // Y is the lateral channel (positive = left, negative = right)
        // X is front-back (positive = front)
        // Z is up-down (minimal contribution for binaural)

        // Left ear
        self.decoder_matrix[0][0] = 0.5; // W (omnidirectional)
        self.decoder_matrix[0][1] = 0.5; // Y (positive for left)
        self.decoder_matrix[0][2] = 0.1; // Z (small vertical contribution)
        self.decoder_matrix[0][3] = 0.35; // X (front emphasis for externalization)

        // Right ear
        self.decoder_matrix[1][0] = 0.5; // W
        self.decoder_matrix[1][1] = -0.5; // Y (negative for right)
        self.decoder_matrix[1][2] = 0.1; // Z
        self.decoder_matrix[1][3] = 0.35; // X
    }

    fn compute_soa_matrix(&mut self) {
        // Start with scaled FOA components
        self.decoder_matrix[0][0] = 0.45; // W
        self.decoder_matrix[0][1] = 0.45; // Y
        self.decoder_matrix[0][2] = 0.09; // Z
        self.decoder_matrix[0][3] = 0.32; // X

        self.decoder_matrix[1][0] = 0.45;
        self.decoder_matrix[1][1] = -0.45;
        self.decoder_matrix[1][2] = 0.09;
        self.decoder_matrix[1][3] = 0.32;

        // Order 2 channels (ACN 4-8): V, T, R, S, U
        // V (4) - lateral diagonal, contributes to ILD
        // T (5) - vertical-lateral interaction
        // R (6) - pure vertical
        // S (7) - vertical-frontal interaction
        // U (8) - front-back differentiation

        self.decoder_matrix[0][4] = 0.25; // V (left positive)
        self.decoder_matrix[0][5] = 0.08; // T
        self.decoder_matrix[0][6] = 0.05; // R
        self.decoder_matrix[0][7] = 0.08; // S
        self.decoder_matrix[0][8] = 0.15; // U

        self.decoder_matrix[1][4] = -0.25; // V (right negative)
        self.decoder_matrix[1][5] = 0.08;
        self.decoder_matrix[1][6] = 0.05;
        self.decoder_matrix[1][7] = 0.08;
        self.decoder_matrix[1][8] = 0.15;
    }

    fn compute_toa_matrix(&mut self) {
        // Left ear coefficients for all 16 channels
        let left: [f64; 16] = [
            // Order 0-1
            0.42, 0.42, 0.08, 0.30, // Order 2 (V, T, R, S, U)
            0.22, 0.07, 0.04, 0.07, 0.13, // Order 3 (Q, O, M, K, L, N, P)
            0.15, 0.10, 0.05, 0.03, 0.05, 0.10, 0.10,
        ];

        // Right ear: negate lateral components (Y-like channels)
        let right: [f64; 16] = [
            // Order 0-1 (Y negated)
            0.42, -0.42, 0.08, 0.30, // Order 2 (V negated)
            -0.22, 0.07, 0.04, 0.07, 0.13, // Order 3 (Q, O, N negated - lateral components)
            -0.15, -0.10, 0.05, 0.03, 0.05, -0.10, 0.10,
        ];

        self.decoder_matrix[0].copy_from_slice(&left);
        self.decoder_matrix[1].copy_from_slice(&right);
    }

    fn normalize_matrix(&mut self) {
        let energy_scale = 1.0 / 2.0_f64.sqrt();
        let num_channels = self.input_channel_count();

        for ear in 0..2 {
            for ch in 0..num_channels {
                self.decoder_matrix[ear][ch] *= energy_scale;
            }
        }
    }
}

impl<S: Sample> Block<S> for BinauralDecoderBlock<S> {
    fn process(&mut self, inputs: &[&[S]], outputs: &mut [&mut [S]], _modulation_values: &[S], _context: &DspContext) {
        let num_inputs = self.input_channel_count().min(inputs.len());
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

    #[inline]
    fn input_count(&self) -> usize {
        self.input_channel_count()
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
        let mut decoder = BinauralDecoderBlock::<f32>::new(1);
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
        let mut decoder = BinauralDecoderBlock::<f32>::new(1);
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
        let mut decoder = BinauralDecoderBlock::<f32>::new(1);
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
        let mut decoder = BinauralDecoderBlock::<f32>::new(1);
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
        let mut decoder = BinauralDecoderBlock::<f32>::new(1);
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
        let mut decoder = BinauralDecoderBlock::<f32>::new(2);
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
