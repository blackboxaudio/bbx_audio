//! Pre-allocated MIDI message buffer for real-time audio processing.
//!
//! This module provides a buffer that can accumulate MIDI messages without
//! allocation in the audio callback, which is critical for real-time performance.

use crate::message::MidiMessage;

/// A pre-allocated buffer for accumulating MIDI messages.
///
/// This buffer is designed for use in real-time audio contexts where
/// allocations must be avoided. The buffer has a fixed capacity and
/// will silently drop messages if the capacity is exceeded.
#[derive(Debug)]
pub struct MidiMessageBuffer {
    messages: Vec<MidiMessage>,
    count: usize,
}

impl MidiMessageBuffer {
    /// Create a new buffer with the specified capacity.
    ///
    /// The buffer will be pre-allocated to hold up to `capacity` messages.
    pub fn new(capacity: usize) -> Self {
        Self {
            messages: Vec::with_capacity(capacity),
            count: 0,
        }
    }

    /// Clear all messages from the buffer.
    ///
    /// This does not deallocate memory, only resets the count.
    #[inline]
    pub fn clear(&mut self) {
        self.count = 0;
    }

    /// Add a message to the buffer.
    ///
    /// If the buffer is at capacity, the message is silently dropped.
    #[inline]
    pub fn push(&mut self, message: MidiMessage) {
        if self.count < self.messages.capacity() {
            if self.count < self.messages.len() {
                self.messages[self.count] = message;
            } else {
                self.messages.push(message);
            }
            self.count += 1;
        }
    }

    /// Returns the number of messages currently in the buffer.
    #[inline]
    pub fn len(&self) -> usize {
        self.count
    }

    /// Returns true if the buffer is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.count == 0
    }

    /// Returns the capacity of the buffer.
    #[inline]
    pub fn capacity(&self) -> usize {
        self.messages.capacity()
    }

    /// Iterate over the messages in the buffer.
    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = &MidiMessage> {
        self.messages[..self.count].iter()
    }

    /// Get a slice of all messages in the buffer.
    #[inline]
    pub fn as_slice(&self) -> &[MidiMessage] {
        &self.messages[..self.count]
    }
}

impl Default for MidiMessageBuffer {
    fn default() -> Self {
        Self::new(256)
    }
}
