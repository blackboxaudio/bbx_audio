# bbx_audio

A Rust workspace for audio DSP with C FFI bindings for JUCE plugin integration.

> **Note**: These crates are still in early development. Expect breaking changes in some releases.

## What is bbx_audio?

bbx_audio is a collection of Rust crates designed for real-time audio digital signal processing (DSP). It provides a graph-based architecture for building audio processing chains, with first-class support for integrating Rust DSP code into C++ audio applications like JUCE plugins.

## Crates

| Crate                               | Description |
|-------------------------------------|-------------|
| [`bbx_core`](crates/bbx-core.md)    | Error types and foundational utilities |
| [`bbx_daisy`](crates/bbx-daisy.md)  | Electrosmith Daisy embedded audio (no_std, ARM Cortex-M) |
| [`bbx_draw`](crates/bbx-draw.md)    | Audio visualization primitives for nannou |
| [`bbx_dsp`](crates/bbx-dsp.md)      | DSP graph system, blocks, and `PluginDsp` trait |
| [`bbx_file`](crates/bbx-file.md)    | Audio file I/O (WAV) |
| [`bbx_midi`](crates/bbx-midi.md)    | MIDI message parsing and streaming |
| [`bbx_net`](crates/bbx-net.md)      | OSC and WebSocket for network audio control |
| [`bbx_player`](crates/bbx-player.md) | Audio playback with rodio (default) or cpal backends |
| [`bbx_plugin`](crates/bbx-plugin.md) | C FFI bindings for JUCE integration |
| `bbx_sandbox`                       | Examples and testing playground |

## Architecture Overview

```
bbx_core (foundational utilities)
    └── bbx_dsp (DSP graph system)
            ├── bbx_file (audio file I/O via hound/wavers)
            └── bbx_plugin (FFI bindings)
bbx_midi (MIDI streaming via midir, independent)
bbx_net (OSC/WebSocket control, independent)
```

The core DSP system is built around a **Graph** of connected **Blocks**:

- **`Block` trait**: Defines the DSP processing interface with `process()`, input/output counts, and modulation outputs
- **`BlockType` enum**: Wraps all concrete block implementations (oscillators, effects, I/O, modulators)
- **`Graph`**: Manages block connections, topological sorting for execution order, and buffer allocation
- **`GraphBuilder`**: Fluent API for constructing DSP graphs

## Who is this documentation for?

### Audio Programmers

If you're exploring Rust for audio software and want to understand real-time DSP patterns:

1. [Quick Start](getting-started/quick-start.md) - Build your first DSP graph
2. [Graph Architecture](architecture/graph-architecture.md) - Core design patterns
3. [Real-Time Safety](architecture/realtime-safety.md) - Allocation-free, lock-free processing

### Plugin Developers

If you want to use Rust for your audio plugin DSP while keeping your UI and plugin framework in C++/JUCE, start with:

1. [Quick Start](getting-started/quick-start.md) - Get a simple graph running
2. [JUCE Plugin Integration](juce/overview.md) - Full integration guide
3. [Parameter System](juce/parameters.md) - Managing plugin parameters

### Game Audio Developers

If you're building audio systems for games and want real-time safe DSP in Rust:

1. [Quick Start](getting-started/quick-start.md) - Build your first DSP graph
2. [Real-Time Safety](architecture/realtime-safety.md) - Allocation-free, lock-free processing
3. [Graph Architecture](architecture/graph-architecture.md) - Efficient audio routing

### Experimental Musicians

If you're building instruments, exploring synthesis, or experimenting with sound design:

1. [Terminal Synthesizer](tutorials/terminal-synth.md) - Build a MIDI-controlled synth from scratch
2. [Parameter Modulation](tutorials/modulation.md) - LFOs, envelopes, and modulation routing
3. [MIDI Integration](tutorials/midi.md) - Connect keyboards and controllers

### Generative Artists

If you're creating sound installations, audio visualizations, or experimenting with procedural audio:

1. [Quick Start](getting-started/quick-start.md) - Build your first DSP graph
2. [Real-Time Visualization](tutorials/visualization.md) - Visualize audio with nannou
3. [Sketch Discovery](tutorials/sketchbook.md) - Manage and organize visual sketches

### Library Contributors

If you want to contribute to bbx_audio or understand its internals:

1. [Development Setup](contributing/development-setup.md) - Set up your environment
2. [DSP Graph Architecture](architecture/graph-architecture.md) - Understand the core design
3. [Adding New Blocks](contributing/new-blocks.md) - Extend the block library

## Quick Example

```rust
use bbx_dsp::{blocks::{GainBlock, OscillatorBlock}, graph::GraphBuilder, waveform::Waveform};

// Build a simple oscillator -> gain -> output chain
let mut builder = GraphBuilder::<f32>::new(44100.0, 512, 2);

let osc = builder.add(OscillatorBlock::new(440.0, Waveform::Sine, None));
let gain = builder.add(GainBlock::new(-6.0, None));
builder.connect(osc, 0, gain, 0);

let graph = builder.build();
```

## License

bbx_audio is licensed under the MIT License. See [LICENSE](https://github.com/blackboxaudio/bbx_audio/blob/develop/LICENSE) for details.
