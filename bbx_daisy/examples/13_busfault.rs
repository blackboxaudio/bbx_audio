//! # 13_busfault - decode the BusFault (subtype + faulting-address region)
//!
//! `12` showed the running audio path takes a **BusFault** after a few seconds. This runs the
//! same real audio path but its `HardFault` handler decodes the BusFault Status Register (BFSR)
//! and the BusFault Address Register (BFAR) and blinks them on PC7, so we learn *what kind* of
//! bus fault and *where*.
//!
//! On fault, PC7 repeats this framed sequence:
//!
//! 1. **One LONG (~1.5 s) blink** — frame marker / start of report.
//! 2. **`subtype` quick blinks** — which BFSR bit is set:
//!    `1`=IBUSERR (bad code fetch), `2`=PRECISERR (bad data access, BFAR valid),
//!    `3`=IMPRECISERR (async write error), `4`=UNSTKERR, `5`=STKERR (exception-entry stack),
//!    `6`=**LSPERR (fault while lazily stacking FP regs — the FPU-in-ISR smoking gun)**.
//! 3. gap, then **`region` quick blinks** — the top nibble of BFAR (only meaningful for PRECISERR):
//!    `1`=`0x3…` D2 SRAM / DMA buffers, `2`=`0x4…/0x5…` peripherals (SAI/DMA/RCC/GPIO),
//!    `3`=`0xE…` system (SCB/cache regs), `4`=other, `5`=BFAR not valid (imprecise/no address).
//! 4. long gap, repeat.
//!
//! Example: LONG, 2 blinks, gap, 1 blink → PRECISERR accessing the DMA-buffer SRAM region.
//! Or: LONG, 6 blinks, gap, 5 blinks → LSPERR (FP lazy-stacking BusFault, no address).
//!
//! If instead you see a steady ~1 Hz heartbeat, no fault occurred this run.
//!
//! ## Building & Flashing
//!
//! ```bash
//! cd bbx_daisy
//! # Put the Daisy in DFU mode (hold BOOT, tap RESET, release BOOT), then:
//! cargo run --example 13_busfault --release
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

    const RCC_AHB4ENR: *mut u32 = 0x5802_44E0 as *mut u32;
    const GPIOC_MODER: *mut u32 = 0x5802_0800 as *mut u32;
    const GPIOC_BSRR: *mut u32 = 0x5802_0818 as *mut u32;

    const QUICK: u32 = 55_000_000; // short blink
    const LONG_ON: u32 = 550_000_000; // long frame blink
    const GAP_MID: u32 = 300_000_000; // between the two numbers
    const GAP_END: u32 = 800_000_000; // before repeating

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

    fn quick_blinks(n: u32) {
        for _ in 0..n {
            pc7_set(true);
            cortex_m::asm::delay(QUICK);
            pc7_set(false);
            cortex_m::asm::delay(QUICK);
        }
    }

    #[exception]
    unsafe fn HardFault(_ef: &ExceptionFrame) -> ! {
        pc7_init();
        const CFSR: *const u32 = 0xE000_ED28 as *const u32;
        const BFAR: *const u32 = 0xE000_ED38 as *const u32;
        let cfsr = unsafe { CFSR.read_volatile() };
        let bfsr = (cfsr >> 8) & 0xFF;
        let bfar = unsafe { BFAR.read_volatile() };

        // subtype = first set BFSR bit (1..=6); 0 if none (not a BusFault).
        let subtype = if bfsr & 0x01 != 0 {
            1 // IBUSERR
        } else if bfsr & 0x02 != 0 {
            2 // PRECISERR
        } else if bfsr & 0x04 != 0 {
            3 // IMPRECISERR
        } else if bfsr & 0x08 != 0 {
            4 // UNSTKERR
        } else if bfsr & 0x10 != 0 {
            5 // STKERR
        } else if bfsr & 0x20 != 0 {
            6 // LSPERR (FP lazy stacking)
        } else {
            0
        };
        let bfar_valid = bfsr & 0x80 != 0;
        let region = if !bfar_valid {
            5
        } else {
            match bfar >> 28 {
                0x3 => 1,        // D2/D3 SRAM (DMA buffers @ 0x3004_xxxx)
                0x4 | 0x5 => 2,  // peripherals (SAI 0x4001, DMA1 0x4002, RCC/GPIO 0x5802)
                0xE => 3,        // system PPB (SCB/cache regs @ 0xE000_xxxx)
                _ => 4,          // other (flash/DTCM/AXI/SDRAM/...)
            }
        };

        loop {
            // Frame marker: one long blink.
            pc7_set(true);
            cortex_m::asm::delay(LONG_ON);
            pc7_set(false);
            cortex_m::asm::delay(GAP_MID);
            // subtype, gap, region.
            quick_blinks(subtype);
            cortex_m::asm::delay(GAP_MID);
            quick_blinks(region);
            cortex_m::asm::delay(GAP_END);
        }
    }

    #[bbx_daisy::__internal::entry]
    fn main() -> ! {
        let audio = board::init_audio().expect("Failed to initialize audio hardware");
        pc7_init();
        audio::set_callback(audio_callback);

        quick_blinks(3);
        cortex_m::asm::delay(GAP_END);
        audio::init_and_start(
            audio.sample_rate,
            audio.sai1,
            audio.dma1,
            audio.dma1_rec,
            audio.sai1_pins,
            audio.sai1_rec,
            &audio.clocks,
        );

        // Slow ~1 Hz heartbeat until (if) the BusFault fires.
        loop {
            pc7_set(true);
            cortex_m::asm::delay(200_000_000);
            pc7_set(false);
            cortex_m::asm::delay(200_000_000);
        }
    }
}
