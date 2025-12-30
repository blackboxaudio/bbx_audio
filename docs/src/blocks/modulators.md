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
| Rate | Sample rate | Per-block |
| Range | -1.0 to 1.0 | -1.0 to 1.0 |
| Purpose | Listening | Parameter control |

## Usage Pattern

```rust
use bbx_dsp::{graph::GraphBuilder, waveform::Waveform};

let mut builder = GraphBuilder::<f32>::new(44100.0, 512, 2);

// Create modulator
let lfo = builder.add_lfo(5.0, Waveform::Sine);

// Use it to modulate an oscillator
let osc = builder.add_oscillator(440.0, Waveform::Sine, Some(lfo));
```

## Modulation Routing

Modulators connect differently than audio blocks:

```rust
// Audio connection
builder.connect(audio_block, output_port, target_block, input_port);

// Modulation is specified at block creation
let osc = builder.add_oscillator(freq, waveform, Some(modulator_id));
let gain = builder.add_gain_with_modulation(level, Some(modulator_id));
```

## Combining Modulators

Layer multiple modulation sources:

```rust
// Slow sweep
let slow_lfo = builder.add_lfo(0.1, Waveform::Sine);

// Fast vibrato
let fast_lfo = builder.add_lfo(6.0, Waveform::Sine);

// Use for different parameters
let osc = builder.add_oscillator(440.0, Waveform::Sine, Some(fast_lfo));
let gain = builder.add_gain_with_modulation(-6.0, Some(slow_lfo));
```
