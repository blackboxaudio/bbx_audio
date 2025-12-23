# JUCE FFI Integration Guide

This document describes the FFI layer for integrating bbx_dsp with JUCE audio plugins. The goal is to enable full Rust DSP with a thin C++ wrapper, minimizing C++ code while maintaining compatibility with JUCE's plugin framework.

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────┐
│ JUCE C++ Layer (Thin Wrapper)                               │
│  - AudioProcessor owns Rust Graph via opaque pointer        │
│  - AudioProcessorValueTreeState for DAW parameters          │
│  - Calls Rust FFI on prepareToPlay/processBlock             │
│  - Maps JUCE atomics → Rust parameter updates               │
└─────────────────────────────────────────────────────────────┘
                           │ FFI (extern "C")
                           ▼
┌─────────────────────────────────────────────────────────────┐
│ Rust FFI Layer (bbx_dsp/src/ffi.rs)                         │
│  - #[no_mangle] C functions                                 │
│  - Opaque DspEngine pointer                                 │
│  - Parameter binding and update functions                   │
│  - Graph configuration from JSON/YAML                       │
└─────────────────────────────────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────────┐
│ Rust DSP Core (bbx_dsp)                                     │
│  - Graph execution (enum dispatch, no dyn)                  │
│  - BlockType variants for all DSP components                │
│  - Runtime modulation via ModulatableParam                  │
└─────────────────────────────────────────────────────────────┘
```

## FFI API Reference

### Lifecycle Functions

```c
// Create a new DSP engine with default configuration
DspEngine* bbx_create_engine(
    double sample_rate,
    size_t buffer_size,
    size_t num_channels
);

// Create engine from JSON/YAML configuration file
DspEngine* bbx_create_engine_from_config(
    const char* config_path,
    double sample_rate,
    size_t buffer_size,
    size_t num_channels
);

// Create engine from configuration string (JSON/YAML)
DspEngine* bbx_create_engine_from_config_str(
    const char* config_str,
    size_t config_len,
    double sample_rate,
    size_t buffer_size,
    size_t num_channels
);

// Destroy engine and free resources
void bbx_destroy_engine(DspEngine* engine);

// Prepare for playback (call from prepareToPlay)
void bbx_prepare(
    DspEngine* engine,
    double sample_rate,
    size_t buffer_size
);

// Reset all DSP state (clear filters, reset phases, etc.)
void bbx_reset(DspEngine* engine);
```

### Audio Processing

```c
// Process stereo audio (in-place supported: input and output can be same buffer)
void bbx_process(
    DspEngine* engine,
    const float* input_left,
    const float* input_right,
    float* output_left,
    float* output_right,
    size_t num_samples
);

// Process with variable channel count
void bbx_process_multi(
    DspEngine* engine,
    const float* const* inputs,
    float* const* outputs,
    size_t num_channels,
    size_t num_samples
);
```

### Parameter Management

```c
// Set parameter value directly (for non-automatable params)
void bbx_set_parameter(
    DspEngine* engine,
    size_t block_id,
    const char* param_name,
    float value
);

// Bind parameter to JUCE atomic (for automatable params)
// This allows Rust to read directly from JUCE's atomic storage
void bbx_bind_parameter(
    DspEngine* engine,
    size_t block_id,
    const char* param_name,
    const void* atomic_ptr  // std::atomic<float>*
);

// Unbind parameter from external source
void bbx_unbind_parameter(
    DspEngine* engine,
    size_t block_id,
    const char* param_name
);

// Get current parameter value
float bbx_get_parameter(
    DspEngine* engine,
    size_t block_id,
    const char* param_name
);
```

### Graph Configuration

```c
// Add a block to the graph (returns block ID, or -1 on error)
int32_t bbx_add_block(
    DspEngine* engine,
    const char* block_type,  // "oscillator", "gain", "filter", etc.
    const char* config_json  // Block-specific configuration
);

// Connect two blocks
bool bbx_connect(
    DspEngine* engine,
    size_t source_block,
    size_t source_output,
    size_t dest_block,
    size_t dest_input
);

