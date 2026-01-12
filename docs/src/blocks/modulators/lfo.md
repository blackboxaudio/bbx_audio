# LfoBlock

Low-frequency oscillator for parameter modulation.

## Overview

`LfoBlock` generates low-frequency control signals for modulating parameters like pitch, amplitude, and filter cutoff.

## Creating an LFO

```rust
use bbx_dsp::graph::GraphBuilder;

let mut builder = GraphBuilder::<f32>::new(44100.0, 512, 2);

// Parameters: frequency (Hz), depth (0.0-1.0), optional seed
let lfo = builder.add_lfo(5.0, 0.5, None);
```

## Parameters

| Parameter | Type | Range | Default |
|-----------|------|-------|---------|
| frequency | f64 | 0.01 - max* | 1.0 |
| depth | f64 | 0.0 - 1.0 | 1.0 |
| seed | Option\<u64\> | Any | None |

*Max frequency is `sample_rate / (2 * buffer_size)` due to control-rate operation (~43 Hz at 44.1 kHz with 512-sample buffers).

## Port Layout

| Port | Direction | Description |
|------|-----------|-------------|
| 0 | Modulation Output | Control signal |

## Modulation Output

The LFO output ranges from -1.0 to 1.0 (scaled by depth). The receiving block interprets this:

- **Pitch**: Maps to frequency deviation
- **Amplitude**: Maps to gain change
- **Pan**: Maps to position change

## Usage Examples

### Vibrato (Pitch Modulation)

```rust
use bbx_dsp::{graph::GraphBuilder, waveform::Waveform};

let mut builder = GraphBuilder::<f32>::new(44100.0, 512, 2);

// LFO for vibrato (5 Hz, moderate depth)
let lfo = builder.add_lfo(5.0, 0.3, None);

// Oscillator
let osc = builder.add_oscillator(440.0, Waveform::Sine, None);

// Modulate oscillator frequency
builder.modulate(lfo, osc, "frequency");
```

### Tremolo (Amplitude Modulation)

```rust
use bbx_dsp::{
    block::BlockType,
    blocks::GainBlock,
    graph::GraphBuilder,
    waveform::Waveform,
};

let mut builder = GraphBuilder::<f32>::new(44100.0, 512, 2);

let osc = builder.add_oscillator(440.0, Waveform::Sine, None);

// LFO for tremolo (6 Hz, full depth)
let lfo = builder.add_lfo(6.0, 1.0, None);

// Gain block
let gain = builder.add_block(BlockType::Gain(GainBlock::new(-6.0, None)));
builder.connect(osc, 0, gain, 0);

// Modulate gain level
builder.modulate(lfo, gain, "level");
```

### Auto-Pan

```rust
use bbx_dsp::{
    block::BlockType,
    blocks::PannerBlock,
    graph::GraphBuilder,
    waveform::Waveform,
};

let mut builder = GraphBuilder::<f32>::new(44100.0, 512, 2);

let osc = builder.add_oscillator(440.0, Waveform::Sine, None);

// Slow LFO for pan sweep
let lfo = builder.add_lfo(0.25, 1.0, None);

// Panner
let pan = builder.add_block(BlockType::Panner(PannerBlock::new(0.0)));
builder.connect(osc, 0, pan, 0);

// Modulate pan position
builder.modulate(lfo, pan, "position");
```

## Rate Guidelines

| Application | Typical Rate |
|-------------|--------------|
| Vibrato | 4-7 Hz |
| Tremolo | 4-10 Hz |
| Auto-pan | 0.1-1 Hz |
| Wobble bass | 1-4 Hz |
| Sweep | 0.05-0.5 Hz |

## Implementation Notes

- Generates per-block (control-rate, not per-sample)
- Phase is continuous across blocks
- Waveform is fixed to Sine
- Uses deterministic random when seed is provided
