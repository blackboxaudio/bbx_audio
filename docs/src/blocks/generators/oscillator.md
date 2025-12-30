# OscillatorBlock

A waveform generator supporting multiple wave shapes.

## Overview

`OscillatorBlock` generates audio waveforms at a specified frequency with optional frequency modulation.

## Creating an Oscillator

```rust
use bbx_dsp::{graph::GraphBuilder, waveform::Waveform};

let mut builder = GraphBuilder::<f32>::new(44100.0, 512, 2);

// Basic oscillator
let osc = builder.add_oscillator(
    440.0,           // Frequency in Hz
    Waveform::Sine,  // Waveform type
    None,            // No frequency modulation
);
```

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

## Frequency Modulation

Use an LFO for vibrato:

```rust
let lfo = builder.add_lfo(5.0, Waveform::Sine);  // 5 Hz vibrato

let osc = builder.add_oscillator(
    440.0,
    Waveform::Sine,
    Some(lfo),  // LFO modulates frequency
);
```

## Port Layout

| Port | Direction | Description |
|------|-----------|-------------|
| 0 | Output | Audio signal |

## Parameters

| Parameter | Type | Range | Default |
|-----------|------|-------|---------|
| Frequency | f64 | 0.01 - 20000 Hz | 440 |
| Waveform | enum | See above | Sine |

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

## Implementation Notes

- Phase accumulator runs continuously
- No anti-aliasing (naive waveforms)
- Noise is sample-and-hold (per-sample random)
