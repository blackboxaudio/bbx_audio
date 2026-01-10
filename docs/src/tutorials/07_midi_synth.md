# Building a MIDI Synthesizer

This tutorial shows you how to create a real-time MIDI synthesizer that responds to a USB MIDI keyboard.

## Prerequisites

- Rust nightly toolchain
- Audio output device
- USB MIDI keyboard (or virtual MIDI source)

## Overview

We'll build a monophonic subtractive synthesizer with this signal chain:

```
Oscillator → VCA ← Envelope → LowPassFilter → Output
```

The synthesizer uses three threads:
- **Main thread**: Setup and Ctrl+C handling
- **MIDI thread**: Receives MIDI input via midir
- **Audio thread**: Processes DSP graph via rodio

Communication between threads uses a lock-free SPSC ring buffer for real-time safety.

## Dependencies

```toml
[dependencies]
bbx_dsp = "0.3"
bbx_midi = "0.1"
rodio = "0.20"
midir = "0.11"
ctrlc = "3.4"
```

## The MIDI Buffer

bbx_midi provides a lock-free buffer for thread-safe MIDI communication:

```rust
use bbx_midi::{midi_buffer, MidiBufferProducer, MidiBufferConsumer, MidiMessage};

// Create producer/consumer pair
let (mut producer, mut consumer) = midi_buffer(256);

// MIDI thread: push messages
producer.try_send(MidiMessage::new([0x90, 60, 100]));

// Audio thread: pop messages (realtime-safe)
while let Some(msg) = consumer.try_pop() {
    // Process MIDI...
}
```

## Voice State

Track the currently playing note for monophonic behavior:

```rust
struct VoiceState {
    current_note: Option<u8>,
}

impl VoiceState {
    fn new() -> Self {
        Self { current_note: None }
    }

    fn note_on(&mut self, note: u8) {
        self.current_note = Some(note);
    }

    fn note_off(&mut self, note: u8) -> bool {
        if self.current_note == Some(note) {
            self.current_note = None;
            true
        } else {
            false
        }
    }
}
```

## The Synthesizer

```rust
use bbx_dsp::{
    block::BlockId,
    buffer::{AudioBuffer, Buffer},
    context::{DEFAULT_BUFFER_SIZE, DEFAULT_SAMPLE_RATE},
    graph::{Graph, GraphBuilder},
    waveform::Waveform,
};
use bbx_midi::{MidiBufferConsumer, MidiMessage, MidiMessageStatus};

struct MidiSynth {
    graph: Graph<f32>,
    output_buffers: Vec<AudioBuffer<f32>>,
    voice_state: VoiceState,
    oscillator_id: BlockId,
    envelope_id: BlockId,
    midi_consumer: MidiBufferConsumer,
    sample_rate: u32,
    num_channels: usize,
    buffer_size: usize,
    channel_index: usize,
    sample_index: usize,
}

impl MidiSynth {
    fn new(midi_consumer: MidiBufferConsumer) -> Self {
        let sample_rate = DEFAULT_SAMPLE_RATE;
        let buffer_size = DEFAULT_BUFFER_SIZE;
        let num_channels = 2;

        let mut builder = GraphBuilder::new(sample_rate, buffer_size, num_channels);

        // Build signal chain: Oscillator → VCA ← Envelope → Filter → Gain
        let oscillator_id = builder.add_oscillator(440.0, Waveform::Sawtooth, None);
        let envelope_id = builder.add_envelope(0.01, 0.1, 0.7, 0.3);
        let vca_id = builder.add_vca();
        let filter_id = builder.add_low_pass_filter(2000.0, 1.5);
        let gain_id = builder.add_gain(-6.0);

        builder
            .connect(oscillator_id, 0, vca_id, 0)    // Audio to VCA
            .connect(envelope_id, 0, vca_id, 1)      // Envelope controls VCA
            .connect(vca_id, 0, filter_id, 0)
            .connect(filter_id, 0, gain_id, 0);

        let graph = builder.build();

        let mut output_buffers = Vec::with_capacity(num_channels);
        for _ in 0..num_channels {
            output_buffers.push(AudioBuffer::new(buffer_size));
        }

        Self {
            graph,
            output_buffers,
            voice_state: VoiceState::new(),
            oscillator_id,
            envelope_id,
            midi_consumer,
            sample_rate: sample_rate as u32,
            num_channels,
            buffer_size,
            channel_index: 0,
            sample_index: 0,
        }
    }
}
```

## Processing MIDI Events

Poll the MIDI buffer at the start of each audio block:

```rust
impl MidiSynth {
    fn process_midi_events(&mut self) {
        while let Some(msg) = self.midi_consumer.try_pop() {
            match msg.get_status() {
                MidiMessageStatus::NoteOn => {
                    let velocity = msg.get_velocity().unwrap_or(0);
                    if velocity > 0 {
                        if let Some(note) = msg.get_note_number() {
                            self.voice_state.note_on(note);
                            if let Some(freq) = msg.get_note_frequency() {
                                self.set_oscillator_frequency(freq);
                            }
                            self.trigger_envelope();
                        }
                    } else {
                        self.handle_note_off(&msg);
                    }
                }
                MidiMessageStatus::NoteOff => {
                    self.handle_note_off(&msg);
                }
                _ => {}
            }
        }
    }

    fn handle_note_off(&mut self, msg: &MidiMessage) {
        if let Some(note) = msg.get_note_number() {
            if self.voice_state.note_off(note) {
                self.release_envelope();
            }
        }
    }

    fn set_oscillator_frequency(&mut self, frequency: f32) {
        if let Some(bbx_dsp::block::BlockType::Oscillator(osc)) =
            self.graph.get_block_mut(self.oscillator_id)
        {
            osc.set_midi_frequency(frequency);
        }
    }

    fn trigger_envelope(&mut self) {
        if let Some(bbx_dsp::block::BlockType::Envelope(env)) =
            self.graph.get_block_mut(self.envelope_id)
        {
            env.note_on();
        }
    }

    fn release_envelope(&mut self) {
        if let Some(bbx_dsp::block::BlockType::Envelope(env)) =
            self.graph.get_block_mut(self.envelope_id)
        {
            env.note_off();
        }
    }
}
```

