# Building a Terminal Synthesizer

This tutorial shows you how to create a synthesizer that runs in your terminal, starting with a simple tone and building up to a real-time MIDI-controlled instrument.

## Prerequisites

- Rust nightly toolchain installed
- Audio output device (speakers or headphones)
- USB MIDI keyboard (optional, for Part 3)

> **Prior knowledge**: Before starting, review [Your First DSP Graph](first-graph.md) for core graph concepts.
>
> **Note**: This tutorial progressively introduces concepts. Parts 2-3 use techniques covered in more depth in [Parameter Modulation](modulation.md) and [MIDI Integration](midi.md).

## Part 1: Basic Sine Wave Synth

### Creating Your Project

Create a new Rust project:

```bash
cargo new my_synth
cd my_synth
```

Set up the nightly toolchain for this project:

```bash
rustup override set nightly
```

### Configure Dependencies

Update your `Cargo.toml`:

```toml
[package]
name = "my_synth"
version = "0.1.0"
edition = "2024"

[dependencies]
bbx_dsp = "0.3.0"
rodio = "0.20.1"
```

### Writing the Code

Replace the contents of `src/main.rs` with the following:

```rust
use std::time::Duration;

use bbx_dsp::prelude::*;
use rodio::{OutputStream, Source};

struct Signal {
    graph: Graph<f32>,
    buffers: Vec<Vec<f32>>,
    num_channels: usize,
    buffer_size: usize,
    sample_rate: u32,
    ch: usize,
    idx: usize,
}

impl Signal {
    fn new(graph: Graph<f32>) -> Self {
        let ctx = graph.context();
        Self {
            buffers: (0..ctx.num_channels).map(|_| vec![0.0; ctx.buffer_size]).collect(),
            num_channels: ctx.num_channels,
            buffer_size: ctx.buffer_size,
            sample_rate: ctx.sample_rate as u32,
            graph,
            ch: 0,
            idx: 0,
        }
    }
}

impl Iterator for Signal {
    type Item = f32;

    fn next(&mut self) -> Option<f32> {
        if self.ch == 0 && self.idx == 0 {
            let mut refs: Vec<&mut [f32]> = self.buffers.iter_mut().map(|b| &mut b[..]).collect();
            self.graph.process_buffers(&mut refs);
        }
        let sample = self.buffers[self.ch][self.idx];
        self.ch += 1;
        if self.ch >= self.num_channels {
            self.ch = 0;
            self.idx = (self.idx + 1) % self.buffer_size;
        }
        Some(sample)
    }
}

impl Source for Signal {
    fn current_frame_len(&self) -> Option<usize> { None }
    fn channels(&self) -> u16 { self.num_channels as u16 }
    fn sample_rate(&self) -> u32 { self.sample_rate }
    fn total_duration(&self) -> Option<Duration> { None }
}

fn play(graph: Graph<f32>, seconds: u64) {
    let (_stream, handle) = OutputStream::try_default().unwrap();
    handle.play_raw(Signal::new(graph).convert_samples()).unwrap();
    std::thread::sleep(Duration::from_secs(seconds));
}

fn create_graph() -> Graph<f32> {
    use bbx_dsp::block::BlockType;
    use bbx_dsp::blocks::GainBlock;

    let mut builder = GraphBuilder::new(DEFAULT_SAMPLE_RATE, DEFAULT_BUFFER_SIZE, 2);

    let osc = builder.add_oscillator(440.0, Waveform::Sine, None);
    let gain = builder.add_block(BlockType::Gain(GainBlock::new(-6.0)));

    builder.connect(osc, 0, gain, 0);
    builder.build()
}

fn main() {
    play(create_graph(), 3);
}
```

### Understanding the Code

**Signal struct**: Wraps a DSP `Graph` and implements the `Iterator` and `Source` traits required by rodio for audio playback. The `play()` function handles audio output setup.

