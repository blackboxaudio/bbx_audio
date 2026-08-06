# OSC Audio Control

This tutorial covers using `bbx_net` to control audio parameters over the network via OSC (Open Sound Control).

## Prerequisites

> **Prior knowledge**: Familiarity with:
> - [Your First DSP Graph](first-graph.md) - Graph basics
> - [Parameter Modulation](modulation.md) - Understanding parameters

## When to Use OSC

OSC is ideal when you need to connect external applications or hardware:

- **TouchOSC** - Mobile control surfaces on phones/tablets
- **Max/MSP, Pure Data** - Visual programming environments
- **Hardware controllers** - OSC-capable MIDI controllers, Lemur
- **Live performance** - Low-latency parameter control from dedicated apps

If you need browser-based control or phone PWAs, see [WebSocket Audio Control](websocket-audio.md) instead.

## Project Configuration

Add OSC support to your `Cargo.toml`:

```toml
[dependencies]
bbx_net = { version = "0.4", features = ["osc"] }
bbx_dsp = "0.4"
```

## Core Concepts

### Producer/Consumer Pattern

Network messages flow through a lock-free ring buffer:

```
Network Thread  ─────►  NetBufferProducer  ─────►  NetBufferConsumer  ─────►  Audio Thread
   (OSC)                 (sends messages)           (receives messages)        (applies changes)
```

The audio thread never allocates or blocks—it reads messages using `drain_into_stack()`, which returns a stack-allocated array.

### Parameter Hashing

Parameters are identified by 32-bit FNV-1a hashes instead of strings. This avoids string comparisons in the audio thread:

```rust
use bbx_net::hash_param_name;

// Pre-compute hashes once at startup
let freq_hash = hash_param_name("freq");
let gain_hash = hash_param_name("gain");

// In audio callback, compare integers (fast)
if msg.param_hash == freq_hash {
    // Handle frequency change
}
```

## Creating an OSC Server

```rust
use bbx_net::{net_buffer, osc::{OscServer, OscServerConfig}};

fn main() {
    // Create the message buffer
    let (producer, mut consumer) = net_buffer(256);

    // Configure the OSC server
    let config = OscServerConfig {
        bind_addr: "0.0.0.0:9000".parse().unwrap(),
        ..Default::default()
    };

    // Start the server in a background thread
    let server = OscServer::new(config, producer);
    let _handle = server.spawn();

    // Your audio loop reads from consumer
    loop {
        let messages = consumer.drain_into_stack();
        for msg in messages {
            if let Some(value) = msg.payload.value() {
                println!("Received: hash={}, value={}", msg.param_hash, value);
            }
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
}
```

## OSC Address Format

Messages must use one of these address patterns:

| Address | Description |
|---------|-------------|
| `/blocks/param/<name>` | Broadcast to all blocks |
| `/block/<uuid>/param/<name>` | Target specific block by UUID |

Examples:
- `/blocks/param/freq` — Set frequency for all listeners
- `/blocks/param/gain` — Set gain for all listeners
- `/blocks/param/cutoff` — Set filter cutoff

## TouchOSC Setup

1. Install TouchOSC on your phone or tablet
2. Open **Connections** settings
3. Set **Host** to your computer's IP address (e.g., `192.168.1.100`)
4. Set **Port** to `9000`
5. Create a fader with address `/blocks/param/freq`
6. Set the fader range to `0.0` – `1.0`

Now moving the fader sends OSC messages to your Rust application.

### Finding Your IP Address

On macOS/Linux:
```bash
ifconfig | grep "inet " | grep -v 127.0.0.1
```

On Windows:
```bash
ipconfig | findstr /i "IPv4"
```

## Pre-configured TouchOSC Interface

A ready-to-use TouchOSC interface is included with the examples:

```
bbx_sandbox/examples/14_osc_synth.tosc
```

Import this file into TouchOSC to skip manual fader configuration. It includes four faders:
- **freq** - Oscillator frequency
- **cutoff** - Filter cutoff
- **gain** - Output gain
- **pan** - Stereo pan position

## Complete Example

Here's the full OSC synthesizer from `bbx_sandbox/examples/14_osc_synth.rs`:

