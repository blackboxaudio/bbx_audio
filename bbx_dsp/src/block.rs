use crate::sample::Sample;

pub trait Block<T> {
    fn process(&mut self, sample: Option<Sample<T>>) -> Sample<T>;
}