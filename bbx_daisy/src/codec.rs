//! Audio codec initialization and control.
//!
//! This module provides codec drivers for Daisy hardware variants:
//!
//! | Board         | Codec    | Interface | Bit Depth |
//! |---------------|----------|-----------|-----------|
//! | Seed          | AK4556   | I2S       | 24-bit    |
//! | Seed 1.1      | WM8731   | I2S + I2C | 24-bit    |
//! | Seed 1.2      | PCM3060  | I2S + I2C | 24-bit    |
//! | Pod           | WM8731   | I2S + I2C | 24-bit    |
//! | Patch SM      | PCM3060  | I2S + I2C | 24-bit    |

use crate::clock::SampleRate;

/// Error type for codec operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CodecError {
    /// I2C communication error
    I2cError,
    /// Invalid configuration
    InvalidConfig,
    /// Codec not responding
    NotResponding,
    /// Timeout waiting for codec
    Timeout,
}

/// Codec initialization result.
pub type CodecResult<T> = Result<T, CodecError>;

/// Common trait for all audio codecs.
///
/// Codecs implement this trait to provide a unified interface for
/// initialization and configuration.
pub trait Codec {
    /// Initialize the codec for audio operation.
    fn init(&mut self, sample_rate: SampleRate) -> CodecResult<()>;

    /// Set the output volume (0.0 = mute, 1.0 = max).
    fn set_output_volume(&mut self, volume: f32) -> CodecResult<()>;

    /// Set the input gain (0.0 = min, 1.0 = max).
    fn set_input_gain(&mut self, gain: f32) -> CodecResult<()>;

    /// Mute/unmute the output.
    fn set_mute(&mut self, mute: bool) -> CodecResult<()>;

    /// Check if the codec is ready for audio streaming.
    fn is_ready(&self) -> bool;
}

// ============================================================================
// AK4556 Codec (Original Daisy Seed)
// ============================================================================

/// AK4556 codec driver for original Daisy Seed.
///
/// The AK4556 is an I2S-only codec with no I2C control interface.
/// It auto-detects sample rate from the MCLK/LRCK ratio and requires
/// no software configuration beyond SAI clock setup.
pub struct Ak4556 {
    ready: bool,
}

impl Ak4556 {
    /// Create a new AK4556 codec driver.
    pub const fn new() -> Self {
        Self { ready: false }
    }
}

impl Default for Ak4556 {
    fn default() -> Self {
        Self::new()
    }
}

impl Codec for Ak4556 {
    fn init(&mut self, _sample_rate: SampleRate) -> CodecResult<()> {
        // AK4556 auto-detects sample rate from MCLK/LRCK ratio.
        // No I2C configuration needed - just mark as ready.
        self.ready = true;
        Ok(())
    }

    fn set_output_volume(&mut self, _volume: f32) -> CodecResult<()> {
        // AK4556 has no volume control - always passes through at unity gain.
        // Volume must be controlled in software.
        Ok(())
    }

    fn set_input_gain(&mut self, _gain: f32) -> CodecResult<()> {
        // AK4556 has no gain control - always passes through at unity gain.
        Ok(())
    }

    fn set_mute(&mut self, _mute: bool) -> CodecResult<()> {
        // AK4556 has no mute control - muting must be done in software.
        Ok(())
    }

    fn is_ready(&self) -> bool {
        self.ready
    }
}

// ============================================================================
// WM8731 Codec (Daisy Seed 1.1, Pod)
// ============================================================================

/// WM8731 I2C register addresses.
mod wm8731_regs {
    pub const LEFT_LINE_IN: u8 = 0x00;
    pub const RIGHT_LINE_IN: u8 = 0x01;
    pub const LEFT_HP_OUT: u8 = 0x02;
    pub const RIGHT_HP_OUT: u8 = 0x03;
    pub const ANALOG_PATH: u8 = 0x04;
    pub const DIGITAL_PATH: u8 = 0x05;
    pub const POWER_DOWN: u8 = 0x06;
    pub const DIGITAL_IF: u8 = 0x07;
    pub const SAMPLING: u8 = 0x08;
    pub const ACTIVE: u8 = 0x09;
    pub const RESET: u8 = 0x0F;
}

/// WM8731 codec driver for Daisy Seed 1.1 and Pod.
///
/// The WM8731 requires I2C configuration for sample rate, volume,
/// and routing control.
pub struct Wm8731<I2C> {
    i2c: I2C,
    address: u8,
    ready: bool,
}

impl<I2C> Wm8731<I2C> {
    /// WM8731 I2C address (CSB pin low)
    pub const I2C_ADDR_LOW: u8 = 0x1A;
    /// WM8731 I2C address (CSB pin high)
    pub const I2C_ADDR_HIGH: u8 = 0x1B;

