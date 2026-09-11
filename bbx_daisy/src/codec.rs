//! Audio codec initialization and control.
//!
//! This module provides codec drivers for Daisy hardware variants:
//!
//! | Board         | Codec    | Interface | Bit Depth | I2C Address |
//! |---------------|----------|-----------|-----------|-------------|
//! | Seed          | AK4556   | I2S       | 24-bit    | N/A         |
//! | Seed 1.1      | WM8731   | I2S + I2C | 24-bit    | 0x1A        |
//! | Seed 1.2      | PCM3060  | I2S + I2C | 24-bit    | 0x46        |
//! | Pod           | WM8731   | I2S + I2C | 24-bit    | 0x1A        |
//! | Patch SM      | PCM3060  | I2S + I2C | 24-bit    | 0x46        |
//!
//! # Codec Initialization Sequences
//!
//! ## WM8731 (Pod/Seed 1.1)
//!
//! 1. Reset codec (reg 0x0F = 0x00)
//! 2. Wait 10ms
//! 3. Power down unused sections (reg 0x06 = 0x07: line in, mic powered down)
//! 4. Configure analog path (reg 0x04 = 0x10: DAC selected)
//! 5. Configure digital path (reg 0x05 = 0x00: no soft mute, HPF enabled)
//! 6. Configure digital interface (reg 0x07 = 0x0A: I2S, 24-bit, slave)
//! 7. Configure sample rate (reg 0x08 = 0x00 @ 48kHz, 0x1C @ 96kHz)
//! 8. Set output volume (reg 0x02/0x03 = 0x79: 0dB)
//! 9. Activate codec (reg 0x09 = 0x01)
//!
//! **Timing**: 10ms delays between writes are conservative; most registers take effect immediately.
//! PLL settling time is ~1ms after activation.
//!
//! ## PCM3060 (Seed 1.2/Patch SM)
//!
//! 1. Master reset (reg 0x40, MRST bit 7 pulsed LOW — active-low, self-recovers), wait 4ms
//! 2. System reset (reg 0x40, SRST bit 6 pulsed LOW), wait 4ms
//! 3. Normal operation, power-save off (reg 0x40 = 0xC0), wait 1ms
//! 4. DAC format: 24-bit left-justified, slave (reg 0x43 = 0x01)
//! 5. ADC format: 24-bit left-justified, slave (reg 0x48 = 0x01)
//! 6. Attenuation registers (0x41/0x42 DAC, 0x46/0x47 ADC) are left at their power-on defaults, which are 0 dB — their
//!    encoding is inverted (DAC: 0xFF = 0 dB, 0x36 and below = mute)
//!
//! **Note**: Reset sequence timing per datasheet section 8.5.1. Left-justified
//! format matches the SAI's MSB-justified frame configuration.
//!
//! ## AK4556 (Seed)
//!
//! - No I2C control required
//! - Auto-detects sample rate from MCLK/LRCK ratio
//! - Fixed unity gain (no software volume control)
//!
//! # Error Handling
//!
//! All codec operations return `Result<T, CodecError>`:
//! - `I2cError`: I2C communication failure
//! - `InvalidConfig`: Unsupported configuration
//! - `NotResponding`: Codec not responding to I2C
//! - `Timeout`: Operation timed out

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

    /// Delay for approximately the given number of milliseconds.
    ///
    /// Uses a busy-wait loop calibrated for ~480MHz STM32H7.
    /// This is intentionally conservative to ensure codec stability.
    #[cfg(all(target_arch = "arm", target_os = "none"))]
    fn delay_ms(ms: u32) {
        // Matches ClockConfig's 480 MHz SYSCLK; cortex_m::asm::delay counts cycles.
        const CYCLES_PER_MS: u32 = 480_000;
        cortex_m::asm::delay(ms * CYCLES_PER_MS);
    }

    /// No-op delay for non-embedded targets (testing).
    #[cfg(not(all(target_arch = "arm", target_os = "none")))]
    fn delay_ms(_ms: u32) {
        // No delay needed for host testing
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
        Self::delay_ms(10);

        // Power down line input and mic (we only use DAC output typically)
        // Keep everything else powered up
        self.write_reg(POWER_DOWN, 0x07)?;
        Self::delay_ms(10);

        // Configure analog audio path: DAC selected, no bypass, no sidetone
        self.write_reg(ANALOG_PATH, 0x10)?;
        Self::delay_ms(10);

        // Configure digital audio path: no soft mute, no de-emphasis, ADC HPF enabled
        self.write_reg(DIGITAL_PATH, 0x00)?;
        Self::delay_ms(10);

        // Configure digital interface: I2S format, 24-bit, slave mode
        // Bits [1:0] = 0x02 (I2S format), Bits [3:2] = 0x08 (24-bit word length)
        self.write_reg(DIGITAL_IF, 0x0A)?;
        Self::delay_ms(10);

        // Configure sampling: normal mode, USB mode disabled
        let sr_bits = match sample_rate {
            SampleRate::Rate48000 => 0x00, // 48kHz, MCLK = 12.288MHz
            // UNVERIFIED on hardware: 0x1C (SR=0111, CLKIDIV2=0) assumes
            // 12.288MHz MCLK per the WM8731 table, but the clock tree runs
            // 256×Fs = 24.576MHz at 96kHz — CLKIDIV2 (bit 6) may be required.
            SampleRate::Rate96000 => 0x1C,
        };
        self.write_reg(SAMPLING, sr_bits)?;
        Self::delay_ms(10);

        // Set headphone output volume to 0dB
        self.write_reg(LEFT_HP_OUT, 0x79)?;
        Self::delay_ms(10);
        self.write_reg(RIGHT_HP_OUT, 0x79)?;
        Self::delay_ms(10);

        // Activate the codec (deactivate first to ensure clean transition)
        self.write_reg(ACTIVE, 0x00)?;
        Self::delay_ms(10);
        self.write_reg(ACTIVE, 0x01)?;
        Self::delay_ms(10);

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
        // NOTE: overwrites any level set via set_output_volume with 0dB —
        // per-instance volume tracking is not implemented yet.
        let current_vol = 0x79;

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

/// PCM3060 I2C register addresses (datasheet registers 64-73).
mod pcm3060_regs {
    pub const REG_SYS_CTRL: u8 = 0x40; // MRST/SRST (active-low) + ADC/DAC power-save
    pub const REG_DAC_ATT_L: u8 = 0x41; // DAC digital attenuation left (0xFF = 0 dB)
    pub const REG_DAC_ATT_R: u8 = 0x42; // DAC digital attenuation right
    pub const REG_DAC_CTRL1: u8 = 0x43; // DAC format / master-slave select
    pub const REG_DAC_CTRL2: u8 = 0x44; // DAC soft mute / de-emphasis
    pub const REG_ADC_ATT_L: u8 = 0x46; // ADC digital attenuation left (0xD7 = 0 dB)
    pub const REG_ADC_ATT_R: u8 = 0x47; // ADC digital attenuation right
    pub const REG_ADC_CTRL1: u8 = 0x48; // ADC format / master-slave select
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

    /// Delay for approximately the given number of milliseconds.
    ///
    /// Uses a busy-wait loop calibrated for ~480MHz STM32H7.
    /// This is intentionally conservative to ensure codec stability.
    #[cfg(all(target_arch = "arm", target_os = "none"))]
    fn delay_ms(ms: u32) {
        const CYCLES_PER_MS: u32 = 480_000;
        cortex_m::asm::delay(ms * CYCLES_PER_MS);
    }

    /// No-op delay for non-embedded targets (testing).
    #[cfg(not(all(target_arch = "arm", target_os = "none")))]
    fn delay_ms(_ms: u32) {}
}

impl<I2C, E> Codec for Pcm3060<I2C>
where
    I2C: embedded_hal::blocking::i2c::Write<Error = E>,
{
    fn init(&mut self, _sample_rate: SampleRate) -> CodecResult<()> {
        use pcm3060_regs::*;

        // MRST and SRST are ACTIVE-LOW and self-recover to 1: pulse the master
        // reset (registers back to defaults), then the system reset
        // (resynchronizes the audio clocks), then settle into normal operation
        // with ADC/DAC power-save disabled. Mirrors libDaisy's bring-up order.
        self.write_reg(REG_SYS_CTRL, 0x40)?; // MRST=0: master reset
        Self::delay_ms(4);
        self.write_reg(REG_SYS_CTRL, 0x80)?; // SRST=0: system reset
        Self::delay_ms(4);
        self.write_reg(REG_SYS_CTRL, 0xC0)?; // normal operation, both halves powered
        Self::delay_ms(1);

        // 24-bit left-justified, slave mode, on both halves — must match the
        // SAI's MSB-justified frame (I2S here would shift everything a bit).
        self.write_reg(REG_DAC_CTRL1, 0x01)?;
        self.write_reg(REG_ADC_CTRL1, 0x01)?;

        // The attenuation registers keep their power-on defaults (0 dB). Their
        // encoding is inverted from intuition — DAC: 0xFF = 0 dB, 0x36 and
        // below = mute — so writing "0x00 for 0 dB" hard-mutes the codec.
        self.ready = true;
        Ok(())
    }

    fn set_output_volume(&mut self, volume: f32) -> CodecResult<()> {
        use pcm3060_regs::*;

        // DAC attenuation: 0xFF = 0 dB, 0.5 dB per code below it, 0x36 and
        // below = mute. Map 0.0-1.0 onto mute..0 dB.
        let volume = volume.clamp(0.0, 1.0);
        let attenuation = if volume == 0.0 {
            0x36 // Mute
        } else {
            0x37 + (volume * 200.0) as u8 // up to 0xFF = 0 dB
        };

        self.write_reg(REG_DAC_ATT_L, attenuation)?;
        self.write_reg(REG_DAC_ATT_R, attenuation)?;
        Ok(())
    }

    fn set_input_gain(&mut self, gain: f32) -> CodecResult<()> {
        use pcm3060_regs::*;

        // ADC attenuation: 0xD7 = 0 dB (the power-on default), codes below
        // attenuate toward mute; codes above add gain (up to +20 dB at 0xFF,
        // not exposed here). Map 0.0-1.0 onto mute..0 dB.
        let gain = gain.clamp(0.0, 1.0);
        let attenuation = if gain == 0.0 {
            0x13 // Mute
        } else {
            0x14 + (gain * (0xD7 - 0x14) as f32) as u8 // up to 0xD7 = 0 dB
        };

        self.write_reg(REG_ADC_ATT_L, attenuation)?;
        self.write_reg(REG_ADC_ATT_R, attenuation)?;
        Ok(())
    }

    fn set_mute(&mut self, mute: bool) -> CodecResult<()> {
        use pcm3060_regs::*;

        // DAC control 2 bits 1:0 are the left/right soft-mute flags.
        let control = if mute { 0x03 } else { 0x00 };
        self.write_reg(REG_DAC_CTRL2, control)?;
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
