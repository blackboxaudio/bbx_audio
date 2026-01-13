# Generators

Generator blocks create audio signals from nothing.

## Available Generators

| Block | Description |
|-------|-------------|
| [OscillatorBlock](generators/oscillator.md) | Waveform generator |

## Characteristics

Generators have:
- **0 inputs** - They create signal, not process it
- **1+ outputs** - Audio signal output
- **No modulation outputs** - They produce audio, not control signals

## Usage Pattern

```rust
use bbx_dsp::{
    blocks::{GainBlock, OscillatorBlock},
    graph::GraphBuilder,
    waveform::Waveform,
};

let mut builder = GraphBuilder::<f32>::new(44100.0, 512, 2);

// Add a generator
let osc = builder.add(OscillatorBlock::new(440.0, Waveform::Sine, None));

// Connect to effects or output
let gain = builder.add(GainBlock::new(-6.0, None));
builder.connect(osc, 0, gain, 0);
```

## Polyphony

Create multiple generators for polyphonic sounds:

```rust
use bbx_dsp::{
    blocks::{GainBlock, OscillatorBlock},
    graph::GraphBuilder,
    waveform::Waveform,
};

let mut builder = GraphBuilder::<f32>::new(44100.0, 512, 2);

// Three oscillators for a chord
let c4 = builder.add(OscillatorBlock::new(261.63, Waveform::Sine, None));
let e4 = builder.add(OscillatorBlock::new(329.63, Waveform::Sine, None));
let g4 = builder.add(OscillatorBlock::new(392.00, Waveform::Sine, None));

// Mix them
let mixer = builder.add(GainBlock::new(-9.0, None));
builder.connect(c4, 0, mixer, 0);
builder.connect(e4, 0, mixer, 0);
builder.connect(g4, 0, mixer, 0);
```

## Future Generators

Potential additions:
- SamplerBlock - Sample playback
- WavetableBlock - Wavetable synthesis
- NoiseBlock - Dedicated noise generator
- GranularBlock - Granular synthesis
