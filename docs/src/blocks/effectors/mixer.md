# MixerBlock

Channel-wise audio mixing with automatic normalization.

## Overview

`MixerBlock` sums multiple audio sources into a single output while maintaining proper gain staging. It automatically groups inputs by channel and applies normalization to prevent clipping and preserve consistent loudness.

## Mathematical Foundation

### Signal Summation

When $N$ audio signals are summed, the amplitude of the combined signal can theoretically reach $N$ times the amplitude of any individual signal (if all signals happen to be in phase):

$$
y[n] = \sum_{i=0}^{N-1} x_i[n]
$$

Without normalization, mixing 4 signals could produce output up to 4× the input levels, causing clipping.

### Normalization Strategies

#### Average Normalization

Divide by the number of sources:

$$
y[n] = \frac{1}{N} \sum_{i=0}^{N-1} x_i[n]
$$

This ensures the output never exceeds the amplitude of any single input, but reduces overall level as more sources are added.

| Sources | Normalization Factor | dB Reduction |
|---------|---------------------|--------------|
| 2 | 0.5 | -6 dB |
| 3 | 0.333 | -9.5 dB |
| 4 | 0.25 | -12 dB |
| 8 | 0.125 | -18 dB |

**Use case**: When sources might constructively interfere (correlated signals, like multiple takes of the same performance).

#### Constant-Power Normalization (Default)

Divide by the square root of the number of sources:

$$
y[n] = \frac{1}{\sqrt{N}} \sum_{i=0}^{N-1} x_i[n]
$$

| Sources | Normalization Factor | dB Reduction |
|---------|---------------------|--------------|
| 2 | 0.707 | -3 dB |
| 3 | 0.577 | -4.8 dB |
| 4 | 0.5 | -6 dB |
| 8 | 0.354 | -9 dB |

**Use case**: When sources are uncorrelated (different instruments, independent oscillators).

### Why Constant-Power?

For **uncorrelated signals**, the power (not amplitude) sums:

$$
P_{total} = \sum_{i=0}^{N-1} P_i = N \cdot P_{avg}
$$

The amplitude (RMS voltage) is proportional to the square root of power:

$$
V_{RMS} = \sqrt{P} \propto \sqrt{N}
$$

To maintain the same output power regardless of how many sources are mixed, we divide by $\sqrt{N}$:

$$
\frac{N \cdot P_{avg}}{(\sqrt{N})^2} = \frac{N \cdot P_{avg}}{N} = P_{avg}
$$

This keeps the perceived loudness roughly constant whether you're mixing 2 or 8 sources.

### Correlated vs Uncorrelated Signals

**Correlated signals** (in-phase copies):
$$
x_1(t) = x_2(t) \implies (x_1 + x_2) = 2x_1
$$
Amplitude doubles, power quadruples. Use **average** normalization.

**Uncorrelated signals** (independent):
$$
\langle x_1 \cdot x_2 \rangle = 0
$$
Powers add, not amplitudes. Use **constant-power** normalization.

Real-world audio sources are typically uncorrelated, making constant-power the better default.

## Input Organization

Inputs are grouped by channel. For stereo output with 3 sources:

```
Source A:  Input 0 (L), Input 1 (R)
Source B:  Input 2 (L), Input 3 (R)
Source C:  Input 4 (L), Input 5 (R)
           ─────────────────────────
Output:    Output 0 (L = A.L + B.L + C.L)
           Output 1 (R = A.R + B.R + C.R)
```

Total inputs = `num_sources × num_channels`

## Creating a Mixer

```rust
use bbx_dsp::{blocks::MixerBlock, graph::GraphBuilder};

let mut builder = GraphBuilder::<f32>::new(44100.0, 512, 2);

// Mix 3 stereo sources into stereo output
let mixer = builder.add(MixerBlock::stereo(3));

// Or specify source and channel counts directly
let mono_mixer = builder.add(MixerBlock::new(4, 1));
```

Or with custom normalization:

```rust
use bbx_dsp::{
    blocks::{MixerBlock, NormalizationStrategy},
    graph::GraphBuilder,
};

let mut builder = GraphBuilder::<f32>::new(44100.0, 512, 2);

// Mix 2 stereo sources with average normalization
let mixer = MixerBlock::<f32>::stereo(2)
    .with_normalization(NormalizationStrategy::Average);

let mix = builder.add(mixer);
```

## Port Layout

Ports depend on configuration:

| Configuration | Inputs | Outputs |
|---------------|--------|---------|
| stereo(2) | 4 | 2 |
| stereo(3) | 6 | 2 |
| stereo(4) | 8 | 2 |
| new(3, 1) (mono) | 3 | 1 |
| new(2, 6) (5.1) | 12 | 6 |

