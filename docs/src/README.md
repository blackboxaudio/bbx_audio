# bbx_audio

A Rust workspace for audio DSP with C FFI bindings for JUCE plugin integration.

> **Note**: These crates are still in early development. Expect breaking changes in some releases.

## What is bbx_audio?

bbx_audio is a collection of Rust crates designed for real-time audio digital signal processing (DSP). It provides a graph-based architecture for building audio processing chains, with first-class support for integrating Rust DSP code into C++ audio applications like JUCE plugins.

## Crates

| Crate | Description |
|-------|-------------|
| [`bbx_core`](crates/bbx-core.md) | Error types and foundational utilities |
| [`bbx_dsp`](crates/bbx-dsp.md) | DSP graph system, blocks, and `PluginDsp` trait |
| [`bbx_plugin`](crates/bbx-plugin.md) | C FFI bindings for JUCE integration |
| [`bbx_file`](crates/bbx-file.md) | Audio file I/O (WAV) |
| [`bbx_midi`](crates/bbx-midi.md) | MIDI message parsing and streaming |

## Architecture Overview

```
bbx_core (foundational utilities)
    └── bbx_dsp (DSP graph system)
            ├── bbx_file (audio file I/O via hound/wavers)
            └── bbx_plugin (FFI bindings)
bbx_midi (MIDI streaming via midir, independent)
```

The core DSP system is built around a **Graph** of connected **Blocks**:

- **`Block` trait**: Defines the DSP processing interface with `process()`, input/output counts, and modulation outputs
- **`BlockType` enum**: Wraps all concrete block implementations (oscillators, effects, I/O, modulators)
- **`Graph`**: Manages block connections, topological sorting for execution order, and buffer allocation
- **`GraphBuilder`**: Fluent API for constructing DSP graphs

## Who is this documentation for?

### Plugin Developers

If you want to use Rust for your audio plugin DSP while keeping your UI and plugin framework in C++/JUCE, start with:

1. [Quick Start](getting-started/quick-start.md) - Get a simple graph running
2. [JUCE Plugin Integration](juce/overview.md) - Full integration guide
3. [Parameter System](juce/parameters.md) - Managing plugin parameters

### Library Contributors

If you want to contribute to bbx_audio or understand its internals:

1. [Development Setup](contributing/development-setup.md) - Set up your environment
2. [DSP Graph Architecture](architecture/graph-architecture.md) - Understand the core design
3. [Adding New Blocks](contributing/new-blocks.md) - Extend the block library

## Quick Example

```rust
use bbx_dsp::{GraphBuilder, blocks::*};

// Build a simple oscillator -> gain -> output chain
let graph = GraphBuilder::new()
    .add_block(OscillatorBlock::new(440.0, Waveform::Sine))
    .add_block(GainBlock::new(-6.0))
    .add_block(OutputBlock::new(2))
    .connect(0, 0, 1, 0)?  // Oscillator -> Gain
    .connect(1, 0, 2, 0)?  // Gain -> Output
    .build()?;
```

## License

bbx_audio is licensed under the MIT License. See [LICENSE](https://github.com/blackboxaudio/bbx_audio/blob/develop/LICENSE) for details.
