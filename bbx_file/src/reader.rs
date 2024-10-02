pub trait Reader {
    fn read_channel(&mut self, channel_idx: usize, sample_idx: usize, buffer_len: usize) -> &[f32];
}
