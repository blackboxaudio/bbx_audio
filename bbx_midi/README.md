# bbx_midi

MIDI message parsing and streaming utilities for audio applications.

## Features

- **MIDI message parsing**: Parse raw MIDI bytes into structured messages
- **Message type helpers**: Extract note, velocity, pitch wheel, control data
- **Real-time buffer**: Pre-allocated buffer for use in audio callbacks
- **Input streaming**: Real-time MIDI input via `midir`
- **FFI compatible**: `#[repr(C)]` types for C interop

## Message Types

- `NoteOn` / `NoteOff` - Note events with velocity
- `ControlChange` - CC messages (mod wheel, sustain, etc.)
- `PitchWheel` - Pitch bend with 14-bit resolution
- `ProgramChange` - Instrument/patch changes
- `PolyphonicAftertouch` - Per-note pressure
- `ChannelAftertouch` - Channel-wide pressure

## Usage

```rust
use bbx_midi::{MidiMessage, MidiMessageStatus, MidiMessageBuffer};

// Parse raw MIDI bytes
let msg = MidiMessage::new([0x90, 60, 100]); // Note On, C4, velocity 100

if msg.get_status() == MidiMessageStatus::NoteOn {
    println!("Note: {}", msg.get_note().unwrap());          // "C4"
    println!("Frequency: {} Hz", msg.get_note_frequency().unwrap()); // ~261.6
    println!("Velocity: {}", msg.get_velocity().unwrap());   // 100
}

// Pre-allocated buffer for real-time use
let mut buffer = MidiMessageBuffer::new(256);
buffer.push(msg);

for message in buffer.iter() {
    // Process messages...
}

buffer.clear(); // Reset without deallocation
```

## License

MIT
