# Parameter System

The bbx_dsp parameter system supports static values and modulation.

## Parameter Type

```rust
pub enum Parameter<S: Sample> {
    /// Static value
    Constant(S),

    /// Modulated by a block (e.g., LFO)
    Modulated(BlockId),
}
```

## Constant Parameters

Parameters with fixed values:

```rust
use bbx_dsp::parameter::Parameter;

let gain = Parameter::Constant(-6.0_f32);
let frequency = Parameter::Constant(440.0_f32);
```

## Modulated Parameters

Parameters controlled by modulator blocks use the `GraphBuilder::modulate()` method:

```rust
use bbx_dsp::{blocks::{LfoBlock, OscillatorBlock}, graph::GraphBuilder, waveform::Waveform};

let mut builder = GraphBuilder::<f32>::new(44100.0, 512, 2);

// Create an LFO (frequency, depth, waveform, seed)
let lfo = builder.add(LfoBlock::new(5.0, 0.3, Waveform::Sine, None));

// Create an oscillator
let osc = builder.add(OscillatorBlock::new(440.0, Waveform::Sine, None));

// Modulate oscillator frequency with the LFO
builder.modulate(lfo, osc, "frequency");
```

## Modulation Flow

1. Modulator blocks (LFO, Envelope) output control values
2. These values are collected during graph processing
3. Target blocks receive values in the `modulation_values` parameter

```rust
fn process(
    &mut self,
    inputs: &[&[S]],
    outputs: &mut [&mut [S]],
    modulation_values: &[S],  // Modulation values from connected blocks
    context: &DspContext,
) {
    let mod_value = modulation_values.get(0).copied().unwrap_or(S::ZERO);
    // Use mod_value to affect processing
}
```

## Modulation Depth

Blocks interpret modulation values differently:

### Frequency Modulation

```rust
// LFO range: -1.0 to 1.0 (scaled by depth)
// This maps to frequency deviation
let mod_range = 0.1;  // +/-10% frequency change
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
use bbx_dsp::{blocks::{GainBlock, LfoBlock, OscillatorBlock}, graph::GraphBuilder, waveform::Waveform};

let mut builder = GraphBuilder::<f32>::new(44100.0, 512, 2);

// Audio source
let osc = builder.add(OscillatorBlock::new(440.0, Waveform::Sine, None));

// Tremolo LFO (6 Hz, full depth)
let lfo = builder.add(LfoBlock::new(6.0, 1.0, Waveform::Sine, None));

// Gain block
let gain = builder.add(GainBlock::new(-6.0, None));
builder.connect(osc, 0, gain, 0);

// Modulate gain level with LFO
builder.modulate(lfo, gain, "level");

let graph = builder.build();
```

## Example: Vibrato

```rust
use bbx_dsp::{blocks::{LfoBlock, OscillatorBlock}, graph::GraphBuilder, waveform::Waveform};

let mut builder = GraphBuilder::<f32>::new(44100.0, 512, 2);

// Vibrato LFO (5 Hz, moderate depth)
let lfo = builder.add(LfoBlock::new(5.0, 0.3, Waveform::Sine, None));

// Oscillator
let osc = builder.add(OscillatorBlock::new(440.0, Waveform::Sine, None));

// Modulate oscillator frequency
builder.modulate(lfo, osc, "frequency");

let graph = builder.build();
```
