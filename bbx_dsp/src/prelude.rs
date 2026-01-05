//! Convenience re-exports for common bbx_dsp usage.
//!
//! This module provides a single import for the most commonly used types
//! when building DSP graphs.
//!
//! # Example
//!
//! ```ignore
//! use bbx_dsp::prelude::*;
//!
//! let mut builder = GraphBuilder::<f32>::new(44100.0, 512, 2);
//! let osc = builder.add_oscillator(440.0, Waveform::Sine, None);
//! let graph = builder.build();
//! ```

// Core types
// Buffers
// Configuration
// Parameters
// Smoothing (for custom block implementations)
pub use crate::{
    block::{Block, BlockId, BlockType},
    buffer::AudioBuffer,
    context::{DEFAULT_BUFFER_SIZE, DEFAULT_SAMPLE_RATE, DspContext},
    graph::{Graph, GraphBuilder},
    parameter::Parameter,
    sample::Sample,
    smoothing::{
        Linear, LinearSmoothedValue, Multiplicative, MultiplicativeSmoothedValue, SmoothedValue, SmoothingStrategy,
    },
    waveform::Waveform,
};
