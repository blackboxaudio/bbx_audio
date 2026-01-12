# Parameter\<S\> Type

A smoothed parameter type that combines value sources with built-in interpolation.

## Definition

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

## Creating Parameters

```rust
use bbx_dsp::parameter::Parameter;

// Constant value with default linear smoothing (50ms ramp)
let gain = Parameter::<f32>::constant(0.5);

// Modulated by a block (e.g., LFO) with initial value
let freq = Parameter::<f32>::modulated(lfo_id, 440.0);

// Custom ramp time
let pan = Parameter::<f32>::constant(0.0).with_ramp_ms(100.0);
```

## Usage in Blocks

Blocks store parameters and use the built-in smoothing:

```rust
pub struct GainBlock<S: Sample> {
    pub level_db: Parameter<S>,
}

impl<S: Sample> Block<S> for GainBlock<S> {
    fn prepare(&mut self, context: &DspContext) {
        self.level_db.prepare(context.sample_rate);
    }

    fn process(&mut self, inputs: &[&[S]], outputs: &mut [&mut [S]],
               modulation_values: &[S], context: &DspContext) {
        // Get raw value and apply transform (dB to linear)
        let db = self.level_db.get_raw_value(modulation_values);
        let linear = db_to_linear(db);
        self.level_db.set_target(linear);

        // Fast path when not smoothing
        if !self.level_db.is_smoothing() {
            let gain = self.level_db.current();
            // Apply constant gain...
            return;
        }

        // Smoothing path: generate per-sample values
        for i in 0..context.buffer_size {
            let gain = self.level_db.next_value();
            outputs[0][i] = inputs[0][i] * gain;
        }
    }
}
```

## Smoothing Strategies

Two strategies are available via the generic parameter:

### Linear (default)

Additive interpolation using `current + increment`. Best for parameters with linear perception like pan position.

### Multiplicative

Exponential interpolation using `current * e^increment`. Best for parameters with logarithmic perception like gain and frequency.

```rust
use bbx_dsp::smoothing::Multiplicative;

// Use multiplicative smoothing for frequency
let freq: Parameter<f32, Multiplicative> = Parameter::constant(440.0);
```

## Key Methods

| Method | Description |
|--------|-------------|
| `prepare(sample_rate)` | Initialize smoother for given sample rate |
| `get_raw_value(modulation)` | Get unsmoothed source value |
| `update_target(modulation)` | Set smoother target from source |
| `set_target(value)` | Set smoother target directly (for transformed values) |
| `next_value()` | Get next smoothed sample |
| `current()` | Get current value without advancing |
| `is_smoothing()` | Check if actively interpolating |
| `fill_buffer(buffer, modulation)` | Fill buffer with smoothed values |

## Design Rationale

The `Parameter` type:

1. **Unifies value source and smoothing** - Blocks don't manage separate smoothers
2. **Click-free changes** - Built-in interpolation prevents audible artifacts
3. **Configurable strategy** - Choose linear or multiplicative based on perception
4. **Optimizable** - `is_smoothing()` enables fast-path when values are stable
5. **Sample-type generic** - Works with f32 and f64
