//! # BBX DSP
//!
//! This crate contains the core DSP components and operations
//! used when assembling DSP chains.
//!
//! ## FFI Support
//!
//! This crate can be built as a static library for use with C/C++ (JUCE plugins).
//! The `ffi` module provides C-compatible functions for interacting with the DSP graph.

pub mod block;
pub mod blocks;
pub mod buffer;
pub mod config;
pub mod context;
pub mod ffi;
pub mod graph;
pub mod parameter;
pub mod reader;
pub mod sample;
pub mod waveform;
pub mod writer;
