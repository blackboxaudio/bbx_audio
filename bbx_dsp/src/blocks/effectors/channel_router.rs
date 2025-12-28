//! Channel routing and manipulation for stereo signals.

use std::marker::PhantomData;

use crate::{
    block::Block,
    context::DspContext,
    parameter::ModulationOutput,
    sample::Sample,
};

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
    fn process(
        &mut self,
        inputs: &[&[S]],
        outputs: &mut [&mut [S]],
        _modulation_values: &[S],
        _context: &DspContext,
    ) {
        // Handle mono input
        if inputs.is_empty() {
            return;
        }

        let left_in = inputs.first().map(|b| *b);
        let right_in = inputs.get(1).copied().or(left_in);

        let (left_in, right_in) = match (left_in, right_in) {
            (Some(l), Some(r)) => (l, r),
            _ => return,
        };

        let num_samples = left_in.len().min(
            outputs
                .first()
                .map(|o| o.len())
                .unwrap_or(0),
        );

        if num_samples == 0 {
            return;
        }

        for i in 0..num_samples {
            let l_sample = left_in[i].to_f64();
            let r_sample = if inputs.len() > 1 {
                right_in[i].to_f64()
            } else {
                l_sample
            };

            // Apply channel mode
            let (mut l_out, mut r_out) = match self.mode {
                ChannelMode::Stereo => (l_sample, r_sample),
                ChannelMode::Left => (l_sample, l_sample),
                ChannelMode::Right => (r_sample, r_sample),
                ChannelMode::Swap => (r_sample, l_sample),
            };

            // Apply mono summing
            if self.mono {
                let mono = (l_out + r_out) * 0.5;
                l_out = mono;
                r_out = mono;
            }

            // Apply phase inversion
            if self.invert_left {
                l_out = -l_out;
            }
            if self.invert_right {
                r_out = -r_out;
            }

            // Write outputs
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
