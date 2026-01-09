# Adding Effects

This tutorial covers the effect blocks available in bbx_audio.

## Available Effects

- `GainBlock` - Level control in dB
- `PannerBlock` - Stereo panning
- `OverdriveBlock` - Soft-clipping distortion
- `DcBlockerBlock` - DC offset removal
- `ChannelRouterBlock` - Channel routing/manipulation
- `LowPassFilterBlock` - SVF-based low-pass filter

## GainBlock

Control signal level in decibels:

```rust
use bbx_dsp::{
    block::BlockType,
    blocks::GainBlock,
    graph::GraphBuilder,
    waveform::Waveform,
};

let mut builder = GraphBuilder::<f32>::new(44100.0, 512, 2);

let osc = builder.add_oscillator(440.0, Waveform::Sine, None);
let gain = builder.add_block(BlockType::Gain(GainBlock::new(-6.0)));  // -6 dB

builder.connect(osc, 0, gain, 0);

let graph = builder.build();
```

Common gain values:
- `0.0` dB = unity (no change)
- `-6.0` dB = half amplitude
- `-12.0` dB = quarter amplitude
- `+6.0` dB = double amplitude (watch for clipping!)

## PannerBlock

Position audio in the stereo field:

```rust
use bbx_dsp::{
    block::BlockType,
    blocks::PannerBlock,
    graph::GraphBuilder,
    waveform::Waveform,
};

let mut builder = GraphBuilder::<f32>::new(44100.0, 512, 2);

let osc = builder.add_oscillator(440.0, Waveform::Sine, None);
let pan = builder.add_block(BlockType::Panner(PannerBlock::new(0.0)));  // Center

builder.connect(osc, 0, pan, 0);

let graph = builder.build();
```

Pan values:
- `-100.0` = Hard left
- `0.0` = Center
- `+100.0` = Hard right

The panner uses constant-power panning for natural-sounding transitions.

## OverdriveBlock

Soft-clipping distortion for warmth and saturation:

```rust
use bbx_dsp::{graph::GraphBuilder, waveform::Waveform};

let mut builder = GraphBuilder::<f32>::new(44100.0, 512, 2);

let osc = builder.add_oscillator(440.0, Waveform::Sine, None);

// Overdrive with drive, level, tone, sample_rate
let overdrive = builder.add_overdrive(
    5.0,      // Drive amount (higher = more distortion)
    1.0,      // Output level
    0.7,      // Tone (0.0-1.0, higher = brighter)
    44100.0   // Sample rate
);

builder.connect(osc, 0, overdrive, 0);

let graph = builder.build();
```

## DcBlockerBlock

Remove DC offset from signals:

```rust
use bbx_dsp::{
    block::BlockType,
    blocks::DcBlockerBlock,
    graph::GraphBuilder,
    waveform::Waveform,
};

let mut builder = GraphBuilder::<f32>::new(44100.0, 512, 2);

let osc = builder.add_oscillator(440.0, Waveform::Sine, None);
let dc_blocker = builder.add_block(BlockType::DcBlocker(DcBlockerBlock::new(true)));

builder.connect(osc, 0, dc_blocker, 0);

let graph = builder.build();
```

Use DC blockers after:
- Distortion effects
- Asymmetric waveforms
- External audio input

## Building Effect Chains

Chain multiple effects together:

```rust
use bbx_dsp::{
    block::BlockType,
    blocks::{DcBlockerBlock, GainBlock, PannerBlock},
    graph::GraphBuilder,
    waveform::Waveform,
};

let mut builder = GraphBuilder::<f32>::new(44100.0, 512, 2);

// Source
let osc = builder.add_oscillator(440.0, Waveform::Saw, None);

// Effect chain
let overdrive = builder.add_overdrive(3.0, 1.0, 0.8, 44100.0);
let dc_blocker = builder.add_block(BlockType::DcBlocker(DcBlockerBlock::new(true)));
let gain = builder.add_block(BlockType::Gain(GainBlock::new(-6.0)));
let pan = builder.add_block(BlockType::Panner(PannerBlock::new(0.0)));

// Connect: Osc -> Overdrive -> DC Blocker -> Gain -> Pan
builder.connect(osc, 0, overdrive, 0);
builder.connect(overdrive, 0, dc_blocker, 0);
builder.connect(dc_blocker, 0, gain, 0);
builder.connect(gain, 0, pan, 0);

let graph = builder.build();
```

## Parallel Effects

Route signals to multiple effects in parallel:

```rust
use bbx_dsp::{
    block::BlockType,
    blocks::GainBlock,
    graph::GraphBuilder,
    waveform::Waveform,
};

let mut builder = GraphBuilder::<f32>::new(44100.0, 512, 2);

let osc = builder.add_oscillator(440.0, Waveform::Saw, None);

// Two parallel paths
let clean_gain = builder.add_block(BlockType::Gain(GainBlock::new(-6.0)));
let dirty_overdrive = builder.add_overdrive(8.0, 1.0, 0.5, 44100.0);

// Mix them back together
let mix = builder.add_block(BlockType::Gain(GainBlock::new(-3.0)));

// Split to both paths
builder.connect(osc, 0, clean_gain, 0);
builder.connect(osc, 0, dirty_overdrive, 0);

// Merge at mix
builder.connect(clean_gain, 0, mix, 0);
builder.connect(dirty_overdrive, 0, mix, 0);

let graph = builder.build();
```

## Effect Order Matters

The order of effects changes the sound:

```
Osc -> Overdrive -> Gain    // Distort first, then control level
Osc -> Gain -> Overdrive    // Control level into distortion (changes character)
```

General guidelines:
1. **Gain staging** - Control levels before distortion
2. **Distortion** - Apply saturation
3. **DC blocking** - Remove offset after distortion
4. **EQ/Filtering** - Shape the tone
5. **Panning** - Position in stereo field
6. **Final gain** - Set output level

## Next Steps

- [Parameter Modulation](modulation.md) - Animate effect parameters
- [Working with Audio Files](audio-files.md) - Process real audio
