use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};

use rodio::{OutputStream, Source as RodioSource};

use crate::{Source, backend::Backend, error::Result};

/// High-level audio backend using rodio.
///
/// Rodio handles device selection and stream management automatically,
/// making it the easiest backend to use.
pub struct RodioBackend {
    _private: (),
}

impl RodioBackend {
    /// Create a new rodio backend using the default output device.
    pub fn try_default() -> Result<Self> {
        Ok(Self { _private: () })
    }
}

impl Backend for RodioBackend {
    fn play(self: Box<Self>, source: Box<dyn Source<f32>>, stop_flag: Arc<AtomicBool>) -> Result<()> {
        let signal_source = SignalSource::new(source, stop_flag.clone());

        std::thread::spawn(move || {
            let (_stream, stream_handle) = match OutputStream::try_default() {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("Failed to open audio output: {e}");
                    return;
                }
            };

            if let Err(e) = stream_handle.play_raw(signal_source) {
                eprintln!("Failed to start playback: {e}");
                return;
            }

            while !stop_flag.load(Ordering::SeqCst) {
                std::thread::sleep(Duration::from_millis(10));
            }
        });

        Ok(())
    }
}

struct SignalSource {
    source: Box<dyn Source<f32>>,
    stop_flag: Arc<AtomicBool>,
}

impl SignalSource {
    fn new(source: Box<dyn Source<f32>>, stop_flag: Arc<AtomicBool>) -> Self {
        Self { source, stop_flag }
    }
}

impl Iterator for SignalSource {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        if self.stop_flag.load(Ordering::SeqCst) {
            return None;
        }
        self.source.next()
    }
}

impl RodioSource for SignalSource {
    fn current_frame_len(&self) -> Option<usize> {
        None
    }

    fn channels(&self) -> u16 {
        self.source.channels()
    }

    fn sample_rate(&self) -> u32 {
        self.source.sample_rate()
    }

    fn total_duration(&self) -> Option<Duration> {
        None
    }
}
