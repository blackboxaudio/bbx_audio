//! # BBX MIDI
//!
//! MIDI message parsing and streaming utilities.
//!
//! This crate provides:
//! - [`MidiMessage`] - Parsed MIDI message with helper methods
//! - [`MidiMessageStatus`] - Message type enumeration
//! - [`MidiEvent`] - MIDI message with sample-accurate timing for audio processing
//! - [`buffer`] - Lock-free MIDI buffer for thread-safe communication
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

pub use buffer::{MidiBufferConsumer, MidiBufferProducer, midi_buffer};
pub use message::{MidiEvent, MidiMessage, MidiMessageStatus};
