//! # BBX Net
//!
//! Network audio control for art installations and live performance: OSC and WebSocket protocols.
//!
//! This crate provides lock-free, realtime-safe network message passing for
//! distributed audio systems. It supports:
//!
//! - **OSC (UDP)**: For TouchOSC, Max/MSP, and other creative tools
//! - **WebSocket**: For phone PWA interfaces
//!
//! ## Features
//!
//! - `osc` - Enable OSC protocol support (requires `rosc` crate)
//! - `websocket` - Enable WebSocket support (requires tokio runtime)
//! - `full` - Enable all protocols
//!
//! ## Example
//!
//! ```rust,ignore
//! use bbx_net::{net_buffer, NetMessage, NodeId};
//! use bbx_core::random::XorShiftRng;
//!
//! // Create a buffer for network -> audio thread communication
//! let (mut producer, mut consumer) = net_buffer(256);
//!
//! // Generate a node ID
//! let mut rng = XorShiftRng::new(12345);
//! let node_id = NodeId::generate(&mut rng);
//!
//! // In network thread: send parameter changes
//! let msg = NetMessage::param_change("gain", 0.5, node_id);
//! producer.try_send(msg);
//!
//! // In audio thread: receive messages (realtime-safe)
//! let events = consumer.drain_into_stack();
//! for event in events {
//!     // Apply parameter change to DSP graph
//! }
//! ```

pub mod address;
pub mod buffer;
pub mod clock;
pub mod error;
pub mod message;

#[cfg(feature = "osc")]
pub mod osc;

#[cfg(feature = "websocket")]
pub mod websocket;

pub use address::{AddressPath, NodeId};
pub use buffer::{MAX_NET_EVENTS_PER_BUFFER, NetBufferConsumer, NetBufferProducer, net_buffer};
pub use clock::{ClockSync, SyncedTimestamp};
pub use error::{NetError, Result};
pub use message::{NetEvent, NetMessage, NetMessageType, NetPayload, hash_param_name};
