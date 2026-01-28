# Introduction to Embedded Rust

This guide bridges the gap between desktop Rust and embedded Rust development. If you've written Rust applications that run on your computer, embedded development introduces a fundamentally different execution environment.

## What Makes Embedded Different

Desktop applications run on top of an operating system that handles memory allocation, threading, file systems, and device drivers. Embedded code runs directly on hardware—there's no OS, no dynamic memory allocator, and no standard library.

**Fixed Resources**: Your microcontroller has a specific amount of RAM and flash. There's no swap space, no virtual memory. If you run out of RAM, the program crashes or behaves unpredictably.

**Real-Time Constraints**: Audio processing must complete within strict deadlines. At 48 kHz with a 48-sample buffer, you have approximately 1 millisecond to process audio. Miss that deadline, and you get glitches.

**Direct Hardware Access**: You interact with peripherals through memory-mapped registers at specific addresses. There's no abstraction layer unless you build one.

## The `#![no_std]` Attribute

Standard Rust programs start with an implicit `use std::prelude::*`. The standard library (`std`) provides heap allocation, threading, file I/O, networking—features that require an operating system.

```rust
#![no_std]  // Don't link against std
```

With `#![no_std]`, you lose access to:
- `Vec`, `String`, `Box` (heap allocation)
- `std::thread` (OS threads)
- `std::fs`, `std::net` (file/network I/O)
- `println!` (requires stdout)

You retain access to `core`, which provides:
- Primitive types (`u32`, `f32`, `bool`)
- `Option`, `Result`
- Iterators and traits
- `core::mem`, `core::ptr`
- Math operations (with `libm` for transcendentals)

## The `#![no_main]` Attribute

Standard Rust programs have a `fn main()` that the runtime calls after setting up the environment. On embedded, there's no runtime to do that setup.

```rust
#![no_main]  // Don't expect a standard main function
```

Instead, you use an entry point macro from `cortex-m-rt`:

```rust
use cortex_m_rt::entry;

#[entry]
fn main() -> ! {
    // Initialize hardware
    // Enter main loop
    loop {
        // Process forever
    }
}
```

The `-> !` return type indicates this function never returns. Embedded main loops run until power is removed.

## Panic Handlers

When a panic occurs in `std` Rust, the runtime unwinds the stack and prints an error. Without a runtime, you must define what happens:

```rust
use panic_halt as _;  // Halt on panic (infinite loop)
```

Alternatives include:
- `panic-abort`: Immediately abort
- `panic-semihosting`: Print via debug probe (development only)
- `panic-rtt`: Send panic info via RTT debug channel

For production audio devices, `panic-halt` is typical—the device freezes rather than continuing in an undefined state.

## Essential Embedded Crates

### cortex-m

Low-level access to ARM Cortex-M processor features:

```rust
use cortex_m::peripheral::NVIC;
use cortex_m::interrupt;

// Disable interrupts in a critical section
interrupt::free(|_cs| {
    // Atomic operations here
});
```

### cortex-m-rt

Runtime for Cortex-M processors. Provides:
- Reset handler (entry point)
- Vector table
- Memory initialization (`.bss` zeroing, `.data` copying)

### embedded-hal

Hardware abstraction traits that work across different chips:

```rust
use embedded_hal::digital::OutputPin;
use embedded_hal::i2c::I2c;

fn configure_codec<I: I2c>(i2c: &mut I) -> Result<(), I::Error> {
    i2c.write(CODEC_ADDR, &[REG_CONTROL, 0x00])?;
    Ok(())
}
```

Code written against `embedded-hal` traits works with any chip that implements them.

## PAC vs HAL

**PAC (Peripheral Access Crate)**: Auto-generated from SVD files, provides raw register access.

```rust
// PAC: Direct register manipulation
let gpioa = unsafe { &*stm32h7::stm32h750::GPIOA::ptr() };
gpioa.odr.modify(|r, w| w.bits(r.bits() | (1 << 5)));
```

**HAL (Hardware Abstraction Layer)**: Ergonomic API built on top of PAC.

```rust
// HAL: Type-safe, ergonomic API
let mut led = gpioa.pa5.into_push_pull_output();
led.set_high();
```

For bbx_daisy, we use `stm32h7xx-hal` which wraps the `stm32h7` PAC.

## Minimal Blink Example

Here's a complete embedded program that blinks an LED:

```rust
#![no_std]
#![no_main]

use cortex_m_rt::entry;
use panic_halt as _;
use stm32h7xx_hal::{pac, prelude::*};

#[entry]
fn main() -> ! {
    // Take ownership of peripherals
    let dp = pac::Peripherals::take().unwrap();

    // Configure clocks
    let pwr = dp.PWR.constrain();
    let pwrcfg = pwr.freeze();
    let rcc = dp.RCC.constrain();
    let ccdr = rcc.sys_ck(400.MHz()).freeze(pwrcfg, &dp.SYSCFG);

    // Configure GPIO
    let gpioc = dp.GPIOC.split(ccdr.peripheral.GPIOC);
    let mut led = gpioc.pc7.into_push_pull_output();

    loop {
        led.set_high();
        cortex_m::asm::delay(40_000_000);  // ~100ms at 400MHz
        led.set_low();
        cortex_m::asm::delay(40_000_000);
    }
}
```

Key patterns:
1. **Peripheral singletons**: `take().unwrap()` ensures only one owner
2. **Builder pattern**: Clock configuration chains methods
3. **Type-state**: `into_push_pull_output()` changes the pin's type
4. **Blocking delay**: `asm::delay` burns cycles (use timers for real code)

## From Blink to Audio

Audio processing adds complexity:
- **DMA**: Hardware moves audio samples to/from RAM automatically
- **Interrupts**: DMA triggers callbacks when buffers are ready
- **Timing**: Processing must complete before the next buffer is needed
- **Cache coherency**: CPU cache and DMA don't see the same memory view

The `bbx_daisy` crate handles these details. Your code implements `AudioProcessor`:

```rust
impl AudioProcessor for MyDsp {
    fn process(&mut self, input: &FrameBuffer<BLOCK_SIZE>, output: &mut FrameBuffer<BLOCK_SIZE>) {
        // Called by DMA interrupt
        // ~1ms deadline at 48kHz/48 samples
    }
}
```

## Further Reading

- [Hardware Peripherals](hardware-peripherals.md) - GPIO, DMA, SAI explained
- [Memory Model](memory-model.md) - Stack, static allocation, linker sections
- [Toolchain](../compilation/toolchain.md) - How Rust compiles for ARM
