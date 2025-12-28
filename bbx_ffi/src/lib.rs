//! # BBX FFI
//!
//! C FFI bindings for the bbx_audio DSP library.
//!
//! This crate provides a C-compatible API for using the Rust DSP graph
//! from C++ code, specifically designed for integration with JUCE audio plugins.

mod audio;
mod handle;
mod params;

pub use audio::bbx_graph_process;
pub use handle::BbxGraph;
pub use params::*;

use bbx_core::BbxError;
use bbx_midi::MidiMessage;
use handle::{graph_from_handle, handle_from_graph, GraphInner};

// Re-export types needed by C code
pub use bbx_core::BbxError as BbxErrorCode;

// ============================================================================
// Lifecycle Functions
// ============================================================================

/// Create a new DSP graph.
///
/// Returns a handle to the graph, or null if allocation fails.
/// The graph must be destroyed with `bbx_graph_destroy` when no longer needed.
#[unsafe(no_mangle)]
pub extern "C" fn bbx_graph_create() -> *mut BbxGraph {
    let inner = Box::new(GraphInner::new());
    handle_from_graph(inner)
}

/// Destroy a DSP graph and free all associated resources.
///
/// Safe to call with a null pointer.
#[unsafe(no_mangle)]
pub extern "C" fn bbx_graph_destroy(handle: *mut BbxGraph) {
    if !handle.is_null() {
        unsafe {
            drop(Box::from_raw(handle as *mut GraphInner));
        }
    }
}

/// Prepare the graph for playback with the given audio specifications.
///
/// This must be called before processing audio, and whenever the
/// sample rate, buffer size, or channel count changes.
///
/// # Parameters
///
/// - `handle`: Valid graph handle.
/// - `sample_rate`: Sample rate in Hz (e.g., 44100.0, 48000.0).
/// - `buffer_size`: Number of samples per buffer.
/// - `num_channels`: Number of audio channels.
///
/// # Returns
///
/// `BbxError::Ok` on success, or an error code on failure.
#[unsafe(no_mangle)]
pub extern "C" fn bbx_graph_prepare(
    handle: *mut BbxGraph,
    sample_rate: f64,
    buffer_size: u32,
    num_channels: u32,
) -> BbxError {
    if handle.is_null() {
        return BbxError::NullPointer;
    }
    if buffer_size == 0 {
        return BbxError::InvalidBufferSize;
    }
    if num_channels == 0 {
        return BbxError::InvalidParameter;
    }

    unsafe {
        let inner = graph_from_handle(handle);
        inner.prepare(sample_rate, buffer_size as usize, num_channels as usize);
    }

    BbxError::Ok
}

/// Reset the graph state.
///
/// Clears all voice state, MIDI buffers, and resets DSP blocks
/// (e.g., clears delay lines, resets oscillator phases).
#[unsafe(no_mangle)]
pub extern "C" fn bbx_graph_reset(handle: *mut BbxGraph) -> BbxError {
    if handle.is_null() {
        return BbxError::NullPointer;
    }

    unsafe {
        let inner = graph_from_handle(handle);
        inner.reset();
    }

    BbxError::Ok
}

// ============================================================================
// MIDI Functions
// ============================================================================

/// Add MIDI events to the graph's buffer for processing.
///
/// Call this before `bbx_graph_process` to provide MIDI input.
/// Events are accumulated until `bbx_graph_clear_midi` is called.
///
/// # Parameters
///
/// - `handle`: Valid graph handle.
/// - `events`: Pointer to array of MIDI messages.
/// - `num_events`: Number of events in the array.
///
/// # Returns
///
/// `BbxError::Ok` on success, or an error code on failure.
///
/// # Safety
///
/// - `events` must be valid for `num_events` elements, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn bbx_graph_add_midi_events(
    handle: *mut BbxGraph,
    events: *const MidiMessage,
    num_events: u32,
) -> BbxError {
    if handle.is_null() {
        return BbxError::NullPointer;
    }

    unsafe {
        let inner = graph_from_handle(handle);

        if !events.is_null() && num_events > 0 {
            let event_slice = std::slice::from_raw_parts(events, num_events as usize);
            for event in event_slice {
                inner.midi_buffer.push(*event);
            }
        }
    }

    BbxError::Ok
}

/// Clear accumulated MIDI events.
///
/// Call this after processing to clear the MIDI buffer for the next block.
#[unsafe(no_mangle)]
pub extern "C" fn bbx_graph_clear_midi(handle: *mut BbxGraph) {
    if !handle.is_null() {
        unsafe {
            let inner = graph_from_handle(handle);
            inner.midi_buffer.clear();
        }
    }
}
