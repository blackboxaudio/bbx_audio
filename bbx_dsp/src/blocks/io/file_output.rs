use crate::{
    block::Block,
    buffer::{AudioBuffer, Buffer},
    context::{DEFAULT_SAMPLE_RATE, DspContext},
    parameter::ModulationOutput,
    sample::Sample,
    writer::Writer,
};

pub struct FileOutputBlock<S: Sample> {
    writer: Box<dyn Writer<S>>,
    is_recording: bool,
    sample_buffer: Vec<AudioBuffer<S>>,
    buffer_position: usize,
}

impl<S: Sample> FileOutputBlock<S> {
    pub fn new(writer: Box<dyn Writer<S>>) -> Self {
        let num_channels = writer.num_channels();
        let sample_buffer = vec![AudioBuffer::new(DEFAULT_SAMPLE_RATE as usize); num_channels];

        Self {
            writer,
            is_recording: true,
            sample_buffer,
            buffer_position: 0,
        }
    }

    pub fn start_recording(&mut self) {
        self.is_recording = true;
        for channel in &mut self.sample_buffer {
            channel.zeroize();
        }
        self.buffer_position = 0;
    }

    pub fn stop_recording(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.is_recording = false;
        self.flush_buffers()?;
        self.writer.finalize()
    }

    pub fn is_recording(&self) -> bool {
        self.is_recording
    }

    fn flush_buffers(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        for (channel_index, channel_buffer) in self.sample_buffer.iter().enumerate() {
            if !channel_buffer.is_empty() {
                self.writer.write_channel(channel_index, channel_buffer.as_slice())?;
            }
        }

        for channel in &mut self.sample_buffer {
            channel.zeroize();
        }

        self.buffer_position = 0;

        Ok(())
    }
}

impl<S: Sample> Block<S> for FileOutputBlock<S> {
    fn process(&mut self, inputs: &[&[S]], _outputs: &mut [&mut [S]], _modulation_values: &[S], _context: &DspContext) {
        if !self.is_recording || inputs.is_empty() {
            return;
        }

        let num_channels = inputs.len().min(self.sample_buffer.len());
        for (channel_idx, channel) in inputs.iter().enumerate().take(num_channels) {
            if let Err(e) = self.writer.write_channel(channel_idx, channel) {
                eprintln!("Error writing to file: {e}");
            }
        }
    }

    fn input_count(&self) -> usize {
        self.writer.num_channels()
    }

    fn output_count(&self) -> usize {
        0
    }

    fn modulation_outputs(&self) -> &[ModulationOutput] {
        &[]
    }
}
