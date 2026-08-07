//! # 06_board_id - Daisy Seed codec/version detector (diagnostic)
//!
//! Reads the PD3/PD4 strap pins (libDaisy's `CheckBoardVersion`) and blinks the Seed's
//! onboard LED (PC7) to report which codec your Seed has:
//!
//! - **1 blink**  = original Daisy Seed  → AK4556  (build with `--features seed`)
//! - **2 blinks** = Daisy Seed 1.1       → WM8731  (build with `--features seed_1_1`, or `pod`)
//! - **3 blinks** = Daisy Seed 2 DFM     → PCM3060 (build with `--features seed_1_2`)
//!
//! It blinks the group, pauses ~1.5 s, and repeats. No codec/audio is touched, so it runs
//! on any Seed-based board (including a Pod) regardless of the codec.
//!
//! ## Building & Flashing
//!
//! ```bash
//! cd bbx_daisy
//! # Put the Daisy in DFU mode (hold BOOT, tap RESET, release BOOT), then:
//! cargo run --example 06_board_id --release
//! ```

#![cfg_attr(all(target_arch = "arm", target_os = "none"), no_std, no_main)]

// See 01_blink.rs: ARM-only firmware with a host stub so the workspace still builds/tests
// on non-embedded targets.
#[cfg(not(all(target_arch = "arm", target_os = "none")))]
fn main() {}

#[cfg(all(target_arch = "arm", target_os = "none"))]
mod app {
    use bbx_daisy::{bbx_daisy_run, prelude::*};

    fn diagnose(mut board: Board) -> ! {
        let pd3 = board.gpiod.pd3.into_pull_up_input();
        let pd4 = board.gpiod.pd4.into_pull_up_input();
        let mut led = Led::new(board.gpioc.pc7.into_push_pull_output());

        // libDaisy: PD3 low → Seed 1.1 (WM8731); PD4 low → Seed 2 DFM (PCM3060); else AK4556.
        let blinks: u8 = if pd3.is_low() {
            2
        } else if pd4.is_low() {
            3
        } else {
            1
        };

        loop {
            for _ in 0..blinks {
                led.on();
                board.delay.delay_ms(200u16);
                led.off();
                board.delay.delay_ms(200u16);
            }
            board.delay.delay_ms(1500u16);
        }
    }

    bbx_daisy_run!(diagnose);
}
