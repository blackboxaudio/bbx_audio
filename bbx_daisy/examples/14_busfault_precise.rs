//! # 14_busfault_precise - force the imprecise BusFault to become precise, then read its address
//!
//! `13` reported subtype 3 (IMPRECISERR) / region 5 (BFAR invalid): an **imprecise** bus fault,
//! so the faulting address wasn't captured. This is identical to `13` except it sets
//! **`ACTLR.DISDEFWBUF`** (disable the default write buffer) at startup, which forces every store
//! to complete in order — turning the imprecise fault into a **precise** one with a valid BFAR.
//!
//! Read the same framed sequence as `13`:
//!
//! > **one LONG (~1.5 s) blink** → **`subtype` quick blinks** → gap → **`region` quick blinks** → repeat
//!
//! Expect **subtype 2 (PRECISERR)** now, and a **valid region**:
//!
//! | region | faulting address top nibble | what lives there                              |
//! |--------|-----------------------------|-----------------------------------------------|
//! | 1      | `0x3…`                      | D2 SRAM — the DMA buffers (`0x3004_xxxx`)      |
//! | 2      | `0x4…/0x5…`                 | peripherals: SAI `0x4001`, DMA1 `0x4002`, RCC/GPIO `0x5802` |
//! | 3      | `0xE…`                      | system — SCB / cache maintenance registers     |
//! | 4      | other                       | flash / DTCM / AXI / SDRAM                      |
//! | 5      | (still invalid)             | fault is still imprecise — tell me             |
//!
//! ## Building & Flashing
//!
//! ```bash
//! cd bbx_daisy
//! # Put the Daisy in DFU mode (hold BOOT, tap RESET, release BOOT), then:
//! cargo run --example 14_busfault_precise --release
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
    const ACTLR: *mut u32 = 0xE000_E008 as *mut u32; // bit 1 = DISDEFWBUF

    const QUICK: u32 = 55_000_000;
    const LONG_ON: u32 = 550_000_000;
    const GAP_MID: u32 = 300_000_000;
    const GAP_END: u32 = 800_000_000;

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

        let subtype = if bfsr & 0x01 != 0 {
            1
        } else if bfsr & 0x02 != 0 {
            2
        } else if bfsr & 0x04 != 0 {
            3
        } else if bfsr & 0x08 != 0 {
            4
        } else if bfsr & 0x10 != 0 {
            5
        } else if bfsr & 0x20 != 0 {
            6
        } else {
            0
        };
        let region = if bfsr & 0x80 == 0 {
            5
        } else {
            match bfar >> 28 {
                0x3 => 1,
                0x4 | 0x5 => 2,
                0xE => 3,
                _ => 4,
            }
        };

        loop {
            pc7_set(true);
            cortex_m::asm::delay(LONG_ON);
            pc7_set(false);
            cortex_m::asm::delay(GAP_MID);
            quick_blinks(subtype);
            cortex_m::asm::delay(GAP_MID);
            quick_blinks(region);
            cortex_m::asm::delay(GAP_END);
        }
    }

    #[bbx_daisy::__internal::entry]
    fn main() -> ! {
        // Disable the default write buffer so bus faults are precise (BFAR becomes valid).
        unsafe { ACTLR.write_volatile(ACTLR.read_volatile() | (1 << 1)) };
        cortex_m::asm::dsb();
        cortex_m::asm::isb();

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

        loop {
            pc7_set(true);
            cortex_m::asm::delay(200_000_000);
            pc7_set(false);
            cortex_m::asm::delay(200_000_000);
        }
    }
}
