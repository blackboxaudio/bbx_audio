//! # 12_faultcheck - is the freeze a CPU fault (and which kind) or a true hang?
//!
//! `08`/`11` showed `main` freezes on the *first* `process_audio_buffer` call, even though that
//! code should be ~40 µs and fault-free. A frozen CPU at that point is most likely a **fault**
//! landing in cortex-m-rt's default `HardFault` handler (an infinite loop). This example runs the
//! real audio path (sine through `process`) but installs a `HardFault` handler that blinks the
//! **fault class** on PC7, so we learn whether it faults and what kind.
//!
//! Watch from reset: **3 quick blinks** → **2 quick blinks** (init_and_start returned) → then:
//!
//! | Phase 3                                  | Conclusion                                                        |
//! |------------------------------------------|-------------------------------------------------------------------|
//! | **Rapid bursts of 1** (repeating)        | **MemManage fault** (CFSR & 0x0000_00FF) — MPU/access.            |
//! | **Rapid bursts of 2** (repeating)        | **BusFault** (CFSR & 0x0000_FF00) — bad memory/peripheral access. |
//! | **Rapid bursts of 3** (repeating)        | **UsageFault** (CFSR & 0x00FF_0000) — FPU/coprocessor, unaligned. |
//! | **Rapid bursts of 4** (repeating)        | HardFault with no CFSR bits (forced/escalated) — tell me.         |
//! | **Slow ~1 Hz heartbeat**                 | No fault — `process` ran fine (unexpected); audio path is alive.  |
//! | **Frozen** (solid/dark, no blinks)       | True hang (infinite loop), not a fault — tell me.                 |
//!
//! The rapid bursts (~50 ms blinks) are clearly faster than the slow heartbeat.
//!
//! ## Building & Flashing
//!
//! ```bash
//! cd bbx_daisy
//! # Put the Daisy in DFU mode (hold BOOT, tap RESET, release BOOT), then:
//! cargo run --example 12_faultcheck --release
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
    use cortex_m_rt::{ExceptionFrame, exception};

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

    fn blink(count: u32, on_cycles: u32, gap_cycles: u32) {
        for _ in 0..count {
            pc7_set(true);
            cortex_m::asm::delay(on_cycles);
            pc7_set(false);
            cortex_m::asm::delay(on_cycles);
        }
        cortex_m::asm::delay(gap_cycles);
    }

    /// HardFault handler: blink the fault class forever so we can see it on PC7.
    #[exception]
    unsafe fn HardFault(_ef: &ExceptionFrame) -> ! {
        pc7_init(); // ensure PC7 is an output even if we faulted very early
        const CFSR: *const u32 = 0xE000_ED28 as *const u32;
        let cfsr = unsafe { CFSR.read_volatile() };
        let class = if cfsr & 0x0000_00FF != 0 {
            1 // MemManage
        } else if cfsr & 0x0000_FF00 != 0 {
            2 // BusFault
        } else if cfsr & 0x00FF_0000 != 0 {
            3 // UsageFault
        } else {
            4 // forced/escalated, no CFSR detail
        };
        loop {
            blink(class, 20_000_000, 150_000_000); // ~50 ms blinks, ~0.4 s gap
        }
    }

    #[bbx_daisy::__internal::entry]
    fn main() -> ! {
        let audio = board::init_audio().expect("Failed to initialize audio hardware");
        pc7_init();
        audio::set_callback(audio_callback);

        blink(3, 60_000_000, 200_000_000);
        audio::init_and_start(
            audio.sample_rate,
            audio.sai1,
            audio.dma1,
            audio.dma1_rec,
            audio.sai1_pins,
            audio.sai1_rec,
            &audio.clocks,
        );
        blink(2, 60_000_000, 200_000_000);

        // Slow ~1 Hz heartbeat: only seen if no fault occurs.
        loop {
            pc7_set(true);
            cortex_m::asm::delay(200_000_000);
            pc7_set(false);
            cortex_m::asm::delay(200_000_000);
        }
    }
}
