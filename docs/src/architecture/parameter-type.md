# Parameter<S> Type

The generic parameter type for static and modulated values.

## Definition

```rust
pub enum Parameter<S: Sample> {
    /// Fixed value
    Static(S),

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
            Parameter::Static(value) => *value,
            Parameter::Modulated(block_id) => {
                let base = S::from_f64(440.0);
                let mod_value = modulation[block_id.0];
                base * (S::ONE + mod_value * S::from_f64(0.1))  // ±10%
            }
        }
    }
}
```

## Static vs Modulated

### Static

- Value known at creation
- No per-block overhead
- Simple and direct

```rust
let gain = Parameter::Static(S::from_f64(-6.0));
```

### Modulated

- Value changes each buffer
- Requires modulator block
- Adds routing complexity

```rust
let frequency = Parameter::Modulated(lfo_block_id);
```

## Design Rationale

The `Parameter` enum:

1. **Unifies static and dynamic** - Same API for both
2. **Type-safe modulation** - Compile-time block ID checking
3. **Zero-cost static** - No indirection for static values
4. **Sample-type generic** - Works with f32 and f64
