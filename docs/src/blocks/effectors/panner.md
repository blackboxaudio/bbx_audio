# PannerBlock

Multi-format spatial panning supporting stereo, surround, and ambisonics.

## Overview

`PannerBlock` positions a mono signal in 2D or 3D space using three different algorithms:

- **Stereo**: Constant-power panning between left and right
- **Surround**: VBAP (Vector Base Amplitude Panning) for 5.1/7.1 layouts
- **Ambisonic**: Spherical harmonic encoding for immersive audio

## Mathematical Foundation

### Coordinate System

For surround and ambisonic modes, positions are specified in spherical coordinates:

- **Azimuth** $\theta$: Horizontal angle from front
  - $0°$ = front
  - $90°$ = left
  - $-90°$ = right
  - $±180°$ = rear
- **Elevation** $\phi$: Vertical angle from horizon
  - $0°$ = horizon
  - $90°$ = directly above
  - $-90°$ = directly below

### Stereo: Constant-Power Pan Law

Simple left/right panning uses a **constant-power pan law** based on sine and cosine functions. This preserves perceived loudness as the sound moves across the stereo field.

For a pan position $p \in [-100, 100]$:

1. Normalize to $[0, 1]$:
$$
t = \frac{p + 100}{200}
$$

2. Convert to angle:
$$
\alpha = t \cdot \frac{\pi}{2}
$$

3. Calculate gains:
$$
g_L = \cos(\alpha), \quad g_R = \sin(\alpha)
$$

#### Why Constant-Power?

The key insight is that perceived loudness relates to **power**, not amplitude. When a sound is centered, it plays equally from both speakers, and the listener perceives both contributions combined.

**Linear panning** (using $g_L = 1-t$ and $g_R = t$) causes a loudness dip in the center because:
$$
g_L^2 + g_R^2 = (1-t)^2 + t^2 \neq 1 \quad \text{(varies from 0.5 to 1)}
$$

**Constant-power panning** maintains consistent loudness because:
$$
g_L^2 + g_R^2 = \cos^2(\alpha) + \sin^2(\alpha) = 1 \quad \text{(always)}
$$

| Position | $t$ | $\alpha$ | $g_L$ | $g_R$ | $g_L^2 + g_R^2$ |
|----------|-----|----------|-------|-------|-----------------|
| Full left (-100) | 0.0 | 0° | 1.0 | 0.0 | 1.0 |
| Center (0) | 0.5 | 45° | 0.707 | 0.707 | 1.0 |
| Full right (+100) | 1.0 | 90° | 0.0 | 1.0 | 1.0 |

### Surround: Vector Base Amplitude Panning (VBAP)

**VBAP** generalizes panning to arbitrary speaker layouts by computing gains based on angular distance from each speaker.

#### Speaker Positions

Standard speaker layouts define specific azimuths:

**5.1 (ITU-R BS.775-1):**
| Channel | Name | Azimuth |
|---------|------|---------|
| 0 | L (Left) | 30° |
| 1 | R (Right) | -30° |
| 2 | C (Center) | 0° |
| 3 | LFE | — (omnidirectional) |
| 4 | Ls (Left Surround) | 110° |
| 5 | Rs (Right Surround) | -110° |

**7.1 (ITU-R BS.2051):**
| Channel | Name | Azimuth |
|---------|------|---------|
| 0 | L (Left) | 30° |
| 1 | R (Right) | -30° |
| 2 | C (Center) | 0° |
| 3 | LFE | — |
| 4 | Ls (Left Side) | 90° |
| 5 | Rs (Right Side) | -90° |
| 6 | Lrs (Left Rear) | 150° |
| 7 | Rrs (Right Rear) | -150° |

#### Gain Calculation

For a source at azimuth $\theta_s$ and speaker $i$ at azimuth $\theta_i$:

1. Calculate angular difference (handling wrap-around):
$$
\Delta\theta_i = \min(|\theta_s - \theta_i|, 2\pi - |\theta_s - \theta_i|)
$$

