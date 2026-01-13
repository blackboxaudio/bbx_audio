//! Channel routing and manipulation for stereo signals.

use std::marker::PhantomData;

use crate::{block::Block, channel::ChannelConfig, context::DspContext, parameter::ModulationOutput, sample::Sample};

/// Channel routing mode for stereo signals.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(u8)]
pub enum ChannelMode {
    /// Normal stereo (L -> L, R -> R)
    #[default]
    Stereo = 0,
    /// Left only (L -> L, L -> R)
    Left = 1,
    /// Right only (R -> L, R -> R)
    Right = 2,
    /// Swap channels (L -> R, R -> L)
    Swap = 3,
}

impl From<i32> for ChannelMode {
    fn from(value: i32) -> Self {
        match value {
            0 => Self::Stereo,
            1 => Self::Left,
            2 => Self::Right,
            3 => Self::Swap,
            _ => Self::Stereo,
        }
    }
}

impl From<f32> for ChannelMode {
    fn from(value: f32) -> Self {
        Self::from(value as i32)
    }
}

/// A channel router block for stereo signal manipulation.
///
/// Supports channel selection, mono summing, and phase inversion.
pub struct ChannelRouterBlock<S: Sample> {
    /// Channel routing mode.
    pub mode: ChannelMode,
    /// Sum to mono (L+R)/2 on both channels.
    pub mono: bool,
    /// Invert left channel phase.
    pub invert_left: bool,
    /// Invert right channel phase.
    pub invert_right: bool,

    _phantom: PhantomData<S>,
}

impl<S: Sample> ChannelRouterBlock<S> {
    /// Create a new `ChannelRouterBlock`.
    pub fn new(mode: ChannelMode, mono: bool, invert_left: bool, invert_right: bool) -> Self {
        Self {
            mode,
            mono,
            invert_left,
            invert_right,
            _phantom: PhantomData,
        }
    }

    /// Create with default settings (stereo passthrough).
    pub fn default_new() -> Self {
        Self::new(ChannelMode::Stereo, false, false, false)
    }
}

