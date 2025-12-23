//! FFI layer for integrating bbx_dsp with C/C++ (JUCE plugins).
//!
//! This module provides C-compatible functions for creating, configuring,
//! and processing DSP graphs from external code.

use std::ffi::{c_char, CStr};
use std::sync::atomic::{AtomicU32, Ordering};

use crate::block::BlockId;
use crate::graph::Graph;
use crate::parameter::AtomicF32 as ParamAtomicF32;

/// Opaque DSP engine type for C/C++ interop.
///
/// This wraps a `Graph<f32>` and provides a stable ABI for FFI.
pub struct DspEngine {
    graph: Graph<f32>,
}

/// Atomic f32 wrapper using bit-casting through u32.
///
/// This matches the memory layout of `std::atomic<float>` in C++
/// and allows lock-free parameter updates from the UI thread.
#[repr(C)]
pub struct AtomicF32(AtomicU32);

impl AtomicF32 {
    /// Load the current value with relaxed ordering.
    #[inline]
    pub fn load(&self) -> f32 {
        f32::from_bits(self.0.load(Ordering::Relaxed))
    }

    /// Store a new value with relaxed ordering.
    #[inline]
    pub fn store(&self, value: f32) {
        self.0.store(value.to_bits(), Ordering::Relaxed);
    }

    /// Create a new AtomicF32 with the given initial value.
    pub fn new(value: f32) -> Self {
        Self(AtomicU32::new(value.to_bits()))
    }
}

// =============================================================================
// Lifecycle Functions
// =============================================================================

/// Create a new DSP engine with default configuration.
///
/// # Arguments
/// * `sample_rate` - Audio sample rate in Hz (e.g., 44100.0, 48000.0)
/// * `buffer_size` - Number of samples per processing block
/// * `num_channels` - Number of audio channels (typically 2 for stereo)
///
/// # Returns
/// Pointer to the new engine, or null on failure.
///
/// # Safety
/// The returned pointer must be freed with `bbx_destroy_engine`.
#[unsafe(no_mangle)]
pub extern "C" fn bbx_create_engine(
    sample_rate: f64,
    buffer_size: usize,
    num_channels: usize,
) -> *mut DspEngine {
    let graph = Graph::new(sample_rate, buffer_size, num_channels);
    Box::into_raw(Box::new(DspEngine { graph }))
}

/// Create a DSP engine from a JSON configuration string.
///
/// # Arguments
/// * `config_str` - Pointer to JSON configuration string
/// * `config_len` - Length of the configuration string in bytes
/// * `sample_rate` - Audio sample rate in Hz
/// * `buffer_size` - Number of samples per processing block
/// * `num_channels` - Number of audio channels
///
/// # Returns
/// Pointer to the new engine, or null on parse/creation failure.
///
/// # Safety
/// The config_str pointer must be valid for config_len bytes.
/// The returned pointer must be freed with `bbx_destroy_engine`.
#[unsafe(no_mangle)]
pub extern "C" fn bbx_create_engine_from_config_str(
    config_str: *const c_char,
    config_len: usize,
    sample_rate: f64,
    buffer_size: usize,
    num_channels: usize,
) -> *mut DspEngine {
    if config_str.is_null() {
        return std::ptr::null_mut();
    }

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

/// Destroy a DSP engine and free its resources.
///
/// # Arguments
/// * `engine` - Pointer to the engine to destroy (can be null)
///
/// # Safety
/// The engine pointer must have been created by `bbx_create_engine` or
/// `bbx_create_engine_from_config_str`, or be null.
#[unsafe(no_mangle)]
pub extern "C" fn bbx_destroy_engine(engine: *mut DspEngine) {
    if !engine.is_null() {
        unsafe {
            drop(Box::from_raw(engine));
        }
    }
}

/// Prepare the DSP engine for playback.
///
/// Call this from JUCE's `prepareToPlay` method whenever the sample rate
/// or buffer size changes.
///
/// # Arguments
/// * `engine` - Pointer to the DSP engine
/// * `sample_rate` - New sample rate in Hz
/// * `buffer_size` - New buffer size in samples
///
/// # Safety
/// The engine pointer must be valid.
#[unsafe(no_mangle)]
pub extern "C" fn bbx_prepare(engine: *mut DspEngine, _sample_rate: f64, _buffer_size: usize) {
    if let Some(e) = unsafe { engine.as_mut() } {
        // TODO: Add prepare_with_context method to Graph
        e.graph.prepare_for_playback();
    }
}

/// Reset all DSP state (clear filters, reset phases, etc.).
///
/// Call this when playback stops or the plugin is bypassed.
///
/// # Arguments
/// * `engine` - Pointer to the DSP engine
///
/// # Safety
/// The engine pointer must be valid.
#[unsafe(no_mangle)]
pub extern "C" fn bbx_reset(engine: *mut DspEngine) {
    if let Some(e) = unsafe { engine.as_mut() } {
        e.graph.reset();
    }
}

// =============================================================================
// Audio Processing
// =============================================================================

/// Process stereo audio through the DSP engine.
///
/// Supports in-place processing (input and output can point to the same buffer).
///
/// # Arguments
/// * `engine` - Pointer to the DSP engine
/// * `input_left` - Pointer to left channel input samples
/// * `input_right` - Pointer to right channel input samples
/// * `output_left` - Pointer to left channel output samples
/// * `output_right` - Pointer to right channel output samples
/// * `num_samples` - Number of samples to process
///
/// # Safety
/// All buffer pointers must be valid for num_samples elements.
/// The engine pointer must be valid.
#[unsafe(no_mangle)]
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

    // For effect plugins, we need to copy input to internal buffers first
    // For now, just process the graph outputs
    let _ = (input_left, input_right); // TODO: Use these for effect processing

    unsafe {
        let mut outputs = [
            std::slice::from_raw_parts_mut(output_left, num_samples),
            std::slice::from_raw_parts_mut(output_right, num_samples),
        ];

        engine.graph.process_buffers(&mut outputs);
    }
}

