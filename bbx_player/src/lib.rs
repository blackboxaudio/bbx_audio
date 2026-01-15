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

pub use backend::{Backend, PlayHandle};
pub use error::{PlayerError, Result};
pub use player::Player;
pub use signal::Signal;
