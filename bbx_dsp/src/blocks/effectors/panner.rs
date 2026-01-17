//! Multi-format spatial panning block.
//!
//! Supports stereo panning, surround (VBAP), and ambisonic encoding.

#[cfg(feature = "simd")]
use std::simd::{StdFloat, f64x4, num::SimdFloat};

#[cfg(feature = "simd")]
use crate::sample::SIMD_LANES;
use crate::{
    block::Block,
    channel::{ChannelConfig, ChannelLayout},
    context::DspContext,
    graph::MAX_BLOCK_OUTPUTS,
    parameter::{ModulationOutput, Parameter},
    sample::Sample,
    smoothing::LinearSmoothedValue,
};

/// Maximum buffer size for stack-allocated gain arrays.
const MAX_BUFFER_SIZE: usize = 4096;

/// Panning mode determining the algorithm and output format.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PannerMode {
    /// Stereo constant-power panning.
    #[default]
    Stereo,
    /// VBAP-based surround panning.
    Surround,
    /// Spherical harmonic ambisonic encoding.
    Ambisonic,
}

/// A spatial panning block supporting stereo, surround, and ambisonic formats.
///
/// # Stereo Mode
/// Position ranges from -100 (full left) to +100 (full right).
/// Uses constant-power pan law for smooth transitions.
///
/// # Surround Mode
/// Uses Vector Base Amplitude Panning (VBAP) with azimuth and elevation.
/// Supports 5.1 and 7.1 speaker layouts.
///
/// # Ambisonic Mode
/// Encodes mono input to SN3D normalized, ACN ordered B-format.
/// Supports 1st through 3rd order (4, 9, or 16 channels).
pub struct PannerBlock<S: Sample> {
    /// Pan position: -100 (left) to +100 (right). Used in stereo mode.
    pub position: Parameter<S>,

    /// Source azimuth in degrees (-180 to +180). Used in surround/ambisonic modes.
    /// 0 = front, 90 = left, -90 = right, 180/-180 = rear.
    pub azimuth: Parameter<S>,

    /// Source elevation in degrees (-90 to +90). Used in surround/ambisonic modes.
    /// 0 = horizon, 90 = above, -90 = below.
    pub elevation: Parameter<S>,

    mode: PannerMode,
    output_layout: ChannelLayout,

    position_smoother: LinearSmoothedValue<S>,
    azimuth_smoother: LinearSmoothedValue<S>,
    elevation_smoother: LinearSmoothedValue<S>,

    speaker_azimuths: [f64; MAX_BLOCK_OUTPUTS],
}

impl<S: Sample> PannerBlock<S> {
    /// Create a new stereo panner with the given position (-100 to +100).
    pub fn new(position: f64) -> Self {
        Self::new_stereo(position)
    }

    /// Create a new stereo panner with the given position (-100 to +100).
    pub fn new_stereo(position: f64) -> Self {
        let pos = S::from_f64(position);
        Self {
            position: Parameter::Constant(pos),
            azimuth: Parameter::Constant(S::ZERO),
            elevation: Parameter::Constant(S::ZERO),
            mode: PannerMode::Stereo,
            output_layout: ChannelLayout::Stereo,
            position_smoother: LinearSmoothedValue::new(pos),
            azimuth_smoother: LinearSmoothedValue::new(S::ZERO),
            elevation_smoother: LinearSmoothedValue::new(S::ZERO),
            speaker_azimuths: [0.0; MAX_BLOCK_OUTPUTS],
        }
    }

    /// Create a centered stereo panner.
    pub fn centered() -> Self {
        Self::new_stereo(0.0)
    }

