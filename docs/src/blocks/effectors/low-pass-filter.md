# LowPassFilterBlock

SVF-based low-pass filter with cutoff and resonance control.

## Overview

`LowPassFilterBlock` is a State Variable Filter (SVF) using the TPT (Topology Preserving Transform) algorithm. It provides:
- Stable filtering at all cutoff frequencies
- No delay-free loops
- Consistent behavior regardless of sample rate

## Mathematical Foundation

### What is a Low-Pass Filter?

A low-pass filter attenuates frequencies above a specified **cutoff frequency** $f_c$ while allowing lower frequencies to pass through. The rate of attenuation above the cutoff is determined by the filter's order:

- **First-order**: -6 dB/octave (-20 dB/decade)
- **Second-order**: -12 dB/octave (-40 dB/decade)

This SVF implementation is second-order, providing -12 dB/octave rolloff.

### Transfer Function

A second-order low-pass filter has the general continuous-time transfer function:

$$
H(s) = \frac{\omega_c^2}{s^2 + \frac{\omega_c}{Q}s + \omega_c^2}
$$

where:
- $s$ is the complex frequency variable
- $\omega_c = 2\pi f_c$ is the cutoff angular frequency
- $Q$ is the quality factor (resonance)

### The Q Factor (Resonance)

The **quality factor** $Q$ controls the filter's behavior near the cutoff frequency:

| Q Value | Character | Peak at Cutoff |
|---------|-----------|----------------|
| 0.5 | Heavily damped, gradual rolloff | No peak |
| 0.707 | **Butterworth** (maximally flat passband) | No peak |
| 1.0 | Slight resonance | Small peak |
| 2.0+ | Pronounced resonance | Noticeable peak |
| 10.0 | Near self-oscillation | Large peak |

The Butterworth response ($Q = \frac{1}{\sqrt{2}} \approx 0.707$) is called "maximally flat" because it has no ripple in the passband—the flattest possible response before rolloff.

At high Q values, the filter emphasizes frequencies near the cutoff, creating the classic "resonant sweep" sound when the cutoff is modulated.

### The TPT/SVF Algorithm

The **Topology Preserving Transform** discretizes analog filter structures while maintaining their essential characteristics. Unlike naive discretization methods, TPT:

1. Preserves the analog frequency response shape
2. Avoids the "cramping" effect near Nyquist
3. Has no delay-free loops (important for real-time stability)

#### Coefficient Calculation

The key coefficient $g$ uses the **tangent pre-warping** formula:

$$
g = \tan\left(\frac{\pi f_c}{f_s}\right)
$$

This maps the analog cutoff frequency to the correct digital frequency, compensating for the frequency warping inherent in bilinear transform discretization.

The damping factor $k$ is the inverse of Q:

$$
k = \frac{1}{Q}
$$

From these, we compute the filter coefficients:

$$
\begin{aligned}
a_1 &= \frac{1}{1 + g(g + k)} \\
a_2 &= g \cdot a_1 \\
a_3 &= g \cdot a_2
\end{aligned}
$$

#### State Variable Processing

The SVF maintains two state variables, $ic_1$ and $ic_2$, representing the integrator states. For each input sample $v_0$:

$$
\begin{aligned}
v_3 &= v_0 - ic_2 \\
v_1 &= a_1 \cdot ic_1 + a_2 \cdot v_3 \\
v_2 &= ic_2 + a_2 \cdot ic_1 + a_3 \cdot v_3
\end{aligned}
$$

The state variables are then updated using the trapezoidal integration rule:

$$
\begin{aligned}
ic_1 &\leftarrow 2v_1 - ic_1 \\
ic_2 &\leftarrow 2v_2 - ic_2
\end{aligned}
$$

The **low-pass output** is $v_2$.

The elegant aspect of this structure is that it simultaneously computes low-pass, high-pass, and band-pass outputs—though this implementation only uses the low-pass.

### Frequency Response

The filter's **magnitude response** $|H(f)|$ describes how much each frequency is attenuated:

$$
|H(f)| = \frac{1}{\sqrt{(1 - (f/f_c)^2)^2 + (f/(f_c \cdot Q))^2}}
$$

Key points:
- At $f = 0$: $|H| = 1$ (unity gain at DC)
- At $f = f_c$: $|H| = Q$ (gain equals Q at cutoff)
- At $f \gg f_c$: $|H| \approx (f_c/f)^2$ (second-order rolloff)

