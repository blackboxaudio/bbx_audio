# ChannelRouterBlock

Flexible channel routing and manipulation.

## Overview

`ChannelRouterBlock` provides various channel routing options for stereo audio, including swapping, duplicating, and inverting channels.

## Creating a Router

```rust
use bbx_dsp::{
    block::BlockType,
    blocks::{ChannelMode, ChannelRouterBlock},
    graph::GraphBuilder,
};

let mut builder = GraphBuilder::<f32>::new(44100.0, 512, 2);

// Constructor: new(mode, mono, invert_left, invert_right)
let router = builder.add_block(BlockType::ChannelRouter(
    ChannelRouterBlock::new(ChannelMode::Stereo, false, false, false)
));
```

## Constructor Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| mode | ChannelMode | Channel routing mode |
| mono | bool | Sum to mono (L+R)/2 on both channels |
| invert_left | bool | Invert left channel phase |
| invert_right | bool | Invert right channel phase |

## Channel Modes

```rust
use bbx_dsp::blocks::ChannelMode;

ChannelMode::Stereo    // Pass through unchanged
ChannelMode::Left      // Left channel to both outputs
ChannelMode::Right     // Right channel to both outputs
ChannelMode::Swap      // Swap left and right
```

### Mode Details

| Mode | Left Out | Right Out |
|------|----------|-----------|
| Stereo | Left In | Right In |
| Left | Left In | Left In |
| Right | Right In | Right In |
| Swap | Right In | Left In |

## Port Layout

| Port | Direction | Description |
|------|-----------|-------------|
| 0 | Input | Left channel |
| 1 | Input | Right channel |
| 0 | Output | Left channel |
| 1 | Output | Right channel |

## Usage Examples

### Mono Summing

For mono compatibility checking:

```rust
use bbx_dsp::{
    block::BlockType,
    blocks::{ChannelMode, ChannelRouterBlock},
    graph::GraphBuilder,
};

let mut builder = GraphBuilder::<f32>::new(44100.0, 512, 2);

// Route left to both channels
let mono = builder.add_block(BlockType::ChannelRouter(
    ChannelRouterBlock::new(ChannelMode::Left, false, false, false)
));
```

### Swap Channels

For correcting reversed cables:

```rust
use bbx_dsp::{
    block::BlockType,
    blocks::{ChannelMode, ChannelRouterBlock},
    graph::GraphBuilder,
};

let mut builder = GraphBuilder::<f32>::new(44100.0, 512, 2);

let swap = builder.add_block(BlockType::ChannelRouter(
    ChannelRouterBlock::new(ChannelMode::Swap, false, false, false)
));
```

### Phase Inversion

For polarity correction or creative effects:

```rust
use bbx_dsp::{
    block::BlockType,
    blocks::{ChannelMode, ChannelRouterBlock},
    graph::GraphBuilder,
};

let mut builder = GraphBuilder::<f32>::new(44100.0, 512, 2);

// Invert left channel phase only
let phase_flip = builder.add_block(BlockType::ChannelRouter(
    ChannelRouterBlock::new(ChannelMode::Stereo, false, true, false)
));
```

### True Mono Sum

Sum both channels to mono output:

```rust
use bbx_dsp::{
    block::BlockType,
    blocks::{ChannelMode, ChannelRouterBlock},
    graph::GraphBuilder,
};

let mut builder = GraphBuilder::<f32>::new(44100.0, 512, 2);

// Enable mono summing
let mono_sum = builder.add_block(BlockType::ChannelRouter(
    ChannelRouterBlock::new(ChannelMode::Stereo, true, false, false)
));
```

### Default Passthrough

For a simple stereo passthrough with no modifications:

```rust
use bbx_dsp::{
    block::BlockType,
    blocks::ChannelRouterBlock,
    graph::GraphBuilder,
};

let mut builder = GraphBuilder::<f32>::new(44100.0, 512, 2);

// Use default_new() for stereo passthrough
let passthrough = builder.add_block(BlockType::ChannelRouter(
    ChannelRouterBlock::default_new()
));
```

## Implementation Notes

- Zero processing latency
- No allocation during process
- Simple sample copying/routing
