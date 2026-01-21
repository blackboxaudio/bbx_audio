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

use cortex_m_rt::entry;
use panic_halt as _;
use stm32h7xx_hal::{pac, prelude::*};

use bbx_daisy::peripherals::gpio::Led;

#[entry]
fn main() -> ! {
    // Get access to device peripherals
    let dp = pac::Peripherals::take().unwrap();
    let cp = cortex_m::Peripherals::take().unwrap();

    // Configure power
    let pwr = dp.PWR.constrain().freeze();

    // Configure clocks (use default internal oscillator for simplicity)
    let rcc = dp.RCC.constrain();
    let ccdr = rcc
        .sys_ck(480.MHz())
        .freeze(pwr, &dp.SYSCFG);

    // Configure GPIO port C
    let gpioc = dp.GPIOC.split(ccdr.peripheral.GPIOC);

    // Configure PC7 as output for the user LED
    let led_pin = gpioc.pc7.into_push_pull_output();
    let mut led = Led::new(led_pin);

    // Configure SysTick for delays
    let mut delay = cp.SYST.delay(ccdr.clocks);

    // Blink the LED
    loop {
        led.on();
        delay.delay_ms(500u16);
        led.off();
        delay.delay_ms(500u16);
    }
}
