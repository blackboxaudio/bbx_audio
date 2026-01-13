# OverdriveBlock

Asymmetric soft-clipping distortion with tone control.

## Overview

`OverdriveBlock` applies warm, tube-like distortion using hyperbolic tangent saturation with different curves for positive and negative signal halves. This asymmetric approach creates a more organic sound than symmetric clipping.

## Mathematical Foundation

### What is Distortion?

Distortion occurs when a system's output is a **nonlinear function** of its input. In audio, this nonlinearity creates new frequencies (harmonics) that weren't present in the original signal.

For a pure sine wave input at frequency $f$:
- **Linear system**: Output is still just frequency $f$
- **Nonlinear system**: Output contains $f$, $2f$, $3f$, $4f$... (harmonics)

### Soft vs Hard Clipping

**Hard clipping** simply limits the signal:
$$
y = \begin{cases}
1 & \text{if } x > 1 \\
x & \text{if } |x| \leq 1 \\
-1 & \text{if } x < -1
\end{cases}
$$

This creates harsh distortion with many high-frequency harmonics.

**Soft clipping** uses a smooth saturation curve:
$$
y = \tanh(x)
$$

The hyperbolic tangent smoothly approaches $\pm 1$ as $|x| \to \infty$, creating a warmer, more musical distortion.

### The Hyperbolic Tangent

The **tanh** function is ideal for soft clipping:

$$
\tanh(x) = \frac{e^x - e^{-x}}{e^x + e^{-x}}
$$

Properties:
- Output range: $(-1, 1)$
- Passes through origin: $\tanh(0) = 0$
- Odd function: $\tanh(-x) = -\tanh(x)$
- Smooth everywhere
- Approaches $\pm 1$ asymptotically

### Saturation Curve with Adjustable Knee

This implementation uses a scaled tanh to control the "knee" (how abruptly clipping begins):

$$
y = \frac{\tanh(k \cdot x)}{k}
$$

where $k$ is the knee factor (this block uses $k = 1.5$).

- Larger $k$: Sharper knee, more aggressive clipping
- Smaller $k$: Softer knee, more gradual compression

### Asymmetric Saturation

Real tube amplifiers and analog circuits often clip asymmetrically—the positive and negative halves of the waveform are processed differently. This creates **even harmonics** (2nd, 4th, 6th...) in addition to odd harmonics.

This block applies different scaling to positive and negative signals:

$$
y = \begin{cases}
1.4 \cdot \text{softclip}(0.7x) & \text{if } x > 0 \quad \text{(softer, more headroom)} \\
0.8 \cdot \text{softclip}(1.2x) & \text{if } x < 0 \quad \text{(harder, more compression)}
\end{cases}
$$

where:
$$
\text{softclip}(x) = \frac{\tanh(1.5x)}{1.5}
$$

### Harmonic Content

| Clipping Type | Harmonics Generated | Character |
|--------------|---------------------|-----------|
| Symmetric soft | Odd only (3rd, 5th, 7th...) | Clean, tube-like |
| Asymmetric soft | Odd + Even (2nd, 3rd, 4th...) | Warm, organic |
| Hard clip | Many high-order | Harsh, buzzy |

The 2nd harmonic is an octave above the fundamental and adds "warmth." The 3rd harmonic is an octave + fifth, adding "body."

### Tone Control Filter

After clipping, a **one-pole low-pass filter** controls brightness:

$$
y[n] = y[n-1] + \alpha \cdot (x[n] - y[n-1])
$$

where:
$$
\alpha = 1 - e^{-2\pi f_c / f_s}
$$

The cutoff frequency $f_c$ is mapped from the tone parameter:
- Tone 0.0: $f_c \approx 300$ Hz (dark)
- Tone 1.0: $f_c \approx 3000$ Hz (bright)

## Creating an Overdrive

```rust
use bbx_dsp::{blocks::OverdriveBlock, graph::GraphBuilder};

let mut builder = GraphBuilder::<f32>::new(44100.0, 512, 2);

// Parameters: drive, level, tone, sample_rate
let od = builder.add(OverdriveBlock::new(2.0, 0.8, 0.5, 44100.0));
```

