# OscillatorBlock

A waveform generator supporting multiple wave shapes with band-limited output.

## Overview

`OscillatorBlock` generates audio waveforms at a specified frequency with optional frequency modulation via the `modulate()` method.

## Mathematical Foundation

### Phase Accumulator

At the heart of every oscillator is a **phase accumulator**—a counter that tracks the current position within a waveform's cycle. The phase $\phi$ advances by a fixed increment each sample:

$$
\phi[n] = \phi[n-1] + \Delta\phi
$$

where the **phase increment** is determined by the desired frequency and sample rate:

$$
\Delta\phi = \frac{2\pi f}{f_s}
$$

- $f$ is the oscillator frequency in Hz
- $f_s$ is the sample rate (e.g., 44100 Hz)
- $2\pi$ represents one complete cycle (radians)

The phase wraps around at $2\pi$ to stay within one period: $\phi = \phi \mod 2\pi$

**Example:** A 440 Hz oscillator at 44100 Hz sample rate advances by $\frac{2\pi \times 440}{44100} \approx 0.0627$ radians per sample, completing one cycle in approximately 100 samples.

### Waveform Equations

Each waveform shape maps the phase $\phi \in [0, 2\pi)$ to an output amplitude $y \in [-1, 1]$. These "naive" formulas produce the correct shape but suffer from aliasing at higher frequencies.

| Waveform | Formula | Output Range |
|----------|---------|--------------|
| Sine | $y = \sin(\phi)$ | $[-1, 1]$ |
| Sawtooth | $y = \frac{\phi}{\pi} - 1$ | $[-1, 1]$ |
| Square | $y = \text{sign}(\sin(\phi))$ | $\{-1, 1\}$ |
| Triangle | $y = \frac{2}{\pi}\|\phi - \pi\| - 1$ | $[-1, 1]$ |
| Pulse | $y = \begin{cases} 1 & \text{if } \phi < 2\pi d \\ -1 & \text{otherwise} \end{cases}$ | $\{-1, 1\}$ |

where $d$ is the duty cycle (default 0.5 for 50% duty).

### Harmonic Content

Different waveforms contain different harmonics, giving them distinct timbres:

| Waveform | Harmonics Present | Amplitude Falloff |
|----------|-------------------|-------------------|
| Sine | Fundamental only | — |
| Square | Odd (1, 3, 5, 7...) | $\frac{1}{n}$ |
| Sawtooth | All (1, 2, 3, 4...) | $\frac{1}{n}$ |
| Triangle | Odd (1, 3, 5, 7...) | $\frac{1}{n^2}$ |
| Pulse | All | Varies with duty cycle |

The **Fourier series** for a sawtooth wave illustrates why it sounds bright:

$$
y(t) = -\frac{2}{\pi} \sum_{n=1}^{\infty} \frac{(-1)^n}{n} \sin(2\pi n f t)
$$

### The Aliasing Problem

The **Nyquist theorem** states that we can only accurately represent frequencies below $\frac{f_s}{2}$ (the Nyquist frequency). For a 44.1 kHz sample rate, this limit is 22.05 kHz.

Waveforms with sharp discontinuities (square, sawtooth, pulse) theoretically contain infinite harmonics. When the oscillator frequency $f$ is high enough that harmonics exceed the Nyquist frequency, those harmonics "fold back" into the audible range as **aliasing**—harsh, unmusical artifacts.

**Example:** A 10 kHz sawtooth has harmonics at 10k, 20k, 30k, 40k... Hz. At 44.1 kHz sample rate:
- The 30 kHz harmonic aliases to $44.1 - 30 = 14.1$ kHz
- The 40 kHz harmonic aliases to $44.1 - 40 = 4.1$ kHz

### Band-Limited Synthesis with PolyBLEP

**PolyBLEP** (Polynomial Band-Limited Step) is an efficient technique that reduces aliasing by smoothing discontinuities. Rather than computing a band-limited waveform from scratch, it starts with the naive waveform and applies a correction near transitions.

The key insight: aliasing comes from **step discontinuities** (sudden jumps in value). If we can smooth these jumps, we reduce the high-frequency content that causes aliasing.

#### The PolyBLEP Correction

For a discontinuity at normalized phase $t = 0$ (where $t \in [0, 1]$), the PolyBLEP correction is:

$$
\text{polyBLEP}(t, \Delta t) =
\begin{cases}
2t' - t'^2 - 1 & \text{if } t < \Delta t \\
t'^2 + 2t' + 1 & \text{if } t > 1 - \Delta t \\
0 & \text{otherwise}
\end{cases}
$$