    /// Create a surround panner for the given layout.
    ///
    /// Supports `Surround51` and `Surround71` layouts.
    /// Uses VBAP (Vector Base Amplitude Panning) algorithm.
    pub fn new_surround(layout: ChannelLayout) -> Self {
        let mut panner = Self {
            position: Parameter::Constant(S::ZERO),
            azimuth: Parameter::Constant(S::ZERO),
            elevation: Parameter::Constant(S::ZERO),
            mode: PannerMode::Surround,
            output_layout: layout,
            position_smoother: LinearSmoothedValue::new(S::ZERO),
            azimuth_smoother: LinearSmoothedValue::new(S::ZERO),
            elevation_smoother: LinearSmoothedValue::new(S::ZERO),
            speaker_azimuths: [0.0; MAX_BLOCK_OUTPUTS],
        };
        panner.initialize_speaker_positions();
        panner
    }

    /// Create an ambisonic encoder for the given order (1-3).
    ///
    /// - Order 1: First-order ambisonics (4 channels: W, Y, Z, X)
    /// - Order 2: Second-order ambisonics (9 channels)
    /// - Order 3: Third-order ambisonics (16 channels)
    ///
    /// Uses SN3D normalization and ACN channel ordering.
    pub fn new_ambisonic(order: usize) -> Self {
        let layout = ChannelLayout::from_ambisonic_order(order).unwrap_or(ChannelLayout::AmbisonicFoa);

        Self {
            position: Parameter::Constant(S::ZERO),
            azimuth: Parameter::Constant(S::ZERO),
            elevation: Parameter::Constant(S::ZERO),
            mode: PannerMode::Ambisonic,
            output_layout: layout,
            position_smoother: LinearSmoothedValue::new(S::ZERO),
            azimuth_smoother: LinearSmoothedValue::new(S::ZERO),
            elevation_smoother: LinearSmoothedValue::new(S::ZERO),
            speaker_azimuths: [0.0; MAX_BLOCK_OUTPUTS],
        }
    }

    /// Initialize speaker positions for the current surround layout.
    fn initialize_speaker_positions(&mut self) {
        match self.output_layout {
            ChannelLayout::Surround51 => {
                // 5.1: L, R, C, LFE, Ls, Rs
                self.speaker_azimuths[0] = 30.0; // L
                self.speaker_azimuths[1] = -30.0; // R
                self.speaker_azimuths[2] = 0.0; // C
                self.speaker_azimuths[3] = 0.0; // LFE (not directional)
                self.speaker_azimuths[4] = 110.0; // Ls
                self.speaker_azimuths[5] = -110.0; // Rs
            }
            ChannelLayout::Surround71 => {
                // 7.1: L, R, C, LFE, Ls, Rs, Lrs, Rrs
                self.speaker_azimuths[0] = 30.0; // L
                self.speaker_azimuths[1] = -30.0; // R
                self.speaker_azimuths[2] = 0.0; // C
                self.speaker_azimuths[3] = 0.0; // LFE (not directional)
                self.speaker_azimuths[4] = 90.0; // Ls
                self.speaker_azimuths[5] = -90.0; // Rs
                self.speaker_azimuths[6] = 150.0; // Lrs
                self.speaker_azimuths[7] = -150.0; // Rrs
            }
            _ => {}
        }
    }

    /// Calculate left and right gains using constant power pan law.
    #[inline]
    fn calculate_stereo_gains(&self, position: f64) -> (f64, f64) {
        let normalized = (position + 100.0) / 200.0;
        let normalized = normalized.clamp(0.0, 1.0);

        let angle = normalized * S::FRAC_PI_2.to_f64();
        let left_gain = angle.cos();
        let right_gain = angle.sin();

        (left_gain, right_gain)
    }

