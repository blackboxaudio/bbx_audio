pub trait Reader {
    fn read_channel(&mut self, channel_idx: usize, sample_idx: usize, buffer_len: usize) -> &[f32];
    fn read_sample(&self, channel_idx: usize, sample_idx: usize) -> f32;
}