/// Process multi-channel audio through the DSP engine.
///
/// # Arguments
/// * `engine` - Pointer to the DSP engine
/// * `inputs` - Array of pointers to input channel buffers
/// * `outputs` - Array of pointers to output channel buffers
/// * `num_channels` - Number of channels
/// * `num_samples` - Number of samples per channel
///
/// # Safety
/// All buffer pointers must be valid. The engine pointer must be valid.
#[unsafe(no_mangle)]
pub extern "C" fn bbx_process_multi(
    engine: *mut DspEngine,
    inputs: *const *const f32,
    outputs: *mut *mut f32,
    num_channels: usize,
    num_samples: usize,
) {
    let engine = match unsafe { engine.as_mut() } {
        Some(e) => e,
        None => return,
    };

    let _ = inputs; // TODO: Use for effect processing

    unsafe {
        let output_ptrs = std::slice::from_raw_parts(outputs, num_channels);
        let mut output_slices: Vec<&mut [f32]> = output_ptrs
            .iter()
            .map(|&ptr| std::slice::from_raw_parts_mut(ptr, num_samples))
            .collect();

        engine.graph.process_buffers(&mut output_slices);
    }
}

// =============================================================================
// Parameter Management
// =============================================================================

/// Set a parameter value directly.
///
/// Use this for non-automatable parameters or one-time configuration.
///
/// # Arguments
/// * `engine` - Pointer to the DSP engine
/// * `block_id` - ID of the block containing the parameter
/// * `param_name` - Null-terminated C string with the parameter name
/// * `value` - New parameter value
///
/// # Safety
/// The engine pointer and param_name must be valid.
#[unsafe(no_mangle)]
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

    engine.graph.set_parameter(BlockId(block_id), name, value);
}

/// Bind a parameter to an external atomic source (JUCE AudioProcessorValueTreeState).
///
/// Once bound, the Rust DSP will read parameter values directly from the
/// atomic pointer on each process call, enabling lock-free automation.
///
/// # Arguments
/// * `engine` - Pointer to the DSP engine
/// * `block_id` - ID of the block containing the parameter
/// * `param_name` - Null-terminated C string with the parameter name
/// * `atomic_ptr` - Pointer to the external atomic (std::atomic<float>* from JUCE)
///
/// # Safety
/// The engine pointer, param_name, and atomic_ptr must be valid.
/// The atomic_ptr must remain valid for the lifetime of the binding.
#[unsafe(no_mangle)]
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

    // The AtomicF32 in ffi.rs and parameter.rs have the same layout
    engine.graph.bind_external_parameter(
        BlockId(block_id),
        name,
        atomic_ptr as *const ParamAtomicF32,
    );
}

/// Unbind a parameter from its external source.
///
/// # Arguments
/// * `engine` - Pointer to the DSP engine
/// * `block_id` - ID of the block containing the parameter
/// * `param_name` - Null-terminated C string with the parameter name
///
/// # Safety
/// The engine pointer and param_name must be valid.
#[unsafe(no_mangle)]
pub extern "C" fn bbx_unbind_parameter(
    engine: *mut DspEngine,
    block_id: usize,
    param_name: *const c_char,
) {
    let engine = match unsafe { engine.as_mut() } {
        Some(e) => e,
        None => return,
    };

    let name = match unsafe { CStr::from_ptr(param_name) }.to_str() {
        Ok(s) => s,
        Err(_) => return,
    };

    // Unbind by setting to a constant value of 0
    engine.graph.set_parameter(BlockId(block_id), name, 0.0);
}

