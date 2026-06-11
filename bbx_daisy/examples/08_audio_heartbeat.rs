//! # 08_audio_heartbeat - "is the audio engine storming, hung, or just silent?" diagnostic
//!
//! Runs the **real** audio bring-up (`init_audio` → `set_callback` → `init_and_start`, the
//! exact path `bbx_daisy_audio!` uses) but, instead of `wfi`, drives the onboard LED (PC7)
//! from `main` so we can see whether `main` is alive. The DMA RX interrupt is left untouched
//! (do **not** build this with `diag_led` — that would fight `main` for PC7).
//!
//! The LED tells a 3-phase story. Watch from reset:
//!
//! 1. **3 quick blinks** = reached the audio start (clock + codec init succeeded).
//! 2. **2 quick blinks** = `init_and_start()` *returned* (SAI+DMA setup didn't hang).
//! 3. **slow ~1 Hz heartbeat, forever** = `main` keeps running.
//!
//! Decode:
//!
//! | What you see                                   | Conclusion                                                        |
//! |------------------------------------------------|-------------------------------------------------------------------|
//! | No blinks at all (dark)                        | Hung in `init_audio` (clock/codec) — upstream of audio start.     |
//! | 3 blinks, then stuck (solid or dark)           | `init_and_start()` **hung** — almost certainly the TX FIFO-fill   |
//! |                                                | spin `while …flvl().is_empty() {}` (SAI/TX-DMA never delivered).   |
//! | 3 blinks, 2 blinks, then **steady heartbeat**  | `main` is alive → **NO interrupt storm**. Silence is downstream   |
//! |                                                | (SAI not clocking / MCLK / codec / data path).                    |
//! | 3 blinks, 2 blinks, heartbeat then **freezes** | DMA ISR is **storming** and starving `main`.                      |
//!
//! Phases 1–2 are *fast* blinks; the heartbeat is *slow* — they're easy to tell apart.
//! A 440 Hz sine is fed to the output, so if it turns out audio is alive you'll also hear it.
//!
//! ## Building & Flashing
//!
//! ```bash
//! cd bbx_daisy
//! # Put the Daisy in DFU mode (hold BOOT, tap RESET, release BOOT), then:
//! cargo run --example 08_audio_heartbeat --release
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

    const FREQUENCY: f32 = 440.0;
    const AMPLITUDE: f32 = 0.5;
    const PHASE_INC: f32 = FREQUENCY / 48_000.0;

    static mut PHASE: f32 = 0.0;

    /// Plain SAI callback (no `AudioProcessor`/controls): synthesize a 440 Hz sine.
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

    // Raw GPIOC access: `init_audio()` consumes the PAC singleton, so the HAL GPIO API is
    // unavailable here. These mirror the `diag_led` raw access in `audio.rs`.
    const RCC_AHB4ENR: *mut u32 = 0x5802_44E0 as *mut u32; // GPIOCEN = bit 2
    const GPIOC_MODER: *mut u32 = 0x5802_0800 as *mut u32; // PC7 mode = bits [15:14]
    const GPIOC_BSRR: *mut u32 = 0x5802_0818 as *mut u32; // set = bit 7, reset = bit 23

    // CPU runs at the audio clock's sys_ck = 400 MHz, so cycles ≈ seconds × 4e8.
    // `asm::delay` is a lower bound, so real blinks are at least this slow — fine for the eye.
    const QUICK: u32 = 60_000_000; // ~150 ms
    const GAP: u32 = 400_000_000; // ~1 s
    const SLOW: u32 = 200_000_000; // ~500 ms

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

        // Phase 1: clock + codec are up, about to start SAI/DMA.
        blink(3);

        // Phase 2 only happens if this returns (i.e. the FIFO-fill spin completed).
        audio::init_and_start(
            audio.sample_rate,
            audio.sai1,
            audio.dma1,
            audio.dma1_rec,
            audio.sai1_pins,
            audio.sai1_rec,
            &audio.clocks,
        );
        blink(2);

        // Phase 3: slow heartbeat. Steady => main alive (no storm). Frozen => storm.
        loop {
            pc7_set(true);
            cortex_m::asm::delay(SLOW);
            pc7_set(false);
            cortex_m::asm::delay(SLOW);
        }
    }
}
