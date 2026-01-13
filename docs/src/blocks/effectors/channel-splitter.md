# ChannelSplitterBlock

Splits multi-channel input into individual mono outputs.

## Overview

`ChannelSplitterBlock` separates a multi-channel signal into individual mono outputs, allowing downstream blocks to process each channel independently.

## Creating a Splitter

```rust
use bbx_dsp::{blocks::ChannelSplitterBlock, graph::GraphBuilder};

let mut builder = GraphBuilder::<f32>::new(44100.0, 512, 2);

// Split 6 channels (e.g., 5.1 surround)
let splitter = builder.add(ChannelSplitterBlock::new(6));
```

Or with direct construction:

```rust
use bbx_dsp::blocks::ChannelSplitterBlock;

let splitter = ChannelSplitterBlock::<f32>::new(4);
```

## Port Layout

| Port | Direction | Description |
|------|-----------|-------------|
| 0..N | Input | Multi-channel input |
| 0..N | Output | Individual mono outputs |

Input and output counts are equal, determined by the `channels` parameter (1-16).

## Parameters

| Parameter | Type | Range | Description |
|-----------|------|-------|-------------|
| channels | usize | 1-16 | Number of channels to split |

## Usage Examples

### Split Stereo for Independent Processing

```rust
use bbx_dsp::{
    blocks::{ChannelMergerBlock, ChannelSplitterBlock, GainBlock, OscillatorBlock},
    graph::GraphBuilder,
    waveform::Waveform,
};

let mut builder = GraphBuilder::<f32>::new(44100.0, 512, 2);

// Source with stereo output
let osc = builder.add(OscillatorBlock::new(440.0, Waveform::Sine, None));

// Split stereo to mono channels
let splitter = builder.add(ChannelSplitterBlock::new(2));
builder.connect(osc, 0, splitter, 0);
builder.connect(osc, 0, splitter, 1);

// Process left channel differently
let left_gain = builder.add(GainBlock::new(-3.0, None));
builder.connect(splitter, 0, left_gain, 0);

// Process right channel differently
let right_gain = builder.add(GainBlock::new(-6.0, None));
builder.connect(splitter, 1, right_gain, 0);

// Merge back to stereo
let merger = builder.add(ChannelMergerBlock::new(2));
builder.connect(left_gain, 0, merger, 0);
builder.connect(right_gain, 0, merger, 1);
```

### Split 5.1 Surround for Per-Channel Effects

```rust
use bbx_dsp::{blocks::ChannelSplitterBlock, graph::GraphBuilder};

let mut builder = GraphBuilder::<f32>::new(44100.0, 512, 6);

// Split 6-channel surround input
let splitter = builder.add(ChannelSplitterBlock::new(6));

// Now you can process L, R, C, LFE, Ls, Rs independently
// splitter output 0 = L
// splitter output 1 = R
// splitter output 2 = C
// splitter output 3 = LFE
// splitter output 4 = Ls
// splitter output 5 = Rs
```

## Implementation Notes

- Zero-latency operation (direct copy)
- Uses `ChannelConfig::Explicit` (handles routing internally)
- Panics if `channels` is 0 or greater than 16
