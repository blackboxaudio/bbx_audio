# MatrixMixerBlock

An NxM mixing matrix for flexible channel routing with gain control.

## Overview

`MatrixMixerBlock` provides arbitrary channel mixing where each output is a weighted sum of all inputs. This enables complex routing scenarios like downmixing, upmixing, and channel swapping.

## Creating a Matrix Mixer

```rust
use bbx_dsp::graph::GraphBuilder;

let mut builder = GraphBuilder::<f32>::new(44100.0, 512, 2);

// Create a 4-input, 2-output mixer
let mixer = builder.add_matrix_mixer(4, 2);
```

Or with direct construction:

```rust
use bbx_dsp::{
    block::BlockType,
    blocks::MatrixMixerBlock,
    graph::GraphBuilder,
};

let mut builder = GraphBuilder::<f32>::new(44100.0, 512, 2);

// 4x2 matrix (4 inputs, 2 outputs)
let mut matrix = MatrixMixerBlock::<f32>::new(4, 2);

// Configure gains before adding to graph
matrix.set_gain(0, 0, 1.0);  // input 0 -> output 0 at unity
matrix.set_gain(1, 1, 1.0);  // input 1 -> output 1 at unity

let mixer = builder.add_block(BlockType::MatrixMixer(matrix));
```

## Port Layout

| Port | Direction | Description |
|------|-----------|-------------|
| 0..N | Input | N input channels |
| 0..M | Output | M output channels |

Input and output counts are configured independently (1-16 each).

## Parameters

| Parameter | Type | Range | Description |
|-----------|------|-------|-------------|
| inputs | usize | 1-16 | Number of input channels |
| outputs | usize | 1-16 | Number of output channels |

## API Methods

### set_gain

Set the gain for a specific input-to-output routing:

```rust
mixer.set_gain(input, output, gain);
```

- `input`: Source channel index (0-based)
- `output`: Destination channel index (0-based)
- `gain`: Linear gain (typically 0.0 to 1.0)

### get_gain

Get the current gain for a routing:

```rust
let gain = mixer.get_gain(input, output);
```

### identity

Create an identity matrix (passthrough):

```rust
let mixer = MatrixMixerBlock::<f32>::identity(4);
// Input 0 -> Output 0 at 1.0
// Input 1 -> Output 1 at 1.0
// etc.
```

## Usage Examples

### Stereo to Mono Downmix

```rust
use bbx_dsp::{
    block::BlockType,
    blocks::MatrixMixerBlock,
    graph::GraphBuilder,
};

let mut builder = GraphBuilder::<f32>::new(44100.0, 512, 2);

// 2 inputs -> 1 output
let mut mixer = MatrixMixerBlock::<f32>::new(2, 1);
mixer.set_gain(0, 0, 0.5);  // Left -> Mono at 0.5
mixer.set_gain(1, 0, 0.5);  // Right -> Mono at 0.5

let downmix = builder.add_block(BlockType::MatrixMixer(mixer));
```

### Channel Swap

```rust
use bbx_dsp::{
    block::BlockType,
    blocks::MatrixMixerBlock,
    graph::GraphBuilder,
};

let mut builder = GraphBuilder::<f32>::new(44100.0, 512, 2);

// 2 inputs -> 2 outputs, swapped
let mut mixer = MatrixMixerBlock::<f32>::new(2, 2);
mixer.set_gain(0, 1, 1.0);  // Left input -> Right output
mixer.set_gain(1, 0, 1.0);  // Right input -> Left output

let swap = builder.add_block(BlockType::MatrixMixer(mixer));
```

### Quad to Stereo Downmix

```rust
use bbx_dsp::{
    block::BlockType,
    blocks::MatrixMixerBlock,
    graph::GraphBuilder,
};

let mut builder = GraphBuilder::<f32>::new(44100.0, 512, 2);

// 4 inputs -> 2 outputs
let mut mixer = MatrixMixerBlock::<f32>::new(4, 2);

// Front left + rear left -> Left output
mixer.set_gain(0, 0, 0.7);  // Front left
mixer.set_gain(2, 0, 0.5);  // Rear left

// Front right + rear right -> Right output
mixer.set_gain(1, 1, 0.7);  // Front right
mixer.set_gain(3, 1, 0.5);  // Rear right

let downmix = builder.add_block(BlockType::MatrixMixer(mixer));
```

### Mid-Side Encoding

```rust
use bbx_dsp::{
    block::BlockType,
    blocks::MatrixMixerBlock,
    graph::GraphBuilder,
};

let mut builder = GraphBuilder::<f32>::new(44100.0, 512, 2);

// Stereo to M/S: M = (L+R)/2, S = (L-R)/2
let mut ms_encode = MatrixMixerBlock::<f32>::new(2, 2);
ms_encode.set_gain(0, 0, 0.5);   // L -> M
ms_encode.set_gain(1, 0, 0.5);   // R -> M
ms_encode.set_gain(0, 1, 0.5);   // L -> S
ms_encode.set_gain(1, 1, -0.5);  // -R -> S

let encoder = builder.add_block(BlockType::MatrixMixer(ms_encode));
```

## Implementation Notes

- All gains default to 0.0 (silent until configured)
- Uses `ChannelConfig::Explicit` (handles routing internally)
- Panics if `inputs` or `outputs` is 0 or greater than 16
- Output is sum of all weighted inputs (may need gain reduction to avoid clipping)
