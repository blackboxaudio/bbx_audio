# EnvelopeBlock

ADSR envelope generator for amplitude and parameter shaping.

## Overview

`EnvelopeBlock` generates attack-decay-sustain-release envelopes, typically used for controlling amplitude or filter cutoff over time.

## Creating an Envelope

```rust
use bbx_dsp::graph::GraphBuilder;

let mut builder = GraphBuilder::<f32>::new(44100.0, 512, 2);

let envelope = builder.add_envelope(
    0.01,   // Attack: 10ms
    0.1,    // Decay: 100ms
    0.7,    // Sustain: 70% level
    0.3,    // Release: 300ms
);
```

## Parameters

| Parameter | Type | Range | Default |
|-----------|------|-------|---------|
| Attack | f64 | 0.0 - 10.0 s | 0.01 |
| Decay | f64 | 0.0 - 10.0 s | 0.1 |
| Sustain | f64 | 0.0 - 1.0 | 0.5 |
| Release | f64 | 0.0 - 10.0 s | 0.3 |

### ADSR Stages

```
Level
  ^
  |    /\
  |   /  \______ Sustain
  |  /          \
  | /            \
  |/              \______
  +--A--D----S----R--> Time
```

- **Attack**: Time to reach maximum level
- **Decay**: Time to fall to sustain level
- **Sustain**: Level held while note is held
- **Release**: Time to fall to zero after note off

## Port Layout

| Port | Direction | Description |
|------|-----------|-------------|
| 0 | Modulation Output | Envelope signal (0.0 - 1.0) |

## Usage Examples

### Amplitude Envelope with VCA

The typical pattern for envelope-controlled amplitude uses a VCA:

```rust
use bbx_dsp::{graph::GraphBuilder, waveform::Waveform};

let mut builder = GraphBuilder::<f32>::new(44100.0, 512, 2);

let env = builder.add_envelope(0.01, 0.1, 0.7, 0.3);
let osc = builder.add_oscillator(440.0, Waveform::Sine, None);
let vca = builder.add_vca();

// Audio to VCA input 0, envelope to VCA input 1
builder.connect(osc, 0, vca, 0);
builder.connect(env, 0, vca, 1);
```

### Pluck Sound (Short Decay)

```rust
let env = builder.add_envelope(
    0.001,  // Very fast attack
    0.2,    // Medium decay
    0.0,    // No sustain
    0.1,    // Short release
);
```

### Pad Sound (Long Attack)

```rust
let env = builder.add_envelope(
    0.5,    // Slow attack
    0.3,    // Medium decay
    0.8,    // High sustain
    1.0,    // Long release
);
```

### Percussive (No Sustain)

```rust
let env = builder.add_envelope(
    0.002,  // Instant attack
    0.5,    // Decay only
    0.0,    // No sustain
    0.0,    // No release
);
```

## Envelope Shapes

### Linear

Constant rate of change:
```
Attack: level += rate_per_sample
```

### Exponential

More natural-sounding:
```
Attack: level *= (1.0 - coefficient)
```

## Triggering

Envelopes need a trigger signal:

- Note On → Start attack stage
- Note Off → Start release stage (if sustain > 0)

Trigger integration depends on your implementation (MIDI, gate signal, etc.).

## Output Range

- Output: 0.0 to 1.0 (unipolar)
- Starts at 0.0 (not triggered)
- Reaches 1.0 at end of attack
- Falls to sustain level during decay
- Returns to 0.0 during release
