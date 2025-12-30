//! # BBX MIDI
//!
//! MIDI message parsing and streaming utilities.
//!
//! This crate provides:
//! - [`MidiMessage`] - Parsed MIDI message with helper methods
//! - [`MidiMessageStatus`] - Message type enumeration
//! - [`MidiMessageBuffer`] - Pre-allocated buffer for real-time use
//! - [`stream::MidiInputStream`] - Real-time MIDI input via `midir`
//!
//! ## FFI Compatibility
//!
//! [`MidiMessage`] and [`MidiMessageStatus`] use `#[repr(C)]` for C FFI compatibility.

pub mod buffer;
pub mod message;
pub mod stream;

pub use buffer::MidiMessageBuffer;
pub use message::{MidiMessage, MidiMessageStatus};
