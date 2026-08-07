//! Gain control block with dB input.

#[cfg(feature = "simd")]
use bbx_core::simd::apply_gain;

use crate::{
    block::{Block, DEFAULT_EFFECTOR_INPUT_COUNT, DEFAULT_EFFECTOR_OUTPUT_COUNT},
    context::DspContext,
    math,
    parameter::{ModulationOutput, Parameter},
    sample::Sample,
    smoothing::LinearSmoothedValue,
};

/// Maximum buffer size for stack-allocated smoothing cache.
const MAX_BUFFER_SIZE: usize = 4096;

/// A gain control block that applies amplitude scaling.
///
/// Level is specified in decibels (dB).
pub struct GainBlock<S: Sample> {
    /// Gain level in dB (-80 to +30).
    pub level_db: Parameter<S>,

    /// Base gain multiplier (linear) applied statically to the signal.
    pub base_gain: S,

    /// Smoothed linear gain value for click-free parameter changes.
    gain_smoother: LinearSmoothedValue<S>,
}

impl<S: Sample> GainBlock<S> {
    /// Minimum gain in dB (silence threshold).
    const MIN_DB: f64 = -80.0;
    /// Maximum gain in dB.
    const MAX_DB: f64 = 30.0;

    /// Create a new `GainBlock` with the given level in dB and an optional base gain multiplier.
    pub fn new(level_db: f64, base_gain: Option<f64>) -> Self {
        let clamped_db = level_db.clamp(Self::MIN_DB, Self::MAX_DB);
        let initial_gain = Self::db_to_linear(clamped_db);

        Self {
            level_db: Parameter::Constant(S::from_f64(level_db)),
            base_gain: S::from_f64(base_gain.unwrap_or(1.0)),
            gain_smoother: LinearSmoothedValue::new(S::from_f64(initial_gain)),
        }
    }

    /// Create a unity gain (0 dB) block.
    pub fn unity() -> Self {
        Self::new(0.0, None)
    }

    /// Convert dB to linear gain with range clamping.
    #[inline]
    fn db_to_linear(db: f64) -> f64 {
        let clamped = db.clamp(Self::MIN_DB, Self::MAX_DB);
        math::powf(10.0_f64, clamped / 20.0)
    }
}

