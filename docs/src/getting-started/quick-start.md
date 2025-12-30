# Quick Start

This guide walks you through creating your first DSP graph with bbx_audio.

## Building a Simple Synthesizer

Let's create a sine wave oscillator with gain control:

```rust
use bbx_dsp::{
    Graph, GraphBuilder,
    blocks::{OscillatorBlock, GainBlock, OutputBlock, Waveform},
    context::DspContext,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a DSP context
    let context = DspContext::new(44100.0, 512, 2);

    // Build the graph
    let mut graph = GraphBuilder::new()
        .add_block(OscillatorBlock::new(440.0, Waveform::Sine))  // Block 0
        .add_block(GainBlock::new(-6.0))                          // Block 1
        .add_block(OutputBlock::new(2))                           // Block 2
        .connect(0, 0, 1, 0)?  // Oscillator output -> Gain input
        .connect(1, 0, 2, 0)?  // Gain output -> Output block
        .build()?;

    // Prepare the graph
    graph.prepare(&context);

    // Process audio
    let mut output = vec![vec![0.0f32; 512]; 2];
    let inputs: Vec<&[f32]> = vec![];
    let mut outputs: Vec<&mut [f32]> = output.iter_mut().map(|v| v.as_mut_slice()).collect();

    graph.process(&inputs, &mut outputs, &context);

    // output now contains 512 samples of a 440Hz sine wave at -6dB
    println!("Generated {} samples", output[0].len());

    Ok(())
}
```

## Understanding the Code

### DspContext

The `DspContext` holds audio processing parameters:

```rust
let context = DspContext::new(
    44100.0,  // Sample rate in Hz
    512,      // Buffer size in samples
    2,        // Number of channels
);
```

### GraphBuilder

The `GraphBuilder` provides a fluent API for constructing DSP graphs:

```rust
let graph = GraphBuilder::new()
    .add_block(/* block */)  // Returns block index
    .connect(from_block, from_port, to_block, to_port)?
    .build()?;
```

### Connections

Connections are made between block outputs and inputs using indices:

```rust
.connect(0, 0, 1, 0)?  // Block 0, output 0 -> Block 1, input 0
```

## Adding Effects

Let's add some effects to our oscillator:

```rust
use bbx_dsp::blocks::{PannerBlock, OverdriveBlock};

let mut graph = GraphBuilder::new()
    .add_block(OscillatorBlock::new(440.0, Waveform::Saw))   // 0: Oscillator
    .add_block(OverdriveBlock::new(0.7))                      // 1: Overdrive
    .add_block(GainBlock::new(-12.0))                         // 2: Gain
    .add_block(PannerBlock::new(0.0))                         // 3: Panner (center)
    .add_block(OutputBlock::new(2))                           // 4: Output
    .connect(0, 0, 1, 0)?  // Osc -> Overdrive
    .connect(1, 0, 2, 0)?  // Overdrive -> Gain
    .connect(2, 0, 3, 0)?  // Gain -> Panner
    .connect(3, 0, 4, 0)?  // Panner L -> Output
    .connect(3, 1, 4, 1)?  // Panner R -> Output
    .build()?;
```

## Next Steps

- [Creating a Simple Oscillator](../tutorials/oscillator.md) - Explore oscillator waveforms
- [Adding Effects](../tutorials/effects.md) - Learn about effect blocks
- [Parameter Modulation](../tutorials/modulation.md) - Use LFOs to modulate parameters
- [JUCE Integration](../juce/overview.md) - Integrate with JUCE plugins
