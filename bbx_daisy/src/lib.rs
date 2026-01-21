//! # BBX Daisy
//!
//! Electrosmith Daisy hardware support for bbx_audio.
//!
//! This crate provides stack-allocated buffer types and hardware abstractions
//! for running bbx_audio DSP on Electrosmith Daisy platforms (Seed, Pod, Patch SM).
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

// HAL-dependent modules (only available on ARM Cortex-M targets)
#[cfg(all(target_arch = "arm", target_os = "none"))]
pub mod audio;
#[cfg(all(target_arch = "arm", target_os = "none"))]
pub mod clock;
#[cfg(all(target_arch = "arm", target_os = "none"))]
pub mod codec;
#[cfg(all(target_arch = "arm", target_os = "none"))]
pub mod peripherals;
#[cfg(all(target_arch = "arm", target_os = "none"))]
pub mod pins;

pub use buffer::{FrameBuffer, StaticSampleBuffer};
pub use context::EmbeddedDspContext;
