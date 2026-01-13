//! Block identification and address parsing for network messages.
//!
//! Provides `NodeId` for unique identification and `AddressPath` for
//! parsing OSC-style address patterns that target DSP blocks.

use std::sync::atomic::{AtomicU64, Ordering};

use bbx_core::random::XorShiftRng;

use crate::error::{NetError, Result};

static NODE_ID_COUNTER: AtomicU64 = AtomicU64::new(0);

/// A 128-bit node identifier generated from XorShiftRng.
///
/// NodeIds uniquely identify clients and nodes in the network. They are
/// formatted as UUID-style strings for human readability and protocol
/// compatibility.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct NodeId {
    pub high: u64,
    pub low: u64,
}

impl NodeId {
    /// Generate a new random NodeId using XorShiftRng.
    ///
    /// # Example
    ///
    /// ```
    /// use bbx_core::random::XorShiftRng;
    /// use bbx_net::address::NodeId;
    ///
    /// let mut rng = XorShiftRng::new(12345);
    /// let node_id = NodeId::generate(&mut rng);
    /// ```
    pub fn generate(rng: &mut XorShiftRng) -> Self {
        let sample1 = (rng.next_noise_sample() + 1.0) / 2.0;
        let sample2 = (rng.next_noise_sample() + 1.0) / 2.0;
        Self {
            high: (sample1 * u64::MAX as f64) as u64,
            low: (sample2 * u64::MAX as f64) as u64,
        }
    }

    /// Generate a NodeId with mixed entropy sources for improved unpredictability.
    ///
    /// Combines multiple entropy sources: time, process ID, memory address (ASLR),
    /// and a monotonic counter to create a more unpredictable seed.
    pub fn generate_with_entropy(clock_micros: u64) -> Self {
        let counter = NODE_ID_COUNTER.fetch_add(1, Ordering::Relaxed);
        let pid = std::process::id() as u64;
        let addr_entropy = &counter as *const _ as u64;

        let seed = clock_micros.wrapping_mul(0x517cc1b727220a95)
            ^ pid.rotate_left(17)
            ^ addr_entropy.rotate_left(31)
            ^ counter.wrapping_mul(0x2545f4914f6cdd1d);

        let mut rng = XorShiftRng::new(seed.max(1));
        Self::generate(&mut rng)
    }

    /// Generate a unique NodeId that doesn't collide with existing IDs.
    ///
    /// Uses `generate_with_entropy` internally and loops until finding an ID
    /// that the `exists` predicate returns false for.
    pub fn generate_unique<F>(clock_micros: u64, mut exists: F) -> Self
    where
        F: FnMut(&NodeId) -> bool,
    {
        loop {
            let candidate = Self::generate_with_entropy(clock_micros);
            if !exists(&candidate) {
                return candidate;
            }
        }
    }

    /// Create a NodeId from explicit high and low parts.
    pub const fn from_parts(high: u64, low: u64) -> Self {
        Self { high, low }
    }

    /// Format as hyphenated UUID string.
    ///
    /// Returns a string in the format `xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx`.
    pub fn to_uuid_string(&self) -> String {
        format!(
            "{:08x}-{:04x}-{:04x}-{:04x}-{:012x}",
            (self.high >> 32) as u32,
            ((self.high >> 16) & 0xFFFF) as u16,
            (self.high & 0xFFFF) as u16,
            ((self.low >> 48) & 0xFFFF) as u16,
            (self.low & 0xFFFF_FFFF_FFFF)
        )
    }

    /// Parse from UUID string.
    ///
    /// Accepts the format `xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx`.
    pub fn from_uuid_string(s: &str) -> Result<Self> {
        let s = s.trim();
        let parts: Vec<&str> = s.split('-').collect();

        if parts.len() != 5 {
            return Err(NetError::InvalidNodeId);
        }

        let p0 = u32::from_str_radix(parts[0], 16).map_err(|_| NetError::InvalidNodeId)?;
        let p1 = u16::from_str_radix(parts[1], 16).map_err(|_| NetError::InvalidNodeId)?;
        let p2 = u16::from_str_radix(parts[2], 16).map_err(|_| NetError::InvalidNodeId)?;
        let p3 = u16::from_str_radix(parts[3], 16).map_err(|_| NetError::InvalidNodeId)?;
        let p4 = u64::from_str_radix(parts[4], 16).map_err(|_| NetError::InvalidNodeId)?;

        if p4 > 0xFFFF_FFFF_FFFF {
            return Err(NetError::InvalidNodeId);
        }

        let high = ((p0 as u64) << 32) | ((p1 as u64) << 16) | (p2 as u64);
        let low = ((p3 as u64) << 48) | p4;

        Ok(Self { high, low })
    }
}

/// Parsed OSC-style address path.
///
/// Supports formats:
/// - `/block/<uuid>/param/<name>` - Specific block parameter
/// - `/blocks/param/<name>` - Broadcast to all blocks
#[derive(Debug, Clone)]
pub struct AddressPath {
    /// Target block (None for broadcast).
    pub block_id: Option<NodeId>,
    /// Parameter name.
    pub param_name: String,
}

