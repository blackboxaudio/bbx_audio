//! # BBX MIDI
//!
//! MIDI message parsing and streaming utilities.
//!
//! This crate provides:
//! - [`MidiMessage`] - Parsed MIDI message with helper methods
//! - [`MidiMessageStatus`] - Message type enumeration
//! - [`MidiEvent`] - MIDI message with sample-accurate timing for audio processing
//! - [`MidiMessageBuffer`] - Pre-allocated buffer for real-time use
//! - [`stream::MidiInputStream`] - Real-time MIDI input via `midir` (requires `streaming` feature)
//!
//! ## Features
//!
//! - `streaming` - Enables real-time MIDI input via the `midir` crate
//!
//! ## FFI Compatibility
//!
//! [`MidiMessage`], [`MidiMessageStatus`], and [`MidiEvent`] use `#[repr(C)]` for C FFI compatibility.

pub mod buffer;
pub mod message;

#[cfg(feature = "streaming")]
pub mod stream;

pub use buffer::MidiMessageBuffer;
pub use message::{MidiEvent, MidiMessage, MidiMessageStatus};
