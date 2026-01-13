//! OSC message parsing utilities.

use rosc::{OscMessage, OscPacket, OscType};

use crate::{
    address::{AddressPath, NodeId},
    clock::SyncedTimestamp,
    error::Result,
    message::{NetMessage, NetMessageType, hash_param_name},
};

/// Parsed OSC data ready for conversion to NetMessage.
pub struct ParsedOscMessage {
    pub address: AddressPath,
    pub value: f32,
}

/// Parse an OSC packet into network messages.
///
/// Handles both single messages and bundles. Returns a Vec of parsed messages.
pub fn parse_osc_packet(packet: &OscPacket) -> Vec<ParsedOscMessage> {
    let mut messages = Vec::new();
    collect_messages(packet, &mut messages);
    messages
}

fn collect_messages(packet: &OscPacket, out: &mut Vec<ParsedOscMessage>) {
    match packet {
        OscPacket::Message(msg) => {
            if let Some(parsed) = parse_single_message(msg) {
                out.push(parsed);
            }
        }
        OscPacket::Bundle(bundle) => {
            for content in &bundle.content {
                collect_messages(content, out);
            }
        }
    }
}

fn parse_single_message(msg: &OscMessage) -> Option<ParsedOscMessage> {
    let address = AddressPath::parse(&msg.addr).ok()?;

    let value = msg.args.first().and_then(|arg| match arg {
        OscType::Float(f) => Some(*f),
        OscType::Int(i) => Some(*i as f32),
        OscType::Double(d) => Some(*d as f32),
        OscType::Bool(b) => Some(if *b { 1.0 } else { 0.0 }),
        _ => None,
    })?;

    Some(ParsedOscMessage { address, value })
}

/// Parse a raw OSC message and convert to NetMessage.
///
/// # Arguments
///
/// * `data` - Raw OSC packet bytes
/// * `source_node_id` - Node ID to attribute the message to
///
/// # Returns
///
/// A vector of NetMessages parsed from the packet.
pub fn parse_osc_message(data: &[u8], source_node_id: NodeId) -> Result<Vec<NetMessage>> {
    let packet = rosc::decoder::decode_udp(data)
        .map_err(|_| crate::error::NetError::ParseError)?
        .1;

    let parsed = parse_osc_packet(&packet);

    Ok(parsed
        .into_iter()
        .map(|p| NetMessage {
            message_type: NetMessageType::ParameterChange,
            param_hash: hash_param_name(&p.address.param_name),
            value: p.value,
            node_id: source_node_id,
            timestamp: SyncedTimestamp::default(),
        })
        .collect())
}

#[cfg(test)]
mod tests {
    use rosc::encoder;

    use super::*;

    #[test]
    fn test_parse_float_message() {
        let msg = OscMessage {
            addr: "/blocks/param/gain".to_string(),
            args: vec![OscType::Float(0.75)],
        };
        let packet = OscPacket::Message(msg);
        let bytes = encoder::encode(&packet).unwrap();

        let node_id = NodeId::default();
        let messages = parse_osc_message(&bytes, node_id).unwrap();

        assert_eq!(messages.len(), 1);
        assert!((messages[0].value - 0.75).abs() < f32::EPSILON);
    }

    #[test]
    fn test_parse_int_message() {
        let msg = OscMessage {
            addr: "/blocks/param/level".to_string(),
            args: vec![OscType::Int(100)],
        };
        let packet = OscPacket::Message(msg);
        let bytes = encoder::encode(&packet).unwrap();

        let node_id = NodeId::default();
        let messages = parse_osc_message(&bytes, node_id).unwrap();

        assert_eq!(messages.len(), 1);
        assert!((messages[0].value - 100.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_parse_bool_message() {
        let msg = OscMessage {
            addr: "/blocks/param/enabled".to_string(),
            args: vec![OscType::Bool(true)],
        };
        let packet = OscPacket::Message(msg);
        let bytes = encoder::encode(&packet).unwrap();

        let node_id = NodeId::default();
        let messages = parse_osc_message(&bytes, node_id).unwrap();

        assert_eq!(messages.len(), 1);
        assert!((messages[0].value - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_parse_invalid_address() {
        let msg = OscMessage {
            addr: "/invalid/path".to_string(),
            args: vec![OscType::Float(0.5)],
        };
        let packet = OscPacket::Message(msg);
        let bytes = encoder::encode(&packet).unwrap();

        let node_id = NodeId::default();
        let messages = parse_osc_message(&bytes, node_id).unwrap();

        assert!(messages.is_empty());
    }
}
