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

### Adding Blocks

Each `add_*` method returns a `BlockId`:

```rust
use bbx_dsp::{graph::GraphBuilder, waveform::Waveform};

let mut builder = GraphBuilder::<f32>::new(44100.0, 512, 2);

let osc = builder.add_oscillator(440.0, Waveform::Sine, None);
let gain = builder.add_gain(-6.0);
let pan = builder.add_panner(0.0);
```

### Connecting Blocks

Connect block outputs to inputs:

```rust
// connect(from_block, from_port, to_block, to_port)
builder.connect(osc, 0, gain, 0);
builder.connect(gain, 0, pan, 0);
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

### Preparing for Playback

Call `prepare()` when audio specs change:

```rust
use bbx_dsp::context::DspContext;

let context = DspContext::new(48000.0, 1024, 2);
graph.prepare(&context);
```

### Resetting State

Clear all DSP state (filters, delay lines, etc.):

```rust
graph.reset();
```

### Finalization

For file output, call `finalize()` to flush buffers:

```rust
graph.finalize();
```

## BlockId

A handle to a block in the graph:

```rust
let osc: BlockId = builder.add_oscillator(440.0, Waveform::Sine, None);
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
use bbx_dsp::{graph::GraphBuilder, waveform::Waveform};

let mut builder = GraphBuilder::<f32>::new(44100.0, 512, 2);

// Create two oscillators
let osc1 = builder.add_oscillator(440.0, Waveform::Saw, None);
let osc2 = builder.add_oscillator(441.0, Waveform::Saw, None);  // Slight detune

// Mix them
let mixer = builder.add_gain(-6.0);
builder.connect(osc1, 0, mixer, 0);
builder.connect(osc2, 0, mixer, 0);

// Add effects
let overdrive = builder.add_overdrive(3.0, 1.0, 0.8, 44100.0);
let dc_blocker = builder.add_dc_blocker();
let pan = builder.add_panner(0.0);

// Chain effects
builder.connect(mixer, 0, overdrive, 0);
builder.connect(overdrive, 0, dc_blocker, 0);
builder.connect(dc_blocker, 0, pan, 0);

let graph = builder.build();
```
