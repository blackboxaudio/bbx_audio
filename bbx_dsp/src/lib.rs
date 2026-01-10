//! # BBX DSP
//!
//! A block-based audio DSP system for building signal processing graphs.
//!
//! ## Overview
//!
//! This crate provides a graph-based architecture for constructing DSP chains.
//! Blocks (individual DSP units) are connected together to form a processing graph,
//! which is then executed in topologically sorted order.
//!
//! ## Core Types
//!
//! - [`Block`](block::Block) - Trait for DSP processing units
//! - [`BlockType`](block::BlockType) - Enum wrapping all block implementations
//! - [`Graph`](graph::Graph) - Container for connected blocks
//! - [`GraphBuilder`](graph::GraphBuilder) - Fluent API for graph construction
//! - [`Sample`](sample::Sample) - Trait abstracting over f32/f64
//!
//! ## Block Categories
//!
//! - **Generators**: Create audio (oscillators)
//! - **Effectors**: Transform audio (gain, overdrive, panning)
//! - **Modulators**: Generate control signals (LFOs, envelopes)
//! - **I/O**: File input/output and graph output
//!
//! ## Example
//!
//! ```ignore
//! use bbx_dsp::{graph::GraphBuilder, waveform::Waveform};
//!
//! let mut builder = GraphBuilder::<f32>::new(44100.0, 512, 2);
//! let osc = builder.add_oscillator(440.0, Waveform::Sine, None);
//! let graph = builder.build();
//! ```

#![cfg_attr(feature = "simd", feature(portable_simd))]

pub mod block;
pub mod blocks;
pub mod buffer;
pub mod context;
pub mod frame;
pub mod graph;
pub mod parameter;
pub mod plugin;
pub mod polyblep;
pub mod prelude;
pub mod reader;
pub mod sample {
    //! Audio sample type abstraction.
    //!
    //! Re-exported from `bbx_core` for convenience.
    pub use bbx_core::sample::*;
}
pub mod smoothing;
pub mod voice;
pub mod waveform;
pub mod writer;

pub use block::BlockCategory;
pub use frame::{Frame, MAX_FRAME_SAMPLES};
pub use plugin::PluginDsp;
pub use voice::VoiceState;