**create_graph()**: Builds our synthesizer:
- Creates a `GraphBuilder` with 44.1kHz sample rate, 512-sample buffer, and stereo output
- Adds a 440Hz sine wave oscillator (concert A)
- Adds a gain block at -6dB (half amplitude, for comfortable listening)
- Connects the oscillator to the gain block

**main()**: Creates the graph and plays it for 3 seconds.

### Running Your Synth

Run your synthesizer with:

```bash
cargo run --release
```

You should hear a 3-second sine wave tone at 440Hz.

## Part 2: Subtractive Synth Voice

Now let's build a more interesting synthesizer with an oscillator, envelope, filter, and VCA. This is the classic subtractive synthesis signal chain:

```
Oscillator -> VCA <- Envelope -> LowPassFilter -> Gain -> Output
```

The VCA (Voltage Controlled Amplifier) multiplies the oscillator's audio by the envelope's control signal, giving us attack/decay/sustain/release dynamics. See [VcaBlock](../blocks/effectors/vca.md#subtractive-synth-voice) for more on this pattern.

### Updated Dependencies

Add the `bbx_dsp` block import:

```toml
[dependencies]
bbx_dsp = "0.3.0"
rodio = "0.20.1"
```

### Subtractive Synth Code

Replace `create_graph()` with:

```rust
use bbx_dsp::{
    block::BlockId,
    graph::{Graph, GraphBuilder},
    context::{DEFAULT_SAMPLE_RATE, DEFAULT_BUFFER_SIZE},
    waveform::Waveform,
};

fn create_synth_voice() -> (Graph<f32>, BlockId, BlockId) {
    let mut builder = GraphBuilder::new(DEFAULT_SAMPLE_RATE, DEFAULT_BUFFER_SIZE, 2);

    // Sound source: sawtooth wave (rich harmonics for filtering)
    let oscillator_id = builder.add_oscillator(440.0, Waveform::Sawtooth, None);

    // Amplitude envelope: controls volume over time
    // See: modulation.md#envelope-generator
    let envelope_id = builder.add_envelope(
        0.01,  // Attack: 10ms
        0.1,   // Decay: 100ms
        0.7,   // Sustain: 70%
        0.3,   // Release: 300ms
    );

    // VCA: multiplies audio by envelope
    let vca_id = builder.add_vca();

    // Low-pass filter: removes harsh high frequencies
    let filter_id = builder.add_low_pass_filter(2000.0, 1.5);

    // Output gain: -6dB for comfortable listening
    let gain_id = builder.add_gain(-6.0);

    // Connect the signal chain
    builder
        .connect(oscillator_id, 0, vca_id, 0)   // Audio to VCA
        .connect(envelope_id, 0, vca_id, 1)     // Envelope controls VCA
        .connect(vca_id, 0, filter_id, 0)       // VCA to filter
        .connect(filter_id, 0, gain_id, 0);     // Filter to output

    (builder.build(), oscillator_id, envelope_id)
}
```

The function returns the graph plus the oscillator and envelope block IDs, which we'll need in Part 3 to control the synth from MIDI.

### Testing the Voice

Update `main()` to trigger the envelope:

```rust
fn main() {
    let (mut graph, _osc_id, env_id) = create_synth_voice();

    // Trigger the envelope
    if let Some(bbx_dsp::block::BlockType::Envelope(env)) = graph.get_block_mut(env_id) {
        env.note_on();
    }

    play(graph, 2);
}
```

Run with `cargo run --release` and you'll hear the envelope shape the sound.

## Part 3: Adding MIDI Input

Now let's make the synth respond to a MIDI keyboard. We'll use a lock-free ring buffer to safely pass MIDI messages from the input thread to the audio thread. See [Lock-Free MIDI Buffer](midi.md#lock-free-midi-buffer) for background on this pattern.

### Additional Dependencies

Update `Cargo.toml`:

```toml
[dependencies]
bbx_dsp = "0.3.0"
bbx_midi = "0.1.0"
rodio = "0.20.1"
midir = "0.11"
ctrlc = "3.4"
```

### Voice State

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

### MIDI Synth Struct

Build on the `Signal` struct pattern, adding MIDI handling:

```rust
use bbx_dsp::{
    block::BlockId,
    buffer::{AudioBuffer, Buffer},
    context::{DEFAULT_BUFFER_SIZE, DEFAULT_SAMPLE_RATE},
    graph::{Graph, GraphBuilder},
    waveform::Waveform,
};
use bbx_midi::{MidiBufferConsumer, MidiMessage, MidiMessageStatus, midi_buffer};
use rodio::Source;
use std::time::Duration;

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

        let oscillator_id = builder.add_oscillator(440.0, Waveform::Sawtooth, None);
        let envelope_id = builder.add_envelope(0.01, 0.1, 0.7, 0.3);
        let vca_id = builder.add_vca();
        let filter_id = builder.add_low_pass_filter(2000.0, 1.5);
        let gain_id = builder.add_gain(-6.0);

        builder
            .connect(oscillator_id, 0, vca_id, 0)
            .connect(envelope_id, 0, vca_id, 1)
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

### Processing MIDI Events

Poll the MIDI buffer at the start of each audio block. See [MIDI Message Types](midi.md#midi-message-types) for the full list of status codes.

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
                            // get_note_frequency() converts MIDI note to Hz
                            // See: midi.md#midi-to-frequency
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

### Audio Source Implementation

Implement `Iterator` and `Source` for rodio playback:

```rust
impl MidiSynth {
    fn process(&mut self) -> f32 {
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

### MIDI Input Setup

Connect to a MIDI port using midir. See [Real-Time MIDI Input](midi.md#real-time-midi-input) for more details.

```rust
use std::io::{Write, stdin, stdout};
use midir::{Ignore, MidiInput, MidiInputConnection};
use bbx_midi::MidiBufferProducer;

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

            if idx >= in_ports.len() {
                println!("Invalid port selection.");
                return None;
            }

            let port_name = midi_in.port_name(&in_ports[idx]).unwrap_or_default();
            println!("Using MIDI port: {port_name}");
            in_ports[idx].clone()
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

### Updated Main Function

```rust
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use rodio::OutputStream;

fn main() {
    println!("BBX MIDI Synthesizer\n");

    // Ctrl+C handler
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    }).expect("Error setting Ctrl-C handler");

    // Create MIDI buffer (lock-free producer/consumer pair)
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

## Running the MIDI Synth

```bash
cargo run --release
```

Connect a USB MIDI keyboard before running. The program will list available ports and let you select one. Play notes to hear the synthesizer respond.

The complete example is also available in the bbx_sandbox crate:

```bash
cargo run --release --example 06_midi_synth -p bbx_sandbox
```

## Experimenting

Try modifying the code to explore different sounds:

**Change the oscillator waveform:**
```rust
let oscillator_id = builder.add_oscillator(440.0, Waveform::Square, None);  // Hollow, woody
let oscillator_id = builder.add_oscillator(440.0, Waveform::Triangle, None); // Soft, flute-like
```

**Adjust the envelope shape:**
```rust
// Plucky sound: fast attack and decay, no sustain
let envelope_id = builder.add_envelope(0.001, 0.2, 0.0, 0.1);

// Pad sound: slow attack and release
let envelope_id = builder.add_envelope(0.5, 0.3, 0.8, 1.0);
```

**Change the filter cutoff:**
```rust
let filter_id = builder.add_low_pass_filter(500.0, 2.0);   // Darker, more resonant
let filter_id = builder.add_low_pass_filter(4000.0, 0.7);  // Brighter, less resonant
```

## Next Steps

Now that you've built a working synthesizer:

- [Parameter Modulation with LFOs](modulation.md) - Add vibrato and filter sweeps (deeper coverage of Part 2 concepts)
- [Working with Audio Files](audio-files.md) - Add sample playback to your synth
- [VcaBlock Reference](../blocks/effectors/vca.md) - VCA block API details

For the concepts used in Part 3:

- [MIDI Integration](midi.md) - Complete MIDI API reference and patterns
