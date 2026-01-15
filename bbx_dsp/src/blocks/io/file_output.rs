//! Audio file output block with non-blocking I/O.

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

/// Writes audio from the DSP graph to a file.
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
        let mut channel_buffers: Vec<Vec<S>> = vec![Vec::new(); num_channels];
        let mut current_channel = 0;

        // Flush threshold (samples per channel before writing to disk)
        const FLUSH_THRESHOLD: usize = 4096;

        loop {
            while let Some(sample) = consumer.try_pop() {
                channel_buffers[current_channel].push(sample);
                current_channel = (current_channel + 1) % num_channels;

                if channel_buffers[0].len() >= FLUSH_THRESHOLD {
                    for (ch, buffer) in channel_buffers.iter_mut().enumerate() {
                        if writer.write_channel(ch, buffer).is_err() {
                            error_flag.store(true, Ordering::Relaxed);
                        }
                        buffer.clear();
                    }
                }
            }

            if stop_signal.load(Ordering::Acquire) {
                for (ch, buffer) in channel_buffers.iter().enumerate() {
                    if !buffer.is_empty() && writer.write_channel(ch, buffer).is_err() {
                        error_flag.store(true, Ordering::Relaxed);
                    }
                }

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

        self.stop_signal.store(true, Ordering::Release);

        if let Some(handle) = self.writer_thread.take() {
            handle.join().map_err(|_| "Writer thread panicked")?;
        }

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

        let buffer_len = inputs[0].len();

        for sample_idx in 0..buffer_len {
            for ch in 0..self.num_channels {
                let sample = inputs.get(ch).map_or(S::ZERO, |input| input[sample_idx]);
                let _ = producer.try_push(sample);
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
        self.stop_signal.store(true, Ordering::Release);

        // Ignore errors in drop
        if let Some(handle) = self.writer_thread.take() {
            let _ = handle.join();
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use super::*;
    use crate::channel::ChannelLayout;

    struct MockWriter<S: Sample> {
        sample_rate: f64,
        num_channels: usize,
        channels: Arc<Mutex<Vec<Vec<S>>>>,
        finalized: Arc<AtomicBool>,
    }

    impl<S: Sample> MockWriter<S> {
        fn new(sample_rate: f64, num_channels: usize) -> Self {
            let channels: Vec<Vec<S>> = (0..num_channels).map(|_| Vec::new()).collect();
            Self {
                sample_rate,
                num_channels,
                channels: Arc::new(Mutex::new(channels)),
                finalized: Arc::new(AtomicBool::new(false)),
            }
        }

        fn get_channels(&self) -> Arc<Mutex<Vec<Vec<S>>>> {
            self.channels.clone()
        }

        fn get_finalized(&self) -> Arc<AtomicBool> {
            self.finalized.clone()
        }
    }

    impl<S: Sample> Writer<S> for MockWriter<S> {
        fn sample_rate(&self) -> f64 {
            self.sample_rate
        }

        fn num_channels(&self) -> usize {
            self.num_channels
        }

        fn can_write(&self) -> bool {
            true
        }

        fn write_channel(&mut self, channel_index: usize, samples: &[S]) -> Result<(), Box<dyn std::error::Error>> {
            let mut channels = self.channels.lock().unwrap();
            if channel_index < channels.len() {
                channels[channel_index].extend_from_slice(samples);
            }
            Ok(())
        }

        fn finalize(&mut self) -> Result<(), Box<dyn std::error::Error>> {
            self.finalized.store(true, Ordering::Relaxed);
            Ok(())
        }
    }

    fn test_context(buffer_size: usize) -> DspContext {
        DspContext {
            sample_rate: 44100.0,
            buffer_size,
            num_channels: 2,
            current_sample: 0,
            channel_layout: ChannelLayout::Stereo,
        }
    }

    #[test]
    fn test_file_output_block_counts() {
        let writer = MockWriter::<f32>::new(44100.0, 2);
        let block = FileOutputBlock::new(Box::new(writer));
        assert_eq!(block.input_count(), 2);
        assert_eq!(block.output_count(), 0);
    }

    #[test]
    fn test_file_output_block_recording_state() {
        let writer = MockWriter::<f32>::new(44100.0, 2);
        let mut block = FileOutputBlock::new(Box::new(writer));

        assert!(block.is_recording());

        block.start_recording();
        assert!(block.is_recording());
    }

    #[test]
    fn test_file_output_block_writes_and_finalizes() {
        let writer = MockWriter::<f32>::new(44100.0, 1);
        let channels = writer.get_channels();
        let finalized = writer.get_finalized();

        let mut block = FileOutputBlock::new(Box::new(writer));

        let context = test_context(10);
        let input: Vec<f32> = vec![0.5; 10];
        let inputs: [&[f32]; 1] = [&input];
        let mut outputs: [&mut [f32]; 0] = [];

        block.process(&inputs, &mut outputs, &[], &context);

        block.stop_recording().unwrap();

        assert!(finalized.load(Ordering::Relaxed));
        let written = channels.lock().unwrap();
        assert!(!written[0].is_empty());
    }

    #[test]
    fn test_file_output_block_no_error_initially() {
        let writer = MockWriter::<f32>::new(44100.0, 2);
        let block = FileOutputBlock::new(Box::new(writer));
        assert!(!block.error_occurred());
    }

    #[test]
    fn test_file_output_block_empty_inputs() {
        let writer = MockWriter::<f32>::new(44100.0, 1);
        let mut block = FileOutputBlock::new(Box::new(writer));

        let context = test_context(10);
        let inputs: [&[f32]; 0] = [];
        let mut outputs: [&mut [f32]; 0] = [];

        block.process(&inputs, &mut outputs, &[], &context);

        block.stop_recording().unwrap();
        assert!(!block.error_occurred());
    }

    #[test]
    fn test_file_output_block_mono_input_to_stereo_file() {
        let writer = MockWriter::<f32>::new(44100.0, 2);
        let channels = writer.get_channels();

        let mut block = FileOutputBlock::new(Box::new(writer));

        let context = test_context(4);
        let mono_input: Vec<f32> = vec![0.1, 0.2, 0.3, 0.4];
        let inputs: [&[f32]; 1] = [&mono_input];
        let mut outputs: [&mut [f32]; 0] = [];

        block.process(&inputs, &mut outputs, &[], &context);
        block.stop_recording().unwrap();

        let written = channels.lock().unwrap();
        assert_eq!(written[0], vec![0.1, 0.2, 0.3, 0.4]);
        assert_eq!(written[1], vec![0.0, 0.0, 0.0, 0.0]);
    }

    #[test]
    fn test_file_output_block_interleaving_order() {
        let writer = MockWriter::<f32>::new(44100.0, 2);
        let channels = writer.get_channels();

        let mut block = FileOutputBlock::new(Box::new(writer));

        let context = test_context(3);
        let left: Vec<f32> = vec![1.0, 2.0, 3.0];
        let right: Vec<f32> = vec![0.1, 0.2, 0.3];
        let inputs: [&[f32]; 2] = [&left, &right];
        let mut outputs: [&mut [f32]; 0] = [];

        block.process(&inputs, &mut outputs, &[], &context);
        block.stop_recording().unwrap();

        let written = channels.lock().unwrap();
        assert_eq!(written[0], vec![1.0, 2.0, 3.0]);
        assert_eq!(written[1], vec![0.1, 0.2, 0.3]);
    }

    #[test]
    fn test_file_output_block_excess_inputs_ignored() {
        let writer = MockWriter::<f32>::new(44100.0, 2);
        let channels = writer.get_channels();

        let mut block = FileOutputBlock::new(Box::new(writer));

        let context = test_context(2);
        let ch0: Vec<f32> = vec![1.0, 2.0];
        let ch1: Vec<f32> = vec![0.1, 0.2];
        let ch2: Vec<f32> = vec![9.9, 9.9];
        let inputs: [&[f32]; 3] = [&ch0, &ch1, &ch2];
        let mut outputs: [&mut [f32]; 0] = [];

        block.process(&inputs, &mut outputs, &[], &context);
        block.stop_recording().unwrap();

        let written = channels.lock().unwrap();
        assert_eq!(written[0], vec![1.0, 2.0]);
        assert_eq!(written[1], vec![0.1, 0.2]);
        assert_eq!(written.len(), 2);
    }
}
