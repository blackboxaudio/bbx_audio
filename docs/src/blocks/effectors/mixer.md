# MixerBlock

A channel-wise audio mixer that sums multiple sources per output channel.

## Overview

`MixerBlock` automatically groups inputs by channel and sums them, making it easy to combine multiple audio sources into a single multi-channel output. Unlike `MatrixMixerBlock` which requires explicit gain configuration, `MixerBlock` handles grouping automatically.

## Creating a Mixer

```rust
use bbx_dsp::graph::GraphBuilder;

let mut builder = GraphBuilder::<f32>::new(44100.0, 512, 2);

// Mix 3 stereo sources into stereo output
let mixer = builder.add_mixer(3, 2);

// Convenience method for stereo mixing
let stereo_mixer = builder.add_stereo_mixer(4);
```

Or with direct construction:

```rust
use bbx_dsp::{
    block::BlockType,
    blocks::MixerBlock,
    graph::GraphBuilder,
};

let mut builder = GraphBuilder::<f32>::new(44100.0, 512, 2);

// Mix 2 stereo sources with average normalization
let mixer = MixerBlock::<f32>::stereo(2)
    .with_normalization(NormalizationStrategy::Average);

let mix = builder.add_block(BlockType::Mixer(mixer));
```

## Port Layout

| Port | Direction | Description |
|------|-----------|-------------|
| 0..N*C | Input | N sources * C channels each |
| 0..C | Output | C output channels |

Inputs are organized in groups. For a stereo mixer with 3 sources:
- Inputs 0, 1: Source A (L, R)
- Inputs 2, 3: Source B (L, R)
- Inputs 4, 5: Source C (L, R)

The mixer sums: Output L = A.L + B.L + C.L, Output R = A.R + B.R + C.R

## Parameters

| Parameter | Type | Range | Description |
|-----------|------|-------|-------------|
| num_sources | usize | 1+ | Number of sources to mix |
| num_channels | usize | 1-16 | Channels per source/output |

Total inputs (`num_sources * num_channels`) must not exceed 16.

## Normalization Strategies

`MixerBlock` applies normalization to prevent clipping:

| Strategy | Formula | Use Case |
|----------|---------|----------|
| `ConstantPower` (default) | sum / sqrt(N) | Preserves perceived loudness |
| `Average` | sum / N | Simple averaging |

## Usage Examples

### Mix Two Stereo Signals

```rust
use bbx_dsp::{graph::GraphBuilder, waveform::Waveform};

let mut builder = GraphBuilder::<f32>::new(44100.0, 512, 2);

// Two oscillators
let osc1 = builder.add_oscillator(440.0, Waveform::Sine, None);
let osc2 = builder.add_oscillator(880.0, Waveform::Sine, None);

// Pan them to different positions
let pan1 = builder.add_panner_stereo(-50.0);
let pan2 = builder.add_panner_stereo(50.0);

builder.connect(osc1, 0, pan1, 0);
builder.connect(osc2, 0, pan2, 0);

// Mix the stereo signals together
let mixer = builder.add_stereo_mixer(2);
builder
    .connect(pan1, 0, mixer, 0)  // Source A left
    .connect(pan1, 1, mixer, 1)  // Source A right
    .connect(pan2, 0, mixer, 2)  // Source B left
    .connect(pan2, 1, mixer, 3); // Source B right

let graph = builder.build();
```

### Mix Three Mono Sources

```rust
use bbx_dsp::{
    block::BlockType,
    blocks::MixerBlock,
    graph::GraphBuilder,
    waveform::Waveform,
};

let mut builder = GraphBuilder::<f32>::new(44100.0, 512, 1);

let osc1 = builder.add_oscillator(220.0, Waveform::Sine, None);
let osc2 = builder.add_oscillator(330.0, Waveform::Sine, None);
let osc3 = builder.add_oscillator(440.0, Waveform::Sine, None);

// 3 mono sources -> 1 mono output
let mixer = builder.add_mixer(3, 1);
builder
    .connect(osc1, 0, mixer, 0)
    .connect(osc2, 0, mixer, 1)
    .connect(osc3, 0, mixer, 2);

let graph = builder.build();
```

### Custom Normalization

```rust
use bbx_dsp::{
    block::BlockType,
    blocks::effectors::mixer::{MixerBlock, NormalizationStrategy},
    graph::GraphBuilder,
};

let mut builder = GraphBuilder::<f32>::new(44100.0, 512, 2);

// Use average normalization instead of constant power
let mixer = MixerBlock::<f32>::stereo(4)
    .with_normalization(NormalizationStrategy::Average);

let mix = builder.add_block(BlockType::Mixer(mixer));
```

## Implementation Notes

- Uses `ChannelConfig::Explicit` (handles channel routing internally)
- Default normalization is `ConstantPower` for natural-sounding mixes
- Panics if `num_sources` or `num_channels` is 0
- Panics if total inputs exceed `MAX_BLOCK_INPUTS` (16)
