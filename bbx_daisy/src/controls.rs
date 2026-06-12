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
    /// CV / knob inputs, normalized to 0.0-1.0 and smoothed.
    ///
    /// On Patch.Init (patch_sm): `cv[0]`=CV_1 (PC0), `cv[1]`=CV_2 (PA3),
    /// `cv[2]`=CV_3 (PB1), `cv[3]`=CV_4 (PA7). Unused on Pod/Seed.
    pub cv: [f32; 4],
    /// Toggle / button state (`true` = active).
    ///
    /// On Patch.Init (patch_sm): the B8 switch (PB9), read active-low. Unused on Pod/Seed.
    pub switch: bool,
}

impl Controls {
    /// Create controls with default center values (0.5).
    #[inline]
    pub const fn new() -> Self {
        Self {
            knob1: 0.5,
            knob2: 0.5,
            cv: [0.5; 4],
            switch: false,
        }
    }

    /// Create controls with zero values.
    #[inline]
    pub const fn zero() -> Self {
        Self {
            knob1: 0.0,
            knob2: 0.0,
            cv: [0.0; 4],
            switch: false,
        }
    }

    /// Create controls with specific initial knob values (CVs centered, switch off).
    #[inline]
    pub const fn with_values(knob1: f32, knob2: f32) -> Self {
        Self {
            knob1,
            knob2,
            cv: [0.5; 4],
            switch: false,
        }
    }
}
