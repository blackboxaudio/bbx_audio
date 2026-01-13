# DcBlockerBlock

DC offset removal using a first-order high-pass filter.

## Overview

`DcBlockerBlock` removes DC offset (constant voltage) from audio signals, preventing speaker damage and headroom loss. It uses a very low-frequency high-pass filter that passes all audible content while removing the DC component.

## Mathematical Foundation

### What is DC Offset?

**DC offset** occurs when a signal's average value is not zero. In audio, signals should oscillate symmetrically around zero:

```
Centered signal:        Signal with DC offset:
      +                       +
   __/ \__                 __/ \__
--/------\--           --/------\-- ← shifted up
  \_    _/                \_    _/
     --
```

### Why Remove DC?

A constant (DC) component wastes headroom and causes problems:

1. **Reduced headroom**: Asymmetric clipping occurs sooner
2. **Speaker damage**: DC current heats voice coils
3. **Incorrect metering**: Peak meters show false values
4. **Downstream effects**: Some effects (like compressors) behave poorly

### The High-Pass Filter

A DC blocker is simply a **high-pass filter** with a very low cutoff frequency (~5 Hz). This removes the DC component (0 Hz) while passing all audible frequencies (20 Hz and above).

#### First-Order High-Pass

The difference equation for a first-order high-pass filter:

$$
y[n] = x[n] - x[n-1] + R \cdot y[n-1]
$$

where:
- $x[n]$ is the current input sample
- $y[n]$ is the current output sample
- $R$ is the filter coefficient (typically ~0.995)

#### Filter Coefficient

The coefficient $R$ determines the cutoff frequency:

$$
R = 1 - \frac{2\pi f_c}{f_s}
$$

where $f_c$ is the cutoff frequency (5 Hz) and $f_s$ is the sample rate.

For 5 Hz cutoff at 44.1 kHz:
$$
R = 1 - \frac{2\pi \times 5}{44100} \approx 0.9993
$$

The coefficient is clamped to [0.9, 0.9999] for stability.

### Transfer Function

The Z-domain transfer function:

$$
H(z) = \frac{1 - z^{-1}}{1 - R \cdot z^{-1}}
$$

#### Pole-Zero Analysis

- **Zero** at $z = 1$ (DC is completely blocked)
- **Pole** at $z = R$ (just inside the unit circle)

The zero at $z = 1$ guarantees complete DC rejection. The pole close to $z = 1$ creates a gentle rolloff.

### Frequency Response

The magnitude response:

$$
|H(f)| = \frac{\sqrt{2(1 - \cos(2\pi f / f_s))}}{\sqrt{1 + R^2 - 2R\cos(2\pi f / f_s)}}
$$

Key points:
- At DC ($f = 0$): $|H| = 0$ (complete rejection)
- At cutoff ($f = f_c$): $|H| \approx 0.707$ (-3 dB)
- At 20 Hz: $|H| > 0.99$ (minimal attenuation)

## Creating a DC Blocker

```rust
use bbx_dsp::{
    block::BlockType,
    blocks::DcBlockerBlock,
    graph::GraphBuilder,
};

let mut builder = GraphBuilder::<f32>::new(44100.0, 512, 2);

let dc = builder.add_dc_blocker();
```

Or with direct construction:

```rust
// Enabled DC blocker
let dc = DcBlockerBlock::<f32>::new(true);

// Disabled (bypass)
let dc = DcBlockerBlock::<f32>::new(false);
```

## Port Layout

| Port | Direction | Description |
|------|-----------|-------------|
| 0 | Input | Audio with DC |
| 0 | Output | DC-free audio |

## Parameters

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| enabled | bool | true | Enable/disable filtering |

## Usage Examples

### After Distortion

Asymmetric distortion often introduces DC offset:

```rust
use bbx_dsp::{graph::GraphBuilder, waveform::Waveform};

let mut builder = GraphBuilder::<f32>::new(44100.0, 512, 2);

let osc = builder.add_oscillator(440.0, Waveform::Sine, None);
let drive = builder.add_overdrive(5.0, 0.7, 0.5);
let dc = builder.add_dc_blocker();

builder.connect(osc, 0, drive, 0);
builder.connect(drive, 0, dc, 0);
```

### On External Input

Remove DC from external audio sources:

```rust
use bbx_dsp::{graph::GraphBuilder};

let mut builder = GraphBuilder::<f32>::new(44100.0, 512, 2);

let input = builder.add_file_input(Box::new(reader));
let dc = builder.add_dc_blocker();

builder.connect(input, 0, dc, 0);
```

### Output Stage

Standard practice to include DC blocking on the master output:

```rust
use bbx_dsp::{graph::GraphBuilder, waveform::Waveform};

let mut builder = GraphBuilder::<f32>::new(44100.0, 512, 2);

let osc = builder.add_oscillator(440.0, Waveform::Saw, None);
let gain = builder.add_gain(-12.0, None);
let dc = builder.add_dc_blocker();
let output = builder.add_output();

builder.connect(osc, 0, gain, 0);
builder.connect(gain, 0, dc, 0);
builder.connect(dc, 0, output, 0);
```

## When to Use

**Do use** after:
- Distortion/saturation effects
- Asymmetric waveshaping
- Sample rate conversion
- External audio input
- Mixing multiple sources

**Don't use**:
- In series (one is enough)
- When sub-bass content is critical (though 5 Hz is inaudible)
- On already DC-free signals (adds unnecessary processing)

## Implementation Notes

- First-order high-pass filter (~5 Hz cutoff)
- Coefficient recalculated when sample rate changes
- Denormals flushed to prevent CPU slowdown
- Multi-channel with independent filter states
- Can be disabled for bypass (passthrough mode)
- Resettable filter state via `reset()` method

## Further Reading

- Smith, J.O. (2007). *Introduction to Digital Filters*, Chapter 1. [online](https://ccrma.stanford.edu/~jos/filters/)
- Orfanidis, S. (2010). *Introduction to Signal Processing*, Chapter 4. Rutgers.
- Pirkle, W. (2019). *Designing Audio Effect Plugins in C++*, Chapter 6: Filters. Focal Press.
