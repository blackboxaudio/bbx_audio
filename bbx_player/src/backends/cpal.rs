use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    thread,
};

use bbx_core::{Producer, SpscRingBuffer};
use cpal::{
    SampleFormat,
    traits::{DeviceTrait, HostTrait, StreamTrait},
};

use crate::{Source, backend::Backend, error::Result};

fn producer_thread_fn(mut source: Box<dyn Source<f32>>, mut producer: Producer<f32>, stop_flag: Arc<AtomicBool>) {
    while !stop_flag.load(Ordering::Relaxed) {
        while !producer.is_full() {
            if stop_flag.load(Ordering::Relaxed) {
                return;
            }
            match source.next() {
                Some(sample) => {
                    let _ = producer.try_push(sample);
                }
                None => {
                    return;
                }
            }
        }
        thread::sleep(std::time::Duration::from_micros(500));
    }
}

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
    fn play(self: Box<Self>, source: Box<dyn Source<f32>>, stop_flag: Arc<AtomicBool>) -> Result<()> {
        let sample_rate = source.sample_rate();
        let num_channels = source.channels();
        let buffer_capacity = (sample_rate as usize / 10) * num_channels as usize;
        let (producer, mut consumer) = SpscRingBuffer::new::<f32>(buffer_capacity);

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

            let stop_flag_producer = Arc::clone(&stop_flag);
            let producer_handle = thread::spawn(move || {
                producer_thread_fn(source, producer, stop_flag_producer);
            });

            let stop_flag_callback = Arc::clone(&stop_flag);

            let stream = match device.build_output_stream(
                &config,
                move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                    for sample in data.iter_mut() {
                        if stop_flag_callback.load(Ordering::Relaxed) {
                            *sample = 0.0;
                        } else {
                            *sample = consumer.try_pop().unwrap_or(0.0);
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

            while !stop_flag.load(Ordering::Relaxed) {
                std::thread::sleep(std::time::Duration::from_millis(10));
            }

            producer_handle.join().ok();
            drop(stream);
        });

        Ok(())
    }
}
