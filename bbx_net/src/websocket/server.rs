//! WebSocket server for phone PWA connections.

use std::{collections::HashMap, net::SocketAddr, sync::Arc};

use tokio::{
    net::TcpListener,
    sync::{RwLock, broadcast, mpsc},
};

use crate::{
    address::NodeId,
    buffer::NetBufferProducer,
    clock::ClockSync,
    error::{NetError, Result},
    message::{NetMessage, NetMessageType, hash_param_name},
    websocket::{
        connection::ConnectionState,
        protocol::{ClientMessage, ParamState, ServerMessage},
        room::RoomManager,
    },
};

/// Configuration for the WebSocket server.
pub struct WsServerConfig {
    /// Address to bind to (e.g., "0.0.0.0:8080").
    pub bind_addr: SocketAddr,
    /// Maximum connections per room.
    pub max_connections_per_room: usize,
    /// Ping interval in milliseconds.
    pub ping_interval_ms: u64,
    /// Connection timeout in milliseconds.
    pub connection_timeout_ms: u64,
    /// Capacity for the state broadcast channel.
    pub broadcast_capacity: usize,
}

impl Default for WsServerConfig {
    fn default() -> Self {
        Self {
            bind_addr: "0.0.0.0:8080".parse().unwrap(),
            max_connections_per_room: 100,
            ping_interval_ms: 5000,
            connection_timeout_ms: 30000,
            broadcast_capacity: 1024,
        }
    }
}

/// Commands from main thread to server.
pub enum ServerCommand {
    /// Broadcast a parameter update to all connected clients.
    BroadcastParam { name: String, value: f32 },
    /// Broadcast visualization data to all clients.
    BroadcastVisualization { data: Vec<f32> },
    /// Create a new room and return its code via the response channel.
    CreateRoom {
        response: tokio::sync::oneshot::Sender<String>,
    },
    /// Close a room.
    CloseRoom { code: String },
    /// Shutdown the server.
    Shutdown,
}

/// WebSocket server for phone PWA connections.
pub struct WsServer {
    config: WsServerConfig,
    room_manager: Arc<RwLock<RoomManager>>,
    clock_sync: Arc<ClockSync>,
    #[allow(dead_code)]
    producer: NetBufferProducer,
    state_broadcast: broadcast::Sender<ServerMessage>,
    command_rx: mpsc::Receiver<ServerCommand>,
    connections: Arc<RwLock<HashMap<NodeId, ConnectionState>>>,
}