2. Apply gain based on a **spread angle** $\sigma$ (typically $\frac{\pi}{2}$):
$$
g_i = \begin{cases}
\cos\left(\frac{\sigma - \Delta\theta_i}{\sigma} \cdot \frac{\pi}{2}\right) & \text{if } \Delta\theta_i < \sigma \\
0 & \text{otherwise}
\end{cases}
$$

3. Normalize for constant energy:
$$
g_i' = \frac{g_i}{\sqrt{\sum_j g_j^2}}
$$

This ensures that the total power remains constant regardless of source position.

### Ambisonics: Spherical Harmonic Encoding

**Ambisonics** represents sound fields using **spherical harmonics**—mathematical functions that describe how sound varies with direction. Unlike channel-based formats, ambisonics is speaker-independent: the encoded signal can be decoded to any speaker layout.

#### Spherical Harmonics

Spherical harmonics $Y_l^m(\theta, \phi)$ form an orthonormal basis for functions on the sphere, where:
- $l$ is the **order** (0, 1, 2, 3...)
- $m$ is the **degree** ($-l \leq m \leq l$)

Higher orders capture finer spatial detail but require more channels:
- Order 0: 1 channel (omnidirectional)
- Order 1: 4 channels (directional)
- Order 2: 9 channels (improved localization)
- Order 3: 16 channels (high spatial resolution)

The channel count for order $L$ is $(L+1)^2$.

#### SN3D Normalization and ACN Ordering

This implementation uses:
- **SN3D** (Semi-Normalized 3D): Schmidt semi-normalization for consistent gain
- **ACN** (Ambisonic Channel Number): Standard channel ordering

#### Encoding Coefficients

For a source at azimuth $\theta$ and elevation $\phi$:

**Order 0 (omnidirectional):**
$$
Y_0^0 = 1 \quad \text{(W channel)}
$$

**Order 1 (first-order, directional):**
$$
\begin{aligned}
Y_1^{-1} &= \cos\phi \sin\theta \quad \text{(Y channel)} \\
Y_1^0 &= \sin\phi \quad \text{(Z channel)} \\
Y_1^1 &= \cos\phi \cos\theta \quad \text{(X channel)}
\end{aligned}
$$

**Order 2:**
$$
\begin{aligned}
Y_2^{-2} &= \sqrt{\frac{3}{4}} \cos^2\phi \sin(2\theta) \quad \text{(V channel)} \\
Y_2^{-1} &= \sqrt{\frac{3}{4}} \sin(2\phi) \sin\theta \quad \text{(T channel)} \\
Y_2^0 &= \frac{3\sin^2\phi - 1}{2} \quad \text{(R channel)} \\
Y_2^1 &= \sqrt{\frac{3}{4}} \sin(2\phi) \cos\theta \quad \text{(S channel)} \\
Y_2^2 &= \sqrt{\frac{3}{4}} \cos^2\phi \cos(2\theta) \quad \text{(U channel)}
\end{aligned}
$$

**Order 3** follows similar patterns with $\cos(3\theta)$, $\sin(3\theta)$, and higher powers of $\cos\phi$ and $\sin\phi$.

#### Intuition for Spherical Harmonics

- **W (order 0)**: Captures overall level—same in all directions
- **X, Y, Z (order 1)**: Capture front-back, left-right, and up-down gradients
- **Higher orders**: Capture increasingly fine directional detail

A source at the front ($\theta=0, \phi=0$) encodes as:
- $W = 1$ (present everywhere)
- $Y = 0$ (no left-right component)
- $Z = 0$ (on horizon)
- $X = 1$ (fully in front)

## Panning Modes

### Stereo Mode

Traditional left-right panning using constant-power pan law.

```rust
use bbx_dsp::{blocks::PannerBlock, graph::GraphBuilder};

let mut builder = GraphBuilder::<f32>::new(44100.0, 512, 2);

// Position: -100 (left) to +100 (right)
let pan = builder.add(PannerBlock::new(0.0));  // Center
let pan = builder.add(PannerBlock::new_stereo(-50.0));  // Half left
```

