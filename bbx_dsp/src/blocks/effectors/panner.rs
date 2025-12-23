//! Stereo panner block with equal power panning law.

use crate::{
    block::Block,
    context::DspContext,
    parameter::{ModulationOutput, ModulatableParam},
    sample::Sample,
};

/// Stereo panner with equal power panning law.
///
/// Position ranges from -1.0 (full left) to 1.0 (full right), with 0.0 being center.
/// Uses constant power panning to maintain perceived loudness across the stereo field.
pub struct PannerBlock<S: Sample> {
    /// Pan position: -1.0 (left) to 1.0 (right).
    pub position: ModulatableParam<S>,
}

impl<S: Sample> PannerBlock<S> {
    /// Create a new panner at the given position.
    ///
    /// # Arguments
    /// * `position` - Pan position from -1.0 (left) to 1.0 (right)
    pub fn new(position: S) -> Self {
        Self {
            position: ModulatableParam::new(position),
        }
    }

    /// Create a centered panner.
    pub fn center() -> Self {
        Self::new(S::ZERO)
    }

    /// Calculate left and right gains for a given pan position.
    ///
    /// Uses equal power (constant power) panning law.
    #[inline]
    fn calculate_gains(position: f64) -> (f64, f64) {
        // Clamp position to valid range
        let pos = position.clamp(-1.0, 1.0);

        // Convert position to angle (0 to π/2)
        // pos = -1 -> angle = 0 (full left)
        // pos = 0 -> angle = π/4 (center)
        // pos = 1 -> angle = π/2 (full right)
        let angle = (pos + 1.0) * std::f64::consts::FRAC_PI_4;

        // Equal power panning: left = cos(angle), right = sin(angle)
        let left_gain = angle.cos();
        let right_gain = angle.sin();

        (left_gain, right_gain)
    }
}

impl<S: Sample> Block<S> for PannerBlock<S> {
    fn process(
        &mut self,
        inputs: &[&[S]],
        outputs: &mut [&mut [S]],
        modulation_values: &[S],
        _context: &DspContext,
    ) {
        // Need at least stereo I/O
        if inputs.len() < 2 || outputs.len() < 2 {
            return;
        }

        let position = self.position.evaluate(modulation_values);
        let (left_gain, right_gain) = Self::calculate_gains(position.to_f64());

        let left_gain_s = S::from_f64(left_gain);
        let right_gain_s = S::from_f64(right_gain);

        let input_left = inputs[0];
        let input_right = inputs[1];

        // Use split_at_mut to get two mutable slices
        let (left_outputs, right_outputs) = outputs.split_at_mut(1);
        let output_left = &mut left_outputs[0];
        let output_right = &mut right_outputs[0];

        let len = input_left.len().min(input_right.len())
            .min(output_left.len()).min(output_right.len());

        for i in 0..len {
            output_left[i] = input_left[i] * left_gain_s;
            output_right[i] = input_right[i] * right_gain_s;
        }
    }

    #[inline]
    fn input_count(&self) -> usize {
        2 // Stereo input
    }

    #[inline]
    fn output_count(&self) -> usize {
        2 // Stereo output
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
    fn test_center_pan() {
        let (left, right) = PannerBlock::<f32>::calculate_gains(0.0);

        // At center, both channels should have equal gain
        assert!((left - right).abs() < 0.001);

        // And the combined power should be approximately 1
        // (cos²(π/4) + sin²(π/4) = 0.5 + 0.5 = 1)
        let power = left * left + right * right;
        assert!((power - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_full_left_pan() {
        let (left, right) = PannerBlock::<f32>::calculate_gains(-1.0);

        // Full left: left channel should be 1, right should be 0
        assert!((left - 1.0).abs() < 0.001);
        assert!(right.abs() < 0.001);
    }

    #[test]
    fn test_full_right_pan() {
        let (left, right) = PannerBlock::<f32>::calculate_gains(1.0);

        // Full right: left channel should be 0, right should be 1
        assert!(left.abs() < 0.001);
        assert!((right - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_equal_power_maintained() {
        // Test that power is approximately constant across all pan positions
        for i in 0..21 {
            let pos = (i as f64 / 10.0) - 1.0; // -1.0 to 1.0
            let (left, right) = PannerBlock::<f32>::calculate_gains(pos);
            let power = left * left + right * right;

            // Power should be approximately 1.0 at all positions
            assert!((power - 1.0).abs() < 0.001, "Power at pos {} is {}", pos, power);
        }
    }

    #[test]
    fn test_panner_process() {
        let mut panner = PannerBlock::<f32>::new(0.5); // Slightly right
        let context = make_context();

        let input_left = vec![1.0f32; 64];
        let input_right = vec![1.0f32; 64];
        let mut output_left = vec![0.0f32; 64];
        let mut output_right = vec![0.0f32; 64];

        panner.process(
            &[&input_left, &input_right],
            &mut [&mut output_left, &mut output_right],
            &[],
            &context,
        );

        // Right channel should be louder than left when panned right
        assert!(output_right[32] > output_left[32]);
    }
}
