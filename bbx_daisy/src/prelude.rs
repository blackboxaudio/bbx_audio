//! Convenient re-exports for Daisy applications.
//!
//! Import everything you need with a single `use` statement:
//!
//! ```ignore
//! use bbx_daisy::prelude::*;
//! ```

// Core types
// Math functions (so users don't need to import libm directly)
pub use core::f32::consts::PI;

pub use libm::{ceilf, cosf, expf, fabsf, floorf, log10f, logf, powf, sinf, sqrtf, tanf};
// HAL prelude for common traits (into_push_pull_output, etc.)
#[cfg(all(target_arch = "arm", target_os = "none"))]
pub use stm32h7xx_hal::prelude::*;

#[cfg(all(target_arch = "arm", target_os = "none", feature = "pod"))]
pub use crate::board::{AudioBoard, AudioBoardWithAdc, AudioPeripherals};
// High-level peripheral types
#[cfg(all(target_arch = "arm", target_os = "none"))]
pub use crate::flash::Flash;
#[cfg(all(target_arch = "arm", target_os = "none"))]
pub use crate::led::UserLed;
// Peripheral abstractions
#[cfg(all(target_arch = "arm", target_os = "none"))]
pub use crate::peripherals::{
    Button, CvInput, CvRange, Direction, Encoder, EncoderWithButton, GateIn, GateOut, Knob, Led, VelocityEncoder,
};
#[cfg(all(target_arch = "arm", target_os = "none"))]
pub use crate::sdram::Sdram;
// Audio processing
#[cfg(all(target_arch = "arm", target_os = "none"))]
pub use crate::{audio::BLOCK_SIZE, audio::DEFAULT_SAMPLE_RATE, board::Board, processor::AudioProcessor};
pub use crate::{
    buffer::{FrameBuffer, StaticSampleBuffer},
    context::EmbeddedDspContext,
    controls::Controls,
};