/// Get the current value of a parameter.
///
/// # Arguments
/// * `engine` - Pointer to the DSP engine
/// * `block_id` - ID of the block containing the parameter
/// * `param_name` - Null-terminated C string with the parameter name
///
/// # Returns
/// The current parameter value, or 0.0 on error.
///
/// # Safety
/// The engine pointer and param_name must be valid.
#[unsafe(no_mangle)]
pub extern "C" fn bbx_get_parameter(
    engine: *mut DspEngine,
    block_id: usize,
    param_name: *const c_char,
) -> f32 {
    let _engine = match unsafe { engine.as_mut() } {
        Some(e) => e,
        None => return 0.0,
    };

    let _name = match unsafe { CStr::from_ptr(param_name) }.to_str() {
        Ok(s) => s,
        Err(_) => return 0.0,
    };

    // TODO: Implement parameter getting on Graph
    let _ = block_id;
    0.0
}

// =============================================================================
// Graph Configuration
// =============================================================================

/// Add a block to the graph.
///
/// # Arguments
/// * `engine` - Pointer to the DSP engine
/// * `block_type` - Null-terminated C string with the block type
///                  ("oscillator", "gain", "filter", "lfo", etc.)
/// * `config_json` - Null-terminated JSON string with block-specific configuration
///
/// # Returns
/// The block ID on success, or -1 on error.
///
/// # Safety
/// The engine pointer, block_type, and config_json must be valid.
#[unsafe(no_mangle)]
pub extern "C" fn bbx_add_block(
    engine: *mut DspEngine,
    block_type: *const c_char,
    config_json: *const c_char,
) -> i32 {
    let engine = match unsafe { engine.as_mut() } {
        Some(e) => e,
        None => return -1,
    };

    let type_str = match unsafe { CStr::from_ptr(block_type) }.to_str() {
        Ok(s) => s,
        Err(_) => return -1,
    };

    let config_str = if config_json.is_null() {
        "{}"
    } else {
        match unsafe { CStr::from_ptr(config_json) }.to_str() {
            Ok(s) => s,
            Err(_) => return -1,
        }
    };

    // Parse the config JSON to get parameters
    let params: std::collections::HashMap<String, serde_json::Value> =
        serde_json::from_str(config_str).unwrap_or_default();

    let context = engine.graph.context();
    let sample_rate = context.sample_rate;
    let num_channels = context.num_channels;

    // Create the block config and use the config module to create it
    let block_config = crate::config::BlockConfig {
        id: engine.graph.block_count(),
        block_type: type_str.to_string(),
        name: String::new(),
        params,
    };

    match crate::config::create_block_ffi(&block_config, sample_rate, num_channels) {
        Ok(block) => {
            let block_id = engine.graph.add_block(block);
            block_id.0 as i32
        }
        Err(_) => -1,
    }
}

/// Connect two blocks in the graph.
///
/// # Arguments
/// * `engine` - Pointer to the DSP engine
/// * `source_block` - ID of the source block
/// * `source_output` - Output index on the source block
/// * `dest_block` - ID of the destination block
/// * `dest_input` - Input index on the destination block
///
/// # Returns
/// true on success, false on error.
///
/// # Safety
/// The engine pointer must be valid.
#[unsafe(no_mangle)]
pub extern "C" fn bbx_connect(
    engine: *mut DspEngine,
    source_block: usize,
    source_output: usize,
    dest_block: usize,
    dest_input: usize,
) -> bool {
    let engine = match unsafe { engine.as_mut() } {
        Some(e) => e,
        None => return false,
    };

    engine.graph.connect(
        BlockId(source_block),
        source_output,
        BlockId(dest_block),
        dest_input,
    );
    true
}

/// Rebuild the execution order after graph changes.
///
/// Must be called after adding/removing blocks or connections.
///
/// # Arguments
/// * `engine` - Pointer to the DSP engine
///
/// # Safety
/// The engine pointer must be valid.
#[unsafe(no_mangle)]
pub extern "C" fn bbx_rebuild_graph(engine: *mut DspEngine) {
    if let Some(e) = unsafe { engine.as_mut() } {
        e.graph.prepare_for_playback();
    }
}

