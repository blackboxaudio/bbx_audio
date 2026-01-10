# bbx_midi

MIDI message parsing and streaming utilities.

## Overview

bbx_midi provides:

- MIDI message parsing from raw bytes
- Message type helpers (note, velocity, CC, etc.)
- Lock-free MIDI buffer for thread-safe communication
- Input streaming via midir
- FFI-compatible types

## Installation

```toml
[dependencies]
bbx_midi = "0.1"
```

## Features

| Feature | Description |
|---------|-------------|
| [MIDI Messages](midi/messages.md) | Message parsing and types |
| [Lock-Free Buffer](midi/buffer.md) | Thread-safe MIDI transfer |

## Quick Example

```rust
use bbx_midi::{MidiMessage, MidiMessageStatus};

// Parse raw MIDI bytes
let msg = MidiMessage::new([0x90, 60, 100]); // Note On, C4, velocity 100

if msg.get_status() == MidiMessageStatus::NoteOn {
    println!("Note: {}", msg.get_note().unwrap());
    println!("Velocity: {}", msg.get_velocity().unwrap());
    println!("Frequency: {:.2} Hz", msg.get_note_frequency().unwrap());
}
```

## Lock-Free Buffer

For thread-safe MIDI communication between MIDI and audio threads:

```rust
use bbx_midi::{midi_buffer, MidiMessage};

// Create producer/consumer pair
let (mut producer, mut consumer) = midi_buffer(64);

// MIDI thread: push messages
let msg = MidiMessage::new([0x90, 60, 100]);
producer.try_send(msg);

// Audio thread: pop messages (realtime-safe)
while let Some(msg) = consumer.try_pop() {
    // Process MIDI
}
```

## Message Types

| Status | Description |
|--------|-------------|
| NoteOn | Key pressed |
| NoteOff | Key released |
| ControlChange | CC (knobs, pedals) |
| PitchWheel | Pitch bend |
| ProgramChange | Preset change |
| PolyphonicAftertouch | Per-note pressure |
| ChannelAftertouch | Channel pressure |

## FFI Compatibility

MIDI types use `#[repr(C)]` for C interop:

```c
typedef struct {
    uint8_t data[3];
} MidiMessage;
```
