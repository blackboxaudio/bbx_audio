use bbx_buffer::buffer::{AudioBuffer, Buffer};
use bbx_file::{reader::Reader, readers::wav::WavFileReader};

use crate::{
    context::Context,
    process::{AudioInput, Process},
    utils::clear_output,
};

pub struct FileReaderGenerator {
    reader: WavFileReader,
    sample_idx: usize,
}

impl FileReaderGenerator {
    pub fn new(_context: Context, file_path: String) -> Self {
        let reader = WavFileReader::new(file_path);
        FileReaderGenerator { reader, sample_idx: 0 }
    }
}

impl Process for FileReaderGenerator {
    fn process(&mut self, _inputs: &[AudioInput], output: &mut [AudioBuffer<f32>]) {
        clear_output(output);
        for (channel_idx, channel_buffer) in output.iter_mut().enumerate() {
            channel_buffer.copy_from_slice(
                self.reader
                    .read_channel(channel_idx, self.sample_idx, channel_buffer.len()),
            )
        }
        self.sample_idx += output[0].len();
    }
}
