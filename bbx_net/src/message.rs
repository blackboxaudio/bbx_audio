//! Network message types for audio control.
//!
//! Provides `NetMessage` for parameter changes and triggers, and `NetEvent`
//! for sample-accurate timing within audio buffers.

use crate::{address::NodeId, clock::SyncedTimestamp};

/// Network message types for audio control.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(C)]
pub enum NetMessageType {
    /// Parameter value change (float value).
    ParameterChange = 0,
    /// Trigger event (momentary, no value).
    Trigger = 1,
    /// Request current state (server responds with current values).
    StateRequest = 2,
    /// State response from server.
    StateResponse = 3,
    /// Ping for latency measurement.
    Ping = 4,
    /// Pong response with server timestamp.
    Pong = 5,
}

/// Payload data for network messages.
///
/// Type-safe discriminated union that can carry different data types
/// while maintaining `Copy` semantics for lock-free buffers.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub enum NetPayload {
    /// No payload data.
    #[default]
    None,
    /// Single float value (for parameter changes).
    Value(f32),
    /// 2D coordinates (for spatial triggers).
    Coordinates { x: f32, y: f32 },
}

impl NetPayload {
    /// Extract coordinates if this payload contains them.
    pub fn coordinates(&self) -> Option<(f32, f32)> {
        match self {
            NetPayload::Coordinates { x, y } => Some((*x, *y)),
            _ => None,
        }
    }

    /// Extract single value if this payload contains one.
    pub fn value(&self) -> Option<f32> {
        match self {
            NetPayload::Value(v) => Some(*v),
            _ => None,
        }
    }
}

/// A network message for audio control.
///
/// Uses `#[repr(C)]` for FFI compatibility.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct NetMessage {
    /// The type of message.
    pub message_type: NetMessageType,
    /// Target parameter name hash (FNV-1a for fast lookup).
    pub param_hash: u32,
    /// Message payload data.
    pub payload: NetPayload,
    /// Source node identifier.
    pub node_id: NodeId,
    /// Scheduled timestamp (synced clock).
    pub timestamp: SyncedTimestamp,
}

impl NetMessage {
    /// Create a new parameter change message.
    pub fn param_change(param_name: &str, value: f32, node_id: NodeId) -> Self {
        Self {
            message_type: NetMessageType::ParameterChange,
            param_hash: hash_param_name(param_name),
            payload: NetPayload::Value(value),
            node_id,
            timestamp: SyncedTimestamp::default(),
        }
    }

    /// Create a new trigger message.
    pub fn trigger(trigger_name: &str, node_id: NodeId) -> Self {
        Self {
            message_type: NetMessageType::Trigger,
            param_hash: hash_param_name(trigger_name),
            payload: NetPayload::None,
            node_id,
            timestamp: SyncedTimestamp::default(),
        }
    }

    /// Create a trigger with 2D coordinates.
    pub fn trigger_with_coordinates(trigger_name: &str, x: f32, y: f32, node_id: NodeId) -> Self {
        Self {
            message_type: NetMessageType::Trigger,
            param_hash: hash_param_name(trigger_name),
            payload: NetPayload::Coordinates { x, y },
            node_id,
            timestamp: SyncedTimestamp::default(),
        }
    }

    /// Create a ping message for latency measurement.
    pub fn ping(node_id: NodeId, timestamp: SyncedTimestamp) -> Self {
        Self {
            message_type: NetMessageType::Ping,
            param_hash: 0,
            payload: NetPayload::None,
            node_id,
            timestamp,
        }
    }

    /// Create a pong response message.
    pub fn pong(node_id: NodeId, timestamp: SyncedTimestamp) -> Self {
        Self {
            message_type: NetMessageType::Pong,
            param_hash: 0,
            payload: NetPayload::None,
            node_id,
            timestamp,
        }
    }

    /// Set the timestamp for scheduled delivery.
    pub fn with_timestamp(mut self, timestamp: SyncedTimestamp) -> Self {
        self.timestamp = timestamp;
        self
    }
}

/// A network event with sample-accurate timing for audio buffer processing.
///
/// Combines a network message with a sample offset indicating when the event
/// should be processed within the current audio buffer.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct NetEvent {
    /// The network message data.
    pub message: NetMessage,
    /// Sample offset within the current buffer (0 to buffer_size - 1).
    pub sample_offset: u32,
}

impl NetEvent {
    /// Create a new network event with sample offset.
    pub fn new(message: NetMessage, sample_offset: u32) -> Self {
        Self { message, sample_offset }
    }

    /// Create an event to be processed at the start of the buffer.
    pub fn immediate(message: NetMessage) -> Self {
        Self {
            message,
            sample_offset: 0,
        }
    }
}

/// FNV-1a hash for fast parameter name lookup.
///
/// This hash function is fast and produces good distribution for short strings.
/// Used to avoid string comparisons in the audio thread hot path.
#[inline]
pub fn hash_param_name(name: &str) -> u32 {
    const FNV_OFFSET: u32 = 2166136261;
    const FNV_PRIME: u32 = 16777619;

    let mut hash = FNV_OFFSET;
    for byte in name.bytes() {
        hash ^= byte as u32;
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    hash
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_param_name_deterministic() {
        let hash1 = hash_param_name("gain");
        let hash2 = hash_param_name("gain");
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_hash_param_name_different_strings() {
        let hash1 = hash_param_name("gain");
        let hash2 = hash_param_name("volume");
        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_hash_param_name_similar_strings() {
        let hash1 = hash_param_name("param1");
        let hash2 = hash_param_name("param2");
        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_net_message_param_change() {
        let node_id = NodeId::from_parts(1, 2);
        let msg = NetMessage::param_change("gain", 0.5, node_id);

        assert_eq!(msg.message_type, NetMessageType::ParameterChange);
        assert_eq!(msg.param_hash, hash_param_name("gain"));
        assert!((msg.payload.value().unwrap() - 0.5).abs() < f32::EPSILON);
        assert_eq!(msg.node_id, node_id);
    }

    #[test]
    fn test_net_message_trigger() {
        let node_id = NodeId::from_parts(3, 4);
        let msg = NetMessage::trigger("start", node_id);

        assert_eq!(msg.message_type, NetMessageType::Trigger);
        assert_eq!(msg.param_hash, hash_param_name("start"));
        assert!(matches!(msg.payload, NetPayload::None));
    }

    #[test]
    fn test_net_message_trigger_with_coordinates() {
        let node_id = NodeId::from_parts(5, 6);
        let msg = NetMessage::trigger_with_coordinates("droplet", 0.25, 0.75, node_id);

        assert_eq!(msg.message_type, NetMessageType::Trigger);
        assert_eq!(msg.param_hash, hash_param_name("droplet"));
        let (x, y) = msg.payload.coordinates().unwrap();
        assert!((x - 0.25).abs() < f32::EPSILON);
        assert!((y - 0.75).abs() < f32::EPSILON);
    }

    #[test]
    fn test_net_event_immediate() {
        let node_id = NodeId::from_parts(5, 6);
        let msg = NetMessage::param_change("freq", 440.0, node_id);
        let event = NetEvent::immediate(msg);

        assert_eq!(event.sample_offset, 0);
    }

    #[test]
    fn test_net_event_with_offset() {
        let node_id = NodeId::from_parts(7, 8);
        let msg = NetMessage::param_change("pan", 0.0, node_id);
        let event = NetEvent::new(msg, 256);

        assert_eq!(event.sample_offset, 256);
    }
}
