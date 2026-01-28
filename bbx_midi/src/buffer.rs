//! Lock-free MIDI buffer for thread-safe communication.
//!
//! Provides a realtime-safe channel for passing MIDI messages between threads,
//! suitable for MIDI input thread to audio thread communication.

use alloc::vec::Vec;

use bbx_core::spsc::{Consumer, Producer, SpscRingBuffer};

use crate::message::MidiMessage;

/// Producer side of the MIDI buffer (used in MIDI input thread).
pub struct MidiBufferProducer {
    producer: Producer<MidiMessage>,
}

impl MidiBufferProducer {
    /// Try to send a MIDI message to the consumer.
    ///
    /// Returns `true` if the message was sent, `false` if the buffer was full.
    #[inline]
    pub fn try_send(&mut self, message: MidiMessage) -> bool {
        self.producer.try_push(message).is_ok()
    }

    /// Check if the buffer is full.
    #[inline]
    pub fn is_full(&self) -> bool {
        self.producer.is_full()
    }
}

/// Consumer side of the MIDI buffer (used in audio thread).
///
/// All methods are realtime-safe and do not allocate.
pub struct MidiBufferConsumer {
    consumer: Consumer<MidiMessage>,
}

impl MidiBufferConsumer {
    /// Pop a single MIDI message from the buffer (realtime-safe).
    ///
    /// Returns `Some(MidiMessage)` if available, `None` if empty.
    #[inline]
    pub fn try_pop(&mut self) -> Option<MidiMessage> {
        self.consumer.try_pop()
    }

    /// Drain all available MIDI messages into the provided buffer.
    ///
    /// Returns the number of messages drained.
    #[inline]
    pub fn drain_into(&mut self, buffer: &mut Vec<MidiMessage>) -> usize {
        let mut count = 0;
        while let Some(msg) = self.consumer.try_pop() {
            buffer.push(msg);
            count += 1;
        }
        count
    }

    /// Check if the buffer is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.consumer.is_empty()
    }
}

/// Create a MIDI buffer pair for thread-safe MIDI message transfer.
///
/// The `capacity` determines how many MIDI messages can be buffered.
/// A typical value is 64-256 messages.
///
/// # Examples
///
/// ```
/// use bbx_midi::{MidiMessage, buffer::midi_buffer};
///
/// let (mut producer, mut consumer) = midi_buffer(64);
///
/// // In MIDI input thread
/// let msg = MidiMessage::new([0x90, 60, 100]);
/// producer.try_send(msg);
///
/// // In audio thread
/// while let Some(msg) = consumer.try_pop() {
///     // Process MIDI message
/// }
/// ```
pub fn midi_buffer(capacity: usize) -> (MidiBufferProducer, MidiBufferConsumer) {
    let (producer, consumer) = SpscRingBuffer::new(capacity);
    (MidiBufferProducer { producer }, MidiBufferConsumer { consumer })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_send_receive() {
        let (mut producer, mut consumer) = midi_buffer(8);

        let msg = MidiMessage::new([0x90, 60, 100]);
        assert!(producer.try_send(msg));

        let received = consumer.try_pop().expect("should have message");
        assert_eq!(received.get_note_number(), Some(60));
        assert!(consumer.try_pop().is_none());
    }

    #[test]
    fn test_buffer_overflow() {
        let (mut producer, _consumer) = midi_buffer(2);

        let msg = MidiMessage::new([0x90, 60, 100]);
        assert!(producer.try_send(msg));
        assert!(producer.try_send(msg));
        assert!(!producer.try_send(msg));
        assert!(producer.is_full());
    }

    #[test]
    fn test_drain_into() {
        let (mut producer, mut consumer) = midi_buffer(8);

        producer.try_send(MidiMessage::new([0x90, 60, 100]));
        producer.try_send(MidiMessage::new([0x90, 64, 80]));
        producer.try_send(MidiMessage::new([0x80, 60, 0]));

        let mut buffer = Vec::new();
        let count = consumer.drain_into(&mut buffer);

        assert_eq!(count, 3);
        assert_eq!(buffer.len(), 3);
        assert_eq!(buffer[0].get_note_number(), Some(60));
        assert_eq!(buffer[1].get_note_number(), Some(64));
        assert_eq!(buffer[2].get_note_number(), Some(60));
        assert!(consumer.is_empty());
    }

    #[test]
    fn test_is_empty() {
        let (mut producer, mut consumer) = midi_buffer(4);

        assert!(consumer.is_empty());

        producer.try_send(MidiMessage::new([0x90, 60, 100]));
        assert!(!consumer.is_empty());

        consumer.try_pop();
        assert!(consumer.is_empty());
    }
}