Maximum total inputs: 16 (constrained by `MAX_BLOCK_INPUTS`)

## Parameters

| Parameter | Type | Range | Default |
|-----------|------|-------|---------|
| num_sources | usize | 1-8 | - |
| num_channels | usize | 1-16 | - |
| normalization | enum | Average, ConstantPower | ConstantPower |

## Usage Examples

### Mix Two Stereo Signals

```rust
use bbx_dsp::{blocks::{MixerBlock, OscillatorBlock, PannerBlock}, graph::GraphBuilder, waveform::Waveform};

let mut builder = GraphBuilder::<f32>::new(44100.0, 512, 2);

// Two oscillators
let osc1 = builder.add(OscillatorBlock::new(440.0, Waveform::Sine, None));
let osc2 = builder.add(OscillatorBlock::new(880.0, Waveform::Sine, None));

// Pan them to different positions
let pan1 = builder.add(PannerBlock::new_stereo(-50.0));
let pan2 = builder.add(PannerBlock::new_stereo(50.0));

builder.connect(osc1, 0, pan1, 0);
builder.connect(osc2, 0, pan2, 0);

// Mix the stereo signals together
let mixer = builder.add(MixerBlock::stereo(2));
builder
    .connect(pan1, 0, mixer, 0)  // Source A left
    .connect(pan1, 1, mixer, 1)  // Source A right
    .connect(pan2, 0, mixer, 2)  // Source B left
    .connect(pan2, 1, mixer, 3); // Source B right
```

### Mix Three Mono Sources

```rust
use bbx_dsp::{
    blocks::{MixerBlock, OscillatorBlock},
    graph::GraphBuilder,
    waveform::Waveform,
};

let mut builder = GraphBuilder::<f32>::new(44100.0, 512, 1);

let osc1 = builder.add(OscillatorBlock::new(220.0, Waveform::Sine, None));
let osc2 = builder.add(OscillatorBlock::new(330.0, Waveform::Sine, None));
let osc3 = builder.add(OscillatorBlock::new(440.0, Waveform::Sine, None));

// 3 mono sources -> 1 mono output
let mixer = builder.add(MixerBlock::new(3, 1));
builder
    .connect(osc1, 0, mixer, 0)
    .connect(osc2, 0, mixer, 1)
    .connect(osc3, 0, mixer, 2);
```

### Layered Synth

```rust
use bbx_dsp::{blocks::{MixerBlock, OscillatorBlock}, graph::GraphBuilder, waveform::Waveform};

let mut builder = GraphBuilder::<f32>::new(44100.0, 512, 2);

// Three detuned oscillators
let osc1 = builder.add(OscillatorBlock::new(440.0, Waveform::Saw, None));
let osc2 = builder.add(OscillatorBlock::new(440.0 * 1.003, Waveform::Saw, None));  // +5 cents
let osc3 = builder.add(OscillatorBlock::new(440.0 / 1.003, Waveform::Saw, None));  // -5 cents

let mixer = builder.add(MixerBlock::stereo(3));

// Mix all three (mono -> stereo via single-channel inputs)
builder.connect(osc1, 0, mixer, 0);
builder.connect(osc1, 0, mixer, 1);
builder.connect(osc2, 0, mixer, 2);
builder.connect(osc2, 0, mixer, 3);
builder.connect(osc3, 0, mixer, 4);
builder.connect(osc3, 0, mixer, 5);
```

### Custom Normalization

```rust
use bbx_dsp::{
    blocks::{MixerBlock, NormalizationStrategy},
    graph::GraphBuilder,
};

let mut builder = GraphBuilder::<f32>::new(44100.0, 512, 2);

// Use average normalization instead of constant power
let mixer = MixerBlock::<f32>::stereo(4)
    .with_normalization(NormalizationStrategy::Average);

let mix = builder.add(mixer);
```

## Implementation Notes

- Uses `ChannelConfig::Explicit` (handles channel routing internally)
- Default normalization is `ConstantPower` for natural-sounding mixes
- Panics if `num_sources` or `num_channels` is 0
- Panics if total inputs exceed `MAX_BLOCK_INPUTS` (16)
- Zero-allocation processing

## Further Reading

- Huber, D. & Runstein, R. (2017). *Modern Recording Techniques*, Chapter 3: Mixers. Focal Press.
- Katz, B. (2015). *Mastering Audio*, Chapter 4: Gain Staging. Focal Press.
- Ballou, G. (2015). *Handbook for Sound Engineers*, Chapter 24: Mixing Consoles. Focal Press.
