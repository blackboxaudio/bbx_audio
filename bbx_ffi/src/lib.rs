//! # BBX FFI
//!
//! C FFI bindings for the bbx_audio DSP library.
//!
//! This crate provides a macro-based API for generating C-compatible FFI
//! functions from any `PluginDsp` implementation. Consumers invoke the
//! `bbx_plugin_ffi!` macro with their DSP type to generate all exports.
//!
//! # Example
//!
//! ```ignore
//! use bbx_dsp::{PluginDsp, context::DspContext};
//! use bbx_ffi::bbx_plugin_ffi;
//!
//! pub struct PluginGraph { /* DSP blocks */ }
//!
//! impl PluginDsp for PluginGraph {
//!     fn new() -> Self { /* ... */ }
//!     fn prepare(&mut self, context: &DspContext) { /* ... */ }
//!     fn reset(&mut self) { /* ... */ }
//!     fn apply_parameters(&mut self, params: &[f32]) { /* ... */ }
//!     fn process(&mut self, inputs: &[&[f32]], outputs: &mut [&mut [f32]], context: &DspContext) { /* ... */ }
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

// Re-export types needed by the macro
pub use audio::process_audio;
pub use bbx_core::BbxError;
pub use bbx_dsp::PluginDsp;
// Re-export commonly used types from bbx_dsp for convenience
pub use bbx_dsp::context::DspContext;
pub use handle::{BbxGraph, GraphInner, graph_from_handle, handle_from_graph};
// Re-export parameter utilities
pub use params::{
    JsonParamDef, ParamDef, ParamType, ParamsFile, generate_c_header_from_defs, generate_rust_indices_from_defs,
};
