//! # BBX Daisy
//!
//! Electrosmith Daisy hardware support for bbx_audio.
//!
//! This crate provides stack-allocated buffer types and hardware abstractions
//! for running bbx_audio DSP on Electrosmith Daisy platforms (Seed, Pod, Patch SM).
//!
//! ## Quick Start
//!
//! For audio processing, implement `AudioProcessor` and use `bbx_daisy_audio!`:
//!
//! ```ignore
//! #![no_std]
//! #![no_main]
//!
//! use bbx_daisy::prelude::*;
//!
//! struct SineOsc { phase: f32, phase_inc: f32 }
//!
//! impl AudioProcessor for SineOsc {
//!     fn process(&mut self, _input: &FrameBuffer<BLOCK_SIZE>, output: &mut FrameBuffer<BLOCK_SIZE>) {
//!         for i in 0..BLOCK_SIZE {
//!             let sample = sinf(self.phase * 2.0 * PI) * 0.5;
//!             output.set_frame(i, sample, sample);
//!             self.phase += self.phase_inc;
//!             if self.phase >= 1.0 { self.phase -= 1.0; }
//!         }
//!     }
//! }
//!
//! bbx_daisy_audio!(SineOsc, SineOsc { phase: 0.0, phase_inc: 440.0 / DEFAULT_SAMPLE_RATE });
//! ```
//!
//! For GPIO-only applications, use `bbx_daisy_run!`:
//!
//! ```ignore
//! #![no_std]
//! #![no_main]
//!
//! use bbx_daisy::prelude::*;
//!
//! fn blink(mut board: Board) -> ! {
//!     let mut led = Led::new(board.gpioc.pc7.into_push_pull_output());
//!     loop {
//!         led.toggle();
//!         board.delay.delay_ms(500u32);
//!     }
//! }
//!
//! bbx_daisy_run!(blink);
//! ```
//!
//! ## Buffer Types
//!
//! - [`StaticSampleBuffer`] - Stack-allocated sample buffer for DSP processing
//! - [`FrameBuffer`] - Interleaved stereo buffer for SAI/DMA hardware output
//!
//! ## Supported Boards
//!
//! Use feature flags to select your target hardware:
//!
//! - `seed` (default) - Daisy Seed with AK4556 codec
//! - `seed_1_1` - Daisy Seed 1.1 with WM8731 codec
//! - `seed_1_2` - Daisy Seed 1.2 with PCM3060 codec
//! - `pod` - Daisy Pod with WM8731 codec
//! - `patch_sm` - Patch SM with PCM3060 codec
//! - `patch_init` - Patch.Init() (uses Patch SM)

#![no_std]

// Core buffer types and context (always available)
pub mod buffer;
pub mod context;
pub mod controls;

// Prelude for convenient imports
pub mod prelude;

/// DSP functionality re-exported from `bbx_dsp`.
///
/// Provides access to blocks, parameters, and other DSP primitives
/// for use in embedded audio processing.
pub use bbx_dsp as dsp;

// HAL-dependent modules (only available on ARM Cortex-M targets)
#[cfg(all(target_arch = "arm", target_os = "none"))]
pub mod audio;
#[cfg(all(target_arch = "arm", target_os = "none"))]
pub mod board;
#[cfg(all(target_arch = "arm", target_os = "none"))]
pub mod clock;
#[cfg(all(target_arch = "arm", target_os = "none"))]
pub mod codec;
#[cfg(all(target_arch = "arm", target_os = "none"))]
mod macros;
#[cfg(all(target_arch = "arm", target_os = "none"))]
pub mod peripherals;
#[cfg(all(target_arch = "arm", target_os = "none"))]
pub mod pins;
#[cfg(all(target_arch = "arm", target_os = "none"))]
pub mod processor;

// Re-exports at crate root
#[cfg(all(target_arch = "arm", target_os = "none"))]
pub use board::Board;
#[cfg(all(target_arch = "arm", target_os = "none", feature = "pod"))]
pub use board::{AudioBoard, AudioBoardWithAdc, AudioPeripherals, BoardWithAdc};
pub use buffer::{FrameBuffer, StaticSampleBuffer};
pub use context::EmbeddedDspContext;
pub use controls::Controls;
#[cfg(all(target_arch = "arm", target_os = "none"))]
pub use processor::AudioProcessor;

/// Internal re-exports for macros.
///
/// This module is not part of the public API and may change without notice.
#[doc(hidden)]
#[cfg(all(target_arch = "arm", target_os = "none"))]
pub mod __internal {
    pub use cortex_m::asm::wfi;
    pub use cortex_m_rt::entry;
    pub use panic_halt;
    pub use stm32h7xx_hal::pac;
}
