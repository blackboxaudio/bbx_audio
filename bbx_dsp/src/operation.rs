use crate::process::Process;

pub type Operation = Box<dyn Process + Send>;

#[derive(PartialEq)]
pub enum OperationType {
    Effector,
    Generator,
}
