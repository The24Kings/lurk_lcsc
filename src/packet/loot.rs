use serde::{Deserialize, Serialize};
use std::io::Write;

use crate::packet::PktType;
use crate::{Packet, Parser};

/// Represents a loot packet containing the message type and target name.
#[derive(Serialize, Deserialize)]
pub struct PktLoot {
    /// The type of the packet message.
    pub packet_type: PktType,
    /// The name of the loot target.
    pub target_name: Box<str>,
}

impl PktLoot {
    /// Create a new PktLoot packet from a given name
    pub fn loot(name: &str) -> Self {
        Self {
            packet_type: PktType::LOOT,
            target_name: Box::from(name),
        }
    }
}

#[macro_export]
/// Send `PktLoot` over `TcpStream` to connected user
///
/// ```no_run
/// use lurk_lcsc::{Protocol, PktLoot, LurkError, send_loot};
/// use std::sync::Arc;
/// use std::net::TcpStream;
///
/// let stream = Arc::new(TcpStream::connect("127.0.0.1:8080").unwrap());
///
/// send_loot!(stream.clone(), PktLoot::loot("Test"))
/// ```
macro_rules! send_loot {
    ($stream:expr, $pkt_loot:expr) => {
        if let Err(e) = $crate::send_to($stream.as_ref(), &$pkt_loot) {
            eprintln!("Failed to send loot packet: {}", e);
        }
    };
}

impl std::fmt::Display for PktLoot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            serde_json::to_string(self).unwrap_or_else(|_| "Failed to serialize Loot".to_string())
        )
    }
}

impl Parser<'_> for PktLoot {
    fn write_to<W: Write>(&self, writer: &mut W) -> Result<(), std::io::Error> {
        // Package into a byte array
        let mut packet: Vec<u8> = vec![self.packet_type.into()];

        let mut target_name_bytes = self.target_name.as_bytes().to_vec();
        target_name_bytes.resize(32, 0x00); // Pad the name to 32 bytes
        packet.extend(target_name_bytes);

        // Write the packet to the buffer
        writer
            .write_all(&packet)
            .map_err(|_| std::io::Error::other("Failed to write packet to buffer"))?;

        Ok(())
    }

    fn decode(packet: Packet) -> Self {
        let message_type = packet.packet_type;
        let target_name = String::from_utf8_lossy(&packet.body[0..32])
            .split('\0')
            .take(1)
            .collect();

        Self {
            packet_type: message_type,
            target_name,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::test_common;

    use super::*;

    #[test]
    fn loot_parse_and_serialize() {
        let stream = test_common::setup();
        let type_byte = PktType::LOOT;
        let original_bytes: &[u8; 33] = &[
            0x05, 0x54, 0x65, 0x73, 0x74, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00,
        ];

        // Create a packet with known bytes, excluding the type byte
        let packet = Packet::new(&stream, type_byte, &original_bytes[1..]);

        // Deserialize the packet into a PktLoot
        let message = PktLoot::decode(packet);

        // Assert the fields were parsed correctly
        assert_eq!(message.packet_type, PktType::LOOT);
        assert_eq!(message.target_name.as_ref(), "Test");

        // Serialize the message back into bytes
        let mut buffer: Vec<u8> = Vec::new();
        message.write_to(&mut buffer).expect("Encoding failed");

        // Assert that the serialized bytes match the original
        assert_eq!(buffer, original_bytes);
        assert_eq!(buffer[0], u8::from(type_byte));
    }

    /// Parse from trace: loot target "Deku Baba".
    #[test]
    fn loot_parse_trace_deku_baba() {
        let stream = test_common::setup();
        let mut body: Vec<u8> = Vec::new();
        let mut name = b"Deku Baba".to_vec();
        name.resize(32, 0x00);
        body.extend(&name);

        let packet = Packet::new(&stream, PktType::LOOT, &body);
        let loot = PktLoot::decode(packet);

        assert_eq!(loot.target_name.as_ref(), "Deku Baba");
    }

    /// PktLoot::loot helper constructs correctly.
    #[test]
    fn loot_helper() {
        let loot = PktLoot::loot("Monster");
        assert_eq!(loot.packet_type, PktType::LOOT);
        assert_eq!(loot.target_name.as_ref(), "Monster");
    }

    /// Empty target name.
    #[test]
    fn loot_empty_name() {
        let stream = test_common::setup();
        let body: Vec<u8> = vec![0x00; 32];
        let packet = Packet::new(&stream, PktType::LOOT, &body);
        let loot = PktLoot::decode(packet);

        assert_eq!(loot.target_name.as_ref(), "");
    }

    /// Max-length target name (32 bytes).
    #[test]
    fn loot_max_length_name() {
        let stream = test_common::setup();
        let long_name = "M".repeat(32);
        let body: Vec<u8> = long_name.as_bytes().to_vec();
        let packet = Packet::new(&stream, PktType::LOOT, &body);
        let loot = PktLoot::decode(packet);

        assert_eq!(loot.target_name.as_ref(), &long_name);
    }

    /// Roundtrip.
    #[test]
    fn loot_roundtrip() {
        let stream = test_common::setup();
        let original = PktLoot::loot("DragonBoss");

        let mut buffer: Vec<u8> = Vec::new();
        original.write_to(&mut buffer).expect("Encoding failed");

        assert_eq!(buffer.len(), 33); // type(1) + name(32)

        let packet = Packet::new(&stream, PktType::LOOT, &buffer[1..]);
        let deserialized = PktLoot::decode(packet);
        assert_eq!(deserialized.target_name.as_ref(), "DragonBoss");
    }

    /// Non-UTF8 name.
    #[test]
    fn loot_non_utf8_name() {
        let stream = test_common::setup();
        let mut body = vec![0xFF, 0xFE, 0xFD];
        body.resize(32, 0x00);
        let packet = Packet::new(&stream, PktType::LOOT, &body);
        let loot = PktLoot::decode(packet);

        assert!(loot.target_name.contains('\u{FFFD}'));
    }

    /// Body too short should panic.
    #[test]
    #[should_panic]
    fn loot_body_too_short_panics() {
        let stream = test_common::setup();
        let body: &[u8] = &[0x41, 0x42]; // Only 2 bytes, need 32
        let packet = Packet::new(&stream, PktType::LOOT, body);
        let _ = PktLoot::decode(packet);
    }

    /// Empty body should panic.
    #[test]
    #[should_panic]
    fn loot_empty_body_panics() {
        let stream = test_common::setup();
        let body: &[u8] = &[];
        let packet = Packet::new(&stream, PktType::LOOT, body);
        let _ = PktLoot::decode(packet);
    }

    /// All 0xFF body.
    #[test]
    fn loot_all_ones_body() {
        let stream = test_common::setup();
        let body: Vec<u8> = vec![0xFF; 32];
        let packet = Packet::new(&stream, PktType::LOOT, &body);
        let loot = PktLoot::decode(packet);

        assert!(!loot.target_name.is_empty());
    }

    /// Display/JSON output should be valid JSON.
    #[test]
    fn loot_display_valid_json() {
        let loot = PktLoot::loot("Goblin");
        let json_str = format!("{}", loot);
        let parsed: serde_json::Value = serde_json::from_str(&json_str).expect("Invalid JSON");
        assert_eq!(parsed["target_name"], "Goblin");
    }
}
////////////////////////////////////////////////////////////////////////////////
