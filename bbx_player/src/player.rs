use std::sync::{Arc, atomic::AtomicBool};

use bbx_dsp::{graph::Graph, sample::Sample};

#[cfg(feature = "rodio")]
use crate::backends::RodioBackend;
use crate::{
    backend::{Backend, PlayHandle},
    error::Result,
    signal::Signal,
};

/// Audio player that plays a DSP graph through a configurable backend.
///
/// `Player` wraps a DSP [`Graph`] and handles the conversion to an
/// audio stream that can be played through the system's audio output.
///
/// # Examples
///
/// Using the default rodio backend (requires `rodio` feature):
///
/// ```ignore
/// use bbx_player::Player;
///
/// let player = Player::new(graph)?;
/// let handle = player.play()?;
///
/// std::thread::sleep(Duration::from_secs(5));
/// handle.stop();
/// ```
///
/// Using a custom backend:
///
/// ```ignore
/// use bbx_player::{Player, backends::CpalBackend};
///
/// let backend = CpalBackend::try_default()?;
/// let player = Player::with_backend(graph, backend);
/// let handle = player.play()?;
/// ```
pub struct Player<S: Sample> {
    graph: Graph<S>,
    backend: Box<dyn Backend<S>>,
}

#[cfg(feature = "rodio")]
impl<S: Sample> Player<S> {
    /// Create a new player with the default rodio backend.
    pub fn new(graph: Graph<S>) -> Result<Self> {
        let backend = RodioBackend::try_default()?;
        Ok(Self {
            graph,
            backend: Box::new(backend),
        })
    }
}

impl<S: Sample> Player<S> {
    /// Create a new player with a custom backend.
    pub fn with_backend<B: Backend<S>>(graph: Graph<S>, backend: B) -> Self {
        Self {
            graph,
            backend: Box::new(backend),
        }
    }

    /// Start non-blocking playback.
    ///
    /// Returns a [`PlayHandle`] that can be used to stop playback.
    /// The player is consumed by this method.
    pub fn play(self) -> Result<PlayHandle> {
        let stop_flag = Arc::new(AtomicBool::new(false));
        let handle = PlayHandle::new(Arc::clone(&stop_flag));

        let signal = Signal::new(self.graph, Arc::clone(&stop_flag));

        self.backend.play(Box::new(signal), stop_flag)?;

        Ok(handle)
    }
}