    /// Calculate VBAP gains for surround panning.
    fn calculate_vbap_gains(&self, azimuth_deg: f64, _elevation_deg: f64, gains: &mut [f64; MAX_BLOCK_OUTPUTS]) {
        let num_speakers = self.output_layout.channel_count();
        let azimuth_rad = azimuth_deg.to_radians();

        for (i, gain) in gains.iter_mut().enumerate().take(num_speakers) {
            if i == 3
                && matches!(
                    self.output_layout,
                    ChannelLayout::Surround51 | ChannelLayout::Surround71
                )
            {
                *gain = 0.0;
                continue;
            }

            let speaker_rad = self.speaker_azimuths[i].to_radians();
            let angle_diff = (azimuth_rad - speaker_rad).abs();
            let angle_diff = if angle_diff > std::f64::consts::PI {
                2.0 * std::f64::consts::PI - angle_diff
            } else {
                angle_diff
            };

            let spread = std::f64::consts::FRAC_PI_2;
            if angle_diff < spread {
                *gain = ((spread - angle_diff) / spread * std::f64::consts::FRAC_PI_2).cos();
                *gain = gain.max(0.0);
            } else {
                *gain = 0.0;
            }
        }

        let sum_sq: f64 = gains.iter().take(num_speakers).map(|g| g * g).sum();
        if sum_sq > 1e-10 {
            let norm = 1.0 / sum_sq.sqrt();
            for gain in gains.iter_mut().take(num_speakers) {
                *gain *= norm;
            }
        }
    }

    /// Calculate SN3D normalized spherical harmonic coefficients for ambisonic encoding.
    fn calculate_ambisonic_gains(&self, azimuth_deg: f64, elevation_deg: f64, gains: &mut [f64; MAX_BLOCK_OUTPUTS]) {
        let az = azimuth_deg.to_radians();
        let el = elevation_deg.to_radians();

        let cos_el = el.cos();
        let sin_el = el.sin();
        let cos_az = az.cos();
        let sin_az = az.sin();

        let order = self.output_layout.ambisonic_order().unwrap_or(1);

        // Order 0 (W channel)
        gains[0] = 1.0; // W

        if order >= 1 {
            // Order 1: Y, Z, X (ACN 1, 2, 3)
            gains[1] = cos_el * sin_az; // Y
            gains[2] = sin_el; // Z
            gains[3] = cos_el * cos_az; // X
        }

        if order >= 2 {
            // Order 2: ACN 4-8
            let cos_2az = (2.0 * az).cos();
            let sin_2az = (2.0 * az).sin();
            let sin_2el = (2.0 * el).sin();
            let cos_el_sq = cos_el * cos_el;

            gains[4] = 0.8660254037844386 * cos_el_sq * sin_2az; // V
            gains[5] = 0.8660254037844386 * sin_2el * sin_az; // T
            gains[6] = 0.5 * (3.0 * sin_el * sin_el - 1.0); // R
            gains[7] = 0.8660254037844386 * sin_2el * cos_az; // S
            gains[8] = 0.8660254037844386 * cos_el_sq * cos_2az; // U
        }

        if order >= 3 {
            // Order 3: ACN 9-15
            let cos_3az = (3.0 * az).cos();
            let sin_3az = (3.0 * az).sin();
            let cos_el_sq = cos_el * cos_el;
            let cos_el_cu = cos_el_sq * cos_el;
            let sin_el_sq = sin_el * sin_el;

            gains[9] = 0.7905694150420949 * cos_el_cu * sin_3az; // Q
            gains[10] = 1.9364916731037085 * cos_el_sq * sin_el * (2.0 * az).sin(); // O
            gains[11] = 0.6123724356957945 * cos_el * (5.0 * sin_el_sq - 1.0) * sin_az; // M
            gains[12] = 0.5 * sin_el * (5.0 * sin_el_sq - 3.0); // K
            gains[13] = 0.6123724356957945 * cos_el * (5.0 * sin_el_sq - 1.0) * cos_az; // L
            gains[14] = 1.9364916731037085 * cos_el_sq * sin_el * (2.0 * az).cos(); // N
            gains[15] = 0.7905694150420949 * cos_el_cu * cos_3az; // P
        }
    }