where $t' = t / \Delta t$ near the start of the cycle, or $t' = (t - 1) / \Delta t$ near the end, and $\Delta t$ is the normalized phase increment.

The correction is applied by subtracting from the naive waveform:

$$
y_{\text{bandlimited}} = y_{\text{naive}} - \text{polyBLEP}(t, \Delta t)
$$

#### PolyBLAMP for Triangle Waves

Triangle waves have **slope discontinuities** (sudden changes in direction) rather than step discontinuities. **PolyBLAMP** (Band-Limited rAMP) is the integrated form of PolyBLEP, designed for these cases:

$$
\text{polyBLAMP}(t, \Delta t) =
\begin{cases}
(t'^2 - \frac{t'^3}{3} - t') \cdot \Delta t & \text{if } t < \Delta t \\
(\frac{t'^3}{3} + t'^2 + t') \cdot \Delta t & \text{if } t > 1 - \Delta t \\
0 & \text{otherwise}
\end{cases}
$$

The triangle wave correction uses a scaling factor of 8:

$$
y_{\text{triangle}} = y_{\text{naive}} + 8 \cdot \text{polyBLAMP}(t, \Delta t) - 8 \cdot \text{polyBLAMP}(t + 0.5, \Delta t)
$$

### Pitch Modulation

Pitch offset is applied using the equal-tempered tuning formula:

$$
f_{\text{actual}} = f_{\text{base}} \cdot 2^{\frac{s}{12}}
$$

where $s$ is the offset in semitones. This formula means:
- +12 semitones doubles the frequency (one octave up)
- -12 semitones halves the frequency (one octave down)
- +1 semitone multiplies by $2^{1/12} \approx 1.0595$

## Creating an Oscillator

```rust
use bbx_dsp::{blocks::OscillatorBlock, graph::GraphBuilder, waveform::Waveform};

let mut builder = GraphBuilder::<f32>::new(44100.0, 512, 2);

// Parameters: frequency (Hz), waveform, optional seed for noise
let osc = builder.add(OscillatorBlock::new(440.0, Waveform::Sine, None));
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
use bbx_dsp::{blocks::{LfoBlock, OscillatorBlock}, graph::GraphBuilder, waveform::Waveform};

let mut builder = GraphBuilder::<f32>::new(44100.0, 512, 2);

// Create LFO for vibrato (5 Hz, moderate depth)
let lfo = builder.add(LfoBlock::new(5.0, 0.3, Waveform::Sine, None));

// Create oscillator
let osc = builder.add(OscillatorBlock::new(440.0, Waveform::Sine, None));

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
let osc = builder.add(OscillatorBlock::new(440.0, Waveform::Sine, None));
```

### Detuned Oscillators

```rust
let osc1 = builder.add(OscillatorBlock::new(440.0, Waveform::Saw, None));
let osc2 = builder.add(OscillatorBlock::new(440.0 * 1.005, Waveform::Saw, None));  // +8.6 cents
let osc3 = builder.add(OscillatorBlock::new(440.0 / 1.005, Waveform::Saw, None));  // -8.6 cents
```

### Sub-Oscillator

```rust
let main = builder.add(OscillatorBlock::new(440.0, Waveform::Saw, None));
let sub = builder.add(OscillatorBlock::new(220.0, Waveform::Sine, None));  // One octave down
```

### Deterministic Noise

For reproducible noise output:

```rust
// Same seed produces same noise pattern
let noise1 = builder.add(OscillatorBlock::new(0.0, Waveform::Noise, Some(12345)));
let noise2 = builder.add(OscillatorBlock::new(0.0, Waveform::Noise, Some(12345)));  // Same as noise1
```

## Implementation Notes

- Phase accumulator runs continuously
- **Band-limited output** using PolyBLEP/PolyBLAMP anti-aliasing:
  - Saw, Square, Pulse: PolyBLEP corrects step discontinuities
  - Triangle: PolyBLAMP corrects slope discontinuities
  - Sine: Naturally band-limited (no correction needed)
  - Noise: No discontinuities (no correction needed)
- Noise is sample-and-hold (per-sample random)
- Frequency is ignored for Noise waveform

## Further Reading

- Välimäki, V., et al. (2010). "Oscillator and Filter Algorithms for Virtual Analog Synthesis." *Computer Music Journal*, 30(2).
- Smith, J.O. (2010). *Physical Audio Signal Processing*. [online](https://ccrma.stanford.edu/~jos/pasp/)
- Pirkle, W. (2019). *Designing Audio Effect Plugins in C++*, Chapter 8: Oscillators.
