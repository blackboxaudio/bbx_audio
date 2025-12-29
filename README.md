# `bbx_audio`

[![Test](https://github.com/blackboxaudio/bbx_audio/actions/workflows/ci-test.yml/badge.svg)](https://github.com/blackboxaudio/bbx_audio/actions/workflows/ci-test.yml)
[![Version: v0.1.0](https://img.shields.io/badge/Version-v0.1.0-blue.svg)](https://github.com/blackboxaudio/bbx_audio)
[![License](https://img.shields.io/badge/License-MIT-yellow)](https://github.com/blackboxaudio/bbx_audio/blob/develop/LICENSE)

> A collection of Rust crates for audio-related DSP operations 🧮

## Overview

`bbx_audio` is a collection of crates focused around audio-related DSP operations. The following crates are included within this repository:

- [`bbx_buffer`](https://github.com/blackboxaudio/bbx_audio/tree/develop/bbx_buffer) - Buffer trait, types, and operations
- [`bbx_draw`](https://github.com/blackboxaudio/bbx_audio/tree/develop/bbx_draw) - Visualization tooling
- [`bbx_dsp`](https://github.com/blackboxaudio/bbx_audio/tree/develop/bbx_dsp) - Graphs, nodes, and DSP logic
- [`bbx_file`](https://github.com/blackboxaudio/bbx_audio/tree/develop/bbx_file) - Reading / writing audio files
- [`bbx_midi`](https://github.com/blackboxaudio/bbx_audio/tree/develop/bbx_midi) - Streaming MIDI events
- [`bbx_sample`](https://github.com/blackboxaudio/bbx_audio/tree/develop/bbx_sample) - Numerical data traits, types, and operations
- [`bbx_sandbox`](https://github.com/blackboxaudio/bbx_audio/tree/develop/bbx_sandbox) - Playground for all crates

## Getting Started

Setup is quite minimal except for a few required installations on Linux-based platforms.

:information_source: If you would like to use the `bbx_draw` crate for visualizations, follow the [instruction guide](https://guide.nannou.cc/getting_started/platform-specific_setup) to setup your environment for [Nannou](https://nannou.cc/).
Otherwise, continue on to the following steps.

### Linux

Install the following packages:
```bash
sudo apt install alsa libasound2-dev libssl-dev pkg-config
```

## JUCE Plugin Integration

`bbx_ffi` provides C FFI bindings for integrating Rust DSP into JUCE audio plugins.

### Quick Start

1. **Create a `dsp/` crate in your JUCE project:**

   ```toml
   # dsp/Cargo.toml
   [package]
   name = "dsp"
   version = "0.1.0"
   edition = "2024"

   [lib]
   crate-type = ["staticlib", "cdylib"]

   [dependencies]
   bbx_dsp = { git = "https://github.com/blackboxaudio/bbx_audio" }
   bbx_ffi = { git = "https://github.com/blackboxaudio/bbx_audio" }

   [build-dependencies]
   serde = { version = "1.0", features = ["derive"] }
   serde_json = "1.0"
   ```

2. **Implement `PluginDsp` trait and invoke the macro:**

   ```rust
   // dsp/src/lib.rs
   use bbx_dsp::{PluginDsp, context::DspContext};
   use bbx_ffi::bbx_plugin_ffi;

   pub struct PluginGraph { /* your DSP blocks */ }

   impl Default for PluginGraph {
       fn default() -> Self { Self::new() }
   }

   impl PluginDsp for PluginGraph {
       fn new() -> Self { /* ... */ }
       fn prepare(&mut self, ctx: &DspContext) { /* ... */ }
       fn reset(&mut self) { /* ... */ }
       fn apply_parameters(&mut self, params: &[f32]) { /* ... */ }
       fn process(&mut self, inputs: &[&[f32]], outputs: &mut [&mut [f32]], ctx: &DspContext) { /* ... */ }
   }

   // Generate all FFI exports
   bbx_plugin_ffi!(PluginGraph);
   ```

3. **Add to CMakeLists.txt:**

   ```cmake
   add_subdirectory(vendor/corrosion)
   corrosion_import_crate(MANIFEST_PATH dsp/Cargo.toml)

   target_include_directories(${PLUGIN_TARGET} PRIVATE ${CMAKE_CURRENT_SOURCE_DIR}/dsp/include)
   target_link_libraries(${PLUGIN_TARGET} PRIVATE dsp)
   ```

4. **Include headers in C++:**

   ```cpp
   #include <bbx_ffi.h>    // FFI types and functions
   #include <bbx_params.h> // Generated parameter constants
   ```

### Local Development

For local development, create `.cargo/config.toml` (gitignored) to override git dependencies with local paths:

```toml
[patch."https://github.com/blackboxaudio/bbx_audio"]
bbx_core = { path = "/path/to/bbx_audio/bbx_core" }
bbx_dsp = { path = "/path/to/bbx_audio/bbx_dsp" }
bbx_ffi = { path = "/path/to/bbx_audio/bbx_ffi" }
bbx_midi = { path = "/path/to/bbx_audio/bbx_midi" }
```

## Using the Sandbox

To run an example in the sandbox, use the following command:

```bash
# From the bbx_sandbox/ directory
cargo run --release --example <example_name>
```
