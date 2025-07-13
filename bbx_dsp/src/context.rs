#[derive(Clone)]
pub struct DspContext {
    pub sample_rate: f64,
    pub buffer_size: usize,
    pub channels: usize,
    pub current_sample: u64,
}
