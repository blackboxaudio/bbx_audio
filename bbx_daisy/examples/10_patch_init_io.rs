//! # Patch.Init() I/O Check
//!
//! Bring-up example exercising the full Patch.Init() control surface:
//!
//! - Audio In L/R passes to Audio Out L/R with panel knob 1 as level
//! - The front-panel LED lights while Gate In 1 is high or the B7 button is held
//! - Gate Out 1 echoes that same trigger (patch it back into Gate In 2 and the B8 toggle swaps the LED source to Gate
//!   In 2 to verify the loop)
//! - The CV OUT jack mirrors CV jack 1, mapping -5..+5 V onto 0..5 V
//!
//! Build (from `bbx_daisy/`):
//!
//! ```sh
//! cargo build --example 10_patch_init_io --features patch_sm --release
//! ```

#![no_std]
#![no_main]

use bbx_daisy::{bbx_daisy_audio_with_controls, prelude::*};

struct IoCheck;

impl AudioProcessor for IoCheck {
    fn process(&mut self, input: &FrameBuffer<BLOCK_SIZE>, output: &mut FrameBuffer<BLOCK_SIZE>, controls: &Controls) {
        let level = controls.knobs[0];
        for i in 0..BLOCK_SIZE {
            let [left, right] = *input.frame(i);
            output.set_frame(i, left * level, right * level);
        }

        // B8 selects which gate drives the LED, so a Gate Out 1 -> Gate In 2
        // patch cable verifies the whole digital loop.
        let triggered = controls.gate1 || controls.button;
        let led_source = if controls.switch { controls.gate2 } else { triggered };
        outputs().set_led(if led_source { 1.0 } else { 0.0 });
        outputs().set_gate_out1(triggered);

        // CV jack 1 (-5..+5 V) mirrored onto the CV OUT jack (0..5 V).
        outputs().set_cv_out((controls.cv[0] + 1.0) * 0.5);
    }
}

bbx_daisy_audio_with_controls!(IoCheck, IoCheck);
