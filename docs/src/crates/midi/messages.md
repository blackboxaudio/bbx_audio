# MIDI Messages

Parse and work with MIDI messages.

## MidiMessage

```rust
use bbx_midi::MidiMessage;

// Create from raw bytes
let msg = MidiMessage::new([0x90, 60, 100]);

// Or from parts
let msg = MidiMessage::note_on(0, 60, 100);  // Channel 0, Note 60, Velocity 100
```

## MidiMessageStatus

```rust
use bbx_midi::MidiMessageStatus;

pub enum MidiMessageStatus {
    NoteOff,
    NoteOn,
    PolyphonicAftertouch,
    ControlChange,
    ProgramChange,
    ChannelAftertouch,
    PitchWheel,
    Unknown,
}
```

## Constructors

```rust
use bbx_midi::MidiMessage;

// Note events
let note_on = MidiMessage::note_on(channel, note, velocity);
let note_off = MidiMessage::note_off(channel, note, velocity);

// Control change
let cc = MidiMessage::control_change(channel, controller, value);

// Pitch bend
let bend = MidiMessage::pitch_bend(channel, value);  // value: 0-16383

// Program change
let program = MidiMessage::program_change(channel, program);
```

## Accessors

### Status and Channel

```rust
let msg = MidiMessage::note_on(5, 60, 100);

let status = msg.get_status();  // MidiMessageStatus::NoteOn
let channel = msg.get_channel();  // 5
```

### Note Events

```rust
let msg = MidiMessage::note_on(0, 60, 100);

let note = msg.get_note().unwrap();           // 60
let velocity = msg.get_velocity().unwrap();   // 100
let frequency = msg.get_note_frequency().unwrap(); // 261.63
```

### Control Change

```rust
let msg = MidiMessage::control_change(0, 1, 64);

let controller = msg.get_controller().unwrap();  // 1 (mod wheel)
let value = msg.get_value().unwrap();            // 64
```

### Pitch Bend

```rust
let msg = MidiMessage::pitch_bend(0, 8192);  // Center

let bend = msg.get_pitch_bend().unwrap();  // 8192
// bend range: 0 (full down) - 8192 (center) - 16383 (full up)
```

## Note Frequency

Convert MIDI note numbers to Hz:

```rust
use bbx_midi::MidiMessage;

let msg = MidiMessage::note_on(0, 69, 100);  // A4
let freq = msg.get_note_frequency().unwrap(); // 440.0

let msg = MidiMessage::note_on(0, 60, 100);  // Middle C
let freq = msg.get_note_frequency().unwrap(); // 261.63
```

## Common Controller Numbers

```rust
const CC_MOD_WHEEL: u8 = 1;
const CC_BREATH: u8 = 2;
const CC_VOLUME: u8 = 7;
const CC_BALANCE: u8 = 8;
const CC_PAN: u8 = 10;
const CC_EXPRESSION: u8 = 11;
const CC_SUSTAIN: u8 = 64;
const CC_PORTAMENTO: u8 = 65;
const CC_SOSTENUTO: u8 = 66;
const CC_SOFT_PEDAL: u8 = 67;
const CC_ALL_SOUND_OFF: u8 = 120;
const CC_ALL_NOTES_OFF: u8 = 123;
```

## Pattern Matching

```rust
use bbx_midi::{MidiMessage, MidiMessageStatus};

fn handle_midi(msg: &MidiMessage) {
    match msg.get_status() {
        MidiMessageStatus::NoteOn => {
            if msg.get_velocity().unwrap() > 0 {
                // Note on with velocity
            } else {
                // Note on with velocity 0 = note off
            }
        }
        MidiMessageStatus::NoteOff => {
            // Note off
        }
        MidiMessageStatus::ControlChange => {
            // CC message
        }
        _ => {}
    }
}
```
