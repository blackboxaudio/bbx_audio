pub trait Process<T> {
    fn process(&mut self, sample: Option<T>) -> T;
}
