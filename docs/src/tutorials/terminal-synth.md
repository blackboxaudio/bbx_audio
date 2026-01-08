# Building a Terminal Synthesizer

This tutorial shows you how to create a simple synthesizer that runs in your terminal and plays audio through your speakers.

## Prerequisites

- Rust nightly toolchain installed
- Audio output device (speakers or headphones)

## Creating Your Project

Create a new Rust project:

```bash
cargo new my_synth
cd my_synth
```

Set up the nightly toolchain for this project:

```bash
rustup override set nightly
```

## Configure Dependencies

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

## Writing the Code

Replace the contents of `src/main.rs` with the following:

```rust
use std::time::Duration;

use bbx_dsp::prelude::*;
use rodio::{OutputStream, Source};

// === Audio Playback Boilerplate ===

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

// === Your Synthesizer ===

fn create_graph() -> Graph<f32> {
    let mut builder = GraphBuilder::new(DEFAULT_SAMPLE_RATE, DEFAULT_BUFFER_SIZE, 2);

    let osc = builder.add_oscillator(440.0, Waveform::Sine, None);
    let gain = builder.add_gain(-6.0);

    builder.connect(osc, 0, gain, 0);
    builder.build()
}

fn main() {
    play(create_graph(), 3);
}
```

## Understanding the Code

**Audio Playback Boilerplate**: The `Signal` struct wraps a DSP `Graph` and implements the `Iterator` and `Source` traits required by rodio for audio playback. The `play()` function handles setting up the audio output stream. You can copy this boilerplate to any project that uses bbx_dsp.

**create_graph()**: This function builds our synthesizer:
- Creates a `GraphBuilder` with 44.1kHz sample rate, 512-sample buffer, and stereo output
- Adds a 440Hz sine wave oscillator (concert A)
- Adds a gain block at -6dB (half amplitude, for comfortable listening)
- Connects the oscillator to the gain block

**main()**: Creates the graph and plays it for 3 seconds.

## Running Your Synth

Run your synthesizer with:

```bash
cargo run --release
```

You should hear a 3-second sine wave tone at 440Hz.

## Experimenting

Try modifying the code to explore different sounds:

**Change the frequency**:
```rust
let osc = builder.add_oscillator(220.0, Waveform::Sine, None);  // One octave lower
```

**Change the waveform**:
```rust
let osc = builder.add_oscillator(440.0, Waveform::Saw, None);   // Brighter, buzzier
let osc = builder.add_oscillator(440.0, Waveform::Square, None); // Hollow, woody
```

**Adjust the volume**:
```rust
let gain = builder.add_gain(-12.0);  // Quieter (-12dB)
let gain = builder.add_gain(0.0);    // Full volume (0dB)
```

**Play longer** (press Ctrl+C to stop):
```rust
play(create_graph(), 60);  // Play for 60 seconds
```

## Next Steps

- [Creating a Simple Oscillator](oscillator.md) - Explore all waveform types
- [Adding Effects](effects.md) - Add filters and distortion
- [Parameter Modulation with LFOs](modulation.md) - Animate parameters over time
