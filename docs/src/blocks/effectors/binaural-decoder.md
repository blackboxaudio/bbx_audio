# BinauralDecoderBlock

Decodes multi-channel audio to stereo for headphone listening using HRTF convolution.

## Overview

`BinauralDecoderBlock` converts ambisonic B-format or surround sound signals to binaural stereo output. Two decoding strategies are available:

- **HRTF (default)**: Full Head-Related Transfer Function convolution for accurate 3D spatial rendering with proper externalization
- **Matrix**: Lightweight ILD-based approximation for lower CPU usage

The HRTF strategy uses measured impulse responses from the MIT KEMAR database to model how sounds from different directions are filtered by the head and ears.

## Decoding Strategies

### BinauralStrategy::Hrtf (Default)

Uses time-domain convolution with Head-Related Impulse Responses (HRIRs) from virtual speaker positions. Provides accurate:
- **ITD** (Interaural Time Difference): Timing differences between ears
- **ILD** (Interaural Level Difference): Level differences from head shadowing
- **Spectral cues**: Frequency-dependent filtering from pinnae

Results in sounds that appear "outside the head" with convincing 3D positioning.

### BinauralStrategy::Matrix

Uses pre-computed psychoacoustic coefficients for basic left/right panning. Lower CPU but limited spatial accuracy—sounds may appear "inside the head".

## Creating a Decoder

```rust
use bbx_dsp::{blocks::BinauralDecoderBlock, graph::GraphBuilder};

let mut builder = GraphBuilder::<f32>::new(44100.0, 512, 2);

// Default: HRTF decoding for first-order ambisonics
let decoder = builder.add(BinauralDecoderBlock::new(1));

// Surround sound (5.1 or 7.1) with HRTF
let surround_decoder = builder.add(BinauralDecoderBlock::new_surround(6));
```

Or with explicit strategy selection:

```rust
use bbx_dsp::{
    blocks::{BinauralDecoderBlock, BinauralStrategy},
    graph::GraphBuilder,
};

let mut builder = GraphBuilder::<f32>::new(44100.0, 512, 2);

// HRTF decoding (default)
let hrtf_decoder = builder.add(BinauralDecoderBlock::new(1));

// Matrix decoding (lightweight)
let matrix_decoder = builder.add(BinauralDecoderBlock::with_strategy(2, BinauralStrategy::Matrix));

// Surround 5.1 with HRTF
let surround = builder.add(BinauralDecoderBlock::new_surround_with_strategy(6, BinauralStrategy::Hrtf));
```

## Supported Configurations

### Ambisonic Input

| Order | Channels | Format |
|-------|----------|--------|
| 1 (FOA) | 4 | W, Y, Z, X |
| 2 (SOA) | 9 | W, Y, Z, X, V, T, R, S, U |
| 3 (TOA) | 16 | Full third-order |

### Surround Input

| Layout | Channels | Description |
|--------|----------|-------------|
| 5.1 | 6 | L, R, C, LFE, Ls, Rs |
| 7.1 | 8 | L, R, C, LFE, Ls, Rs, Lrs, Rrs |

### Output

Always stereo (2 channels): Left ear, Right ear.

## Port Layout

| Port | Direction | Description |
|------|-----------|-------------|
| 0..N | Input | Multi-channel audio (4/6/8/9/16) |
| 0 | Output | Left ear |
| 1 | Output | Right ear |

Input count depends on configuration: `(order + 1)^2` for ambisonics, or 6/8 for surround.

## Usage Examples

### First-Order Ambisonics to Binaural

```rust
use bbx_dsp::{blocks::{BinauralDecoderBlock, OscillatorBlock, PannerBlock}, graph::GraphBuilder, waveform::Waveform};

let mut builder = GraphBuilder::<f32>::new(44100.0, 512, 2);

// Create ambisonic encoder
let encoder = builder.add(PannerBlock::new_ambisonic(1));
let osc = builder.add(OscillatorBlock::new(440.0, Waveform::Sine, None));
builder.connect(osc, 0, encoder, 0);

// Decode to binaural stereo (HRTF by default)
let decoder = builder.add(BinauralDecoderBlock::new(1));

// Connect all 4 FOA channels
builder.connect(encoder, 0, decoder, 0);  // W
builder.connect(encoder, 1, decoder, 1);  // Y
builder.connect(encoder, 2, decoder, 2);  // Z
builder.connect(encoder, 3, decoder, 3);  // X
```

