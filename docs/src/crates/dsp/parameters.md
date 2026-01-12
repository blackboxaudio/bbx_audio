# Parameter System

The bbx_dsp parameter system supports static values and modulation.

## Parameter Type

```rust
pub enum Parameter<S: Sample> {
    /// Static value
    Static(S),

    /// Modulated by a block (e.g., LFO)
    Modulated(BlockId),
}
```

## Static Parameters

Parameters with fixed values:

```rust
use bbx_dsp::parameter::Parameter;

let gain = Parameter::Static(-6.0_f32);
let frequency = Parameter::Static(440.0_f32);
```

## Modulated Parameters

Parameters controlled by modulator blocks:

```rust
use bbx_dsp::{graph::GraphBuilder, waveform::Waveform, parameter::Parameter};

let mut builder = GraphBuilder::<f32>::new(44100.0, 512, 2);

// Create an LFO
let lfo = builder.add_lfo(5.0, Waveform::Sine);

// Use it to modulate oscillator frequency
let osc = builder.add_oscillator(440.0, Waveform::Sine, Some(lfo));
```

## Modulation Flow

1. Modulator blocks (LFO, Envelope) output control values
2. These values are collected during graph processing
3. Target blocks receive values in the `modulation` parameter

```rust
fn process(
    &mut self,
    inputs: &[&[S]],
    outputs: &mut [&mut [S]],
    context: &DspContext,
    modulation: &[S],  // Modulation values from connected blocks
) {
    let mod_value = modulation.get(0).copied().unwrap_or(S::ZERO);
    // Use mod_value to affect processing
}
```

## Modulation Depth

Blocks interpret modulation values differently:

### Frequency Modulation

```rust
// LFO range: -1.0 to 1.0
// Modulation depth scales this
let mod_range = 0.1;  // ±10% frequency change
let modulated_freq = base_freq * (1.0 + mod_value * mod_range);
```

### Amplitude Modulation

```rust
// LFO directly scales amplitude
let modulated_amp = base_amp * (1.0 + mod_value);
```

### Bipolar vs Unipolar

Some modulators are bipolar (-1 to 1), others unipolar (0 to 1):

```rust
// Convert bipolar to unipolar
let unipolar = (bipolar + 1.0) * 0.5;  // 0.0 to 1.0

// Convert unipolar to bipolar
let bipolar = unipolar * 2.0 - 1.0;  // -1.0 to 1.0
```

## Example: Tremolo

```rust
use bbx_dsp::{graph::GraphBuilder, waveform::Waveform};

let mut builder = GraphBuilder::<f32>::new(44100.0, 512, 2);

// Audio source
let osc = builder.add_oscillator(440.0, Waveform::Sine, None);

// Tremolo LFO
let lfo = builder.add_lfo(6.0, Waveform::Sine);

// Gain with modulation
let gain = builder.add_gain_with_modulation(-6.0, Some(lfo));

builder.connect(osc, 0, gain, 0);

let graph = builder.build();
```

## Example: Vibrato

```rust
use bbx_dsp::{graph::GraphBuilder, waveform::Waveform};

let mut builder = GraphBuilder::<f32>::new(44100.0, 512, 2);

// Vibrato LFO
let lfo = builder.add_lfo(5.0, Waveform::Sine);

// Oscillator with frequency modulation
let osc = builder.add_oscillator(440.0, Waveform::Sine, Some(lfo));

let graph = builder.build();
```