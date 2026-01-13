//! Opaque handle wrapper for the DSP graph.
//!
//! This module provides the opaque `BbxGraph` type that C code uses
//! to reference the Rust effects chain, along with the generic
//! `GraphInner` wrapper that holds any `PluginDsp` implementation.

use bbx_dsp::{ChannelLayout, PluginDsp, context::DspContext};

/// Opaque handle representing a DSP effects chain.
///
/// This type has zero size and prevents C code from directly
/// accessing the internal Rust structures. All operations must
/// go through the FFI functions.
#[repr(C)]
pub struct BbxGraph {
    _private: [u8; 0],
}

/// Internal wrapper holding the plugin DSP and context.
///
/// Generic over any type implementing `PluginDsp`.
pub struct GraphInner<D: PluginDsp> {
    /// DSP context with sample rate, buffer size, etc.
    pub context: DspContext,

    /// The plugin's DSP graph implementation.
    pub dsp: D,

    /// Whether the graph has been prepared for playback.
    pub prepared: bool,
}

impl<D: PluginDsp> GraphInner<D> {
    /// Create a new GraphInner with default configuration.
    pub fn new() -> Self {
        Self {
            context: DspContext {
                sample_rate: 44100.0,
                buffer_size: 512,
                num_channels: 2,
                current_sample: 0,
                channel_layout: ChannelLayout::default(),
            },
            dsp: D::new(),
            prepared: false,
        }
    }

    /// Prepare the effects chain for playback with the given audio specifications.
    ///
    /// When the `ftz-daz` feature is enabled, this also configures CPU-level
    /// FTZ/DAZ modes to prevent denormal performance penalties.
    pub fn prepare(&mut self, sample_rate: f64, buffer_size: usize, num_channels: usize) {
        #[cfg(feature = "ftz-daz")]
        bbx_core::denormal::enable_ftz_daz();

        self.context = DspContext {
            sample_rate,
            buffer_size,
            num_channels,
            current_sample: 0,
            channel_layout: ChannelLayout::default(),
        };

        self.dsp.prepare(&self.context);
        self.prepared = true;
    }

    /// Reset the effects chain state.
    pub fn reset(&mut self) {
        self.dsp.reset();
    }
}

impl<D: PluginDsp> Default for GraphInner<D> {
    fn default() -> Self {
        Self::new()
    }
}

/// Convert a raw pointer to a GraphInner reference.
///
/// # Safety
///
/// The caller must ensure:
/// - The pointer is valid and was created by `handle_from_graph` with the same DSP type.
/// - The returned reference is not used beyond the lifetime of the handle.
/// - No other mutable references to the same handle exist concurrently.
#[inline]
pub unsafe fn graph_from_handle<'a, D: PluginDsp>(handle: *mut BbxGraph) -> &'a mut GraphInner<D> {
    unsafe { &mut *(handle as *mut GraphInner<D>) }
}

/// Convert a GraphInner to an opaque handle.
#[inline]
pub fn handle_from_graph<D: PluginDsp>(inner: Box<GraphInner<D>>) -> *mut BbxGraph {
    Box::into_raw(inner) as *mut BbxGraph
}
