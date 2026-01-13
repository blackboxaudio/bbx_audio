//! WebSocket connection state management.

use std::time::Instant;

use crate::address::NodeId;

/// Connection state for a single WebSocket client.
#[derive(Debug, Clone)]
pub struct ConnectionState {
    /// Unique identifier for this connection.
    pub node_id: NodeId,
    /// Room code the client joined.
    pub room_code: String,
    /// Client-provided name (optional).
    pub client_name: Option<String>,
    /// When the connection was established.
    pub connected_at: Instant,
    /// When the last ping/pong was received.
    pub last_ping: Instant,
    /// Measured round-trip latency in microseconds.
    pub latency_us: u64,
    /// Number of reconnections for this node.
    pub reconnect_count: u32,
    /// Clock offset in microseconds (positive = client ahead).
    pub clock_offset: i64,
}

impl ConnectionState {
    /// Create a new connection state.
    pub fn new(node_id: NodeId, room_code: String, client_name: Option<String>) -> Self {
        let now = Instant::now();
        Self {
            node_id,
            room_code,
            client_name,
            connected_at: now,
            last_ping: now,
            latency_us: 0,
            reconnect_count: 0,
            clock_offset: 0,
        }
    }

    /// Update latency from ping/pong exchange.
    ///
    /// # Arguments
    ///
    /// * `rtt_us` - Round-trip time in microseconds
    pub fn update_latency(&mut self, rtt_us: u64) {
        self.latency_us = rtt_us / 2;
        self.last_ping = Instant::now();
    }

    /// Update clock offset from ping/pong exchange.
    ///
    /// Uses NTP-style calculation to determine the difference between
    /// client and server clocks.
    pub fn update_clock_offset(
        &mut self,
        client_send: u64,
        server_receive: u64,
        server_send: u64,
        client_receive: u64,
    ) {
        let t1 = client_send as i64;
        let t2 = server_receive as i64;
        let t3 = server_send as i64;
        let t4 = client_receive as i64;

        self.clock_offset = ((t2 - t1) + (t3 - t4)) / 2;
        self.latency_us = ((t4 - t1) - (t3 - t2)) as u64 / 2;
        self.last_ping = Instant::now();
    }

    /// Check if the connection should be considered stale.
    pub fn is_stale(&self, timeout: std::time::Duration) -> bool {
        Instant::now().duration_since(self.last_ping) > timeout
    }

    /// Mark as reconnected, incrementing the counter.
    pub fn mark_reconnected(&mut self) {
        self.reconnect_count += 1;
        self.last_ping = Instant::now();
    }

    /// Convert a client timestamp to server time using the clock offset.
    pub fn client_to_server_time(&self, client_time: u64) -> u64 {
        (client_time as i64 - self.clock_offset) as u64
    }

    /// Convert a server timestamp to client time using the clock offset.
    pub fn server_to_client_time(&self, server_time: u64) -> u64 {
        (server_time as i64 + self.clock_offset) as u64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_connection_state_new() {
        let node_id = NodeId::from_parts(1, 2);
        let state = ConnectionState::new(node_id, "123456".to_string(), None);

        assert_eq!(state.node_id, node_id);
        assert_eq!(state.room_code, "123456");
        assert_eq!(state.reconnect_count, 0);
        assert_eq!(state.latency_us, 0);
    }

    #[test]
    fn test_update_latency() {
        let node_id = NodeId::from_parts(1, 2);
        let mut state = ConnectionState::new(node_id, "123456".to_string(), None);

        state.update_latency(20000);

        assert_eq!(state.latency_us, 10000);
    }

    #[test]
    fn test_update_clock_offset() {
        let node_id = NodeId::from_parts(1, 2);
        let mut state = ConnectionState::new(node_id, "123456".to_string(), None);

        state.update_clock_offset(100, 200, 200, 300);

        assert_eq!(state.clock_offset, 0);
    }

    #[test]
    fn test_clock_offset_client_ahead() {
        let node_id = NodeId::from_parts(1, 2);
        let mut state = ConnectionState::new(node_id, "123456".to_string(), None);

        state.update_clock_offset(200, 100, 100, 200);

        assert!(state.clock_offset < 0);
    }

    #[test]
    fn test_mark_reconnected() {
        let node_id = NodeId::from_parts(1, 2);
        let mut state = ConnectionState::new(node_id, "123456".to_string(), None);

        state.mark_reconnected();
        state.mark_reconnected();

        assert_eq!(state.reconnect_count, 2);
    }

    #[test]
    fn test_time_conversion() {
        let node_id = NodeId::from_parts(1, 2);
        let mut state = ConnectionState::new(node_id, "123456".to_string(), None);

        state.clock_offset = 100;

        let server_time = state.client_to_server_time(1000);
        assert_eq!(server_time, 900);

        let client_time = state.server_to_client_time(900);
        assert_eq!(client_time, 1000);
    }

    #[test]
    fn test_is_stale() {
        use std::time::Duration;

        let node_id = NodeId::from_parts(1, 2);
        let state = ConnectionState::new(node_id, "123456".to_string(), None);

        assert!(!state.is_stale(Duration::from_secs(60)));
    }
}
