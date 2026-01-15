use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::Duration,
};

use rodio::{OutputStream, Source};

use crate::{backend::Backend, error::Result};

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
    fn play(
        self: Box<Self>,
        signal: Box<dyn Iterator<Item = f32> + Send>,
        sample_rate: u32,
        num_channels: u16,
        stop_flag: Arc<AtomicBool>,
    ) -> Result<()> {
        let source = SignalSource::new(signal, sample_rate, num_channels, stop_flag.clone());

        std::thread::spawn(move || {
            let (_stream, stream_handle) = match OutputStream::try_default() {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("Failed to open audio output: {e}");
                    return;
                }
            };

            if let Err(e) = stream_handle.play_raw(source) {
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
    signal: Box<dyn Iterator<Item = f32> + Send>,
    sample_rate: u32,
    num_channels: u16,
    stop_flag: Arc<AtomicBool>,
}

impl SignalSource {
    fn new(
        signal: Box<dyn Iterator<Item = f32> + Send>,
        sample_rate: u32,
        num_channels: u16,
        stop_flag: Arc<AtomicBool>,
    ) -> Self {
        Self {
            signal,
            sample_rate,
            num_channels,
            stop_flag,
        }
    }
}

impl Iterator for SignalSource {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        if self.stop_flag.load(Ordering::SeqCst) {
            return None;
        }
        self.signal.next()
    }
}

impl Source for SignalSource {
    fn current_frame_len(&self) -> Option<usize> {
        None
    }

    fn channels(&self) -> u16 {
        self.num_channels
    }

    fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    fn total_duration(&self) -> Option<Duration> {
        None
    }
}
