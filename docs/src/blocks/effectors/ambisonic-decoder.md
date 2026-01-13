# AmbisonicDecoderBlock

Decodes ambisonics B-format to speaker layouts.

## Overview

`AmbisonicDecoderBlock` converts SN3D normalized, ACN ordered ambisonic signals to discrete speaker feeds using a mode-matching decoder. This enables playback of ambisonic content on standard speaker configurations.

## Creating a Decoder

```rust
use bbx_dsp::{blocks::AmbisonicDecoderBlock, channel::ChannelLayout, graph::GraphBuilder};

let mut builder = GraphBuilder::<f32>::new(44100.0, 512, 2);

// Decode first-order ambisonics to stereo
let decoder = builder.add(AmbisonicDecoderBlock::new(1, ChannelLayout::Stereo));
```

Or for surround layouts:

```rust
use bbx_dsp::{
    blocks::AmbisonicDecoderBlock,
    channel::ChannelLayout,
    graph::GraphBuilder,
};

let mut builder = GraphBuilder::<f32>::new(44100.0, 512, 6);

let decoder = builder.add(AmbisonicDecoderBlock::new(2, ChannelLayout::Surround51));
```

## Supported Configurations

### Input Orders

| Order | Channels | Format |
|-------|----------|--------|
| 1 (FOA) | 4 | W, Y, Z, X |
| 2 (SOA) | 9 | W, Y, Z, X, V, T, R, S, U |
| 3 (TOA) | 16 | Full third-order |

### Output Layouts

| Layout | Channels | Description |
|--------|----------|-------------|
| Mono | 1 | Single speaker |
| Stereo | 2 | L, R at +/-30 degrees |
| Surround51 | 6 | L, R, C, LFE, Ls, Rs |
| Surround71 | 8 | L, R, C, LFE, Ls, Rs, Lrs, Rrs |
| Custom(n) | n | Circular array |

## Port Layout

| Port | Direction | Description |
|------|-----------|-------------|
| 0..N | Input | Ambisonic channels (4/9/16) |
| 0..M | Output | Speaker feeds |

Input count depends on order: `(order + 1)^2`
Output count depends on the target layout.

## Speaker Positions

### Stereo

| Channel | Azimuth |
|---------|---------|
| Left | +30 degrees |
| Right | -30 degrees |

### 5.1 Surround

| Channel | Azimuth |
|---------|---------|
| L | +30 degrees |
| R | -30 degrees |
| C | 0 degrees |
| LFE | N/A (omnidirectional) |
| Ls | +110 degrees |
| Rs | -110 degrees |

### 7.1 Surround

| Channel | Azimuth |
|---------|---------|
| L | +30 degrees |
| R | -30 degrees |
| C | 0 degrees |
| LFE | N/A |
| Ls | +90 degrees |
| Rs | -90 degrees |
| Lrs | +150 degrees |
| Rrs | -150 degrees |

### Custom Layout

For `Custom(n)`, speakers are distributed evenly in a circle:
- Angle step: 360 degrees / n
- Starting offset: -90 degrees

## Usage Examples

### First-Order to Stereo

```rust
use bbx_dsp::{blocks::{AmbisonicDecoderBlock, OscillatorBlock, PannerBlock}, channel::ChannelLayout, graph::GraphBuilder, waveform::Waveform};

let mut builder = GraphBuilder::<f32>::new(44100.0, 512, 2);

// Create ambisonic encoder
let encoder = builder.add(PannerBlock::new_ambisonic(1));
let osc = builder.add(OscillatorBlock::new(440.0, Waveform::Sine, None));
builder.connect(osc, 0, encoder, 0);

// Decode to stereo
let decoder = builder.add(AmbisonicDecoderBlock::new(1, ChannelLayout::Stereo));

// Connect all 4 FOA channels
builder.connect(encoder, 0, decoder, 0);  // W
builder.connect(encoder, 1, decoder, 1);  // Y
builder.connect(encoder, 2, decoder, 2);  // Z
builder.connect(encoder, 3, decoder, 3);  // X
```

### Second-Order to 5.1

```rust
use bbx_dsp::{blocks::AmbisonicDecoderBlock, channel::ChannelLayout, graph::GraphBuilder};

let mut builder = GraphBuilder::<f32>::new(44100.0, 512, 6);

// Decode SOA (9 channels) to 5.1 (6 channels)
let decoder = builder.add(AmbisonicDecoderBlock::new(2, ChannelLayout::Surround51));
```

### Full Ambisonics Pipeline

```rust
use bbx_dsp::{blocks::{AmbisonicDecoderBlock, LfoBlock, OscillatorBlock, PannerBlock}, channel::ChannelLayout, graph::GraphBuilder, waveform::Waveform};

let mut builder = GraphBuilder::<f32>::new(44100.0, 512, 8);

// Source
let osc = builder.add(OscillatorBlock::new(440.0, Waveform::Sine, None));

// Encode to FOA with LFO-modulated azimuth
let encoder = builder.add(PannerBlock::new_ambisonic(1));
let lfo = builder.add(LfoBlock::new(0.5, 1.0, Waveform::Sine, None));
builder.connect(osc, 0, encoder, 0);
builder.modulate(lfo, encoder, "azimuth");

// Decode to 7.1 speakers
let decoder = builder.add(AmbisonicDecoderBlock::new(1, ChannelLayout::Surround71));
builder.connect(encoder, 0, decoder, 0);
builder.connect(encoder, 1, decoder, 1);
builder.connect(encoder, 2, decoder, 2);
builder.connect(encoder, 3, decoder, 3);
```

## Implementation Notes

- Uses mode-matching decoder with energy normalization
- SN3D normalization and ACN channel ordering
- Uses `ChannelConfig::Explicit` (handles routing internally)
- Panics if order is not 1, 2, or 3
- LFE channel receives minimal directional content in surround layouts
