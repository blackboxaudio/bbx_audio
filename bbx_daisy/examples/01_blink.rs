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
//! cargo build --example 01_blink --target thumbv7em-none-eabihf --release
//! probe-rs run --chip STM32H750VBTx target/thumbv7em-none-eabihf/release/examples/01_blink
//! ```

#![no_std]
#![no_main]

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