### Surround Mode (VBAP)

Uses Vector Base Amplitude Panning for 5.1 and 7.1 speaker layouts.

```rust
use bbx_dsp::{blocks::PannerBlock, channel::ChannelLayout, graph::GraphBuilder};

let mut builder = GraphBuilder::<f32>::new(44100.0, 512, 6);

// 5.1 surround panner
let pan = builder.add(PannerBlock::new_surround(ChannelLayout::Surround51));

// 7.1 surround panner
let pan = builder.add(PannerBlock::new_surround(ChannelLayout::Surround71));
```

Control source position with `azimuth` and `elevation` parameters.

### Ambisonic Mode

Encodes mono input to SN3D normalized, ACN ordered B-format for immersive audio.

```rust
use bbx_dsp::{blocks::PannerBlock, graph::GraphBuilder};

let mut builder = GraphBuilder::<f32>::new(44100.0, 512, 4);

// First-order ambisonics (4 channels)
let pan = builder.add(PannerBlock::new_ambisonic(1));

// Second-order ambisonics (9 channels)
let pan = builder.add(PannerBlock::new_ambisonic(2));

// Third-order ambisonics (16 channels)
let pan = builder.add(PannerBlock::new_ambisonic(3));
```

## Port Layout

Port counts vary by mode:

| Mode | Inputs | Outputs |
|------|--------|---------|
| Stereo | 1 | 2 |
| Surround 5.1 | 1 | 6 |
| Surround 7.1 | 1 | 8 |
| Ambisonic FOA | 1 | 4 |
| Ambisonic SOA | 1 | 9 |
| Ambisonic TOA | 1 | 16 |

## Parameters

| Parameter | Type | Range | Mode | Default |
|-----------|------|-------|------|---------|
| position | f32 | -100.0 to 100.0 | Stereo | 0.0 |
| azimuth | f32 | -180.0 to 180.0 | Surround, Ambisonic | 0.0 |
| elevation | f32 | -90.0 to 90.0 | Surround, Ambisonic | 0.0 |

### Position Values (Stereo)

| Value | Position |
|-------|----------|
| -100.0 | Hard left |
| -50.0 | Half left |
| 0.0 | Center |
| +50.0 | Half right |
| +100.0 | Hard right |

### Azimuth/Elevation (Surround/Ambisonic)

| Azimuth | Direction |
|---------|-----------|
| 0 | Front |
| 90 | Left |
| -90 | Right |
| 180 / -180 | Rear |

| Elevation | Direction |
|-----------|-----------|
| 0 | Horizon |
| 90 | Above |
| -90 | Below |

## Usage Examples

### Basic Stereo Panning

```rust
use bbx_dsp::{
    blocks::{OscillatorBlock, PannerBlock},
    graph::GraphBuilder,
    waveform::Waveform,
};

let mut builder = GraphBuilder::<f32>::new(44100.0, 512, 2);

let osc = builder.add(OscillatorBlock::new(440.0, Waveform::Sine, None));
let pan = builder.add(PannerBlock::new(50.0));  // Slightly right

builder.connect(osc, 0, pan, 0);
```

### Auto-Pan with LFO

```rust
use bbx_dsp::{
    blocks::{LfoBlock, OscillatorBlock, PannerBlock},
    graph::GraphBuilder,
    waveform::Waveform,
};

let mut builder = GraphBuilder::<f32>::new(44100.0, 512, 2);

let osc = builder.add(OscillatorBlock::new(440.0, Waveform::Sine, None));
let lfo = builder.add(LfoBlock::new(0.25, 1.0, Waveform::Sine, None));  // 0.25 Hz, full depth
let pan = builder.add(PannerBlock::new(0.0));

builder.connect(osc, 0, pan, 0);
builder.modulate(lfo, pan, "position");
```

### Surround Panning with VBAP

