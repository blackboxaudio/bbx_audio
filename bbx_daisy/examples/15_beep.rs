//! # 15_beep - loud, unmistakable "is any audio coming out?" test
//!
//! The BusFault is fixed and the engine runs, but a quiet 440 Hz sine is hard to judge by ear
//! (and if the SAI sample rate is wrong, its pitch shifts). This plays a **near-full-scale square
//! wave** (much louder/harsher than a sine) that **beeps on ~0.5 s / off ~0.5 s**. The on/off
//! pulsing is impossible to mistake for ambient noise, and — crucially — the beep timing is driven
//! by *counting audio samples*, so its speed reveals the real sample rate:
//!
//! - **Loud buzzy beeping, ~1 per second** → audio works AND the rate is ~48 kHz. 🎉
//! - **Beeping but much slower** (multi-second on/off) and/or very low/rumbly pitch → audio works,
//!   but the SAI is clocking slow (clock/PLL3/MCLK issue — matches test 10's low IRQ rate).
//! - **Beeping much faster** → SAI clocking too fast.
//! - **No sound at all** → the codec/SAI isn't producing output (codec reset / I2S format / MCLK).
//!
//! The onboard LED also blinks a ~1 Hz heartbeat (CPU-timed) so you can compare: if the LED is
//! ~1 Hz but the audio beep is much slower, the audio sample rate is low.
//!
//! ## Building & Flashing
//!
//! ```bash
//! cd bbx_daisy
//! # Put the Daisy in DFU mode (hold BOOT, tap RESET, release BOOT), then:
//! cargo run --example 15_beep --release
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

    const AMPLITUDE: f32 = 0.9; // near full scale — loud
    const TONE_HALF_PERIOD: u32 = 100; // ~240 Hz square at 48 kHz (period 200 samples)
    const BEEP_SAMPLES: u32 = 24_000; // ~0.5 s on / 0.5 s off at 48 kHz

    static mut TONE_PHASE: u32 = 0;
    static mut BEEP_PHASE: u32 = 0;

    fn audio_callback(_input: &FrameBuffer<BLOCK_SIZE>, output: &mut FrameBuffer<BLOCK_SIZE>) {
        // Safety: callback runs only from the DMA ISR — single accessor for these counters.
        let tone = unsafe { &mut *core::ptr::addr_of_mut!(TONE_PHASE) };
        let beep = unsafe { &mut *core::ptr::addr_of_mut!(BEEP_PHASE) };
        for i in 0..BLOCK_SIZE {
            let beep_on = (*beep / BEEP_SAMPLES) % 2 == 0;
            let square = if *tone < TONE_HALF_PERIOD { AMPLITUDE } else { -AMPLITUDE };
            let sample = if beep_on { square } else { 0.0 };
            output.set_frame(i, sample, sample);

            *tone += 1;
            if *tone >= TONE_HALF_PERIOD * 2 {
                *tone = 0;
            }
            *beep += 1;
            if *beep >= BEEP_SAMPLES * 2 {
                *beep = 0;
            }
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

    #[bbx_daisy::__internal::entry]
    fn main() -> ! {
        let audio = board::init_audio().expect("Failed to initialize audio hardware");
        pc7_init();
        audio::set_callback(audio_callback);
        audio::init_and_start(
            audio.sample_rate,
            audio.sai1,
            audio.dma1,
            audio.dma1_rec,
            audio.sai1_pins,
            audio.sai1_rec,
            &audio.clocks,
        );

        // ~1 Hz CPU-timed heartbeat for comparison against the audio beep rate.
        loop {
            pc7_set(true);
            cortex_m::asm::delay(200_000_000);
            pc7_set(false);
            cortex_m::asm::delay(200_000_000);
        }
    }
}
