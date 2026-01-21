//! GPIO abstractions for LEDs, buttons, and digital I/O.
//!
//! This module provides ergonomic wrappers around GPIO pins for common
//! use cases on Daisy hardware.

use embedded_hal::digital::v2::{InputPin, OutputPin, StatefulOutputPin, ToggleableOutputPin};

// ============================================================================
// LED Abstraction
// ============================================================================

/// LED wrapper providing convenient control methods.
///
/// Works with any pin implementing `OutputPin` and `StatefulOutputPin`.
///
/// # Example
///
/// ```ignore
/// use bbx_daisy::peripherals::gpio::Led;
///
/// let mut led = Led::new(user_led_pin);
/// led.on();
/// led.toggle();
/// ```
pub struct Led<P> {
    pin: P,
}

impl<P: OutputPin> Led<P> {
    /// Create a new LED wrapper around an output pin.
    pub fn new(pin: P) -> Self {
        Self { pin }
    }

    /// Turn the LED on.
    #[inline]
    pub fn on(&mut self) {
        let _ = self.pin.set_high();
    }

    /// Turn the LED off.
    #[inline]
    pub fn off(&mut self) {
        let _ = self.pin.set_low();
    }

    /// Set the LED state (true = on, false = off).
    #[inline]
    pub fn set(&mut self, on: bool) {
        if on {
            self.on();
        } else {
            self.off();
        }
    }

    /// Release the underlying pin.
    pub fn release(self) -> P {
        self.pin
    }
}

impl<P: ToggleableOutputPin> Led<P> {
    /// Toggle the LED state.
    #[inline]
    pub fn toggle(&mut self) {
        let _ = self.pin.toggle();
    }
}

impl<P: StatefulOutputPin> Led<P> {
    /// Check if the LED is currently on.
    #[inline]
    pub fn is_on(&self) -> bool {
        self.pin.is_set_high().unwrap_or(false)
    }
}

// ============================================================================
// Button Abstraction
// ============================================================================

/// Button wrapper with optional debouncing.
///
/// Works with any pin implementing `InputPin`.
///
/// # Example
///
/// ```ignore
/// use bbx_daisy::peripherals::gpio::Button;
///
/// let mut button = Button::new(button_pin);
/// if button.is_pressed() {
///     // Handle button press
/// }
/// ```
pub struct Button<P> {
    pin: P,
    active_low: bool,
    debounce_count: u8,
    debounce_threshold: u8,
    last_state: bool,
    stable_state: bool,
}

impl<P: InputPin> Button<P> {
    /// Create a new button wrapper (active high by default).
    pub fn new(pin: P) -> Self {
        Self {
            pin,
            active_low: false,
            debounce_count: 0,
            debounce_threshold: 5,
            last_state: false,
            stable_state: false,
        }
    }

    /// Create a new active-low button (pressed = low signal).
    pub fn new_active_low(pin: P) -> Self {
        Self {
            pin,
            active_low: true,
            debounce_count: 0,
            debounce_threshold: 5,
            last_state: false,
            stable_state: false,
        }
    }

    /// Set the debounce threshold (number of consistent readings required).
    pub fn with_debounce(mut self, threshold: u8) -> Self {
        self.debounce_threshold = threshold;
        self
    }

    /// Read the raw button state (without debouncing).
    #[inline]
    pub fn is_pressed_raw(&self) -> bool {
        let high = self.pin.is_high().unwrap_or(false);
        if self.active_low { !high } else { high }
    }

    /// Update debounce state and return the stable pressed state.
    ///
    /// Call this at a regular rate (e.g., 1kHz) for reliable debouncing.
    pub fn update(&mut self) -> bool {
        let current = self.is_pressed_raw();

        if current == self.last_state {
            if self.debounce_count < self.debounce_threshold {
                self.debounce_count += 1;
                if self.debounce_count >= self.debounce_threshold {
                    self.stable_state = current;
                }
            }
        } else {
            self.debounce_count = 0;
            self.last_state = current;
        }

        self.stable_state
    }

    /// Get the current stable (debounced) button state.
    #[inline]
    pub fn is_pressed(&self) -> bool {
        self.stable_state
    }

    /// Check if the button was just pressed (rising edge).
    ///
    /// Returns true once when the button transitions from released to pressed.
    /// Must be called after `update()`.
    pub fn just_pressed(&mut self) -> bool {
        let current = self.is_pressed_raw();
        let was_released = !self.stable_state;

        if current && was_released && self.debounce_count >= self.debounce_threshold {
            true
        } else {
            false
        }
    }

    /// Release the underlying pin.
    pub fn release(self) -> P {
        self.pin
    }
}

// ============================================================================
// Gate I/O (for Eurorack)
// ============================================================================

/// Gate output for Eurorack modules.
///
/// Provides a simple high/low output typically used for triggers and gates.
pub struct GateOut<P> {
    pin: P,
}

impl<P: OutputPin> GateOut<P> {
    /// Create a new gate output.
    pub fn new(pin: P) -> Self {
        Self { pin }
    }

    /// Set gate high (active).
    #[inline]
    pub fn high(&mut self) {
        let _ = self.pin.set_high();
    }

    /// Set gate low (inactive).
    #[inline]
    pub fn low(&mut self) {
        let _ = self.pin.set_low();
    }

    /// Set gate state.
    #[inline]
    pub fn set(&mut self, active: bool) {
        if active {
            self.high();
        } else {
            self.low();
        }
    }

    /// Release the underlying pin.
    pub fn release(self) -> P {
        self.pin
    }
}

/// Gate input for Eurorack modules.
///
/// Reads digital gate/trigger signals with configurable threshold.
pub struct GateIn<P> {
    pin: P,
    active_low: bool,
}

impl<P: InputPin> GateIn<P> {
    /// Create a new gate input (active high).
    pub fn new(pin: P) -> Self {
        Self { pin, active_low: false }
    }

    /// Create a new active-low gate input.
    pub fn new_active_low(pin: P) -> Self {
        Self { pin, active_low: true }
    }

    /// Check if the gate is currently active.
    #[inline]
    pub fn is_active(&self) -> bool {
        let high = self.pin.is_high().unwrap_or(false);
        if self.active_low { !high } else { high }
    }

    /// Release the underlying pin.
    pub fn release(self) -> P {
        self.pin
    }
}
