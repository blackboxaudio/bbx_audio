//! User LED abstraction for Daisy boards.
//!
//! Provides a simple high-level interface for the on-board user LED (PC7).
//!
//! # Example
//!
//! ```ignore
//! use bbx_daisy::prelude::*;
//!
//! let board = Board::take().unwrap();
//! let mut led = UserLed::new(board.gpioc.pc7);
//!
//! led.on();
//! board.delay.delay_ms(500u32);
//! led.off();
//! board.delay.delay_ms(500u32);
//! led.toggle();
//! ```

use crate::hal::gpio::{self, Output, PinMode, PushPull};

/// User LED pin type (PC7, push-pull output).
pub type UserLedPin = gpio::gpioc::PC7<Output<PushPull>>;

/// User LED abstraction.
///
/// Wraps the on-board LED (PC7) with a simple on/off/toggle interface.
pub struct UserLed {
    pin: UserLedPin,
}

impl UserLed {
    /// Create a new UserLed from any PC7 pin mode.
    ///
    /// The pin is automatically converted to push-pull output mode.
    pub fn new<MODE: PinMode>(pin: gpio::gpioc::PC7<MODE>) -> Self {
        Self {
            pin: pin.into_push_pull_output(),
        }
    }

    /// Turn the LED on.
    #[inline]
    pub fn on(&mut self) {
        self.pin.set_high();
    }

    /// Turn the LED off.
    #[inline]
    pub fn off(&mut self) {
        self.pin.set_low();
    }

    /// Toggle the LED state.
    #[inline]
    pub fn toggle(&mut self) {
        self.pin.toggle();
    }

    /// Set the LED state directly.
    #[inline]
    pub fn set(&mut self, on: bool) {
        if on {
            self.on();
        } else {
            self.off();
        }
    }

    /// Get access to the underlying pin for advanced use.
    #[inline]
    pub fn into_inner(self) -> UserLedPin {
        self.pin
    }
}
