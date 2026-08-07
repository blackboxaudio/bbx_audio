//! # BBX Core
//!
//! This crate contains the core mechanisms and utilities
//! used throughout this workspace.

#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(feature = "simd", feature(portable_simd))]

#[cfg(feature = "alloc")]
extern crate alloc;

pub mod buffer;
pub mod denormal;
pub mod error;
pub mod math;
pub mod random;
pub mod sample;
#[cfg(feature = "simd")]
pub mod simd;
#[cfg(feature = "alloc")]
pub mod spsc;
pub mod stack_vec;

pub use buffer::Buffer;
pub use denormal::{flush_denormal_f32, flush_denormal_f64, flush_denormals_f32_batch, flush_denormals_f64_batch};
pub use error::{BbxError, Result};
pub use math::Real;
pub use sample::Sample;
#[cfg(feature = "alloc")]
pub use spsc::{Consumer, Producer, SpscRingBuffer};
pub use stack_vec::StackVec;
