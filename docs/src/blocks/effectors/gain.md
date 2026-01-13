# GainBlock

Amplitude control with decibel input and smooth parameter changes.

## Overview

`GainBlock` applies amplitude scaling to audio signals. The gain level is specified in **decibels (dB)**, the standard unit for audio levels, and changes are smoothed to prevent clicks and pops.

## Mathematical Foundation

### Decibels (dB)

The **decibel** is a logarithmic unit that measures ratios. In audio, we use decibels to express voltage (amplitude) ratios:

$$
\text{dB} = 20 \log_{10}\left(\frac{V_{out}}{V_{in}}\right)
$$

Rearranging to find the linear gain from a dB value:

$$
G_{linear} = 10^{\frac{G_{dB}}{20}}
$$

### Why Logarithmic?

Human hearing is **approximately logarithmic** in its perception of loudness. A 10 dB increase sounds roughly "twice as loud" regardless of the starting level. This makes dB a perceptually meaningful unit:

| dB Change | Perceived Effect | Linear Multiplier |
|-----------|-----------------|-------------------|
| +20 dB | 4× louder | 10.0 |
| +10 dB | 2× louder | 3.16 |
| +6 dB | Noticeably louder | 2.0 |
| +3 dB | Just noticeably louder | 1.41 |
| 0 dB | No change | 1.0 |
| -3 dB | Just noticeably quieter | 0.71 |
| -6 dB | Noticeably quieter | 0.5 |
| -10 dB | Half as loud | 0.316 |
| -20 dB | Quarter as loud | 0.1 |

### Amplitude vs Power

Audio engineers use two conventions:
- **Amplitude/Voltage**: $\text{dB} = 20 \log_{10}(ratio)$
- **Power/Intensity**: $\text{dB} = 10 \log_{10}(ratio)$

Since power is proportional to amplitude squared ($P \propto V^2$):
$$
20 \log_{10}(V) = 10 \log_{10}(V^2) = 10 \log_{10}(P)
$$

This block uses the **amplitude** convention.

### Common Reference Points

| dB | Linear | Relationship |
|----|--------|--------------|
| +30 dB | 31.6 | Maximum boost in this block |
| +6 dB | 2.0 | Double amplitude |
| +3 dB | 1.414 | Double power |
| 0 dB | 1.0 | Unity gain (no change) |
| -3 dB | 0.707 | Half power |
| -6 dB | 0.5 | Half amplitude |
| -20 dB | 0.1 | One-tenth amplitude |
| -40 dB | 0.01 | One-hundredth amplitude |
| -60 dB | 0.001 | Roughly noise floor |
| -80 dB | 0.0001 | Minimum in this block (near silence) |

### Parameter Smoothing

Abrupt gain changes cause **clicks**—audible discontinuities at the transition point. The block uses linear interpolation (smoothing) to ramp between gain values over several samples:

$$
G[n] = G[n-1] + \frac{G_{target} - G[n-1]}{\tau}
$$

where $\tau$ is the smoothing time constant.

## Creating a Gain Block

```rust
use bbx_dsp::{blocks::GainBlock, graph::GraphBuilder};

let mut builder = GraphBuilder::<f32>::new(44100.0, 512, 2);

// Level in dB, optional base gain multiplier
let gain = builder.add(GainBlock::new(-6.0, None));

// With base gain multiplier (applied statically)
let gain = builder.add(GainBlock::new(0.0, Some(0.5)));  // 0 dB + 50% = -6 dB effective
```

## Port Layout

| Port | Direction | Description |
|------|-----------|-------------|
| 0 | Input | Audio input |
| 0 | Output | Scaled audio output |

## Parameters

| Parameter | Type | Range | Default | Description |
|-----------|------|-------|---------|-------------|
| level_db | f64 | -80.0 to +30.0 dB | 0.0 | Gain level in decibels |
| base_gain | f64 | 0.0 to 1.0+ | 1.0 | Additional linear multiplier |

### Level Values

| dB | Linear | Effect |
|----|--------|--------|
| +12 | 4.0 | 4x louder |
| +6 | 2.0 | 2x louder |
| +3 | 1.41 | ~1.4x louder |
| 0 | 1.0 | No change |
| -3 | 0.71 | ~0.7x amplitude |
| -6 | 0.5 | Half amplitude |
| -12 | 0.25 | Quarter amplitude |
| -20 | 0.1 | 10% amplitude |
| -80 | ~0.0 | Near silence |

## Usage Examples

### Volume Control

```rust
use bbx_dsp::{blocks::{GainBlock, OscillatorBlock}, graph::GraphBuilder, waveform::Waveform};

let mut builder = GraphBuilder::<f32>::new(44100.0, 512, 2);

let source = builder.add(OscillatorBlock::new(440.0, Waveform::Sine, None));
let volume = builder.add(GainBlock::new(-12.0, None));  // -12 dB

builder.connect(source, 0, volume, 0);
```

### Unity Gain (Passthrough)

```rust
use bbx_dsp::blocks::GainBlock;

// Unity gain - useful as a mixing point or modulation target
let gain = GainBlock::<f32>::unity();
```

### Gain Staging

```rust
use bbx_dsp::{blocks::{GainBlock, OscillatorBlock, OverdriveBlock}, graph::GraphBuilder, waveform::Waveform};

let mut builder = GraphBuilder::<f32>::new(44100.0, 512, 2);

let source = builder.add(OscillatorBlock::new(440.0, Waveform::Sine, None));

// Input gain before processing
let input_gain = builder.add(GainBlock::new(6.0, None));

// Processing
let effect = builder.add(OverdriveBlock::new(3.0, 1.0, 0.5, 44100.0));

// Output gain after processing
let output_gain = builder.add(GainBlock::new(-6.0, None));

builder.connect(source, 0, input_gain, 0);
builder.connect(input_gain, 0, effect, 0);
builder.connect(effect, 0, output_gain, 0);
```

### Modulated Gain (Tremolo)

```rust
use bbx_dsp::{blocks::{GainBlock, LfoBlock, OscillatorBlock}, graph::GraphBuilder, waveform::Waveform};

let mut builder = GraphBuilder::<f32>::new(44100.0, 512, 2);

let osc = builder.add(OscillatorBlock::new(440.0, Waveform::Sine, None));
let lfo = builder.add(LfoBlock::new(4.0, 0.5, Waveform::Sine, None));  // 4 Hz tremolo
let gain = builder.add(GainBlock::new(0.0, None));

builder.connect(osc, 0, gain, 0);
builder.modulate(lfo, gain, "level_db");
```

### Base Gain for Fixed Scaling

```rust
use bbx_dsp::{blocks::GainBlock, graph::GraphBuilder};

let mut builder = GraphBuilder::<f32>::new(44100.0, 512, 2);

// Base gain of 0.5 applied statically (multiplied with dB gain)
let gain = builder.add(GainBlock::new(0.0, Some(0.5)));
// Effective: 0 dB * 0.5 = -6.02 dB
```

## Implementation Notes

- dB range: -80 dB to +30 dB (clamped)
- Click-free transitions via linear smoothing
- Multi-channel support (applies same gain to all channels)
- SIMD-optimized when not smoothing
- Base gain multiplied with dB-derived gain

## Further Reading

- Sethares, W. (2005). *Tuning, Timbre, Spectrum, Scale*, Appendix: Decibels. Springer.
- Self, D. (2009). *Audio Power Amplifier Design Handbook*, Chapter 1: Gain Control. Focal Press.
- Pohlmann, K. (2010). *Principles of Digital Audio*, Chapter 2: Sound and Hearing. McGraw-Hill.
