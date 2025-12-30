# PannerBlock

Stereo panning with constant-power law.

## Overview

`PannerBlock` positions a mono signal in the stereo field, or adjusts the stereo balance of a stereo signal.

## Creating a Panner

```rust
use bbx_dsp::graph::GraphBuilder;

let mut builder = GraphBuilder::<f32>::new(44100.0, 512, 2);

let pan = builder.add_panner(0.0);  // Center
```

## Pan Values

| Value | Position |
|-------|----------|
| -1.0 | Hard left |
| -0.5 | Half left |
| 0.0 | Center |
| +0.5 | Half right |
| +1.0 | Hard right |

## Port Layout

| Port | Direction | Description |
|------|-----------|-------------|
| 0 | Input | Audio input (mono or left) |
| 0 | Output | Left channel |
| 1 | Output | Right channel |

## Parameters

| Parameter | Type | Range | Default |
|-----------|------|-------|---------|
| Position | f32 | -1.0 to 1.0 | 0.0 |

## Constant Power Panning

The panner uses a constant-power law (sine/cosine):

```
Left gain  = cos(pan_angle)
Right gain = sin(pan_angle)
```

Where `pan_angle = (position + 1) * π/4`

This maintains perceived loudness as the signal moves across the stereo field.

## Usage Examples

### Basic Panning

```rust
let osc = builder.add_oscillator(440.0, Waveform::Sine, None);
let pan = builder.add_panner(0.5);  // Slightly right

builder.connect(osc, 0, pan, 0);
```

### Auto-Pan with LFO

```rust
let osc = builder.add_oscillator(440.0, Waveform::Sine, None);

// Slow pan LFO
let lfo = builder.add_lfo(0.25, Waveform::Sine);

// Panner with modulation
let pan = builder.add_panner_with_modulation(0.0, Some(lfo));

builder.connect(osc, 0, pan, 0);
```

### Stereo Width

For stereo sources, use two panners:

```rust
let left_source = /* ... */;
let right_source = /* ... */;

// Narrow the stereo field
let left_pan = builder.add_panner(-0.3);   // Less extreme left
let right_pan = builder.add_panner(0.3);   // Less extreme right

builder.connect(left_source, 0, left_pan, 0);
builder.connect(right_source, 0, right_pan, 0);
```

## Implementation Notes

- Constant-power pan law prevents center dip
- Mono input → stereo output
- No interpolation on parameter changes
