# Parameter Modulation with LFOs

This tutorial covers using LFOs and envelopes to modulate block parameters.

> **Prior knowledge**: This tutorial builds on:
> - [Your First DSP Graph](first-graph.md) - GraphBuilder basics
> - [Adding Effects](effects.md) - GainBlock for tremolo examples

## Low-Frequency Oscillators (LFOs)

LFOs generate control signals for modulating parameters like pitch, volume, and filter cutoff.

```rust
use bbx_dsp::graph::GraphBuilder;

let mut builder = GraphBuilder::<f32>::new(44100.0, 512, 2);

// Create an LFO at 4 Hz with 0.5 depth
// Parameters: frequency (Hz), depth (0.0-1.0), optional seed
let lfo = builder.add_lfo(4.0, 0.5, None);
```

## Vibrato (Pitch Modulation)

Modulate oscillator frequency for vibrato using the `modulate()` method:

```rust
use bbx_dsp::{graph::GraphBuilder, waveform::Waveform};

let mut builder = GraphBuilder::<f32>::new(44100.0, 512, 2);

// LFO for vibrato (5 Hz, moderate depth)
let vibrato_lfo = builder.add_lfo(5.0, 0.3, None);

// Oscillator
let osc = builder.add_oscillator(440.0, Waveform::Sine, None);

// Connect LFO to modulate oscillator frequency
builder.modulate(vibrato_lfo, osc, "frequency");

let graph = builder.build();
```

Typical vibrato settings:
- **Rate**: 4-7 Hz
- **Depth**: 0.1-0.5 for subtle vibrato
- **Parameter**: "frequency" or "pitch_offset"

## Tremolo (Amplitude Modulation)

Modulate gain for tremolo effect:

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
let tremolo_lfo = builder.add_lfo(6.0, 1.0, None);

// Gain block
let gain = builder.add_block(BlockType::Gain(GainBlock::new(-6.0)));

// Connect oscillator to gain
builder.connect(osc, 0, gain, 0);

// Modulate gain level with LFO
builder.modulate(tremolo_lfo, gain, "level");

let graph = builder.build();
```

## LFO Parameters

| Parameter | Type | Range | Description |
|-----------|------|-------|-------------|
| frequency | f64 | 0.01 - max* | Rate in Hz |
| depth | f64 | 0.0 - 1.0 | Modulation intensity |
| seed | Option\<u64\> | Any | For deterministic output |

*Max frequency is `sample_rate / (2 * buffer_size)` due to control-rate operation (e.g., ~43 Hz at 44.1 kHz with 512-sample buffers).

## Envelope Generator

ADSR envelopes control how parameters change over time. For a practical application with VCA, see [Building a Terminal Synthesizer - Part 2](terminal-synth.md#part-2-subtractive-synth-voice).

```rust
use bbx_dsp::graph::GraphBuilder;

let mut builder = GraphBuilder::<f32>::new(44100.0, 512, 2);

// Create an ADSR envelope
// Parameters: attack, decay, sustain (level), release (all in seconds except sustain)
let envelope = builder.add_envelope(
    0.01,   // Attack: 10ms
    0.1,    // Decay: 100ms
    0.7,    // Sustain: 70%
    0.3,    // Release: 300ms
);
```

## Combining Modulation Sources

Layer multiple modulation sources:

```rust
use bbx_dsp::{
    block::BlockType,
    blocks::GainBlock,
    graph::GraphBuilder,
    waveform::Waveform,
};

let mut builder = GraphBuilder::<f32>::new(44100.0, 512, 2);

// Slow LFO for overall movement
let slow_lfo = builder.add_lfo(0.5, 0.3, None);

// Fast LFO for vibrato
let fast_lfo = builder.add_lfo(5.0, 0.2, None);

// Oscillator with vibrato
let osc = builder.add_oscillator(440.0, Waveform::Sine, None);
builder.modulate(fast_lfo, osc, "frequency");

// Gain with slow amplitude modulation
let gain = builder.add_block(BlockType::Gain(GainBlock::new(-6.0)));
builder.connect(osc, 0, gain, 0);
builder.modulate(slow_lfo, gain, "level");

let graph = builder.build();
```

## Modulation Depth

Control modulation intensity through the LFO's depth parameter:

```rust
use bbx_dsp::{graph::GraphBuilder, waveform::Waveform};

let mut builder = GraphBuilder::<f32>::new(44100.0, 512, 2);

// Subtle vibrato: low depth
let subtle_lfo = builder.add_lfo(5.0, 0.1, None);

// Intense wobble: high depth
let intense_lfo = builder.add_lfo(2.0, 1.0, None);
```

## Practical Examples

### Wobble Bass

```rust
use bbx_dsp::{
    block::BlockType,
    blocks::GainBlock,
    graph::GraphBuilder,
    waveform::Waveform,
};

let mut builder = GraphBuilder::<f32>::new(44100.0, 512, 2);

// Sub bass oscillator
let osc = builder.add_oscillator(55.0, Waveform::Saw, None);

// Slow LFO for amplitude wobble
let wobble_lfo = builder.add_lfo(2.0, 0.8, None);

// Gain block for wobble effect
let gain = builder.add_block(BlockType::Gain(GainBlock::new(-6.0)));
builder.connect(osc, 0, gain, 0);
builder.modulate(wobble_lfo, gain, "level");

let graph = builder.build();
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

// LFO for pan position (slow sweep)
let pan_lfo = builder.add_lfo(0.25, 1.0, None);

// Panner block
let pan = builder.add_block(BlockType::Panner(PannerBlock::new(0.0)));
builder.connect(osc, 0, pan, 0);

// Modulate pan position
builder.modulate(pan_lfo, pan, "position");

let graph = builder.build();
```

## Next Steps

- [Working with Audio Files](audio-files.md) - Apply modulation to file playback
- [MIDI Integration](midi.md) - Trigger envelopes with MIDI
