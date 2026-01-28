//! Patch SM (Submodule) pin mappings.
//!
//! The Patch SM is a surface-mount module designed for integration
//! into custom hardware. It's used by the Patch.Init() and other products.
//!
//! # Features
//!
//! - 12 CV inputs (ADC)
//! - 2 CV outputs (DAC)
//! - 4 gate inputs
//! - 2 gate outputs
//! - MIDI input
//! - Stereo audio I/O (PCM3060 codec)

use stm32h7xx_hal::gpio::{self, Alternate, Analog, Input, Output, PushPull};

// ============================================================================
// Audio Pins (SAI1)
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
// I2C Pins (for PCM3060 codec)
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
// CV Inputs (ADC)
// ============================================================================

/// CV Input 1 (PC0, ADC1_IN10) - Bipolar -5V to +5V
pub type Cv1 = gpio::PC0<Analog>;

/// CV Input 2 (PA3, ADC1_IN15) - Bipolar -5V to +5V
pub type Cv2 = gpio::PA3<Analog>;

/// CV Input 3 (PB1, ADC1_IN9) - Bipolar -5V to +5V
pub type Cv3 = gpio::PB1<Analog>;

/// CV Input 4 (PA7, ADC1_IN7) - Bipolar -5V to +5V
pub type Cv4 = gpio::PA7<Analog>;

/// CV Input 5 (PA6, ADC1_IN3) - Unipolar 0V to 5V
pub type Cv5 = gpio::PA6<Analog>;

/// CV Input 6 (PC1, ADC1_IN11) - Unipolar 0V to 5V
pub type Cv6 = gpio::PC1<Analog>;

/// CV Input 7 (PC4, ADC1_IN4) - Unipolar 0V to 5V
pub type Cv7 = gpio::PC4<Analog>;

/// CV Input 8 (PA5, ADC1_IN5) - Unipolar 0V to 5V
pub type Cv8 = gpio::PA5<Analog>;

/// CV Input 9 (PA4, ADC1_IN4) - Unipolar 0V to 5V
pub type Cv9 = gpio::PA4<Analog>;

/// CV Input 10 (PA1, ADC1_IN1) - Unipolar 0V to 5V
pub type Cv10 = gpio::PA1<Analog>;

/// CV Input 11 (PA0, ADC1_IN0) - Unipolar 0V to 5V
pub type Cv11 = gpio::PA0<Analog>;

/// CV Input 12 (PA2, ADC1_IN14) - Unipolar 0V to 5V
pub type Cv12 = gpio::PA2<Analog>;

// ============================================================================
// CV Outputs (DAC)
// ============================================================================

/// CV Output 1 (PA4, DAC1_OUT1) - 0V to 5V output
pub type CvOut1 = gpio::PA4<Analog>;

/// CV Output 2 (PA5, DAC1_OUT2) - 0V to 5V output
pub type CvOut2 = gpio::PA5<Analog>;

// ============================================================================
// Gate Inputs
// ============================================================================

/// Gate Input 1 (PB5)
pub type GateIn1 = gpio::PB5<Input>;

/// Gate Input 2 (PB4)
pub type GateIn2 = gpio::PB4<Input>;

/// Gate Input 3 (PG6)
pub type GateIn3 = gpio::PG6<Input>;

/// Gate Input 4 (PG7)
pub type GateIn4 = gpio::PG7<Input>;

// ============================================================================
// Gate Outputs
// ============================================================================

/// Gate Output 1 (PB6)
pub type GateOut1 = gpio::PB6<Output<PushPull>>;

/// Gate Output 2 (PB7)
pub type GateOut2 = gpio::PB7<Output<PushPull>>;

// ============================================================================
// MIDI Input
// ============================================================================

/// MIDI Input RX (PD6, USART2_RX)
pub type MidiRx = gpio::PD6<Alternate<7>>;

// ============================================================================
// User LED
// ============================================================================

/// User LED pin (PC7) - active high
pub type UserLedPin = gpio::PC7<Output<PushPull>>;

// ============================================================================
// Patch SM Board Aggregates
// ============================================================================

/// All CV input pins.
pub struct CvInputs {
    pub cv1: Cv1,
    pub cv2: Cv2,
    pub cv3: Cv3,
    pub cv4: Cv4,
    pub cv5: Cv5,
    pub cv6: Cv6,
    pub cv7: Cv7,
    pub cv8: Cv8,
    pub cv9: Cv9,
    pub cv10: Cv10,
    pub cv11: Cv11,
    pub cv12: Cv12,
}

/// All CV output pins.
pub struct CvOutputs {
    pub cv_out1: CvOut1,
    pub cv_out2: CvOut2,
}

/// All gate input pins.
pub struct GateInputs {
    pub gate1: GateIn1,
    pub gate2: GateIn2,
    pub gate3: GateIn3,
    pub gate4: GateIn4,
}

/// All gate output pins.
pub struct GateOutputs {
    pub gate1: GateOut1,
    pub gate2: GateOut2,
}

/// All Patch SM pins collected together.
pub struct PatchSmPins {
    pub cv_inputs: CvInputs,
    pub cv_outputs: CvOutputs,
    pub gate_inputs: GateInputs,
    pub gate_outputs: GateOutputs,
    pub midi_rx: MidiRx,
    pub user_led: UserLedPin,
    pub codec_i2c: I2c4Pins,
    pub audio: Sai1Pins,
}
