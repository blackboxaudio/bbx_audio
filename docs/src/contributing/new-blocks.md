# Adding New Blocks

Guide to implementing new DSP blocks.

## Block Structure

Create a new file in the appropriate category:

```
bbx_dsp/src/blocks/
├── generators/
│   └── my_generator.rs
├── effectors/
│   └── my_effect.rs
└── modulators/
    └── my_modulator.rs
```

## Implement the Block Trait

```rust
use crate::{
    block::{Block, DEFAULT_EFFECTOR_INPUT_COUNT, DEFAULT_EFFECTOR_OUTPUT_COUNT},
    context::DspContext,
    parameter::{ModulationOutput, ModulationValues, Parameter, parameter_name_matches},
    sample::Sample,
    smoothing::LinearSmoothedValue,
};

const MAX_BUFFER_SIZE: usize = 4096;

pub struct MyEffectBlock<S: Sample> {
    pub gain: Parameter<S>,
    gain_smoother: LinearSmoothedValue<S>,
}

impl<S: Sample> MyEffectBlock<S> {
    pub fn new(gain: f64) -> Self {
        Self {
            gain: Parameter::constant(S::from_f64(gain)),
            gain_smoother: LinearSmoothedValue::new(S::from_f64(gain)),
        }
    }
}

impl<S: Sample> Block<S> for MyEffectBlock<S> {
    fn process(
        &mut self,
        inputs: &[&[S]],
        outputs: &mut [&mut [S]],
        modulation_values: &ModulationValues<S>,
        context: &DspContext,
    ) {
        // Resolve base + Σ depth · source once per buffer, then smooth toward it
        let target = self.gain.value(modulation_values);
        if (target - self.gain_smoother.target()).abs() > S::EPSILON {
            self.gain_smoother.set_target_value(target);
        }

        let len = inputs.first().map_or(0, |ch| ch.len().min(context.buffer_size));
        let num_channels = inputs.len().min(outputs.len());

        // Fast path: constant value when not smoothing
        if !self.gain_smoother.is_smoothing() {
            let gain = self.gain_smoother.current();
            for ch in 0..num_channels {
                for i in 0..len {
                    outputs[ch][i] = inputs[ch][i] * gain;
                }
            }
            return;
        }

        // Smoothing path: pre-compute smoothed values once
        let mut gain_values: [S; MAX_BUFFER_SIZE] = [S::ZERO; MAX_BUFFER_SIZE];
        for gain_value in gain_values.iter_mut().take(len) {
            *gain_value = self.gain_smoother.get_next_value();
        }

        // Apply to all channels
        for ch in 0..num_channels {
            for (i, &gain) in gain_values.iter().enumerate().take(len) {
                outputs[ch][i] = inputs[ch][i] * gain;
            }
        }
    }

    fn input_count(&self) -> usize {
        DEFAULT_EFFECTOR_INPUT_COUNT
    }

    fn output_count(&self) -> usize {
        DEFAULT_EFFECTOR_OUTPUT_COUNT
    }

    fn modulation_outputs(&self) -> &[ModulationOutput] {
        &[]
    }

    // Expose the parameter so GraphBuilder::modulate can find it by name.
    // parameter_names() drives topology snapshots; parameter()/parameter_mut()
    // may accept aliases and ignore case.
    fn parameter_names(&self) -> &'static [&'static str] {
        &["gain"]
    }

    fn parameter(&self, name: &str) -> Option<&Parameter<S>> {
        parameter_name_matches(name, &["gain", "level"]).then_some(&self.gain)
    }

    fn parameter_mut(&mut self, name: &str) -> Option<&mut Parameter<S>> {
        parameter_name_matches(name, &["gain", "level"]).then_some(&mut self.gain)
    }

    fn set_smoothing(&mut self, sample_rate: f64, ramp_time_ms: f64) {
        self.gain_smoother.reset(sample_rate, ramp_time_ms);
    }
}
```

## Key Patterns

### Buffer Length Clamping

`process()` must size every loop by `slice.len().min(context.buffer_size)`, never by
`context.buffer_size` alone. Callers are allowed to pass slices shorter than the graph's
block size — for example when splitting a block at MIDI event offsets for sample-accurate
timing — and indexing by `buffer_size` panics on the shorter slice. Blocks that track a
position or phase must advance it by the samples actually processed.

### Parameters

A `Parameter<S>` is a base value plus up to four summed, depth-scaled modulation routes.
A parameter with no routes is a constant. Smoothing is separate: keep a
`LinearSmoothedValue` beside the parameter and set its target from `parameter.value(...)`
at the top of `process()`.

```rust
let gain = Parameter::constant(S::from_f64(0.5));
let frequency: Parameter<S> = S::from_f64(440.0).into();
```

### Processing Flow

1. **Resolve**: `parameter.value(modulation_values)` once per buffer (or `value_with_base` when another source decides the base, such as a MIDI note)
2. **Smooth**: push the resolved value into a `SmoothedValue` target
3. **Check smoothing**: use `is_smoothing()` for a fast path
4. **Get values**: `current()` when settled, `get_next_value()` per sample while ramping

### Value Transforms

When the raw value needs transformation before smoothing (e.g., dB to linear):

```rust
let db = self.level_db.value(modulation_values).to_f64();
let linear = 10.0_f64.powf(db / 20.0);
self.gain_smoother.set_target_value(S::from_f64(linear));
```

## Add to BlockType

In `bbx_dsp/src/block.rs`:

```rust
pub enum BlockType<S: Sample> {
    // Existing variants...
    MyEffect(MyEffectBlock<S>),
}
```

Update all match arms in `BlockType`'s `Block` implementation.

## Add Builder Method

In `bbx_dsp/src/graph.rs`:

```rust
impl<S: Sample> GraphBuilder<S> {
    pub fn add_my_effect(&mut self, gain: f64) -> BlockId {
        let block = BlockType::MyEffect(MyEffectBlock::new(gain));
        self.graph.add_block(block)
    }
}
```

## Write Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_my_effect_basic() {
        let mut block = MyEffectBlock::<f32>::new(0.5);
        let context = DspContext::new(44100.0, 4, 1);
        block.prepare(&context);

        let input = [1.0, 0.5, 0.25, 0.0];
        let mut output = [0.0; 4];

        block.process(&[&input], &mut [&mut output], &ModulationValues::empty(), &context);

        assert_eq!(output, [0.5, 0.25, 0.125, 0.0]);
    }
}
```

## Update Documentation

1. Add to blocks reference in docs
2. Update README if significant
3. Add examples in bbx_sandbox
