# OSC Server

UDP-based OSC receiver for TouchOSC, Max/MSP, Pure Data, and other creative tools.

## Overview

The OSC server listens for UDP packets on a configurable port and converts them to `NetMessage` structs that can be consumed by the audio thread. OSC is ideal for low-latency control from apps like TouchOSC on mobile devices or Max/MSP patches.

## Enabling OSC

Add bbx_net with the `osc` feature:

```toml
[dependencies]
bbx_net = { version = "0.4", features = ["osc"] }
```

## Basic Usage

```rust
use bbx_net::{net_buffer, osc::{OscServer, OscServerConfig}};

let (producer, mut consumer) = net_buffer(256);

let config = OscServerConfig {
    bind_addr: "0.0.0.0:9000".parse().unwrap(),
    ..Default::default()
};

let server = OscServer::new(config, producer);
let handle = server.spawn(); // Runs in background thread

// In audio callback:
let messages = consumer.drain_into_stack();
for msg in messages {
    // Handle parameter changes
}
```

## Configuration

`OscServerConfig` options:

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `bind_addr` | `SocketAddr` | `0.0.0.0:9000` | UDP address to listen on |
| `node_id` | `NodeId` | Random | Identifier for this server |
| `buffer_size` | `usize` | 1024 | UDP receive buffer size in bytes |

## OSC Address Format

Messages must follow one of these patterns:

| Address Pattern | Description |
|-----------------|-------------|
| `/blocks/param/<name>` | Broadcast parameter to all blocks |
| `/block/<uuid>/param/<name>` | Target specific block by UUID |

Examples:
- `/blocks/param/gain` — Set gain parameter for all blocks
- `/blocks/param/freq` — Set frequency parameter for all blocks
- `/blocks/param/cutoff` — Set filter cutoff for all blocks
- `/block/a1b2c3d4-e5f6-7890-abcd-ef1234567890/param/gain` — Target specific block

## Supported OSC Types

The first argument of each OSC message is converted to `f32`:

| OSC Type | Conversion |
|----------|------------|
| Float | Direct |
| Int | Cast to f32 |
| Double | Cast to f32 |
| Bool | `true` → 1.0, `false` → 0.0 |

## TouchOSC Setup

To control your application from TouchOSC:

1. Open TouchOSC settings
2. Set **Host** to your computer's IP address (e.g., `192.168.1.100`)
3. Set **Port** to `9000` (or your configured port)
4. Create faders with addresses like `/blocks/param/freq`, `/blocks/param/gain`
5. Set fader value range to 0.0–1.0

## Thread Model

The OSC server runs in a dedicated thread using blocking UDP receive. This design:

- Keeps the main thread free
- Provides low latency for incoming packets
- Uses the lock-free buffer to communicate with the audio thread

```rust
// spawn() creates a background thread
let handle = server.spawn();

// Alternatively, run() blocks the current thread
// server.run();
```

## Error Handling

Invalid OSC packets are silently dropped. The server continues listening even if malformed packets are received.
