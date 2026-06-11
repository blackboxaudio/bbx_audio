//! # 11_process_time - measure how long one `process_audio_buffer` call takes
//!
//! `10_irq_rate` showed the DMA interrupt rate is *low* (~hundreds/s), yet `main` froze with
//! `process` in the ISR and ran without it. The only way that adds up is if each `process` call
//! takes roughly as long as the inter-IRQ period (≥~1 ms) — surprising, since 48 sine samples
//! plus two cache ops should be ~100 µs at 400 MHz. This settles it by timing **one** real
//! `process_audio_buffer` call with the DWT cycle counter (`--features diag_process_time`, which
//! times the first call, latches the cycles, then masks DMA1_STR1 so `main` can report).
//!
//! Watch from reset: **3 quick blinks** → short pause → **repeating bursts of N quick blinks**,
//! where N = number of decimal digits in the measured duration in **microseconds**:
//!
//! | Bursts (N) | process duration  | Conclusion                                                        |
//! |------------|-------------------|-------------------------------------------------------------------|
//! | **2–3**    | ~10–999 µs        | `process` is **fast** → duration isn't the freeze cause; rethink   |
//! |            |                   | (rate-with-process, or a side effect). Tell me — I'll re-probe.    |
//! | **4**      | ~1–9 ms           | `process` is **slow** (~ the IRQ period) → it's the spiral. Bisect |
//! |            |                   | which part (cache ops vs callback vs conversions) and optimize.   |
//! | **5+**     | ≥10 ms            | `process` is **very slow** → same, optimize urgently.             |
//! | **1**      | ~0 (no measure)   | First IRQ didn't run the timed path — tell me.                    |
//!
//! (µs = DWT cycles ÷ 400, assuming the 400 MHz audio sys_ck.)
//!
//! ## Building & Flashing
//!
//! ```bash
//! cd bbx_daisy
//! # Put the Daisy in DFU mode (hold BOOT, tap RESET, release BOOT), then:
//! cargo run --example 11_process_time --features diag_process_time --release
//! ```

#![cfg_attr(all(target_arch = "arm", target_os = "none"), no_std, no_main)]

// See 01_blink.rs: ARM-only firmware with a host stub so the workspace still builds/tests
// on non-embedded targets.
#[cfg(not(all(target_arch = "arm", target_os = "none")))]
fn main() {}

#[cfg(all(target_arch = "arm", target_os = "none"))]
mod app {
    use bbx_daisy::__internal::panic_halt as _;
    use bbx_daisy::{audio, board, prelude::*};
    use core::sync::atomic::Ordering;

    const FREQUENCY: f32 = 440.0;
    const AMPLITUDE: f32 = 0.5;
    const PHASE_INC: f32 = FREQUENCY / 48_000.0;

    static mut PHASE: f32 = 0.0;

    fn audio_callback(_input: &FrameBuffer<BLOCK_SIZE>, output: &mut FrameBuffer<BLOCK_SIZE>) {
        let phase = unsafe { &mut *core::ptr::addr_of_mut!(PHASE) };
        for i in 0..BLOCK_SIZE {
            let sample = sinf(*phase * 2.0 * PI) * AMPLITUDE;
            output.set_frame(i, sample, sample);
            *phase += PHASE_INC;
            if *phase >= 1.0 {
                *phase -= 1.0;
            }
        }
    }

    // Raw GPIOC access (init_audio consumed the device PAC); mirrors `diag_led` in audio.rs.
    const RCC_AHB4ENR: *mut u32 = 0x5802_44E0 as *mut u32; // GPIOCEN = bit 2
    const GPIOC_MODER: *mut u32 = 0x5802_0800 as *mut u32; // PC7 mode = bits [15:14]
    const GPIOC_BSRR: *mut u32 = 0x5802_0818 as *mut u32; // set = bit 7, reset = bit 23

    // DWT cycle counter (Cortex-M7 debug block) — enabled in main, read in the ISR.
    const DEMCR: *mut u32 = 0xE000_EDFC as *mut u32; // bit 24 = TRCENA
    const DWT_CTRL: *mut u32 = 0xE000_1000 as *mut u32; // bit 0 = CYCCNTENA
    const DWT_CYCCNT: *mut u32 = 0xE000_1004 as *mut u32;

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

    fn enable_cycle_counter() {
        unsafe {
            DEMCR.write_volatile(DEMCR.read_volatile() | (1 << 24));
            DWT_CYCCNT.write_volatile(0);
            DWT_CTRL.write_volatile(DWT_CTRL.read_volatile() | 1);
        }
    }

    /// Number of decimal digits of `n` (0 -> 1).
    fn digits_of(n: u32) -> u32 {
        let mut d = 1;
        let mut r = n / 10;
        while r > 0 {
            d += 1;
            r /= 10;
        }
        d
    }

    #[bbx_daisy::__internal::entry]
    fn main() -> ! {
        let cp = cortex_m::Peripherals::take().expect("core peripherals already taken");
        enable_cycle_counter();

        let audio = board::init_audio().expect("Failed to initialize audio hardware");
        pc7_init();
        audio::set_callback(audio_callback);

        for _ in 0..3 {
            pc7_set(true);
            cortex_m::asm::delay(60_000_000);
            pc7_set(false);
            cortex_m::asm::delay(60_000_000);
        }
        cortex_m::asm::delay(200_000_000);

        audio::init_and_start(
            audio.sample_rate,
            audio.sai1,
            audio.dma1,
            audio.dma1_rec,
            audio.sai1_pins,
            audio.sai1_rec,
            &audio.clocks,
        );

        // Let the first IRQ time one process call and mask itself.
        let mut delay = cp.SYST.delay(audio.clocks);
        delay.delay_ms(200u32);

        let cycles = audio::DIAG_PROCESS_CYCLES.load(Ordering::SeqCst);
        let micros = cycles / 400; // 400 MHz audio sys_ck
        let bursts = digits_of(micros);

        loop {
            for _ in 0..bursts {
                pc7_set(true);
                delay.delay_ms(150u32);
                pc7_set(false);
                delay.delay_ms(150u32);
            }
            delay.delay_ms(1500u32);
        }
    }
}
