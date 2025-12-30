# GainBlock

Level control in decibels.

## Overview

`GainBlock` applies a gain (volume change) to audio signals, specified in decibels.

## Creating a Gain Block

```rust
use bbx_dsp::graph::GraphBuilder;

let mut builder = GraphBuilder::<f32>::new(44100.0, 512, 2);

let gain = builder.add_gain(-6.0);  // -6 dB (half amplitude)
```

## dB to Linear Conversion

| dB | Linear | Effect |
|----|--------|--------|
| +12 | 4.0 | 4x louder |
| +6 | 2.0 | 2x louder |
| +3 | 1.41 | ~1.4x louder |
| 0 | 1.0 | No change |
| -3 | 0.71 | ~0.7x amplitude |
| -6 | 0.5 | Half amplitude |
| -12 | 0.25 | Quarter amplitude |
| -20 | 0.1 | 10% amplitude |
| -inf | 0.0 | Silence |

## Port Layout

| Port | Direction | Description |
|------|-----------|-------------|
| 0 | Input | Audio input |
| 0 | Output | Gained audio |

## Parameters

| Parameter | Type | Range | Default |
|-----------|------|-------|---------|
| Level | f64 | -inf to +inf dB | 0.0 |

## Usage Examples

### Volume Control

```rust
let source = builder.add_oscillator(440.0, Waveform::Sine, None);
let volume = builder.add_gain(-12.0);  // -12 dB

builder.connect(source, 0, volume, 0);
```

### Gain Staging

```rust
// Input gain before processing
let input_gain = builder.add_gain(6.0);

// Processing
let effect = builder.add_overdrive(3.0, 1.0, 1.0, 44100.0);

// Output gain after processing
let output_gain = builder.add_gain(-6.0);

builder.connect(source, 0, input_gain, 0);
builder.connect(input_gain, 0, effect, 0);
builder.connect(effect, 0, output_gain, 0);
```

### Mixing (Summing)

Multiple inputs are summed:

```rust
let osc1 = builder.add_oscillator(261.63, Waveform::Sine, None);
let osc2 = builder.add_oscillator(329.63, Waveform::Sine, None);
let osc3 = builder.add_oscillator(392.00, Waveform::Sine, None);

// Mix with headroom
let mixer = builder.add_gain(-9.0);  // -9 dB for 3 sources

builder.connect(osc1, 0, mixer, 0);
builder.connect(osc2, 0, mixer, 0);
builder.connect(osc3, 0, mixer, 0);
```

## With Modulation

```rust
// LFO for tremolo
let lfo = builder.add_lfo(6.0, Waveform::Sine);

// Gain with amplitude modulation
let gain = builder.add_gain_with_modulation(-6.0, Some(lfo));
```

## Implementation Notes

- Linear multiplication (no interpolation)
- Instant parameter changes (no smoothing)
- Handles all channel counts