    /// Create a new WM8731 codec driver with the given I2C peripheral.
    pub fn new(i2c: I2C, address: u8) -> Self {
        Self {
            i2c,
            address,
            ready: false,
        }
    }

    /// Create with default address (CSB low).
    pub fn with_default_address(i2c: I2C) -> Self {
        Self::new(i2c, Self::I2C_ADDR_LOW)
    }
}

impl<I2C, E> Codec for Wm8731<I2C>
where
    I2C: embedded_hal::blocking::i2c::Write<Error = E>,
{
    fn init(&mut self, sample_rate: SampleRate) -> CodecResult<()> {
        use wm8731_regs::*;

        // Reset the codec
        self.write_reg(RESET, 0x00)?;

        // Power down line input and mic (we only use DAC output typically)
        // Keep everything else powered up
        self.write_reg(POWER_DOWN, 0x07)?;

        // Configure analog audio path: DAC selected, no bypass, no sidetone
        self.write_reg(ANALOG_PATH, 0x10)?;

        // Configure digital audio path: no soft mute, no de-emphasis, ADC HPF enabled
        self.write_reg(DIGITAL_PATH, 0x00)?;

        // Configure digital interface: I2S format, 24-bit, slave mode
        self.write_reg(DIGITAL_IF, 0x02)?;

        // Configure sampling: normal mode, USB mode disabled
        let sr_bits = match sample_rate {
            SampleRate::Rate48000 => 0x00, // 48kHz, MCLK = 12.288MHz
            SampleRate::Rate96000 => 0x1C, // 96kHz, MCLK = 24.576MHz
        };
        self.write_reg(SAMPLING, sr_bits)?;

        // Set headphone output volume to 0dB
        self.write_reg(LEFT_HP_OUT, 0x79)?;
        self.write_reg(RIGHT_HP_OUT, 0x79)?;

        // Activate the codec
        self.write_reg(ACTIVE, 0x01)?;

        self.ready = true;
        Ok(())
    }

    fn set_output_volume(&mut self, volume: f32) -> CodecResult<()> {
        use wm8731_regs::*;

        // Volume range: 0x00 = mute, 0x30 = -73dB, 0x7F = +6dB
        // Map 0.0-1.0 to 0x30-0x7F (audible range)
        let volume = volume.clamp(0.0, 1.0);
        let reg_value = if volume == 0.0 {
            0x00 // Mute
        } else {
            // Map to 0x30-0x7F range
            let range = 0x7F - 0x30;
            (0x30 + (volume * range as f32) as u16).min(0x7F)
        };

        self.write_reg(LEFT_HP_OUT, reg_value)?;
        self.write_reg(RIGHT_HP_OUT, reg_value)?;
        Ok(())
    }

    fn set_input_gain(&mut self, gain: f32) -> CodecResult<()> {
        use wm8731_regs::*;

        // Line input volume: 0x00 = mute, 0x17 = 0dB, 0x1F = +12dB
        let gain = gain.clamp(0.0, 1.0);
        let reg_value = (gain * 0x1F as f32) as u16;

        self.write_reg(LEFT_LINE_IN, reg_value)?;
        self.write_reg(RIGHT_LINE_IN, reg_value)?;
        Ok(())
    }

    fn set_mute(&mut self, mute: bool) -> CodecResult<()> {
        use wm8731_regs::*;

        // Headphone outputs: bit 7 is mute control when set
        let mute_bit = if mute { 0x80 } else { 0x00 };
        let current_vol = 0x79; // Assume 0dB, could track actual volume

        self.write_reg(LEFT_HP_OUT, current_vol | mute_bit)?;
        self.write_reg(RIGHT_HP_OUT, current_vol | mute_bit)?;
        Ok(())
    }

    fn is_ready(&self) -> bool {
        self.ready
    }
}

impl<I2C, E> Wm8731<I2C>
where
    I2C: embedded_hal::blocking::i2c::Write<Error = E>,
{
    /// Write a 9-bit value to a 7-bit register address.
    ///
    /// WM8731 uses a 16-bit I2C word: [7-bit addr][9-bit data].
    fn write_reg(&mut self, reg: u8, value: u16) -> CodecResult<()> {
        let bytes = [(reg << 1) | ((value >> 8) as u8 & 0x01), (value & 0xFF) as u8];
        self.i2c.write(self.address, &bytes).map_err(|_| CodecError::I2cError)
    }
}

// ============================================================================
// PCM3060 Codec (Daisy Seed 1.2, Patch SM)
// ============================================================================

/// PCM3060 I2C register addresses.
mod pcm3060_regs {
    pub const REG_64: u8 = 64; // System control
    pub const REG_65: u8 = 65; // DAC control 1
    pub const REG_66: u8 = 66; // DAC control 2
    pub const REG_67: u8 = 67; // DAC left attenuation
    pub const REG_68: u8 = 68; // DAC right attenuation
    pub const REG_69: u8 = 69; // ADC left attenuation
    pub const REG_70: u8 = 70; // ADC right attenuation
    pub const REG_71: u8 = 71; // ADC control 1
    pub const REG_72: u8 = 72; // ADC control 2
    pub const REG_73: u8 = 73; // ADC input mux
}

