# PannerBlock

Stereo panning with constant-power law.

## Overview

`PannerBlock` positions a mono signal in the stereo field, or adjusts the stereo balance of a stereo signal.

## Creating a Panner

```rust
use bbx_dsp::{
    block::BlockType,
    blocks::PannerBlock,
    graph::GraphBuilder,
};

let mut builder = GraphBuilder::<f32>::new(44100.0, 512, 2);

let pan = builder.add_block(BlockType::Panner(PannerBlock::new(0.0)));  // Center
```

## Pan Values

| Value | Position |
|-------|----------|
| -100.0 | Hard left |
| -50.0 | Half left |
| 0.0 | Center |
| +50.0 | Half right |
| +100.0 | Hard right |

## Port Layout

| Port | Direction | Description |
|------|-----------|-------------|
| 0 | Input | Audio input (mono or left) |
| 0 | Output | Left channel |
| 1 | Output | Right channel |

## Parameters

| Parameter | Type | Range | Default |
|-----------|------|-------|---------|
| position | f32 | -100.0 to 100.0 | 0.0 |

## Constant Power Panning

The panner uses a constant-power law (sine/cosine):

```
Left gain  = cos(pan_angle)
Right gain = sin(pan_angle)
```

Where `pan_angle = (position + 100) / 200 * π/2`

This maintains perceived loudness as the signal moves across the stereo field.

## Usage Examples

### Basic Panning

```rust
use bbx_dsp::{
    block::BlockType,
    blocks::PannerBlock,
    graph::GraphBuilder,
    waveform::Waveform,
};

let mut builder = GraphBuilder::<f32>::new(44100.0, 512, 2);

let osc = builder.add_oscillator(440.0, Waveform::Sine, None);
let pan = builder.add_block(BlockType::Panner(PannerBlock::new(50.0)));  // Slightly right

builder.connect(osc, 0, pan, 0);
```

### Auto-Pan with LFO

```rust
use bbx_dsp::{
    block::BlockType,
    blocks::PannerBlock,
    graph::GraphBuilder,
    waveform::Waveform,
};

let mut builder = GraphBuilder::<f32>::new(44100.0, 512, 2);

let osc = builder.add_oscillator(440.0, Waveform::Sine, None);

// Slow pan LFO (0.25 Hz, full depth)
let lfo = builder.add_lfo(0.25, 1.0, None);

// Panner block
let pan = builder.add_block(BlockType::Panner(PannerBlock::new(0.0)));

builder.connect(osc, 0, pan, 0);

// Modulate pan position
builder.modulate(lfo, pan, "position");
```

### Stereo Width

For stereo sources, use two panners:

```rust
use bbx_dsp::{
    block::BlockType,
    blocks::PannerBlock,
    graph::GraphBuilder,
};

let mut builder = GraphBuilder::<f32>::new(44100.0, 512, 2);

// Narrow the stereo field
let left_pan = builder.add_block(BlockType::Panner(PannerBlock::new(-30.0)));   // Less extreme left
let right_pan = builder.add_block(BlockType::Panner(PannerBlock::new(30.0)));   // Less extreme right
```

## Implementation Notes

- Constant-power pan law prevents center dip
- Mono input → stereo output
- Click-free panning via linear smoothing
