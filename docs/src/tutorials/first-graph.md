# Your First DSP Graph

This tutorial walks you through creating your first audio processing graph with bbx_audio.

## Prerequisites

Add bbx_dsp to your project:

```toml
[dependencies]
bbx_dsp = "0.1"
```

## Creating a Graph

DSP graphs in bbx_audio are built using `GraphBuilder`:

```rust
use bbx_dsp::graph::GraphBuilder;

fn main() {
    // Create a builder with:
    // - 44100 Hz sample rate
    // - 512 sample buffer size
    // - 2 channels (stereo)
    let mut builder = GraphBuilder::<f32>::new(44100.0, 512, 2);

    // Build the graph
    let graph = builder.build();
}
```

## Adding an Oscillator

Let's add a sine wave oscillator:

```rust
use bbx_dsp::{graph::GraphBuilder, waveform::Waveform};

fn main() {
    let mut builder = GraphBuilder::<f32>::new(44100.0, 512, 2);

    // Add a 440 Hz sine wave oscillator
    // The third parameter is an optional seed for the random number generator
    // (used by the Noise waveform for deterministic output)
    let osc = builder.add_oscillator(440.0, Waveform::Sine, None);

    let graph = builder.build();
}
```

The `add_oscillator` method returns a `BlockId` that you can use to connect blocks.

## Processing Audio

Once you have a graph, you can process audio:

```rust
use bbx_dsp::{graph::GraphBuilder, waveform::Waveform};

fn main() {
    let mut builder = GraphBuilder::<f32>::new(44100.0, 512, 2);
    let _osc = builder.add_oscillator(440.0, Waveform::Sine, None);
    let mut graph = builder.build();

    // Create output buffers
    let mut left = vec![0.0f32; 512];
    let mut right = vec![0.0f32; 512];

    // Process into the buffers
    let mut outputs: [&mut [f32]; 2] = [&mut left, &mut right];
    graph.process_buffers(&mut outputs);

    // left and right now contain 512 samples of a 440 Hz sine wave
    println!("First sample: {}", left[0]);
}
```

## Connecting Blocks

Blocks are connected using the `connect` method:

```rust
use bbx_dsp::{
    block::BlockType,
    blocks::GainBlock,
    graph::GraphBuilder,
    waveform::Waveform,
};

fn main() {
    let mut builder = GraphBuilder::<f32>::new(44100.0, 512, 2);

    // Add blocks
    let osc = builder.add_oscillator(440.0, Waveform::Sine, None);
    let gain = builder.add_block(BlockType::Gain(GainBlock::new(-6.0)));  // -6 dB

    // Connect oscillator output 0 to gain input 0
    builder.connect(osc, 0, gain, 0);

    let graph = builder.build();
}
```

## Understanding Block IDs

Each block added to the graph gets a unique `BlockId`:

```rust
let osc = builder.add_oscillator(440.0, Waveform::Sine, None);                     // Block 0
let gain = builder.add_block(BlockType::Gain(GainBlock::new(-6.0)));               // Block 1
let pan = builder.add_block(BlockType::Panner(PannerBlock::new(0.0)));             // Block 2
```

Use these IDs when connecting blocks:

```rust
builder.connect(from_block, from_port, to_block, to_port);
```

## Complete Example

```rust
use bbx_dsp::{
    block::BlockType,
    blocks::{GainBlock, PannerBlock},
    graph::GraphBuilder,
    waveform::Waveform,
};

fn main() {
    // Create builder
    let mut builder = GraphBuilder::<f32>::new(44100.0, 512, 2);

    // Build a simple synth chain
    let osc = builder.add_oscillator(440.0, Waveform::Saw, None);
    let gain = builder.add_block(BlockType::Gain(GainBlock::new(-12.0)));
    let pan = builder.add_block(BlockType::Panner(PannerBlock::new(25.0)));  // Slightly right

    // Connect: Osc -> Gain -> Panner
    builder.connect(osc, 0, gain, 0);
    builder.connect(gain, 0, pan, 0);

    // Build the graph
    let mut graph = builder.build();

    // Process multiple buffers
    let mut left = vec![0.0f32; 512];
    let mut right = vec![0.0f32; 512];

    for _ in 0..100 {
        let mut outputs: [&mut [f32]; 2] = [&mut left, &mut right];
        graph.process_buffers(&mut outputs);

        // Do something with the audio...
    }
}
```

## Next Steps

- [Building a Terminal Synthesizer](terminal-synth.md) - Hear your first synth
- [Creating a Simple Oscillator](oscillator.md) - Explore different waveforms
- [Adding Effects](effects.md) - Add more processing blocks
- [Parameter Modulation](modulation.md) - Use LFOs to modulate parameters
