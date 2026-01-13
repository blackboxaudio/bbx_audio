# BlockType Enum

`BlockType` is an enum that wraps all concrete block implementations.

## Overview

The graph system uses `BlockType` to store heterogeneous blocks:

```rust
pub enum BlockType<S: Sample> {
    // I/O
    FileInput(FileInputBlock<S>),
    FileOutput(FileOutputBlock<S>),
    Output(OutputBlock<S>),

    // Generators
    Oscillator(OscillatorBlock<S>),

    // Effectors
    ChannelRouter(ChannelRouterBlock<S>),
    DcBlocker(DcBlockerBlock<S>),
    Gain(GainBlock<S>),
    LowPassFilter(LowPassFilterBlock<S>),
    Overdrive(OverdriveBlock<S>),
    Panner(PannerBlock<S>),
    Vca(VcaBlock<S>),

    // Modulators
    Envelope(EnvelopeBlock<S>),
    Lfo(LfoBlock<S>),
}
```

## Usage

`BlockType` is primarily used internally by the graph system. Users interact with blocks through `GraphBuilder`:

```rust
use bbx_dsp::{blocks::{GainBlock, OscillatorBlock}, graph::GraphBuilder, waveform::Waveform};

let mut builder = GraphBuilder::<f32>::new(44100.0, 512, 2);

// These return BlockId, not BlockType
let osc = builder.add(OscillatorBlock::new(440.0, Waveform::Sine, None));
let gain = builder.add(GainBlock::new(-6.0, None));
```

## Block Trait Implementation

`BlockType` implements `Block` by delegating to the wrapped type:

```rust
impl<S: Sample> BlockType<S> {
    fn process(
        &mut self,
        inputs: &[&[S]],
        outputs: &mut [&mut [S]],
        modulation_values: &[S],
        context: &DspContext,
    ) {
        match self {
            BlockType::Oscillator(b) => b.process(inputs, outputs, modulation_values, context),
            BlockType::Gain(b) => b.process(inputs, outputs, modulation_values, context),
            // ... etc
        }
    }

    fn input_count(&self) -> usize {
        match self {
            BlockType::Oscillator(b) => b.input_count(),
            BlockType::Gain(b) => b.input_count(),
            // ... etc
        }
    }

    // ... other methods
}
```

## Adding Custom Blocks

To add a custom block type, you would need to:

1. Implement `Block<S>` for your block
2. Add a variant to `BlockType`
3. Update all match arms in `BlockType`'s `Block` implementation
4. Add a builder method to `GraphBuilder`

For plugin development, consider using `PluginDsp` instead, which doesn't require modifying `BlockType`.

## Pattern Matching

If you need to access the inner block type:

```rust
use bbx_dsp::block::BlockType;

fn get_oscillator_frequency<S: Sample>(block: &BlockType<S>) -> Option<f64> {
    match block {
        BlockType::Oscillator(osc) => Some(osc.frequency()),
        _ => None,
    }
}
```

## Block Categories

Blocks are organized into categories:

| Category | Variants |
|----------|----------|
| Generators | `Oscillator` |
| Effectors | `ChannelRouter`, `DcBlocker`, `Gain`, `LowPassFilter`, `Overdrive`, `Panner`, `Vca` |
| Modulators | `Envelope`, `Lfo` |
| I/O | `FileInput`, `FileOutput`, `Output` |
