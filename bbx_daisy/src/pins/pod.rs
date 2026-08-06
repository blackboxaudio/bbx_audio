//! Daisy Pod pin mappings.
//!
//! The Daisy Pod is a development board with:
//!
//! - 2 encoders with push buttons
//! - 2 potentiometers (knobs)
//! - 2 toggle switches
//! - 2 LEDs (accent color selectable)
//! - 3.5mm audio I/O jacks

use stm32h7xx_hal::gpio::{self, Alternate, Analog, Input, Output, PushPull};

// ============================================================================
// Audio Pins (SAI1 - Same as Seed)
// ============================================================================

/// SAI1 Master Clock (PE2)
pub type Sai1Mclk = gpio::PE2<Alternate<6>>;

/// SAI1 Bit Clock (PE5)
pub type Sai1Sck = gpio::PE5<Alternate<6>>;

/// SAI1 Frame Sync / LRCLK (PE4)
pub type Sai1Fs = gpio::PE4<Alternate<6>>;

/// SAI1 Serial Data A (PE6) - Transmit to codec DAC
pub type Sai1SdA = gpio::PE6<Alternate<6>>;

/// SAI1 Serial Data B (PE3) - Receive from codec ADC
pub type Sai1SdB = gpio::PE3<Alternate<6>>;

/// Collected SAI1 pins for audio interface initialization.
pub struct Sai1Pins {
    pub mclk: Sai1Mclk,
    pub sck: Sai1Sck,
    pub fs: Sai1Fs,
    pub sd_a: Sai1SdA,
    pub sd_b: Sai1SdB,
}

// ============================================================================
// I2C Pins (for WM8731 codec)
// ============================================================================

/// I2C4 SCL (PH11) - Codec I2C clock
pub type I2c4Scl = gpio::PH11<Alternate<4>>;

/// I2C4 SDA (PH12) - Codec I2C data
pub type I2c4Sda = gpio::PH12<Alternate<4>>;

/// Collected I2C4 pins for codec control.
pub struct I2c4Pins {
    pub scl: I2c4Scl,
    pub sda: I2c4Sda,
}

// ============================================================================
// Encoder 1 Pins
// ============================================================================

/// Encoder 1 - Channel A (PD11)
pub type Encoder1A = gpio::PD11<Input>;

/// Encoder 1 - Channel B (PD10)
pub type Encoder1B = gpio::PD10<Input>;

/// Encoder 1 - Switch (PD12)
pub type Encoder1Sw = gpio::PD12<Input>;

/// Collected Encoder 1 pins.
pub struct Encoder1Pins {
    pub a: Encoder1A,
    pub b: Encoder1B,
    pub sw: Encoder1Sw,
}

// ============================================================================
// Encoder 2 Pins
// ============================================================================

/// Encoder 2 - Channel A (PA2)
pub type Encoder2A = gpio::PA2<Input>;

/// Encoder 2 - Channel B (PA1)
pub type Encoder2B = gpio::PA1<Input>;

/// Encoder 2 - Switch (PA0)
pub type Encoder2Sw = gpio::PA0<Input>;

/// Collected Encoder 2 pins.
pub struct Encoder2Pins {
    pub a: Encoder2A,
    pub b: Encoder2B,
    pub sw: Encoder2Sw,
}

// ============================================================================
// Potentiometer (Knob) Pins
// ============================================================================

/// Knob 1 - ADC input (PC4, ADC1_IN4)
pub type Knob1Pin = gpio::PC4<Analog>;

/// Knob 2 - ADC input (PC0, ADC1_IN10)
pub type Knob2Pin = gpio::PC0<Analog>;

// ============================================================================
// Toggle Switch Pins
// ============================================================================

/// Toggle Switch 1 (PG14)
pub type Switch1Pin = gpio::PG14<Input>;

/// Toggle Switch 2 (PB5)
pub type Switch2Pin = gpio::PB5<Input>;

// ============================================================================
// LED Pins (Active-Low - Pod LEDs are common anode)
// ============================================================================

/// LED 1 - Red (PC1)
pub type Led1Red = gpio::PC1<Output<PushPull>>;

/// LED 1 - Green (PA6)
pub type Led1Green = gpio::PA6<Output<PushPull>>;

/// LED 1 - Blue (PA7)
pub type Led1Blue = gpio::PA7<Output<PushPull>>;

/// LED 2 - Red (PB1)
pub type Led2Red = gpio::PB1<Output<PushPull>>;

/// LED 2 - Green (PA1)
pub type Led2Green = gpio::PA1<Output<PushPull>>;

/// LED 2 - Blue (PA4)
pub type Led2Blue = gpio::PA4<Output<PushPull>>;

/// Collected LED 1 pins (RGB).
pub struct Led1Pins {
    pub red: Led1Red,
    pub green: Led1Green,
    pub blue: Led1Blue,
}

/// Collected LED 2 pins (RGB).
pub struct Led2Pins {
    pub red: Led2Red,
    pub green: Led2Green,
    pub blue: Led2Blue,
}

// ============================================================================
// Pod Board Aggregates
// ============================================================================

/// All Pod control pins collected together.
pub struct PodPins {
    pub encoder1: Encoder1Pins,
    pub encoder2: Encoder2Pins,
    pub knob1: Knob1Pin,
    pub knob2: Knob2Pin,
    pub switch1: Switch1Pin,
    pub switch2: Switch2Pin,
    pub led1: Led1Pins,
    pub led2: Led2Pins,
    pub codec_i2c: I2c4Pins,
    pub audio: Sai1Pins,
}
