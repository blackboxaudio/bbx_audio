//! Audio playback abstractions for bbx_audio.
//!
//! This crate provides a simple, flexible API for playing DSP graphs
//! through the system's audio output. It supports multiple backends
//! through feature flags.
//!
//! # Features
//!
//! - `rodio` (default) - High-level backend using rodio, easiest to use
//! - `cpal` - Low-level backend using cpal directly, more control
//!
//! # Quick Start
//!
//! ```ignore
//! use bbx_player::Player;
//! use std::time::Duration;
//!
//! // Create a player with the default backend
//! let player = Player::new(graph)?;
//!
//! // Start non-blocking playback
//! let handle = player.play()?;
//!
//! // Do other work while audio plays...
//! std::thread::sleep(Duration::from_secs(5));
//!
//! // Stop playback
//! handle.stop();
//! ```
//!
//! # Using a Different Backend
//!
//! ```ignore
//! use bbx_player::{Player, backends::CpalBackend};
//!
//! let backend = CpalBackend::try_default()?;
//! let player = Player::with_backend(graph, backend);
//! let handle = player.play()?;
//! ```

mod backend;
pub mod backends;
mod error;
mod player;
mod signal;
mod source;

use std::sync::{Arc, atomic::AtomicBool};

pub use backend::{Backend, PlayHandle};
#[cfg(feature = "rodio")]
use backends::RodioBackend;
pub use error::{PlayerError, Result};
pub use player::Player;
pub use signal::Signal;
pub use source::Source;

/// Play any [`Source<f32>`] through the default audio backend.
///
/// This is a convenience function for playing custom audio sources without
/// needing to create a [`Player`] or manage backends directly.
///
/// # Examples
///
/// ```ignore
/// use bbx_player::{Source, play_source};
///
/// struct MySynth { /* ... */ }
///
/// impl Iterator for MySynth {
///     type Item = f32;
///     fn next(&mut self) -> Option<Self::Item> { /* ... */ }
/// }
///
/// impl Source<f32> for MySynth {
///     fn channels(&self) -> u16 { 2 }
///     fn sample_rate(&self) -> u32 { 44100 }
/// }
///
/// let synth = MySynth::new();
/// let handle = play_source(synth)?;
///
/// // Do other work while audio plays...
/// handle.stop();
/// ```
#[cfg(feature = "rodio")]
pub fn play_source<T: Source<f32>>(source: T) -> Result<PlayHandle> {
    let stop_flag = Arc::new(AtomicBool::new(false));
    let handle = PlayHandle::new(Arc::clone(&stop_flag));

    let backend = Box::new(RodioBackend::try_default()?);
    backend.play(Box::new(source), stop_flag)?;

    Ok(handle)
}