### Resonance Peak Compensation

At high Q values, the resonance peak can cause the output to exceed unity gain significantly. This implementation applies a compensation factor to limit the peak while preserving the filter character:

$$
\text{compensation} = \begin{cases}
1.0 & \text{if } Q \leq 1 \\
\frac{2}{Q} \cdot \text{blend} & \text{if } Q > 1
\end{cases}
$$

This keeps the maximum output level manageable (target peak ≤ 2.0) while maintaining the resonant character.

## Creating a Low-Pass Filter

Using the builder (recommended):

```rust
use bbx_dsp::graph::GraphBuilder;

let mut builder = GraphBuilder::<f32>::new(44100.0, 512, 2);

// Create with cutoff at 1000 Hz, resonance at 0.707 (Butterworth)
let filter = builder.add_low_pass_filter(1000.0, 0.707);
```

Direct construction:

```rust
use bbx_dsp::blocks::LowPassFilterBlock;

let filter = LowPassFilterBlock::<f32>::new(1000.0, 0.707);
```

## Port Layout

| Port | Direction | Description |
|------|-----------|-------------|
| 0 | Input | Audio input |
| 0 | Output | Filtered audio |

## Parameters

| Parameter | Type | Range | Default |
|-----------|------|-------|---------|
| Cutoff | f64 | 20-20000 Hz | - |
| Resonance | f64 | 0.5-10.0 | 0.707 |

### Resonance Values

| Value | Character |
|-------|-----------|
| 0.5 | Heavily damped |
| 0.707 | Butterworth (flat) |
| 1.0 | Slight peak |
| 2.0+ | Pronounced peak |
| 10.0 | Near self-oscillation |

## Usage Examples

### Basic Filtering

```rust
use bbx_dsp::{graph::GraphBuilder, waveform::Waveform};

let mut builder = GraphBuilder::<f32>::new(44100.0, 512, 2);

let source = builder.add_oscillator(440.0, Waveform::Saw, None);
let filter = builder.add_low_pass_filter(2000.0, 0.707);

// Connect oscillator to filter
builder.connect(source, 0, filter, 0);
```

### Synthesizer Voice

```rust
use bbx_dsp::{graph::GraphBuilder, waveform::Waveform};

let mut builder = GraphBuilder::<f32>::new(44100.0, 512, 2);

// Typical synth voice with envelope-modulated filter
let osc = builder.add_oscillator(440.0, Waveform::Saw, None);
let filter = builder.add_low_pass_filter(1000.0, 2.0);  // Resonant
let env = builder.add_envelope(0.01, 0.2, 0.5, 0.3);
let amp = builder.add_gain(-6.0, None);

// Connect: Osc -> Filter -> Gain
builder.connect(osc, 0, filter, 0);
builder.connect(filter, 0, amp, 0);
```

### Filter Sweep with LFO

```rust
use bbx_dsp::{graph::GraphBuilder, waveform::Waveform};

let mut builder = GraphBuilder::<f32>::new(44100.0, 512, 2);

let osc = builder.add_oscillator(440.0, Waveform::Saw, None);
let filter = builder.add_low_pass_filter(1000.0, 4.0);  // High resonance
let lfo = builder.add_lfo(0.5, 1.0, None);  // Slow sweep

builder.connect(osc, 0, filter, 0);
builder.modulate(lfo, filter, "cutoff");
```

## Implementation Notes

- Uses TPT SVF algorithm for numerical stability
- Multi-channel processing (up to MAX_BLOCK_OUTPUTS channels)
- Independent state per channel
- Coefficients recalculated per buffer (supports modulation)
- Denormals flushed to prevent CPU stalls

## Further Reading

- Zavalishin, V. (2012). *The Art of VA Filter Design*. Native Instruments. [PDF](https://www.native-instruments.com/fileadmin/ni_media/downloads/pdf/VAFilterDesign_2.1.0.pdf)
- Smith, J.O. (2007). *Introduction to Digital Filters*. [online](https://ccrma.stanford.edu/~jos/filters/)
- Pirkle, W. (2019). *Designing Audio Effect Plugins in C++*, Chapter 11: Filters.
- Butterworth, S. (1930). "On the Theory of Filter Amplifiers." *Experimental Wireless*, 7, 536-541.
