use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    thread::{self, JoinHandle},
};

use bbx_core::{Consumer, Producer, SpscRingBuffer};

use crate::{block::Block, context::DspContext, parameter::ModulationOutput, sample::Sample, writer::Writer};

/// Default ring buffer capacity in samples (~1 second at 44.1kHz stereo).
const DEFAULT_RING_BUFFER_CAPACITY: usize = 44100 * 2;

/// Used for writing i.e. rendering audio files from a DSP `Graph`.
///
/// This block uses a separate I/O thread to perform disk writes,
/// avoiding blocking the audio thread. Samples are passed through
/// a lock-free ring buffer.
pub struct FileOutputBlock<S: Sample> {
    /// Ring buffer producer (audio thread side).
    producer: Option<Producer<S>>,

    /// Writer thread handle.
    writer_thread: Option<JoinHandle<()>>,

    /// Signal for writer thread to stop.
    stop_signal: Arc<AtomicBool>,

    /// Flag indicating a write error occurred.
    error_flag: Arc<AtomicBool>,

    /// Recording state.
    is_recording: bool,

    /// Number of audio channels.
    num_channels: usize,
}

impl<S: Sample + Send + 'static> FileOutputBlock<S> {
    /// Create a `FileOutputBlock` with the `Writer` implementation for a particular type of audio file.
    ///
    /// The writer is moved to a background I/O thread. Samples are passed through
    /// a lock-free ring buffer to avoid blocking the audio thread.
    pub fn new(writer: Box<dyn Writer<S>>) -> Self {
        let num_channels = writer.num_channels();
        let sample_rate = writer.sample_rate() as usize;

        // Ring buffer capacity: ~1 second of audio
        let buffer_capacity = sample_rate.max(DEFAULT_RING_BUFFER_CAPACITY) * num_channels;
        let (producer, consumer) = SpscRingBuffer::new::<S>(buffer_capacity);

        let stop_signal = Arc::new(AtomicBool::new(false));
        let error_flag = Arc::new(AtomicBool::new(false));

        let stop_signal_clone = stop_signal.clone();
        let error_flag_clone = error_flag.clone();

        // Spawn I/O thread
        let writer_thread = thread::spawn(move || {
            Self::writer_thread_fn(consumer, writer, stop_signal_clone, error_flag_clone, num_channels);
        });

        Self {
            producer: Some(producer),
            writer_thread: Some(writer_thread),
            stop_signal,
            error_flag,
            is_recording: true,
            num_channels,
        }
    }

    /// Writer thread function that consumes samples and writes to disk.
    fn writer_thread_fn(
        mut consumer: Consumer<S>,
        mut writer: Box<dyn Writer<S>>,
        stop_signal: Arc<AtomicBool>,
        error_flag: Arc<AtomicBool>,
        num_channels: usize,
    ) {
        // Per-channel buffers for de-interleaving
        let mut channel_buffers: Vec<Vec<S>> = vec![Vec::new(); num_channels];
        let mut current_channel = 0;

        // Flush threshold (samples per channel before writing to disk)
        const FLUSH_THRESHOLD: usize = 4096;

        loop {
            // Try to consume samples (non-blocking)
            while let Some(sample) = consumer.try_pop() {
                channel_buffers[current_channel].push(sample);
                current_channel = (current_channel + 1) % num_channels;

                // Flush to disk periodically
                if channel_buffers[0].len() >= FLUSH_THRESHOLD {
                    for (ch, buffer) in channel_buffers.iter_mut().enumerate() {
                        if writer.write_channel(ch, buffer).is_err() {
                            error_flag.store(true, Ordering::Relaxed);
                        }
                        buffer.clear();
                    }
                }
            }

            // Check stop signal
            if stop_signal.load(Ordering::Relaxed) {
                // Flush remaining samples
                for (ch, buffer) in channel_buffers.iter().enumerate() {
                    if !buffer.is_empty() && writer.write_channel(ch, buffer).is_err() {
                        error_flag.store(true, Ordering::Relaxed);
                    }
                }

                // Finalize the writer
                if writer.finalize().is_err() {
                    error_flag.store(true, Ordering::Relaxed);
                }

                break;
            }

            // Sleep briefly to avoid busy-waiting
            thread::sleep(std::time::Duration::from_millis(1));
        }
    }

    /// Tell the writer to begin storing audio sample data.
    #[inline]
    pub fn start_recording(&mut self) {
        self.is_recording = true;
    }

    /// Tell the writer to stop storing audio sample data.
    ///
    /// This signals the I/O thread to flush remaining samples and finalize the file.
    /// The method blocks until the I/O thread completes.
    pub fn stop_recording(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.is_recording = false;

        // Signal writer thread to stop
        self.stop_signal.store(true, Ordering::Relaxed);

        // Wait for writer thread to finish
        if let Some(handle) = self.writer_thread.take() {
            handle.join().map_err(|_| "Writer thread panicked")?;
        }

        // Check if any errors occurred during writing
        if self.error_flag.load(Ordering::Relaxed) {
            return Err("Error occurred while writing to file".into());
        }

        Ok(())
    }

    /// Check whether the writer is actively storing audio sample data.
    #[inline]
    pub fn is_recording(&self) -> bool {
        self.is_recording
    }

    /// Check whether any write errors have occurred.
    ///
    /// This can be queried periodically to detect I/O failures
    /// without blocking the audio thread.
    #[inline]
    pub fn error_occurred(&self) -> bool {
        self.error_flag.load(Ordering::Relaxed)
    }
}

impl<S: Sample + Send + 'static> Block<S> for FileOutputBlock<S> {
    fn process(&mut self, inputs: &[&[S]], _outputs: &mut [&mut [S]], _modulation_values: &[S], _context: &DspContext) {
        if !self.is_recording || inputs.is_empty() {
            return;
        }

        let producer = match &mut self.producer {
            Some(p) => p,
            None => return,
        };

        // Interleave and push to ring buffer (non-blocking)
        let active_channels = inputs.len().min(self.num_channels);
        let buffer_len = inputs[0].len();

        for sample_idx in 0..buffer_len {
            for input in inputs.iter().take(active_channels) {
                // Non-blocking push - drops sample if buffer full
                let _ = producer.try_push(input[sample_idx]);
            }
        }
    }

    #[inline]
    fn input_count(&self) -> usize {
        self.num_channels
    }

    #[inline]
    fn output_count(&self) -> usize {
        0
    }

    #[inline]
    fn modulation_outputs(&self) -> &[ModulationOutput] {
        &[]
    }
}

impl<S: Sample + Send + 'static> Drop for FileOutputBlock<S> {
    fn drop(&mut self) {
        // Signal writer thread to stop
        self.stop_signal.store(true, Ordering::Relaxed);

        // Wait for writer thread to finish (ignore errors in drop)
        if let Some(handle) = self.writer_thread.take() {
            let _ = handle.join();
        }
    }
}
