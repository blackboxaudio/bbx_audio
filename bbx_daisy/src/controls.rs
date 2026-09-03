//! Control input/output state shared between the main loop and the audio ISR.
//!
//! [`Controls`] carries the hardware control surface *into* the audio callback
//! (knobs, CV jacks, gates, buttons), and [`Outputs`] carries processor-driven
//! signals back *out* (LED, CV out, gate outs). Both cross the main-loop/ISR
//! boundary through their lock-free atomic twins ([`AtomicControls`],
//! [`AtomicOutputs`]).

/// Full-scale voltage of a bipolar CV input at `cv[i] == ±1.0` (Patch SM: ±5 V).
pub const CV_FULL_SCALE_VOLTS: f32 = 5.0;

/// Control inputs available during the audio callback.
///
/// One struct serves every board; only the fields the board actually has are
/// written (the rest stay at their defaults):
///
/// | Field    | Pod                          | Patch.Init (`patch_sm`)                          |
/// |----------|------------------------------|--------------------------------------------------|
/// | `knobs`  | `[0]`=knob 1, `[1]`=knob 2   | panel knobs 1-4 (SM channels CV_1-4)             |
/// | `cv`     | —                            | panel CV jacks 1-4 (SM channels CV_5-8), bipolar |
/// | `gate1/2`| —                            | gate inputs 1/2 (raw level, undebounced)         |
/// | `button` | —                            | B7 momentary button (debounced)                  |
/// | `switch` | —                            | B8 toggle (debounced)                            |
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
///         // Map knob 1 to frequency (110Hz - 880Hz)
///         let freq = 110.0 + controls.knobs[0] * 770.0;
///         // 1V/oct pitch tracking from CV jack 1
///         let volts = controls.cv_volts(0);
///         // ...
///     }
/// }
/// ```
#[derive(Clone, Copy, Default)]
pub struct Controls {
    /// Panel knob values, normalized 0.0 to 1.0 and smoothed.
    ///
    /// Pod: `knobs[0]` = knob 1 (PC4), `knobs[1]` = knob 2 (PC0).
    /// Patch.Init: `knobs[0..4]` = panel knobs 1-4 (SM channels CV_1-4:
    /// PA3, PA6, PA2, PA7).
    pub knobs: [f32; 4],
    /// Bipolar CV jack values, normalized -1.0 to +1.0 (±5 V) and smoothed.
    ///
    /// Patch.Init: `cv[0..4]` = panel CV jacks 1-4 (SM channels CV_5-8:
    /// PC1, PC0, PB1, PC4). The module's inverting input stage is corrected
    /// here, so +5 V at the jack reads +1.0. Unused on Pod/Seed.
    pub cv: [f32; 4],
    /// Gate input 1 level (`true` = gate high at the jack).
    ///
    /// Patch.Init: Gate In 1 (header B10, PG13). Raw and undebounced so
    /// triggers arrive with minimum latency — edge detection belongs in the
    /// processor, which owns the previous-state memory.
    pub gate1: bool,
    /// Gate input 2 level (`true` = gate high at the jack).
    ///
    /// Patch.Init: Gate In 2 (header B9, PG14). Raw and undebounced.
    pub gate2: bool,
    /// Momentary button state (`true` = pressed), debounced.
    ///
    /// Patch.Init: the B7 push button (PB8, active-low). Unused on Pod/Seed.
    pub button: bool,
    /// Toggle switch state (`true` = active), debounced.
    ///
    /// Patch.Init: the B8 toggle (PB9, active-low). Unused on Pod/Seed.
    pub switch: bool,
}

impl Controls {
    /// Create controls at rest: knobs centered (0.5), CVs at 0 V, everything off.
    #[inline]
    pub const fn new() -> Self {
        Self {
            knobs: [0.5; 4],
            cv: [0.0; 4],
            gate1: false,
            gate2: false,
            button: false,
            switch: false,
        }
    }

    /// Create controls with all values zeroed (knobs fully CCW).
    #[inline]
    pub const fn zero() -> Self {
        Self {
            knobs: [0.0; 4],
            cv: [0.0; 4],
            gate1: false,
            gate2: false,
            button: false,
            switch: false,
        }
    }

