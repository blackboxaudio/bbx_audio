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
    parameter::{ModulationOutput, Parameter},
    sample::Sample,
};

const MAX_BUFFER_SIZE: usize = 4096;

pub struct MyEffectBlock<S: Sample> {
    pub gain: Parameter<S>,
    state: S,
}

impl<S: Sample> MyEffectBlock<S> {
    pub fn new(gain: f64) -> Self {
        Self {
            gain: Parameter::Constant(S::from_f64(gain)),
            state: S::ZERO,
        }
    }
}

impl<S: Sample> Block<S> for MyEffectBlock<S> {
    fn prepare(&mut self, context: &DspContext) {
        // Initialize parameter smoothing with sample rate
        self.gain.prepare(context.sample_rate);
    }

    fn process(
        &mut self,
        inputs: &[&[S]],
        outputs: &mut [&mut [S]],
        modulation_values: &[S],
        context: &DspContext,
    ) {
        // Update smoothing target from modulation source
        self.gain.update_target(modulation_values);

        let len = inputs.first().map_or(0, |ch| ch.len().min(context.buffer_size));
        let num_channels = inputs.len().min(outputs.len());

        // Fast path: constant value when not smoothing
        if !self.gain.is_smoothing() {
            let gain = self.gain.current();
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
            *gain_value = self.gain.next_value();
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

    fn reset(&mut self) {
        self.state = S::ZERO;
    }
}
```

## Key Patterns

### Parameter Initialization

Parameters combine value source with built-in smoothing:

```rust
// Constant parameter (50ms default ramp)
let gain = Parameter::Constant(S::from_f64(0.5));

// Custom ramp time
let freq = Parameter::Constant(S::from_f64(440.0)).with_ramp_ms(100.0);
```

### Processing Flow

1. **`prepare()`**: Call `parameter.prepare(sample_rate)` to initialize smoothing
2. **Update target**: Use `update_target()` or `set_target()` at buffer start
3. **Check smoothing**: Use `is_smoothing()` for fast-path optimization
4. **Get values**: Use `current()` for constant, `next_value()` for smoothing

### Value Transforms

When the raw value needs transformation before smoothing (e.g., dB to linear):

```rust
// Get raw dB value
let db = self.level_db.get_raw_value(modulation_values).to_f64();

// Convert to linear and set as smooth target
let linear = 10.0_f64.powf(db / 20.0);
self.level_db.set_target(S::from_f64(linear));
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

        block.process(&[&input], &mut [&mut output], &[], &context);

        assert_eq!(output, [0.5, 0.25, 0.125, 0.0]);
    }
}
```

## Update Documentation

1. Add to blocks reference in docs
2. Update README if significant
3. Add examples in bbx_sandbox
