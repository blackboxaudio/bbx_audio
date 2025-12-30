# LfoBlock

Low-frequency oscillator for parameter modulation.

## Overview

`LfoBlock` generates low-frequency control signals for modulating parameters like pitch, amplitude, and filter cutoff.

## Creating an LFO

```rust
use bbx_dsp::{graph::GraphBuilder, waveform::Waveform};

let mut builder = GraphBuilder::<f32>::new(44100.0, 512, 2);

let lfo = builder.add_lfo(
    5.0,            // Rate in Hz
    Waveform::Sine, // Waveform
);
```

## Parameters

| Parameter | Type | Range | Default |
|-----------|------|-------|---------|
| Rate | f64 | 0.01 - 100 Hz | 1.0 |
| Waveform | enum | See below | Sine |

## Waveforms

| Waveform | Output Range | Character |
|----------|--------------|-----------|
| Sine | -1.0 to 1.0 | Smooth, natural |
| Triangle | -1.0 to 1.0 | Linear sweeps |
| Square | -1.0 and 1.0 | On/off switching |
| Saw | -1.0 to 1.0 | Ramp up, reset |

## Port Layout

| Port | Direction | Description |
|------|-----------|-------------|
| 0 | Modulation Output | Control signal |

## Usage Examples

### Vibrato (Pitch Modulation)

```rust
let lfo = builder.add_lfo(5.0, Waveform::Sine);
let osc = builder.add_oscillator(440.0, Waveform::Sine, Some(lfo));
```

### Tremolo (Amplitude Modulation)

```rust
let lfo = builder.add_lfo(6.0, Waveform::Sine);
let gain = builder.add_gain_with_modulation(-6.0, Some(lfo));
```

### Auto-Pan

```rust
let lfo = builder.add_lfo(0.25, Waveform::Sine);  // Slow
let pan = builder.add_panner_with_modulation(0.0, Some(lfo));
```

## Rate Guidelines

| Application | Typical Rate |
|-------------|--------------|
| Vibrato | 4-7 Hz |
| Tremolo | 4-10 Hz |
| Auto-pan | 0.1-1 Hz |
| Wobble bass | 1-4 Hz |
| Sweep | 0.05-0.5 Hz |

## Output Scaling

LFO output is -1.0 to 1.0. The receiving block interprets this:

- **Pitch**: Maps to frequency deviation
- **Amplitude**: Maps to gain change
- **Pan**: Maps to position change

## Implementation Notes

- Generates per-block (not per-sample)
- Phase is continuous across blocks
- Rate can be modulated (for complex modulation)