impl AddressPath {
    /// Parse an OSC address string.
    ///
    /// # Supported Formats
    ///
    /// - `/block/<uuid>/param/<name>` - Specific block parameter
    /// - `/blocks/param/<name>` - Broadcast to all blocks
    ///
    /// # Example
    ///
    /// ```
    /// use bbx_net::address::AddressPath;
    ///
    /// let path = AddressPath::parse("/blocks/param/gain").unwrap();
    /// assert!(path.block_id.is_none());
    /// assert_eq!(path.param_name, "gain");
    /// ```
    pub fn parse(address: &str) -> Result<Self> {
        let parts: Vec<&str> = address.split('/').filter(|s| !s.is_empty()).collect();

        match parts.as_slice() {
            ["block", uuid, "param", name] => {
                let block_id = NodeId::from_uuid_string(uuid)?;
                Ok(Self {
                    block_id: Some(block_id),
                    param_name: (*name).to_string(),
                })
            }
            ["blocks", "param", name] => Ok(Self {
                block_id: None,
                param_name: (*name).to_string(),
            }),
            _ => Err(NetError::InvalidAddress),
        }
    }

    /// Format as an OSC address string.
    pub fn to_address_string(&self) -> String {
        match &self.block_id {
            Some(id) => format!("/block/{}/param/{}", id.to_uuid_string(), self.param_name),
            None => format!("/blocks/param/{}", self.param_name),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_id_generate() {
        let mut rng = XorShiftRng::new(12345);
        let id1 = NodeId::generate(&mut rng);
        let id2 = NodeId::generate(&mut rng);

        assert_ne!(id1, id2);
        assert_ne!(id1.high, 0);
        assert_ne!(id1.low, 0);
    }

    #[test]
    fn test_node_id_deterministic() {
        let mut rng1 = XorShiftRng::new(42);
        let mut rng2 = XorShiftRng::new(42);

        let id1 = NodeId::generate(&mut rng1);
        let id2 = NodeId::generate(&mut rng2);

        assert_eq!(id1, id2);
    }

    #[test]
    fn test_node_id_uuid_roundtrip() {
        let mut rng = XorShiftRng::new(54321);
        let original = NodeId::generate(&mut rng);

        let uuid_str = original.to_uuid_string();
        let parsed = NodeId::from_uuid_string(&uuid_str).unwrap();

        assert_eq!(original, parsed);
    }

    #[test]
    fn test_node_id_uuid_format() {
        let id = NodeId::from_parts(0x12345678_abcd_ef01, 0x2345_6789abcdef01);
        let uuid = id.to_uuid_string();

        assert_eq!(uuid, "12345678-abcd-ef01-2345-6789abcdef01");
    }

    #[test]
    fn test_node_id_invalid_uuid() {
        assert!(NodeId::from_uuid_string("invalid").is_err());
        assert!(NodeId::from_uuid_string("12345678-abcd").is_err());
        assert!(NodeId::from_uuid_string("zzzzzzzz-zzzz-zzzz-zzzz-zzzzzzzzzzzz").is_err());
    }

    #[test]
    fn test_address_path_broadcast() {
        let path = AddressPath::parse("/blocks/param/gain").unwrap();
        assert!(path.block_id.is_none());
        assert_eq!(path.param_name, "gain");
    }

    #[test]
    fn test_address_path_targeted() {
        let id = NodeId::from_parts(0x12345678_abcd_ef01, 0x2345_6789abcdef01);
        let address = format!("/block/{}/param/volume", id.to_uuid_string());

        let path = AddressPath::parse(&address).unwrap();
        assert_eq!(path.block_id, Some(id));
        assert_eq!(path.param_name, "volume");
    }

    #[test]
    fn test_address_path_roundtrip() {
        let original = AddressPath {
            block_id: Some(NodeId::from_parts(0xaabb_ccdd_eeff_0011, 0x2233_445566778899)),
            param_name: "frequency".to_string(),
        };

        let address_str = original.to_address_string();
        let parsed = AddressPath::parse(&address_str).unwrap();

        assert_eq!(original.block_id, parsed.block_id);
        assert_eq!(original.param_name, parsed.param_name);
    }

    #[test]
    fn test_address_path_invalid() {
        assert!(AddressPath::parse("/invalid/path").is_err());
        assert!(AddressPath::parse("/block/notauuid/param/test").is_err());
        assert!(AddressPath::parse("").is_err());
    }

    #[test]
    fn test_generate_with_entropy_uniqueness() {
        let mut ids = std::collections::HashSet::new();
        for _ in 0..10_000 {
            let id = NodeId::generate_with_entropy(0);
            assert!(ids.insert(id), "Collision detected");
        }
    }

    #[test]
    fn test_generate_unique_avoids_collision() {
        let existing = NodeId::from_parts(123, 456);
        let mut attempts = 0;
        let new_id = NodeId::generate_unique(0, |id| {
            attempts += 1;
            *id == existing && attempts == 1
        });
        assert_ne!(new_id, existing);
    }
}
