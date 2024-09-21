pub trait Reader {
    type Metadata;

    fn open(filename: &str) -> Self;
    fn read_file(&mut self) -> (Self::Metadata, Vec<f32>);
}
