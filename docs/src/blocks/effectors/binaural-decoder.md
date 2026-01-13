# BinauralDecoderBlock

Decodes ambisonics B-format to stereo for headphone listening.

## Overview

`BinauralDecoderBlock` converts SN3D normalized, ACN ordered ambisonic signals to stereo headphone output using psychoacoustically-informed matrix coefficients. It approximates binaural cues (primarily ILD - Interaural Level Difference) without full HRTF convolution, providing a lightweight, realtime-safe decoder for basic spatial audio reproduction on headphones.

## Creating a Decoder

```rust
use bbx_dsp::graph::GraphBuilder;

let mut builder = GraphBuilder::<f32>::new(44100.0, 512, 2);

// Decode first-order ambisonics to stereo binaural
let decoder = builder.add_binaural_decoder(1);
```

Or with direct construction:

```rust
use bbx_dsp::{
    block::BlockType,
    blocks::BinauralDecoderBlock,
    graph::GraphBuilder,
};

let mut builder = GraphBuilder::<f32>::new(44100.0, 512, 2);

let decoder = builder.add_block(BlockType::BinauralDecoder(
    BinauralDecoderBlock::new(2)
));
```

## Supported Configurations

### Input Orders

| Order | Channels | Format |
|-------|----------|--------|
| 1 (FOA) | 4 | W, Y, Z, X |
| 2 (SOA) | 9 | W, Y, Z, X, V, T, R, S, U |
| 3 (TOA) | 16 | Full third-order |

### Output

| Layout | Channels | Description |
|--------|----------|-------------|
| Stereo | 2 | Left, Right (headphone output) |

Output is always stereo regardless of input order.

## Port Layout

| Port | Direction | Description |
|------|-----------|-------------|
| 0..N | Input | Ambisonic channels (4/9/16) |
| 0 | Output | Left ear |
| 1 | Output | Right ear |

Input count depends on order: `(order + 1)^2`

## Usage Examples

### First-Order to Binaural

```rust
use bbx_dsp::{graph::GraphBuilder, waveform::Waveform};

let mut builder = GraphBuilder::<f32>::new(44100.0, 512, 2);

// Create ambisonic encoder
let encoder = builder.add_panner_ambisonic(1);
let osc = builder.add_oscillator(440.0, Waveform::Sine, None);
builder.connect(osc, 0, encoder, 0);

// Decode to binaural stereo
let decoder = builder.add_binaural_decoder(1);

// Connect all 4 FOA channels
builder.connect(encoder, 0, decoder, 0);  // W
builder.connect(encoder, 1, decoder, 1);  // Y
builder.connect(encoder, 2, decoder, 2);  // Z
builder.connect(encoder, 3, decoder, 3);  // X
```

### Full Ambisonics Pipeline with Modulation

```rust
use bbx_dsp::{graph::GraphBuilder, waveform::Waveform};

let mut builder = GraphBuilder::<f32>::new(44100.0, 512, 2);

// Source
let osc = builder.add_oscillator(440.0, Waveform::Sine, None);

// Encode to FOA with LFO-modulated azimuth
let encoder = builder.add_panner_ambisonic(1);
let lfo = builder.add_lfo(0.5, 1.0, None);
builder.connect(osc, 0, encoder, 0);
builder.modulate(lfo, encoder, "azimuth");

// Decode to headphones
let decoder = builder.add_binaural_decoder(1);
builder.connect(encoder, 0, decoder, 0);
builder.connect(encoder, 1, decoder, 1);
builder.connect(encoder, 2, decoder, 2);
builder.connect(encoder, 3, decoder, 3);
```

## Implementation Notes

- Uses ILD-based psychoacoustic weighting (not HRTF convolution)
- Matrix coefficients are pre-computed at construction time
- SN3D normalization and ACN channel ordering
- Uses `ChannelConfig::Explicit` (handles routing internally)
- Panics if order is not 1, 2, or 3
- Suitable for basic spatial audio; for high-fidelity binaural reproduction, consider external HRTF convolution