    fn process_stereo(&mut self, inputs: &[&[S]], outputs: &mut [&mut [S]], modulation_values: &[S]) {
        if inputs.is_empty() || outputs.is_empty() {
            return;
        }

        let target_position = self.position.get_value(modulation_values);
        if (target_position - self.position_smoother.target()).abs() > S::EPSILON {
            self.position_smoother.set_target_value(target_position);
        }

        let mono_in = inputs[0];
        let has_stereo_output = outputs.len() > 1;

        let num_samples = mono_in
            .len()
            .min(outputs.first().map(|o| o.len()).unwrap_or(0))
            .min(MAX_BUFFER_SIZE);

        let mut positions: [f64; MAX_BUFFER_SIZE] = [0.0; MAX_BUFFER_SIZE];
        for position in positions.iter_mut().take(num_samples) {
            *position = self.position_smoother.get_next_value().to_f64();
        }

        let mut left_gains: [S; MAX_BUFFER_SIZE] = [S::ZERO; MAX_BUFFER_SIZE];
        let mut right_gains: [S; MAX_BUFFER_SIZE] = [S::ZERO; MAX_BUFFER_SIZE];

        #[cfg(feature = "simd")]
        {
            let chunks = num_samples / SIMD_LANES;
            let remainder_start = chunks * SIMD_LANES;

            for chunk_idx in 0..chunks {
                let offset = chunk_idx * SIMD_LANES;
                let pos_vec = f64x4::from_slice(&positions[offset..]);

                let normalized = ((pos_vec + f64x4::splat(100.0)) / f64x4::splat(200.0))
                    .simd_clamp(f64x4::splat(0.0), f64x4::splat(1.0));

                let angle = normalized * f64x4::splat(S::FRAC_PI_2.to_f64());

                let l_arr = angle.cos().to_array();
                let r_arr = angle.sin().to_array();
                for i in 0..SIMD_LANES {
                    left_gains[offset + i] = S::from_f64(l_arr[i]);
                    right_gains[offset + i] = S::from_f64(r_arr[i]);
                }
            }

            for i in remainder_start..num_samples {
                let (l, r) = self.calculate_stereo_gains(positions[i]);
                left_gains[i] = S::from_f64(l);
                right_gains[i] = S::from_f64(r);
            }
        }

        #[cfg(not(feature = "simd"))]
        {
            for i in 0..num_samples {
                let (l, r) = self.calculate_stereo_gains(positions[i]);
                left_gains[i] = S::from_f64(l);
                right_gains[i] = S::from_f64(r);
            }
        }

        #[cfg(feature = "simd")]
        {
            let chunks = num_samples / SIMD_LANES;
            let remainder_start = chunks * SIMD_LANES;

            for chunk_idx in 0..chunks {
                let offset = chunk_idx * SIMD_LANES;

                let input = S::simd_from_slice(&mono_in[offset..]);
                let l_gain = S::simd_from_slice(&left_gains[offset..]);
                let l_out = input * l_gain;
                outputs[0][offset..offset + SIMD_LANES].copy_from_slice(&S::simd_to_array(l_out));

                if has_stereo_output {
                    let r_gain = S::simd_from_slice(&right_gains[offset..]);
                    let r_out = input * r_gain;
                    outputs[1][offset..offset + SIMD_LANES].copy_from_slice(&S::simd_to_array(r_out));
                }
            }

            for i in remainder_start..num_samples {
                outputs[0][i] = mono_in[i] * left_gains[i];
                if has_stereo_output {
                    outputs[1][i] = mono_in[i] * right_gains[i];
                }
            }
        }

        #[cfg(not(feature = "simd"))]
        {
            for i in 0..num_samples {
                outputs[0][i] = mono_in[i] * left_gains[i];
                if has_stereo_output {
                    outputs[1][i] = mono_in[i] * right_gains[i];
                }
            }
        }
    }

