//! Daisy Seed pin mappings.
//!
//! The Daisy Seed exposes 31 GPIO pins on a 2x20 header plus dedicated audio pins.
//! This module defines type aliases for STM32H750 pin assignments.

use stm32h7xx_hal::gpio::{self, Alternate, Analog, Input, Output, PushPull};

// ============================================================================
// Audio Pins (SAI1 - Fixed on all Seed variants)
// ============================================================================

/// SAI1 Master Clock (PE2) - 12.288 MHz for 48kHz
pub type Sai1Mclk = gpio::PE2<Alternate<6>>;

/// SAI1 Bit Clock (PE5) - 3.072 MHz for 48kHz stereo 24-bit
pub type Sai1Sck = gpio::PE5<Alternate<6>>;

/// SAI1 Frame Sync / LRCLK (PE4) - 48 kHz
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
// I2C Pins (for codec control on Seed 1.1, 1.2)
// ============================================================================

/// I2C4 SCL (PH11) - Codec I2C clock (Seed 1.1, 1.2)
pub type I2c4Scl = gpio::PH11<Alternate<4>>;

/// I2C4 SDA (PH12) - Codec I2C data (Seed 1.1, 1.2)
pub type I2c4Sda = gpio::PH12<Alternate<4>>;

/// Collected I2C4 pins for codec control.
pub struct I2c4Pins {
    pub scl: I2c4Scl,
    pub sda: I2c4Sda,
}

// ============================================================================
// User LED (PC7 on all Seed variants)
// ============================================================================

/// User LED pin (PC7) - active high
pub type UserLedPin = gpio::PC7<Output<PushPull>>;

// ============================================================================
// GPIO Header Pins (D0-D30)
// ============================================================================

/// D0 / Pin 1 - PB12 (SPI2_NSS, UART3_RX, ADC1_IN11)
pub type D0 = gpio::PB12<Input>;
/// D1 / Pin 2 - PC11 (SPI3_MISO, UART3_RX, SDMMC1_D3)
pub type D1 = gpio::PC11<Input>;
/// D2 / Pin 3 - PC10 (SPI3_SCK, UART3_TX, SDMMC1_D2)
pub type D2 = gpio::PC10<Input>;
/// D3 / Pin 4 - PC9 (I2C3_SDA, SDMMC1_D1)
pub type D3 = gpio::PC9<Input>;
/// D4 / Pin 5 - PC8 (I2C3_SCL, SDMMC1_D0)
pub type D4 = gpio::PC8<Input>;
/// D5 / Pin 6 - PD2 (UART5_RX, SDMMC1_CMD)
pub type D5 = gpio::PD2<Input>;
/// D6 / Pin 7 - PC12 (SPI3_MOSI, UART5_TX, SDMMC1_CK)
pub type D6 = gpio::PC12<Input>;
/// D7 / Pin 8 - PG10 (SAI2_SD_B)
pub type D7 = gpio::PG10<Input>;
/// D8 / Pin 9 - PG11 (SAI2_SD_A)
pub type D8 = gpio::PG11<Input>;
/// D9 / Pin 10 - PB4 (SPI3_MISO, SPI1_MISO)
pub type D9 = gpio::PB4<Input>;
/// D10 / Pin 11 - PB5 (SPI3_MOSI, SPI1_MOSI)
pub type D10 = gpio::PB5<Input>;
/// D11 / Pin 12 - PB8 (I2C1_SCL, CAN1_RX)
pub type D11 = gpio::PB8<Input>;
/// D12 / Pin 13 - PB9 (I2C1_SDA, CAN1_TX)
pub type D12 = gpio::PB9<Input>;
/// D13 / Pin 14 - PB6 (UART1_TX, I2C1_SCL)
pub type D13 = gpio::PB6<Input>;
/// D14 / Pin 15 - PB7 (UART1_RX, I2C1_SDA)
pub type D14 = gpio::PB7<Input>;
/// D15 / Pin 16 - PC0 (ADC1_IN10, ADC2_IN10)
pub type D15 = gpio::PC0<Input>;
/// D16 / Pin 17 - PA3 (ADC1_IN15, UART2_RX)
pub type D16 = gpio::PA3<Input>;
/// D17 / Pin 18 - PB1 (ADC1_IN9)
pub type D17 = gpio::PB1<Input>;
/// D18 / Pin 19 - PA7 (ADC1_IN7, SPI1_MOSI)
pub type D18 = gpio::PA7<Input>;
/// D19 / Pin 20 - PA6 (ADC1_IN3, SPI1_MISO)
pub type D19 = gpio::PA6<Input>;
/// D20 / Pin 21 - PC1 (ADC1_IN11, ADC2_IN11, ADC3_IN1)
pub type D20 = gpio::PC1<Input>;
/// D21 / Pin 22 - PC4 (ADC1_IN4, ADC2_IN4)
pub type D21 = gpio::PC4<Input>;
/// D22 / Pin 23 - PA5 (ADC1_IN5, DAC1_OUT2, SPI1_SCK)
pub type D22 = gpio::PA5<Input>;
/// D23 / Pin 24 - PA4 (ADC1_IN4, DAC1_OUT1, SPI1_NSS)
pub type D23 = gpio::PA4<Input>;
/// D24 / Pin 25 - PA1 (ADC1_IN1, UART4_RX)
pub type D24 = gpio::PA1<Input>;
/// D25 / Pin 26 - PA0 (ADC1_IN0, UART4_TX)
pub type D25 = gpio::PA0<Input>;
/// D26 / Pin 27 - PD11 (UART3_CTS, QUADSPI_BK1_IO0)
pub type D26 = gpio::PD11<Input>;
/// D27 / Pin 28 - PG9 (UART6_RX, QUADSPI_BK2_IO2)
pub type D27 = gpio::PG9<Input>;
/// D28 / Pin 29 - PA2 (ADC1_IN14, UART2_TX)
pub type D28 = gpio::PA2<Input>;
/// D29 / Pin 30 - PB14 (SPI2_MISO, UART3_RTS)
pub type D29 = gpio::PB14<Input>;
/// D30 / Pin 31 - PB15 (SPI2_MOSI)
pub type D30 = gpio::PB15<Input>;

