//! I/O blocks are responsible for handling input from and output to external sources,
//! whether that is an audio file, microphone input, speaker output, and so forth.

#[cfg(feature = "std")]
pub mod file_input;
#[cfg(feature = "std")]
pub mod file_output;
pub mod output;