// Disconnect blocks
bool bbx_disconnect(
    DspEngine* engine,
    size_t source_block,
    size_t source_output,
    size_t dest_block,
    size_t dest_input
);

// Remove a block from the graph
bool bbx_remove_block(DspEngine* engine, size_t block_id);

// Rebuild execution order after graph changes
void bbx_rebuild_graph(DspEngine* engine);
```

### Modulation

```c
// Connect modulator to parameter (up to 4 slots per parameter)
bool bbx_add_modulation(
    DspEngine* engine,
    size_t modulator_block,
    size_t target_block,
    const char* param_name,
    float depth  // -1.0 to 1.0
);

// Remove modulation connection
bool bbx_remove_modulation(
    DspEngine* engine,
    size_t modulator_block,
    size_t target_block,
    const char* param_name
);

// Set modulation depth
void bbx_set_modulation_depth(
    DspEngine* engine,
    size_t modulator_block,
    size_t target_block,
    const char* param_name,
    float depth
);
```

## Graph Configuration Schema

Default graph configuration loaded from JSON:

```json
{
    "blocks": [
        {
            "id": 0,
            "type": "input",
            "name": "audio_in"
        },
        {
            "id": 1,
            "type": "dc_blocker",
            "name": "dc_block",
            "params": {
                "coefficient": 0.995
            }
        },
        {
            "id": 2,
            "type": "gain",
            "name": "main_gain",
            "params": {
                "level": 0.0,
                "smoothing_ms": 20.0
            }
        },
        {
            "id": 3,
            "type": "panner",
            "name": "main_pan",
            "params": {
                "position": 0.0
            }
        },
        {
            "id": 4,
            "type": "lfo",
            "name": "mod_lfo",
            "params": {
                "frequency": 1.0,
                "waveform": "sine"
            }
        },
        {
            "id": 5,
            "type": "output",
            "name": "audio_out"
        }
    ],
    "connections": [
        { "from": [0, 0], "to": [1, 0] },
        { "from": [0, 1], "to": [1, 1] },
        { "from": [1, 0], "to": [2, 0] },
        { "from": [1, 1], "to": [2, 1] },
        { "from": [2, 0], "to": [3, 0] },
        { "from": [2, 1], "to": [3, 1] },
        { "from": [3, 0], "to": [5, 0] },
        { "from": [3, 1], "to": [5, 1] }
    ],
    "modulations": [
        {
            "source": 4,
            "target": 2,
            "param": "level",
            "depth": 0.0
        }
    ],
    "parameter_bindings": {
        "GAIN": { "block": 2, "param": "level" },
        "PAN": { "block": 3, "param": "position" },
        "LFO_RATE": { "block": 4, "param": "frequency" },
        "LFO_DEPTH": { "mod_source": 4, "mod_target": 2, "mod_param": "level" }
    }
}
```

## C++ Integration Example

### processor.h

```cpp
#pragma once
#include <juce_audio_processors/juce_audio_processors.h>
#include "bbx_dsp.h"

class PluginAudioProcessor : public juce::AudioProcessor
{
public:
    PluginAudioProcessor();
    ~PluginAudioProcessor() override;

    void prepareToPlay(double sampleRate, int samplesPerBlock) override;
    void releaseResources() override;
    void processBlock(juce::AudioBuffer<float>&, juce::MidiBuffer&) override;

    // ... other AudioProcessor methods

    juce::AudioProcessorValueTreeState& getParameters() { return m_parameters; }

private:
    DspEngine* m_dsp = nullptr;
    juce::AudioProcessorValueTreeState m_parameters;

    void bindParameters();
};
```

### processor.cpp

```cpp
#include "processor.h"

namespace {
    // Parameter IDs matching the graph config
    constexpr const char* GAIN_ID = "GAIN";
    constexpr const char* PAN_ID = "PAN";
    constexpr const char* LFO_RATE_ID = "LFO_RATE";
    constexpr const char* LFO_DEPTH_ID = "LFO_DEPTH";
}

