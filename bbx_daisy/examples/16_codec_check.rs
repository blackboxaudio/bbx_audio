//! # 16_codec_check - is the codec alive, and is the TX DMA feeding it?
//!
//! The SAI clocks at 48 kHz, MCLK is output, the engine runs — yet no sound. This localizes the
//! silence by checking two things after running the real audio path for ~1 s:
//!
//! 1. **Codec ADC alive?** — `audio::diag_rx_accumulate()` ORs every RX-buffer sample. Nonzero
//!    means the codec is powered, out of reset, and receiving MCLK (its ADC is producing data).
//! 2. **TX DMA advancing?** — reads `DMA1_S0NDTR` (the TX stream's remaining-count) repeatedly;
//!    if it changes, the TX DMA is actively moving samples to the SAI/codec.
//!
//! Report on PC7 (after **3 startup blinks**), framed by **two LONG blinks**, then two numbers:
//!
//! - **RX**: `1` quick blink = codec ADC alive (nonzero data); `2` = RX all zeros (codec not running).
//! - **TX**: `1` quick blink = TX DMA advancing; `2` = TX DMA stalled.
//!
//! | RX, TX | Conclusion |
//! |--------|------------|
//! | 1, 1   | Codec runs (clocks good) AND TX delivers → silence is the DAC analog out / codec init / wiring. |
//! | 2, 1   | TX delivers but the **codec isn't running** → MCLK/reset/codec-init problem. |
//! | 1, 2   | Codec runs but **TX DMA stalled** → the output path never reaches the codec. |
//! | 2, 2   | Neither — deeper SAI/DMA issue. |
//!
//! ## Building & Flashing
//!
//! ```bash
//! cd bbx_daisy
//! # Put the Daisy in DFU mode (hold BOOT, tap RESET, release BOOT), then:
//! cargo run --example 16_codec_check --features diag_peek --release
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

    // Passthrough: the RX buffer fills from the codec ADC regardless; TX carries it back.
    fn audio_callback(input: &FrameBuffer<BLOCK_SIZE>, output: &mut FrameBuffer<BLOCK_SIZE>) {
        for i in 0..BLOCK_SIZE {
            let f = input.frame(i);
            output.set_frame(i, f[0], f[1]);
        }
    }

    const RCC_AHB4ENR: *mut u32 = 0x5802_44E0 as *mut u32;
    const GPIOC_MODER: *mut u32 = 0x5802_0800 as *mut u32;
    const GPIOC_BSRR: *mut u32 = 0x5802_0818 as *mut u32;
    const DMA1_S0NDTR: *const u32 = 0x4002_0014 as *const u32; // TX stream (stream 0) item count

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

    fn quick_blink(n: u32) {
        for _ in 0..n {
            pc7_set(true);
            cortex_m::asm::delay(45_000_000);
            pc7_set(false);
            cortex_m::asm::delay(80_000_000);
        }
    }

    fn long_blink() {
        pc7_set(true);
        cortex_m::asm::delay(260_000_000);
        pc7_set(false);
        cortex_m::asm::delay(90_000_000);
    }

    #[bbx_daisy::__internal::entry]
    fn main() -> ! {
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

        // Let the codec/DMA run so RX fills with ADC data.
        cortex_m::asm::delay(1_200_000_000);

        // 1) Codec ADC alive? (OR of all RX samples)
        let rx_nonzero = audio::diag_rx_accumulate() != 0;

        // 2) TX DMA advancing? (does S0NDTR change?)
        let tx0 = unsafe { DMA1_S0NDTR.read_volatile() };
        let mut tx_advancing = false;
        for _ in 0..2_000_000u32 {
            if unsafe { DMA1_S0NDTR.read_volatile() } != tx0 {
                tx_advancing = true;
                break;
            }
        }

        // Stop the audio interrupt for a clean report.
        cortex_m::peripheral::NVIC::mask(pac::Interrupt::DMA1_STR1);

        let rx_code = if rx_nonzero { 1 } else { 2 };
        let tx_code = if tx_advancing { 1 } else { 2 };

        loop {
            long_blink();
            long_blink();
            cortex_m::asm::delay(300_000_000);
            quick_blink(rx_code);
            cortex_m::asm::delay(500_000_000);
            quick_blink(tx_code);
            cortex_m::asm::delay(900_000_000);
        }
    }
}
