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
use crate::{block::Block, context::DspContext, sample::Sample};

pub struct MyEffectBlock<S: Sample> {
    // Block state
    gain: S,
    state: S,
}

impl<S: Sample> MyEffectBlock<S> {
    pub fn new(gain: f64) -> Self {
        Self {
            gain: S::from_f64(gain),
            state: S::ZERO,
        }
    }
}

impl<S: Sample> Block<S> for MyEffectBlock<S> {
    fn process(
        &mut self,
        inputs: &[&[S]],
        outputs: &mut [&mut [S]],
        context: &DspContext,
        _modulation: &[S],
    ) {
        for ch in 0..inputs.len().min(outputs.len()) {
            for i in 0..context.buffer_size {
                outputs[ch][i] = inputs[ch][i] * self.gain;
            }
        }
    }

    fn num_inputs(&self) -> usize { 1 }
    fn num_outputs(&self) -> usize { 1 }

    fn prepare(&mut self, _context: &DspContext) {
        // Recalculate coefficients if needed
    }

    fn reset(&mut self) {
        self.state = S::ZERO;
    }
}
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
    /// Add a MyEffectBlock to the graph.
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

        let input = [1.0, 0.5, 0.25, 0.0];
        let mut output = [0.0; 4];

        block.process(&[&input], &mut [&mut output], &context, &[]);

        assert_eq!(output, [0.5, 0.25, 0.125, 0.0]);
    }
}
```

## Update Documentation

1. Add to blocks reference in docs
2. Update README if significant
3. Add examples in bbx_sandbox
