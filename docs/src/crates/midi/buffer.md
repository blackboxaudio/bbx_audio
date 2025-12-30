# Message Buffer

A pre-allocated buffer for collecting MIDI messages in real-time audio callbacks.

## Overview

`MidiMessageBuffer` provides:

- Fixed capacity (no allocations during use)
- Push/pop operations
- Iteration
- Clear without deallocation

## Creating a Buffer

```rust
use bbx_midi::MidiMessageBuffer;

// Buffer with capacity for 256 messages
let mut buffer = MidiMessageBuffer::new(256);
```

## Adding Messages

```rust
use bbx_midi::{MidiMessage, MidiMessageBuffer};

let mut buffer = MidiMessageBuffer::new(256);

buffer.push(MidiMessage::note_on(0, 60, 100));
buffer.push(MidiMessage::note_on(0, 64, 100));
buffer.push(MidiMessage::note_on(0, 67, 100));
```

## Iterating

```rust
for msg in buffer.iter() {
    println!("{:?}", msg.get_status());
}
```

## Clearing

Clear the buffer for the next audio block:

```rust
buffer.clear();  // Sets length to 0, retains capacity
```

## Usage Pattern

Typical audio callback pattern:

```rust
use bbx_midi::{MidiMessage, MidiMessageBuffer};

struct AudioProcessor {
    midi_buffer: MidiMessageBuffer,
}

impl AudioProcessor {
    fn new() -> Self {
        Self {
            midi_buffer: MidiMessageBuffer::new(256),
        }
    }

    // Called from MIDI callback
    fn on_midi_message(&mut self, data: &[u8]) {
        if data.len() >= 3 {
            let msg = MidiMessage::new([data[0], data[1], data[2]]);
            self.midi_buffer.push(msg);
        }
    }

    // Called from audio callback
    fn process_audio(&mut self, output: &mut [f32]) {
        // Process pending MIDI messages
        for msg in self.midi_buffer.iter() {
            self.handle_midi(msg);
        }

        // Clear for next block
        self.midi_buffer.clear();

        // Generate audio...
    }

    fn handle_midi(&mut self, msg: &MidiMessage) {
        // Handle message
    }
}
```

## Capacity

Check buffer state:

```rust
let capacity = buffer.capacity();  // Maximum messages
let len = buffer.len();            // Current count
let is_empty = buffer.is_empty();
let is_full = buffer.len() == buffer.capacity();
```

## Thread Safety

`MidiMessageBuffer` is NOT thread-safe. For cross-thread MIDI:

- Use SPSC ring buffer for MIDI messages
- Or process MIDI on the same thread as audio
