//! # 03_filter_knob - Filter with Knob Control Example
//!
//! A sawtooth oscillator through a low-pass filter, with the cutoff mapped
//! from `controls.knobs[0]`.
//!
//! Note: this Seed example uses `bbx_daisy_audio!`, which reads no hardware
//! controls — `knobs[0]` stays at its 0.5 default, so the cutoff sits
//! mid-range. It demonstrates a processor with multiple internal state
//! variables (oscillator phase and filter state) and how a control value
//! flows into DSP; see `05_pod_synth` for live knobs via
//! `bbx_daisy_audio_with_controls!`.
//!
//! ## Hardware
//!
//! - Daisy Seed (any variant) or Daisy Pod
//! - Audio output connected to speakers/headphones
//!
//! ## Building & Flashing
//!
//! ```bash
//! cd bbx_daisy
//! # Put the Daisy in DFU mode (hold BOOT, tap RESET, release BOOT), then:
//! cargo run --example 03_filter_knob --release
//! ```

#![cfg_attr(all(target_arch = "arm", target_os = "none"), no_std, no_main)]

// See 01_blink.rs: ARM-only firmware with a host stub so the workspace still builds/tests
// on non-embedded targets.
#[cfg(not(all(target_arch = "arm", target_os = "none")))]
fn main() {}

#[cfg(all(target_arch = "arm", target_os = "none"))]
mod app {
    use bbx_daisy::{bbx_daisy_audio, prelude::*};

    const FREQUENCY: f32 = 110.0;
    const AMPLITUDE: f32 = 0.5;
    const MIN_CUTOFF: f32 = 100.0;
    const MAX_CUTOFF: f32 = 8000.0;

    struct FilteredSaw {
        osc_phase: f32,
        phase_inc: f32,
        filter_state: f32,
    }

    impl FilteredSaw {
        fn new(frequency: f32) -> Self {
            Self {
                osc_phase: 0.0,
                phase_inc: frequency / DEFAULT_SAMPLE_RATE,
                filter_state: 0.0,
            }
        }

        fn calculate_coefficient(cutoff: f32) -> f32 {
            let omega = 2.0 * PI * cutoff / DEFAULT_SAMPLE_RATE;
            omega / (omega + 1.0)
        }
    }

    impl AudioProcessor for FilteredSaw {
        fn process(
            &mut self,
            _input: &FrameBuffer<BLOCK_SIZE>,
            output: &mut FrameBuffer<BLOCK_SIZE>,
            controls: &Controls,
        ) {
            // Map knob 1 to filter cutoff (100Hz - 8000Hz)
            let cutoff = MIN_CUTOFF + controls.knobs[0] * (MAX_CUTOFF - MIN_CUTOFF);
            let filter_coefficient = Self::calculate_coefficient(cutoff);

            for i in 0..BLOCK_SIZE {
                let saw = (self.osc_phase * 2.0 - 1.0) * AMPLITUDE;

                self.filter_state += filter_coefficient * (saw - self.filter_state);
                let filtered = self.filter_state;

                output.set_frame(i, filtered, filtered);

                self.osc_phase += self.phase_inc;
                if self.osc_phase >= 1.0 {
                    self.osc_phase -= 1.0;
                }
            }
        }
    }

    bbx_daisy_audio!(FilteredSaw, FilteredSaw::new(FREQUENCY));
}
