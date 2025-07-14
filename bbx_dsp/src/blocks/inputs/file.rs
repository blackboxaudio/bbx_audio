use crate::block::Block;
use crate::context::DspContext;
use crate::parameter::ModulationOutput;
use crate::reader::Reader;
use crate::sample::Sample;

pub struct FileInputBlock<S: Sample> {
    reader: Box<dyn Reader<S>>,
    current_position: usize,
    loop_enabled: bool,
}

impl<S: Sample> FileInputBlock<S> {
    pub fn new(reader: Box<dyn Reader<S>>) -> Self {
        Self {
            reader,
            current_position: 0,
            loop_enabled: false,
        }
    }

    pub fn set_loop_enabled(&mut self, enabled: bool) {
        self.loop_enabled = enabled;
    }

    pub fn set_position(&mut self, position: usize) {
        self.current_position = position;
    }

    pub fn get_position(&self) -> usize {
        self.current_position
    }

    pub fn is_finished(&self) -> bool {
        self.current_position >= self.reader.num_samples()
    }

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

    fn input_count(&self) -> usize {
        0
    }

    fn output_count(&self) -> usize {
        self.reader.num_channels()
    }

    fn modulation_outputs(&self) -> &[ModulationOutput] {
        &[]
    }
}
