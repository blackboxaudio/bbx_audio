//! Opaque handle wrapper for the DSP graph.
//!
//! This module provides the opaque `BbxGraph` type that C code uses
//! to reference the Rust effects chain, along with internal wrapper types.

use bbx_dsp::{
    blocks::effectors::{
        channel_router::{ChannelMode, ChannelRouterBlock},
        dc_blocker::DcBlockerBlock,
        gain::GainBlock,
        panner::PannerBlock,
    },
    context::DspContext,
};

use crate::params::default_params;

/// Opaque handle representing a DSP effects chain.
///
/// This type has zero size and prevents C code from directly
/// accessing the internal Rust structures. All operations must
/// go through the FFI functions.
#[repr(C)]
pub struct BbxGraph {
    _private: [u8; 0],
}

/// Internal wrapper holding the effect blocks and DSP context.
pub(crate) struct GraphInner {
    /// DSP context with sample rate, buffer size, etc.
    pub context: DspContext,

    /// DC blocker block (removes DC offset).
    pub dc_blocker: DcBlockerBlock<f32>,

    /// Channel router block (stereo routing, mono, invert).
    pub channel_router: ChannelRouterBlock<f32>,

    /// Panner block (stereo pan).
    pub panner: PannerBlock<f32>,

    /// Gain block (amplitude control).
    pub gain: GainBlock<f32>,

    /// Whether the graph has been prepared for playback.
    pub prepared: bool,
}

impl GraphInner {
    /// Create a new GraphInner with default configuration from parameters.json.
    pub fn new() -> Self {
        let defaults = default_params();

        Self {
            context: DspContext {
                sample_rate: 44100.0,
                buffer_size: 512,
                num_channels: 2,
                current_sample: 0,
            },
            dc_blocker: DcBlockerBlock::new(defaults.dc_offset),
            channel_router: ChannelRouterBlock::new(
                ChannelMode::from(defaults.channel_mode),
                defaults.mono,
                defaults.invert_left,
                defaults.invert_right,
            ),
            panner: PannerBlock::new(defaults.pan),
            gain: GainBlock::new(defaults.gain_db),
            prepared: false,
        }
    }

    /// Prepare the effects chain for playback with the given audio specifications.
    pub fn prepare(&mut self, sample_rate: f64, buffer_size: usize, num_channels: usize) {
        self.context = DspContext {
            sample_rate,
            buffer_size,
            num_channels,
            current_sample: 0,
        };

        // Update DC blocker filter coefficient for new sample rate
        self.dc_blocker.set_sample_rate(sample_rate);

        self.prepared = true;
    }

    /// Reset the effects chain state.
    pub fn reset(&mut self) {
        self.dc_blocker.reset();
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
