use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

use bbx_dsp::sample::Sample;

use crate::{error::Result, source::Source};

/// Handle returned from `Player::play()` that allows stopping playback.
pub struct PlayHandle {
    stop_flag: Arc<AtomicBool>,
}

impl PlayHandle {
    pub(crate) fn new(stop_flag: Arc<AtomicBool>) -> Self {
        Self { stop_flag }
    }

    /// Stop playback.
    pub fn stop(&self) {
        self.stop_flag.store(true, Ordering::SeqCst);
    }

    /// Check if playback has been stopped.
    pub fn is_stopped(&self) -> bool {
        self.stop_flag.load(Ordering::SeqCst)
    }
}

/// Trait for audio playback backends.
///
/// Backends receive a [`Source<S>`] of interleaved samples and are
/// responsible for sending them to the audio output device. The generic
/// parameter allows backends to accept any sample type, with conversion
/// to the output format handled internally.
pub trait Backend<S: Sample>: Send + 'static {
    /// Start playback of the given source.
    ///
    /// This method consumes `self` (via `Box<Self>`) because backends
    /// typically need to move ownership into a background thread.
    fn play(self: Box<Self>, source: Box<dyn Source<S>>, stop_flag: Arc<AtomicBool>) -> Result<()>;
}