impl<S: Sample> Block<S> for GainBlock<S> {
    fn process(&mut self, inputs: &[&[S]], outputs: &mut [&mut [S]], modulation_values: &[S], context: &DspContext) {
        let level_db = self.level_db.get_value(modulation_values).to_f64();
        let target_gain = S::from_f64(Self::db_to_linear(level_db));

        let current_target = self.gain_smoother.target();
        if (target_gain - current_target).abs() > S::EPSILON {
            self.gain_smoother.set_target_value(target_gain);
        }

        let num_channels = inputs.len().min(outputs.len());

        if !self.gain_smoother.is_smoothing() {
            let gain = self.gain_smoother.current() * self.base_gain;

            #[cfg(feature = "simd")]
            {
                for ch in 0..num_channels {
                    let len = inputs[ch].len().min(outputs[ch].len());
                    apply_gain(&inputs[ch][..len], &mut outputs[ch][..len], gain);
                }
                return;
            }

            #[cfg(not(feature = "simd"))]
            {
                for ch in 0..num_channels {
                    let len = inputs[ch].len().min(outputs[ch].len());
                    for i in 0..len {
                        outputs[ch][i] = inputs[ch][i] * gain;
                    }
                }
                return;
            }
        }

        let len = inputs.first().map_or(0, |ch| ch.len().min(context.buffer_size));
        debug_assert!(len <= MAX_BUFFER_SIZE, "buffer_size exceeds MAX_BUFFER_SIZE");

        let mut gain_values: [S; MAX_BUFFER_SIZE] = [S::ZERO; MAX_BUFFER_SIZE];
        for gain_value in gain_values.iter_mut().take(len) {
            *gain_value = self.gain_smoother.get_next_value() * self.base_gain;
        }

        for ch in 0..num_channels {
            let ch_len = inputs[ch].len().min(outputs[ch].len()).min(len);
            for (i, &gain) in gain_values.iter().enumerate().take(ch_len) {
                outputs[ch][i] = inputs[ch][i] * gain;
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

    fn set_smoothing(&mut self, sample_rate: f64, ramp_time_ms: f64) {
        self.gain_smoother.reset(sample_rate, ramp_time_ms);
    }

    fn prepare(&mut self, context: &DspContext) {
        self.gain_smoother.reset(context.sample_rate, 10.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::channel::ChannelLayout;

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
    fn test_gain_input_output_counts_f32() {
        let gain = GainBlock::<f32>::new(0.0, None);
        assert_eq!(gain.input_count(), DEFAULT_EFFECTOR_INPUT_COUNT);
        assert_eq!(gain.output_count(), DEFAULT_EFFECTOR_OUTPUT_COUNT);
    }

    #[test]
    fn test_gain_input_output_counts_f64() {
        let gain = GainBlock::<f64>::new(0.0, None);
        assert_eq!(gain.input_count(), DEFAULT_EFFECTOR_INPUT_COUNT);
        assert_eq!(gain.output_count(), DEFAULT_EFFECTOR_OUTPUT_COUNT);
    }

    #[test]
    fn test_unity_gain_passthrough_f32() {
        let mut gain = GainBlock::<f32>::unity();
        let context = test_context(4);

        let input = [0.5f32, -0.5, 0.25, -0.25];
        let mut output = [0.0f32; 4];

        let inputs: [&[f32]; 1] = [&input];
        let mut outputs: [&mut [f32]; 1] = [&mut output];

        for _ in 0..10 {
            gain.process(&inputs, &mut outputs, &[], &context);
        }

        for (i, (&inp, &out)) in input.iter().zip(output.iter()).enumerate() {
            assert!(
                (inp - out).abs() < 1e-5,
                "Unity gain should passthrough: input[{}]={}, output[{}]={}",
                i,
                inp,
                i,
                out
            );
        }
    }

    #[test]
    fn test_unity_gain_passthrough_f64() {
        let mut gain = GainBlock::<f64>::unity();
        let context = test_context(4);

        let input = [0.5f64, -0.5, 0.25, -0.25];
        let mut output = [0.0f64; 4];

        let inputs: [&[f64]; 1] = [&input];
        let mut outputs: [&mut [f64]; 1] = [&mut output];

        for _ in 0..10 {
            gain.process(&inputs, &mut outputs, &[], &context);
        }

        for (i, (&inp, &out)) in input.iter().zip(output.iter()).enumerate() {
            assert!(
                (inp - out).abs() < 1e-10,
                "Unity gain should passthrough: input[{}]={}, output[{}]={}",
                i,
                inp,
                i,
                out
            );
        }
    }

    #[test]
    fn test_silence_at_min_db_f32() {
        let mut gain = GainBlock::<f32>::new(-80.0, None);
        let context = test_context(4);

        let input = [1.0f32; 4];
        let mut output = [1.0f32; 4];

        let inputs: [&[f32]; 1] = [&input];
        let mut outputs: [&mut [f32]; 1] = [&mut output];

        for _ in 0..10 {
            gain.process(&inputs, &mut outputs, &[], &context);
        }

        for (i, &out) in output.iter().enumerate() {
            assert!(
                out.abs() < 0.001,
                "Output should be nearly silent at -80dB: output[{}]={}",
                i,
                out
            );
        }
    }

    #[test]
    fn test_silence_at_min_db_f64() {
        let mut gain = GainBlock::<f64>::new(-80.0, None);
        let context = test_context(4);

        let input = [1.0f64; 4];
        let mut output = [1.0f64; 4];

        let inputs: [&[f64]; 1] = [&input];
        let mut outputs: [&mut [f64]; 1] = [&mut output];

        for _ in 0..10 {
            gain.process(&inputs, &mut outputs, &[], &context);
        }

        for (i, &out) in output.iter().enumerate() {
            assert!(
                out.abs() < 0.001,
                "Output should be nearly silent at -80dB: output[{}]={}",
                i,
                out
            );
        }
    }

    #[test]
    fn test_amplification_at_positive_db_f32() {
        let mut gain = GainBlock::<f32>::new(6.0, None);
        let context = test_context(4);

        let input = [0.5f32; 4];
        let mut output = [0.0f32; 4];

        let inputs: [&[f32]; 1] = [&input];
        let mut outputs: [&mut [f32]; 1] = [&mut output];

        for _ in 0..10 {
            gain.process(&inputs, &mut outputs, &[], &context);
        }

        let expected_linear = 10.0_f32.powf(6.0 / 20.0);
        for (i, &out) in output.iter().enumerate() {
            let expected = 0.5 * expected_linear;
            assert!(
                (out - expected).abs() < 0.05,
                "Output should be amplified at +6dB: expected={}, output[{}]={}",
                expected,
                i,
                out
            );
        }
    }

    #[test]
    fn test_amplification_at_positive_db_f64() {
        let mut gain = GainBlock::<f64>::new(6.0, None);
        let context = test_context(4);

        let input = [0.5f64; 4];
        let mut output = [0.0f64; 4];

        let inputs: [&[f64]; 1] = [&input];
        let mut outputs: [&mut [f64]; 1] = [&mut output];

        for _ in 0..10 {
            gain.process(&inputs, &mut outputs, &[], &context);
        }

        let expected_linear = 10.0_f64.powf(6.0 / 20.0);
        for (i, &out) in output.iter().enumerate() {
            let expected = 0.5 * expected_linear;
            assert!(
                (out - expected).abs() < 0.05,
                "Output should be amplified at +6dB: expected={}, output[{}]={}",
                expected,
                i,
                out
            );
        }
    }

    #[test]
    fn test_attenuation_at_negative_db_f32() {
        let mut gain = GainBlock::<f32>::new(-6.0, None);
        let context = test_context(4);

        let input = [1.0f32; 4];
        let mut output = [0.0f32; 4];

        let inputs: [&[f32]; 1] = [&input];
        let mut outputs: [&mut [f32]; 1] = [&mut output];

        for _ in 0..10 {
            gain.process(&inputs, &mut outputs, &[], &context);
        }

        let expected_linear = 10.0_f32.powf(-6.0 / 20.0);
        for (i, &out) in output.iter().enumerate() {
            assert!(
                (out - expected_linear).abs() < 0.05,
                "Output should be attenuated at -6dB: expected={}, output[{}]={}",
                expected_linear,
                i,
                out
            );
        }
    }

    #[test]
    fn test_attenuation_at_negative_db_f64() {
        let mut gain = GainBlock::<f64>::new(-6.0, None);
        let context = test_context(4);

        let input = [1.0f64; 4];
        let mut output = [0.0f64; 4];

        let inputs: [&[f64]; 1] = [&input];
        let mut outputs: [&mut [f64]; 1] = [&mut output];

        for _ in 0..10 {
            gain.process(&inputs, &mut outputs, &[], &context);
        }

        let expected_linear = 10.0_f64.powf(-6.0 / 20.0);
        for (i, &out) in output.iter().enumerate() {
            assert!(
                (out - expected_linear).abs() < 0.05,
                "Output should be attenuated at -6dB: expected={}, output[{}]={}",
                expected_linear,
                i,
                out
            );
        }
    }

    #[test]
    fn test_base_gain_multiplier_f32() {
        let mut gain = GainBlock::<f32>::new(0.0, Some(0.5));
        let context = test_context(4);

        let input = [1.0f32; 4];
        let mut output = [0.0f32; 4];

        let inputs: [&[f32]; 1] = [&input];
        let mut outputs: [&mut [f32]; 1] = [&mut output];

        for _ in 0..10 {
            gain.process(&inputs, &mut outputs, &[], &context);
        }

        for (i, &out) in output.iter().enumerate() {
            assert!(
                (out - 0.5).abs() < 0.05,
                "Base gain of 0.5 should halve output: output[{}]={}",
                i,
                out
            );
        }
    }

    #[test]
    fn test_multichannel_processing_f32() {
        let mut gain = GainBlock::<f32>::new(0.0, None);
        let context = test_context(4);

        let input_l = [1.0f32, 0.5, 0.25, 0.125];
        let input_r = [0.8f32, 0.4, 0.2, 0.1];
        let mut output_l = [0.0f32; 4];
        let mut output_r = [0.0f32; 4];

        let inputs: [&[f32]; 2] = [&input_l, &input_r];
        let mut outputs: [&mut [f32]; 2] = [&mut output_l, &mut output_r];

        for _ in 0..10 {
            gain.process(&inputs, &mut outputs, &[], &context);
        }

        for (i, (&inp, &out)) in input_l.iter().zip(output_l.iter()).enumerate() {
            assert!(
                (inp - out).abs() < 0.05,
                "Left channel passthrough: input[{}]={}, output[{}]={}",
                i,
                inp,
                i,
                out
            );
        }
        for (i, (&inp, &out)) in input_r.iter().zip(output_r.iter()).enumerate() {
            assert!(
                (inp - out).abs() < 0.05,
                "Right channel passthrough: input[{}]={}, output[{}]={}",
                i,
                inp,
                i,
                out
            );
        }
    }

    #[test]
    fn test_db_clamping_below_min_f32() {
        let mut gain = GainBlock::<f32>::new(-100.0, None);
        let context = test_context(4);

        let input = [1.0f32; 4];
        let mut output = [0.0f32; 4];

        let inputs: [&[f32]; 1] = [&input];
        let mut outputs: [&mut [f32]; 1] = [&mut output];

        for _ in 0..10 {
            gain.process(&inputs, &mut outputs, &[], &context);
        }

        let expected = 10.0_f32.powf(-80.0 / 20.0);
        for &out in &output {
            assert!(out.abs() < expected * 2.0, "Should clamp to -80dB minimum");
        }
    }

    #[test]
    fn test_db_clamping_above_max_f32() {
        let mut gain = GainBlock::<f32>::new(50.0, None);
        let context = test_context(4);

        let input = [0.1f32; 4];
        let mut output = [0.0f32; 4];

        let inputs: [&[f32]; 1] = [&input];
        let mut outputs: [&mut [f32]; 1] = [&mut output];

        for _ in 0..10 {
            gain.process(&inputs, &mut outputs, &[], &context);
        }

        let max_gain = 10.0_f32.powf(30.0 / 20.0);
        for &out in &output {
            assert!(
                out <= 0.1 * max_gain * 1.1,
                "Should clamp to +30dB maximum, got {}",
                out
            );
        }
    }

    #[test]
    fn test_silence_input_f32() {
        let mut gain = GainBlock::<f32>::new(20.0, None);
        let context = test_context(4);

        let input = [0.0f32; 4];
        let mut output = [1.0f32; 4];

        let inputs: [&[f32]; 1] = [&input];
        let mut outputs: [&mut [f32]; 1] = [&mut output];

        gain.process(&inputs, &mut outputs, &[], &context);

        for (i, &out) in output.iter().enumerate() {
            assert!(
                out.abs() < 1e-10,
                "Silence input should produce silence: output[{}]={}",
                i,
                out
            );
        }
    }

    #[test]
    fn test_silence_input_f64() {
        let mut gain = GainBlock::<f64>::new(20.0, None);
        let context = test_context(4);

        let input = [0.0f64; 4];
        let mut output = [1.0f64; 4];

        let inputs: [&[f64]; 1] = [&input];
        let mut outputs: [&mut [f64]; 1] = [&mut output];

        gain.process(&inputs, &mut outputs, &[], &context);

        for (i, &out) in output.iter().enumerate() {
            assert!(
                out.abs() < 1e-15,
                "Silence input should produce silence: output[{}]={}",
                i,
                out
            );
        }
    }
}
