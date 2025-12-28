//! Opaque handle wrapper for the DSP graph.
//!
//! This module provides the opaque `BbxGraph` type that C code uses
//! to reference the Rust Graph, along with internal wrapper types.

use bbx_dsp::{graph::Graph, VoiceState};
use bbx_midi::MidiMessageBuffer;

/// Opaque handle representing a DSP graph.
///
/// This type has zero size and prevents C code from directly
/// accessing the internal Rust structures. All operations must
/// go through the FFI functions.
#[repr(C)]
pub struct BbxGraph {
    _private: [u8; 0],
}

/// Internal wrapper holding the actual Graph<f32> and related state.
pub(crate) struct GraphInner {
    /// The DSP graph.
    pub graph: Graph<f32>,
    /// Voice state for monophonic synthesis.
    pub voice_state: VoiceState,
    /// Pre-allocated MIDI message buffer.
    pub midi_buffer: MidiMessageBuffer,
    /// Cached parameter values.
    pub param_cache: Vec<f32>,
    /// Whether the graph has been prepared for playback.
    pub prepared: bool,
}

impl GraphInner {
    /// Create a new GraphInner with default configuration.
    pub fn new() -> Self {
        Self {
            graph: Graph::new(44100.0, 512, 2),
            voice_state: VoiceState::new(),
            midi_buffer: MidiMessageBuffer::new(256),
            param_cache: Vec::with_capacity(32),
            prepared: false,
        }
    }

    /// Prepare the graph for playback with the given audio specifications.
    pub fn prepare(&mut self, sample_rate: f64, buffer_size: usize, num_channels: usize) {
        self.graph = Graph::new(sample_rate, buffer_size, num_channels);
        // TODO: Re-add blocks and connections based on configuration
        self.graph.prepare_for_playback();
        self.prepared = true;
    }

    /// Reset the graph state.
    pub fn reset(&mut self) {
        self.voice_state.reset();
        self.midi_buffer.clear();
    }
}

impl Default for GraphInner {
    fn default() -> Self {
        Self::new()
    }
}

/// Convert a raw pointer to a GraphInner reference.
///
/// # Safety
///
/// The caller must ensure the pointer is valid and was created by
/// `handle_from_graph`.
#[inline]
pub(crate) unsafe fn graph_from_handle(handle: *mut BbxGraph) -> &'static mut GraphInner {
    unsafe { &mut *(handle as *mut GraphInner) }
}

/// Convert a GraphInner to an opaque handle.
#[inline]
pub(crate) fn handle_from_graph(inner: Box<GraphInner>) -> *mut BbxGraph {
    Box::into_raw(inner) as *mut BbxGraph
}
