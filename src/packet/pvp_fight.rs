use serde::{Deserialize, Serialize};
use std::io::Write;

use crate::packet::PktType;
use crate::{Packet, Parser};

#[derive(Serialize, Deserialize)]
/// Initiate a fight against another player.
///
/// - The server will determine the results of the fight, and allocate damage and rewards appropriately.
/// - The server may include players with join battle in the fight, on either side.
/// - Monsters may or may not be involved in the fight as well.
/// - This message is sent by the client.
///
/// If the server does not support PVP, it should send `LurkError::NOPLAYERCOMBAT` to the client.
pub struct PktPVPFight {
    /// The type of message for the `PVPFIGHT` packet. Defaults to 4.
    pub packet_type: PktType,
    /// The name of the target player, up to 32 bytes.
    pub target_name: Box<str>,
}

impl PktPVPFight {
    /// Create a new PktPVPFight packet from a given name
    pub fn fight(name: &str) -> Self {
        Self {
            packet_type: PktType::PVPFIGHT,
            target_name: Box::from(name),
        }
    }
}

#[macro_export]
/// Send `PktPVPFight` over `TcpStream` to connected user
///
/// ```no_run
/// use lurk_lcsc::{Protocol, PktPVPFight, LurkError, send_pvp};
/// use std::sync::Arc;
/// use std::net::TcpStream;
///
/// let stream = Arc::new(TcpStream::connect("127.0.0.1:8080").unwrap());
///
/// send_pvp!(stream.clone(), PktPVPFight::fight("Test"))
/// ```
macro_rules! send_pvp {
    ($stream:expr, $pkt_pvp:expr) => {
        if let Err(e) = $crate::send_to($stream.as_ref(), &$pkt_pvp) {
            eprintln!("Failed to send pvp fight packet: {}", e);
        }
    };
}

impl std::fmt::Display for PktPVPFight {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            serde_json::to_string(self)
                .unwrap_or_else(|_| "Failed to serialize PVPFight".to_string())
        )
    }
}

impl Parser<'_> for PktPVPFight {
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
    use super::*;

    #[test]
    fn pvp_fight_parse_and_serialize() {
        let type_byte = PktType::PVPFIGHT;
        let original_bytes: &[u8; 33] = &[
            0x04, 0x54, 0x65, 0x73, 0x74, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00,
        ];

        // Create a packet with known bytes, excluding the type byte
        let packet = Packet::new(type_byte, &original_bytes[1..]);

        // Deserialize the packet into a PktPVPFight
        let message = PktPVPFight::decode(packet);

        // Assert the fields were parsed correctly
        assert_eq!(message.packet_type, PktType::PVPFIGHT);
        assert_eq!(message.target_name.as_ref(), "Test");

        // Serialize the message back into bytes
        let mut buffer: Vec<u8> = Vec::new();
        message.write_to(&mut buffer).expect("Encoding failed");

        // Assert that the serialized bytes match the original
        assert_eq!(buffer, original_bytes);
        assert_eq!(buffer[0], u8::from(type_byte));
    }

    /// PktPVPFight::fight helper constructs correctly.
    #[test]
    fn pvp_fight_helper() {
        let pvp = PktPVPFight::fight("EnemyPlayer");
        assert_eq!(pvp.packet_type, PktType::PVPFIGHT);
        assert_eq!(pvp.target_name.as_ref(), "EnemyPlayer");
    }

    /// Empty target name.
    #[test]
    fn pvp_fight_empty_name() {
        let body: Vec<u8> = vec![0x00; 32];
        let packet = Packet::new(PktType::PVPFIGHT, &body);
        let pvp = PktPVPFight::decode(packet);

        assert_eq!(pvp.target_name.as_ref(), "");
    }

    /// Max-length target name (32 bytes).
    #[test]
    fn pvp_fight_max_length_name() {
        let long_name = "P".repeat(32);
        let body: Vec<u8> = long_name.as_bytes().to_vec();
        let packet = Packet::new(PktType::PVPFIGHT, &body);
        let pvp = PktPVPFight::decode(packet);

        assert_eq!(pvp.target_name.as_ref(), &long_name);
    }

    /// Roundtrip.
    #[test]
    fn pvp_fight_roundtrip() {
        let original = PktPVPFight::fight("Rival");

        let mut buffer: Vec<u8> = Vec::new();
        original.write_to(&mut buffer).expect("Encoding failed");

        assert_eq!(buffer.len(), 33); // type(1) + name(32)

        let packet = Packet::new(PktType::PVPFIGHT, &buffer[1..]);
        let deserialized = PktPVPFight::decode(packet);
        assert_eq!(deserialized.target_name.as_ref(), "Rival");
    }

    /// Non-UTF8 name.
    #[test]
    fn pvp_fight_non_utf8_name() {
        let mut body = vec![0xFF, 0xFE, 0xFD];
        body.resize(32, 0x00);
        let packet = Packet::new(PktType::PVPFIGHT, &body);
        let pvp = PktPVPFight::decode(packet);

        assert!(pvp.target_name.contains('\u{FFFD}'));
    }

    /// Body too short should panic.
    #[test]
    #[should_panic]
    fn pvp_fight_body_too_short_panics() {
        let body: &[u8] = &[0x41, 0x42]; // Only 2 bytes, need 32
        let packet = Packet::new(PktType::PVPFIGHT, body);
        let _ = PktPVPFight::decode(packet);
    }

    /// Empty body should panic.
    #[test]
    #[should_panic]
    fn pvp_fight_empty_body_panics() {
        let body: &[u8] = &[];
        let packet = Packet::new(PktType::PVPFIGHT, body);
        let _ = PktPVPFight::decode(packet);
    }

    /// All 0xFF body.
    #[test]
    fn pvp_fight_all_ones_body() {
        let body: Vec<u8> = vec![0xFF; 32];
        let packet = Packet::new(PktType::PVPFIGHT, &body);
        let pvp = PktPVPFight::decode(packet);

        assert!(!pvp.target_name.is_empty());
    }

    /// Display/JSON output should be valid JSON.
    #[test]
    fn pvp_fight_display_valid_json() {
        let pvp = PktPVPFight::fight("Enemy");
        let json_str = format!("{}", pvp);
        let parsed: serde_json::Value = serde_json::from_str(&json_str).expect("Invalid JSON");
        assert_eq!(parsed["target_name"], "Enemy");
    }
}
////////////////////////////////////////////////////////////////////////////////
