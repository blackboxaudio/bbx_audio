//! Room management for WebSocket connections.

use std::{
    collections::HashMap,
    time::{Duration, Instant},
};

use bbx_core::random::XorShiftRng;

use crate::{
    address::NodeId,
    error::{NetError, Result},
};

/// Room configuration.
pub struct RoomConfig {
    /// Number of digits in room code (4-6).
    pub code_length: usize,
    /// Room expiration time.
    pub expiration: Duration,
    /// Maximum clients per room.
    pub max_clients: usize,
}

impl Default for RoomConfig {
    fn default() -> Self {
        Self {
            code_length: 6,
            expiration: Duration::from_secs(3600 * 24),
            max_clients: 100,
        }
    }
}

/// Information about a connected client.
#[derive(Debug, Clone)]
pub struct ClientInfo {
    pub name: Option<String>,
    pub connected_at: Instant,
    pub last_activity: Instant,
}

/// A room that clients can join.
pub struct Room {
    pub code: String,
    pub created_at: Instant,
    pub clients: HashMap<NodeId, ClientInfo>,
}

impl Room {
    fn new(code: String) -> Self {
        Self {
            code,
            created_at: Instant::now(),
            clients: HashMap::new(),
        }
    }

    /// Get the number of connected clients.
    pub fn client_count(&self) -> usize {
        self.clients.len()
    }
}

/// Manages active rooms and room codes.
pub struct RoomManager {
    rooms: HashMap<String, Room>,
    config: RoomConfig,
    rng: XorShiftRng,
}

impl RoomManager {
    /// Create a new room manager with default configuration.
    pub fn new() -> Self {
        Self::with_config(RoomConfig::default())
    }

    /// Create a new room manager with custom configuration.
    pub fn with_config(config: RoomConfig) -> Self {
        let seed = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(1);

        Self {
            rooms: HashMap::new(),
            config,
            rng: XorShiftRng::new(seed),
        }
    }

    /// Generate a new room code and create the room.
    pub fn create_room(&mut self) -> String {
        loop {
            let code = self.generate_code();
            if !self.rooms.contains_key(&code) {
                self.rooms.insert(code.clone(), Room::new(code.clone()));
                return code;
            }
        }
    }

    /// Check if a room code is valid.
    pub fn room_exists(&self, code: &str) -> bool {
        self.rooms.contains_key(code)
    }

    /// Get the number of clients in a room.
    pub fn client_count(&self, code: &str) -> Option<usize> {
        self.rooms.get(code).map(|r| r.client_count())
    }

    /// Validate a room code and add client.
    pub fn join_room(&mut self, code: &str, node_id: NodeId, client_name: Option<String>) -> Result<()> {
        let room = self.rooms.get_mut(code).ok_or(NetError::InvalidRoomCode)?;

        if room.clients.len() >= self.config.max_clients {
            return Err(NetError::RoomFull);
        }

        let now = Instant::now();
        room.clients.insert(
            node_id,
            ClientInfo {
                name: client_name,
                connected_at: now,
                last_activity: now,
            },
        );

        Ok(())
    }

    /// Remove a client from their room.
    pub fn leave_room(&mut self, code: &str, node_id: NodeId) -> bool {
        if let Some(room) = self.rooms.get_mut(code) {
            room.clients.remove(&node_id).is_some()
        } else {
            false
        }
    }

    /// Update client activity timestamp.
    pub fn update_activity(&mut self, code: &str, node_id: NodeId) {
        if let Some(room) = self.rooms.get_mut(code)
            && let Some(client) = room.clients.get_mut(&node_id)
        {
            client.last_activity = Instant::now();
        }
    }

    /// Get all node IDs in a room.
    pub fn get_room_clients(&self, code: &str) -> Vec<NodeId> {
        self.rooms
            .get(code)
            .map(|r| r.clients.keys().copied().collect())
            .unwrap_or_default()
    }

    /// Close a room and return all client node IDs.
    pub fn close_room(&mut self, code: &str) -> Vec<NodeId> {
        self.rooms
            .remove(code)
            .map(|r| r.clients.keys().copied().collect())
            .unwrap_or_default()
    }

