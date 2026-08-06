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

use core::sync::atomic::{AtomicBool, AtomicU32, Ordering};

/// Lock-free shared storage for [`Controls`], safe to share between the
/// main loop (writer) and the audio ISR (reader).
///
/// Each f32 is stored as its `u32` bit pattern in an [`AtomicU32`]. `Relaxed`
/// ordering is sufficient: the Cortex-M7 is single-core, the fields are
/// independent, and control values carry no cross-field invariants — the ISR
/// just needs tear-free, race-free reads that the compiler cannot cache or
/// elide (which a plain `static mut` did not guarantee).
pub struct AtomicControls {
    knob1: AtomicU32,
    knob2: AtomicU32,
    cv: [AtomicU32; 4],
    switch_state: AtomicBool,
}

impl AtomicControls {
    /// Create storage with center default values (matches [`Controls::new`]).
    pub const fn new() -> Self {
        const CENTER: u32 = 0.5f32.to_bits();
        Self {
            knob1: AtomicU32::new(CENTER),
            knob2: AtomicU32::new(CENTER),
            cv: [
                AtomicU32::new(CENTER),
                AtomicU32::new(CENTER),
                AtomicU32::new(CENTER),
                AtomicU32::new(CENTER),
            ],
            switch_state: AtomicBool::new(false),
        }
    }

    /// Store knob 1 (0.0 to 1.0).
    #[inline]
    pub fn set_knob1(&self, value: f32) {
        self.knob1.store(value.to_bits(), Ordering::Relaxed);
    }

    /// Store knob 2 (0.0 to 1.0).
    #[inline]
    pub fn set_knob2(&self, value: f32) {
        self.knob2.store(value.to_bits(), Ordering::Relaxed);
    }

    /// Store a CV input (index 0-3, value 0.0 to 1.0).
    ///
    /// Out-of-range indices are ignored.
    #[inline]
    pub fn set_cv(&self, index: usize, value: f32) {
        if let Some(slot) = self.cv.get(index) {
            slot.store(value.to_bits(), Ordering::Relaxed);
        }
    }

    /// Store the switch state.
    #[inline]
    pub fn set_switch(&self, active: bool) {
        self.switch_state.store(active, Ordering::Relaxed);
    }

    /// Store a whole [`Controls`] value field-by-field.
    #[inline]
    pub fn store_from(&self, controls: &Controls) {
        self.set_knob1(controls.knob1);
        self.set_knob2(controls.knob2);
        for (i, v) in controls.cv.iter().enumerate() {
            self.set_cv(i, *v);
        }
        self.set_switch(controls.switch);
    }

    /// Load a plain [`Controls`] snapshot — what the audio ISR passes to the
    /// processor's `process` each block.
    #[inline]
    pub fn load(&self) -> Controls {
        Controls {
            knob1: f32::from_bits(self.knob1.load(Ordering::Relaxed)),
            knob2: f32::from_bits(self.knob2.load(Ordering::Relaxed)),
            cv: [
                f32::from_bits(self.cv[0].load(Ordering::Relaxed)),
                f32::from_bits(self.cv[1].load(Ordering::Relaxed)),
                f32::from_bits(self.cv[2].load(Ordering::Relaxed)),
                f32::from_bits(self.cv[3].load(Ordering::Relaxed)),
            ],
            switch: self.switch_state.load(Ordering::Relaxed),
        }
    }
}

impl Default for AtomicControls {
    fn default() -> Self {
        Self::new()
    }
}
