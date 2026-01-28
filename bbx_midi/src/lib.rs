//! # BBX MIDI
//!
//! MIDI message parsing and streaming utilities.
//!
//! This crate provides:
//! - [`MidiMessage`] - Parsed MIDI message with helper methods
//! - [`MidiMessageStatus`] - Message type enumeration
//! - [`MidiEvent`] - MIDI message with sample-accurate timing for audio processing
//! - [`buffer`] - Lock-free MIDI buffer for thread-safe communication (requires `alloc` feature)
//! - [`stream::MidiInputStream`] - Real-time MIDI input via `midir` (requires `streaming` feature)
//!
//! ## Features
//!
//! - `std` (default) - Enables standard library support
//! - `alloc` - Enables allocation-based features like the MIDI buffer
//! - `streaming` - Enables real-time MIDI input via the `midir` crate (requires `std`)
//!
//! ## FFI Compatibility
//!
//! [`MidiMessage`], [`MidiMessageStatus`], and [`MidiEvent`] use `#[repr(C)]` for C FFI compatibility.

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "alloc")]
pub mod buffer;
pub mod message;

#[cfg(feature = "streaming")]
pub mod stream;

#[cfg(feature = "alloc")]
pub use buffer::{MidiBufferConsumer, MidiBufferProducer, midi_buffer};
pub use message::{MidiEvent, MidiMessage, MidiMessageStatus};
