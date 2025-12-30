# Parameter Modulation with LFOs

This tutorial covers using LFOs and envelopes to modulate block parameters.

## Low-Frequency Oscillators (LFOs)

LFOs generate control signals for modulating parameters like pitch, volume, and filter cutoff.

```rust
use bbx_dsp::{graph::GraphBuilder, waveform::Waveform};

let mut builder = GraphBuilder::<f32>::new(44100.0, 512, 2);

// Create an LFO at 4 Hz with sine waveform
let lfo = builder.add_lfo(4.0, Waveform::Sine);
```

## Vibrato (Pitch Modulation)

Modulate oscillator frequency for vibrato:

```rust
use bbx_dsp::{graph::GraphBuilder, waveform::Waveform};

let mut builder = GraphBuilder::<f32>::new(44100.0, 512, 2);

// LFO for vibrato
let vibrato_lfo = builder.add_lfo(5.0, Waveform::Sine);

// Oscillator with frequency modulation
let osc = builder.add_oscillator(440.0, Waveform::Sine, Some(vibrato_lfo));

let graph = builder.build();
```

Typical vibrato settings:
- **Rate**: 4-7 Hz
- **Waveform**: Sine or Triangle
- **Depth**: Subtle (a few cents)

## Tremolo (Amplitude Modulation)

Modulate gain for tremolo effect:

```rust
use bbx_dsp::{graph::GraphBuilder, waveform::Waveform};

let mut builder = GraphBuilder::<f32>::new(44100.0, 512, 2);

let osc = builder.add_oscillator(440.0, Waveform::Sine, None);

// LFO for tremolo
let tremolo_lfo = builder.add_lfo(6.0, Waveform::Sine);

// Gain block (LFO modulates the level)
let gain = builder.add_gain_with_modulation(-6.0, Some(tremolo_lfo));

builder.connect(osc, 0, gain, 0);

let graph = builder.build();
```

## LFO Waveforms

Different waveforms create different modulation characters:

| Waveform | Effect |
|----------|--------|
| Sine | Smooth, natural modulation |
| Triangle | Linear sweep, similar to sine |
| Square | Abrupt on/off switching |
| Saw | Ramp up, sudden reset |
| Pulse | Variable duty cycle switching |

## Envelope Generator

ADSR envelopes control how parameters change over time:

```rust
use bbx_dsp::graph::GraphBuilder;

let mut builder = GraphBuilder::<f32>::new(44100.0, 512, 2);

// Create an ADSR envelope
// Attack, Decay, Sustain (level), Release (all in seconds except sustain)
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
use bbx_dsp::{graph::GraphBuilder, waveform::Waveform};

let mut builder = GraphBuilder::<f32>::new(44100.0, 512, 2);

// Slow LFO for overall movement
let slow_lfo = builder.add_lfo(0.5, Waveform::Sine);

// Fast LFO for vibrato
let fast_lfo = builder.add_lfo(5.0, Waveform::Sine);

// Use slow LFO to modulate the fast LFO rate (complex modulation)
// Note: This requires connecting modulation outputs
```

## Modulation Depth

Control modulation intensity by adjusting the modulator's output level or the target's modulation sensitivity:

```rust
use bbx_dsp::{graph::GraphBuilder, waveform::Waveform};

let mut builder = GraphBuilder::<f32>::new(44100.0, 512, 2);

// Subtle vibrato: low-amplitude LFO
let subtle_lfo = builder.add_lfo(5.0, Waveform::Sine);

// The modulation depth is controlled by the LFO's amplitude setting
// and how the receiving block interprets the modulation signal
```

## Practical Examples

### Wobble Bass

```rust
use bbx_dsp::{graph::GraphBuilder, waveform::Waveform};

let mut builder = GraphBuilder::<f32>::new(44100.0, 512, 2);

// Sub bass oscillator
let osc = builder.add_oscillator(55.0, Waveform::Saw, None);

// Slow LFO for filter wobble (when filter is available)
let wobble_lfo = builder.add_lfo(2.0, Waveform::Sine);

// For now, use amplitude modulation
let gain = builder.add_gain_with_modulation(-6.0, Some(wobble_lfo));

builder.connect(osc, 0, gain, 0);

let graph = builder.build();
```

### Auto-Pan

```rust
use bbx_dsp::{graph::GraphBuilder, waveform::Waveform};

let mut builder = GraphBuilder::<f32>::new(44100.0, 512, 2);

let osc = builder.add_oscillator(440.0, Waveform::Sine, None);

// LFO for pan position
let pan_lfo = builder.add_lfo(0.25, Waveform::Sine);  // Slow sweep

// Panner with modulation (when supported)
let pan = builder.add_panner_with_modulation(0.0, Some(pan_lfo));

builder.connect(osc, 0, pan, 0);

let graph = builder.build();
```

## Next Steps

- [Working with Audio Files](audio-files.md) - Apply modulation to file playback
- [MIDI Integration](midi.md) - Trigger envelopes with MIDI
