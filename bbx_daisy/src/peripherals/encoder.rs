//! Rotary encoder abstraction with quadrature decoding.
//!
//! This module provides support for reading rotary encoders with push buttons,
//! commonly found on Daisy Pod and similar hardware.

use embedded_hal::digital::v2::InputPin;

// ============================================================================
// Encoder Direction
// ============================================================================

/// Direction of encoder rotation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Direction {
    /// No movement detected
    None,
    /// Clockwise rotation
    Clockwise,
    /// Counter-clockwise rotation
    CounterClockwise,
}

impl Direction {
    /// Convert direction to a delta value (-1, 0, or +1).
    #[inline]
    pub fn as_delta(self) -> i32 {
        match self {
            Direction::None => 0,
            Direction::Clockwise => 1,
            Direction::CounterClockwise => -1,
        }
    }
}

// ============================================================================
// Quadrature Encoder (Gray Code State Machine)
// ============================================================================

/// Quadrature rotary encoder with Gray code state machine.
///
/// Uses polling-based reading suitable for UI encoders at control rate.
/// The state machine handles bouncing and provides reliable direction detection.
///
/// # Example
///
/// ```ignore
/// use bbx_daisy::peripherals::encoder::Encoder;
///
/// let mut encoder = Encoder::new(pin_a, pin_b);
///
/// // In control loop (1kHz recommended):
/// let direction = encoder.update();
/// match direction {
///     Direction::Clockwise => value += 1,
///     Direction::CounterClockwise => value -= 1,
///     Direction::None => {},
/// }
/// ```
pub struct Encoder<A, B> {
    pin_a: A,
    pin_b: B,
    state: u8,
    position: i32,
}

impl<A: InputPin, B: InputPin> Encoder<A, B> {
    /// Create a new encoder from two GPIO pins.
    pub fn new(pin_a: A, pin_b: B) -> Self {
        Self {
            pin_a,
            pin_b,
            state: 0,
            position: 0,
        }
    }

    /// Update the encoder state and return the direction of rotation.
    ///
    /// Call this at a regular rate (1kHz recommended for UI encoders).
    pub fn update(&mut self) -> Direction {
        // Read current pin states
        let a = self.pin_a.is_high().unwrap_or(false);
        let b = self.pin_b.is_high().unwrap_or(false);

        // Build 2-bit gray code state
        let new_state = ((a as u8) << 1) | (b as u8);

        // Gray code state transitions:
        // CW:  00 -> 01 -> 11 -> 10 -> 00
        // CCW: 00 -> 10 -> 11 -> 01 -> 00

        let direction = match (self.state, new_state) {
            // Clockwise transitions
            (0b00, 0b01) | (0b01, 0b11) | (0b11, 0b10) | (0b10, 0b00) => {
                self.position = self.position.wrapping_add(1);
                Direction::Clockwise
            }
            // Counter-clockwise transitions
            (0b00, 0b10) | (0b10, 0b11) | (0b11, 0b01) | (0b01, 0b00) => {
                self.position = self.position.wrapping_sub(1);
                Direction::CounterClockwise
            }
            // No change or invalid transition
            _ => Direction::None,
        };

        self.state = new_state;
        direction
    }

    /// Get the current position (accumulated delta since creation).
    #[inline]
    pub fn position(&self) -> i32 {
        self.position
    }

    /// Reset the position counter to zero.
    pub fn reset_position(&mut self) {
        self.position = 0;
    }

    /// Set the position counter to a specific value.
    pub fn set_position(&mut self, position: i32) {
        self.position = position;
    }

    /// Release the underlying pins.
    pub fn release(self) -> (A, B) {
        (self.pin_a, self.pin_b)
    }
}

// ============================================================================
// Encoder with Button
// ============================================================================

/// Rotary encoder with integrated push button.
///
/// Combines quadrature encoder with debounced button for complete
/// encoder control typically found on audio hardware.
///
/// # Example
///
/// ```ignore
/// use bbx_daisy::peripherals::encoder::EncoderWithButton;
///
/// let mut encoder = EncoderWithButton::new(pin_a, pin_b, pin_sw);
///
/// // In control loop:
/// let (direction, pressed) = encoder.update();
/// if pressed {
///     // Handle button press
/// }
/// ```
pub struct EncoderWithButton<A, B, S> {
    encoder: Encoder<A, B>,
    switch: S,
    switch_state: bool,
    switch_debounce: u8,
    debounce_threshold: u8,
    pressed_event: bool,
    released_event: bool,
}

