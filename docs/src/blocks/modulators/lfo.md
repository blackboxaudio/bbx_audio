# LfoBlock

Low-frequency oscillator for parameter modulation.

## Overview

`LfoBlock` generates low-frequency control signals for modulating parameters like pitch, amplitude, filter cutoff, and panning. Unlike audio-rate oscillators, LFOs operate at **control rate** (typically < 20 Hz), producing smooth, slowly-varying signals.

## Mathematical Foundation

### What is Modulation?

**Modulation** is the process of varying one signal (the **carrier**) using another signal (the **modulator**). In synthesis:

- **Carrier**: The audio signal being modified (e.g., an oscillator)
- **Modulator**: The control signal doing the modifying (the LFO)
- **Modulation depth**: How much the modulator affects the carrier

### Phase Accumulation

Like audio oscillators, LFOs use a **phase accumulator**:

$$
\phi[n] = \phi[n-1] + \Delta\phi
$$

where the phase increment is:

$$
\Delta\phi = \frac{2\pi f_{LFO}}{f_s}
$$

The key difference is that $f_{LFO}$ is typically 0.01-20 Hz rather than 20-20000 Hz.

### Output Scaling

The raw oscillator output $w(\phi) \in [-1, 1]$ is scaled by **depth**:

$$
y[n] = d \cdot w(\phi[n])
$$

where $d$ is the depth parameter (0.0 to 1.0).

The final output range is $[-d, +d]$, centered at zero.

### Control Rate vs Audio Rate

**Audio-rate modulation** (sample-by-sample):
- Frequency: 20 Hz - 20 kHz
- Creates new frequencies (sidebands)
- Used for FM synthesis, ring modulation

**Control-rate modulation** (per-buffer):
- Frequency: 0.01 Hz - ~20 Hz
- Smoothly varies parameters
- Used for vibrato, tremolo, auto-pan

Control-rate processing is more efficient because it computes one value per buffer rather than one per sample.

### Maximum LFO Frequency

Due to control-rate operation, the maximum useful LFO frequency is limited by the Nyquist criterion for control signals:

$$
f_{max} = \frac{f_s}{2 \cdot B}
$$

where $B$ is the buffer size.

For 44.1 kHz sample rate and 512-sample buffers:
$$
f_{max} = \frac{44100}{2 \times 512} \approx 43 \text{ Hz}
$$

Above this frequency, the LFO output will alias in the control domain.

### Common Modulation Effects

| Effect | Target Parameter | Typical Rate | Typical Depth |
|--------|------------------|--------------|---------------|
| Vibrato | Pitch/Frequency | 4-7 Hz | 0.1-0.5 |
| Tremolo | Amplitude/Gain | 4-10 Hz | 0.3-1.0 |
| Auto-pan | Pan position | 0.1-1 Hz | 0.5-1.0 |
| Filter sweep | Filter cutoff | 0.05-2 Hz | 0.3-0.8 |
| Wobble bass | Filter cutoff | 1-4 Hz | 0.5-1.0 |

## Creating an LFO

```rust
use bbx_dsp::{blocks::LfoBlock, graph::GraphBuilder, waveform::Waveform};

let mut builder = GraphBuilder::<f32>::new(44100.0, 512, 2);

let lfo = builder.add(LfoBlock::new(5.0, 0.5, Waveform::Sine, None));
```

For non-sine waveforms:

```rust
use bbx_dsp::{blocks::LfoBlock, graph::GraphBuilder, waveform::Waveform};

let mut builder = GraphBuilder::<f32>::new(44100.0, 512, 2);

let lfo = builder.add(LfoBlock::new(2.0, 0.8, Waveform::Triangle, None));
```

## Port Layout

| Port | Direction | Description |
|------|-----------|-------------|
| 0 | Modulation Output | Control signal (-depth to +depth) |

## Parameters

| Parameter | Type | Range | Default | Description |
|-----------|------|-------|---------|-------------|
| frequency | f64 | 0.01 - ~43 Hz | 1.0 | Oscillation rate |
| depth | f64 | 0.0 - 1.0 | 1.0 | Output amplitude |
| waveform | Waveform | - | Sine | Shape of modulation |
| seed | Option\<u64\> | Any | None | Random seed (for Noise) |

## Waveforms

| Waveform | Character | Use Case |
|----------|-----------|----------|
| Sine | Smooth, natural | Vibrato, tremolo |
| Triangle | Linear, symmetric | Pitch wobble |
| Sawtooth | Rising ramp | Filter sweeps |
| Square | Abrupt on/off | Gated effects |
| Noise | Random | Organic variation |

