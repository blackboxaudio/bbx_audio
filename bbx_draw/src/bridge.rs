//! Audio and MIDI data bridges for thread-safe communication.
//!
//! Uses `bbx_core::SpscRingBuffer` for lock-free, real-time-safe communication
//! between audio/MIDI threads and the visualization thread.

use bbx_core::spsc::{Consumer, Producer, SpscRingBuffer};
use bbx_midi::message::MidiMessage;

/// A frame of audio data for visualization.
#[derive(Clone)]
pub struct AudioFrame {
    /// Interleaved audio samples.
    pub samples: Vec<f32>,
    /// Sample rate in Hz.
    pub sample_rate: u32,
    /// Number of audio channels.
    pub num_channels: usize,
}

impl AudioFrame {
    /// Create a new audio frame.
    pub fn new(samples: Vec<f32>, sample_rate: u32, num_channels: usize) -> Self {
        Self {
            samples,
            sample_rate,
            num_channels,
        }
    }

    /// Get samples for a specific channel (de-interleaved).
    pub fn channel_samples(&self, channel: usize) -> Vec<f32> {
        if channel >= self.num_channels {
            return Vec::new();
        }

        self.samples
            .iter()
            .skip(channel)
            .step_by(self.num_channels)
            .copied()
            .collect()
    }

    /// Get the number of samples per channel.
    pub fn samples_per_channel(&self) -> usize {
        if self.num_channels == 0 {
            0
        } else {
            self.samples.len() / self.num_channels
        }
    }
}

/// Producer side of the audio bridge (used in audio thread).
pub struct AudioBridgeProducer {
    producer: Producer<AudioFrame>,
}

impl AudioBridgeProducer {
    /// Try to send an audio frame to the visualization thread.
    ///
    /// Returns `true` if the frame was sent, `false` if the buffer was full.
    /// Dropping frames is acceptable for visualization purposes.
    pub fn try_send(&mut self, frame: AudioFrame) -> bool {
        self.producer.try_push(frame).is_ok()
    }

    /// Check if the buffer is full.
    pub fn is_full(&self) -> bool {
        self.producer.is_full()
    }
}

/// Consumer side of the audio bridge (used in visualization thread).
pub struct AudioBridgeConsumer {
    consumer: Consumer<AudioFrame>,
}

impl AudioBridgeConsumer {
    /// Drain all available audio frames from the buffer.
    pub fn drain(&mut self) -> Vec<AudioFrame> {
        let mut frames = Vec::new();
        while let Some(frame) = self.consumer.try_pop() {
            frames.push(frame);
        }
        frames
    }

    /// Get the latest frame, discarding older ones.
    pub fn latest(&mut self) -> Option<AudioFrame> {
        let mut latest = None;
        while let Some(frame) = self.consumer.try_pop() {
            latest = Some(frame);
        }
        latest
    }

    /// Check if there are frames available.
    pub fn is_empty(&self) -> bool {
        self.consumer.is_empty()
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

        let frame = AudioFrame::new(vec![0.5, -0.5, 0.25, -0.25], 44100, 2);
        assert!(producer.try_send(frame));

        let frames = consumer.drain();
        assert_eq!(frames.len(), 1);
        assert_eq!(frames[0].samples.len(), 4);
        assert_eq!(frames[0].sample_rate, 44100);
    }

    #[test]
    fn test_audio_frame_channel_samples() {
        let frame = AudioFrame::new(vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0], 44100, 2);

        let left = frame.channel_samples(0);
        let right = frame.channel_samples(1);

        assert_eq!(left, vec![1.0, 3.0, 5.0]);
        assert_eq!(right, vec![2.0, 4.0, 6.0]);
    }

    #[test]
    fn test_audio_bridge_overflow() {
        let (mut producer, _consumer) = audio_bridge(2);

        let frame = AudioFrame::new(vec![0.0], 44100, 1);
        assert!(producer.try_send(frame.clone()));
        assert!(producer.try_send(frame.clone()));
        assert!(!producer.try_send(frame));
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