    fn process_surround(&mut self, inputs: &[&[S]], outputs: &mut [&mut [S]], modulation_values: &[S]) {
        if inputs.is_empty() || outputs.is_empty() {
            return;
        }

        let target_azimuth = self.azimuth.get_value(modulation_values);
        let target_elevation = self.elevation.get_value(modulation_values);

        if (target_azimuth - self.azimuth_smoother.target()).abs() > S::EPSILON {
            self.azimuth_smoother.set_target_value(target_azimuth);
        }
        if (target_elevation - self.elevation_smoother.target()).abs() > S::EPSILON {
            self.elevation_smoother.set_target_value(target_elevation);
        }

        let mono_in = inputs[0];
        let num_outputs = self.output_layout.channel_count().min(outputs.len());
        let num_samples = mono_in.len().min(outputs[0].len()).min(MAX_BUFFER_SIZE);

        let mut gains: [f64; MAX_BLOCK_OUTPUTS] = [0.0; MAX_BLOCK_OUTPUTS];

        for (i, &input_sample) in mono_in.iter().enumerate().take(num_samples) {
            let azimuth = self.azimuth_smoother.get_next_value().to_f64();
            let elevation = self.elevation_smoother.get_next_value().to_f64();

            self.calculate_vbap_gains(azimuth, elevation, &mut gains);

            for ch in 0..num_outputs {
                outputs[ch][i] = input_sample * S::from_f64(gains[ch]);
            }
        }
    }

    fn process_ambisonic(&mut self, inputs: &[&[S]], outputs: &mut [&mut [S]], modulation_values: &[S]) {
        if inputs.is_empty() || outputs.is_empty() {
            return;
        }

        let target_azimuth = self.azimuth.get_value(modulation_values);
        let target_elevation = self.elevation.get_value(modulation_values);

        if (target_azimuth - self.azimuth_smoother.target()).abs() > S::EPSILON {
            self.azimuth_smoother.set_target_value(target_azimuth);
        }
        if (target_elevation - self.elevation_smoother.target()).abs() > S::EPSILON {
            self.elevation_smoother.set_target_value(target_elevation);
        }

        let mono_in = inputs[0];
        let num_outputs = self.output_layout.channel_count().min(outputs.len());
        let num_samples = mono_in.len().min(outputs[0].len()).min(MAX_BUFFER_SIZE);

        let mut gains: [f64; MAX_BLOCK_OUTPUTS] = [0.0; MAX_BLOCK_OUTPUTS];

        for (i, &input_sample) in mono_in.iter().enumerate().take(num_samples) {
            let azimuth = self.azimuth_smoother.get_next_value().to_f64();
            let elevation = self.elevation_smoother.get_next_value().to_f64();

            self.calculate_ambisonic_gains(azimuth, elevation, &mut gains);

            for ch in 0..num_outputs {
                outputs[ch][i] = input_sample * S::from_f64(gains[ch]);
            }
        }
    }
}

impl<S: Sample> Block<S> for PannerBlock<S> {
    fn process(&mut self, inputs: &[&[S]], outputs: &mut [&mut [S]], modulation_values: &[S], _context: &DspContext) {
        match self.mode {
            PannerMode::Stereo => self.process_stereo(inputs, outputs, modulation_values),
            PannerMode::Surround => self.process_surround(inputs, outputs, modulation_values),
            PannerMode::Ambisonic => self.process_ambisonic(inputs, outputs, modulation_values),
        }
    }

