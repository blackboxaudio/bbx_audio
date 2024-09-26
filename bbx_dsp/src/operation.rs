use crate::process::Process;

/// Represents a heap-allocated container for a node and its DSP process.
pub type Operation = Box<dyn Process + Send>;

/// Type of DSP operation.
#[derive(PartialEq)]
pub enum OperationType {
    Effector,
    Generator,
}
