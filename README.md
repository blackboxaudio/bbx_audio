# `bbx_audio`

[![Test](https://github.com/blackboxaudio/bbx_audio/actions/workflows/ci.test.yml/badge.svg)](https://github.com/blackboxaudio/bbx_audio/actions/workflows/ci.test.yml)
[![Clippy](https://github.com/blackboxaudio/bbx_audio/actions/workflows/ci.clippy.yml/badge.svg)](https://github.com/blackboxaudio/bbx_audio/actions/workflows/ci.clippy.yml)
[![Version: v0.3.0](https://img.shields.io/badge/Version-v0.3.0-blue.svg)](https://github.com/blackboxaudio/bbx_audio)
[![License](https://img.shields.io/badge/License-MIT-yellow)](https://github.com/blackboxaudio/bbx_audio/blob/develop/LICENSE)

A Rust workspace for audio DSP with C FFI bindings for JUCE plugin integration.

> **Note:** These crates are still in early development. Expect breaking changes in some releases.

## Overview

`bbx_audio` provides a modular DSP graph system for building audio effects and synthesizers in Rust. The workspace includes blocks for oscillators, effects, and modulators that can be connected into processing graphs. For plugin developers, `bbx_plugin` provides C FFI bindings to integrate Rust DSP into JUCE audio plugins.

## Crates

| Crate | Description |
|-------|-------------|
| [`bbx_core`](./bbx_core) | Error types and foundational utilities |
| [`bbx_draw`](./bbx_draw) | Audio visualization primitives for `nannou` |
| [`bbx_dsp`](./bbx_dsp) | DSP graph system, blocks, and `PluginDsp` trait |
| [`bbx_file`](./bbx_file) | Audio file I/O (WAV/MP3) |
| [`bbx_midi`](./bbx_midi) | MIDI messages, events, and streaming |
| [`bbx_plugin`](./bbx_plugin) | C FFI bindings for JUCE integration |
| [`bbx_sandbox`](./bbx_sandbox) | Examples and testing playground |

## Quick Start

Add dependencies to your `Cargo.toml`:

```toml
[dependencies]
bbx_dsp = { git = "https://github.com/blackboxaudio/bbx_audio" }
```

Build a simple DSP graph:

```rust
use bbx_dsp::{graph::GraphBuilder, waveform::Waveform};

let mut builder = GraphBuilder::<f32>::new(44100.0, 512, 2);

let osc = builder.add_oscillator(440.0, Waveform::Sine, None);
let drive = builder.add_overdrive(5.0, 1.0, 1.0, 44100.0);
builder.connect(osc, 0, drive, 0);

let graph = builder.build();
```

See [`bbx_sandbox/examples/`](./bbx_sandbox/examples/) for working examples, or the [Quick Start Guide](https://blackboxaudio.github.io/bbx_audio/getting-started/quick-start.html) for a complete walkthrough.

### Linux Dependencies

```bash
sudo apt install alsa libasound2-dev libssl-dev pkg-config
```

## Documentation

Full documentation is available at **[blackboxaudio.github.io/bbx_audio](https://blackboxaudio.github.io/bbx_audio/)**:

- [Getting Started](https://blackboxaudio.github.io/bbx_audio/getting-started/installation.html) - Installation and first steps
- [Tutorials](https://blackboxaudio.github.io/bbx_audio/tutorials/first-graph.html) - Graphs, oscillators, effects, modulation
- [JUCE Integration](https://blackboxaudio.github.io/bbx_audio/juce/overview.html) - Complete guide for C++ plugin integration
- [API Reference](https://blackboxaudio.github.io/bbx_audio/api/) - Rust crate documentation

## Contributing

```bash
# Clone the repository
git clone https://github.com/blackboxaudio/bbx_audio.git
cd bbx_audio

# Build all crates
cargo build --workspace

# Run tests
cargo test --workspace --release

# Run linting (requires nightly)
cargo +nightly clippy

# Format code (requires nightly)
cargo +nightly fmt
```

See the [Contributing Guide](https://blackboxaudio.github.io/bbx_audio/contributing/development-setup.html) for more details.

## License

MIT License - see [LICENSE](./LICENSE) for details.
