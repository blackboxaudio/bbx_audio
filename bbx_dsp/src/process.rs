pub trait Process<T> {
    fn process(&mut self) -> T;
}
