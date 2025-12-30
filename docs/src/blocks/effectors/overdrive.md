# OverdriveBlock

Soft-clipping distortion for warmth and saturation.

## Overview

`OverdriveBlock` applies asymmetric soft-clipping distortion, adding harmonics and warmth to audio signals.

## Creating an Overdrive

```rust
use bbx_dsp::graph::GraphBuilder;

let mut builder = GraphBuilder::<f32>::new(44100.0, 512, 2);

let drive = builder.add_overdrive(
    3.0,      // Drive amount (higher = more distortion)
    1.0,      // Input gain
    0.8,      // Output gain (compensate for level increase)
    44100.0,  // Sample rate
);
```

## Parameters

| Parameter | Type | Range | Default |
|-----------|------|-------|---------|
| Drive | f64 | 0.1 - 20.0 | 1.0 |
| Input Gain | f64 | 0.0 - 10.0 | 1.0 |
| Output Gain | f64 | 0.0 - 2.0 | 1.0 |

### Drive Amount

Controls distortion intensity:

| Drive | Character |
|-------|-----------|
| 1.0 | Subtle warmth |
| 3.0 | Moderate saturation |
| 5.0 | Heavy overdrive |
| 10.0+ | Aggressive distortion |

### Input/Output Gain

- **Input Gain**: Boost signal before clipping (affects distortion character)
- **Output Gain**: Compensate for level increase from distortion

## Port Layout

| Port | Direction | Description |
|------|-----------|-------------|
| 0 | Input | Audio input |
| 0 | Output | Distorted audio |

## Usage Examples

### Basic Overdrive

```rust
let osc = builder.add_oscillator(440.0, Waveform::Sine, None);
let drive = builder.add_overdrive(3.0, 1.0, 0.8, 44100.0);

builder.connect(osc, 0, drive, 0);
```

### Aggressive Distortion

```rust
let osc = builder.add_oscillator(110.0, Waveform::Saw, None);
let drive = builder.add_overdrive(
    8.0,   // High drive
    1.5,   // Boost input
    0.5,   // Lower output to compensate
    44100.0,
);

builder.connect(osc, 0, drive, 0);
```

### With DC Blocker

Distortion can introduce DC offset:

```rust
let osc = builder.add_oscillator(440.0, Waveform::Sine, None);
let drive = builder.add_overdrive(5.0, 1.0, 0.7, 44100.0);
let dc = builder.add_dc_blocker();

builder.connect(osc, 0, drive, 0);
builder.connect(drive, 0, dc, 0);
```

## Algorithm

The overdrive uses asymmetric soft-clipping:

```rust
fn soft_clip(x: f32, drive: f32) -> f32 {
    let driven = x * drive;
    driven / (1.0 + driven.abs())
}
```

This produces:
- Even and odd harmonics
- Asymmetric positive/negative response
- Gradual compression at high levels
