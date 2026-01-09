//! Stereo panning with constant power law.

#[cfg(feature = "simd")]
use std::simd::{StdFloat, f64x4, num::SimdFloat};

#[cfg(feature = "simd")]
use crate::sample::SIMD_LANES;
use crate::{
    block::Block,
    context::DspContext,
    parameter::{ModulationOutput, Parameter},
    sample::Sample,
    smoothing::LinearSmoothedValue,
};

/// Maximum buffer size for stack-allocated gain arrays.
const MAX_BUFFER_SIZE: usize = 4096;

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

        let target_position = self.position.get_value(modulation_values);
        if (target_position.to_f64() - self.position_smoother.target().to_f64()).abs() > 1e-9 {
            self.position_smoother.set_target_value(target_position);
        }

        let left_in = inputs[0];
        let right_in = inputs.get(1).copied().unwrap_or(left_in);
        let has_stereo_input = inputs.len() > 1;
        let has_stereo_output = outputs.len() > 1;

        let num_samples = left_in
            .len()
            .min(outputs.first().map(|o| o.len()).unwrap_or(0))
            .min(MAX_BUFFER_SIZE);

        let mut positions: [f64; MAX_BUFFER_SIZE] = [0.0; MAX_BUFFER_SIZE];
        for position in positions.iter_mut().take(num_samples) {
            *position = self.position_smoother.get_next_value().to_f64();
        }

        // Compute gains in f64 for precision, then convert to S for SIMD sample processing
        let mut left_gains: [S; MAX_BUFFER_SIZE] = [S::ZERO; MAX_BUFFER_SIZE];
        let mut right_gains: [S; MAX_BUFFER_SIZE] = [S::ZERO; MAX_BUFFER_SIZE];

        #[cfg(feature = "simd")]
        {
            let chunks = num_samples / 4;
            let remainder_start = chunks * 4;

            for chunk_idx in 0..chunks {
                let offset = chunk_idx * 4;
                let pos_vec = f64x4::from_slice(&positions[offset..]);

                let normalized = ((pos_vec + f64x4::splat(100.0)) / f64x4::splat(200.0))
                    .simd_clamp(f64x4::splat(0.0), f64x4::splat(1.0));

                let angle = normalized * f64x4::splat(std::f64::consts::FRAC_PI_2);

                let l_arr = angle.cos().to_array();
                let r_arr = angle.sin().to_array();
                for i in 0..4 {
                    left_gains[offset + i] = S::from_f64(l_arr[i]);
                    right_gains[offset + i] = S::from_f64(r_arr[i]);
                }
            }

            for i in remainder_start..num_samples {
                let (l, r) = self.calculate_gains(positions[i]);
                left_gains[i] = S::from_f64(l);
                right_gains[i] = S::from_f64(r);
            }
        }

        #[cfg(not(feature = "simd"))]
        {
            for i in 0..num_samples {
                let (l, r) = self.calculate_gains(positions[i]);
                left_gains[i] = S::from_f64(l);
                right_gains[i] = S::from_f64(r);
            }
        }

        // Apply gains to samples using generic SIMD
        #[cfg(feature = "simd")]
        {
            let chunks = num_samples / SIMD_LANES;
            let remainder_start = chunks * SIMD_LANES;

            for chunk_idx in 0..chunks {
                let offset = chunk_idx * SIMD_LANES;

                let l_in = S::simd_from_slice(&left_in[offset..]);
                let l_gain = S::simd_from_slice(&left_gains[offset..]);
                let l_out = l_in * l_gain;
                outputs[0][offset..offset + SIMD_LANES].copy_from_slice(&S::simd_to_array(l_out));

                if has_stereo_output {
                    let r_in = if has_stereo_input {
                        S::simd_from_slice(&right_in[offset..])
                    } else {
                        l_in
                    };
                    let r_gain = S::simd_from_slice(&right_gains[offset..]);
                    let r_out = r_in * r_gain;
                    outputs[1][offset..offset + SIMD_LANES].copy_from_slice(&S::simd_to_array(r_out));
                }
            }

            for i in remainder_start..num_samples {
                outputs[0][i] = left_in[i] * left_gains[i];
                if has_stereo_output {
                    let r = if has_stereo_input { right_in[i] } else { left_in[i] };
                    outputs[1][i] = r * right_gains[i];
                }
            }
        }

        #[cfg(not(feature = "simd"))]
        {
            for i in 0..num_samples {
                outputs[0][i] = left_in[i] * left_gains[i];
                if has_stereo_output {
                    let r = if has_stereo_input { right_in[i] } else { left_in[i] };
                    outputs[1][i] = r * right_gains[i];
                }
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
