# Parameter\<S\> Type

The generic parameter type for static and modulated values.

## Definition

```rust
pub enum Parameter<S: Sample> {
    /// Fixed value
    Constant(S),

    /// Value controlled by a modulator block
    Modulated(BlockId),
}
```

## Usage in Blocks

Blocks store parameters as `Parameter<S>`:

```rust
pub struct OscillatorBlock<S: Sample> {
    frequency: Parameter<S>,
    waveform: Waveform,
    phase: S,
}
```

## Resolving Values

During processing, resolve the actual value:

```rust
impl<S: Sample> OscillatorBlock<S> {
    fn get_frequency(&self, modulation: &[S]) -> S {
        match &self.frequency {
            Parameter::Constant(value) => *value,
            Parameter::Modulated(block_id) => {
                let base = S::from_f64(440.0);
                let mod_value = modulation[block_id.0];
                base * (S::ONE + mod_value * S::from_f64(0.1))  // ±10%
            }
        }
    }
}
```

## Constant vs Modulated

### Constant

- Value known at creation
- No per-block overhead
- Simple and direct

```rust
let gain = Parameter::Constant(S::from_f64(-6.0));
```

### Modulated

- Value changes each buffer
- Requires modulator block
- Adds routing complexity

```rust
let frequency = Parameter::Modulated(lfo_block_id);
```

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

The `Parameter` enum:

1. **Unifies constant and dynamic** - Same API for both
2. **Type-safe modulation** - Compile-time block ID checking
3. **Zero-cost constant** - No indirection for constant values
4. **Sample-type generic** - Works with f32 and f64