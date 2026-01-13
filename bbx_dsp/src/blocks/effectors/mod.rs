//! `Effector`s are blocks that perform some arbitrary processing from the outputs
//! of other blocks.

pub mod ambisonic_decoder;
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
