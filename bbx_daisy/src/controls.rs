//! Control input state for audio processing.
//!
//! This module provides the [`Controls`] struct which holds normalized
//! values for hardware control inputs (knobs, CVs) during audio processing.

/// Control inputs available during audio callback.
///
/// Provides access to hardware controls (knobs, CVs) during audio processing.
/// Values are normalized to 0.0-1.0 range and smoothed to prevent jitter.
///
/// # Example
///
/// ```ignore
/// impl AudioProcessor for MySynth {
///     fn process(
///         &mut self,
///         _input: &FrameBuffer<BLOCK_SIZE>,
///         output: &mut FrameBuffer<BLOCK_SIZE>,
///         controls: &Controls,
///     ) {
///         // Map knob1 to frequency (110Hz - 880Hz)
///         let freq = 110.0 + controls.knob1 * 770.0;
///         // ...
///     }
/// }
/// ```
#[derive(Clone, Copy, Default)]
pub struct Controls {
    /// Knob 1 value (0.0 to 1.0).
    ///
    /// On Pod: Physical knob 1 (PC4)
    pub knob1: f32,
    /// Knob 2 value (0.0 to 1.0).
    ///
    /// On Pod: Physical knob 2 (PC1)
    pub knob2: f32,
}

impl Controls {
    /// Create controls with default center values (0.5).
    #[inline]
    pub const fn new() -> Self {
        Self { knob1: 0.5, knob2: 0.5 }
    }

    /// Create controls with zero values.
    #[inline]
    pub const fn zero() -> Self {
        Self { knob1: 0.0, knob2: 0.0 }
    }

    /// Create controls with specific initial values.
    #[inline]
    pub const fn with_values(knob1: f32, knob2: f32) -> Self {
        Self { knob1, knob2 }
    }
}
