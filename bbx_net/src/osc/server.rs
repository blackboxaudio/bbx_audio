//! OSC UDP server for receiving control messages.

use std::{
    net::{SocketAddr, UdpSocket},
    sync::Arc,
    thread::{self, JoinHandle},
};

use crate::{
    address::NodeId,
    buffer::NetBufferProducer,
    clock::ClockSync,
    error::{NetError, Result},
    osc::parser::parse_osc_message,
};

/// Configuration for the OSC server.
pub struct OscServerConfig {
    /// Address to bind to (e.g., "0.0.0.0:9000").
    pub bind_addr: SocketAddr,
    /// This node's ID for filtering targeted messages.
    pub node_id: NodeId,
    /// Buffer size for UDP packets.
    pub recv_buffer_size: usize,
}

impl Default for OscServerConfig {
    fn default() -> Self {
        Self {
            bind_addr: "0.0.0.0:9000".parse().unwrap(),
            node_id: NodeId::default(),
            recv_buffer_size: 1024,
        }
    }
}

/// OSC server for TouchOSC/Max/PD integration.
///
/// Listens on a UDP port for OSC messages and forwards them to a
/// `NetBufferProducer` for consumption by the audio thread.
pub struct OscServer {
    config: OscServerConfig,
    producer: NetBufferProducer,
    clock_sync: Arc<ClockSync>,
}

impl OscServer {
    /// Create a new OSC server.
    ///
    /// # Arguments
    ///
    /// * `config` - Server configuration
    /// * `producer` - Buffer producer for sending messages to audio thread
    pub fn new(config: OscServerConfig, producer: NetBufferProducer) -> Self {
        Self {
            config,
            producer,
            clock_sync: Arc::new(ClockSync::new()),
        }
    }

    /// Create with default configuration.
    pub fn with_defaults(producer: NetBufferProducer) -> Self {
        Self::new(OscServerConfig::default(), producer)
    }

    /// Get the clock synchronization instance.
    pub fn clock(&self) -> Arc<ClockSync> {
        Arc::clone(&self.clock_sync)
    }

    /// Start the OSC server in the current thread (blocking).
    ///
    /// This method blocks forever, listening for OSC messages.
    /// Call from a dedicated thread.
    pub fn run(mut self) -> Result<()> {
        let socket = UdpSocket::bind(self.config.bind_addr)?;
        let mut buf = vec![0u8; self.config.recv_buffer_size];

        loop {
            match socket.recv_from(&mut buf) {
                Ok((len, src_addr)) => {
                    self.handle_packet(&buf[..len], src_addr);
                }
                Err(e) => {
                    if e.kind() != std::io::ErrorKind::WouldBlock {
                        return Err(NetError::IoError);
                    }
                }
            }
        }
    }

    /// Start the OSC server in a background thread.
    ///
    /// Returns a `JoinHandle` that can be used to wait for the server
    /// to finish (which normally only happens on error).
    pub fn spawn(self) -> JoinHandle<Result<()>> {
        thread::spawn(move || self.run())
    }

    fn handle_packet(&mut self, data: &[u8], _src_addr: SocketAddr) {
        let source_node_id = self.config.node_id;

        if let Ok(messages) = parse_osc_message(data, source_node_id) {
            for mut msg in messages {
                msg.timestamp = self.clock_sync.now();

                let _ = self.producer.try_send(msg);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::buffer::net_buffer;

    #[test]
    fn test_osc_server_config_default() {
        let config = OscServerConfig::default();
        assert_eq!(config.bind_addr.port(), 9000);
        assert_eq!(config.recv_buffer_size, 1024);
    }

    #[test]
    fn test_osc_server_creation() {
        let (producer, _consumer) = net_buffer(64);
        let server = OscServer::with_defaults(producer);
        assert!(Arc::strong_count(&server.clock_sync) >= 1);
    }
}
