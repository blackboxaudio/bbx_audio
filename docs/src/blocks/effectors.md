# Effectors

Effector blocks process and transform audio signals.

## Available Effectors

| Block | Description |
|-------|-------------|
| [GainBlock](effectors/gain.md) | Level control in dB |
| [VcaBlock](effectors/vca.md) | Voltage controlled amplifier |
| [PannerBlock](effectors/panner.md) | Stereo, surround (VBAP), and ambisonic panning |
| [OverdriveBlock](effectors/overdrive.md) | Soft-clipping distortion |
| [DcBlockerBlock](effectors/dc-blocker.md) | DC offset removal |
| [ChannelRouterBlock](effectors/channel-router.md) | Simple stereo channel routing |
| [ChannelSplitterBlock](effectors/channel-splitter.md) | Split multi-channel to mono outputs |
| [ChannelMergerBlock](effectors/channel-merger.md) | Merge mono inputs to multi-channel |
| [MatrixMixerBlock](effectors/matrix-mixer.md) | NxM mixing matrix |
| [MixerBlock](effectors/mixer.md) | Channel-wise audio mixer |
| [AmbisonicDecoderBlock](effectors/ambisonic-decoder.md) | Ambisonics B-format decoder |
| [BinauralDecoderBlock](effectors/binaural-decoder.md) | B-format to stereo binaural |
| [LowPassFilterBlock](effectors/low-pass-filter.md) | SVF low-pass filter |

## Characteristics

Effectors have:
- **1+ inputs** - Audio to process
- **1+ outputs** - Processed audio
- **No modulation outputs** - They produce audio, not control

## Effect Chain Order

Order matters for sound quality:

```
Recommended order:
Source -> Gain (input level)
       -> Distortion
       -> DC Blocker
       -> Filter
       -> Panning
       -> Gain (output level)
```

## Usage Pattern

```rust
use bbx_dsp::{
    block::BlockType,
    blocks::{DcBlockerBlock, GainBlock},
    graph::GraphBuilder,
    waveform::Waveform,
};

let mut builder = GraphBuilder::<f32>::new(44100.0, 512, 2);

// Source
let osc = builder.add_oscillator(440.0, Waveform::Saw, None);

// Effect chain
let drive = builder.add_overdrive(3.0, 1.0, 0.8, 44100.0);
let dc = builder.add_block(BlockType::DcBlocker(DcBlockerBlock::new(true)));
let gain = builder.add_block(BlockType::Gain(GainBlock::new(-6.0, None)));

// Connect in series
builder.connect(osc, 0, drive, 0);
builder.connect(drive, 0, dc, 0);
builder.connect(dc, 0, gain, 0);
```

## Parallel Processing

Split signal to multiple effects:

```rust
use bbx_dsp::{
    block::BlockType,
    blocks::GainBlock,
    graph::GraphBuilder,
    waveform::Waveform,
};

let mut builder = GraphBuilder::<f32>::new(44100.0, 512, 2);

let source = builder.add_oscillator(440.0, Waveform::Saw, None);

// Dry path
let dry_gain = builder.add_block(BlockType::Gain(GainBlock::new(-6.0, None)));

// Wet path (distorted)
let wet_drive = builder.add_overdrive(5.0, 1.0, 0.5, 44100.0);

// Connect source to both
builder.connect(source, 0, dry_gain, 0);
builder.connect(source, 0, wet_drive, 0);

// Mix back together
let mixer = builder.add_block(BlockType::Gain(GainBlock::new(-3.0, None)));
builder.connect(dry_gain, 0, mixer, 0);
builder.connect(wet_drive, 0, mixer, 0);
```
