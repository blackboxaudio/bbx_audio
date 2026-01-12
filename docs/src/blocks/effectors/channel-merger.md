# ChannelMergerBlock

Merges individual mono inputs into a multi-channel output.

## Overview

`ChannelMergerBlock` combines separate mono signals into a single multi-channel stream, typically used after splitting and processing channels independently.

## Creating a Merger

```rust
use bbx_dsp::graph::GraphBuilder;

let mut builder = GraphBuilder::<f32>::new(44100.0, 512, 2);

// Merge 6 mono inputs into 6-channel output
let merger = builder.add_channel_merger(6);
```

Or with direct construction:

```rust
use bbx_dsp::{
    block::BlockType,
    blocks::ChannelMergerBlock,
    graph::GraphBuilder,
};

let mut builder = GraphBuilder::<f32>::new(44100.0, 512, 2);

let merger = builder.add_block(BlockType::ChannelMerger(
    ChannelMergerBlock::new(4)
));
```

## Port Layout

| Port | Direction | Description |
|------|-----------|-------------|
| 0..N | Input | Individual mono inputs |
| 0..N | Output | Multi-channel output |

Input and output counts are equal, determined by the `channels` parameter (1-16).

## Parameters

| Parameter | Type | Range | Description |
|-----------|------|-------|-------------|
| channels | usize | 1-16 | Number of channels to merge |

## Usage Examples

### Merge Two Mono Signals to Stereo

```rust
use bbx_dsp::{graph::GraphBuilder, waveform::Waveform};

let mut builder = GraphBuilder::<f32>::new(44100.0, 512, 2);

// Two separate oscillators
let osc_left = builder.add_oscillator(440.0, Waveform::Sine, None);
let osc_right = builder.add_oscillator(442.0, Waveform::Sine, None);

// Merge to stereo
let merger = builder.add_channel_merger(2);
builder.connect(osc_left, 0, merger, 0);
builder.connect(osc_right, 0, merger, 1);
```

### Reassemble Split Channels After Processing

```rust
use bbx_dsp::graph::GraphBuilder;

let mut builder = GraphBuilder::<f32>::new(44100.0, 512, 2);

// Split stereo input
let splitter = builder.add_channel_splitter(2);

// Process each channel (add effects here)
let left_drive = builder.add_overdrive(2.0, 1.0, 0.5, 44100.0);
let right_drive = builder.add_overdrive(4.0, 1.0, 0.8, 44100.0);

builder.connect(splitter, 0, left_drive, 0);
builder.connect(splitter, 1, right_drive, 0);

// Merge back to stereo
let merger = builder.add_channel_merger(2);
builder.connect(left_drive, 0, merger, 0);
builder.connect(right_drive, 0, merger, 1);
```

### Create Custom Surround Layout

```rust
use bbx_dsp::{graph::GraphBuilder, waveform::Waveform};

let mut builder = GraphBuilder::<f32>::new(44100.0, 512, 4);

// Four separate sources
let front_left = builder.add_oscillator(440.0, Waveform::Sine, None);
let front_right = builder.add_oscillator(440.0, Waveform::Sine, None);
let rear_left = builder.add_oscillator(220.0, Waveform::Sine, None);
let rear_right = builder.add_oscillator(220.0, Waveform::Sine, None);

// Merge to quad
let merger = builder.add_channel_merger(4);
builder.connect(front_left, 0, merger, 0);
builder.connect(front_right, 0, merger, 1);
builder.connect(rear_left, 0, merger, 2);
builder.connect(rear_right, 0, merger, 3);
```

## Implementation Notes

- Zero-latency operation (direct copy)
- Uses `ChannelConfig::Explicit` (handles routing internally)
- Panics if `channels` is 0 or greater than 16
- Complement to `ChannelSplitterBlock`
