//! Stereo panning with constant power law.

use crate::{
    block::Block,
    context::DspContext,
    parameter::{ModulationOutput, Parameter},
    sample::Sample,
    smoothing::LinearSmoothedValue,
};

/// A stereo panner block with constant power pan law.
///
/// Position ranges from -100 (full left) to +100 (full right).
pub struct PannerBlock<S: Sample> {
    /// Pan position: -100 (left) to +100 (right).
    pub position: Parameter<S>,

    /// Smoothed position value for click-free panning.
    position_smoother: LinearSmoothedValue<S>,
}

impl<S: Sample> PannerBlock<S> {
    /// Create a new `PannerBlock` with the given position.
    pub fn new(position: S) -> Self {
        Self {
            position: Parameter::Constant(position),
            position_smoother: LinearSmoothedValue::new(position),
        }
    }

    /// Create a centered panner.
    pub fn centered() -> Self {
        Self::new(S::ZERO)
    }

    /// Calculate left and right gains using constant power pan law.
    #[inline]
    fn calculate_gains(&self, position: f64) -> (f64, f64) {
        // Normalize position to 0..1 range (0 = full left, 1 = full right)
        let normalized = (position + 100.0) / 200.0;
        let normalized = normalized.clamp(0.0, 1.0);

        // Constant power pan law using sine/cosine
        let angle = normalized * std::f64::consts::FRAC_PI_2;
        let left_gain = angle.cos();
        let right_gain = angle.sin();

        (left_gain, right_gain)
    }
}

impl<S: Sample> Block<S> for PannerBlock<S> {
    fn process(&mut self, inputs: &[&[S]], outputs: &mut [&mut [S]], modulation_values: &[S], _context: &DspContext) {
        if inputs.is_empty() || outputs.is_empty() {
            return;
        }

        // Get target position and set up smoothing
        let target_position = self.position.get_value(modulation_values);
        if (target_position.to_f64() - self.position_smoother.target().to_f64()).abs() > 1e-9 {
            self.position_smoother.set_target_value(target_position);
        }

        let left_in = inputs[0];
        let right_in = inputs.get(1).copied().unwrap_or(left_in);

        let num_samples = left_in.len().min(outputs.first().map(|o| o.len()).unwrap_or(0));

        for i in 0..num_samples {
            let position = self.position_smoother.get_next_value().to_f64();
            let (left_gain, right_gain) = self.calculate_gains(position);

            let l = left_in[i].to_f64();
            let r = if inputs.len() > 1 { right_in[i].to_f64() } else { l };

            let l_out = l * left_gain;
            let r_out = r * right_gain;

            if !outputs.is_empty() {
                outputs[0][i] = S::from_f64(l_out);
            }
            if outputs.len() > 1 {
                outputs[1][i] = S::from_f64(r_out);
            }
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
