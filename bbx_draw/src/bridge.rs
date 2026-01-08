//! Audio and MIDI data bridges for thread-safe communication.
//!
//! Uses `bbx_core::SpscRingBuffer` for lock-free, real-time-safe communication
//! between audio/MIDI threads and the visualization thread.

use bbx_core::spsc::{Consumer, Producer, SpscRingBuffer};
use bbx_dsp::Frame;
use bbx_midi::message::MidiMessage;

/// Producer side of the audio bridge (used in audio thread).
pub struct AudioBridgeProducer {
    producer: Producer<Frame>,
}

impl AudioBridgeProducer {
    /// Try to send a frame to the visualization thread.
    ///
    /// Returns `true` if the frame was sent, `false` if the buffer was full.
    /// Dropping frames is acceptable for visualization purposes.
    pub fn try_send(&mut self, frame: Frame) -> bool {
        self.producer.try_push(frame).is_ok()
    }

    /// Check if the buffer is full.
    pub fn is_full(&self) -> bool {
        self.producer.is_full()
    }
}

/// Consumer side of the audio bridge (used in visualization or audio thread).
///
/// All methods are realtime-safe and do not allocate.
pub struct AudioBridgeConsumer {
    consumer: Consumer<Frame>,
}

impl AudioBridgeConsumer {
    /// Pop a single frame from the buffer (realtime-safe).
    ///
    /// Returns `Some(Frame)` if available, `None` if empty.
    #[inline]
    pub fn try_pop(&mut self) -> Option<Frame> {
        self.consumer.try_pop()
    }

    /// Check if there are frames available.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.consumer.is_empty()
    }

    /// Returns the number of frames currently available.
    #[inline]
    pub fn len(&self) -> usize {
        self.consumer.len()
    }
}

/// Create an audio bridge pair for thread-safe audio data transfer.
///
/// The `capacity` determines how many audio frames can be buffered.
/// A typical value is 4-16 frames.
pub fn audio_bridge(capacity: usize) -> (AudioBridgeProducer, AudioBridgeConsumer) {
    let (producer, consumer) = SpscRingBuffer::new(capacity);
    (AudioBridgeProducer { producer }, AudioBridgeConsumer { consumer })
}

/// Producer side of the MIDI bridge (used in MIDI input thread).
pub struct MidiBridgeProducer {
    producer: Producer<MidiMessage>,
}

impl MidiBridgeProducer {
    /// Try to send a MIDI message to the visualization thread.
    ///
    /// Returns `true` if the message was sent, `false` if the buffer was full.
    pub fn try_send(&mut self, message: MidiMessage) -> bool {
        self.producer.try_push(message).is_ok()
    }

    /// Check if the buffer is full.
    pub fn is_full(&self) -> bool {
        self.producer.is_full()
    }
}

/// Consumer side of the MIDI bridge (used in visualization thread).
pub struct MidiBridgeConsumer {
    consumer: Consumer<MidiMessage>,
}

impl MidiBridgeConsumer {
    /// Drain all available MIDI messages from the buffer.
    pub fn drain(&mut self) -> Vec<MidiMessage> {
        let mut messages = Vec::new();
        while let Some(msg) = self.consumer.try_pop() {
            messages.push(msg);
        }
        messages
    }

    /// Check if there are messages available.
    pub fn is_empty(&self) -> bool {
        self.consumer.is_empty()
    }
}

/// Create a MIDI bridge pair for thread-safe MIDI message transfer.
///
/// The `capacity` determines how many MIDI messages can be buffered.
/// A typical value is 64-256 messages.
pub fn midi_bridge(capacity: usize) -> (MidiBridgeProducer, MidiBridgeConsumer) {
    let (producer, consumer) = SpscRingBuffer::new(capacity);
    (MidiBridgeProducer { producer }, MidiBridgeConsumer { consumer })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audio_bridge_send_receive() {
        let (mut producer, mut consumer) = audio_bridge(4);

        let samples = [0.5f32, -0.5, 0.25, -0.25];
        let frame = Frame::new(&samples, 44100, 2);
        assert!(producer.try_send(frame));

        let frame = consumer.try_pop().expect("should have frame");
        assert_eq!(frame.samples.len(), 4);
        assert_eq!(frame.sample_rate, 44100);
        assert!(consumer.try_pop().is_none());
    }

    #[test]
    fn test_audio_bridge_overflow() {
        let (mut producer, _consumer) = audio_bridge(2);

        let samples = [0.0f32];
        let frame = Frame::new(&samples, 44100, 1);
        assert!(producer.try_send(frame.clone()));
        assert!(producer.try_send(frame.clone()));
        assert!(!producer.try_send(frame));
    }

    #[test]
    fn test_try_pop_empty() {
        let (_producer, mut consumer) = audio_bridge(4);
        assert!(consumer.try_pop().is_none());
        assert!(consumer.is_empty());
        assert_eq!(consumer.len(), 0);
    }

    #[test]
    fn test_try_pop_multiple() {
        let (mut producer, mut consumer) = audio_bridge(4);

        let samples1 = [1.0f32, 2.0];
        let samples2 = [3.0f32, 4.0];
        producer.try_send(Frame::new(&samples1, 44100, 1));
        producer.try_send(Frame::new(&samples2, 44100, 1));

        assert_eq!(consumer.len(), 2);

        let f1 = consumer.try_pop().unwrap();
        let f2 = consumer.try_pop().unwrap();
        assert!(consumer.try_pop().is_none());

        assert_eq!(f1.samples.as_slice(), &[1.0, 2.0]);
        assert_eq!(f2.samples.as_slice(), &[3.0, 4.0]);
    }

    #[test]
    fn test_midi_bridge_send_receive() {
        let (mut producer, mut consumer) = midi_bridge(8);

        let msg = MidiMessage::new([0x90, 60, 100]);
        assert!(producer.try_send(msg));

        let messages = consumer.drain();
        assert_eq!(messages.len(), 1);
    }
}