/// PCM3060 codec driver for Daisy Seed 1.2 and Patch SM.
///
/// The PCM3060 is a high-quality 24-bit codec with separate ADC and DAC
/// paths, requiring I2C configuration for full control.
pub struct Pcm3060<I2C> {
    i2c: I2C,
    address: u8,
    ready: bool,
}

impl<I2C> Pcm3060<I2C> {
    /// PCM3060 I2C address when MD1=0, MD0=0
    pub const I2C_ADDR_00: u8 = 0x46;
    /// PCM3060 I2C address when MD1=0, MD0=1
    pub const I2C_ADDR_01: u8 = 0x47;

    /// Create a new PCM3060 codec driver.
    pub fn new(i2c: I2C, address: u8) -> Self {
        Self {
            i2c,
            address,
            ready: false,
        }
    }

    /// Create with default address for Daisy Seed 1.2.
    pub fn with_default_address(i2c: I2C) -> Self {
        Self::new(i2c, Self::I2C_ADDR_00)
    }
}

impl<I2C, E> Codec for Pcm3060<I2C>
where
    I2C: embedded_hal::blocking::i2c::Write<Error = E>,
{
    fn init(&mut self, _sample_rate: SampleRate) -> CodecResult<()> {
        use pcm3060_regs::*;

        // System control: Reset off, power up all sections
        self.write_reg(REG_64, 0x00)?;

        // DAC control 1: I2S format, 24-bit
        self.write_reg(REG_65, 0x00)?;

        // DAC control 2: Normal operation
        self.write_reg(REG_66, 0x00)?;

        // DAC attenuation: 0dB (0xFF = mute, 0x00 = 0dB)
        self.write_reg(REG_67, 0x00)?; // Left
        self.write_reg(REG_68, 0x00)?; // Right

        // ADC attenuation: 0dB
        self.write_reg(REG_69, 0x00)?; // Left
        self.write_reg(REG_70, 0x00)?; // Right

        // ADC control 1: I2S format, 24-bit
        self.write_reg(REG_71, 0x00)?;

        // ADC control 2: Normal operation
        self.write_reg(REG_72, 0x00)?;

        // ADC input mux: Normal input (not differential)
        self.write_reg(REG_73, 0x00)?;

        self.ready = true;
        Ok(())
    }

    fn set_output_volume(&mut self, volume: f32) -> CodecResult<()> {
        use pcm3060_regs::*;

        // Attenuation: 0x00 = 0dB, 0xFF = mute
        // Map 0.0-1.0 to 0xFF-0x00 (inverted because it's attenuation)
        let volume = volume.clamp(0.0, 1.0);
        let attenuation = if volume == 0.0 {
            0xFF // Mute
        } else {
            ((1.0 - volume) * 0xD8 as f32) as u8 // -54dB range
        };

        self.write_reg(REG_67, attenuation)?;
        self.write_reg(REG_68, attenuation)?;
        Ok(())
    }

    fn set_input_gain(&mut self, gain: f32) -> CodecResult<()> {
        use pcm3060_regs::*;

        // ADC attenuation (similar to DAC)
        let gain = gain.clamp(0.0, 1.0);
        let attenuation = ((1.0 - gain) * 0xD8 as f32) as u8;

        self.write_reg(REG_69, attenuation)?;
        self.write_reg(REG_70, attenuation)?;
        Ok(())
    }

    fn set_mute(&mut self, mute: bool) -> CodecResult<()> {
        use pcm3060_regs::*;

        // DAC control 2: bit 0 is soft mute
        let control = if mute { 0x01 } else { 0x00 };
        self.write_reg(REG_66, control)?;
        Ok(())
    }

    fn is_ready(&self) -> bool {
        self.ready
    }
}

impl<I2C, E> Pcm3060<I2C>
where
    I2C: embedded_hal::blocking::i2c::Write<Error = E>,
{
    /// Write an 8-bit value to an 8-bit register address.
    fn write_reg(&mut self, reg: u8, value: u8) -> CodecResult<()> {
        self.i2c
            .write(self.address, &[reg, value])
            .map_err(|_| CodecError::I2cError)
    }
}

// ============================================================================
// Board-Specific Codec Selection
// ============================================================================

/// Get the appropriate codec for the current board variant.
#[cfg(feature = "seed")]
pub type BoardCodec = Ak4556;

#[cfg(any(feature = "seed_1_1", feature = "pod"))]
pub type BoardCodec<I2C> = Wm8731<I2C>;

#[cfg(any(feature = "seed_1_2", feature = "patch_sm"))]
pub type BoardCodec<I2C> = Pcm3060<I2C>;
