//! WebSocket protocol support for phone PWA interfaces.
//!
//! This module provides a WebSocket server for receiving control messages
//! from web browsers (phones), with support for:
//!
//! - Room-based authentication via room codes
//! - Bidirectional communication (server can push state to clients)
//! - JSON message protocol
//!
//! Enable with the `websocket` feature flag.

mod connection;
mod protocol;
mod room;
mod server;

pub use connection::ConnectionState;
pub use protocol::{ClientMessage, ParamState, ServerMessage};
pub use room::{Room, RoomConfig, RoomManager};
pub use server::{ServerCommand, WsServer, WsServerConfig};
