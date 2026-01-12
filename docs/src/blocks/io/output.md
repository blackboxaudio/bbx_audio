# OutputBlock

Terminal output block for collecting processed audio.

## Overview

`OutputBlock` is the graph's audio output destination. It collects processed audio from connected blocks and provides it via `process_buffers()`.

## Automatic Creation

`OutputBlock` is automatically added when building a graph:

```rust
let mut builder = GraphBuilder::<f32>::new(44100.0, 512, 2);
let osc = builder.add_oscillator(440.0, Waveform::Sine, None);
// OutputBlock added automatically during build()
let graph = builder.build();
```

## Port Layout

| Port | Direction | Description |
|------|-----------|-------------|
| 0 | Input | Left channel |
| 1 | Input | Right channel |
| N | Input | Channel N (up to num_channels) |

## Usage

### Basic Usage

```rust
let mut builder = GraphBuilder::<f32>::new(44100.0, 512, 2);

let osc = builder.add_oscillator(440.0, Waveform::Sine, None);
// Oscillator automatically connects to output

let mut graph = builder.build();

// Collect processed audio
let mut left = vec![0.0f32; 512];
let mut right = vec![0.0f32; 512];
let mut outputs: [&mut [f32]; 2] = [&mut left, &mut right];
graph.process_buffers(&mut outputs);

// 'left' and 'right' now contain the oscillator output
```

### Explicit Connection

For complex routing, connect blocks explicitly:

```rust
use bbx_dsp::{
    block::BlockType,
    blocks::{GainBlock, PannerBlock},
    graph::GraphBuilder,
    waveform::Waveform,
};

let mut builder = GraphBuilder::<f32>::new(44100.0, 512, 2);

let osc = builder.add_oscillator(440.0, Waveform::Sine, None);
let gain = builder.add_block(BlockType::Gain(GainBlock::new(-6.0, None)));
let pan = builder.add_block(BlockType::Panner(PannerBlock::new(0.0)));

builder.connect(osc, 0, gain, 0);
builder.connect(gain, 0, pan, 0);
// pan's outputs will be collected

let graph = builder.build();
```

## Channel Count

The number of output channels is set at graph creation:

```rust
// Mono output
let builder = GraphBuilder::<f32>::new(44100.0, 512, 1);

// Stereo output
let builder = GraphBuilder::<f32>::new(44100.0, 512, 2);

// 5.1 surround
let builder = GraphBuilder::<f32>::new(44100.0, 512, 6);
```

## Output Buffer

The output buffer must match the graph's configuration:

```rust
let num_channels = 2;
let buffer_size = 512;

let mut buffers: Vec<Vec<f32>> = (0..num_channels)
    .map(|_| vec![0.0; buffer_size])
    .collect();

let mut outputs: Vec<&mut [f32]> = buffers
    .iter_mut()
    .map(|b| b.as_mut_slice())
    .collect();

graph.process_buffers(&mut outputs);
```

## Implementation Notes

- Terminal block (no outputs)
- Collects audio from all connected sources
- Multiple connections are summed
- Part of the graph's execution order
