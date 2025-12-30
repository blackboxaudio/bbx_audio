# ChannelRouterBlock

Flexible channel routing and manipulation.

## Overview

`ChannelRouterBlock` provides various channel routing options for stereo audio, including swapping, duplicating, and inverting channels.

## Creating a Router

```rust
use bbx_dsp::graph::GraphBuilder;
use bbx_dsp::blocks::ChannelMode;

let mut builder = GraphBuilder::<f32>::new(44100.0, 512, 2);

let router = builder.add_channel_router(ChannelMode::Stereo);
```

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
// Route left to both channels (or right)
let mono = builder.add_channel_router(ChannelMode::Left);
```

### Swap Channels

For correcting reversed cables:

```rust
let swap = builder.add_channel_router(ChannelMode::Swap);
```

### In Effect Chain

```rust
let stereo_source = /* ... */;
let router = builder.add_channel_router(ChannelMode::Stereo);
let pan = builder.add_panner(0.0);

builder.connect(stereo_source, 0, router, 0);  // Left
builder.connect(stereo_source, 1, router, 1);  // Right
builder.connect(router, 0, pan, 0);
```

## Extended Features

Some implementations support additional options:

- **Invert Phase**: Flip polarity of one or both channels
- **Mono Sum**: Mix L+R to mono
- **Mid/Side**: Convert between L/R and M/S

Check documentation for available features in your version.

## Implementation Notes

- Zero processing latency
- No allocation during process
- Simple sample copying/routing
