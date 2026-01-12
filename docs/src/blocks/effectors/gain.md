# GainBlock

Level control in decibels.

## Overview

`GainBlock` applies a gain (volume change) to audio signals, specified in decibels.

## Creating a Gain Block

```rust
use bbx_dsp::{
    block::BlockType,
    blocks::GainBlock,
    graph::GraphBuilder,
};

let mut builder = GraphBuilder::<f32>::new(44100.0, 512, 2);

let gain = builder.add_block(BlockType::Gain(GainBlock::new(-6.0, None)));  // -6 dB
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
| -80 | ~0.0 | Near silence |

## Port Layout

| Port | Direction | Description |
|------|-----------|-------------|
| 0 | Input | Audio input |
| 0 | Output | Gained audio |

## Parameters

| Parameter | Type | Range | Default |
|-----------|------|-------|---------|
| level_db | f64 | -80 to +30 dB | 0.0 |

## Usage Examples

### Volume Control

```rust
use bbx_dsp::{
    block::BlockType,
    blocks::GainBlock,
    graph::GraphBuilder,
    waveform::Waveform,
};

let mut builder = GraphBuilder::<f32>::new(44100.0, 512, 2);

let source = builder.add_oscillator(440.0, Waveform::Sine, None);
let volume = builder.add_block(BlockType::Gain(GainBlock::new(-12.0, None)));  // -12 dB

builder.connect(source, 0, volume, 0);
```

### Gain Staging

```rust
use bbx_dsp::{
    block::BlockType,
    blocks::GainBlock,
    graph::GraphBuilder,
    waveform::Waveform,
};

let mut builder = GraphBuilder::<f32>::new(44100.0, 512, 2);

let source = builder.add_oscillator(440.0, Waveform::Sine, None);

// Input gain before processing
let input_gain = builder.add_block(BlockType::Gain(GainBlock::new(6.0, None)));

// Processing
let effect = builder.add_overdrive(3.0, 1.0, 1.0, 44100.0);

// Output gain after processing
let output_gain = builder.add_block(BlockType::Gain(GainBlock::new(-6.0, None)));

builder.connect(source, 0, input_gain, 0);
builder.connect(input_gain, 0, effect, 0);
builder.connect(effect, 0, output_gain, 0);
```

### Mixing (Summing)

Multiple inputs are summed:

```rust
use bbx_dsp::{
    block::BlockType,
    blocks::GainBlock,
    graph::GraphBuilder,
    waveform::Waveform,
};

let mut builder = GraphBuilder::<f32>::new(44100.0, 512, 2);

let osc1 = builder.add_oscillator(261.63, Waveform::Sine, None);
let osc2 = builder.add_oscillator(329.63, Waveform::Sine, None);
let osc3 = builder.add_oscillator(392.00, Waveform::Sine, None);

// Mix with headroom
let mixer = builder.add_block(BlockType::Gain(GainBlock::new(-9.0, None)));

builder.connect(osc1, 0, mixer, 0);
builder.connect(osc2, 0, mixer, 0);
builder.connect(osc3, 0, mixer, 0);
```

## With Modulation

```rust
use bbx_dsp::{
    block::BlockType,
    blocks::GainBlock,
    graph::GraphBuilder,
    waveform::Waveform,
};

let mut builder = GraphBuilder::<f32>::new(44100.0, 512, 2);

let osc = builder.add_oscillator(440.0, Waveform::Sine, None);

// LFO for tremolo (6 Hz, full depth)
let lfo = builder.add_lfo(6.0, 1.0, None);

// Gain block
let gain = builder.add_block(BlockType::Gain(GainBlock::new(-6.0, None)));

builder.connect(osc, 0, gain, 0);

// Modulate gain level with LFO
builder.modulate(lfo, gain, "level");
```

## Implementation Notes

- Click-free gain changes via linear smoothing
- Handles all channel counts
- Range clamped to -80 to +30 dB