    /// CV jack value in volts (Patch SM bipolar range: -5.0 to +5.0).
    ///
    /// Convenient for 1V/oct pitch math: `f = f0 * powf(2.0, controls.cv_volts(0))`.
    /// Out-of-range indices return 0.0.
    #[inline]
    pub fn cv_volts(&self, index: usize) -> f32 {
        self.cv.get(index).copied().unwrap_or(0.0) * CV_FULL_SCALE_VOLTS
    }
}

/// Processor-driven outputs applied by the main loop.
///
/// Boards without these outputs simply never apply them. On Patch.Init:
///
/// | Field       | Hardware                                                  |
/// |-------------|-----------------------------------------------------------|
/// | `led`       | front-panel LED (DAC channel 2 — brightness is analog)    |
/// | `cv_out`    | CV OUT jack (DAC channel 1, 0..1 → 0-5 V)                 |
/// | `gate_out1` | Gate Out 1 (header B5, PC14)                              |
/// | `gate_out2` | Gate Out 2 (header B6, PC13)                              |
#[derive(Clone, Copy, Default)]
pub struct Outputs {
    /// LED brightness, 0.0 (off) to 1.0 (full).
    pub led: f32,
    /// CV output level, 0.0 to 1.0 (full scale ≈ 5 V at the jack).
    pub cv_out: f32,
    /// Gate output 1 level.
    pub gate_out1: bool,
    /// Gate output 2 level.
    pub gate_out2: bool,
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
    knobs: [AtomicU32; 4],
    cv: [AtomicU32; 4],
    gate1: AtomicBool,
    gate2: AtomicBool,
    button: AtomicBool,
    switch_state: AtomicBool,
}

impl AtomicControls {
    /// Create storage with at-rest default values (matches [`Controls::new`]).
    pub const fn new() -> Self {
        const CENTER: u32 = 0.5f32.to_bits();
        const ZERO: u32 = 0.0f32.to_bits();
        Self {
            knobs: [
                AtomicU32::new(CENTER),
                AtomicU32::new(CENTER),
                AtomicU32::new(CENTER),
                AtomicU32::new(CENTER),
            ],
            cv: [
                AtomicU32::new(ZERO),
                AtomicU32::new(ZERO),
                AtomicU32::new(ZERO),
                AtomicU32::new(ZERO),
            ],
            gate1: AtomicBool::new(false),
            gate2: AtomicBool::new(false),
            button: AtomicBool::new(false),
            switch_state: AtomicBool::new(false),
        }
    }

    /// Store a knob value (index 0-3, value 0.0 to 1.0).
    ///
    /// Out-of-range indices are ignored.
    #[inline]
    pub fn set_knob(&self, index: usize, value: f32) {
        if let Some(slot) = self.knobs.get(index) {
            slot.store(value.to_bits(), Ordering::Relaxed);
        }
    }

    /// Store a CV jack value (index 0-3, value -1.0 to +1.0).
    ///
    /// Out-of-range indices are ignored.
    #[inline]
    pub fn set_cv(&self, index: usize, value: f32) {
        if let Some(slot) = self.cv.get(index) {
            slot.store(value.to_bits(), Ordering::Relaxed);
        }
    }

    /// Store the gate input 1 level.
    #[inline]
    pub fn set_gate1(&self, active: bool) {
        self.gate1.store(active, Ordering::Relaxed);
    }

    /// Store the gate input 2 level.
    #[inline]
    pub fn set_gate2(&self, active: bool) {
        self.gate2.store(active, Ordering::Relaxed);
    }

    /// Store the button state.
    #[inline]
    pub fn set_button(&self, pressed: bool) {
        self.button.store(pressed, Ordering::Relaxed);
    }

    /// Store the switch state.
    #[inline]
    pub fn set_switch(&self, active: bool) {
        self.switch_state.store(active, Ordering::Relaxed);
    }

    /// Store a whole [`Controls`] value field-by-field.
    #[inline]
    pub fn store_from(&self, controls: &Controls) {
        for (i, v) in controls.knobs.iter().enumerate() {
            self.set_knob(i, *v);
        }
        for (i, v) in controls.cv.iter().enumerate() {
            self.set_cv(i, *v);
        }
        self.set_gate1(controls.gate1);
        self.set_gate2(controls.gate2);
        self.set_button(controls.button);
        self.set_switch(controls.switch);
    }

