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
//! - [`Graph`](graph::Graph) - Container for connected blocks (requires `std` feature)
//! - [`GraphBuilder`](graph::GraphBuilder) - Fluent API for graph construction (requires `std` feature)
//! - [`Sample`](sample::Sample) - Trait abstracting over f32/f64
//!
//! ## Block Categories
//!
//! - **Generators**: Create audio (oscillators)
//! - **Effectors**: Transform audio (gain, overdrive, panning)
//! - **Modulators**: Generate control signals (LFOs, envelopes)
//! - **I/O**: File input/output and graph output
//!
//! ## Features
//!
//! - `std` (default) - Enables standard library support and all features (Graph, prelude, file I/O)
//! - `alloc` - Enables allocation-based features (SampleBuffer)
//! - `simd` - Enables SIMD optimizations (requires `std` and nightly)
//! - `ftz-daz` - Enables flush-to-zero/denormals-are-zero for denormal handling
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

#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(feature = "simd", feature(portable_simd))]

#[cfg(feature = "alloc")]
extern crate alloc;

// Always available (no_std compatible)
pub mod block;
pub mod blocks;
pub mod channel;
pub mod context;
pub mod frame;
pub mod parameter;
pub mod polyblep;
pub mod sample {
    //! Audio sample type abstraction.
    //!
    //! Re-exported from `bbx_core` for convenience.
    pub use bbx_core::sample::*;
}
pub mod smoothing;
pub mod voice;
pub mod waveform;

pub mod math {
    //! Mathematical operations for DSP.
    //!
    //! Re-exported from `bbx_core` for convenience.
    pub use bbx_core::math::*;
}

// Requires alloc
#[cfg(feature = "alloc")]
pub mod buffer;

// Requires std (graph uses HashMap, prelude re-exports graph types)
#[cfg(feature = "std")]
pub mod graph;
#[cfg(feature = "std")]
pub mod plugin;
#[cfg(feature = "std")]
pub mod prelude;
#[cfg(feature = "std")]
pub mod reader;
#[cfg(feature = "std")]
pub mod writer;

pub use block::BlockCategory;
pub use channel::{ChannelConfig, ChannelLayout};
pub use frame::{Frame, MAX_FRAME_SAMPLES};
#[cfg(feature = "std")]
pub use plugin::PluginDsp;
pub use voice::VoiceState;
