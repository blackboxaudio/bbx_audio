# WebSocket Server

Room-based WebSocket server for phone PWAs and browser-based control interfaces.

## Overview

The WebSocket server provides bidirectional communication over JSON, with room-based authentication for installations where multiple users connect via their phones. It includes ping/pong for latency measurement and clock synchronization.

## Enabling WebSocket

Add bbx_net with the `websocket` feature:

```toml
[dependencies]
bbx_net = { version = "0.4", features = ["websocket"] }
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
```

## Basic Usage

```rust
use bbx_net::{
    net_buffer,
    websocket::{WsServer, WsServerConfig, ServerCommand},
};
use tokio::sync::mpsc;

#[tokio::main]
async fn main() {
    let (producer, mut consumer) = net_buffer(256);
    let (command_tx, command_rx) = mpsc::channel(256);

    let config = WsServerConfig {
        bind_addr: "0.0.0.0:8080".parse().unwrap(),
        ..Default::default()
    };

    let server = WsServer::new(config, producer, command_rx);

    // Create a room for clients to join
    let (response_tx, response_rx) = tokio::sync::oneshot::channel();
    command_tx.send(ServerCommand::CreateRoom { response: response_tx }).await.unwrap();
    let room_code = response_rx.await.unwrap();
    println!("Room code: {room_code}");

    // Run server (blocks)
    server.run().await.unwrap();
}
```

## Configuration

`WsServerConfig` options:

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `bind_addr` | `SocketAddr` | `0.0.0.0:8080` | TCP address to listen on |
| `ping_interval` | `Duration` | 5 seconds | How often to ping clients |
| `connection_timeout` | `Duration` | 30 seconds | Disconnect after no pong |
| `broadcast_capacity` | `usize` | 256 | Broadcast channel buffer size |

## JSON Protocol

### Client to Server

```json
// Join a room (required first message)
{"type": "join", "room_code": "1234", "client_name": "Phone 1"}

// Send parameter change
{"type": "param", "param": "freq", "value": 0.5}

// Send parameter with scheduled time
{"type": "param", "param": "freq", "value": 0.5, "at": 1234567890}

// Send trigger event
{"type": "trigger", "name": "note_on"}

// Request current parameter state
{"type": "sync"}

// Measure latency
{"type": "ping", "client_time": 1234567890}

// Disconnect gracefully
{"type": "leave"}
```

### Server to Client

```json
// Welcome after successful join
{"type": "welcome", "node_id": "uuid-string", "server_time": 1234567890}

// Current parameter state (response to sync)
{"type": "state", "params": [
    {"name": "freq", "value": 0.5, "min": 0.0, "max": 1.0},
    {"name": "gain", "value": 0.8, "min": 0.0, "max": 1.0}
]}

// Broadcast parameter update
{"type": "update", "param": "freq", "value": 0.7}

// Pong response with server time
{"type": "pong", "client_time": 1234567890, "server_time": 1234567891}

// Error notification
{"type": "error", "code": "INVALID_ROOM", "message": "Invalid room code"}

// Room closed by server
{"type": "closed"}
```

## Room Management

Rooms provide simple authentication via numeric codes:

```rust
use bbx_net::websocket::{ServerCommand, RoomConfig};

// Create a room (returns room code like "1234")
let (tx, rx) = tokio::sync::oneshot::channel();
command_tx.send(ServerCommand::CreateRoom { response: tx }).await?;
let room_code = rx.await?;

// Close a room (disconnects all clients)
command_tx.send(ServerCommand::CloseRoom { code: room_code }).await?;
```

Room configuration:

| Option | Default | Description |
|--------|---------|-------------|
| Code length | 4 digits | Numeric code for easy phone entry |
| Expiration | 24 hours | Rooms auto-close after this time |
| Max clients | 100 | Per-room connection limit |

## Server Commands

Control the server from your main application:

```rust
pub enum ServerCommand {
    // Broadcast parameter to all connected clients
    BroadcastParam { name: String, value: f32 },

    // Broadcast visualization data
    BroadcastVisualization { data: Vec<f32> },

    // Create a new room (returns code via oneshot)
    CreateRoom { response: oneshot::Sender<String> },

    // Close and evict a room
    CloseRoom { code: String },

    // Graceful shutdown
    Shutdown,
}
```

## Clock Synchronization

The server tracks clock offset for each client using NTP-style calculation:

1. Client sends `ping` with its local timestamp
2. Server responds with `pong` containing both timestamps
3. Client calculates round-trip time and clock offset

This enables sample-accurate scheduling of parameter changes across the network.

## Thread Model

The WebSocket server uses tokio's async runtime:

- Main server task accepts connections
- Per-connection tasks handle message parsing
- All tasks share the `NetBufferProducer` for sending to audio thread
- Broadcast channel distributes server-to-client updates

```rust
// The server requires a tokio runtime
#[tokio::main]
async fn main() {
    // ...
    server.run().await.unwrap();
}
```
