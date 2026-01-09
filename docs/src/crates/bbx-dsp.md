# bbx_dsp

A block-based audio DSP system for building signal processing graphs.

## Overview

bbx_dsp is the core DSP crate providing:

- Graph-based architecture for connecting DSP blocks
- Automatic topological sorting for correct execution order
- Real-time safe processing with stack-allocated buffers
- Parameter modulation via LFOs and envelopes

## Installation

```toml
[dependencies]
bbx_dsp = "0.1"
```

## Features

| Feature | Description |
|---------|-------------|
| [Graph](dsp/graph.md) | Block graph and builder |
| [Block Trait](dsp/block-trait.md) | Interface for DSP blocks |
| [BlockType](dsp/block-type.md) | Enum wrapping all blocks |
| Sample | Re-exported from [bbx_core](bbx-core.md) |
| [DspContext](dsp/context.md) | Processing context |
| [Parameters](dsp/parameters.md) | Modulation system |

## Quick Example

```rust
use bbx_dsp::{
    block::BlockType,
    blocks::GainBlock,
    graph::GraphBuilder,
    waveform::Waveform,
};

// Create a graph: 44.1kHz, 512 samples, stereo
let mut builder = GraphBuilder::<f32>::new(44100.0, 512, 2);

// Add blocks
let osc = builder.add_oscillator(440.0, Waveform::Sine, None);
let gain = builder.add_block(BlockType::Gain(GainBlock::new(-6.0)));

// Connect: oscillator -> gain
builder.connect(osc, 0, gain, 0);

// Build the graph
let mut graph = builder.build();

// Process audio
let mut left = vec![0.0f32; 512];
let mut right = vec![0.0f32; 512];
let mut outputs: [&mut [f32]; 2] = [&mut left, &mut right];
graph.process_buffers(&mut outputs);
```

## Block Categories

### Generators

Blocks that create audio signals:

- `OscillatorBlock` - Waveform generator

### Effectors

Blocks that process audio:

- `GainBlock` - Level control
- `PannerBlock` - Stereo panning
- `OverdriveBlock` - Distortion
- `DcBlockerBlock` - DC removal
- `ChannelRouterBlock` - Channel routing

### Modulators

Blocks that generate control signals:

- `LfoBlock` - Low-frequency oscillator
- `EnvelopeBlock` - ADSR envelope

### I/O

Blocks for input/output:

- `FileInputBlock` - Audio file input
- `FileOutputBlock` - Audio file output
- `OutputBlock` - Graph output

## Architecture

The DSP system uses a pull model:

1. `GraphBuilder` collects blocks and connections
2. `build()` creates an optimized `Graph`
3. Topological sorting determines execution order
4. `process_buffers()` runs all blocks in order

See [DSP Graph Architecture](../architecture/graph-architecture.md) for details.