impl<A: InputPin, B: InputPin, S: InputPin> EncoderWithButton<A, B, S> {
    /// Create a new encoder with button.
    ///
    /// The switch is assumed to be active-low (pressed = low).
    pub fn new(pin_a: A, pin_b: B, switch: S) -> Self {
        Self {
            encoder: Encoder::new(pin_a, pin_b),
            switch,
            switch_state: false,
            switch_debounce: 0,
            debounce_threshold: 5,
            pressed_event: false,
            released_event: false,
        }
    }

    /// Update encoder and button state.
    ///
    /// Returns (direction, button_pressed).
    pub fn update(&mut self) -> (Direction, bool) {
        let direction = self.encoder.update();

        // Update button with debouncing
        let raw_pressed = !self.switch.is_high().unwrap_or(true); // Active low

        self.pressed_event = false;
        self.released_event = false;

        if raw_pressed != self.switch_state {
            self.switch_debounce += 1;
            if self.switch_debounce >= self.debounce_threshold {
                let was_pressed = self.switch_state;
                self.switch_state = raw_pressed;
                self.switch_debounce = 0;

                if raw_pressed && !was_pressed {
                    self.pressed_event = true;
                } else if !raw_pressed && was_pressed {
                    self.released_event = true;
                }
            }
        } else {
            self.switch_debounce = 0;
        }

        (direction, self.switch_state)
    }

    /// Check if the button was just pressed (edge detection).
    #[inline]
    pub fn just_pressed(&self) -> bool {
        self.pressed_event
    }

    /// Check if the button was just released (edge detection).
    #[inline]
    pub fn just_released(&self) -> bool {
        self.released_event
    }

    /// Check if the button is currently pressed.
    #[inline]
    pub fn is_pressed(&self) -> bool {
        self.switch_state
    }

    /// Get the encoder position.
    #[inline]
    pub fn position(&self) -> i32 {
        self.encoder.position()
    }

    /// Reset the encoder position.
    pub fn reset_position(&mut self) {
        self.encoder.reset_position();
    }

    /// Get a reference to the underlying encoder.
    pub fn encoder(&self) -> &Encoder<A, B> {
        &self.encoder
    }

    /// Get a mutable reference to the underlying encoder.
    pub fn encoder_mut(&mut self) -> &mut Encoder<A, B> {
        &mut self.encoder
    }
}

// ============================================================================
// Velocity-Sensitive Encoder
// ============================================================================

/// Encoder with velocity/acceleration detection.
///
/// Tracks rotation speed to allow faster parameter changes when
/// the encoder is turned quickly.
pub struct VelocityEncoder<A, B> {
    encoder: Encoder<A, B>,
    velocity: i32,
    last_direction: Direction,
    same_direction_count: u8,
    acceleration_threshold: u8,
    max_velocity: i32,
}

impl<A: InputPin, B: InputPin> VelocityEncoder<A, B> {
    /// Create a new velocity-sensitive encoder.
    pub fn new(pin_a: A, pin_b: B) -> Self {
        Self {
            encoder: Encoder::new(pin_a, pin_b),
            velocity: 1,
            last_direction: Direction::None,
            same_direction_count: 0,
            acceleration_threshold: 3,
            max_velocity: 10,
        }
    }

    /// Set the maximum velocity multiplier.
    pub fn with_max_velocity(mut self, max: i32) -> Self {
        self.max_velocity = max.max(1);
        self
    }

    /// Update and return the velocity-adjusted delta.
    ///
    /// Returns a larger delta when the encoder is turned quickly.
    pub fn update(&mut self) -> i32 {
        let direction = self.encoder.update();

        if direction == Direction::None {
            // Decay velocity when not moving
            if self.same_direction_count > 0 {
                self.same_direction_count = self.same_direction_count.saturating_sub(1);
            }
            if self.same_direction_count == 0 {
                self.velocity = 1;
            }
            return 0;
        }

        // Track consecutive movements in the same direction
        if direction == self.last_direction {
            self.same_direction_count = self.same_direction_count.saturating_add(1);

            // Increase velocity after threshold
            if self.same_direction_count >= self.acceleration_threshold {
                self.velocity = (self.velocity + 1).min(self.max_velocity);
                self.same_direction_count = 0;
            }
        } else {
            // Direction changed, reset velocity
            self.velocity = 1;
            self.same_direction_count = 0;
        }

        self.last_direction = direction;

        // Return velocity-adjusted delta
        direction.as_delta() * self.velocity
    }

    /// Get the current velocity multiplier.
    #[inline]
    pub fn velocity(&self) -> i32 {
        self.velocity
    }

    /// Get the encoder position.
    #[inline]
    pub fn position(&self) -> i32 {
        self.encoder.position()
    }
}