PluginAudioProcessor::PluginAudioProcessor()
    : AudioProcessor(BusesProperties()
          .withInput("Input", juce::AudioChannelSet::stereo(), true)
          .withOutput("Output", juce::AudioChannelSet::stereo(), true))
    , m_parameters(*this, nullptr, "Parameters", createParameterLayout())
{
    // Create DSP engine from embedded config
    m_dsp = bbx_create_engine_from_config_str(
        BinaryData::graph_json,
        BinaryData::graph_jsonSize,
        44100.0, 512, 2
    );

    bindParameters();
}

PluginAudioProcessor::~PluginAudioProcessor()
{
    if (m_dsp) {
        bbx_destroy_engine(m_dsp);
    }
}

void PluginAudioProcessor::bindParameters()
{
    // Bind JUCE parameters to Rust DSP
    // The atomic pointers from JUCE are passed to Rust for direct reading

    auto* gain = m_parameters.getRawParameterValue(GAIN_ID);
    bbx_bind_parameter(m_dsp, 2, "level", gain);

    auto* pan = m_parameters.getRawParameterValue(PAN_ID);
    bbx_bind_parameter(m_dsp, 3, "position", pan);

    auto* lfoRate = m_parameters.getRawParameterValue(LFO_RATE_ID);
    bbx_bind_parameter(m_dsp, 4, "frequency", lfoRate);

    // LFO depth is a modulation depth, not a block parameter
    // This requires special handling via modulation API
    m_parameters.addParameterListener(LFO_DEPTH_ID, this);
}

void PluginAudioProcessor::prepareToPlay(double sampleRate, int samplesPerBlock)
{
    bbx_prepare(m_dsp, sampleRate, static_cast<size_t>(samplesPerBlock));
}

void PluginAudioProcessor::releaseResources()
{
    bbx_reset(m_dsp);
}

void PluginAudioProcessor::processBlock(
    juce::AudioBuffer<float>& buffer,
    juce::MidiBuffer& /* midiMessages */
)
{
    juce::ScopedNoDenormals noDenormals;

    auto* left = buffer.getWritePointer(0);
    auto* right = buffer.getWritePointer(1);
    auto numSamples = static_cast<size_t>(buffer.getNumSamples());

    // All DSP happens in Rust - just one FFI call
    bbx_process(m_dsp, left, right, left, right, numSamples);
}
```

## Rust Implementation

### ffi.rs

```rust
use std::ffi::{c_char, CStr};
use std::sync::atomic::{AtomicU32, Ordering};

use crate::graph::Graph;
use crate::sample::Sample;

/// Opaque engine type for C++
pub struct DspEngine {
    graph: Graph<f32>,
}

/// Atomic f32 wrapper (bit-cast through u32)
#[repr(C)]
pub struct AtomicF32(AtomicU32);

impl AtomicF32 {
    pub fn load(&self) -> f32 {
        f32::from_bits(self.0.load(Ordering::Relaxed))
    }
}

#[no_mangle]
pub extern "C" fn bbx_create_engine(
    sample_rate: f64,
    buffer_size: usize,
    num_channels: usize,
) -> *mut DspEngine {
    let graph = Graph::new(sample_rate, buffer_size, num_channels);
    Box::into_raw(Box::new(DspEngine { graph }))
}

#[no_mangle]
pub extern "C" fn bbx_create_engine_from_config_str(
    config_str: *const c_char,
    config_len: usize,
    sample_rate: f64,
    buffer_size: usize,
    num_channels: usize,
) -> *mut DspEngine {
    let config = unsafe {
        let slice = std::slice::from_raw_parts(config_str as *const u8, config_len);
        match std::str::from_utf8(slice) {
            Ok(s) => s,
            Err(_) => return std::ptr::null_mut(),
        }
    };

    match Graph::from_config(config, sample_rate, buffer_size, num_channels) {
        Ok(graph) => Box::into_raw(Box::new(DspEngine { graph })),
        Err(_) => std::ptr::null_mut(),
    }
}

#[no_mangle]
pub extern "C" fn bbx_destroy_engine(engine: *mut DspEngine) {
    if !engine.is_null() {
        unsafe {
            drop(Box::from_raw(engine));
        }
    }
}