    #[inline]
    fn input_count(&self) -> usize {
        1
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

    fn set_smoothing(&mut self, sample_rate: f64, ramp_time_ms: f64) {
        self.position_smoother.reset(sample_rate, ramp_time_ms);
        self.azimuth_smoother.reset(sample_rate, ramp_time_ms);
        self.elevation_smoother.reset(sample_rate, ramp_time_ms);
    }

    fn prepare(&mut self, context: &DspContext) {
        self.position_smoother.reset(context.sample_rate, 10.0);
        self.azimuth_smoother.reset(context.sample_rate, 10.0);
        self.elevation_smoother.reset(context.sample_rate, 10.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::DspContext;

    fn test_context(buffer_size: usize) -> DspContext {
        DspContext {
            sample_rate: 44100.0,
            num_channels: 2,
            buffer_size,
            current_sample: 0,
            channel_layout: ChannelLayout::Stereo,
        }
    }

    #[test]
    fn test_stereo_panner_centered() {
        let mut panner = PannerBlock::<f32>::centered();
        let context = test_context(4);

        let input = [1.0f32; 4];
        let mut left_out = [0.0f32; 4];
        let mut right_out = [0.0f32; 4];

        let inputs: [&[f32]; 1] = [&input];
        let mut outputs: [&mut [f32]; 2] = [&mut left_out, &mut right_out];

        panner.process(&inputs, &mut outputs, &[], &context);

        let expected_gain = (std::f32::consts::FRAC_PI_4).cos();
        for i in 0..4 {
            assert!((left_out[i] - expected_gain).abs() < 0.01);
            assert!((right_out[i] - expected_gain).abs() < 0.01);
        }
    }

    #[test]
    fn test_stereo_panner_full_left() {
        let mut panner = PannerBlock::<f32>::new(-100.0);
        let context = test_context(4);

        let input = [1.0f32; 4];
        let mut left_out = [0.0f32; 4];
        let mut right_out = [0.0f32; 4];

        let inputs: [&[f32]; 1] = [&input];
        let mut outputs: [&mut [f32]; 2] = [&mut left_out, &mut right_out];

        panner.process(&inputs, &mut outputs, &[], &context);

        for i in 0..4 {
            assert!((left_out[i] - 1.0).abs() < 0.01);
            assert!(right_out[i].abs() < 0.01);
        }
    }

    #[test]
    fn test_surround_panner_output_count() {
        let panner_51 = PannerBlock::<f32>::new_surround(ChannelLayout::Surround51);
        assert_eq!(panner_51.output_count(), 6);

        let panner_71 = PannerBlock::<f32>::new_surround(ChannelLayout::Surround71);
        assert_eq!(panner_71.output_count(), 8);
    }

    #[test]
    fn test_ambisonic_panner_output_count() {
        let panner_foa = PannerBlock::<f32>::new_ambisonic(1);
        assert_eq!(panner_foa.output_count(), 4);

        let panner_soa = PannerBlock::<f32>::new_ambisonic(2);
        assert_eq!(panner_soa.output_count(), 9);

        let panner_toa = PannerBlock::<f32>::new_ambisonic(3);
        assert_eq!(panner_toa.output_count(), 16);
    }

    #[test]
    fn test_panner_input_count_is_mono() {
        let stereo = PannerBlock::<f32>::centered();
        let surround = PannerBlock::<f32>::new_surround(ChannelLayout::Surround51);
        let ambisonic = PannerBlock::<f32>::new_ambisonic(1);

        assert_eq!(stereo.input_count(), 1);
        assert_eq!(surround.input_count(), 1);
        assert_eq!(ambisonic.input_count(), 1);
    }

    #[test]
    fn test_panner_channel_config_is_explicit() {
        let stereo = PannerBlock::<f32>::centered();
        let surround = PannerBlock::<f32>::new_surround(ChannelLayout::Surround51);
        let ambisonic = PannerBlock::<f32>::new_ambisonic(1);

        assert_eq!(stereo.channel_config(), ChannelConfig::Explicit);
        assert_eq!(surround.channel_config(), ChannelConfig::Explicit);
        assert_eq!(ambisonic.channel_config(), ChannelConfig::Explicit);
    }

    #[test]
    fn test_ambisonic_front_encoding() {
        let mut panner = PannerBlock::<f32>::new_ambisonic(1);
        let context = test_context(1);

        let input = [1.0f32];
        let mut w = [0.0f32];
        let mut y = [0.0f32];
        let mut z = [0.0f32];
        let mut x = [0.0f32];

        let inputs: [&[f32]; 1] = [&input];
        let mut outputs: [&mut [f32]; 4] = [&mut w, &mut y, &mut z, &mut x];

        panner.process(&inputs, &mut outputs, &[], &context);

        assert!((w[0] - 1.0).abs() < 0.01, "W should be 1.0 for front");
        assert!(y[0].abs() < 0.01, "Y should be 0 for front");
        assert!(z[0].abs() < 0.01, "Z should be 0 for horizon");
        assert!((x[0] - 1.0).abs() < 0.01, "X should be 1.0 for front");
    }
}
