//! WebSocket JSON message protocol.

use serde::{Deserialize, Serialize};

/// Incoming message from client (phone).
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type")]
pub enum ClientMessage {
    /// Join a room with room code.
    #[serde(rename = "join")]
    Join {
        room_code: String,
        client_name: Option<String>,
    },

    /// Parameter value change.
    #[serde(rename = "param")]
    Parameter {
        param: String,
        value: f32,
        /// Optional scheduled time (microseconds since session start).
        #[serde(default)]
        at: Option<u64>,
    },

    /// Trigger event (momentary).
    #[serde(rename = "trigger")]
    Trigger {
        name: String,
        #[serde(default)]
        at: Option<u64>,
    },

    /// Request current state.
    #[serde(rename = "sync")]
    Sync,

    /// Ping for latency measurement.
    #[serde(rename = "ping")]
    Ping { client_time: u64 },

    /// Leave current room.
    #[serde(rename = "leave")]
    Leave,
}

/// Outgoing message to client.
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type")]
pub enum ServerMessage {
    /// Welcome message after joining.
    #[serde(rename = "welcome")]
    Welcome { node_id: String, server_time: u64 },

    /// Current parameter state.
    #[serde(rename = "state")]
    State { params: Vec<ParamState> },

    /// Parameter update notification (bidirectional sync).
    #[serde(rename = "update")]
    Update { param: String, value: f32 },

    /// Pong response.
    #[serde(rename = "pong")]
    Pong { client_time: u64, server_time: u64 },

    /// Error notification.
    #[serde(rename = "error")]
    Error { code: String, message: String },

    /// Room closed notification.
    #[serde(rename = "closed")]
    RoomClosed,
}

/// Parameter state for sync responses.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParamState {
    pub name: String,
    pub value: f32,
    #[serde(default)]
    pub min: f32,
    #[serde(default = "default_max")]
    pub max: f32,
}

fn default_max() -> f32 {
    1.0
}

impl ParamState {
    pub fn new(name: impl Into<String>, value: f32) -> Self {
        Self {
            name: name.into(),
            value,
            min: 0.0,
            max: 1.0,
        }
    }

    pub fn with_range(mut self, min: f32, max: f32) -> Self {
        self.min = min;
        self.max = max;
        self
    }
}

impl ServerMessage {
    /// Create an error message.
    pub fn error(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self::Error {
            code: code.into(),
            message: message.into(),
        }
    }

    /// Create an invalid room code error.
    pub fn invalid_room_code() -> Self {
        Self::error("INVALID_ROOM", "Invalid room code")
    }

    /// Create a room full error.
    pub fn room_full() -> Self {
        Self::error("ROOM_FULL", "Room is at capacity")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_message_join() {
        let json = r#"{"type": "join", "room_code": "123456"}"#;
        let msg: ClientMessage = serde_json::from_str(json).unwrap();

        match msg {
            ClientMessage::Join { room_code, .. } => {
                assert_eq!(room_code, "123456");
            }
            _ => panic!("Expected Join message"),
        }
    }

    #[test]
    fn test_client_message_param() {
        let json = r#"{"type": "param", "param": "gain", "value": 0.5}"#;
        let msg: ClientMessage = serde_json::from_str(json).unwrap();

        match msg {
            ClientMessage::Parameter { param, value, at } => {
                assert_eq!(param, "gain");
                assert!((value - 0.5).abs() < f32::EPSILON);
                assert!(at.is_none());
            }
            _ => panic!("Expected Parameter message"),
        }
    }

    #[test]
    fn test_client_message_param_with_time() {
        let json = r#"{"type": "param", "param": "freq", "value": 440.0, "at": 1000000}"#;
        let msg: ClientMessage = serde_json::from_str(json).unwrap();

        match msg {
            ClientMessage::Parameter { at, .. } => {
                assert_eq!(at, Some(1000000));
            }
            _ => panic!("Expected Parameter message"),
        }
    }

    #[test]
    fn test_client_message_trigger() {
        let json = r#"{"type": "trigger", "name": "note_on"}"#;
        let msg: ClientMessage = serde_json::from_str(json).unwrap();

        match msg {
            ClientMessage::Trigger { name, .. } => {
                assert_eq!(name, "note_on");
            }
            _ => panic!("Expected Trigger message"),
        }
    }

    #[test]
    fn test_client_message_ping() {
        let json = r#"{"type": "ping", "client_time": 12345}"#;
        let msg: ClientMessage = serde_json::from_str(json).unwrap();

        match msg {
            ClientMessage::Ping { client_time } => {
                assert_eq!(client_time, 12345);
            }
            _ => panic!("Expected Ping message"),
        }
    }

    #[test]
    fn test_server_message_welcome() {
        let msg = ServerMessage::Welcome {
            node_id: "abc-123".to_string(),
            server_time: 1000,
        };
        let json = serde_json::to_string(&msg).unwrap();

        assert!(json.contains("\"type\":\"welcome\""));
        assert!(json.contains("\"node_id\":\"abc-123\""));
    }

    #[test]
    fn test_server_message_state() {
        let msg = ServerMessage::State {
            params: vec![
                ParamState::new("gain", 0.5),
                ParamState::new("freq", 440.0).with_range(20.0, 20000.0),
            ],
        };
        let json = serde_json::to_string(&msg).unwrap();

        assert!(json.contains("\"type\":\"state\""));
        assert!(json.contains("\"name\":\"gain\""));
    }

    #[test]
    fn test_server_message_error() {
        let msg = ServerMessage::error("TEST_ERROR", "Test error message");
        let json = serde_json::to_string(&msg).unwrap();

        assert!(json.contains("\"type\":\"error\""));
        assert!(json.contains("\"code\":\"TEST_ERROR\""));
    }

    #[test]
    fn test_param_state_defaults() {
        let state = ParamState::new("test", 0.5);
        assert!((state.min - 0.0).abs() < f32::EPSILON);
        assert!((state.max - 1.0).abs() < f32::EPSILON);
    }
}
