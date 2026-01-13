//! OSC (Open Sound Control) protocol support.
//!
//! This module provides OSC message parsing and a UDP server for receiving
//! control messages from TouchOSC, Max/MSP, and other OSC-compatible tools.
//!
//! Enable with the `osc` feature flag.

mod parser;
mod server;

pub use parser::parse_osc_message;
pub use server::{OscServer, OscServerConfig};
