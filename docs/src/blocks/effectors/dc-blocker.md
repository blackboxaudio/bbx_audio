# DcBlockerBlock

DC offset removal filter.

## Overview

`DcBlockerBlock` removes DC offset (constant voltage) from audio signals, preventing speaker damage and headroom loss.

## Creating a DC Blocker

```rust
use bbx_dsp::graph::GraphBuilder;

let mut builder = GraphBuilder::<f32>::new(44100.0, 512, 2);

let dc = builder.add_dc_blocker();
```

## Why DC Blocking?

DC offset occurs when a signal's average value is not zero:

- Asymmetric distortion
- Some waveforms (e.g., asymmetric pulse)
- Sample rate conversion artifacts
- External audio input

Problems caused by DC offset:
- Reduced headroom
- Asymmetric clipping
- Potential speaker damage
- Incorrect metering

## Port Layout

| Port | Direction | Description |
|------|-----------|-------------|
| 0 | Input | Audio with DC |
| 0 | Output | DC-free audio |

## Usage Examples

### After Distortion

```rust
let osc = builder.add_oscillator(440.0, Waveform::Sine, None);
let drive = builder.add_overdrive(5.0, 1.0, 0.7, 44100.0);
let dc = builder.add_dc_blocker();

builder.connect(osc, 0, drive, 0);
builder.connect(drive, 0, dc, 0);
```

### On External Input

```rust
// External audio (e.g., from file or interface)
let input = builder.add_file_input(Box::new(reader));
let dc = builder.add_dc_blocker();

builder.connect(input, 0, dc, 0);
```

## Algorithm

A simple high-pass filter:

```rust
// Single-pole high-pass at ~5 Hz
fn process(&mut self, input: f32) -> f32 {
    let output = input - self.x1 + self.r * self.y1;
    self.x1 = input;
    self.y1 = output;
    output
}
```

Where `r` is typically 0.995 (5 Hz cutoff at 44.1 kHz).

## Characteristics

- First-order high-pass filter
- Very low cutoff frequency (~5 Hz)
- Minimal impact on audible frequencies
- Introduces tiny phase shift at low frequencies
- Needs `prepare()` when sample rate changes

## When to Use

**Do use** after:
- Distortion effects
- Asymmetric processing
- External audio input
- Sample rate conversion

**Don't use**:
- On pure sine waves (adds unnecessary processing)
- When sub-bass content is critical
- In series (one is enough)
