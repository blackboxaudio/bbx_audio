//! # BBX File
//!
//! Audio file I/O implementations for the `bbx_dsp` crate.
//!
//! This crate provides [`Reader`](bbx_dsp::reader::Reader) and
//! [`Writer`](bbx_dsp::writer::Writer) implementations for common audio formats.
//!
//! ## Supported Formats
//!
//! - **WAV**: Via `hound` (writing) and `wavers` (reading)
//!
//! ## Usage
//!
//! ```ignore
//! use bbx_file::readers::wav::WavFileReader;
//! use bbx_file::writers::wav::WavFileWriter;
//!
//! // Reading
//! let reader = WavFileReader::<f32>::from_path("input.wav")?;
//!
//! // Writing
//! let writer = WavFileWriter::<f32>::new("output.wav", 44100.0, 2)?;
//! ```

pub mod readers;
pub mod writers;
