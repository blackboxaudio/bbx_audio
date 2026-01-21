//! DSP block implementations.
//!
//! Blocks are organized into categories:
//! - [`generators`]: Create audio signals (oscillators)
//! - [`effectors`]: Transform audio (gain, overdrive, panning)
//! - [`modulators`]: Generate control signals (LFOs, envelopes)
//! - [`io`]: Handle file and audio I/O

pub mod effectors;
pub mod generators;
pub mod io;
pub mod modulators;

// Re-export block types for ergonomic imports
pub use effectors::{
    ambisonic_decoder::AmbisonicDecoderBlock,
    binaural_decoder::{BinauralDecoderBlock, BinauralStrategy},
    channel_merger::ChannelMergerBlock,
    channel_router::{ChannelMode, ChannelRouterBlock},
    channel_splitter::ChannelSplitterBlock,
    dc_blocker::DcBlockerBlock,
    gain::GainBlock,
    low_pass_filter::LowPassFilterBlock,
    matrix_mixer::MatrixMixerBlock,
    mixer::MixerBlock,
    overdrive::OverdriveBlock,
    panner::{PannerBlock, PannerMode},
    vca::VcaBlock,
};
pub use generators::oscillator::OscillatorBlock;
pub use io::output::OutputBlock;
#[cfg(feature = "std")]
pub use io::{file_input::FileInputBlock, file_output::FileOutputBlock};
pub use modulators::{envelope::EnvelopeBlock, lfo::LfoBlock};