impl<S: Sample> Block<S> for ChannelRouterBlock<S> {
    fn process(&mut self, inputs: &[&[S]], outputs: &mut [&mut [S]], _modulation_values: &[S], _context: &DspContext) {
        // Handle mono input
        if inputs.is_empty() {
            return;
        }

        let left_in = inputs.first().copied();
        let right_in = inputs.get(1).copied().or(left_in);

        let (left_in, right_in) = match (left_in, right_in) {
            (Some(l), Some(r)) => (l, r),
            _ => return,
        };

        let num_samples = left_in.len().min(outputs.first().map(|o| o.len()).unwrap_or(0));

        if num_samples == 0 {
            return;
        }

        let half = S::from_f64(0.5);

        for i in 0..num_samples {
            let l_sample = left_in[i];
            let r_sample = if inputs.len() > 1 { right_in[i] } else { l_sample };

            let (mut l_out, mut r_out) = match self.mode {
                ChannelMode::Stereo => (l_sample, r_sample),
                ChannelMode::Left => (l_sample, l_sample),
                ChannelMode::Right => (r_sample, r_sample),
                ChannelMode::Swap => (r_sample, l_sample),
            };

            if self.mono {
                let mono = (l_out + r_out) * half;
                l_out = mono;
                r_out = mono;
            }

            if self.invert_left {
                l_out = -l_out;
            }
            if self.invert_right {
                r_out = -r_out;
            }

            if !outputs.is_empty() {
                outputs[0][i] = l_out;
            }
            if outputs.len() > 1 {
                outputs[1][i] = r_out;
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

    #[inline]
    fn channel_config(&self) -> ChannelConfig {
        ChannelConfig::Explicit
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
    fn test_channel_router_input_output_counts_f32() {
        let router = ChannelRouterBlock::<f32>::default_new();
        assert_eq!(router.input_count(), 2);
        assert_eq!(router.output_count(), 2);
    }

    #[test]
    fn test_channel_router_input_output_counts_f64() {
        let router = ChannelRouterBlock::<f64>::default_new();
        assert_eq!(router.input_count(), 2);
        assert_eq!(router.output_count(), 2);
    }

    #[test]
    fn test_channel_router_returns_explicit_config() {
        let router = ChannelRouterBlock::<f32>::default_new();
        assert_eq!(router.channel_config(), ChannelConfig::Explicit);
    }

    #[test]
    fn test_channel_router_stereo_passthrough_f32() {
        let mut router = ChannelRouterBlock::<f32>::new(ChannelMode::Stereo, false, false, false);
        let context = test_context(4);

        let left_in = [1.0f32, 2.0, 3.0, 4.0];
        let right_in = [5.0f32, 6.0, 7.0, 8.0];
        let mut left_out = [0.0f32; 4];
        let mut right_out = [0.0f32; 4];

        let inputs: [&[f32]; 2] = [&left_in, &right_in];
        let mut outputs: [&mut [f32]; 2] = [&mut left_out, &mut right_out];

        router.process(&inputs, &mut outputs, &[], &context);

        assert_eq!(left_out, left_in);
        assert_eq!(right_out, right_in);
    }

    #[test]
    fn test_channel_router_stereo_passthrough_f64() {
        let mut router = ChannelRouterBlock::<f64>::new(ChannelMode::Stereo, false, false, false);
        let context = test_context(4);

        let left_in = [1.0f64, 2.0, 3.0, 4.0];
        let right_in = [5.0f64, 6.0, 7.0, 8.0];
        let mut left_out = [0.0f64; 4];
        let mut right_out = [0.0f64; 4];

        let inputs: [&[f64]; 2] = [&left_in, &right_in];
        let mut outputs: [&mut [f64]; 2] = [&mut left_out, &mut right_out];

        router.process(&inputs, &mut outputs, &[], &context);

        assert_eq!(left_out, left_in);
        assert_eq!(right_out, right_in);
    }

    #[test]
    fn test_channel_router_left_mode_f32() {
        let mut router = ChannelRouterBlock::<f32>::new(ChannelMode::Left, false, false, false);
        let context = test_context(4);

        let left_in = [1.0f32, 2.0, 3.0, 4.0];
        let right_in = [5.0f32, 6.0, 7.0, 8.0];
        let mut left_out = [0.0f32; 4];
        let mut right_out = [0.0f32; 4];

        let inputs: [&[f32]; 2] = [&left_in, &right_in];
        let mut outputs: [&mut [f32]; 2] = [&mut left_out, &mut right_out];

        router.process(&inputs, &mut outputs, &[], &context);

        assert_eq!(left_out, left_in);
        assert_eq!(right_out, left_in);
    }

    #[test]
    fn test_channel_router_right_mode_f32() {
        let mut router = ChannelRouterBlock::<f32>::new(ChannelMode::Right, false, false, false);
        let context = test_context(4);

        let left_in = [1.0f32, 2.0, 3.0, 4.0];
        let right_in = [5.0f32, 6.0, 7.0, 8.0];
        let mut left_out = [0.0f32; 4];
        let mut right_out = [0.0f32; 4];

        let inputs: [&[f32]; 2] = [&left_in, &right_in];
        let mut outputs: [&mut [f32]; 2] = [&mut left_out, &mut right_out];

        router.process(&inputs, &mut outputs, &[], &context);

        assert_eq!(left_out, right_in);
        assert_eq!(right_out, right_in);
    }

    #[test]
    fn test_channel_router_swap_mode_f32() {
        let mut router = ChannelRouterBlock::<f32>::new(ChannelMode::Swap, false, false, false);
        let context = test_context(4);

        let left_in = [1.0f32, 2.0, 3.0, 4.0];
        let right_in = [5.0f32, 6.0, 7.0, 8.0];
        let mut left_out = [0.0f32; 4];
        let mut right_out = [0.0f32; 4];

        let inputs: [&[f32]; 2] = [&left_in, &right_in];
        let mut outputs: [&mut [f32]; 2] = [&mut left_out, &mut right_out];

        router.process(&inputs, &mut outputs, &[], &context);

        assert_eq!(left_out, right_in);
        assert_eq!(right_out, left_in);
    }

    #[test]
    fn test_channel_router_mono_sum_f32() {
        let mut router = ChannelRouterBlock::<f32>::new(ChannelMode::Stereo, true, false, false);
        let context = test_context(4);

        let left_in = [2.0f32, 4.0, 6.0, 8.0];
        let right_in = [4.0f32, 6.0, 8.0, 10.0];
        let mut left_out = [0.0f32; 4];
        let mut right_out = [0.0f32; 4];

        let inputs: [&[f32]; 2] = [&left_in, &right_in];
        let mut outputs: [&mut [f32]; 2] = [&mut left_out, &mut right_out];

        router.process(&inputs, &mut outputs, &[], &context);

        let expected = [3.0f32, 5.0, 7.0, 9.0];
        for i in 0..4 {
            assert!((left_out[i] - expected[i]).abs() < 1e-6);
            assert!((right_out[i] - expected[i]).abs() < 1e-6);
        }
    }

    #[test]
    fn test_channel_router_invert_left_f32() {
        let mut router = ChannelRouterBlock::<f32>::new(ChannelMode::Stereo, false, true, false);
        let context = test_context(4);

        let left_in = [1.0f32, 2.0, 3.0, 4.0];
        let right_in = [5.0f32, 6.0, 7.0, 8.0];
        let mut left_out = [0.0f32; 4];
        let mut right_out = [0.0f32; 4];

        let inputs: [&[f32]; 2] = [&left_in, &right_in];
        let mut outputs: [&mut [f32]; 2] = [&mut left_out, &mut right_out];

        router.process(&inputs, &mut outputs, &[], &context);

        let expected_left = [-1.0f32, -2.0, -3.0, -4.0];
        assert_eq!(left_out, expected_left);
        assert_eq!(right_out, right_in);
    }

    #[test]
    fn test_channel_router_invert_right_f32() {
        let mut router = ChannelRouterBlock::<f32>::new(ChannelMode::Stereo, false, false, true);
        let context = test_context(4);

        let left_in = [1.0f32, 2.0, 3.0, 4.0];
        let right_in = [5.0f32, 6.0, 7.0, 8.0];
        let mut left_out = [0.0f32; 4];
        let mut right_out = [0.0f32; 4];

        let inputs: [&[f32]; 2] = [&left_in, &right_in];
        let mut outputs: [&mut [f32]; 2] = [&mut left_out, &mut right_out];

        router.process(&inputs, &mut outputs, &[], &context);

        let expected_right = [-5.0f32, -6.0, -7.0, -8.0];
        assert_eq!(left_out, left_in);
        assert_eq!(right_out, expected_right);
    }

    #[test]
    fn test_channel_router_invert_both_f32() {
        let mut router = ChannelRouterBlock::<f32>::new(ChannelMode::Stereo, false, true, true);
        let context = test_context(4);

        let left_in = [1.0f32, 2.0, 3.0, 4.0];
        let right_in = [5.0f32, 6.0, 7.0, 8.0];
        let mut left_out = [0.0f32; 4];
        let mut right_out = [0.0f32; 4];

        let inputs: [&[f32]; 2] = [&left_in, &right_in];
        let mut outputs: [&mut [f32]; 2] = [&mut left_out, &mut right_out];

        router.process(&inputs, &mut outputs, &[], &context);

        let expected_left = [-1.0f32, -2.0, -3.0, -4.0];
        let expected_right = [-5.0f32, -6.0, -7.0, -8.0];
        assert_eq!(left_out, expected_left);
        assert_eq!(right_out, expected_right);
    }

    #[test]
    fn test_channel_mode_from_i32() {
        assert_eq!(ChannelMode::from(0), ChannelMode::Stereo);
        assert_eq!(ChannelMode::from(1), ChannelMode::Left);
        assert_eq!(ChannelMode::from(2), ChannelMode::Right);
        assert_eq!(ChannelMode::from(3), ChannelMode::Swap);
        assert_eq!(ChannelMode::from(999), ChannelMode::Stereo);
    }

    #[test]
    fn test_channel_mode_from_f32() {
        assert_eq!(ChannelMode::from(0.0f32), ChannelMode::Stereo);
        assert_eq!(ChannelMode::from(1.0f32), ChannelMode::Left);
        assert_eq!(ChannelMode::from(2.0f32), ChannelMode::Right);
        assert_eq!(ChannelMode::from(3.0f32), ChannelMode::Swap);
    }
}
