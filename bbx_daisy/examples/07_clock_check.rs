//! # 07_clock_check - Audio clock bring-up diagnostic
//!
//! Runs the **full audio clock configuration** (VOS0 + PLL3 for the SAI MCLK) — the same
//! one `init_audio` uses — then blinks the onboard LED (PC7) **fast (~5 Hz)**.
//!
//! - **LED blinks fast** → the audio clock (PLL3 / VOS0) came up fine; the silence is
//!   downstream (SAI / DMA / codec), not the clock.
//! - **LED stays dark** → the audio clock `freeze()` is hanging (PLL lock / VOS0). That's
//!   the bug — execution never reaches the audio loop.
//!
//! Compare with `01_blink` (slow ~1 Hz, simpler non-audio 400 MHz clock) which works.
//!
//! ## Building & Flashing
//!
//! ```bash
//! cd bbx_daisy
//! cargo run --example 07_clock_check --release
//! ```

#![cfg_attr(all(target_arch = "arm", target_os = "none"), no_std, no_main)]

#[cfg(not(all(target_arch = "arm", target_os = "none")))]
fn main() {}

#[cfg(all(target_arch = "arm", target_os = "none"))]
mod app {
    use bbx_daisy::__internal::panic_halt as _;
    use bbx_daisy::clock::{ClockConfig, SampleRate};
    use bbx_daisy::prelude::*;
    use stm32h7xx_hal::pac;

    #[bbx_daisy::__internal::entry]
    fn main() -> ! {
        let dp = pac::Peripherals::take().unwrap();
        let cp = cortex_m::Peripherals::take().unwrap();

        // Run the audio clock config (VOS0 + PLL3 for SAI). If this hangs, the LED never lights.
        let ccdr = ClockConfig::new(SampleRate::Rate48000).configure(dp.PWR, dp.RCC, &dp.SYSCFG);

        // Clock came up — blink PC7 fast (~5 Hz) to prove it.
        let gpioc = dp.GPIOC.split(ccdr.peripheral.GPIOC);
        let mut led = Led::new(gpioc.pc7.into_push_pull_output());
        let mut delay = cp.SYST.delay(ccdr.clocks);

        loop {
            led.on();
            delay.delay_ms(100u16);
            led.off();
            delay.delay_ms(100u16);
        }
    }
}