## Audio Source Implementation

Implement `Iterator` and `Source` for rodio playback:

```rust
use std::time::Duration;
use rodio::Source;

impl MidiSynth {
    fn process(&mut self) -> f32 {
        // Process MIDI at start of each buffer
        if self.channel_index == 0 && self.sample_index == 0 {
            self.process_midi_events();

            let mut output_refs: Vec<&mut [f32]> =
                self.output_buffers.iter_mut().map(|b| b.as_mut_slice()).collect();
            self.graph.process_buffers(&mut output_refs);
        }

        let sample = self.output_buffers[self.channel_index][self.sample_index];

        self.channel_index += 1;
        if self.channel_index >= self.num_channels {
            self.channel_index = 0;
            self.sample_index += 1;
            self.sample_index %= self.buffer_size;
        }

        sample
    }
}

impl Iterator for MidiSynth {
    type Item = f32;
    fn next(&mut self) -> Option<Self::Item> {
        Some(self.process())
    }
}

impl Source for MidiSynth {
    fn current_frame_len(&self) -> Option<usize> { None }
    fn channels(&self) -> u16 { self.num_channels as u16 }
    fn sample_rate(&self) -> u32 { self.sample_rate }
    fn total_duration(&self) -> Option<Duration> { None }
}
```

## MIDI Input Setup

Connect to a MIDI port using midir:

```rust
use std::io::{Write, stdin, stdout};
use midir::{Ignore, MidiInput, MidiInputConnection};
use bbx_midi::{MidiBufferProducer, MidiMessage};

fn setup_midi_input(mut producer: MidiBufferProducer) -> Option<MidiInputConnection<()>> {
    let mut midi_in = MidiInput::new("MIDI Synth Input").ok()?;
    midi_in.ignore(Ignore::None);

    let in_ports = midi_in.ports();
    let in_port = match in_ports.len() {
        0 => {
            println!("No MIDI input ports found.");
            return None;
        }
        1 => {
            let port_name = midi_in.port_name(&in_ports[0]).unwrap_or_default();
            println!("Using MIDI port: {port_name}");
            in_ports[0].clone()
        }
        _ => {
            println!("\nAvailable MIDI input ports:");
            for (idx, port) in in_ports.iter().enumerate() {
                let name = midi_in.port_name(port).unwrap_or_default();
                println!("  {idx}: {name}");
            }
            print!("\nSelect port: ");
            stdout().flush().ok()?;

            let mut input = String::new();
            stdin().read_line(&mut input).ok()?;
            let idx: usize = input.trim().parse().ok()?;

            in_ports.get(idx)?.clone()
        }
    };

    midi_in.connect(
        &in_port,
        "midi-synth-input",
        move |_timestamp, message_bytes, _| {
            let message = MidiMessage::from(message_bytes);
            let _ = producer.try_send(message);
        },
        (),
    ).ok()
}
```

## Main Function

```rust
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use bbx_midi::midi_buffer;
use rodio::OutputStream;

fn main() {
    println!("BBX MIDI Synthesizer\n");

    // Ctrl+C handler
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    }).expect("Error setting Ctrl-C handler");

    // Create MIDI buffer
    let (producer, consumer) = midi_buffer(256);

    // Set up MIDI input
    let _midi_connection = match setup_midi_input(producer) {
        Some(conn) => conn,
        None => {
            println!("Failed to set up MIDI input.");
            return;
        }
    };

    println!("\nSynth ready! Play notes on your MIDI keyboard.");
    println!("Press Ctrl+C to exit.\n");

    // Create synth and start audio
    let synth = MidiSynth::new(consumer);

    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    stream_handle.play_raw(synth.convert_samples()).unwrap();

    // Wait for Ctrl+C
    while running.load(Ordering::SeqCst) {
        std::thread::sleep(Duration::from_millis(100));
    }

    println!("\nShutting down...");
}
```

## Running the Example

The complete example is available in the bbx_sandbox crate:

```bash
cargo run --release --example 06_midi_synth -p bbx_sandbox
```

## Modifications to Try

**Change the oscillator waveform:**
```rust
let oscillator_id = builder.add_oscillator(440.0, Waveform::Square, None);
```

**Adjust the envelope shape:**
```rust
// Plucky sound: fast attack and decay
let envelope_id = builder.add_envelope(0.001, 0.2, 0.0, 0.1);

// Pad sound: slow attack and release
let envelope_id = builder.add_envelope(0.5, 0.3, 0.8, 1.0);
```

**Change the filter cutoff:**
```rust
let filter_id = builder.add_low_pass_filter(800.0, 2.0);  // Darker, more resonant
```

## Next Steps

- [Parameter Modulation with LFOs](modulation.md) - Add vibrato and filter sweeps
- [Working with Audio Files](audio-files.md) - Sample playback
- [VcaBlock Reference](../blocks/effectors/vca.md) - VCA block details
