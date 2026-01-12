# PannerBlock

Multi-format spatial panning supporting stereo, surround, and ambisonics.

## Overview

`PannerBlock` positions a mono signal in 2D or 3D space using three different algorithms:

- **Stereo**: Constant-power panning between left and right
- **Surround**: VBAP (Vector Base Amplitude Panning) for 5.1/7.1 layouts
- **Ambisonic**: Spherical harmonic encoding for immersive audio

## Panning Modes

### Stereo Mode

Traditional left-right panning using constant-power pan law.

```rust
use bbx_dsp::{
    block::BlockType,
    blocks::PannerBlock,
    graph::GraphBuilder,
};

let mut builder = GraphBuilder::<f32>::new(44100.0, 512, 2);

// Position: -100 (left) to +100 (right)
let pan = builder.add_block(BlockType::Panner(PannerBlock::new(0.0)));  // Center
let pan = builder.add_block(BlockType::Panner(PannerBlock::new_stereo(-50.0)));  // Half left
```

### Surround Mode (VBAP)

Uses Vector Base Amplitude Panning for 5.1 and 7.1 speaker layouts.

```rust
use bbx_dsp::{channel::ChannelLayout, graph::GraphBuilder};

let mut builder = GraphBuilder::<f32>::new(44100.0, 512, 6);

// 5.1 surround panner
let pan = builder.add_panner_surround(ChannelLayout::Surround51);

// 7.1 surround panner
let pan = builder.add_panner_surround(ChannelLayout::Surround71);
```

Control source position with `azimuth` and `elevation` parameters.

### Ambisonic Mode

Encodes mono input to SN3D normalized, ACN ordered B-format for immersive audio.

```rust
use bbx_dsp::graph::GraphBuilder;

let mut builder = GraphBuilder::<f32>::new(44100.0, 512, 4);

// First-order ambisonics (4 channels)
let pan = builder.add_panner_ambisonic(1);

// Second-order ambisonics (9 channels)
let pan = builder.add_panner_ambisonic(2);

// Third-order ambisonics (16 channels)
let pan = builder.add_panner_ambisonic(3);
```

## Port Layout

Port counts vary by mode:

| Mode | Inputs | Outputs |
|------|--------|---------|
| Stereo | 1 | 2 |
| Surround 5.1 | 1 | 6 |
| Surround 7.1 | 1 | 8 |
| Ambisonic FOA | 1 | 4 |
| Ambisonic SOA | 1 | 9 |
| Ambisonic TOA | 1 | 16 |

## Parameters

| Parameter | Type | Range | Mode | Default |
|-----------|------|-------|------|---------|
| position | f32 | -100.0 to 100.0 | Stereo | 0.0 |
| azimuth | f32 | -180.0 to 180.0 | Surround, Ambisonic | 0.0 |
| elevation | f32 | -90.0 to 90.0 | Surround, Ambisonic | 0.0 |

### Position Values (Stereo)

| Value | Position |
|-------|----------|
| -100.0 | Hard left |
| -50.0 | Half left |
| 0.0 | Center |
| +50.0 | Half right |
| +100.0 | Hard right |

### Azimuth/Elevation (Surround/Ambisonic)

| Azimuth | Direction |
|---------|-----------|
| 0 | Front |
| 90 | Left |
| -90 | Right |
| 180 / -180 | Rear |

| Elevation | Direction |
|-----------|-----------|
| 0 | Horizon |
| 90 | Above |
| -90 | Below |

## Usage Examples

### Basic Stereo Panning

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
let lfo = builder.add_lfo(0.25, 1.0, None);  // 0.25 Hz, full depth
let pan = builder.add_block(BlockType::Panner(PannerBlock::new(0.0)));

builder.connect(osc, 0, pan, 0);
builder.modulate(lfo, pan, "position");
```

### Surround Panning with VBAP

```rust
use bbx_dsp::{channel::ChannelLayout, graph::GraphBuilder, waveform::Waveform};

let mut builder = GraphBuilder::<f32>::new(44100.0, 512, 6);

let osc = builder.add_oscillator(440.0, Waveform::Sine, None);
let pan = builder.add_panner_surround(ChannelLayout::Surround51);

builder.connect(osc, 0, pan, 0);

// Modulate azimuth for circular motion
let lfo = builder.add_lfo(0.1, 1.0, None);
builder.modulate(lfo, pan, "azimuth");
```

### Ambisonic Encoding with Rotating Source

```rust
use bbx_dsp::{channel::ChannelLayout, graph::GraphBuilder, waveform::Waveform};

let mut builder = GraphBuilder::<f32>::new(44100.0, 512, 4);

let osc = builder.add_oscillator(440.0, Waveform::Sine, None);
let encoder = builder.add_panner_ambisonic(1);

builder.connect(osc, 0, encoder, 0);

// Rotate source around listener
let az_lfo = builder.add_lfo(0.2, 1.0, None);
builder.modulate(az_lfo, encoder, "azimuth");

// Output channels: W, Y, Z, X (ACN order)
```

### Full Ambisonics Pipeline

```rust
use bbx_dsp::{channel::ChannelLayout, graph::GraphBuilder, waveform::Waveform};

let mut builder = GraphBuilder::<f32>::new(44100.0, 512, 2);

// Source
let osc = builder.add_oscillator(440.0, Waveform::Sine, None);

// Encode to FOA
let encoder = builder.add_panner_ambisonic(1);
builder.connect(osc, 0, encoder, 0);

// Decode to stereo for headphones
let decoder = builder.add_ambisonic_decoder(1, ChannelLayout::Stereo);
builder.connect(encoder, 0, decoder, 0);  // W
builder.connect(encoder, 1, decoder, 1);  // Y
builder.connect(encoder, 2, decoder, 2);  // Z
builder.connect(encoder, 3, decoder, 3);  // X
```

## Constant Power Panning

Stereo mode uses a constant-power law (sine/cosine):

```
Left gain  = cos(pan_angle)
Right gain = sin(pan_angle)
```

Where `pan_angle = (position + 100) / 200 * pi/2`

This maintains perceived loudness as the signal moves across the stereo field.

## Implementation Notes

- Click-free panning via linear smoothing on all parameters
- Uses `ChannelConfig::Explicit` (handles routing internally)
- All modes accept mono input (1 channel)
- VBAP normalizes gains for energy preservation
- Ambisonic encoding uses SN3D normalization and ACN channel ordering
