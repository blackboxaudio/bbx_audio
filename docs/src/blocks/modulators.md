# Modulators

Modulator blocks generate control signals for parameter modulation.

## Available Modulators

| Block | Description |
|-------|-------------|
| [LfoBlock](modulators/lfo.md) | Low-frequency oscillator |
| [EnvelopeBlock](modulators/envelope.md) | ADSR envelope |

## Characteristics

Modulators have:
- **0 audio inputs** - They generate control signals
- **0 audio outputs** - Output is via modulation system
- **1+ modulation outputs** - Control signal outputs

## Modulation vs Audio

| Aspect | Audio | Modulation |
|--------|-------|------------|
| Rate | Sample rate | Per-block (control rate) |
| Range | -1.0 to 1.0 | -1.0 to 1.0 |
| Purpose | Listening | Parameter control |

## Usage Pattern

```rust
use bbx_dsp::{blocks::{LfoBlock, OscillatorBlock}, graph::GraphBuilder, waveform::Waveform};

let mut builder = GraphBuilder::<f32>::new(44100.0, 512, 2);

// Create modulator (frequency, depth, waveform, seed)
let lfo = builder.add(LfoBlock::new(5.0, 0.5, Waveform::Sine, None));

// Create oscillator
let osc = builder.add(OscillatorBlock::new(440.0, Waveform::Sine, None));

// Connect modulation using the modulate() method
builder.modulate(lfo, osc, "frequency");
```

## Modulation Routing

Modulators connect using the `modulate()` builder method:

```rust
use bbx_dsp::{
    blocks::{GainBlock, LfoBlock, OscillatorBlock},
    graph::GraphBuilder,
    waveform::Waveform,
};

let mut builder = GraphBuilder::<f32>::new(44100.0, 512, 2);

// Create LFO
let lfo = builder.add(LfoBlock::new(6.0, 1.0, Waveform::Sine, None));

// Create oscillator and gain
let osc = builder.add(OscillatorBlock::new(440.0, Waveform::Sine, None));
let gain = builder.add(GainBlock::new(-6.0, None));

// Audio connection
builder.connect(osc, 0, gain, 0);

// Modulation connection (source, target, parameter_name)
builder.modulate(lfo, gain, "level");
```

## Combining Modulators

Layer multiple modulation sources:

```rust
use bbx_dsp::{
    blocks::{GainBlock, LfoBlock, OscillatorBlock},
    graph::GraphBuilder,
    waveform::Waveform,
};

let mut builder = GraphBuilder::<f32>::new(44100.0, 512, 2);

// Slow sweep for amplitude
let slow_lfo = builder.add(LfoBlock::new(0.1, 0.5, Waveform::Sine, None));

// Fast vibrato for pitch
let fast_lfo = builder.add(LfoBlock::new(6.0, 0.3, Waveform::Sine, None));

// Oscillator with vibrato
let osc = builder.add(OscillatorBlock::new(440.0, Waveform::Sine, None));
builder.modulate(fast_lfo, osc, "frequency");

// Gain with tremolo
let gain = builder.add(GainBlock::new(-6.0, None));
builder.connect(osc, 0, gain, 0);
builder.modulate(slow_lfo, gain, "level");
```

## Modulatable Parameters

Common parameters that can be modulated:

| Block | Parameter | Effect |
|-------|-----------|--------|
| Oscillator | "frequency" | Vibrato |
| Oscillator | "pitch_offset" | Pitch shift |
| Gain | "level" | Tremolo |
| Panner | "position" | Auto-pan |
| LowPassFilter | "cutoff" | Filter sweep |
| Overdrive | "drive" | Drive intensity |