### Rotating Sound with LFO Modulation

```rust
use bbx_dsp::{blocks::{BinauralDecoderBlock, LfoBlock, OscillatorBlock, PannerBlock}, graph::GraphBuilder, waveform::Waveform};

let mut builder = GraphBuilder::<f32>::new(44100.0, 512, 2);

// Source
let osc = builder.add(OscillatorBlock::new(440.0, Waveform::Sine, None));

// Encode to FOA with LFO-modulated azimuth
let encoder = builder.add(PannerBlock::new_ambisonic(1));
let lfo = builder.add(LfoBlock::new(0.5, 1.0, Waveform::Sine, None));
builder.connect(osc, 0, encoder, 0);
builder.modulate(lfo, encoder, "azimuth");

// Decode to headphones with HRTF
let decoder = builder.add(BinauralDecoderBlock::new(1));
builder.connect(encoder, 0, decoder, 0);
builder.connect(encoder, 1, decoder, 1);
builder.connect(encoder, 2, decoder, 2);
builder.connect(encoder, 3, decoder, 3);
```

### Surround Sound Downmix

```rust
use bbx_dsp::{blocks::{BinauralDecoderBlock, FileInputBlock}, graph::GraphBuilder};

let mut builder = GraphBuilder::<f32>::new(48000.0, 512, 2);

// File input with 5.1 surround content
let file = builder.add(FileInputBlock::new(Box::new(reader)));

// Decode to binaural for headphone listening
let decoder = builder.add(BinauralDecoderBlock::new_surround(6));

// Connect all 6 channels
for ch in 0..6 {
    builder.connect(file, ch, decoder, ch);
}
```

### Comparing Strategies

```rust
use bbx_dsp::{
    blocks::{BinauralDecoderBlock, BinauralStrategy},
    graph::GraphBuilder,
};

let mut builder = GraphBuilder::<f32>::new(44100.0, 512, 2);

// High-quality HRTF rendering
let hrtf = builder.add(BinauralDecoderBlock::new(1));

// Lightweight matrix rendering
let matrix = builder.add(BinauralDecoderBlock::with_strategy(1, BinauralStrategy::Matrix));
```

## HRTF Implementation Details

The HRTF strategy uses a virtual speaker approach:

1. **Virtual speaker layout**: 4 speakers at ±45° and ±135° azimuth for FOA
2. **Spherical harmonic decoding**: Input channels weighted by SH coefficients for each speaker position
3. **HRIR convolution**: Each speaker signal convolved with position-specific HRIRs
4. **Binaural summation**: All convolved outputs summed for left and right ears

### HRIR Data Source

HRIRs are from the MIT KEMAR database (Gardner & Martin, 1994):
- 256 samples per impulse response
- Measured on KEMAR mannequin
- Positions quantized to cardinal and 45° diagonal directions

For detailed mathematical background, see [HRTF Architecture](../../architecture/hrtf.md).

## Implementation Notes

- **Default strategy is HRTF** for best spatial quality
- Uses `ChannelConfig::Explicit` (handles routing internally)
- SN3D normalization and ACN channel ordering for ambisonics
- Pre-allocated circular buffers for realtime-safe convolution
- `reset()` clears convolution state (useful when seeking in playback)
- Panics if ambisonic order is not 1, 2, or 3
- Panics if surround channel count is not 6 or 8

## Performance Comparison

| Strategy | CPU Usage | Spatial Accuracy | Externalization |
|----------|-----------|------------------|-----------------|
| HRTF | Higher | Excellent | Yes |
| Matrix | Lower | Basic | Limited |

HRTF complexity: O(samples × speakers × hrir_length) per buffer.

## See Also

- [HRTF Architecture](../../architecture/hrtf.md) — Mathematical foundations and implementation details
- [AmbisonicDecoderBlock](ambisonic-decoder.md) — Decode to speaker arrays
- [PannerBlock](panner.md) — Ambisonic encoding
- [Multi-Channel System](../../architecture/multi-channel.md) — Channel layout architecture
