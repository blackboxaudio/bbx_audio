# Parameter System

The bbx_dsp parameter system provides smoothed parameters with support for both constant values and modulation.

## Parameter Type

```rust
pub struct Parameter<S: Sample, T: SmoothingStrategy = Linear> {
    source: ParameterSource<S>,
    smoother: SmoothedValue<S, T>,
    ramp_length_ms: f64,
}

pub enum ParameterSource<S: Sample> {
    Constant(S),
    Modulated(BlockId),
}
```

The `Parameter` type combines a value source (constant or modulated) with built-in smoothing for click-free parameter changes.

## Creating Parameters

### Constant Parameters

Parameters with fixed values:

```rust
use bbx_dsp::parameter::Parameter;

let gain = Parameter::<f32>::constant(0.5);
let frequency = Parameter::<f32>::constant(440.0);

// With custom ramp time (default is 50ms)
let pan = Parameter::<f32>::constant(0.0).with_ramp_ms(100.0);
```

### Modulated Parameters

Parameters controlled by modulator blocks:

```rust
use bbx_dsp::{graph::GraphBuilder, waveform::Waveform};

let mut builder = GraphBuilder::<f32>::new(44100.0, 512, 2);

// Create an LFO
let lfo = builder.add_lfo(5.0, Waveform::Sine);

// Use it to modulate oscillator frequency
let osc = builder.add_oscillator(440.0, Waveform::Sine, None);
builder.modulate(lfo, osc, "frequency");
```

## Using Parameters in Blocks

Parameters handle smoothing internally. The typical usage pattern:

```rust
fn prepare(&mut self, context: &DspContext) {
    self.gain.prepare(context.sample_rate);
}

fn process(&mut self, inputs: &[&[S]], outputs: &mut [&mut [S]],
           modulation_values: &[S], context: &DspContext) {
    // Update target from modulation source
    self.gain.update_target(modulation_values);

    // Fast path when value is stable
    if !self.gain.is_smoothing() {
        let value = self.gain.current();
        // Apply constant value to all samples...
        return;
    }

    // Smoothing path: per-sample interpolation
    for i in 0..context.buffer_size {
        let value = self.gain.next_value();
        outputs[0][i] = inputs[0][i] * value;
    }
}
```

## Modulation Flow

1. Modulator blocks (LFO, Envelope) output control values
2. These values are collected during graph processing
3. Target blocks receive values in the `modulation_values` parameter
4. Parameters use `get_raw_value()` or `update_target()` to access modulation

```rust
fn process(&mut self, inputs: &[&[S]], outputs: &mut [&mut [S]],
           modulation_values: &[S], context: &DspContext) {
    // For direct use (no transform needed)
    self.depth.update_target(modulation_values);

    // For transformed values (e.g., dB to linear)
    let db = self.level_db.get_raw_value(modulation_values);
    let linear = db_to_linear(db);
    self.level_db.set_target(linear);
}
```

## Modulation Depth

Blocks interpret modulation values differently:

### Frequency Modulation

```rust
// LFO range: -1.0 to 1.0
// Depth parameter scales the effect
let base_freq = 440.0;
let mod_depth = 0.1;  // 10% range
let modulated_freq = base_freq * (1.0 + mod_value * mod_depth);
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
let unipolar = (bipolar + 1.0) * 0.5;

// Convert unipolar to bipolar
let bipolar = unipolar * 2.0 - 1.0;
```

## Example: Tremolo

```rust
use bbx_dsp::{graph::GraphBuilder, waveform::Waveform};

let mut builder = GraphBuilder::<f32>::new(44100.0, 512, 2);

// Audio source
let osc = builder.add_oscillator(440.0, Waveform::Sine, None);

// Tremolo LFO (6 Hz)
let lfo = builder.add_lfo(6.0, Waveform::Sine);

// VCA for amplitude modulation
let vca = builder.add_vca();

// Connect audio and modulation
builder.connect(osc, 0, vca, 0);
builder.modulate(lfo, vca, "amplitude");

let graph = builder.build();
```

## Example: Vibrato

```rust
use bbx_dsp::{graph::GraphBuilder, waveform::Waveform};

let mut builder = GraphBuilder::<f32>::new(44100.0, 512, 2);

// Vibrato LFO (5 Hz)
let lfo = builder.add_lfo(5.0, Waveform::Sine);

// Oscillator with frequency modulation
let osc = builder.add_oscillator(440.0, Waveform::Sine, None);
builder.modulate(lfo, osc, "frequency");

let graph = builder.build();
```
