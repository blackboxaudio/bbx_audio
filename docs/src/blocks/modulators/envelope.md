# EnvelopeBlock

ADSR envelope generator for amplitude and parameter modulation.

## Overview

`EnvelopeBlock` generates time-varying control signals using the classic **ADSR** (Attack-Decay-Sustain-Release) model. It produces output values from 0 to 1 that can be used to shape amplitude, filter cutoff, or any other modulatable parameter.

## Mathematical Foundation

### The ADSR Model

An **envelope** is a time-varying amplitude contour that shapes how a sound evolves. The ADSR model breaks this into four stages:

1. **Attack**: Signal rises from 0 to peak (1.0)
2. **Decay**: Signal falls from peak to sustain level
3. **Sustain**: Signal holds at a fixed level while key is held
4. **Release**: Signal falls from current level to 0 when key is released

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

### State Machine

The envelope progresses through discrete states:

```
    ┌─────────────────────────────────────────────────┐
    │                                                 │
    ▼                                                 │
  IDLE ──note_on()──► ATTACK ──► DECAY ──► SUSTAIN ──┤
    ▲                                        │        │
    │                                   note_off()    │
    │                                        │        │
    └─────────────── RELEASE ◄───────────────┘        │
                        │                             │
                        └─────── (level < floor) ─────┘
```

### Linear Ramp Equations

This implementation uses **linear ramps** for simplicity and predictability.

**Attack Stage:**

$$
L(t) = \frac{t}{t_A}
$$

where $t_A$ is the attack time and $t$ is time since note-on.

**Decay Stage:**

$$
L(t) = 1 - (1 - S) \cdot \frac{t}{t_D}
$$

where $S$ is the sustain level and $t_D$ is decay time.

**Sustain Stage:**

$$
L(t) = S
$$

The level holds constant at the sustain value.

**Release Stage:**

$$
L(t) = L_r \cdot \left(1 - \frac{t}{t_R}\right)
$$

where $L_r$ is the level when release started (captured at note-off) and $t_R$ is release time.

### Linear vs Exponential Curves

**Linear curves** (used here):
- Constant rate of change: $\frac{dL}{dt} = \text{const}$
- Predictable timing
- Sound transitions can feel abrupt at the endpoints

**Exponential curves** (alternative approach):
$$
L(t) = L_{target} + (L_{start} - L_{target}) \cdot e^{-t/\tau}
$$

- Natural decay behavior (like capacitor discharge)
- Sound transitions feel smoother
- Asymptotic approach—never truly reaches target

### Envelope Floor

The implementation uses a floor threshold of $10^{-6}$ (~-120 dB) to detect when release is effectively complete:

$$
\text{if } L < 10^{-6} \text{ then } L \leftarrow 0, \text{ state} \leftarrow \text{Idle}
$$

This prevents floating-point precision issues from keeping the envelope in release state indefinitely.

## Creating an Envelope

```rust
use bbx_dsp::graph::GraphBuilder;

let mut builder = GraphBuilder::<f32>::new(44100.0, 512, 2);

// Parameters: attack(s), decay(s), sustain(0-1), release(s)
let env = builder.add_envelope(0.01, 0.2, 0.7, 0.3);
```

## Port Layout

| Port | Direction | Description |
|------|-----------|-------------|
| 0 | Output | Envelope value (0.0 to 1.0) |

## Parameters

| Parameter | Type | Range | Default | Description |
|-----------|------|-------|---------|-------------|
| attack | f64 | 0.001 - 10.0 s | 0.01 | Time to reach peak |
| decay | f64 | 0.001 - 10.0 s | 0.1 | Time to reach sustain |
| sustain | f64 | 0.0 - 1.0 | 0.5 | Hold level (fraction of peak) |
| release | f64 | 0.001 - 10.0 s | 0.3 | Time to reach zero |

Time values are clamped to [0.001, 10.0] seconds to prevent numerical issues.

### Typical Settings

| Sound Type | Attack | Decay | Sustain | Release |
|------------|--------|-------|---------|---------|
| Pluck/Stab | 0.001 | 0.1 | 0.0 | 0.2 |
| Piano | 0.001 | 0.5 | 0.3 | 0.5 |
| Pad | 0.5 | 0.2 | 0.8 | 1.0 |
| Organ | 0.001 | 0.0 | 1.0 | 0.1 |
| Brass | 0.05 | 0.1 | 0.8 | 0.2 |

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

### Filter Envelope

```rust
use bbx_dsp::{graph::GraphBuilder, waveform::Waveform};

let mut builder = GraphBuilder::<f32>::new(44100.0, 512, 2);

let osc = builder.add_oscillator(440.0, Waveform::Saw, None);
let filter = builder.add_low_pass_filter(500.0, 2.0);
let env = builder.add_envelope(0.01, 0.3, 0.2, 0.5);

builder.connect(osc, 0, filter, 0);
builder.modulate(env, filter, "cutoff");
```

## Control Methods

The envelope provides methods for note control:

```rust
// Trigger the envelope (start attack phase)
env.note_on();

// Release the envelope (start release phase from current level)
env.note_off();

// Reset to idle state immediately
env.reset();
```

## Implementation Notes

- Output range: 0.0 to 1.0
- Linear ramps for all stages
- Times clamped to [0.001, 10.0] seconds
- Envelope floor at $10^{-6}$ for reliable release termination
- Retrigger behavior: calling `note_on()` restarts attack from current level

## Further Reading

- Roads, C. (1996). *The Computer Music Tutorial*, Chapter 4: Synthesis. MIT Press.
- Puckette, M. (2007). *Theory and Techniques of Electronic Music*, Chapter 4. World Scientific.
- Smith, J.O. (2010). *Physical Audio Signal Processing*, Appendix E: Envelope Generators. [online](https://ccrma.stanford.edu/~jos/pasp/)
