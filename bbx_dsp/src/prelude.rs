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
//! let osc = builder.add(OscillatorBlock::new(440.0, Waveform::Sine, None));
//! let gain = builder.add(GainBlock::new(-6.0, None));
//! builder.connect(osc, 0, gain, 0);
//! let graph = builder.build();
//! ```

// Core types
// Block types for ergonomic graph building
pub use crate::blocks::{
    // Effectors
    AmbisonicDecoderBlock,
    BinauralDecoderBlock,
    BinauralStrategy,
    ChannelMergerBlock,
    ChannelMode,
    ChannelRouterBlock,
    ChannelSplitterBlock,
    DcBlockerBlock,
    // Modulators
    EnvelopeBlock,
    // I/O
    FileInputBlock,
    FileOutputBlock,
    GainBlock,
    LfoBlock,
    LowPassFilterBlock,
    MatrixMixerBlock,
    MixerBlock,
    // Generators
    OscillatorBlock,
    OutputBlock,
    OverdriveBlock,
    PannerBlock,
    PannerMode,
    VcaBlock,
};
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