// ============================================================================
// ADC Pin Type Aliases (Analog mode)
// ============================================================================

/// ADC input on D15 (PC0) - ADC1_IN10, ADC2_IN10
pub type Adc15 = gpio::PC0<Analog>;
/// ADC input on D16 (PA3) - ADC1_IN15
pub type Adc16 = gpio::PA3<Analog>;
/// ADC input on D17 (PB1) - ADC1_IN9
pub type Adc17 = gpio::PB1<Analog>;
/// ADC input on D18 (PA7) - ADC1_IN7
pub type Adc18 = gpio::PA7<Analog>;
/// ADC input on D19 (PA6) - ADC1_IN3
pub type Adc19 = gpio::PA6<Analog>;
/// ADC input on D20 (PC1) - ADC1_IN11, ADC2_IN11, ADC3_IN1
pub type Adc20 = gpio::PC1<Analog>;
/// ADC input on D21 (PC4) - ADC1_IN4, ADC2_IN4
pub type Adc21 = gpio::PC4<Analog>;
/// ADC input on D22 (PA5) - ADC1_IN5
pub type Adc22 = gpio::PA5<Analog>;
/// ADC input on D23 (PA4) - ADC1_IN4
pub type Adc23 = gpio::PA4<Analog>;
/// ADC input on D24 (PA1) - ADC1_IN1
pub type Adc24 = gpio::PA1<Analog>;
/// ADC input on D25 (PA0) - ADC1_IN0
pub type Adc25 = gpio::PA0<Analog>;
/// ADC input on D28 (PA2) - ADC1_IN14
pub type Adc28 = gpio::PA2<Analog>;

// ============================================================================
// Convenience Type Aliases
// ============================================================================

/// I2C1 SCL on D11 (PB8)
pub type I2c1Scl = gpio::PB8<Alternate<4>>;
/// I2C1 SDA on D12 (PB9)
pub type I2c1Sda = gpio::PB9<Alternate<4>>;

/// SPI1 SCK on D22 (PA5)
pub type Spi1Sck = gpio::PA5<Alternate<5>>;
/// SPI1 MISO on D19 (PA6)
pub type Spi1Miso = gpio::PA6<Alternate<5>>;
/// SPI1 MOSI on D18 (PA7)
pub type Spi1Mosi = gpio::PA7<Alternate<5>>;

/// UART1 TX on D13 (PB6)
pub type Uart1Tx = gpio::PB6<Alternate<7>>;
/// UART1 RX on D14 (PB7)
pub type Uart1Rx = gpio::PB7<Alternate<7>>;
