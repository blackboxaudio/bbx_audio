//! Ambisonic decoder block for converting B-format to speaker layouts.

use core::marker::PhantomData;

use crate::{
    block::{Block, MAX_BLOCK_INPUTS, MAX_BLOCK_OUTPUTS},
    channel::{ChannelConfig, ChannelLayout},
    context::DspContext,
    parameter::ModulationOutput,
    sample::Sample,
};

/// Decodes ambisonics B-format to a speaker layout.
///
/// Converts SN3D normalized, ACN ordered ambisonic signals to
/// discrete speaker feeds using a mode-matching decoder.
///
/// # Supported Configurations
/// - Input: 1st order (4 ch), 2nd order (9 ch), or 3rd order (16 ch)
/// - Output: Stereo, 5.1, 7.1, or custom layouts
///
/// # Example
/// Decode first-order ambisonics to stereo using virtual speakers
/// at ±30 degrees azimuth.
pub struct AmbisonicDecoderBlock<S: Sample> {
    input_order: usize,
    output_layout: ChannelLayout,
    decoder_matrix: Box<[[f64; MAX_BLOCK_INPUTS]; MAX_BLOCK_OUTPUTS]>,
    _phantom: PhantomData<S>,
}

impl<S: Sample> AmbisonicDecoderBlock<S> {
    /// Create a new ambisonic decoder.
    ///
    /// # Arguments
    /// * `order` - Ambisonic order (1, 2, or 3)
    /// * `output_layout` - Target speaker layout
    ///
    /// # Panics
    /// Panics if order is not 1, 2, or 3.
    pub fn new(order: usize, output_layout: ChannelLayout) -> Self {
        assert!((1..=3).contains(&order), "Ambisonic order must be 1, 2, or 3");

        let mut decoder = Self {
            input_order: order,
            output_layout,
            decoder_matrix: Box::new([[0.0; MAX_BLOCK_INPUTS]; MAX_BLOCK_OUTPUTS]),
            _phantom: PhantomData,
        };
        decoder.compute_decoder_matrix();
        decoder
    }

    /// Returns the ambisonic order.
    pub fn order(&self) -> usize {
        self.input_order
    }

    /// Returns the output layout.
    pub fn output_layout(&self) -> ChannelLayout {
        self.output_layout
    }

    fn compute_decoder_matrix(&mut self) {
        let speaker_positions = self.get_speaker_positions();
        let num_speakers = self.output_layout.channel_count();
        let num_channels = self.input_channel_count();

        for (spk, &(azimuth, elevation)) in speaker_positions.iter().enumerate().take(num_speakers) {
            let coeffs = self.compute_sh_coefficients(azimuth, elevation);
            self.decoder_matrix[spk][..num_channels].copy_from_slice(&coeffs[..num_channels]);
        }

        self.normalize_decoder_matrix();
    }

    fn get_speaker_positions(&self) -> [(f64, f64); MAX_BLOCK_OUTPUTS] {
        let mut positions = [(0.0, 0.0); MAX_BLOCK_OUTPUTS];

        match self.output_layout {
            ChannelLayout::Mono => {
                positions[0] = (0.0, 0.0);
            }
            ChannelLayout::Stereo => {
                positions[0] = (30.0, 0.0); // Left
                positions[1] = (-30.0, 0.0); // Right
            }
            ChannelLayout::Surround51 => {
                positions[0] = (30.0, 0.0); // L
                positions[1] = (-30.0, 0.0); // R
                positions[2] = (0.0, 0.0); // C
                positions[3] = (0.0, 0.0); // LFE (not directional)
                positions[4] = (110.0, 0.0); // Ls
                positions[5] = (-110.0, 0.0); // Rs
            }
            ChannelLayout::Surround71 => {
                positions[0] = (30.0, 0.0); // L
                positions[1] = (-30.0, 0.0); // R
                positions[2] = (0.0, 0.0); // C
                positions[3] = (0.0, 0.0); // LFE (not directional)
                positions[4] = (90.0, 0.0); // Ls
                positions[5] = (-90.0, 0.0); // Rs
                positions[6] = (150.0, 0.0); // Lrs
                positions[7] = (-150.0, 0.0); // Rrs
            }
            ChannelLayout::Custom(n) => {
                let angle_step = 360.0 / n as f64;
                for (i, pos) in positions.iter_mut().enumerate().take(n) {
                    *pos = (i as f64 * angle_step - 90.0, 0.0);
                }
            }
            _ => {}
        }

        positions
    }