#[no_mangle]
pub extern "C" fn bbx_prepare(engine: *mut DspEngine, sample_rate: f64, buffer_size: usize) {
    if let Some(e) = unsafe { engine.as_mut() } {
        e.graph.prepare_for_playback(sample_rate, buffer_size);
    }
}

#[no_mangle]
pub extern "C" fn bbx_reset(engine: *mut DspEngine) {
    if let Some(e) = unsafe { engine.as_mut() } {
        e.graph.reset();
    }
}

#[no_mangle]
pub extern "C" fn bbx_process(
    engine: *mut DspEngine,
    input_left: *const f32,
    input_right: *const f32,
    output_left: *mut f32,
    output_right: *mut f32,
    num_samples: usize,
) {
    let engine = match unsafe { engine.as_mut() } {
        Some(e) => e,
        None => return,
    };

    unsafe {
        let inputs = [
            std::slice::from_raw_parts(input_left, num_samples),
            std::slice::from_raw_parts(input_right, num_samples),
        ];
        let mut outputs = [
            std::slice::from_raw_parts_mut(output_left, num_samples),
            std::slice::from_raw_parts_mut(output_right, num_samples),
        ];

        engine.graph.process(&inputs, &mut outputs);
    }
}

#[no_mangle]
pub extern "C" fn bbx_bind_parameter(
    engine: *mut DspEngine,
    block_id: usize,
    param_name: *const c_char,
    atomic_ptr: *const AtomicF32,
) {
    let engine = match unsafe { engine.as_mut() } {
        Some(e) => e,
        None => return,
    };

    let name = match unsafe { CStr::from_ptr(param_name) }.to_str() {
        Ok(s) => s,
        Err(_) => return,
    };

    engine.graph.bind_external_parameter(block_id, name, atomic_ptr);
}

#[no_mangle]
pub extern "C" fn bbx_set_parameter(
    engine: *mut DspEngine,
    block_id: usize,
    param_name: *const c_char,
    value: f32,
) {
    let engine = match unsafe { engine.as_mut() } {
        Some(e) => e,
        None => return,
    };

    let name = match unsafe { CStr::from_ptr(param_name) }.to_str() {
        Ok(s) => s,
        Err(_) => return,
    };

    engine.graph.set_parameter(block_id, name, value);
}
```

## Parameter System

### ModulatableParam

The core parameter type supporting both external binding and modulation:

```rust
use crate::block::BlockId;
use crate::ffi::AtomicF32;
use crate::sample::Sample;

/// A parameter that can be externally bound and modulated
pub struct ModulatableParam<S: Sample, const N: usize = 4> {
    /// Base value (used when no external source)
    base_value: S,

    /// External atomic source (from JUCE)
    external_source: Option<*const AtomicF32>,

    /// Modulation slots: (source_block, depth)
    modulation_slots: [(Option<BlockId>, S); N],
}

impl<S: Sample, const N: usize> ModulatableParam<S, N> {
    pub fn new(value: S) -> Self {
        Self {
            base_value: value,
            external_source: None,
            modulation_slots: [(None, S::ZERO); N],
        }
    }

    /// Set base value
    pub fn set(&mut self, value: S) {
        self.base_value = value;
    }

    /// Bind to external atomic (JUCE parameter)
    pub fn bind_external(&mut self, source: *const AtomicF32) {
        self.external_source = Some(source);
    }

    /// Unbind from external source
    pub fn unbind_external(&mut self) {
        self.external_source = None;
    }

    /// Add modulation source
    pub fn add_modulation(&mut self, source: BlockId, depth: S) -> bool {
        for slot in &mut self.modulation_slots {
            if slot.0.is_none() {
                *slot = (Some(source), depth);
                return true;
            }
        }
        false // All slots full
    }

    /// Remove modulation source
    pub fn remove_modulation(&mut self, source: BlockId) -> bool {
        for slot in &mut self.modulation_slots {
            if slot.0 == Some(source) {
                *slot = (None, S::ZERO);
                return true;
            }
        }
        false
    }

