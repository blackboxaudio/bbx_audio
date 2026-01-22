//! `Effector`s are blocks that perform some arbitrary processing from the outputs
//! of other blocks.

#[cfg(feature = "alloc")]
pub mod ambisonic_decoder;
#[cfg(feature = "alloc")]
pub mod binaural_decoder;
pub mod channel_merger;
pub mod channel_router;
pub mod channel_splitter;
pub mod dc_blocker;
pub mod gain;
pub mod low_pass_filter;
pub mod matrix_mixer;
pub mod mixer;
pub mod overdrive;
pub mod panner;
pub mod vca;
