//! Audio file input block.

use crate::{block::Block, context::DspContext, parameter::ModulationOutput, reader::Reader, sample::Sample};

/// Reads audio from a file into the DSP graph.
///
/// Wraps a [`Reader`] implementation to stream audio samples into the graph.
/// Supports optional looping for continuous playback.
pub struct FileInputBlock<S: Sample> {
    reader: Box<dyn Reader<S>>,
    current_position: usize,
    loop_enabled: bool,
}

impl<S: Sample> FileInputBlock<S> {
    /// Create a `FileInputBlock` with the `Reader` implementation for a particular type of audio file.
    pub fn new(reader: Box<dyn Reader<S>>) -> Self {
        Self {
            reader,
            current_position: 0,
            loop_enabled: false,
        }
    }

    /// Set whether the audio will be looped or not.
    #[inline]
    pub fn set_loop_enabled(&mut self, enabled: bool) {
        self.loop_enabled = enabled;
    }

    /// Set the position at which the audio file's samples are being read.
    #[inline]
    pub fn set_position(&mut self, position: usize) {
        self.current_position = position;
    }

    /// Get the position at which the audio file's samples are being read.
    #[inline]
    pub fn get_position(&self) -> usize {
        self.current_position
    }

    /// Check whether the reader has finished reading every sample in the audio file.
    #[inline]
    pub fn is_finished(&self) -> bool {
        self.current_position >= self.reader.num_samples()
    }

    /// Get the underlying `Reader` implementation of the `FileInputBlock`.
    #[inline]
    pub fn get_reader(&self) -> &dyn Reader<S> {
        self.reader.as_ref()
    }

    fn advance_position(&mut self, samples: usize) {
        self.current_position += samples;

        if self.loop_enabled {
            let file_length = self.reader.num_samples();
            if file_length > 0 && self.current_position >= file_length {
                self.current_position %= file_length;
            }
        }
    }
}

impl<S: Sample> Block<S> for FileInputBlock<S> {
    fn process(&mut self, _inputs: &[&[S]], outputs: &mut [&mut [S]], _modulation_values: &[S], context: &DspContext) {
        let buffer_size = context.buffer_size;
        let num_file_channels = self.reader.num_channels();
        let file_length = self.reader.num_samples();

        for (channel_index, output_buffer) in outputs.iter_mut().enumerate() {
            if channel_index >= num_file_channels {
                output_buffer.fill(S::ZERO);
                continue;
            }

            let input_channel = self.reader.read_channel(channel_index);

            for (sample_index, output_sample) in output_buffer.iter_mut().enumerate() {
                let read_position = self.current_position + sample_index;
                if read_position < file_length {
                    *output_sample = input_channel[read_position];
                } else if self.loop_enabled && file_length > 0 {
                    *output_sample = input_channel[read_position % file_length];
                } else {
                    *output_sample = S::ZERO;
                }
            }
        }

        self.advance_position(buffer_size);
    }

    #[inline]
    fn input_count(&self) -> usize {
        0
    }

    #[inline]
    fn output_count(&self) -> usize {
        self.reader.num_channels()
    }

