use crate::{effector::Effector, generator::Generator};

pub enum Block {
    Effector(Effector),
    Generator(Generator),
}
