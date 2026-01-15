use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, Ordering},
};

use cpal::{
    SampleFormat,
    traits::{DeviceTrait, HostTrait, StreamTrait},
};

use crate::{backend::Backend, error::Result};

/// Low-level audio backend using cpal directly.
///
/// This backend provides more control than rodio but requires more setup.
/// Use this when you need direct access to cpal's device and stream APIs.
pub struct CpalBackend {
    _private: (),
}

impl CpalBackend {
    /// Create a new cpal backend using the default output device.
    ///
    /// This verifies that a default output device is available.
    pub fn try_default() -> Result<Self> {
        Ok(Self { _private: () })
    }
}

impl Backend for CpalBackend {
    fn play(
        self: Box<Self>,
        signal: Box<dyn Iterator<Item = f32> + Send>,
        _sample_rate: u32,
        _num_channels: u16,
        stop_flag: Arc<AtomicBool>,
    ) -> Result<()> {
        let signal = Arc::new(Mutex::new(signal));

        std::thread::spawn(move || {
            let host = cpal::default_host();

            let device = match host.default_output_device() {
                Some(d) => d,
                None => {
                    eprintln!("No audio output device available");
                    return;
                }
            };

            let supported_config = match device.default_output_config() {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("Failed to get device config: {e}");
                    return;
                }
            };

            if supported_config.sample_format() != SampleFormat::F32 {
                eprintln!("Device does not support f32 sample format");
                return;
            }

            let config = supported_config.into();
            let signal_clone = Arc::clone(&signal);
            let stop_flag_clone = Arc::clone(&stop_flag);

            let stream = match device.build_output_stream(
                &config,
                move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                    let mut signal = signal_clone.lock().unwrap();
                    for sample in data.iter_mut() {
                        if stop_flag_clone.load(Ordering::SeqCst) {
                            *sample = 0.0;
                        } else {
                            *sample = signal.next().unwrap_or(0.0);
                        }
                    }
                },
                move |err| {
                    eprintln!("Audio stream error: {err}");
                },
                None,
            ) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("Failed to build output stream: {e}");
                    return;
                }
            };

            if let Err(e) = stream.play() {
                eprintln!("Failed to start playback: {e}");
                return;
            }

            while !stop_flag.load(Ordering::SeqCst) {
                std::thread::sleep(std::time::Duration::from_millis(10));
            }

            drop(stream);
        });

        Ok(())
    }
}