    #[inline]
    fn modulation_outputs(&self) -> &[ModulationOutput] {
        &[]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::channel::ChannelLayout;

    struct MockReader<S: Sample> {
        sample_rate: f64,
        channels: Vec<Vec<S>>,
    }

    impl<S: Sample> MockReader<S> {
        fn new(sample_rate: f64, channels: Vec<Vec<S>>) -> Self {
            Self { sample_rate, channels }
        }
    }

    impl<S: Sample> Reader<S> for MockReader<S> {
        fn sample_rate(&self) -> f64 {
            self.sample_rate
        }

        fn num_channels(&self) -> usize {
            self.channels.len()
        }

        fn num_samples(&self) -> usize {
            self.channels.first().map(|c| c.len()).unwrap_or(0)
        }

        fn read_channel(&self, channel_index: usize) -> &[S] {
            &self.channels[channel_index]
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
    fn test_file_input_block_counts() {
        let reader = MockReader::new(44100.0, vec![vec![0.0f32; 100], vec![0.0f32; 100]]);
        let block = FileInputBlock::new(Box::new(reader));
        assert_eq!(block.input_count(), 0);
        assert_eq!(block.output_count(), 2);
    }

    #[test]
    fn test_file_input_block_reads_samples() {
        let samples: Vec<f32> = (0..100).map(|i| i as f32 / 100.0).collect();
        let reader = MockReader::new(44100.0, vec![samples.clone()]);
        let mut block = FileInputBlock::new(Box::new(reader));

        let context = test_context(10);
        let mut output = vec![0.0f32; 10];
        let mut outputs: [&mut [f32]; 1] = [&mut output];

        block.process(&[], &mut outputs, &[], &context);

        for (i, &sample) in output.iter().enumerate() {
            assert!((sample - (i as f32 / 100.0)).abs() < 1e-6);
        }
    }

    #[test]
    fn test_file_input_block_position_advances() {
        let reader = MockReader::new(44100.0, vec![vec![0.0f32; 100]]);
        let mut block = FileInputBlock::new(Box::new(reader));

        assert_eq!(block.get_position(), 0);

        let context = test_context(10);
        let mut output = vec![0.0f32; 10];
        let mut outputs: [&mut [f32]; 1] = [&mut output];

        block.process(&[], &mut outputs, &[], &context);
        assert_eq!(block.get_position(), 10);

        block.process(&[], &mut outputs, &[], &context);
        assert_eq!(block.get_position(), 20);
    }

    #[test]
    fn test_file_input_block_set_position() {
        let reader = MockReader::new(44100.0, vec![vec![0.0f32; 100]]);
        let mut block = FileInputBlock::new(Box::new(reader));

        block.set_position(50);
        assert_eq!(block.get_position(), 50);
    }

    #[test]
    fn test_file_input_block_is_finished() {
        let reader = MockReader::new(44100.0, vec![vec![0.0f32; 20]]);
        let mut block = FileInputBlock::new(Box::new(reader));

        assert!(!block.is_finished());

        let context = test_context(10);
        let mut output = vec![0.0f32; 10];
        let mut outputs: [&mut [f32]; 1] = [&mut output];

        block.process(&[], &mut outputs, &[], &context);
        assert!(!block.is_finished());

        block.process(&[], &mut outputs, &[], &context);
        assert!(block.is_finished());
    }

    #[test]
    fn test_file_input_block_outputs_zero_past_end() {
        let samples: Vec<f32> = vec![1.0; 5];
        let reader = MockReader::new(44100.0, vec![samples]);
        let mut block = FileInputBlock::new(Box::new(reader));

        let context = test_context(10);
        let mut output = vec![0.0f32; 10];
        let mut outputs: [&mut [f32]; 1] = [&mut output];

        block.process(&[], &mut outputs, &[], &context);

        for i in 0..5 {
            assert!((output[i] - 1.0).abs() < 1e-6);
        }
        for i in 5..10 {
            assert!((output[i]).abs() < 1e-6);
        }
    }

    #[test]
    fn test_file_input_block_looping() {
        let samples: Vec<f32> = vec![0.0, 1.0, 2.0, 3.0];
        let reader = MockReader::new(44100.0, vec![samples]);
        let mut block = FileInputBlock::new(Box::new(reader));
        block.set_loop_enabled(true);

        let context = test_context(8);
        let mut output = vec![0.0f32; 8];
        let mut outputs: [&mut [f32]; 1] = [&mut output];

        block.process(&[], &mut outputs, &[], &context);

        assert!((output[0] - 0.0).abs() < 1e-6);
        assert!((output[1] - 1.0).abs() < 1e-6);
        assert!((output[2] - 2.0).abs() < 1e-6);
        assert!((output[3] - 3.0).abs() < 1e-6);
        assert!((output[4] - 0.0).abs() < 1e-6);
        assert!((output[5] - 1.0).abs() < 1e-6);
        assert!((output[6] - 2.0).abs() < 1e-6);
        assert!((output[7] - 3.0).abs() < 1e-6);
    }

    #[test]
    fn test_file_input_block_stereo() {
        let left: Vec<f32> = vec![1.0; 10];
        let right: Vec<f32> = vec![0.5; 10];
        let reader = MockReader::new(44100.0, vec![left, right]);
        let mut block = FileInputBlock::new(Box::new(reader));

        let context = test_context(5);
        let mut output_l = vec![0.0f32; 5];
        let mut output_r = vec![0.0f32; 5];
        let mut outputs: [&mut [f32]; 2] = [&mut output_l, &mut output_r];

        block.process(&[], &mut outputs, &[], &context);

        for &sample in &output_l {
            assert!((sample - 1.0).abs() < 1e-6);
        }
        for &sample in &output_r {
            assert!((sample - 0.5).abs() < 1e-6);
        }
    }

    #[test]
    fn test_file_input_block_more_outputs_than_channels() {
        let samples: Vec<f32> = vec![1.0; 10];
        let reader = MockReader::new(44100.0, vec![samples]);
        let mut block = FileInputBlock::new(Box::new(reader));

        let context = test_context(5);
        let mut output_0 = vec![0.0f32; 5];
        let mut output_1 = vec![0.5f32; 5];
        let mut outputs: [&mut [f32]; 2] = [&mut output_0, &mut output_1];

        block.process(&[], &mut outputs, &[], &context);

        for &sample in &output_0 {
            assert!((sample - 1.0).abs() < 1e-6);
        }
        for &sample in &output_1 {
            assert!(sample.abs() < 1e-6);
        }
    }
}
