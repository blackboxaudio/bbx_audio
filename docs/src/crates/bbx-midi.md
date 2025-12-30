# bbx_midi

MIDI message parsing and streaming utilities.

## Overview

bbx_midi provides:

- MIDI message parsing from raw bytes
- Message type helpers (note, velocity, CC, etc.)
- Real-time message buffer
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
| [Message Buffer](midi/buffer.md) | Real-time buffer |

## Quick Example

```rust
use bbx_midi::{MidiMessage, MidiMessageStatus, MidiMessageBuffer};

// Parse raw MIDI bytes
let msg = MidiMessage::new([0x90, 60, 100]); // Note On, C4, velocity 100

if msg.get_status() == MidiMessageStatus::NoteOn {
    println!("Note: {}", msg.get_note().unwrap());
    println!("Velocity: {}", msg.get_velocity().unwrap());
    println!("Frequency: {:.2} Hz", msg.get_note_frequency().unwrap());
}

// Buffer for real-time use
let mut buffer = MidiMessageBuffer::new(256);
buffer.push(msg);

for message in buffer.iter() {
    // Process...
}

buffer.clear();
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