## Modulation Output

The LFO output ranges from -1.0 to 1.0 (scaled by depth). The receiving block interprets this:

- **Pitch**: Maps to frequency deviation
- **Amplitude**: Maps to gain change
- **Pan**: Maps to position change

## Usage Examples

### Vibrato (Pitch Modulation)

```rust
use bbx_dsp::{blocks::{LfoBlock, OscillatorBlock}, graph::GraphBuilder, waveform::Waveform};

let mut builder = GraphBuilder::<f32>::new(44100.0, 512, 2);

let lfo = builder.add(LfoBlock::new(5.0, 0.3, Waveform::Sine, None));
let osc = builder.add(OscillatorBlock::new(440.0, Waveform::Sine, None));

builder.modulate(lfo, osc, "frequency");
```

### Tremolo (Amplitude Modulation)

```rust
use bbx_dsp::{blocks::{GainBlock, LfoBlock, OscillatorBlock}, graph::GraphBuilder, waveform::Waveform};

let mut builder = GraphBuilder::<f32>::new(44100.0, 512, 2);

let osc = builder.add(OscillatorBlock::new(440.0, Waveform::Sine, None));
let lfo = builder.add(LfoBlock::new(6.0, 1.0, Waveform::Sine, None));
let gain = builder.add(GainBlock::new(-6.0, None));

builder.connect(osc, 0, gain, 0);
builder.modulate(lfo, gain, "level_db");
```

### Auto-Pan

```rust
use bbx_dsp::{blocks::{LfoBlock, OscillatorBlock, PannerBlock}, graph::GraphBuilder, waveform::Waveform};

let mut builder = GraphBuilder::<f32>::new(44100.0, 512, 2);

let osc = builder.add(OscillatorBlock::new(440.0, Waveform::Sine, None));
let lfo = builder.add(LfoBlock::new(0.25, 1.0, Waveform::Sine, None));
let pan = builder.add(PannerBlock::new(0.0));

builder.connect(osc, 0, pan, 0);
builder.modulate(lfo, pan, "position");
```

### Filter Sweep

```rust
use bbx_dsp::{blocks::{LfoBlock, LowPassFilterBlock, OscillatorBlock}, graph::GraphBuilder, waveform::Waveform};

let mut builder = GraphBuilder::<f32>::new(44100.0, 512, 2);

let osc = builder.add(OscillatorBlock::new(440.0, Waveform::Saw, None));
let filter = builder.add(LowPassFilterBlock::new(1000.0, 4.0));
let lfo = builder.add(LfoBlock::new(0.1, 0.8, Waveform::Sine, None));

builder.connect(osc, 0, filter, 0);
builder.modulate(lfo, filter, "cutoff");
```

### Square LFO for Gated Effect

```rust
use bbx_dsp::{blocks::{GainBlock, LfoBlock, OscillatorBlock}, graph::GraphBuilder, waveform::Waveform};

let mut builder = GraphBuilder::<f32>::new(44100.0, 512, 2);

let osc = builder.add(OscillatorBlock::new(440.0, Waveform::Saw, None));
let lfo = builder.add(LfoBlock::new(4.0, 1.0, Waveform::Square, None));
let gain = builder.add(GainBlock::new(0.0, None));

builder.connect(osc, 0, gain, 0);
builder.modulate(lfo, gain, "level_db");
```

## Rate Guidelines

| Application | Rate Range | Notes |
|-------------|------------|-------|
| Vibrato | 4-7 Hz | Natural vocal/string range |
| Tremolo | 4-10 Hz | Faster = more intense |
| Auto-pan | 0.1-1 Hz | Slower = more subtle |
| Filter wobble | 1-4 Hz | Dubstep/bass music |
| Slow evolution | 0.01-0.1 Hz | Pad textures |

## Implementation Notes

- Operates at control rate (per-buffer, not per-sample)
- Phase is continuous across buffer boundaries
- Uses band-limited waveforms (PolyBLEP) to reduce aliasing
- Deterministic output when seed is provided
- SIMD-optimized for non-Noise waveforms

## Further Reading

- Roads, C. (1996). *The Computer Music Tutorial*, Chapter 5: Modulation Synthesis. MIT Press.
- Puckette, M. (2007). *Theory and Techniques of Electronic Music*, Chapter 7. World Scientific.
- Russ, M. (2012). *Sound Synthesis and Sampling*, Chapter 3: Modifiers. Focal Press.
