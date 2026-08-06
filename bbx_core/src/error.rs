//! Error types for the bbx_audio workspace.
//!
//! This module provides a C-compatible error enum and a Result type alias
//! for use across all crates in the workspace.

use core::fmt;

/// Error codes for bbx_audio operations.
///
/// Uses `#[repr(C)]` for C-compatible memory layout, enabling FFI usage.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BbxError {
    /// No error occurred.
    Ok = 0,
    /// A null pointer was passed where a valid pointer was expected.
    NullPointer = 1,
    /// An invalid parameter value was provided.
    InvalidParameter = 2,
    /// An invalid buffer size was specified.
    InvalidBufferSize = 3,
    /// The graph has not been prepared for playback.
    GraphNotPrepared = 4,
    /// Memory allocation failed.
    AllocationFailed = 5,
}

impl fmt::Display for BbxError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BbxError::Ok => write!(f, "no error"),
            BbxError::NullPointer => write!(f, "null pointer"),
            BbxError::InvalidParameter => write!(f, "invalid parameter"),
            BbxError::InvalidBufferSize => write!(f, "invalid buffer size"),
            BbxError::GraphNotPrepared => write!(f, "graph not prepared"),
            BbxError::AllocationFailed => write!(f, "allocation failed"),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for BbxError {}

/// Result type alias for bbx_audio operations.
pub type Result<T> = core::result::Result<T, BbxError>;