    /// Load a plain [`Controls`] snapshot — what the audio ISR passes to the
    /// processor's `process` each block.
    #[inline]
    pub fn load(&self) -> Controls {
        Controls {
            knobs: [
                f32::from_bits(self.knobs[0].load(Ordering::Relaxed)),
                f32::from_bits(self.knobs[1].load(Ordering::Relaxed)),
                f32::from_bits(self.knobs[2].load(Ordering::Relaxed)),
                f32::from_bits(self.knobs[3].load(Ordering::Relaxed)),
            ],
            cv: [
                f32::from_bits(self.cv[0].load(Ordering::Relaxed)),
                f32::from_bits(self.cv[1].load(Ordering::Relaxed)),
                f32::from_bits(self.cv[2].load(Ordering::Relaxed)),
                f32::from_bits(self.cv[3].load(Ordering::Relaxed)),
            ],
            gate1: self.gate1.load(Ordering::Relaxed),
            gate2: self.gate2.load(Ordering::Relaxed),
            button: self.button.load(Ordering::Relaxed),
            switch: self.switch_state.load(Ordering::Relaxed),
        }
    }
}

impl Default for AtomicControls {
    fn default() -> Self {
        Self::new()
    }
}

/// Lock-free shared storage for [`Outputs`] — the reverse of [`AtomicControls`]:
/// the audio ISR (or anything else) writes, the main loop reads and applies to
/// hardware once per control tick (~1 kHz).
pub struct AtomicOutputs {
    led: AtomicU32,
    cv_out: AtomicU32,
    gate_out1: AtomicBool,
    gate_out2: AtomicBool,
}

impl AtomicOutputs {
    /// Create storage with everything off (matches `Outputs::default`).
    pub const fn new() -> Self {
        const ZERO: u32 = 0.0f32.to_bits();
        Self {
            led: AtomicU32::new(ZERO),
            cv_out: AtomicU32::new(ZERO),
            gate_out1: AtomicBool::new(false),
            gate_out2: AtomicBool::new(false),
        }
    }

    /// Store the LED brightness (0.0 to 1.0; clamped when applied to hardware).
    #[inline]
    pub fn set_led(&self, brightness: f32) {
        self.led.store(brightness.to_bits(), Ordering::Relaxed);
    }

    /// Store the CV output level (0.0 to 1.0; clamped when applied to hardware).
    #[inline]
    pub fn set_cv_out(&self, level: f32) {
        self.cv_out.store(level.to_bits(), Ordering::Relaxed);
    }

    /// Store the gate output 1 level.
    #[inline]
    pub fn set_gate_out1(&self, active: bool) {
        self.gate_out1.store(active, Ordering::Relaxed);
    }

    /// Store the gate output 2 level.
    #[inline]
    pub fn set_gate_out2(&self, active: bool) {
        self.gate_out2.store(active, Ordering::Relaxed);
    }

    /// Store a whole [`Outputs`] value field-by-field.
    #[inline]
    pub fn store_from(&self, outputs: &Outputs) {
        self.set_led(outputs.led);
        self.set_cv_out(outputs.cv_out);
        self.set_gate_out1(outputs.gate_out1);
        self.set_gate_out2(outputs.gate_out2);
    }

    /// Load a plain [`Outputs`] snapshot — what the main loop applies to hardware.
    #[inline]
    pub fn load(&self) -> Outputs {
        Outputs {
            led: f32::from_bits(self.led.load(Ordering::Relaxed)),
            cv_out: f32::from_bits(self.cv_out.load(Ordering::Relaxed)),
            gate_out1: self.gate_out1.load(Ordering::Relaxed),
            gate_out2: self.gate_out2.load(Ordering::Relaxed),
        }
    }
}

impl Default for AtomicOutputs {
    fn default() -> Self {
        Self::new()
    }
}

/// The global output store applied to hardware by the entry macros' main loop.
static OUTPUTS: AtomicOutputs = AtomicOutputs::new();

/// Access the global [`AtomicOutputs`] store.
///
/// Call from anywhere — typically inside `AudioProcessor::process` — to drive
/// the LED, CV output, and gate outputs on boards that have them:
///
/// ```ignore
/// // Strike feedback: LED brightness follows the envelope.
/// bbx_daisy::outputs().set_led(envelope);
/// bbx_daisy::outputs().set_gate_out1(triggered);
/// ```
///
/// Writes are applied by the main loop at the control rate (~1 kHz). On boards
/// without a given output the value is simply never read.
#[inline]
pub fn outputs() -> &'static AtomicOutputs {
    &OUTPUTS
}
