# Effectors

Effector blocks process and transform audio signals.

## Available Effectors

| Block | Description |
|-------|-------------|
| [GainBlock](effectors/gain.md) | Level control in dB |
| [PannerBlock](effectors/panner.md) | Stereo panning |
| [OverdriveBlock](effectors/overdrive.md) | Soft-clipping distortion |
| [DcBlockerBlock](effectors/dc-blocker.md) | DC offset removal |
| [ChannelRouterBlock](effectors/channel-router.md) | Channel routing |
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
       -> Filter (when available)
       -> Panning
       -> Gain (output level)
```

## Usage Pattern

```rust
use bbx_dsp::graph::GraphBuilder;

let mut builder = GraphBuilder::<f32>::new(44100.0, 512, 2);

// Source
let osc = builder.add_oscillator(440.0, Waveform::Saw, None);

// Effect chain
let drive = builder.add_overdrive(3.0, 1.0, 0.8, 44100.0);
let dc = builder.add_dc_blocker();
let gain = builder.add_gain(-6.0);

// Connect in series
builder.connect(osc, 0, drive, 0);
builder.connect(drive, 0, dc, 0);
builder.connect(dc, 0, gain, 0);
```

## Parallel Processing

Split signal to multiple effects:

```rust
let source = builder.add_oscillator(440.0, Waveform::Saw, None);

// Dry path
let dry_gain = builder.add_gain(-6.0);

// Wet path (distorted)
let wet_drive = builder.add_overdrive(5.0, 1.0, 0.5, 44100.0);

// Connect source to both
builder.connect(source, 0, dry_gain, 0);
builder.connect(source, 0, wet_drive, 0);

// Mix back together
let mixer = builder.add_gain(-3.0);
builder.connect(dry_gain, 0, mixer, 0);
builder.connect(wet_drive, 0, mixer, 0);
```
