//! # 05_pod_synth - Daisy Pod Synth with Knob Control
//!
//! A sawtooth oscillator through a one-pole low-pass filter, played as a continuous
//! drone and controlled by the Pod's two knobs:
//!
//! - Knob 1: filter cutoff frequency (100 Hz – 8 kHz)
//! - Knob 2: oscillator pitch (55 Hz – 880 Hz)
//!
//! Unlike the Seed examples, this uses `bbx_daisy_audio_with_controls!`, which
//! initializes ADC1 and reads the Pod's knobs into `Controls` each control-loop
//! iteration. It is the reference test for the Pod's audio + control integration.
//!
//! ## Hardware
//!
//! - Daisy Pod
//! - Audio output connected to speakers/headphones
//!
//! ## Building & Flashing
//!
//! ```bash
//! cd bbx_daisy
//! # Put the Pod in DFU mode (hold BOOT, tap RESET, release BOOT), then:
//! cargo run --example 05_pod_synth --features pod --release
//! ```

#![cfg_attr(all(target_arch = "arm", target_os = "none"), no_std, no_main)]

// See 01_blink.rs: ARM-only firmware with a host stub so the workspace still builds/tests
// on non-embedded targets.
#[cfg(not(all(target_arch = "arm", target_os = "none")))]
fn main() {}

#[cfg(all(target_arch = "arm", target_os = "none"))]
mod app {
    use bbx_daisy::{bbx_daisy_audio_with_controls, prelude::*};

    const MIN_FREQ: f32 = 55.0;
    const MAX_FREQ: f32 = 880.0;
    const MIN_CUTOFF: f32 = 100.0;
    const MAX_CUTOFF: f32 = 8000.0;
    const AMPLITUDE: f32 = 0.4;

    struct PodSynth {
        phase: f32,
        filter_state: f32,
    }

    impl PodSynth {
        fn new() -> Self {
            Self {
                phase: 0.0,
                filter_state: 0.0,
            }
        }
    }

    impl AudioProcessor for PodSynth {
        fn process(
            &mut self,
            _input: &FrameBuffer<BLOCK_SIZE>,
            output: &mut FrameBuffer<BLOCK_SIZE>,
            controls: &Controls,
        ) {
            // Knob 2 -> pitch, knob 1 -> filter cutoff.
            let frequency = MIN_FREQ + controls.knob2 * (MAX_FREQ - MIN_FREQ);
            let phase_inc = frequency / DEFAULT_SAMPLE_RATE;

            let cutoff = MIN_CUTOFF + controls.knob1 * (MAX_CUTOFF - MIN_CUTOFF);
            let omega = 2.0 * PI * cutoff / DEFAULT_SAMPLE_RATE;
            let coeff = omega / (omega + 1.0);

            for i in 0..BLOCK_SIZE {
                let saw = (self.phase * 2.0 - 1.0) * AMPLITUDE;

                self.filter_state += coeff * (saw - self.filter_state);
                let out = self.filter_state;

                output.set_frame(i, out, out);

                self.phase += phase_inc;
                if self.phase >= 1.0 {
                    self.phase -= 1.0;
                }
            }
        }
    }

    bbx_daisy_audio_with_controls!(PodSynth, PodSynth::new());
}