    /// Set modulation depth for existing connection
    pub fn set_modulation_depth(&mut self, source: BlockId, depth: S) {
        for slot in &mut self.modulation_slots {
            if slot.0 == Some(source) {
                slot.1 = depth;
                return;
            }
        }
    }

    /// Evaluate parameter with modulation
    pub fn evaluate(&self, modulation_values: &[S]) -> S {
        // Start with base or external value
        let mut value = match self.external_source {
            Some(ptr) => unsafe { S::from_f64((*ptr).load() as f64) },
            None => self.base_value,
        };

        // Add modulation contributions
        for (source, depth) in &self.modulation_slots {
            if let Some(id) = source {
                value = value + modulation_values[id.0] * *depth;
            }
        }

        value
    }
}

// Safety: The external pointer is only read, never written
unsafe impl<S: Sample, const N: usize> Send for ModulatableParam<S, N> {}
unsafe impl<S: Sample, const N: usize> Sync for ModulatableParam<S, N> {}
```

## Build Configuration

### Cargo.toml additions

```toml
[lib]
crate-type = ["staticlib", "rlib"]

[dependencies]
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
serde_yaml = "0.9"

[build-dependencies]
cbindgen = "0.26"
```

### cbindgen.toml

```toml
language = "C"
header = "/* Auto-generated by cbindgen. Do not edit. */"
include_guard = "BBX_DSP_H"
autogen_warning = "/* Warning: this file is auto-generated. Do not modify. */"

[export]
prefix = "bbx_"
include = ["DspEngine"]

[export.rename]
"AtomicF32" = "bbx_AtomicF32"

[fn]
prefix = "BBX_"
```

### build.rs

```rust
fn main() {
    let crate_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();

    cbindgen::Builder::new()
        .with_crate(&crate_dir)
        .with_config(cbindgen::Config::from_file("cbindgen.toml").unwrap())
        .generate()
        .expect("Unable to generate bindings")
        .write_to_file("bbx_dsp.h");
}
```

## CMake Integration

### CMakeLists.txt additions

```cmake
# Find Rust/Cargo
find_program(CARGO cargo REQUIRED)

# Rust library paths
set(RUST_TARGET_DIR "${CMAKE_SOURCE_DIR}/../bbx_audio/target")
set(RUST_LIB_NAME "bbx_dsp")

if(CMAKE_BUILD_TYPE STREQUAL "Debug")
    set(RUST_BUILD_TYPE "debug")
    set(RUST_BUILD_FLAG "")
else()
    set(RUST_BUILD_TYPE "release")
    set(RUST_BUILD_FLAG "--release")
endif()

# Build Rust library
add_custom_target(rust_dsp
    COMMAND ${CARGO} build ${RUST_BUILD_FLAG}
        --manifest-path ${CMAKE_SOURCE_DIR}/../bbx_audio/Cargo.toml
    WORKING_DIRECTORY ${CMAKE_SOURCE_DIR}/../bbx_audio
    COMMENT "Building Rust DSP library"
)

# Platform-specific library name
if(APPLE)
    set(RUST_LIB_PATH "${RUST_TARGET_DIR}/${RUST_BUILD_TYPE}/lib${RUST_LIB_NAME}.a")
elseif(WIN32)
    set(RUST_LIB_PATH "${RUST_TARGET_DIR}/${RUST_BUILD_TYPE}/${RUST_LIB_NAME}.lib")
else()
    set(RUST_LIB_PATH "${RUST_TARGET_DIR}/${RUST_BUILD_TYPE}/lib${RUST_LIB_NAME}.a")
endif()

# Import Rust library
add_library(bbx_dsp STATIC IMPORTED)
set_target_properties(bbx_dsp PROPERTIES
    IMPORTED_LOCATION ${RUST_LIB_PATH}
)
add_dependencies(bbx_dsp rust_dsp)

# Link to plugin
target_link_libraries(${PLUGIN_NAME} PRIVATE bbx_dsp)

# Include header directory
target_include_directories(${PLUGIN_NAME} PRIVATE
    ${CMAKE_SOURCE_DIR}/../bbx_audio/bbx_dsp
)

