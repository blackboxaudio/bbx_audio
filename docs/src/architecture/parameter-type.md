# Parameter\<S\> Type

The generic parameter type: a base value plus summed, depth-scaled modulation routes.

## Definition

```rust
pub struct Parameter<S: Sample> {
    base: S,
    routes: StackVec<ModulationRoute<S>, MAX_MODULATION_ROUTES>, // MAX_MODULATION_ROUTES = 4
}

pub struct ModulationRoute<S: Sample> {
    source: ModulationSource, // { block: BlockId, output: usize }
    depth: S,
}
```

The value read during processing is

```text
base + Σ depth_i · source_i
```

A parameter with no routes is a constant. Depth lives on the route, not in the
modulator, so one LFO can drive two targets at different amounts. Sources address a
specific modulation output, so a modulator with several outputs can be routed from any
of them.

## Usage in Blocks

Blocks store parameters as `Parameter<S>` and name them through three `Block` methods.
`parameter_names()` lists the canonical names; `parameter()` and `parameter_mut()` look
one up, accepting aliases and ignoring case via `parameter_name_matches`:

```rust
pub struct LowPassFilterBlock<S: Sample> {
    pub cutoff: Parameter<S>,
    pub resonance: Parameter<S>,
    // ...
}

impl<S: Sample> Block<S> for LowPassFilterBlock<S> {
    fn parameter_names(&self) -> &'static [&'static str] {
        &["cutoff", "resonance"]
    }

    fn parameter_mut(&mut self, name: &str) -> Option<&mut Parameter<S>> {
        if parameter_name_matches(name, &["cutoff", "frequency"]) {
            Some(&mut self.cutoff)
        } else if parameter_name_matches(name, &["resonance", "q"]) {
            Some(&mut self.resonance)
        } else {
            None
        }
    }

    // parameter() mirrors parameter_mut() with shared references
}
```

## Resolving Values

During processing, resolve the value against the graph's [`ModulationValues`] view:

```rust
fn process(&mut self, inputs: &[&[S]], outputs: &mut [&mut [S]],
           modulation_values: &ModulationValues<S>, context: &DspContext) {
    let cutoff_hz = self.cutoff.value(modulation_values).to_f64();
    // ...
}
```

When the base is decided elsewhere, such as an oscillator following a MIDI note, use
`value_with_base` so the routes still add on top of the substituted value:

```rust
let base = self.midi_frequency.unwrap_or(self.frequency.base());
let frequency = self.frequency.value_with_base(base, modulation_values);
```

Blocks driven outside a graph pass `ModulationValues::empty()`, which reads every
source as zero.

## Constants and Routes

```rust
let gain = Parameter::constant(S::from_f64(-6.0));
let frequency: Parameter<S> = S::from_f64(440.0).into();

assert!(!gain.has_routes());
```

Routes are normally attached by `GraphBuilder::modulate`, which validates the source and
the parameter name at build time. They can also be attached directly:

```rust
let mut cutoff = Parameter::constant(1000.0_f32);
cutoff.add_route(ModulationRoute::new(ModulationSource::first_output(lfo), 500.0))?;
cutoff.routes_mut()[0].set_depth(250.0);
cutoff.clear_routes();
```

`add_route` fails with `ParameterError::RoutesFull` once the parameter holds
`MAX_MODULATION_ROUTES` routes. Nothing here allocates; the route list is a `StackVec`.

## Parameter Smoothing

Many blocks smooth parameter changes to prevent clicks and pops. Smoothing uses linear interpolation to ramp between values over a configurable time period.

### Default Behavior

- Default ramp time: 50ms
- Smoothing is linear interpolation between current and target values
- Smoothing is automatically applied during `process()` when parameters change

### Per-Block Smoothing Configuration

Use `set_smoothing()` to configure smoothing time for all parameters in a block:

```rust
// After building the graph, access the block and configure smoothing
if let Some(block) = graph.get_block_mut(block_id) {
    block.set_smoothing(44100.0, 100.0); // 100ms ramp for all parameters
}
```

**Important:** `set_smoothing()` applies the same ramp time to ALL smoothed parameters in a block. For per-parameter control, create and configure the block before passing it to `GraphBuilder::add()`.

### Blocks with Smoothing

The following blocks implement `set_smoothing()`:

- `GainBlock` - smooths `level_db`
- `OverdriveBlock` - smooths `drive` and `level`
- `PannerBlock` - smooths `position`, `azimuth`, and `elevation`

## Design Rationale

1. **One type for constant and modulated** - a constant is a parameter with no routes; the hot loop over zero routes is free
2. **Depth at the destination** - modulators emit their natural range; each route scales it, like a hardware modulation matrix
3. **Summed sources** - several modulators can drive one parameter without an intermediate mixer
4. **Realtime-safe** - fixed-capacity routes, no allocation in `value()`, out-of-range sources read as zero instead of panicking
5. **Sample-type generic** - depth is `S`, so the multiply stays in the graph's sample type
