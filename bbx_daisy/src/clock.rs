//! STM32H750 clock configuration for Daisy hardware.
//!
//! This module configures the clock tree for audio processing:
//!
//! - HSE: 16 MHz external crystal
//! - PLL1: 400 MHz system clock (SYSCLK)
//! - PLL3: SAI clock source for audio (12.288 MHz for 48kHz)
//!
//! Note: We use 400 MHz instead of 480 MHz due to PLL lock issues
//! observed on some Daisy hardware when running at 480 MHz with VOS0.

use stm32h7xx_hal::{
    pac,
    prelude::*,
    rcc::{Ccdr, PllConfigStrategy, rec::AdcClkSel},
};

/// Audio sample rates supported by the clock configuration.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SampleRate {
    /// 48 kHz (12.288 MHz MCLK)
    #[default]
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
    /// - HSE at 16 MHz (Daisy Seed external crystal)
    /// - PLL1 at 400 MHz for SYSCLK
    /// - PLL3 configured for SAI audio clocking (PLL3_P)
    ///   - 12.288 MHz for 48 kHz (256 * Fs)
    ///   - 24.576 MHz for 96 kHz (256 * Fs)
    /// - VOS0 power mode for headroom
    /// - ADC clock muxed to peripheral clock
    ///
    /// Note: SAI1 clock source must be set to PLL3_P by the caller.
    pub fn configure(self, pwr: pac::PWR, rcc: pac::RCC, syscfg: &pac::SYSCFG) -> Ccdr {
        // Enable VOS0 power mode for 400 MHz operation with headroom
        // Note: We use 400 MHz instead of 480 MHz due to PLL lock issues
        // observed on some Daisy hardware.
        let pwr = pwr.constrain().vos0(syscfg).freeze();

        // Configure clocks:
        // - HSE: 16 MHz external crystal (Daisy Seed)
        // - PLL1: 400 MHz system clock
        // - PLL3: SAI audio clock (sample rate dependent)
        let rcc = rcc.constrain();
        let mut ccdr = rcc
            .use_hse(16.MHz()) // External 16MHz crystal
            .sys_ck(400.MHz()) // System clock at 400 MHz
            .pll3_strategy(PllConfigStrategy::Fractional) // Precise audio clock
            .pll3_p_ck(self.pll3_p_frequency()) // SAI MCLK (12.288/24.576 MHz)
            .freeze(pwr, syscfg);

        // Configure ADC clock source for knob/CV inputs
        ccdr.peripheral.kernel_adc_clk_mux(AdcClkSel::Per);

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
    #[allow(dead_code)]
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
