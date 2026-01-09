# OscillatorBlock

A waveform generator supporting multiple wave shapes.

## Overview

`OscillatorBlock` generates audio waveforms at a specified frequency with optional frequency modulation via the `modulate()` method.

## Creating an Oscillator

```rust
use bbx_dsp::{graph::GraphBuilder, waveform::Waveform};

let mut builder = GraphBuilder::<f32>::new(44100.0, 512, 2);

// Parameters: frequency (Hz), waveform, optional seed for noise
let osc = builder.add_oscillator(440.0, Waveform::Sine, None);
```

The third parameter is an optional seed (`Option<u64>`) for deterministic random number generation, used by the Noise waveform.

## Waveforms

```rust
use bbx_dsp::waveform::Waveform;

Waveform::Sine      // Pure sine wave
Waveform::Square    // Square wave (50% duty)
Waveform::Saw       // Sawtooth (ramp up)
Waveform::Triangle  // Triangle wave
Waveform::Pulse     // Pulse with variable width
Waveform::Noise     // White noise
```

### Waveform Characteristics

| Waveform | Harmonics | Character |
|----------|-----------|-----------|
| Sine | Fundamental only | Pure, clean |
| Square | Odd harmonics | Hollow, woody |
| Saw | All harmonics | Bright, buzzy |
| Triangle | Odd (weak) | Soft, flute-like |
| Pulse | Variable | Nasal, reedy |
| Noise | All frequencies | Airy, percussive |

## Parameters

| Parameter | Type | Range | Default | Description |
|-----------|------|-------|---------|-------------|
| frequency | f64 | 0.01 - 20000 Hz | 440 | Base frequency |
| pitch_offset | f64 | -24 to +24 semitones | 0 | Pitch offset from base |

Both parameters can be modulated using the `modulate()` method.

## Port Layout

| Port | Direction | Description |
|------|-----------|-------------|
| 0 | Output | Audio signal |

## Frequency Modulation

Use an LFO for vibrato via the `modulate()` method:

```rust
use bbx_dsp::{graph::GraphBuilder, waveform::Waveform};

let mut builder = GraphBuilder::<f32>::new(44100.0, 512, 2);

// Create LFO for vibrato (5 Hz, moderate depth)
let lfo = builder.add_lfo(5.0, 0.3, None);

// Create oscillator
let osc = builder.add_oscillator(440.0, Waveform::Sine, None);

// Connect LFO to modulate frequency
builder.modulate(lfo, osc, "frequency");
```

You can also modulate the pitch_offset parameter:

```rust
// Modulate pitch offset instead of frequency
builder.modulate(lfo, osc, "pitch_offset");
```

## Usage Examples

### Basic Tone

```rust
let osc = builder.add_oscillator(440.0, Waveform::Sine, None);
```

### Detuned Oscillators

```rust
let osc1 = builder.add_oscillator(440.0, Waveform::Saw, None);
let osc2 = builder.add_oscillator(440.0 * 1.005, Waveform::Saw, None);  // +8.6 cents
let osc3 = builder.add_oscillator(440.0 / 1.005, Waveform::Saw, None);  // -8.6 cents
```

### Sub-Oscillator

```rust
let main = builder.add_oscillator(440.0, Waveform::Saw, None);
let sub = builder.add_oscillator(220.0, Waveform::Sine, None);  // One octave down
```

### Deterministic Noise

For reproducible noise output:

```rust
// Same seed produces same noise pattern
let noise1 = builder.add_oscillator(0.0, Waveform::Noise, Some(12345));
let noise2 = builder.add_oscillator(0.0, Waveform::Noise, Some(12345));  // Same as noise1
```

## Implementation Notes

- Phase accumulator runs continuously
- No anti-aliasing (naive waveforms)
- Noise is sample-and-hold (per-sample random)
- Frequency is ignored for Noise waveform