# Additional system libraries for Rust runtime (macOS)
if(APPLE)
    target_link_libraries(${PLUGIN_NAME} PRIVATE
        "-framework Security"
        "-framework CoreFoundation"
    )
endif()
```

## Effect Blocks

### GainBlock

```rust
pub struct GainBlock<S: Sample> {
    pub level: ModulatableParam<S>,
    smoothed_gain: SmoothedValue<S>,
    smoothing_ms: f64,
}

impl<S: Sample> GainBlock<S> {
    pub fn new(level_db: S, smoothing_ms: f64) -> Self {
        Self {
            level: ModulatableParam::new(level_db),
            smoothed_gain: SmoothedValue::new(db_to_linear(level_db)),
            smoothing_ms,
        }
    }
}

impl<S: Sample> Block<S> for GainBlock<S> {
    fn process(
        &mut self,
        inputs: &[&[S]],
        outputs: &mut [&mut [S]],
        modulation_values: &[S],
        context: &DspContext,
    ) {
        let target_db = self.level.evaluate(modulation_values);
        let target_linear = db_to_linear(target_db);
        self.smoothed_gain.set_target(target_linear, context.sample_rate, self.smoothing_ms);

        for (input, output) in inputs.iter().zip(outputs.iter_mut()) {
            for (in_sample, out_sample) in input.iter().zip(output.iter_mut()) {
                *out_sample = *in_sample * self.smoothed_gain.next();
            }
        }
    }

    fn input_count(&self) -> usize { 2 }
    fn output_count(&self) -> usize { 2 }
    fn modulation_outputs(&self) -> &[ModulationOutput] { &[] }
}
```

### FilterBlock

```rust
pub enum FilterMode {
    LowPass,
    HighPass,
    BandPass,
}

pub struct FilterBlock<S: Sample> {
    pub cutoff: ModulatableParam<S>,
    pub resonance: ModulatableParam<S>,
    pub mode: FilterMode,

    // State variables (per channel)
    z1: [S; 2],
    z2: [S; 2],
}

impl<S: Sample> FilterBlock<S> {
    pub fn new(cutoff_hz: S, resonance: S, mode: FilterMode) -> Self {
        Self {
            cutoff: ModulatableParam::new(cutoff_hz),
            resonance: ModulatableParam::new(resonance),
            mode,
            z1: [S::ZERO; 2],
            z2: [S::ZERO; 2],
        }
    }

    fn process_sample(&mut self, input: S, channel: usize, cutoff: S, q: S, sample_rate: f64) -> S {
        // SVF (State Variable Filter) implementation
        let g = (std::f64::consts::PI * cutoff.to_f64() / sample_rate).tan();
        let k = S::from_f64(1.0 / q.to_f64());

        let g_s = S::from_f64(g);
        let one = S::ONE;

        let hp = (input - self.z1[channel] * (g_s + k) - self.z2[channel])
            / (one + g_s * (g_s + k));
        let bp = hp * g_s + self.z1[channel];
        let lp = bp * g_s + self.z2[channel];

        self.z1[channel] = hp * g_s + bp;
        self.z2[channel] = bp * g_s + lp;

        match self.mode {
            FilterMode::LowPass => lp,
            FilterMode::HighPass => hp,
            FilterMode::BandPass => bp,
        }
    }
}
```

### PannerBlock

```rust
pub struct PannerBlock<S: Sample> {
    pub position: ModulatableParam<S>,  // -1.0 (left) to 1.0 (right)
}

impl<S: Sample> PannerBlock<S> {
    pub fn new(position: S) -> Self {
        Self {
            position: ModulatableParam::new(position),
        }
    }
}

