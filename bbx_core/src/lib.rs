//! # BBX Core
//!
//! This crate contains the core mechanisms and utilities
//! used throughout this workspace.

#![cfg_attr(feature = "simd", feature(portable_simd))]

pub mod denormal;
pub mod error;
pub mod random;
#[cfg(feature = "simd")]
pub mod simd;
pub mod spsc;
pub mod stack_vec;

pub use denormal::{flush_denormal_f32, flush_denormal_f64, flush_denormals_f32_batch, flush_denormals_f64_batch};
pub use error::{BbxError, Result};
pub use spsc::{Consumer, Producer, SpscRingBuffer};
pub use stack_vec::StackVec;
