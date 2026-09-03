//! DAC abstractions for CV outputs.
//!
//! The STM32H750's DAC1 drives the Patch SM's two CV outputs through an
//! op-amp stage that scales the 0-3.3 V pin swing to 0-5 V at the jack.
//! On the Patch.Init(), channel 1 is the CV OUT jack and channel 2 drives the
//! front-panel LED — which means LED brightness is analog for free.

use stm32h7xx_hal::traits::DacOut;

/// Full-scale voltage at a CV output jack when the DAC is at maximum.
pub const CV_OUT_FULL_SCALE_VOLTS: f32 = 5.0;

/// Maximum 12-bit DAC code.
const DAC_MAX: u16 = 4095;

/// One enabled DAC channel driving a CV output (or the Patch.Init panel LED).
///
/// Wraps an enabled `stm32h7xx-hal` DAC channel with clamped, unit-aware
/// setters so callers never handle raw codes unless they want to.
pub struct CvOut<C> {
    channel: C,
}

impl<C: DacOut<u16>> CvOut<C> {
    /// Wrap an enabled DAC channel.
    pub fn new(channel: C) -> Self {
        Self { channel }
    }

    /// Set the raw 12-bit DAC code (clamped to 0-4095).
    #[inline]
    pub fn set_raw(&mut self, code: u16) {
        self.channel.set_value(code.min(DAC_MAX));
    }

    /// Set a normalized level: 0.0 (0 V) to 1.0 (full scale ≈ 5 V at the jack).
    ///
    /// Values outside 0.0-1.0 are clamped; NaN reads as 0.
    #[inline]
    pub fn set_norm(&mut self, level: f32) {
        let clamped = if level > 0.0 { level.min(1.0) } else { 0.0 };
        self.set_raw((clamped * DAC_MAX as f32) as u16);
    }

    /// Set the output in jack volts (clamped to 0-5 V).
    #[inline]
    pub fn set_volts(&mut self, volts: f32) {
        self.set_norm(volts / CV_OUT_FULL_SCALE_VOLTS);
    }

    /// Digital on/off — 0 or full scale. Handy when the channel drives the LED.
    #[inline]
    pub fn set(&mut self, on: bool) {
        self.set_raw(if on { DAC_MAX } else { 0 });
    }

    /// Release the underlying DAC channel.
    pub fn release(self) -> C {
        self.channel
    }
}
