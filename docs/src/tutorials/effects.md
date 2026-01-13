# Adding Effects

This tutorial covers the effect blocks available in bbx_audio.

> **Prior knowledge**: This tutorial assumes familiarity with:
> - [Your First DSP Graph](first-graph.md) - GraphBuilder and connections
> - [Creating a Simple Oscillator](oscillator.md) - Adding audio sources

## Available Effects

- `GainBlock` - Level control in dB
- `PannerBlock` - Stereo panning
- `OverdriveBlock` - Soft-clipping distortion
- `DcBlockerBlock` - DC offset removal
- `ChannelRouterBlock` - Channel routing/manipulation
- `ChannelSplitterBlock` - Split multi-channel to individual outputs
- `ChannelMergerBlock` - Merge individual inputs to multi-channel
- `LowPassFilterBlock` - SVF-based low-pass filter

## GainBlock

Control signal level in decibels:

```rust
use bbx_dsp::{
    blocks::{GainBlock, OscillatorBlock},
    graph::GraphBuilder,
    waveform::Waveform,
};

let mut builder = GraphBuilder::<f32>::new(44100.0, 512, 2);

let osc = builder.add(OscillatorBlock::new(440.0, Waveform::Sine, None));
let gain = builder.add(GainBlock::new(-6.0, None));  // -6 dB

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
    blocks::{OscillatorBlock, PannerBlock},
    graph::GraphBuilder,
    waveform::Waveform,
};

let mut builder = GraphBuilder::<f32>::new(44100.0, 512, 2);

let osc = builder.add(OscillatorBlock::new(440.0, Waveform::Sine, None));
let pan = builder.add(PannerBlock::new(0.0));  // Center

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
use bbx_dsp::{
    blocks::{OscillatorBlock, OverdriveBlock},
    graph::GraphBuilder,
    waveform::Waveform,
};

let mut builder = GraphBuilder::<f32>::new(44100.0, 512, 2);

let osc = builder.add(OscillatorBlock::new(440.0, Waveform::Sine, None));

// Overdrive with drive, level, tone, sample_rate
let overdrive = builder.add(OverdriveBlock::new(
    5.0,      // Drive amount (higher = more distortion)
    1.0,      // Output level
    0.7,      // Tone (0.0-1.0, higher = brighter)
    44100.0   // Sample rate
));

builder.connect(osc, 0, overdrive, 0);

let graph = builder.build();
```

## DcBlockerBlock

Remove DC offset from signals:

```rust
use bbx_dsp::{
    blocks::{DcBlockerBlock, OscillatorBlock},
    graph::GraphBuilder,
    waveform::Waveform,
};

let mut builder = GraphBuilder::<f32>::new(44100.0, 512, 2);

let osc = builder.add(OscillatorBlock::new(440.0, Waveform::Sine, None));
let dc_blocker = builder.add(DcBlockerBlock::new(true));

builder.connect(osc, 0, dc_blocker, 0);

let graph = builder.build();
```

Use DC blockers after:
- Distortion effects
- Asymmetric waveforms
- External audio input

## LowPassFilterBlock

SVF-based low-pass filter with resonance control:

```rust
use bbx_dsp::{
    blocks::{LowPassFilterBlock, OscillatorBlock},
    graph::GraphBuilder,
    waveform::Waveform,
};

let mut builder = GraphBuilder::<f32>::new(44100.0, 512, 2);

let osc = builder.add(OscillatorBlock::new(110.0, Waveform::Sawtooth, None));

// Low-pass filter with cutoff and resonance (Q)
let filter = builder.add(LowPassFilterBlock::new(
    800.0,   // Cutoff frequency in Hz
    2.0      // Resonance (Q factor, 0.5-10.0)
));

builder.connect(osc, 0, filter, 0);

let graph = builder.build();
```

The filter cutoff can be modulated by an LFO for classic wah/sweep effects. See the `10_filter_modulation` example.

## ChannelSplitterBlock

Split multi-channel audio into individual mono outputs:

```rust
use bbx_dsp::{blocks::ChannelSplitterBlock, graph::GraphBuilder};

let mut builder = GraphBuilder::<f32>::new(44100.0, 512, 2);

// Split stereo into two mono outputs
let splitter = builder.add(ChannelSplitterBlock::new(2));

// splitter output 0 = left channel
// splitter output 1 = right channel
```

## ChannelMergerBlock

Merge individual mono inputs into multi-channel output:

```rust
use bbx_dsp::{blocks::ChannelMergerBlock, graph::GraphBuilder};

let mut builder = GraphBuilder::<f32>::new(44100.0, 512, 2);

// Merge two mono inputs into stereo
let merger = builder.add(ChannelMergerBlock::new(2));

// merger input 0 = left channel
// merger input 1 = right channel
```

### Parallel Processing with Split/Merge

Process channels independently then recombine:

```rust
use bbx_dsp::{
    blocks::{
        ChannelMergerBlock, ChannelSplitterBlock, LowPassFilterBlock,
        OscillatorBlock, PannerBlock,
    },
    graph::GraphBuilder,
    waveform::Waveform,
};

let mut builder = GraphBuilder::<f32>::new(44100.0, 512, 2);

let osc = builder.add(OscillatorBlock::new(110.0, Waveform::Sawtooth, None));
let panner = builder.add(PannerBlock::new(0.0));

// Split stereo signal
let splitter = builder.add(ChannelSplitterBlock::new(2));

// Different filters for each channel
let filter_left = builder.add(LowPassFilterBlock::new(500.0, 2.0));   // Darker
let filter_right = builder.add(LowPassFilterBlock::new(2000.0, 2.0)); // Brighter

// Merge back to stereo
let merger = builder.add(ChannelMergerBlock::new(2));

// Build chain
builder.connect(osc, 0, panner, 0);
builder.connect(panner, 0, splitter, 0);
builder.connect(panner, 1, splitter, 1);
builder.connect(splitter, 0, filter_left, 0);
builder.connect(splitter, 1, filter_right, 0);
builder.connect(filter_left, 0, merger, 0);
builder.connect(filter_right, 0, merger, 1);

let graph = builder.build();
```

See the `11_channel_split_merge` example for a complete demonstration.

## Building Effect Chains

Chain multiple effects together using the same connection pattern from [Your First DSP Graph](first-graph.md#connecting-blocks):

```rust
use bbx_dsp::{
    blocks::{DcBlockerBlock, GainBlock, OscillatorBlock, OverdriveBlock, PannerBlock},
    graph::GraphBuilder,
    waveform::Waveform,
};

let mut builder = GraphBuilder::<f32>::new(44100.0, 512, 2);

// Source
let osc = builder.add(OscillatorBlock::new(440.0, Waveform::Saw, None));

// Effect chain
let overdrive = builder.add(OverdriveBlock::new(3.0, 1.0, 0.8, 44100.0));
let dc_blocker = builder.add(DcBlockerBlock::new(true));
let gain = builder.add(GainBlock::new(-6.0, None));
let pan = builder.add(PannerBlock::new(0.0));

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
    blocks::{GainBlock, OscillatorBlock, OverdriveBlock},
    graph::GraphBuilder,
    waveform::Waveform,
};

let mut builder = GraphBuilder::<f32>::new(44100.0, 512, 2);

let osc = builder.add(OscillatorBlock::new(440.0, Waveform::Saw, None));

// Two parallel paths
let clean_gain = builder.add(GainBlock::new(-6.0, None));
let dirty_overdrive = builder.add(OverdriveBlock::new(8.0, 1.0, 0.5, 44100.0));

// Mix them back together
let mix = builder.add(GainBlock::new(-3.0, None));

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
