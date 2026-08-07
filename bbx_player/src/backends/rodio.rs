use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};

use bbx_dsp::sample::Sample;
use rodio::{OutputStream, Source as RodioSource};

use super::SourceAdapter;
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

impl<S: Sample> Backend<S> for RodioBackend {
    fn play(self: Box<Self>, source: Box<dyn Source<S>>, stop_flag: Arc<AtomicBool>) -> Result<()> {
        let adapter = SourceAdapter::new(source);
        let signal_source = SignalSource::new(adapter, stop_flag.clone());

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

struct SignalSource<S: Sample> {
    adapter: SourceAdapter<S>,
    stop_flag: Arc<AtomicBool>,
}

impl<S: Sample> SignalSource<S> {
    fn new(adapter: SourceAdapter<S>, stop_flag: Arc<AtomicBool>) -> Self {
        Self { adapter, stop_flag }
    }
}

impl<S: Sample> Iterator for SignalSource<S> {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        if self.stop_flag.load(Ordering::SeqCst) {
            return None;
        }
        self.adapter.next()
    }
}

impl<S: Sample> RodioSource for SignalSource<S> {
    fn current_frame_len(&self) -> Option<usize> {
        None
    }

    fn channels(&self) -> u16 {
        self.adapter.channels()
    }

    fn sample_rate(&self) -> u32 {
        self.adapter.sample_rate()
    }

    fn total_duration(&self) -> Option<Duration> {
        None
    }
}
