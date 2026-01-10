# VcaBlock

Voltage Controlled Amplifier - multiplies audio by a control signal.

## Overview

`VcaBlock` multiplies an audio signal by a control signal sample-by-sample. This is the standard way to apply envelope control to audio in synthesizers.

## Creating a VCA Block

```rust
use bbx_dsp::graph::GraphBuilder;

let mut builder = GraphBuilder::<f32>::new(44100.0, 512, 2);

let vca = builder.add_vca();
```

## Port Layout

| Port | Direction | Description |
|------|-----------|-------------|
| 0 | Input | Audio signal |
| 1 | Input | Control signal (0.0 to 1.0) |
| 0 | Output | Modulated audio |

## Parameters

VcaBlock has no parameters - it simply multiplies its two inputs together.

## Usage Examples

### Envelope-Controlled Amplitude

The most common use case: apply an ADSR envelope to an oscillator.

```rust
use bbx_dsp::{
    graph::GraphBuilder,
    waveform::Waveform,
};

let mut builder = GraphBuilder::<f32>::new(44100.0, 512, 2);

// Signal chain: Oscillator → VCA ← Envelope
let osc = builder.add_oscillator(440.0, Waveform::Sawtooth, None);
let env = builder.add_envelope(0.01, 0.1, 0.7, 0.3);
let vca = builder.add_vca();

// Audio to VCA input 0
builder.connect(osc, 0, vca, 0);

// Envelope to VCA input 1 (control)
builder.connect(env, 0, vca, 1);
```

### LFO Tremolo

Use an LFO for amplitude modulation (tremolo effect):

```rust
use bbx_dsp::{
    graph::GraphBuilder,
    waveform::Waveform,
};

let mut builder = GraphBuilder::<f32>::new(44100.0, 512, 2);

let osc = builder.add_oscillator(440.0, Waveform::Sine, None);
let lfo = builder.add_lfo(6.0, 0.5, None);  // 6 Hz, 50% depth
let vca = builder.add_vca();

builder.connect(osc, 0, vca, 0);
builder.connect(lfo, 0, vca, 1);
```

### Subtractive Synth Voice

Complete synthesizer voice with oscillator, envelope, filter, and VCA:

```rust
use bbx_dsp::{
    graph::GraphBuilder,
    waveform::Waveform,
};

let mut builder = GraphBuilder::<f32>::new(44100.0, 512, 2);

// Sound source
let osc = builder.add_oscillator(440.0, Waveform::Sawtooth, None);

// Amplitude envelope
let env = builder.add_envelope(0.01, 0.1, 0.7, 0.3);

// VCA for envelope control
let vca = builder.add_vca();

// Filter and output gain
let filter = builder.add_low_pass_filter(2000.0, 1.5);
let gain = builder.add_gain(-6.0);

// Connect: Osc → VCA → Filter → Gain
builder
    .connect(osc, 0, vca, 0)
    .connect(env, 0, vca, 1)
    .connect(vca, 0, filter, 0)
    .connect(filter, 0, gain, 0);
```

## Behavior

- When control input is missing, defaults to 1.0 (unity gain)
- When audio input is missing, outputs silence
- Output is sample-by-sample multiplication: `output[i] = audio[i] * control[i]`

## VCA vs GainBlock

| Feature | VcaBlock | GainBlock |
|---------|----------|-----------|
| Control | Sample-by-sample from input | Fixed dB parameter |
| Use case | Envelope/LFO modulation | Static level control |
| Inputs | 2 (audio + control) | 1 (audio only) |

Use VCA for dynamic amplitude control. Use Gain for static level adjustments.

## Implementation Notes

- No internal state or smoothing
- Zero-latency processing
- Handles any buffer size