impl<S: Sample> Block<S> for PannerBlock<S> {
    fn process(
        &mut self,
        inputs: &[&[S]],
        outputs: &mut [&mut [S]],
        modulation_values: &[S],
        context: &DspContext,
    ) {
        let pos = self.position.evaluate(modulation_values);
        let pos_f64 = pos.to_f64().clamp(-1.0, 1.0);

        // Equal power panning
        let angle = (pos_f64 + 1.0) * std::f64::consts::FRAC_PI_4;
        let left_gain = S::from_f64(angle.cos());
        let right_gain = S::from_f64(angle.sin());

        if inputs.len() >= 2 && outputs.len() >= 2 {
            for i in 0..inputs[0].len().min(outputs[0].len()) {
                outputs[0][i] = inputs[0][i] * left_gain;
                outputs[1][i] = inputs[1][i] * right_gain;
            }
        }
    }

    fn input_count(&self) -> usize { 2 }
    fn output_count(&self) -> usize { 2 }
    fn modulation_outputs(&self) -> &[ModulationOutput] { &[] }
}
```

### DcBlockerBlock

```rust
pub struct DcBlockerBlock<S: Sample> {
    coefficient: S,  // Typically 0.995
    x_prev: [S; 2],
    y_prev: [S; 2],
}

impl<S: Sample> DcBlockerBlock<S> {
    pub fn new(coefficient: S) -> Self {
        Self {
            coefficient,
            x_prev: [S::ZERO; 2],
            y_prev: [S::ZERO; 2],
        }
    }
}

impl<S: Sample> Block<S> for DcBlockerBlock<S> {
    fn process(
        &mut self,
        inputs: &[&[S]],
        outputs: &mut [&mut [S]],
        _modulation_values: &[S],
        _context: &DspContext,
    ) {
        for (ch, (input, output)) in inputs.iter().zip(outputs.iter_mut()).enumerate() {
            for (x, y) in input.iter().zip(output.iter_mut()) {
                // y[n] = x[n] - x[n-1] + R * y[n-1]
                *y = *x - self.x_prev[ch] + self.coefficient * self.y_prev[ch];
                self.x_prev[ch] = *x;
                self.y_prev[ch] = *y;
            }
        }
    }

    fn input_count(&self) -> usize { 2 }
    fn output_count(&self) -> usize { 2 }
    fn modulation_outputs(&self) -> &[ModulationOutput] { &[] }
}
```

## Testing

### Rust Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gain_block() {
        let mut gain = GainBlock::new(0.0, 0.0);  // 0 dB, no smoothing
        let input = [vec![0.5f32; 64], vec![0.5f32; 64]];
        let mut output = [vec![0.0f32; 64], vec![0.0f32; 64]];

        let context = DspContext::new(44100.0, 64, 2);
        gain.process(
            &[&input[0], &input[1]],
            &mut [&mut output[0], &mut output[1]],
            &[],
            &context,
        );

        // 0 dB = unity gain
        assert!((output[0][63] - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_modulatable_param() {
        let mut param = ModulatableParam::<f32, 4>::new(100.0);

        // Add two modulation sources
        assert!(param.add_modulation(BlockId(0), 10.0));
        assert!(param.add_modulation(BlockId(1), -5.0));

        // Modulation values: [0.5, 1.0, ...]
        let mod_values = vec![0.5f32, 1.0, 0.0, 0.0];

        // Result: 100.0 + (0.5 * 10.0) + (1.0 * -5.0) = 100.0 + 5.0 - 5.0 = 100.0
        let result = param.evaluate(&mod_values);
        assert!((result - 100.0).abs() < 0.001);
    }
}
```

### C++ Integration Test

```cpp
#include <gtest/gtest.h>
#include "bbx_dsp.h"

TEST(BbxDspTest, CreateAndDestroy) {
    auto* engine = bbx_create_engine(44100.0, 512, 2);
    ASSERT_NE(engine, nullptr);
    bbx_destroy_engine(engine);
}

TEST(BbxDspTest, ProcessSilence) {
    auto* engine = bbx_create_engine(44100.0, 512, 2);

    std::vector<float> left(512, 0.0f);
    std::vector<float> right(512, 0.0f);

    bbx_process(engine, left.data(), right.data(),
                left.data(), right.data(), 512);

    // Should remain silent
    for (auto sample : left) {
        EXPECT_NEAR(sample, 0.0f, 1e-6f);
    }

    bbx_destroy_engine(engine);
}
```
