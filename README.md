# `bbx_audio`

[![Test](https://github.com/blackboxaudio/bbx_audio/actions/workflows/ci.test.yml/badge.svg)](https://github.com/blackboxaudio/bbx_audio/actions/workflows/ci.test.yml)
[![Clippy](https://github.com/blackboxaudio/bbx_audio/actions/workflows/ci.clippy.yml/badge.svg)](https://github.com/blackboxaudio/bbx_audio/actions/workflows/ci.clippy.yml)
[![Version: v0.4.2](https://img.shields.io/badge/Version-v0.4.2-blue.svg)](https://github.com/blackboxaudio/bbx_audio)
[![License](https://img.shields.io/badge/License-MIT-yellow)](https://github.com/blackboxaudio/bbx_audio/blob/develop/LICENSE)

A modular, real-time safe audio toolkit in Rust.

> **Note:** These crates are still in early development. Expect breaking changes in some releases.

## Overview

`bbx_audio` provides a real-time safe DSP graph system for building synthesizers, effects, and spatial audio in Rust. The audio components are lock-free and allocation-free by design.

Connect blocks for oscillators, filters, modulators, and routing into processing graphs. The workspace supports mono through ambisonic layouts, MIDI input, OSC/WebSocket control, audio file I/O, and visualization. For plugin developers, `bbx_plugin` provides C FFI bindings to integrate with JUCE or any C/C++ host.

Optional SIMD optimizations are available via the `simd` feature flag (requires nightly Rust).

## Features

- **Real-Time Safe**: Lock-free, allocation-free audio thread with pre-allocated buffers
- **Block-Graph DSP**: Connect oscillators, filters, effects, and modulators into processing graphs
- **Spatial Audio**: Mono through ambisonics (FOA/SOA/TOA), surround (5.1/7.1), and binaural rendering
- **Network Control**: OSC and WebSocket for TouchOSC, Max/MSP, and web/mobile apps
- **MIDI Support**: Message parsing and real-time streaming with sample-accurate timing
- **Visualization**: Waveforms, spectrum analyzers, and graph topology viewers (nannou)
- **Plugin Ready**: C FFI bindings for JUCE or any C/C++ host

## Crates

| Crate | Description |
|-------|-------------|
| [`bbx_core`](./bbx_core) | `Sample` trait, `StackVec`, lock-free ring buffers, error types |
| [`bbx_dsp`](./bbx_dsp) | Block-graph engine with oscillators, filters, panners, mixers, ambisonics |
| [`bbx_draw`](./bbx_draw) | Waveforms, spectrum analyzers, graph topology viewers (nannou) |
| [`bbx_file`](./bbx_file) | Audio file I/O (WAV/MP3) |
| [`bbx_midi`](./bbx_midi) | MIDI parsing, events, and real-time streaming |
| [`bbx_net`](./bbx_net) | OSC + WebSocket for TouchOSC, Max/MSP, web/mobile control; includes [`@bbx-audio/net`](https://www.npmjs.com/package/@bbx-audio/net) TypeScript client |
| [`bbx_plugin`](./bbx_plugin) | C FFI bindings for JUCE or any C/C++ host |
| [`bbx_sandbox`](./bbx_sandbox) | Examples and testing playground |

## Quick Start

Add dependencies to your `Cargo.toml`:

```toml
[dependencies]
bbx_dsp = { git = "https://github.com/blackboxaudio/bbx_audio" }
```

Build a simple DSP graph:

```rust
use bbx_dsp::{blocks::{OscillatorBlock, OverdriveBlock}, graph::GraphBuilder, waveform::Waveform};

let mut builder = GraphBuilder::<f32>::new(44100.0, 512, 2);

let osc = builder.add(OscillatorBlock::new(440.0, Waveform::Sine, None));
let drive = builder.add(OverdriveBlock::new(5.0, 1.0, 1.0, 44100.0));
builder.connect(osc, 0, drive, 0);

let graph = builder.build();
```

See [`bbx_sandbox/examples/`](./bbx_sandbox/examples/) for working examples, or the [Quick Start Guide](https://docs.bbx-audio.com/getting-started/quick-start.html) for a complete walkthrough.

### Linux Dependencies

```bash
sudo apt install libasound2-dev libssl-dev pkg-config
```

## Examples

The [`bbx_sandbox`](./bbx_sandbox/examples/) crate includes examples covering the major features:

```bash
cargo run --example 01_sine_wave -p bbx_sandbox        # Basic oscillator
cargo run --example 06_lfo_modulation -p bbx_sandbox   # Modulation
cargo run --example 08_ambisonic_panner -p bbx_sandbox # Spatial audio
cargo run --example 14_osc_synth -p bbx_sandbox        # OSC control
cargo run --example 15_ws_synth -p bbx_sandbox         # WebSocket control
```

## Documentation

Full documentation is available at **[docs.bbx-audio.com](https://docs.bbx-audio.com/)**:

- [Getting Started](https://docs.bbx-audio.com/getting-started/installation.html) - Installation and first steps
- [Tutorials](https://docs.bbx-audio.com/tutorials/first-graph.html) - Graphs, oscillators, effects, modulation
- [JUCE Integration](https://docs.bbx-audio.com/juce/overview.html) - Complete guide for C++ plugin integration
- [API Reference](https://docs.bbx-audio.com/api/) - Rust crate documentation

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

See the [Contributing Guide](https://docs.bbx-audio.com/contributing/development-setup.html) for more details.

## License

MIT License - see [LICENSE](./LICENSE) for details.
