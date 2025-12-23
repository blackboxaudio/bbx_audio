//! Gain block for applying volume control with smoothing.

use crate::{
    block::Block,
    context::DspContext,
    parameter::{ModulationOutput, ModulatableParam},
    sample::Sample,
};

/// Attempt at linear interpolation for parameter smoothing.
///
/// TODO(mm): Move this to a separate module for reuse.
#[derive(Debug, Clone)]
pub struct SmoothedValue<S: Sample> {
    current: S,
    target: S,
    step: S,
    samples_remaining: usize,
}

impl<S: Sample> SmoothedValue<S> {
    /// Create a new `SmoothedValue` with a given initial value.
    pub fn new(initial: S) -> Self {
        Self {
            current: initial,
            target: initial,
            step: S::ZERO,
            samples_remaining: 0,
        }
    }

    /// Set the target value with smoothing over a given duration.
    pub fn set_target(&mut self, target: S, sample_rate: f64, smoothing_ms: f64) {
        if smoothing_ms <= 0.0 {
            self.current = target;
            self.target = target;
            self.step = S::ZERO;
            self.samples_remaining = 0;
            return;
        }

        self.target = target;
        let samples = ((smoothing_ms / 1000.0) * sample_rate) as usize;
        self.samples_remaining = samples.max(1);

        let diff = target - self.current;
        self.step = diff / S::from_f64(self.samples_remaining as f64);
    }

    /// Get the next smoothed value.
    #[inline]
    pub fn next(&mut self) -> S {
        if self.samples_remaining > 0 {
            self.current = self.current + self.step;
            self.samples_remaining -= 1;
            if self.samples_remaining == 0 {
                self.current = self.target;
            }
        }
        self.current
    }

    /// Check if the value is currently smoothing.
    pub fn is_smoothing(&self) -> bool {
        self.samples_remaining > 0
    }

    /// Get the current value without advancing.
    pub fn current(&self) -> S {
        self.current
    }
}

/// Convert decibels to linear gain.
#[inline]
fn db_to_linear<S: Sample>(db: S) -> S {
    S::from_f64(10.0_f64.powf(db.to_f64() / 20.0))
}

/// Gain block for applying volume control.
///
/// The level parameter is in decibels (-inf to +24 dB typically).
/// Smoothing prevents clicks when the gain changes.
pub struct GainBlock<S: Sample> {
    /// Level in decibels.
    pub level: ModulatableParam<S>,
    /// Smoothing time in milliseconds.
    smoothing_ms: f64,
    /// Smoothed linear gain value.
    smoothed_gain: SmoothedValue<S>,
    /// Number of input/output channels.
    num_channels: usize,
}

impl<S: Sample> GainBlock<S> {
    /// Create a new `GainBlock` with the given level (in dB) and smoothing time.
    pub fn new(level_db: S, smoothing_ms: f64, num_channels: usize) -> Self {
        let linear = db_to_linear(level_db);
        Self {
            level: ModulatableParam::new(level_db),
            smoothing_ms,
            smoothed_gain: SmoothedValue::new(linear),
            num_channels,
        }
    }

    /// Create a unity gain block (0 dB).
    pub fn unity(num_channels: usize) -> Self {
        Self::new(S::ZERO, 20.0, num_channels)
    }
}

impl<S: Sample> Block<S> for GainBlock<S> {
    fn process(
        &mut self,
        inputs: &[&[S]],
        outputs: &mut [&mut [S]],
        modulation_values: &[S],
        context: &DspContext,
    ) {
        // Get target gain from parameter (with modulation)
        let target_db = self.level.evaluate(modulation_values);
        let target_linear = db_to_linear(target_db);
        self.smoothed_gain.set_target(target_linear, context.sample_rate, self.smoothing_ms);

        // Process each channel
        let channel_count = inputs.len().min(outputs.len()).min(self.num_channels);
        for channel in 0..channel_count {
            let input = inputs[channel];
            let output = &mut outputs[channel];

            for i in 0..input.len().min(output.len()) {
                let gain = self.smoothed_gain.next();
                output[i] = input[i] * gain;
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

    #[test]
    fn test_db_to_linear() {
        // 0 dB = unity gain
        assert!((db_to_linear(0.0f32) - 1.0).abs() < 1e-6);
        // -6 dB ≈ 0.5
        assert!((db_to_linear(-6.0206f32) - 0.5).abs() < 0.01);
        // +6 dB ≈ 2.0
        assert!((db_to_linear(6.0206f32) - 2.0).abs() < 0.01);
    }

    #[test]
    fn test_smoothed_value() {
        let mut sv = SmoothedValue::new(0.0f32);
        sv.set_target(1.0, 44100.0, 10.0); // 10ms smoothing

        // Should be smoothing
        assert!(sv.is_smoothing());

        // After advancing, should approach target
        for _ in 0..500 {
            sv.next();
        }
        assert!((sv.current() - 1.0).abs() < 0.1);
    }

    #[test]
    fn test_gain_unity() {
        let mut gain = GainBlock::<f32>::new(0.0, 0.0, 2); // 0 dB, no smoothing
        let input = [vec![0.5f32; 64], vec![0.5f32; 64]];
        let mut output_left = vec![0.0f32; 64];
        let mut output_right = vec![0.0f32; 64];

        let context = DspContext {
            sample_rate: 44100.0,
            buffer_size: 64,
            num_channels: 2,
            current_sample: 0,
        };

        gain.process(
            &[&input[0], &input[1]],
            &mut [&mut output_left, &mut output_right],
            &[],
            &context,
        );

        // 0 dB = unity gain, output should equal input
        assert!((output_left[63] - 0.5).abs() < 0.001);
        assert!((output_right[63] - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_gain_minus_6db() {
        let mut gain = GainBlock::<f32>::new(-6.0206, 0.0, 2); // -6 dB ≈ 0.5x
        let input = [vec![1.0f32; 64], vec![1.0f32; 64]];
        let mut output_left = vec![0.0f32; 64];
        let mut output_right = vec![0.0f32; 64];

        let context = DspContext {
            sample_rate: 44100.0,
            buffer_size: 64,
            num_channels: 2,
            current_sample: 0,
        };

        gain.process(
            &[&input[0], &input[1]],
            &mut [&mut output_left, &mut output_right],
            &[],
            &context,
        );

        // -6 dB ≈ 0.5x gain
        assert!((output_left[63] - 0.5).abs() < 0.01);
    }
}