```rust
use std::{
    sync::{Arc, atomic::{AtomicBool, Ordering}},
    time::Duration,
};

use bbx_dsp::{
    block::BlockId,
    blocks::{GainBlock, LowPassFilterBlock, OscillatorBlock, PannerBlock},
    buffer::{SampleBuffer, Buffer},
    context::{DEFAULT_BUFFER_SIZE, DEFAULT_SAMPLE_RATE},
    graph::GraphBuilder,
    waveform::Waveform,
};
use bbx_net::{
    NetBufferConsumer, NetMessageType, hash_param_name, net_buffer,
    osc::{OscServer, OscServerConfig},
};
use rodio::{OutputStream, Source};

const NET_BUFFER_CAPACITY: usize = 256;

struct ParamHashes {
    freq: u32,
    cutoff: u32,
    gain: u32,
    pan: u32,
}

impl ParamHashes {
    fn new() -> Self {
        Self {
            freq: hash_param_name("freq"),
            cutoff: hash_param_name("cutoff"),
            gain: hash_param_name("gain"),
            pan: hash_param_name("pan"),
        }
    }
}

fn scale_frequency(normalized: f32) -> f32 {
    let min_freq = 20.0_f32;
    let max_freq = 2000.0_f32;
    min_freq * (max_freq / min_freq).powf(normalized.clamp(0.0, 1.0))
}

fn scale_cutoff(normalized: f32) -> f32 {
    let min_cutoff = 100.0_f32;
    let max_cutoff = 5000.0_f32;
    min_cutoff * (max_cutoff / min_cutoff).powf(normalized.clamp(0.0, 1.0))
}

fn scale_gain(normalized: f32) -> f32 {
    let min_db = -60.0_f32;
    let max_db = 0.0_f32;
    min_db + (max_db - min_db) * normalized.clamp(0.0, 1.0)
}

fn scale_pan(normalized: f32) -> f32 {
    let min_pos = -100.0_f32;
    let max_pos = 100.0_f32;
    min_pos + (max_pos - min_pos) * normalized.clamp(0.0, 1.0)
}

fn main() {
    println!("BBX OSC Synthesizer");
    println!("===================\n");

    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();

    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    }).expect("Error setting Ctrl-C handler");

    // Create network buffer
    let (producer, consumer) = net_buffer(NET_BUFFER_CAPACITY);

    // Start OSC server on port 9000
    let config = OscServerConfig {
        bind_addr: "0.0.0.0:9000".parse().unwrap(),
        ..Default::default()
    };
    let osc_server = OscServer::new(config, producer);
    let _server_handle = osc_server.spawn();

    println!("OSC server listening on port 9000");
    println!("\nOSC Address Mappings:");
    println!("  /blocks/param/freq   - Oscillator frequency (0.0-1.0)");
    println!("  /blocks/param/cutoff - Filter cutoff (0.0-1.0)");
    println!("  /blocks/param/gain   - Output gain (0.0-1.0)");
    println!("  /blocks/param/pan    - Stereo pan position (0.0-1.0)");
    println!("\nPress Ctrl+C to exit.\n");

    // Build DSP graph: Oscillator → Filter → Gain → Panner → Output
    let sample_rate = DEFAULT_SAMPLE_RATE;
    let buffer_size = DEFAULT_BUFFER_SIZE;
    let num_channels = 2;

    let mut builder = GraphBuilder::new(sample_rate, buffer_size, num_channels);

    let oscillator_id = builder.add(OscillatorBlock::new(220.0, Waveform::Sawtooth, None));
    let filter_id = builder.add(LowPassFilterBlock::new(1000.0, 2.0));
    let gain_id = builder.add(GainBlock::new(-12.0, None));
    let panner_id = builder.add(PannerBlock::new(0.0));

    builder
        .connect(oscillator_id, 0, filter_id, 0)
        .connect(filter_id, 0, gain_id, 0)
        .connect(gain_id, 0, panner_id, 0);

    let mut graph = builder.build();
    let param_hashes = ParamHashes::new();

    // Audio output setup (using rodio)
    let (_stream, stream_handle) = OutputStream::try_default().unwrap();

    // ... (audio playback loop processes consumer.drain_into_stack()
    //      and applies parameter changes to graph blocks)

    while running.load(Ordering::SeqCst) {
        std::thread::sleep(Duration::from_millis(100));
    }

    println!("\nShutting down...");
}
```

## Running the Example

```bash
cargo run --release --example 14_osc_synth -p bbx_sandbox
```

The synthesizer will display your IP address and port. Connect TouchOSC to that address and use the faders to control the sound in real-time.

## Parameter Scaling

The example uses logarithmic scaling for frequency parameters (more natural for audio) and linear scaling for gain and pan:

| Parameter | Input Range | Output Range | Scaling |
|-----------|-------------|--------------|---------|
| freq | 0.0 – 1.0 | 20 – 2000 Hz | Logarithmic |
| cutoff | 0.0 – 1.0 | 100 – 5000 Hz | Logarithmic |
| gain | 0.0 – 1.0 | -60 – 0 dB | Linear |
| pan | 0.0 – 1.0 | -100 – +100 | Linear |

## Troubleshooting

**No messages received:**
- Verify TouchOSC host IP matches your computer
- Check that port 9000 isn't blocked by firewall
- Ensure both devices are on the same network

**High latency:**
- Use a dedicated WiFi network (not congested public WiFi)
- Reduce buffer size if possible
- OSC over UDP is fast; latency issues are usually network-related

## Next Steps

- [WebSocket Audio Control](websocket-audio.md) — Browser-based control
- [OSC Server Reference](../crates/net/osc.md) — Configuration options
- [Network Architecture](../architecture/networking.md) — Lock-free design details
- [Real-Time Safety](../architecture/realtime-safety.md) — Understanding realtime constraints
