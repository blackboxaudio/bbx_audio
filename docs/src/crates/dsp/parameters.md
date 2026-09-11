# Parameter System

The bbx_dsp parameter system supports static values and summed, depth-scaled modulation.

## Parameter Type

```rust
pub struct Parameter<S: Sample> {
    base: S,
    routes: StackVec<ModulationRoute<S>, 4>,
}
```

Value during processing: `base + Σ depth · source`. See
[Parameter\<S\> Type](../../architecture/parameter-type.md) for the full API.

## Constant Parameters

A parameter with no routes is a constant:

```rust
use bbx_dsp::parameter::Parameter;

let gain = Parameter::constant(-6.0_f32);
let frequency: Parameter<f32> = 440.0.into();
```

## Modulated Parameters

Attach routes with `GraphBuilder::modulate()`. It returns `Result<&mut Self, GraphError>`:
an unknown parameter name, a missing block, a source output that does not exist, or a
parameter that already holds its maximum number of routes is an error at build time,
never a silent no-op.

```rust
use bbx_dsp::{blocks::{LfoBlock, OscillatorBlock}, graph::GraphBuilder, waveform::Waveform};

let mut builder = GraphBuilder::<f32>::new(44100.0, 512, 2);

// Create an LFO (frequency, depth, waveform, seed)
let lfo = builder.add(LfoBlock::new(5.0, 0.3, Waveform::Sine, None));

// Create an oscillator
let osc = builder.add(OscillatorBlock::new(440.0, Waveform::Sine, None));

// Route the LFO's first output into the oscillator frequency at unity depth
builder.modulate(lfo, osc, "frequency")?;
```

Parameter names are case-insensitive and blocks accept aliases (`"level"` for
`"level_db"`, `"q"` for `"resonance"`, `"pan"` for `"position"`).

### Depth and Multiple Sources

`modulate_with` takes a full `ModulationRoute`, which chooses the source output and the
depth. Routes into the same parameter sum:

```rust
use bbx_dsp::parameter::{ModulationRoute, ModulationSource};

let slow = builder.add(LfoBlock::new(0.5, 1.0, Waveform::Sine, None));
let fast = builder.add(LfoBlock::new(6.0, 1.0, Waveform::Sine, None));
let filter = builder.add(LowPassFilterBlock::new(1000.0, 0.707));

builder.modulate_with(ModulationRoute::new(ModulationSource::first_output(slow), 800.0), filter, "cutoff")?;
builder.modulate_with(ModulationRoute::new(ModulationSource::first_output(fast), 100.0), filter, "cutoff")?;
// cutoff = 1000 + 800·slow + 100·fast
```

Each parameter holds up to `MAX_MODULATION_ROUTES` (4) routes.

## Modulation Flow

1. Modulator blocks (LFO, Envelope) render their outputs
2. After each modulator runs, the graph copies the first sample of **every** modulation output into a flat table
3. Target blocks receive that table as a `ModulationValues` view and call `parameter.value(...)`

```rust
fn process(
    &mut self,
    inputs: &[&[S]],
    outputs: &mut [&mut [S]],
    modulation_values: &ModulationValues<S>,
    context: &DspContext,
) {
    let cutoff = self.cutoff.value(modulation_values);
    // ...
}
```

Outside a graph, pass `ModulationValues::empty()`.

## Units

Depth is in the parameter's own units, because the modulated value is `base + depth · source`
and a sine LFO spans −1 to 1:

| Target | Units | Example |
|--------|-------|---------|
| `OscillatorBlock::frequency` | Hz | depth 5 → ±5 Hz vibrato |
| `OscillatorBlock::pitch_offset` | semitones | depth 0.5 → ±50 cents |
| `GainBlock::level_db` | dB | depth 3 → ±3 dB tremolo |
| `LowPassFilterBlock::cutoff` | Hz | depth 500 → ±500 Hz sweep |
| `PannerBlock::position` | −100..100 | depth 100 → full-width auto-pan |

`LfoBlock` still has its own `depth` parameter for compatibility; it multiplies the
output before routing. Prefer leaving it at 1.0 and setting depth on the route.

### Bipolar vs Unipolar

Sine, triangle and square LFOs are bipolar (−1 to 1); envelopes are unipolar (0 to 1). A
route does not change polarity; pick the base value accordingly. For a unipolar sweep from
an LFO, offset the base by the depth.

## Example: Tremolo

```rust
use bbx_dsp::{blocks::{GainBlock, LfoBlock, OscillatorBlock}, graph::GraphBuilder, waveform::Waveform};

let mut builder = GraphBuilder::<f32>::new(44100.0, 512, 2);

// Audio source
let osc = builder.add(OscillatorBlock::new(440.0, Waveform::Sine, None));

// Tremolo LFO (6 Hz)
let lfo = builder.add(LfoBlock::new(6.0, 1.0, Waveform::Sine, None));

// Gain block
let gain = builder.add(GainBlock::new(-6.0, None));
builder.connect(osc, 0, gain, 0);

// ±6 dB around the base level
builder.modulate_with(ModulationRoute::new(ModulationSource::first_output(lfo), 6.0), gain, "level")?;

let graph = builder.build();
```

## Example: Vibrato

```rust
use bbx_dsp::{blocks::{LfoBlock, OscillatorBlock}, graph::GraphBuilder, waveform::Waveform};

let mut builder = GraphBuilder::<f32>::new(44100.0, 512, 2);

// Vibrato LFO (5 Hz)
let lfo = builder.add(LfoBlock::new(5.0, 1.0, Waveform::Sine, None));

// Oscillator
let osc = builder.add(OscillatorBlock::new(440.0, Waveform::Sine, None));

// ±0.5 semitone
builder.modulate_with(ModulationRoute::new(ModulationSource::first_output(lfo), 0.5), osc, "pitch_offset")?;

let graph = builder.build();
```