```rust
use bbx_dsp::{blocks::{LfoBlock, OscillatorBlock, PannerBlock}, channel::ChannelLayout, graph::GraphBuilder, waveform::Waveform};

let mut builder = GraphBuilder::<f32>::new(44100.0, 512, 6);

let osc = builder.add(OscillatorBlock::new(440.0, Waveform::Sine, None));
let pan = builder.add(PannerBlock::new_surround(ChannelLayout::Surround51));

builder.connect(osc, 0, pan, 0);

// Modulate azimuth for circular motion
let lfo = builder.add(LfoBlock::new(0.1, 1.0, Waveform::Sine, None));
builder.modulate(lfo, pan, "azimuth");
```

### Ambisonic Encoding with Rotating Source

```rust
use bbx_dsp::{blocks::{LfoBlock, OscillatorBlock, PannerBlock}, channel::ChannelLayout, graph::GraphBuilder, waveform::Waveform};

let mut builder = GraphBuilder::<f32>::new(44100.0, 512, 4);

let osc = builder.add(OscillatorBlock::new(440.0, Waveform::Sine, None));
let encoder = builder.add(PannerBlock::new_ambisonic(1));

builder.connect(osc, 0, encoder, 0);

// Rotate source around listener
let az_lfo = builder.add(LfoBlock::new(0.2, 1.0, Waveform::Sine, None));
builder.modulate(az_lfo, encoder, "azimuth");

// Output channels: W, Y, Z, X (ACN order)
```

### Full Ambisonics Pipeline

```rust
use bbx_dsp::{blocks::{AmbisonicDecoderBlock, OscillatorBlock, PannerBlock}, channel::ChannelLayout, graph::GraphBuilder, waveform::Waveform};

let mut builder = GraphBuilder::<f32>::new(44100.0, 512, 2);

// Source
let osc = builder.add(OscillatorBlock::new(440.0, Waveform::Sine, None));

// Encode to FOA
let encoder = builder.add(PannerBlock::new_ambisonic(1));
builder.connect(osc, 0, encoder, 0);

// Decode to stereo for headphones
let decoder = builder.add(AmbisonicDecoderBlock::new(1, ChannelLayout::Stereo));
builder.connect(encoder, 0, decoder, 0);  // W
builder.connect(encoder, 1, decoder, 1);  // Y
builder.connect(encoder, 2, decoder, 2);  // Z
builder.connect(encoder, 3, decoder, 3);  // X
```

## Configuring Smoothing

By default, `position`, `azimuth`, and `elevation` parameter changes are smoothed over 50ms to prevent clicks. You can customize the smoothing time using `set_smoothing()`:

```rust
use bbx_dsp::{blocks::PannerBlock, graph::GraphBuilder};

let mut builder = GraphBuilder::<f32>::new(44100.0, 512, 2);
let pan_id = builder.add(PannerBlock::new(0.0));

let mut graph = builder.build();
if let Some(block) = graph.get_block_mut(pan_id) {
    block.set_smoothing(44100.0, 100.0); // 100ms ramp time
}
```

**Note:** `set_smoothing()` applies the same ramp time to all spatial parameters (`position`, `azimuth`, `elevation`). If you need different smoothing times per parameter, create and configure the block before passing it to `GraphBuilder::add()`.

## Implementation Notes

- Click-free panning via linear smoothing on all parameters (default 50ms ramp)
- Uses `ChannelConfig::Explicit` (handles routing internally)
- All modes accept mono input (1 channel)
- VBAP normalizes gains for energy preservation
- Ambisonic encoding uses SN3D normalization and ACN channel ordering

## Further Reading

- Pulkki, V. (1997). "Virtual Sound Source Positioning Using Vector Base Amplitude Panning." *JAES*, 45(6), 456-466.
- Zotter, F. & Frank, M. (2019). *Ambisonics: A Practical 3D Audio Theory for Recording, Studio Production, Sound Reinforcement, and Virtual Reality*. Springer.
- Daniel, J. (2001). *Représentation de champs acoustiques, application à la transmission et à la reproduction de scènes sonores complexes dans un contexte multimédia*. PhD thesis.
- Chapman, M. et al. (2009). "A Standard for Interchange of Ambisonic Signal Sets." *Ambisonics Symposium*.
