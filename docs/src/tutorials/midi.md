# MIDI Integration

This tutorial covers MIDI message handling with bbx_audio.

## Prerequisites

Add bbx_midi to your project:

```toml
[dependencies]
bbx_midi = "0.1"
```

## MIDI Message Types

bbx_midi supports standard MIDI messages:

| Message Type | Description |
|--------------|-------------|
| NoteOn | Key pressed |
| NoteOff | Key released |
| ControlChange | CC messages (knobs, sliders) |
| PitchWheel | Pitch bend |
| ProgramChange | Preset selection |
| PolyphonicAftertouch | Per-key pressure |
| ChannelAftertouch | Channel-wide pressure |

## Parsing MIDI Messages

```rust
use bbx_midi::message::{MidiMessage, MidiMessageStatus};

fn handle_midi(data: &[u8]) {
    if let Some(message) = MidiMessage::from_bytes(data) {
        match message.status() {
            MidiMessageStatus::NoteOn => {
                let note = message.note();
                let velocity = message.velocity();
                println!("Note On: {} vel {}", note, velocity);
            }
            MidiMessageStatus::NoteOff => {
                let note = message.note();
                println!("Note Off: {}", note);
            }
            MidiMessageStatus::ControlChange => {
                let cc = message.controller();
                let value = message.value();
                println!("CC {}: {}", cc, value);
            }
            MidiMessageStatus::PitchWheel => {
                let bend = message.pitch_bend();
                println!("Pitch Bend: {}", bend);
            }
            _ => {}
        }
    }
}
```

## Lock-Free MIDI Buffer

For thread-safe communication between MIDI and audio threads, use the lock-free MIDI buffer:

```rust
use bbx_midi::{midi_buffer, MidiMessage};

fn main() {
    // Create producer/consumer pair
    let (mut producer, mut consumer) = midi_buffer(64);

    // MIDI thread: push messages
    let msg = MidiMessage::new([0x90, 60, 100]);
    producer.try_send(msg);

    // Audio thread: pop messages (realtime-safe)
    while let Some(msg) = consumer.try_pop() {
        println!("{:?}", msg);
    }
}
```

The buffer uses an SPSC ring buffer internally:
- `MidiBufferProducer` for the MIDI input thread
- `MidiBufferConsumer` for the audio thread (all operations are lock-free)

## MIDI to Frequency

Convert MIDI note numbers to frequency:

```rust
fn midi_to_freq(note: u8) -> f32 {
    440.0 * 2.0_f32.powf((note as f32 - 69.0) / 12.0)
}

// Examples:
// midi_to_freq(60) = 261.63 Hz (Middle C)
// midi_to_freq(69) = 440.00 Hz (A4)
// midi_to_freq(72) = 523.25 Hz (C5)
```

## Velocity Scaling

Convert velocity to amplitude:

```rust
fn velocity_to_amplitude(velocity: u8) -> f32 {
    // Linear scaling
    velocity as f32 / 127.0
}

fn velocity_to_amplitude_curve(velocity: u8) -> f32 {
    // Logarithmic curve for more natural response
    let normalized = velocity as f32 / 127.0;
    normalized * normalized  // Square for exponential feel
}
```

## Simple MIDI Synth

Combine MIDI with oscillators:

```rust
use bbx_dsp::{graph::GraphBuilder, waveform::Waveform};
use bbx_midi::message::{MidiMessage, MidiMessageStatus};

struct Voice {
    note: u8,
    frequency: f32,
    velocity: f32,
    active: bool,
}

impl Voice {
    fn new() -> Self {
        Self {
            note: 0,
            frequency: 440.0,
            velocity: 0.0,
            active: false,
        }
    }

    fn note_on(&mut self, note: u8, velocity: u8) {
        self.note = note;
        self.frequency = 440.0 * 2.0_f32.powf((note as f32 - 69.0) / 12.0);
        self.velocity = velocity as f32 / 127.0;
        self.active = true;
    }

    fn note_off(&mut self, note: u8) {
        if self.note == note {
            self.active = false;
        }
    }
}

fn process_midi(voice: &mut Voice, message: &MidiMessage) {
    match message.status() {
        MidiMessageStatus::NoteOn => {
            if message.velocity() > 0 {
                voice.note_on(message.note(), message.velocity());
            } else {
                voice.note_off(message.note());
            }
        }
        MidiMessageStatus::NoteOff => {
            voice.note_off(message.note());
        }
        _ => {}
    }
}
```

## Control Change Mapping

Map CC messages to parameters:

```rust
use bbx_midi::message::{MidiMessage, MidiMessageStatus};

// Standard CC numbers
const CC_MOD_WHEEL: u8 = 1;
const CC_VOLUME: u8 = 7;
const CC_PAN: u8 = 10;
const CC_EXPRESSION: u8 = 11;
const CC_SUSTAIN: u8 = 64;

struct SynthParams {
    volume: f32,
    pan: f32,
    mod_depth: f32,
    sustain: bool,
}

fn handle_cc(params: &mut SynthParams, message: &MidiMessage) {
    if message.status() != MidiMessageStatus::ControlChange {
        return;
    }

    let cc = message.controller();
    let value = message.value() as f32 / 127.0;

    match cc {
        CC_VOLUME => params.volume = value,
        CC_PAN => params.pan = (value * 2.0) - 1.0,  // -1 to +1
        CC_MOD_WHEEL => params.mod_depth = value,
        CC_SUSTAIN => params.sustain = message.value() >= 64,
        _ => {}
    }
}
```

## Real-Time MIDI Input

For real-time MIDI input, use the streaming API:

```rust
use bbx_midi::stream::MidiInputStream;

fn main() {
    // Create MIDI input stream
    // The callback runs on the MIDI thread
    let stream = MidiInputStream::new(vec![], |message| {
        println!("Received: {:?}", message);
    });

    // Initialize and start listening
    let handle = stream.init();

    // Keep running...
    std::thread::sleep(std::time::Duration::from_secs(60));

    // Clean up
    handle.join().unwrap();
}
```

## Next Steps

- [Building a MIDI Synthesizer](07_midi_synth.md) - Complete real-time MIDI synth example
- [JUCE Integration](../juce/overview.md) - Use MIDI in plugins
- [DSP Graph Architecture](../architecture/graph-architecture.md) - Understand the system
