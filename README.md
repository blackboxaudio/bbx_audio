# `bbx_audio`

[![Clippy](https://github.com/blackboxaudio/bbx_audio/actions/workflows/ci.clippy.yml/badge.svg)](https://github.com/blackboxaudio/bbx_audio/actions/workflows/ci.clippy.yml)
[![Test](https://github.com/blackboxaudio/bbx_audio/actions/workflows/ci.test.yml/badge.svg)](https://github.com/blackboxaudio/bbx_audio/actions/workflows/ci.test.yml)
[![Version: v0.1.0](https://img.shields.io/badge/Version-v0.1.0-blue.svg)](https://github.com/blackboxaudio/bbx_audio)
[![License](https://img.shields.io/badge/License-MIT-yellow)](https://github.com/blackboxaudio/bbx_audio/blob/develop/LICENSE)

A Rust workspace for audio DSP with C FFI bindings for JUCE plugin integration.

> :warning: These crates are still in early development. Expect breaking changes in some releases.

## Crates

| Crate | Description |
|-------|-------------|
| [`bbx_core`](./bbx_core) | Error types and foundational utilities |
| [`bbx_dsp`](./bbx_dsp) | DSP graph system, blocks, and `PluginDsp` trait |
| [`bbx_ffi`](./bbx_ffi) | C FFI bindings for JUCE integration |
| [`bbx_file`](./bbx_file) | Audio file I/O (WAV/MP3) |
| [`bbx_midi`](./bbx_midi) | MIDI streaming |
| [`bbx_sandbox`](./bbx_sandbox) | Examples and testing playground |

## Getting Started

### Linux

Install required packages:

```bash
sudo apt install alsa libasound2-dev libssl-dev pkg-config
```

## JUCE Plugin Integration

### Architecture

```
JUCE AudioProcessor (C++)
         |
         v
   +-----------+
   | BbxWrapper |  <-- RAII C++ wrapper
   +-----------+
         |
         v  (C FFI calls)
   +------------+
   | bbx_ffi.h  |  <-- Generated C header
   +------------+
         |
         v
   +-------------+
   | PluginGraph |  <-- Your Rust DSP (implements PluginDsp)
   +-------------+
         |
         v
   +------------+
   | DSP Blocks |  <-- Gain, Panner, Filters, etc.
   +------------+
```

### 1. Create a `dsp/` Crate

Create a Rust crate in your JUCE project that will hold your DSP implementation:

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
```

### 2. Implement `PluginDsp`

```rust
// dsp/src/lib.rs
use bbx_dsp::{PluginDsp, context::DspContext};
use bbx_ffi::bbx_plugin_ffi;

pub struct PluginGraph {
    // Your DSP blocks here
}

impl Default for PluginGraph {
    fn default() -> Self {
        Self::new()
    }
}

impl PluginDsp for PluginGraph {
    fn new() -> Self {
        PluginGraph {
            // Initialize your DSP blocks
        }
    }

    fn prepare(&mut self, context: &DspContext) {
        // Called when sample rate or buffer size changes
        // Initialize blocks with context.sample_rate, context.buffer_size, etc.
    }

    fn reset(&mut self) {
        // Clear delay lines, filter state, etc.
    }

    fn apply_parameters(&mut self, params: &[f32]) {
        // Map parameter values to your DSP blocks
        // Index into params however you like:
        // self.gain.level_db = params[0];
        // self.panner.position = params[1];
    }

    fn process(
        &mut self,
        inputs: &[&[f32]],
        outputs: &mut [&mut [f32]],
        context: &DspContext,
    ) {
        // Process audio through your DSP chain
    }
}

// Generate FFI exports
bbx_plugin_ffi!(PluginGraph);
```

### 3. Configure CMake

Use [Corrosion](https://github.com/corrosion-rs/corrosion) to integrate Rust:

```cmake
# Add Corrosion (as a submodule in vendor/corrosion)
add_subdirectory(vendor/corrosion)
corrosion_import_crate(MANIFEST_PATH dsp/Cargo.toml)

# Include FFI headers
target_include_directories(${PLUGIN_TARGET} PRIVATE
    ${CMAKE_CURRENT_SOURCE_DIR}/dsp/include)

# Link Rust library
target_link_libraries(${PLUGIN_TARGET} PRIVATE dsp)
```

### 4. FFI Reference

Copy [`bbx_ffi.h`](./bbx_ffi/include/bbx_ffi.h) to `dsp/include/` and include it in your C++ code.

#### Types

```c
typedef struct BbxGraph BbxGraph;  // Opaque handle to Rust DSP

typedef enum BbxError {
    BBX_ERROR_OK = 0,
    BBX_ERROR_NULL_POINTER = 1,
    BBX_ERROR_INVALID_PARAMETER = 2,
    BBX_ERROR_INVALID_BUFFER_SIZE = 3,
    BBX_ERROR_GRAPH_NOT_PREPARED = 4,
    BBX_ERROR_ALLOCATION_FAILED = 5,
} BbxError;
```

#### Functions

| Function | Description |
|----------|-------------|
| `BbxGraph* bbx_graph_create(void)` | Create a new DSP graph. Returns `NULL` on allocation failure. |
| `void bbx_graph_destroy(BbxGraph* handle)` | Destroy graph and free resources. Safe to call with `NULL`. |
| `BbxError bbx_graph_prepare(BbxGraph*, double sample_rate, uint32_t buffer_size, uint32_t num_channels)` | Prepare for playback. Call from `prepareToPlay()`. |
| `BbxError bbx_graph_reset(BbxGraph*)` | Reset DSP state. Call from `releaseResources()`. |
| `void bbx_graph_process(BbxGraph*, const float* const* inputs, float* const* outputs, uint32_t num_channels, uint32_t num_samples, const float* params, uint32_t num_params)` | Process audio block. |

### 5. BbxWrapper (C++ RAII)

Create a wrapper class for safe resource management:

**bbx_wrapper.h**

```cpp
#pragma once

#include <bbx_ffi.h>

class BbxWrapper {
public:
    BbxWrapper();
    ~BbxWrapper();

    // Non-copyable
    BbxWrapper(const BbxWrapper&) = delete;
    BbxWrapper& operator=(const BbxWrapper&) = delete;

    // Movable
    BbxWrapper(BbxWrapper&& other) noexcept;
    BbxWrapper& operator=(BbxWrapper&& other) noexcept;

    BbxError Prepare(double sampleRate, uint32_t bufferSize, uint32_t numChannels);
    BbxError Reset();
    void Process(const float* const* inputs,
                 float* const* outputs,
                 uint32_t numChannels,
                 uint32_t numSamples,
                 const float* params,
                 uint32_t numParams);

    bool IsValid() const { return m_handle != nullptr; }

private:
    BbxGraph* m_handle { nullptr };
};
```

**bbx_wrapper.cpp**

```cpp
#include "bbx_wrapper.h"

BbxWrapper::BbxWrapper()
    : m_handle(bbx_graph_create())
{
}

BbxWrapper::~BbxWrapper()
{
    if (m_handle) {
        bbx_graph_destroy(m_handle);
    }
}

BbxWrapper::BbxWrapper(BbxWrapper&& other) noexcept
    : m_handle(other.m_handle)
{
    other.m_handle = nullptr;
}

BbxWrapper& BbxWrapper::operator=(BbxWrapper&& other) noexcept
{
    if (this != &other) {
        if (m_handle) {
            bbx_graph_destroy(m_handle);
        }
        m_handle = other.m_handle;
        other.m_handle = nullptr;
    }
    return *this;
}

BbxError BbxWrapper::Prepare(double sampleRate, uint32_t bufferSize, uint32_t numChannels)
{
    if (!m_handle) {
        return BBX_ERROR_NULL_POINTER;
    }
    return bbx_graph_prepare(m_handle, sampleRate, bufferSize, numChannels);
}

BbxError BbxWrapper::Reset()
{
    if (!m_handle) {
        return BBX_ERROR_NULL_POINTER;
    }
    return bbx_graph_reset(m_handle);
}

void BbxWrapper::Process(const float* const* inputs,
                         float* const* outputs,
                         uint32_t numChannels,
                         uint32_t numSamples,
                         const float* params,
                         uint32_t numParams)
{
    if (m_handle) {
        bbx_graph_process(m_handle, inputs, outputs, numChannels, numSamples, params, numParams);
    }
}
```

### 6. Processor Integration

Use BbxWrapper in your JUCE AudioProcessor:

```cpp
class PluginAudioProcessor : public juce::AudioProcessor {
private:
    BbxWrapper m_rustDsp;
    std::vector<float> m_paramBuffer;
    std::array<const float*, 8> m_inputPtrs {};
    std::array<float*, 8> m_outputPtrs {};

public:
    PluginAudioProcessor() {
        m_paramBuffer.resize(/* your param count */);
    }

    void prepareToPlay(double sampleRate, int samplesPerBlock) override {
        m_rustDsp.Prepare(sampleRate,
            static_cast<uint32_t>(samplesPerBlock),
            static_cast<uint32_t>(getTotalNumOutputChannels()));
    }

    void releaseResources() override {
        m_rustDsp.Reset();
    }

    void processBlock(juce::AudioBuffer<float>& buffer, juce::MidiBuffer&) override {
        auto numChannels = static_cast<uint32_t>(buffer.getNumChannels());
        auto numSamples = static_cast<uint32_t>(buffer.getNumSamples());

        // Gather parameters into flat array
        m_paramBuffer[0] = /* gain parameter */;
        m_paramBuffer[1] = /* pan parameter */;
        // ...

        // Build pointer arrays
        for (uint32_t ch = 0; ch < numChannels && ch < 8; ++ch) {
            m_inputPtrs[ch] = buffer.getReadPointer(static_cast<int>(ch));
            m_outputPtrs[ch] = buffer.getWritePointer(static_cast<int>(ch));
        }

        // Process through Rust DSP
        m_rustDsp.Process(
            m_inputPtrs.data(),
            m_outputPtrs.data(),
            numChannels,
            numSamples,
            m_paramBuffer.data(),
            static_cast<uint32_t>(m_paramBuffer.size()));
    }
};
```

## Local Development

To develop against a local copy of `bbx_audio`, create `.cargo/config.toml` in your plugin project (gitignored):

```toml
[patch."https://github.com/blackboxaudio/bbx_audio"]
bbx_core = { path = "/path/to/bbx_audio/bbx_core" }
bbx_dsp = { path = "/path/to/bbx_audio/bbx_dsp" }
bbx_ffi = { path = "/path/to/bbx_audio/bbx_ffi" }
bbx_midi = { path = "/path/to/bbx_audio/bbx_midi" }
```

## Using the Sandbox

Run examples from the sandbox:

```bash
cd bbx_sandbox
cargo run --release --example <example_name>
```
