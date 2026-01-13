//! Error types for bbx_net network operations.

use std::fmt;

/// Error codes for bbx_net network operations.
///
/// Uses `#[repr(C)]` for C-compatible memory layout, enabling FFI usage.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NetError {
    /// Invalid OSC or WebSocket address format.
    InvalidAddress = 0,
    /// Invalid room code provided.
    InvalidRoomCode = 1,
    /// Room has reached maximum client capacity.
    RoomFull = 2,
    /// Network connection failed.
    ConnectionFailed = 3,
    /// Failed to parse message.
    ParseError = 4,
    /// I/O error during network operation.
    IoError = 5,
    /// WebSocket protocol error.
    WebSocketError = 6,
    /// Connection or operation timeout.
    Timeout = 7,
    /// Invalid node ID format.
    InvalidNodeId = 8,
}

impl fmt::Display for NetError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NetError::InvalidAddress => write!(f, "invalid address format"),
            NetError::InvalidRoomCode => write!(f, "invalid room code"),
            NetError::RoomFull => write!(f, "room is at capacity"),
            NetError::ConnectionFailed => write!(f, "connection failed"),
            NetError::ParseError => write!(f, "message parse error"),
            NetError::IoError => write!(f, "I/O error"),
            NetError::WebSocketError => write!(f, "WebSocket error"),
            NetError::Timeout => write!(f, "connection timeout"),
            NetError::InvalidNodeId => write!(f, "invalid node ID"),
        }
    }
}

impl std::error::Error for NetError {}

impl From<std::io::Error> for NetError {
    fn from(_: std::io::Error) -> Self {
        NetError::IoError
    }
}

/// Result type alias for bbx_net operations.
pub type Result<T> = std::result::Result<T, NetError>;