## Port Layout

| Port | Direction | Description |
|------|-----------|-------------|
| 0 | Input | Audio input |
| 0 | Output | Distorted audio |

## Parameters

| Parameter | Type | Range | Default | Description |
|-----------|------|-------|---------|-------------|
| drive | f64 | 1.0 - 10.0+ | 1.0 | Input gain before clipping |
| level | f64 | 0.0 - 1.0 | 1.0 | Output level |
| tone | f64 | 0.0 - 1.0 | 0.5 | Brightness (0=dark, 1=bright) |

### Drive Values

| Drive | Character |
|-------|-----------|
| 1.0 | Subtle warmth |
| 3.0 | Moderate saturation |
| 5.0 | Heavy overdrive |
| 10.0+ | Aggressive distortion |

## Usage Examples

### Basic Overdrive

```rust
use bbx_dsp::{blocks::{OscillatorBlock, OverdriveBlock}, graph::GraphBuilder, waveform::Waveform};

let mut builder = GraphBuilder::<f32>::new(44100.0, 512, 2);

let osc = builder.add(OscillatorBlock::new(440.0, Waveform::Sine, None));
let od = builder.add(OverdriveBlock::new(3.0, 0.7, 0.5, 44100.0));

builder.connect(osc, 0, od, 0);
```

### Aggressive Distortion

```rust
use bbx_dsp::{blocks::{OscillatorBlock, OverdriveBlock}, graph::GraphBuilder, waveform::Waveform};

let mut builder = GraphBuilder::<f32>::new(44100.0, 512, 2);

let osc = builder.add(OscillatorBlock::new(110.0, Waveform::Saw, None));
let od = builder.add(OverdriveBlock::new(8.0, 0.5, 0.6, 44100.0));  // High drive, lower output

builder.connect(osc, 0, od, 0);
```

### With DC Blocker

Distortion can introduce DC offset:

```rust
use bbx_dsp::{blocks::{DcBlockerBlock, OscillatorBlock, OverdriveBlock}, graph::GraphBuilder, waveform::Waveform};

let mut builder = GraphBuilder::<f32>::new(44100.0, 512, 2);

let osc = builder.add(OscillatorBlock::new(440.0, Waveform::Sine, None));
let od = builder.add(OverdriveBlock::new(5.0, 0.7, 0.5, 44100.0));
let dc = builder.add(DcBlockerBlock::new(true));

builder.connect(osc, 0, od, 0);
builder.connect(od, 0, dc, 0);
```

### Warm Bass Distortion

```rust
use bbx_dsp::{blocks::{OscillatorBlock, OverdriveBlock}, graph::GraphBuilder, waveform::Waveform};

let mut builder = GraphBuilder::<f32>::new(44100.0, 512, 2);

let osc = builder.add(OscillatorBlock::new(80.0, Waveform::Saw, None));
let od = builder.add(OverdriveBlock::new(2.0, 0.8, 0.3, 44100.0));  // Low tone for warmth

builder.connect(osc, 0, od, 0);
```

## Implementation Notes

- Asymmetric soft clipping using scaled tanh
- One-pole low-pass filter for tone control
- Click-free parameter changes via smoothing
- Denormal flushing in filter state
- Multi-channel with independent filter states

## Further Reading

- Pirkle, W. (2019). *Designing Audio Effect Plugins in C++*, Chapter 19: Distortion Effects. Focal Press.
- Smith, J.O. (2010). *Physical Audio Signal Processing*, Appendix I: Nonlinear Distortion. [online](https://ccrma.stanford.edu/~jos/pasp/)
- Schimmel, J. (2008). "Nonlinear Filtering in Computer Music." *Proceedings of DAFx*.
- Pakarinen, J. & Yeh, D. (2009). "A Review of Digital Techniques for Modeling Vacuum-Tube Guitar Amplifiers." *Computer Music Journal*, 33(2).
