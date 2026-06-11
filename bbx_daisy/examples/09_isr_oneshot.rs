//! # 09_isr_oneshot - storm vs. hang triage for the DMA audio interrupt
//!
//! Follow-up to `08_audio_heartbeat`, which showed **3 blinks then a constantly-lit LED** —
//! i.e. `init_and_start()` returned but `main` froze the instant SAI/DMA traffic began. That
//! leaves two causes: an **interrupt storm** (DMA1_STR1 re-fires forever, starving `main`) or a
//! **per-call hang** (`process_audio_buffer` hangs/faults on its first call).
//!
//! This example builds with `--features diag_isr_oneshot`, which makes the DMA ISR, on its
//! **first** IRQ, latch the stream-1 interrupt flags and then **mask itself** so it can't
//! re-fire. The real `process_audio_buffer` still runs once. Then `main` reports via PC7.
//!
//! Watch from reset:
//!
//! 1. **3 quick blinks** = reached the audio start.
//! 2. **2 quick blinks** = `init_and_start()` returned.
//! 3. then, **repeating bursts** (separated by ~1.5 s gaps) — OR a frozen LED.
//!
//! Decode phase 3:
//!
//! | Phase 3                                   | Conclusion                                                          |
//! |-------------------------------------------|---------------------------------------------------------------------|
//! | **Frozen** (solid lit, no bursts)         | `process_audio_buffer` **hangs** on its first call (masking didn't  |
//! |                                           | help, because the CPU is stuck *inside* the one ISR call). → H2.     |
//! | Repeating **1-blink** bursts              | **Storm**, and the first IRQ was a *normal* HT/TC — the DMA          |
//! |                                           | delivered a clean first transfer, so it re-fires too fast / forever. |
//! | Repeating **2-blink** bursts              | **Storm** from a DMA **error** flag (TE/FE/DME) on the first IRQ.    |
//! | Repeating **3-blink** bursts              | ISR fired but no stream-1 flag was set (unexpected — report it).    |
//!
//! Seeing *any* repeating burst means `main` is alive again → the freeze was a storm (now
//! suppressed by the one-shot mask), not a hang.
//!
//! ## Building & Flashing
//!
//! ```bash
//! cd bbx_daisy
//! # Put the Daisy in DFU mode (hold BOOT, tap RESET, release BOOT), then:
//! cargo run --example 09_isr_oneshot --features diag_isr_oneshot --release
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
        // Safety: the callback runs only from the DMA ISR, so `PHASE` has a single accessor.
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

    // Raw GPIOC access (init_audio consumed the PAC singleton); mirrors `diag_led` in audio.rs.
    const RCC_AHB4ENR: *mut u32 = 0x5802_44E0 as *mut u32; // GPIOCEN = bit 2
    const GPIOC_MODER: *mut u32 = 0x5802_0800 as *mut u32; // PC7 mode = bits [15:14]
    const GPIOC_BSRR: *mut u32 = 0x5802_0818 as *mut u32; // set = bit 7, reset = bit 23

    // CPU runs at sys_ck = 400 MHz, so cycles ≈ seconds × 4e8 (`asm::delay` is a lower bound).
    const QUICK: u32 = 60_000_000; // ~150 ms
    const GAP: u32 = 600_000_000; // ~1.5 s

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

    fn blink(count: u32) {
        for _ in 0..count {
            pc7_set(true);
            cortex_m::asm::delay(QUICK);
            pc7_set(false);
            cortex_m::asm::delay(QUICK);
        }
        cortex_m::asm::delay(GAP);
    }

    #[bbx_daisy::__internal::entry]
    fn main() -> ! {
        let audio = board::init_audio().expect("Failed to initialize audio hardware");
        pc7_init();

        audio::set_callback(audio_callback);
        blink(3); // reached audio start

        audio::init_and_start(
            audio.sample_rate,
            audio.sai1,
            audio.dma1,
            audio.dma1_rec,
            audio.sai1_pins,
            audio.sai1_rec,
            &audio.clocks,
        );
        blink(2); // init_and_start returned

        // Phase 3: decode the latched first-IRQ flags. Reaching here at all means main is alive
        // (the one-shot mask suppressed any storm). A frozen LED instead means process hung.
        loop {
            let flags = audio::DIAG_DMA_FLAGS.load(Ordering::SeqCst);
            let bursts = if flags & 0x340 != 0 {
                2 // error flag: FEIF1 (0x40) | DMEIF1 (0x100) | TEIF1 (0x200)
            } else if flags & 0xC00 != 0 {
                1 // normal: HTIF1 (0x400) | TCIF1 (0x800)
            } else {
                3 // ISR fired but no stream-1 flag set
            };
            blink(bursts);
        }
    }
}
