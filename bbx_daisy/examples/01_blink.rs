//! # 01_blink - LED Blink Example
//!
//! Basic LED blink to verify toolchain and GPIO functionality.
//!
//! ## Hardware
//!
//! - Daisy Seed (any variant)
//! - Built-in LED on PC7
//!
//! ## Building & Flashing
//!
//! ```bash
//! cd bbx_daisy
//! # Put the Daisy in DFU mode (hold BOOT, tap RESET, release BOOT), then:
//! cargo run --example 01_blink --release
//! ```

#![cfg_attr(all(target_arch = "arm", target_os = "none"), no_std, no_main)]

// This example is firmware for the Daisy (ARM Cortex-M). On other targets — e.g. a host
// `cargo test` / `cargo build --workspace` — the Daisy HAL, entry macros, and prelude are
// gated out, so it compiles to an empty binary rather than failing the workspace build.
#[cfg(not(all(target_arch = "arm", target_os = "none")))]
fn main() {}

#[cfg(all(target_arch = "arm", target_os = "none"))]
mod app {
    use bbx_daisy::{bbx_daisy_run, prelude::*};

    fn blink(mut board: Board) -> ! {
        let led_pin = board.gpioc.pc7.into_push_pull_output();
        let mut led = Led::new(led_pin);

        loop {
            led.on();
            board.delay.delay_ms(500u16);
            led.off();
            board.delay.delay_ms(500u16);
        }
    }

    bbx_daisy_run!(blink);
}
