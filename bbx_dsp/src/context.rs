pub const DEFAULT_BUFFER_SIZE: usize = 512;

pub const DEFAULT_SAMPLE_RATE: f64 = 44100.0;

#[derive(Clone)]
pub struct DspContext {
    pub sample_rate: f64,
    pub num_channels: usize,
    pub buffer_size: usize,
    pub current_sample: u64,
}
