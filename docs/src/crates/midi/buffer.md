# Lock-Free MIDI Buffer

A lock-free SPSC ring buffer for thread-safe MIDI communication between MIDI input and audio threads.

## Overview

The `midi_buffer` function creates a producer/consumer pair for real-time MIDI message transfer:

- **`MidiBufferProducer`** - Used in the MIDI input thread to push messages
- **`MidiBufferConsumer`** - Used in the audio thread to pop messages

All consumer operations are lock-free and allocation-free, making them safe for real-time audio callbacks.

## Creating a Buffer

```rust
use bbx_midi::midi_buffer;

// Create a buffer with capacity for 64 messages
let (mut producer, mut consumer) = midi_buffer(64);
```

Typical capacity values are 64-256 messages.

## Producer (MIDI Thread)

```rust
use bbx_midi::{midi_buffer, MidiMessage};

let (mut producer, _consumer) = midi_buffer(64);

// Send a MIDI message
let msg = MidiMessage::new([0x90, 60, 100]);
if producer.try_send(msg) {
    // Message sent successfully
} else {
    // Buffer is full, message dropped
}

// Check if buffer is full
if producer.is_full() {
    // Handle overflow
}
```

## Consumer (Audio Thread)

```rust
use bbx_midi::midi_buffer;

let (_producer, mut consumer) = midi_buffer(64);

// Pop messages one at a time (realtime-safe)
while let Some(msg) = consumer.try_pop() {
    // Process MIDI message
}

// Check if buffer is empty
if consumer.is_empty() {
    // No messages pending
}
```

## Draining Messages

For batch processing, drain all messages into a pre-allocated buffer:

```rust
use bbx_midi::{midi_buffer, MidiMessage};

let (_producer, mut consumer) = midi_buffer(64);

let mut messages = Vec::with_capacity(64);
let count = consumer.drain_into(&mut messages);
println!("Received {} messages", count);
```

Note: `drain_into` uses `Vec::push`, which may allocate. For strict real-time safety, use `try_pop` in a loop instead.

## Usage Pattern

Typical MIDI synthesizer pattern with separate threads:

```rust
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use bbx_midi::{midi_buffer, MidiMessage, MidiBufferProducer, MidiBufferConsumer};

fn main() {
    let (producer, consumer) = midi_buffer(256);

    // MIDI input thread (via midir)
    let _midi_connection = setup_midi_input(producer);

    // Audio processing (via rodio or cpal)
    let synth = Synth::new(consumer);
    // ... play synth
}

// MIDI callback pushes to producer
fn setup_midi_input(mut producer: MidiBufferProducer) {
    // In midir callback:
    // producer.try_send(MidiMessage::from(bytes));
}

struct Synth {
    consumer: MidiBufferConsumer,
}

impl Synth {
    fn new(consumer: MidiBufferConsumer) -> Self {
        Self { consumer }
    }

    fn process_audio(&mut self, output: &mut [f32]) {
        // Process pending MIDI (realtime-safe)
        while let Some(msg) = self.consumer.try_pop() {
            self.handle_midi(&msg);
        }

        // Generate audio...
    }

    fn handle_midi(&mut self, msg: &MidiMessage) {
        // Handle note on/off, CC, etc.
    }
}
```

## Thread Safety

- `MidiBufferProducer` and `MidiBufferConsumer` are `Send` but not `Sync`
- Each should be owned by exactly one thread
- The underlying SPSC ring buffer handles synchronization
- All consumer operations are wait-free

## See Also

- [SPSC Ring Buffer](../core/spsc.md) - The underlying lock-free queue
- [Building a MIDI Synthesizer](../../tutorials/07_midi_synth.md) - Complete example
