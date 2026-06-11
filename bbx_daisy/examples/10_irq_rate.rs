//! # 10_irq_rate - measure the exact DMA interrupt rate (= the SAI sample rate)
//!
//! Counts `DMA1_STR1` IRQs over a precise 1-second SysTick window (built with
//! `--features diag_irq_rate`, which counts IRQs and skips `process` so `main` stays alive),
//! then blinks the **exact count** on PC7, digit by digit. The count *is* the sample rate:
//!
//!   **Fs = 48 × (IRQs per second)**  — the 192-word circular buffer = 96 frames = 2 IRQs
//!   (HT+TC) per 96-frame loop, so IRQ/s = Fs / 48.
//!
//! Expected-correct reading is **1000** (→ 48 kHz). 500 → 24 kHz, 250 → 12 kHz, etc.
//!
//! Reading the number on PC7 (after **3 quick startup blinks** and ~1 s of measuring):
//!
//! - Each reading is framed by **two LONG (~0.6 s) blinks**, then a pause.
//! - Then the digits, most-significant first, separated by pauses.
//! - Digit **1–9** = that many quick blinks; digit **0** = one LONG blink.
//! - e.g. `1,0,0,0` = "1 quick … LONG … LONG … LONG" = **1000** (48 kHz, correct).
//!   `2,5,0` = "2 quick … 5 quick … LONG" = **250** (12 kHz, far too slow).
//!
//! ## Building & Flashing
//!
//! ```bash
//! cd bbx_daisy
//! # Put the Daisy in DFU mode (hold BOOT, tap RESET, release BOOT), then:
//! cargo run --example 10_irq_rate --features diag_irq_rate --release
//! ```

#![cfg_attr(all(target_arch = "arm", target_os = "none"), no_std, no_main)]

// See 01_blink.rs: ARM-only firmware with a host stub so the workspace still builds/tests
// on non-embedded targets.
#[cfg(not(all(target_arch = "arm", target_os = "none")))]
fn main() {}

#[cfg(all(target_arch = "arm", target_os = "none"))]
mod app {
    use bbx_daisy::__internal::{pac, panic_halt as _};
    use bbx_daisy::{audio, board, prelude::*};
    use core::sync::atomic::Ordering;

    // process is skipped under diag_irq_rate, so a trivial passthrough callback is fine.
    fn audio_callback(input: &FrameBuffer<BLOCK_SIZE>, output: &mut FrameBuffer<BLOCK_SIZE>) {
        for i in 0..BLOCK_SIZE {
            let f = input.frame(i);
            output.set_frame(i, f[0], f[1]);
        }
    }

    const RCC_AHB4ENR: *mut u32 = 0x5802_44E0 as *mut u32;
    const GPIOC_MODER: *mut u32 = 0x5802_0800 as *mut u32;
    const GPIOC_BSRR: *mut u32 = 0x5802_0818 as *mut u32;

    fn pc7_init() {
        unsafe {
            RCC_AHB4ENR.write_volatile(RCC_AHB4ENR.read_volatile() | (1 << 2));
            let _ = RCC_AHB4ENR.read_volatile();
            let moder = GPIOC_MODER.read_volatile();
            GPIOC_MODER.write_volatile((moder & !(0b11 << 14)) | (0b01 << 14));
        }
    }

    fn pc7_set(on: bool) {
        let bits = if on { 1 << 7 } else { 1 << 23 };
        unsafe { GPIOC_BSRR.write_volatile(bits) };
    }

    // asm::delay for the (timing-insensitive) report blinks. Quick clearly shorter than long.
    fn quick_blink(n: u32) {
        for _ in 0..n {
            pc7_set(true);
            cortex_m::asm::delay(40_000_000);
            pc7_set(false);
            cortex_m::asm::delay(75_000_000);
        }
    }

    fn long_blink() {
        pc7_set(true);
        cortex_m::asm::delay(260_000_000);
        pc7_set(false);
        cortex_m::asm::delay(80_000_000);
    }

    /// Blink `n` as decimal digits, MSB first; digit 0 = one long blink.
    fn report_number(n: u32) {
        let mut divisor = 1u32;
        while n / divisor >= 10 {
            divisor *= 10;
        }
        loop {
            let digit = (n / divisor) % 10;
            if digit == 0 {
                long_blink();
            } else {
                quick_blink(digit);
            }
            cortex_m::asm::delay(350_000_000); // gap between digit positions
            if divisor == 1 {
                break;
            }
            divisor /= 10;
        }
    }

    #[bbx_daisy::__internal::entry]
    fn main() -> ! {
        let cp = cortex_m::Peripherals::take().expect("core peripherals already taken");
        let audio = board::init_audio().expect("Failed to initialize audio hardware");
        pc7_init();
        audio::set_callback(audio_callback);

        quick_blink(3);
        cortex_m::asm::delay(400_000_000);

        audio::init_and_start(
            audio.sample_rate,
            audio.sai1,
            audio.dma1,
            audio.dma1_rec,
            audio.sai1_pins,
            audio.sai1_rec,
            &audio.clocks,
        );

        // Accurate 1-second window via SysTick (hardware-timed, unlike asm::delay).
        let mut delay = cp.SYST.delay(audio.clocks);
        audio::DIAG_IRQ_COUNT.store(0, Ordering::SeqCst);
        delay.delay_ms(1000u32);
        let rate = audio::DIAG_IRQ_COUNT.load(Ordering::SeqCst);
        cortex_m::peripheral::NVIC::mask(pac::Interrupt::DMA1_STR1);

        loop {
            long_blink();
            long_blink();
            cortex_m::asm::delay(300_000_000);
            report_number(rate);
            cortex_m::asm::delay(900_000_000);
        }
    }
}
