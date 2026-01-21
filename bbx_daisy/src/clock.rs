//! STM32H750 clock configuration for Daisy hardware.
//!
//! This module configures the clock tree for audio processing:
//!
//! - HSE: 16 MHz external crystal
//! - PLL1: 480 MHz system clock (SYSCLK)
//! - PLL3: SAI clock source for audio (12.288 MHz for 48kHz)

use stm32h7xx_hal::{
    pac,
    prelude::*,
    rcc::{Ccdr, PllConfigStrategy},
};

/// Audio sample rates supported by the clock configuration.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SampleRate {
    /// 48 kHz (12.288 MHz MCLK)
    Rate48000,
    /// 96 kHz (24.576 MHz MCLK)
    Rate96000,
}

impl SampleRate {
    /// Get the sample rate in Hz.
    #[inline]
    pub const fn hz(self) -> u32 {
        match self {
            SampleRate::Rate48000 => 48_000,
            SampleRate::Rate96000 => 96_000,
        }
    }

    /// Get the sample rate as f32.
    #[inline]
    pub const fn as_f32(self) -> f32 {
        match self {
            SampleRate::Rate48000 => 48_000.0,
            SampleRate::Rate96000 => 96_000.0,
        }
    }
}

impl Default for SampleRate {
    fn default() -> Self {
        SampleRate::Rate48000
    }
}

/// Clock configuration for Daisy hardware.
///
/// Provides helper methods for configuring the STM32H750 clock tree
/// with audio-optimized PLL settings.
pub struct ClockConfig {
    sample_rate: SampleRate,
}

impl ClockConfig {
    /// Create a new clock configuration for the given sample rate.
    pub const fn new(sample_rate: SampleRate) -> Self {
        Self { sample_rate }
    }

    /// Get the configured sample rate.
    pub const fn sample_rate(&self) -> SampleRate {
        self.sample_rate
    }

    /// Configure the clock tree and return the configured Ccdr (clock controller).
    ///
    /// This sets up:
    /// - HSE at 16 MHz
    /// - PLL1 at 480 MHz for SYSCLK
    /// - PLL3 configured for SAI audio clocking
    pub fn configure(self, pwr: pac::PWR, rcc: pac::RCC, syscfg: &pac::SYSCFG) -> Ccdr {
        let pwr = pwr.constrain().freeze();

        let rcc = rcc.constrain();

        let ccdr = rcc
            .use_hse(16.MHz())
            .sys_ck(480.MHz())
            .pll3_strategy(PllConfigStrategy::Iterative)
            .pll3_p_ck(self.pll3_p_frequency())
            .pll3_q_ck(self.pll3_q_frequency())
            .freeze(pwr, syscfg);

        ccdr
    }

    /// Get PLL3_P frequency for the configured sample rate.
    ///
    /// PLL3_P is used as SAI clock source.
    fn pll3_p_frequency(&self) -> stm32h7xx_hal::time::Hertz {
        match self.sample_rate {
            // 12.288 MHz for 48kHz (MCLK = 256 * Fs)
            SampleRate::Rate48000 => 12_288_000.Hz(),
            // 24.576 MHz for 96kHz
            SampleRate::Rate96000 => 24_576_000.Hz(),
        }
    }

    /// Get PLL3_Q frequency (typically used for USB, not audio).
    fn pll3_q_frequency(&self) -> stm32h7xx_hal::time::Hertz {
        48.MHz()
    }
}

impl Default for ClockConfig {
    fn default() -> Self {
        Self::new(SampleRate::Rate48000)
    }
}

/// Enable peripheral clocks required for audio.
///
/// Call this after `ClockConfig::configure()` to enable clocks for:
/// - SAI1 (Serial Audio Interface)
/// - DMA1/DMA2 (Direct Memory Access)
/// - GPIO ports used by audio
pub fn enable_audio_clocks(_ccdr: &Ccdr) {
    // SAI1 clock enable is handled by the HAL when we configure the peripheral.
    // DMA clocks are typically enabled automatically when DMA is used.
    // This function is a placeholder for any additional clock enables needed.
}
