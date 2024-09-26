use crate::process::Process;

pub type Operation = Box<dyn Process<Sample = f32> + Send>;

#[derive(PartialEq)]
pub enum OperationType {
    Effector,
    Generator,
}
