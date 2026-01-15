use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

use crate::error::Result;

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
/// Backends receive an iterator of interleaved f32 samples and are
/// responsible for sending them to the audio output device.
pub trait Backend: Send + 'static {
    /// Start playback of the given signal.
    ///
    /// This method consumes `self` (via `Box<Self>`) because backends
    /// typically need to move ownership into a background thread.
    fn play(
        self: Box<Self>,
        signal: Box<dyn Iterator<Item = f32> + Send>,
        sample_rate: u32,
        num_channels: u16,
        stop_flag: Arc<AtomicBool>,
    ) -> Result<()>;
}
