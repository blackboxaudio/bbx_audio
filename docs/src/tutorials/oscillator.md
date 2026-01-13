# Creating a Simple Oscillator

This tutorial explores the `OscillatorBlock` and its waveform options.

> **Prior knowledge**: This tutorial builds on [Your First DSP Graph](first-graph.md), which covers `GraphBuilder` basics and block connections.

## Available Waveforms

bbx_audio provides several waveform types:

```rust
use bbx_dsp::waveform::Waveform;

let waveform = Waveform::Sine;      // Pure sine wave
let waveform = Waveform::Square;    // Square wave (50% duty cycle)
let waveform = Waveform::Saw;       // Sawtooth wave
let waveform = Waveform::Triangle;  // Triangle wave
let waveform = Waveform::Pulse;     // Pulse wave (variable duty cycle)
let waveform = Waveform::Noise;     // White noise
```

## Adding Oscillators

Add an oscillator using `GraphBuilder`:

```rust
use bbx_dsp::{
    blocks::OscillatorBlock,
    graph::GraphBuilder,
    waveform::Waveform,
};

let mut builder = GraphBuilder::<f32>::new(44100.0, 512, 2);

// 440 Hz sine wave
let sine_osc = builder.add(OscillatorBlock::new(440.0, Waveform::Sine, None));

// 220 Hz sawtooth
let saw_osc = builder.add(OscillatorBlock::new(220.0, Waveform::Saw, None));
```

## Waveform Characteristics

### Sine

Pure sinusoidal wave. No harmonics - the most "pure" tone.

**Use for**: Subharmonics, test tones, smooth modulation sources.

### Square

Equal time spent at maximum and minimum values. Contains only odd harmonics.

**Use for**: Hollow/woody tones, classic synthesizer sounds.

### Sawtooth

Ramps from minimum to maximum, then resets. Contains all harmonics.

**Use for**: Bright, buzzy sounds. Good starting point for subtractive synthesis.

### Triangle

Linear ramp up, linear ramp down. Contains only odd harmonics with steep rolloff.

**Use for**: Softer tones than square, flute-like sounds.

### Pulse

Like square, but with variable duty cycle. The pulse width affects the harmonic content.

**Use for**: Nasal, reedy sounds. Width modulation creates rich timbres.

### Noise

Random samples. Contains all frequencies equally.

**Use for**: Percussion, wind sounds, adding texture.

## Frequency Modulation

Use an LFO to modulate the oscillator frequency. For a deeper dive into modulation, see [Parameter Modulation with LFOs](modulation.md).

```rust
use bbx_dsp::{
    blocks::{LfoBlock, OscillatorBlock},
    graph::GraphBuilder,
    waveform::Waveform,
};

let mut builder = GraphBuilder::<f32>::new(44100.0, 512, 2);

// Add an LFO for vibrato (5 Hz, moderate depth)
let lfo = builder.add(LfoBlock::new(5.0, 0.3, Waveform::Sine, None));

// Create oscillator
let osc = builder.add(OscillatorBlock::new(440.0, Waveform::Sine, None));

// Connect LFO to modulate frequency
builder.modulate(lfo, osc, "frequency");

let graph = builder.build();
```

## Polyphony

Create multiple oscillators for polyphonic sounds:

```rust
use bbx_dsp::{
    blocks::{GainBlock, OscillatorBlock},
    graph::GraphBuilder,
    waveform::Waveform,
};

let mut builder = GraphBuilder::<f32>::new(44100.0, 512, 2);

// C major chord: C4, E4, G4
let c4 = builder.add(OscillatorBlock::new(261.63, Waveform::Sine, None));
let e4 = builder.add(OscillatorBlock::new(329.63, Waveform::Sine, None));
let g4 = builder.add(OscillatorBlock::new(392.00, Waveform::Sine, None));

// Mix them together with a gain block
let mixer = builder.add(GainBlock::new(-9.0, None));  // -9 dB for headroom

builder.connect(c4, 0, mixer, 0);
builder.connect(e4, 0, mixer, 0);
builder.connect(g4, 0, mixer, 0);

let graph = builder.build();
```

## Detuned Oscillators

Create a thicker sound with detuned oscillators:

```rust
use bbx_dsp::{
    blocks::{GainBlock, OscillatorBlock},
    graph::GraphBuilder,
    waveform::Waveform,
};

let mut builder = GraphBuilder::<f32>::new(44100.0, 512, 2);

let base_freq = 440.0;
let detune_cents = 7.0;  // 7 cents detune

// Calculate detuned frequencies
let detune_factor = 2.0_f32.powf(detune_cents / 1200.0);
let freq_up = base_freq * detune_factor;
let freq_down = base_freq / detune_factor;

// Three oscillators: center, up, down
let osc_center = builder.add(OscillatorBlock::new(base_freq as f64, Waveform::Saw, None));
let osc_up = builder.add(OscillatorBlock::new(freq_up as f64, Waveform::Saw, None));
let osc_down = builder.add(OscillatorBlock::new(freq_down as f64, Waveform::Saw, None));

let mixer = builder.add(GainBlock::new(-9.0, None));
builder.connect(osc_center, 0, mixer, 0);
builder.connect(osc_up, 0, mixer, 0);
builder.connect(osc_down, 0, mixer, 0);

let graph = builder.build();
```

## Next Steps

- [Adding Effects](effects.md) - Process oscillator output with effects
- [Parameter Modulation](modulation.md) - Animate parameters with LFOs
