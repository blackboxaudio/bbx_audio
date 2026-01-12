# Multi-Channel System

This document describes how bbx_dsp handles multi-channel audio beyond basic stereo.

## Channel Layout

The `ChannelLayout` enum describes the channel configuration for audio processing:

```rust
pub enum ChannelLayout {
    Mono,           // 1 channel
    Stereo,         // 2 channels (default)
    Surround51,     // 6 channels: L, R, C, LFE, Ls, Rs
    Surround71,     // 8 channels: L, R, C, LFE, Ls, Rs, Lrs, Rrs
    AmbisonicFoa,   // 4 channels: W, Y, Z, X (1st order)
    AmbisonicSoa,   // 9 channels (2nd order)
    AmbisonicToa,   // 16 channels (3rd order)
    Custom(usize),  // Arbitrary channel count
}
```

### Channel Counts

| Layout | Channels | Description |
|--------|----------|-------------|
| Mono | 1 | Single channel |
| Stereo | 2 | Left, Right |
| Surround51 | 6 | 5.1 surround |
| Surround71 | 8 | 7.1 surround |
| AmbisonicFoa | 4 | First-order ambisonics |
| AmbisonicSoa | 9 | Second-order ambisonics |
| AmbisonicToa | 16 | Third-order ambisonics |
| Custom(n) | n | User-defined |

### Ambisonic Channel Formula

For ambisonics, channel count follows: `(order + 1)^2`

```rust
ChannelLayout::ambisonic_channel_count(1)  // 4
ChannelLayout::ambisonic_channel_count(2)  // 9
ChannelLayout::ambisonic_channel_count(3)  // 16
```

## Channel Config

The `ChannelConfig` enum describes how a block handles multi-channel audio:

```rust
pub enum ChannelConfig {
    Parallel,   // Process each channel independently (default)
    Explicit,   // Block handles channel routing internally
}
```

### Parallel Mode

Most DSP blocks use `Parallel` mode. The graph automatically duplicates the block's processing for each channel independently:

- Filters (LowPassFilter)
- Gain controls
- Distortion (Overdrive)
- DC blockers

These blocks receive the same number of input and output channels, applying identical processing to each.

### Explicit Mode

Blocks with `Explicit` config handle their own channel routing. Use this when:

- Input and output channel counts differ
- Channels need mixing or splitting
- Spatial positioning is involved

Blocks using Explicit mode:

| Block | Inputs | Outputs | Purpose |
|-------|--------|---------|---------|
| PannerBlock | 1 | 2-16 | Position mono source in spatial field |
| ChannelSplitterBlock | N | N | Separate channels for individual processing |
| ChannelMergerBlock | N | N | Combine channels back together |
| MatrixMixerBlock | N | M | Arbitrary NxM mixing with gain control |
| AmbisonicDecoderBlock | 4/9/16 | 2-8 | Decode ambisonics to speakers |
| ChannelRouterBlock | 2 | 2 | Simple stereo routing operations |

## Creating Graphs with Layouts

Use `GraphBuilder::with_layout()` to create a graph with a specific channel configuration:

```rust
use bbx_dsp::{channel::ChannelLayout, graph::GraphBuilder};

// Standard stereo
let builder = GraphBuilder::<f32>::new(44100.0, 512, 2);

// 5.1 surround
let builder = GraphBuilder::with_layout(44100.0, 512, ChannelLayout::Surround51);

// First-order ambisonics
let builder = GraphBuilder::with_layout(44100.0, 512, ChannelLayout::AmbisonicFoa);
```

The layout is stored in `DspContext` and accessible to blocks during processing.

## Implementing Channel-Aware Blocks

When implementing a custom block, override `channel_config()` if it handles channel routing:

```rust
impl<S: Sample> Block<S> for MyMixerBlock<S> {
    fn channel_config(&self) -> ChannelConfig {
        ChannelConfig::Explicit
    }

    fn input_count(&self) -> usize { 4 }
    fn output_count(&self) -> usize { 2 }

    fn process(
        &mut self,
        inputs: &[&[S]],
        outputs: &mut [&mut [S]],
        modulation_values: &[S],
        context: &DspContext,
    ) {
        // Custom mixing logic here
    }
}
```

## Buffer Limits

Multi-channel support is bounded by compile-time constants:

```rust
pub const MAX_BLOCK_INPUTS: usize = 16;
pub const MAX_BLOCK_OUTPUTS: usize = 16;
```

These limits support third-order ambisonics (16 channels) while maintaining realtime-safe stack allocation.
