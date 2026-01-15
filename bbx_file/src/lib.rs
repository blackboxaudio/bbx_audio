//! # BBX File
//!
//! Audio file I/O implementations for the `bbx_dsp` crate.
//!
//! This crate provides [`Reader`](bbx_dsp::reader::Reader) and
//! [`Writer`](bbx_dsp::writer::Writer) implementations for common audio formats,
//! as well as [`OfflineRenderer`] for fast non-realtime audio rendering.
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
//!
//! ## Offline Rendering
//!
//! ```ignore
//! use bbx_file::{OfflineRenderer, RenderDuration, writers::wav::WavFileWriter};
//!
//! let writer = WavFileWriter::new("output.wav", 44100.0, 2)?;
//! let mut renderer = OfflineRenderer::new(graph, Box::new(writer));
//! let stats = renderer.render(RenderDuration::Duration(30))?;
//! ```

pub mod readers;
pub mod renderer;
pub mod writers;

pub use renderer::{OfflineRenderer, RenderDuration, RenderError, RenderStats};