impl WsServer {
    /// Create a new WebSocket server.
    ///
    /// # Arguments
    ///
    /// * `config` - Server configuration
    /// * `producer` - Buffer producer for sending messages to audio thread
    /// * `command_rx` - Channel for receiving commands from main thread
    pub fn new(config: WsServerConfig, producer: NetBufferProducer, command_rx: mpsc::Receiver<ServerCommand>) -> Self {
        let (state_broadcast, _) = broadcast::channel(config.broadcast_capacity);

        Self {
            config,
            room_manager: Arc::new(RwLock::new(RoomManager::new())),
            clock_sync: Arc::new(ClockSync::new()),
            producer,
            state_broadcast,
            command_rx,
            connections: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Create a command channel pair for communicating with the server.
    pub fn command_channel() -> (mpsc::Sender<ServerCommand>, mpsc::Receiver<ServerCommand>) {
        mpsc::channel(256)
    }

    /// Get the clock synchronization instance.
    pub fn clock(&self) -> Arc<ClockSync> {
        Arc::clone(&self.clock_sync)
    }

    /// Get a handle to subscribe to state broadcasts.
    pub fn subscribe(&self) -> broadcast::Receiver<ServerMessage> {
        self.state_broadcast.subscribe()
    }

    /// Run the WebSocket server.
    ///
    /// This method runs until a shutdown command is received or an error occurs.
    pub async fn run(mut self) -> Result<()> {
        let listener = TcpListener::bind(&self.config.bind_addr)
            .await
            .map_err(|_| NetError::ConnectionFailed)?;

        loop {
            tokio::select! {
                accept_result = listener.accept() => {
                    match accept_result {
                        Ok((stream, addr)) => {
                            self.handle_connection(stream, addr).await;
                        }
                        Err(_) => {
                            continue;
                        }
                    }
                }

                Some(cmd) = self.command_rx.recv() => {
                    match cmd {
                        ServerCommand::Shutdown => {
                            return Ok(());
                        }
                        ServerCommand::BroadcastParam { name, value } => {
                            let _ = self.state_broadcast.send(ServerMessage::Update {
                                param: name,
                                value,
                            });
                        }
                        ServerCommand::BroadcastVisualization { data: _ } => {
                            // Visualization broadcasts would be handled here
                        }
                        ServerCommand::CreateRoom { response } => {
                            let code = self.room_manager.write().await.create_room();
                            let _ = response.send(code);
                        }
                        ServerCommand::CloseRoom { code } => {
                            let clients = self.room_manager.write().await.close_room(&code);
                            for node_id in clients {
                                self.connections.write().await.remove(&node_id);
                            }
                            let _ = self.state_broadcast.send(ServerMessage::RoomClosed);
                        }
                    }
                }
            }
        }
    }

    async fn handle_connection(&self, stream: tokio::net::TcpStream, _addr: SocketAddr) {
        use futures_util::{SinkExt, StreamExt};
        use tokio_tungstenite::accept_async;

        let ws_stream = match accept_async(stream).await {
            Ok(ws) => ws,
            Err(_) => return,
        };

        let (mut write, mut read) = ws_stream.split();

        let room_manager = Arc::clone(&self.room_manager);
        let clock_sync = Arc::clone(&self.clock_sync);
        let connections = Arc::clone(&self.connections);
        let mut broadcast_rx = self.state_broadcast.subscribe();

        tokio::spawn(async move {
            let mut node_id: Option<NodeId> = None;
            let mut room_code: Option<String> = None;
            let mut should_cleanup = true;

            loop {
                tokio::select! {
                    msg = read.next() => {
                        match msg {
                            Some(Ok(tokio_tungstenite::tungstenite::Message::Text(text))) => {
                                if let Ok(client_msg) = serde_json::from_str::<ClientMessage>(&text) {
                                    match client_msg {
                                        ClientMessage::Join { room_code: code, client_name } => {
                                            let mut rm = room_manager.write().await;
                                            let mut rng = bbx_core::random::XorShiftRng::new(
                                                clock_sync.now().as_micros()
                                            );
                                            let new_node_id = NodeId::generate(&mut rng);

                                            match rm.join_room(&code, new_node_id, client_name.clone()) {
                                                Ok(()) => {
                                                    node_id = Some(new_node_id);
                                                    room_code = Some(code);

                                                    let state = ConnectionState::new(
                                                        new_node_id,
                                                        room_code.clone().unwrap(),
                                                        client_name,
                                                    );
                                                    connections.write().await.insert(new_node_id, state);

                                                    let response = ServerMessage::Welcome {
                                                        node_id: new_node_id.to_uuid_string(),
                                                        server_time: clock_sync.now().as_micros(),
                                                    };

                                                    let json = serde_json::to_string(&response).unwrap();
                                                    let _ = write.send(
                                                        tokio_tungstenite::tungstenite::Message::Text(json)
                                                    ).await;
                                                }
                                                Err(NetError::InvalidRoomCode) => {
                                                    let response = ServerMessage::invalid_room_code();
                                                    let json = serde_json::to_string(&response).unwrap();
                                                    let _ = write.send(
                                                        tokio_tungstenite::tungstenite::Message::Text(json)
                                                    ).await;
                                                }
                                                Err(NetError::RoomFull) => {
                                                    let response = ServerMessage::room_full();
                                                    let json = serde_json::to_string(&response).unwrap();
                                                    let _ = write.send(
                                                        tokio_tungstenite::tungstenite::Message::Text(json)
                                                    ).await;
                                                }
                                                Err(_) => {}
                                            }
                                        }
                                        ClientMessage::Parameter { param, value, at: _ } => {
                                            if let Some(nid) = node_id {
                                                let _msg = NetMessage {
                                                    message_type: NetMessageType::ParameterChange,
                                                    param_hash: hash_param_name(&param),
                                                    value,
                                                    node_id: nid,
                                                    timestamp: clock_sync.now(),
                                                };
                                                // TODO: Send msg via mpsc channel to producer
                                            }
                                        }
                                        ClientMessage::Ping { client_time } => {
                                            let server_time = clock_sync.now().as_micros();
                                            let response = ServerMessage::Pong {
                                                client_time,
                                                server_time,
                                            };
                                            let json = serde_json::to_string(&response).unwrap();
                                            let _ = write.send(
                                                tokio_tungstenite::tungstenite::Message::Text(json)
                                            ).await;
                                        }
                                        ClientMessage::Leave => {
                                            if let (Some(nid), Some(code)) = (node_id, &room_code) {
                                                room_manager.write().await.leave_room(code, nid);
                                                connections.write().await.remove(&nid);
                                            }
                                            should_cleanup = false;
                                            break;
                                        }
                                        _ => {}
                                    }
                                }
                            }
                            Some(Ok(tokio_tungstenite::tungstenite::Message::Close(_))) | None => {
                                break;
                            }
                            _ => {}
                        }
                    }

                    Ok(broadcast_msg) = broadcast_rx.recv() => {
                        if node_id.is_some() {
                            let json = serde_json::to_string(&broadcast_msg).unwrap();
                            let _ = write.send(
                                tokio_tungstenite::tungstenite::Message::Text(json)
                            ).await;
                        }
                    }
                }
            }

            if should_cleanup
                && let (Some(nid), Some(code)) = (node_id, room_code) {
                    room_manager.write().await.leave_room(&code, nid);
                    connections.write().await.remove(&nid);
                }
        });
    }
}

/// Create a parameter state list for state synchronization.
pub fn create_param_state_list(params: &[(String, f32)]) -> Vec<ParamState> {
    params
        .iter()
        .map(|(name, value)| ParamState::new(name.clone(), *value))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_config_default() {
        let config = WsServerConfig::default();
        assert_eq!(config.bind_addr.port(), 8080);
        assert_eq!(config.max_connections_per_room, 100);
    }

    #[test]
    fn test_create_param_state_list() {
        let params = vec![("gain".to_string(), 0.5), ("freq".to_string(), 440.0)];

        let states = create_param_state_list(&params);

        assert_eq!(states.len(), 2);
        assert_eq!(states[0].name, "gain");
        assert!((states[0].value - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn test_command_channel() {
        let (tx, _rx) = WsServer::command_channel();
        assert!(!tx.is_closed());
    }
}
