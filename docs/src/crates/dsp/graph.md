# Graph and GraphBuilder

The core types for building and processing DSP graphs.

## GraphBuilder

`GraphBuilder` provides a fluent API for constructing DSP graphs.

### Creating a Builder

```rust
use bbx_dsp::graph::GraphBuilder;

let mut builder = GraphBuilder::<f32>::new(
    44100.0,  // sample rate
    512,      // buffer size
    2,        // channels
);
```

### Creating with Channel Layout

For multi-channel support beyond stereo, use `with_layout`:

```rust
use bbx_dsp::{channel::ChannelLayout, graph::GraphBuilder};

// 5.1 surround graph
let builder = GraphBuilder::<f32>::with_layout(44100.0, 512, ChannelLayout::Surround51);

// First-order ambisonics graph
let builder = GraphBuilder::<f32>::with_layout(44100.0, 512, ChannelLayout::AmbisonicFoa);
```

### Adding Blocks

Use the generic `add()` method to add any block type:

```rust
use bbx_dsp::{
    blocks::{
        DcBlockerBlock, EnvelopeBlock, GainBlock, LfoBlock, OscillatorBlock,
        OverdriveBlock, PannerBlock,
    },
    graph::GraphBuilder,
    waveform::Waveform,
};

let mut builder = GraphBuilder::<f32>::new(44100.0, 512, 2);

// Oscillator: frequency, waveform, seed
let osc = builder.add(OscillatorBlock::new(440.0, Waveform::Sine, None));

// Overdrive: drive, level, tone, sample_rate
let overdrive = builder.add(OverdriveBlock::new(3.0, 1.0, 0.8, 44100.0));

// LFO: frequency, depth, waveform, seed
let lfo = builder.add(LfoBlock::new(5.0, 0.5, Waveform::Sine, None));

// Envelope: attack, decay, sustain, release
let env = builder.add(EnvelopeBlock::new(0.01, 0.1, 0.7, 0.3));

// Gain: level_db, base_gain
let gain = builder.add(GainBlock::new(-6.0, None));

// Panner: position
let pan = builder.add(PannerBlock::new(0.0));

// DC blocker: enabled
let dc = builder.add(DcBlockerBlock::new(true));
```

### Multi-Channel Blocks

Use the generic `add()` method for multi-channel routing blocks:

```rust
use bbx_dsp::{
    blocks::{
        AmbisonicDecoderBlock, ChannelMergerBlock, ChannelSplitterBlock,
        MatrixMixerBlock, PannerBlock,
    },
    channel::ChannelLayout,
    graph::GraphBuilder,
};

let mut builder = GraphBuilder::<f32>::new(44100.0, 512, 6);

// Channel routing
let splitter = builder.add(ChannelSplitterBlock::new(6));   // Split to mono outputs
let merger = builder.add(ChannelMergerBlock::new(6));       // Merge mono inputs
let mixer = builder.add(MatrixMixerBlock::new(4, 2));       // NxM matrix mixer

// Surround and ambisonic panning
let surround = builder.add(PannerBlock::surround(ChannelLayout::Surround51));
let ambisonic = builder.add(PannerBlock::ambisonic(1));     // FOA encoder

// Ambisonic decoding
let decoder = builder.add(AmbisonicDecoderBlock::new(1, ChannelLayout::Stereo));
```

### Connecting Blocks

Connect block outputs to inputs:

```rust
// connect(from_block, from_port, to_block, to_port)
builder.connect(osc, 0, gain, 0);
builder.connect(gain, 0, pan, 0);
```

### Modulation

Use `modulate()` to connect modulators to parameters:

```rust
// modulate(source, target, parameter_name)
builder.modulate(lfo, osc, "frequency");
builder.modulate(lfo, gain, "level");
```

### Building the Graph

```rust
let graph = builder.build();
```

The build process:
1. Validates all connections
2. Performs topological sorting
3. Allocates processing buffers
4. Returns an optimized `Graph`

## Graph

`Graph` is the compiled, ready-to-process DSP graph.

### Processing Audio

```rust
let mut left = vec![0.0f32; 512];
let mut right = vec![0.0f32; 512];
let mut outputs: [&mut [f32]; 2] = [&mut left, &mut right];

graph.process_buffers(&mut outputs);
```

### Handling Audio Context Changes

Call `prepare()` when sample rate, buffer size, or channel count changes:

```rust
// Sample rate changed to 48kHz, buffer size to 256
graph.prepare(48000.0, 256, 2);
```

This computes the execution order, pre-allocates buffers, and propagates to all blocks, allowing them to recalculate sample-rate-dependent coefficients and reset any state that would cause glitches.

Note: `GraphBuilder::build()` calls this automatically with the initial settings.

### Resetting State

Call `reset()` to clear all block state without changing configuration:

```rust
graph.reset();
```

This clears delay lines, filter states, phase accumulators, etc. Useful when starting fresh playback or when the audio stream is discontinuous.

### Finalization

For file output, call `finalize()` to flush buffers:

```rust
graph.finalize();
```

## BlockId

A handle to a block in the graph:

```rust
let osc: BlockId = builder.add(OscillatorBlock::new(440.0, Waveform::Sine, None));
```

`BlockId` is used for:
- Connecting blocks
- Referencing modulators
- Accessing block state (if needed)

## Connection Rules

- Each output can connect to multiple inputs
- Each input can receive multiple connections (summed)
- Cycles are not allowed (topological sorting will fail)
- Unconnected blocks are still processed

## Example: Complex Graph

```rust
use bbx_dsp::{
    blocks::{DcBlockerBlock, GainBlock, OscillatorBlock, OverdriveBlock, PannerBlock},
    graph::GraphBuilder,
    waveform::Waveform,
};

let mut builder = GraphBuilder::<f32>::new(44100.0, 512, 2);

// Create two oscillators
let osc1 = builder.add(OscillatorBlock::new(440.0, Waveform::Saw, None));
let osc2 = builder.add(OscillatorBlock::new(441.0, Waveform::Saw, None));  // Slight detune

// Mix them
let mixer = builder.add(GainBlock::new(-6.0, None));
builder.connect(osc1, 0, mixer, 0);
builder.connect(osc2, 0, mixer, 0);

// Add effects
let overdrive = builder.add(OverdriveBlock::new(3.0, 1.0, 0.8, 44100.0));
let dc_blocker = builder.add(DcBlockerBlock::new(true));
let pan = builder.add(PannerBlock::new(0.0));

// Chain effects
builder.connect(mixer, 0, overdrive, 0);
builder.connect(overdrive, 0, dc_blocker, 0);
builder.connect(dc_blocker, 0, pan, 0);

let graph = builder.build();
```
