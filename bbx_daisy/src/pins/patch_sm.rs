//! Patch SM (Submodule) pin mappings.
//!
//! The Patch SM is a surface-mount module designed for integration into custom
//! hardware. It's the brain of the Patch.Init() Eurorack module, whose panel
//! wiring is noted per pin below.
//!
//! Pin assignments are cross-checked against libDaisy's `daisy_patch_sm.h/.cpp`
//! (the authoritative board definition). Header names (`B5`, `C10`, …) refer to
//! the Patch SM's A/B/C/D expansion headers.
//!
//! # Patch.Init() control surface
//!
//! - 4 panel knobs → SM channels CV_1-4 ([`Cv1`]-[`Cv4`])
//! - 4 panel CV jacks → SM channels CV_5-8 ([`Cv5`]-[`Cv8`]) — separate channels, *not* analog-summed with the knobs
//! - B7 momentary button ([`ButtonB7`]), B8 toggle ([`SwitchB8`])
//! - 2 gate inputs ([`GateIn1`]/[`GateIn2`]), 2 gate outputs ([`GateOut1`]/[`GateOut2`])
//! - CV OUT jack = DAC channel 1 ([`CvOut1`]); front-panel LED = DAC channel 2 ([`CvOut2`])
//! - The SM's own tiny onboard LED is PC7 (see [`crate::led::UserLed`])
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
// CV Inputs (ADC) — bipolar ±5V channels
// ============================================================================
//
// All eight CV_x channels run through the module's bipolar (-5V to +5V) input
// stage, which is INVERTING: +5V at the jack pulls the ADC pin low. Readers
// must invert and re-center (see `peripherals::adc::patch_sm_bipolar`).
// On the Patch.Init(), CV_1-4 are the panel knobs and CV_5-8 are the CV jacks.

/// CV_1 (PA3, header C5) - Patch.Init() panel knob 1
pub type Cv1 = gpio::PA3<Analog>;

/// CV_2 (PA6, header C4) - Patch.Init() panel knob 2
pub type Cv2 = gpio::PA6<Analog>;

/// CV_3 (PA2, header C3) - Patch.Init() panel knob 3
pub type Cv3 = gpio::PA2<Analog>;

/// CV_4 (PA7, header C2) - Patch.Init() panel knob 4
pub type Cv4 = gpio::PA7<Analog>;

/// CV_5 (PC1, header C9) - Patch.Init() panel CV jack 1
pub type Cv5 = gpio::PC1<Analog>;

/// CV_6 (PC0, header C8) - Patch.Init() panel CV jack 2
pub type Cv6 = gpio::PC0<Analog>;

/// CV_7 (PB1, header C6) - Patch.Init() panel CV jack 3
pub type Cv7 = gpio::PB1<Analog>;

/// CV_8 (PC4, header C7) - Patch.Init() panel CV jack 4
pub type Cv8 = gpio::PC4<Analog>;

// ============================================================================
// Additional ADC Inputs — unipolar 0-3.3V, no input conditioning
// ============================================================================

/// ADC_9 (PA1, header A2 - shared with UART1 RX) - Unipolar 0V to 3.3V
pub type Adc9 = gpio::PA1<Analog>;

/// ADC_10 (PA0, header A3 - shared with UART1 TX) - Unipolar 0V to 3.3V
pub type Adc10 = gpio::PA0<Analog>;

// ADC_11 / ADC_12 live on the D-column SPI2 header pins (D9/D8) and are not
// aliased here; add them alongside an SPI abstraction if ever needed.

// ============================================================================
// CV Outputs (DAC1) — 0V to 5V via the module's op-amp output stage
// ============================================================================

/// CV_OUT_1 (PA4, DAC1_OUT1, header C10) - the Patch.Init() CV OUT jack
pub type CvOut1 = gpio::PA4<Analog>;

/// CV_OUT_2 (PA5, DAC1_OUT2, header C1) - drives the Patch.Init() front-panel LED
pub type CvOut2 = gpio::PA5<Analog>;

// ============================================================================
// Gate Inputs
// ============================================================================
//
// The module's transistor input stage is INVERTING: a high gate at the jack
// pulls the MCU pin low, so wrap these in `GateIn::new_active_low`. No pull
// resistor is needed — the input stage drives the pin.

/// Gate Input 1 (PG13, header B10)
pub type GateIn1 = gpio::PG13<Input>;

/// Gate Input 2 (PG14, header B9)
pub type GateIn2 = gpio::PG14<Input>;

// ============================================================================
// Gate Outputs
// ============================================================================

/// Gate Output 1 (PC14, header B5)
pub type GateOut1 = gpio::PC14<Output<PushPull>>;

/// Gate Output 2 (PC13, header B6)
pub type GateOut2 = gpio::PC13<Output<PushPull>>;

// ============================================================================
// MIDI Input
// ============================================================================

/// MIDI Input RX (PD6, USART2_RX)
pub type MidiRx = gpio::PD6<Alternate<7>>;

// ============================================================================
// Buttons / Switches
// ============================================================================

/// B7 momentary button pin (PB8, also I2C1 SCL) - pull-up input, read active-low.
///
/// On the Patch.Init() this is the front-panel push button.
pub type ButtonB7 = gpio::PB8<Input>;

/// B8 toggle/switch pin (PB9, also I2C1 SDA) - pull-up input, read active-low.
///
/// On the Patch.Init() this is the front-panel toggle switch.
pub type SwitchB8 = gpio::PB9<Input>;
