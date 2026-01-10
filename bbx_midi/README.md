# bbx_midi

MIDI message parsing and streaming utilities for audio applications.

## Features

- **MIDI message parsing**: Parse raw MIDI bytes into structured messages
- **Message type helpers**: Extract note, velocity, pitch wheel, control data
- **Lock-free buffer**: Thread-safe SPSC ring buffer for MIDI-to-audio communication
- **Input streaming**: Real-time MIDI input via `midir`
- **FFI compatible**: `#[repr(C)]` types for C interop

## Message Types

- `NoteOn` / `NoteOff` - Note events with velocity
- `ControlChange` - CC messages (mod wheel, sustain, etc.)
- `PitchWheel` - Pitch bend with 14-bit resolution
- `ProgramChange` - Instrument/patch changes
- `PolyphonicAftertouch` - Per-note pressure
- `ChannelAftertouch` - Channel-wide pressure

## MidiEvent

For sample-accurate MIDI timing in audio callbacks:

```rust
use bbx_midi::{MidiMessage, MidiEvent};

let event = MidiEvent {
    message: MidiMessage::new([0x90, 60, 100]),
    sample_offset: 128,  // Process at sample 128 in buffer
};
```

Used with `PluginDsp::process()` for synthesizer plugins.

## Usage

```rust
use bbx_midi::{MidiMessage, MidiMessageStatus};

// Parse raw MIDI bytes
let msg = MidiMessage::new([0x90, 60, 100]); // Note On, C4, velocity 100

if msg.get_status() == MidiMessageStatus::NoteOn {
    println!("Note: {}", msg.get_note().unwrap());          // "C4"
    println!("Frequency: {} Hz", msg.get_note_frequency().unwrap()); // ~261.6
    println!("Velocity: {}", msg.get_velocity().unwrap());   // 100
}
```

## Lock-Free MIDI Buffer

For thread-safe communication between MIDI and audio threads:

```rust
use bbx_midi::{midi_buffer, MidiMessage};

// Create producer/consumer pair
let (mut producer, mut consumer) = midi_buffer(64);

// MIDI thread: push messages
let msg = MidiMessage::new([0x90, 60, 100]);
producer.try_send(msg);

// Audio thread: pop messages (realtime-safe)
while let Some(msg) = consumer.try_pop() {
    // Process MIDI message
}
```

The buffer uses an SPSC ring buffer internally, making all consumer operations lock-free and allocation-free for real-time safety.

## Cargo Features

### `streaming`

Enables real-time MIDI input via `midir`:

```toml
[dependencies]
bbx_midi = { version = "...", features = ["streaming"] }
```

```rust
use bbx_midi::{stream::MidiInputStream, MidiMessage, MidiMessageStatus};

// Create stream with optional filters
let stream = MidiInputStream::new(
    vec![MidiMessageStatus::NoteOn, MidiMessageStatus::NoteOff],
    |msg| println!("Received: {:?}", msg),
);

// Start listening (prompts for port selection)
let handle = stream.init();
```

Used by `bbx_sandbox` for terminal-based DSP testing with live MIDI input.

## License

MIT