    fn compute_sh_coefficients(&self, azimuth_deg: f64, elevation_deg: f64) -> [f64; MAX_BLOCK_INPUTS] {
        let mut coeffs = [0.0; MAX_BLOCK_INPUTS];

        let az = azimuth_deg.to_radians();
        let el = elevation_deg.to_radians();

        let cos_el = el.cos();
        let sin_el = el.sin();
        let cos_az = az.cos();
        let sin_az = az.sin();

        // Order 0 (W channel)
        coeffs[0] = 1.0;

        if self.input_order >= 1 {
            // Order 1: Y, Z, X (ACN 1, 2, 3)
            coeffs[1] = cos_el * sin_az; // Y
            coeffs[2] = sin_el; // Z
            coeffs[3] = cos_el * cos_az; // X
        }

        if self.input_order >= 2 {
            // Order 2: ACN 4-8
            let cos_2az = (2.0 * az).cos();
            let sin_2az = (2.0 * az).sin();
            let sin_2el = (2.0 * el).sin();
            let cos_el_sq = cos_el * cos_el;

            coeffs[4] = 0.8660254037844386 * cos_el_sq * sin_2az; // V
            coeffs[5] = 0.8660254037844386 * sin_2el * sin_az; // T
            coeffs[6] = 0.5 * (3.0 * sin_el * sin_el - 1.0); // R
            coeffs[7] = 0.8660254037844386 * sin_2el * cos_az; // S
            coeffs[8] = 0.8660254037844386 * cos_el_sq * cos_2az; // U
        }

        if self.input_order >= 3 {
            // Order 3: ACN 9-15
            let cos_3az = (3.0 * az).cos();
            let sin_3az = (3.0 * az).sin();
            let cos_el_sq = cos_el * cos_el;
            let cos_el_cu = cos_el_sq * cos_el;
            let sin_el_sq = sin_el * sin_el;

            coeffs[9] = 0.7905694150420949 * cos_el_cu * sin_3az; // Q
            coeffs[10] = 1.9364916731037085 * cos_el_sq * sin_el * (2.0 * az).sin(); // O
            coeffs[11] = 0.6123724356957945 * cos_el * (5.0 * sin_el_sq - 1.0) * sin_az; // M
            coeffs[12] = 0.5 * sin_el * (5.0 * sin_el_sq - 3.0); // K
            coeffs[13] = 0.6123724356957945 * cos_el * (5.0 * sin_el_sq - 1.0) * cos_az; // L
            coeffs[14] = 1.9364916731037085 * cos_el_sq * sin_el * (2.0 * az).cos(); // N
            coeffs[15] = 0.7905694150420949 * cos_el_cu * cos_3az; // P
        }

        coeffs
    }

    fn normalize_decoder_matrix(&mut self) {
        let num_speakers = self.output_layout.channel_count();
        let num_channels = self.input_channel_count();

        if num_speakers == 0 {
            return;
        }

        let energy_scale = 1.0 / (num_speakers as f64).sqrt();

        for spk in 0..num_speakers {
            for ch in 0..num_channels {
                self.decoder_matrix[spk][ch] *= energy_scale;
            }
        }
    }

    fn input_channel_count(&self) -> usize {
        (self.input_order + 1) * (self.input_order + 1)
    }
}

impl<S: Sample> Block<S> for AmbisonicDecoderBlock<S> {
    fn process(&mut self, inputs: &[&[S]], outputs: &mut [&mut [S]], _modulation_values: &[S], _context: &DspContext) {
        let num_inputs = self.input_channel_count().min(inputs.len());
        let num_outputs = self.output_layout.channel_count().min(outputs.len());

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
        self.output_layout.channel_count()
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
    fn test_decoder_foa_to_stereo_channel_counts() {
        let decoder = AmbisonicDecoderBlock::<f32>::new(1, ChannelLayout::Stereo);
        assert_eq!(decoder.input_count(), 4);
        assert_eq!(decoder.output_count(), 2);
        assert_eq!(decoder.channel_config(), ChannelConfig::Explicit);
    }

    #[test]
    fn test_decoder_soa_to_51_channel_counts() {
        let decoder = AmbisonicDecoderBlock::<f32>::new(2, ChannelLayout::Surround51);
        assert_eq!(decoder.input_count(), 9);
        assert_eq!(decoder.output_count(), 6);
    }

    #[test]
    fn test_decoder_toa_to_71_channel_counts() {
        let decoder = AmbisonicDecoderBlock::<f32>::new(3, ChannelLayout::Surround71);
        assert_eq!(decoder.input_count(), 16);
        assert_eq!(decoder.output_count(), 8);
    }

    #[test]
    fn test_decoder_foa_front_signal() {
        let mut decoder = AmbisonicDecoderBlock::<f32>::new(1, ChannelLayout::Stereo);
        let context = test_context();

        // Encode a front signal: W=1, Y=0, Z=0, X=1
        let w = [1.0f32; 4];
        let y = [0.0f32; 4];
        let z = [0.0f32; 4];
        let x = [1.0f32; 4];
        let mut left_out = [0.0f32; 4];
        let mut right_out = [0.0f32; 4];

        let inputs: [&[f32]; 4] = [&w, &y, &z, &x];
        let mut outputs: [&mut [f32]; 2] = [&mut left_out, &mut right_out];

        decoder.process(&inputs, &mut outputs, &[], &context);

        // Front signal should have similar levels in L and R
        let diff = (left_out[0] - right_out[0]).abs();
        assert!(diff < 0.1, "Front signal should be balanced, diff={}", diff);
    }

    #[test]
    fn test_decoder_foa_left_signal() {
        let mut decoder = AmbisonicDecoderBlock::<f32>::new(1, ChannelLayout::Stereo);
        let context = test_context();

        // Encode a left signal: W=1, Y=1, Z=0, X=0
        let w = [1.0f32; 4];
        let y = [1.0f32; 4]; // Left
        let z = [0.0f32; 4];
        let x = [0.0f32; 4];
        let mut left_out = [0.0f32; 4];
        let mut right_out = [0.0f32; 4];

        let inputs: [&[f32]; 4] = [&w, &y, &z, &x];
        let mut outputs: [&mut [f32]; 2] = [&mut left_out, &mut right_out];

        decoder.process(&inputs, &mut outputs, &[], &context);

        // Left signal should be louder in left channel
        assert!(
            left_out[0] > right_out[0],
            "Left signal should be louder in left channel: L={}, R={}",
            left_out[0],
            right_out[0]
        );
    }

    #[test]
    #[should_panic]
    fn test_decoder_invalid_order_panics() {
        let _ = AmbisonicDecoderBlock::<f32>::new(0, ChannelLayout::Stereo);
    }

    #[test]
    #[should_panic]
    fn test_decoder_order_too_high_panics() {
        let _ = AmbisonicDecoderBlock::<f32>::new(4, ChannelLayout::Stereo);
    }
}