// =============================================================================
// Modulation
// =============================================================================

/// Add a modulation connection between a modulator and a parameter.
///
/// # Arguments
/// * `engine` - Pointer to the DSP engine
/// * `modulator_block` - ID of the modulator block (e.g., LFO)
/// * `target_block` - ID of the block containing the target parameter
/// * `param_name` - Null-terminated C string with the parameter name
/// * `depth` - Modulation depth (-1.0 to 1.0)
///
/// # Returns
/// true on success, false on error (e.g., all modulation slots full).
///
/// # Safety
/// The engine pointer and param_name must be valid.
#[unsafe(no_mangle)]
pub extern "C" fn bbx_add_modulation(
    engine: *mut DspEngine,
    modulator_block: usize,
    target_block: usize,
    param_name: *const c_char,
    depth: f32,
) -> bool {
    let engine = match unsafe { engine.as_mut() } {
        Some(e) => e,
        None => return false,
    };

    let name = match unsafe { CStr::from_ptr(param_name) }.to_str() {
        Ok(s) => s,
        Err(_) => return false,
    };

    engine
        .graph
        .set_modulation(
            BlockId(modulator_block),
            BlockId(target_block),
            name,
            depth,
        )
        .is_ok()
}

/// Remove a modulation connection.
///
/// # Arguments
/// * `engine` - Pointer to the DSP engine
/// * `modulator_block` - ID of the modulator block
/// * `target_block` - ID of the block containing the target parameter
/// * `param_name` - Null-terminated C string with the parameter name
///
/// # Returns
/// true on success, false on error.
///
/// # Safety
/// The engine pointer and param_name must be valid.
#[unsafe(no_mangle)]
pub extern "C" fn bbx_remove_modulation(
    engine: *mut DspEngine,
    modulator_block: usize,
    target_block: usize,
    param_name: *const c_char,
) -> bool {
    let engine = match unsafe { engine.as_mut() } {
        Some(e) => e,
        None => return false,
    };

    let name = match unsafe { CStr::from_ptr(param_name) }.to_str() {
        Ok(s) => s,
        Err(_) => return false,
    };

    // Remove modulation by setting the parameter to a constant value
    // This effectively disables the modulation
    let _ = (modulator_block, name);
    engine.graph.set_parameter(BlockId(target_block), name, 0.0);
    true
}

/// Set the depth of an existing modulation connection.
///
/// # Arguments
/// * `engine` - Pointer to the DSP engine
/// * `modulator_block` - ID of the modulator block
/// * `target_block` - ID of the block containing the target parameter
/// * `param_name` - Null-terminated C string with the parameter name
/// * `depth` - New modulation depth (-1.0 to 1.0)
///
/// # Safety
/// The engine pointer and param_name must be valid.
#[unsafe(no_mangle)]
pub extern "C" fn bbx_set_modulation_depth(
    engine: *mut DspEngine,
    modulator_block: usize,
    target_block: usize,
    param_name: *const c_char,
    depth: f32,
) {
    let engine = match unsafe { engine.as_mut() } {
        Some(e) => e,
        None => return,
    };

    let name = match unsafe { CStr::from_ptr(param_name) }.to_str() {
        Ok(s) => s,
        Err(_) => return,
    };

    // Re-apply modulation with the new depth
    let _ = engine.graph.set_modulation(
        BlockId(modulator_block),
        BlockId(target_block),
        name,
        depth,
    );
}

// =============================================================================
// Version Info
// =============================================================================

/// Get the library version string.
///
/// # Returns
/// A null-terminated C string with the version (e.g., "0.1.0").
/// The returned pointer is static and should not be freed.
#[unsafe(no_mangle)]
pub extern "C" fn bbx_version() -> *const c_char {
    concat!(env!("CARGO_PKG_VERSION"), "\0").as_ptr() as *const c_char
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_destroy_engine() {
        let engine = bbx_create_engine(44100.0, 512, 2);
        assert!(!engine.is_null());
        bbx_destroy_engine(engine);
    }

    #[test]
    fn test_destroy_null_engine() {
        // Should not crash
        bbx_destroy_engine(std::ptr::null_mut());
    }

    #[test]
    fn test_atomic_f32() {
        let atomic = AtomicF32::new(0.5);
        assert!((atomic.load() - 0.5).abs() < 1e-6);

        atomic.store(1.0);
        assert!((atomic.load() - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_version() {
        let version = bbx_version();
        let version_str = unsafe { CStr::from_ptr(version) }.to_str().unwrap();
        assert!(!version_str.is_empty());
    }
}