    /// Clean up expired rooms.
    ///
    /// Returns the number of rooms removed.
    pub fn cleanup_expired(&mut self) -> usize {
        let now = Instant::now();
        let expiration = self.config.expiration;
        let before = self.rooms.len();

        self.rooms
            .retain(|_, room| now.duration_since(room.created_at) < expiration);

        before - self.rooms.len()
    }

    /// Get the total number of active rooms.
    pub fn room_count(&self) -> usize {
        self.rooms.len()
    }

    fn generate_code(&mut self) -> String {
        let mut code = String::with_capacity(self.config.code_length);
        for _ in 0..self.config.code_length {
            let sample = (self.rng.next_noise_sample() + 1.0) / 2.0;
            let digit = (sample * 10.0).min(9.0) as u8;
            code.push((b'0' + digit) as char);
        }
        code
    }
}

impl Default for RoomManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_room() {
        let mut manager = RoomManager::new();
        let code = manager.create_room();

        assert_eq!(code.len(), 6);
        assert!(manager.room_exists(&code));
    }

    #[test]
    fn test_room_code_format() {
        let mut manager = RoomManager::new();
        let code = manager.create_room();

        for c in code.chars() {
            assert!(c.is_ascii_digit());
        }
    }

    #[test]
    fn test_join_room() {
        let mut manager = RoomManager::new();
        let code = manager.create_room();

        let node_id = NodeId::from_parts(1, 2);
        assert!(manager.join_room(&code, node_id, None).is_ok());
        assert_eq!(manager.client_count(&code), Some(1));
    }

    #[test]
    fn test_join_invalid_room() {
        let mut manager = RoomManager::new();
        let node_id = NodeId::from_parts(1, 2);

        let result = manager.join_room("000000", node_id, None);
        assert_eq!(result, Err(NetError::InvalidRoomCode));
    }

    #[test]
    fn test_room_capacity() {
        let config = RoomConfig {
            max_clients: 2,
            ..Default::default()
        };
        let mut manager = RoomManager::with_config(config);
        let code = manager.create_room();

        let id1 = NodeId::from_parts(1, 1);
        let id2 = NodeId::from_parts(2, 2);
        let id3 = NodeId::from_parts(3, 3);

        assert!(manager.join_room(&code, id1, None).is_ok());
        assert!(manager.join_room(&code, id2, None).is_ok());
        assert_eq!(manager.join_room(&code, id3, None), Err(NetError::RoomFull));
    }

    #[test]
    fn test_leave_room() {
        let mut manager = RoomManager::new();
        let code = manager.create_room();

        let node_id = NodeId::from_parts(5, 6);
        manager.join_room(&code, node_id, None).unwrap();

        assert!(manager.leave_room(&code, node_id));
        assert_eq!(manager.client_count(&code), Some(0));
    }

    #[test]
    fn test_get_room_clients() {
        let mut manager = RoomManager::new();
        let code = manager.create_room();

        let id1 = NodeId::from_parts(1, 1);
        let id2 = NodeId::from_parts(2, 2);

        manager.join_room(&code, id1, None).unwrap();
        manager.join_room(&code, id2, None).unwrap();

        let clients = manager.get_room_clients(&code);
        assert_eq!(clients.len(), 2);
        assert!(clients.contains(&id1));
        assert!(clients.contains(&id2));
    }

    #[test]
    fn test_close_room() {
        let mut manager = RoomManager::new();
        let code = manager.create_room();

        let node_id = NodeId::from_parts(7, 8);
        manager.join_room(&code, node_id, None).unwrap();

        let clients = manager.close_room(&code);
        assert_eq!(clients.len(), 1);
        assert!(!manager.room_exists(&code));
    }

    #[test]
    fn test_unique_codes() {
        let mut manager = RoomManager::new();
        let mut codes = Vec::new();

        for _ in 0..100 {
            codes.push(manager.create_room());
        }

        let unique: std::collections::HashSet<_> = codes.iter().collect();
        assert_eq!(unique.len(), 100);
    }
}
