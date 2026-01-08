//! # BBX Plugin
//!
//! Plugin integration crate for the bbx_audio DSP library with C FFI bindings.
//!
//! This crate re-exports the `bbx_dsp` crate and provides a macro-based API for
//! generating C-compatible FFI functions from any `PluginDsp` implementation.
//! Consumers only need to add `bbx_plugin` as a dependency to access both DSP
//! functionality and FFI generation.
//!
//! # Example
//!
//! ```ignore
//! use bbx_plugin::{PluginDsp, DspContext, bbx_plugin_ffi};
//!
//! pub struct PluginGraph { /* DSP blocks */ }
//!
//! impl PluginDsp for PluginGraph {
//!     fn new() -> Self { /* ... */ }
//!     fn prepare(&mut self, context: &DspContext) { /* ... */ }
//!     fn reset(&mut self) { /* ... */ }
//!     fn apply_parameters(&mut self, params: &[f32]) { /* ... */ }
//!     fn process(&mut self, inputs: &[&[f32]], outputs: &mut [&mut [f32]], midi_events: &[MidiEvent], context: &DspContext) { /* ... */ }
//! }
//!
//! impl Default for PluginGraph {
//!     fn default() -> Self { Self::new() }
//! }
//!
//! // Generate all FFI exports
//! bbx_plugin_ffi!(PluginGraph);
//! ```

mod audio;
mod handle;
mod macros;
pub mod params;

// Re-export the entire `bbx_*` crates as `*` so plugin projects only need bbx_plugin
pub mod core {
    pub use bbx_core::*;
}
pub mod dsp {
    pub use bbx_dsp::*;
}
pub mod midi {
    pub use bbx_midi::*;
}

// Re-export types needed by the macro
pub use core::BbxError;

pub use audio::process_audio;
pub use dsp::{PluginDsp, context::DspContext};
pub use handle::{BbxGraph, GraphInner, graph_from_handle, handle_from_graph};
// Re-export parameter utilities
pub use params::{
    JsonParamDef, ParamDef, ParamType, ParamsFile, generate_c_header_from_defs, generate_rust_indices_from_defs,
};
