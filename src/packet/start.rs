use serde::{Deserialize, Serialize};
use std::io::Write;

use crate::packet::PktType;
use crate::{Packet, Parser};

#[derive(Serialize, Deserialize)]
/// Start playing the game.
///
/// - A client will send a `PktType::CHARACTER` message to the server to explain character stats, which the server may either accept or deny (by use of an `PktType::ERROR` message).
/// - If the stats are accepted, the server will not enter the player into the game world until it has received `PktType::START`.
/// - This is sent by the client.
/// - Generally, the server will reply with a `PktType::ROOM`, a `PktType::CHARACTER` message showing the updated room, and a `PktType::CHARACTER` message for each player in the initial room of the game.
pub struct PktStart {
    /// The type of message for the `START` packet. Defaults to 6.
    pub packet_type: PktType,
}

impl Default for PktStart {
    fn default() -> Self {
        Self {
            packet_type: PktType::START,
        }
    }
}

#[macro_export]
/// Send `PktStart` over `TcpStream` to connected user
///
/// ```no_run
/// use lurk_lcsc::{Protocol, PktStart, LurkError, send_start};
/// use std::sync::Arc;
/// use std::net::TcpStream;
///
/// let stream = Arc::new(TcpStream::connect("127.0.0.1:8080").unwrap());
///
/// send_start!(stream.clone(), PktStart::default())
/// ```
macro_rules! send_start {
    ($stream:expr, $pkt_start:expr) => {
        if let Err(e) = $crate::send_to($stream.as_ref(), &$pkt_start) {
            eprintln!("Failed to send start packet: {}", e);
        }
    };
}

impl std::fmt::Display for PktStart {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            serde_json::to_string(self).unwrap_or_else(|_| "Failed to serialize Start".to_string())
        )
    }
}

impl Parser<'_> for PktStart {
    fn write_to<W: Write>(&self, writer: &mut W) -> Result<(), std::io::Error> {
        // Package into a byte array
        let packet: Vec<u8> = vec![self.packet_type.into()];

        // Write the packet to the buffer
        writer
            .write_all(&packet)
            .map_err(|_| std::io::Error::other("Failed to write packet to buffer"))?;

        Ok(())
    }

    fn decode(packet: Packet) -> Self {
        Self {
            packet_type: packet.packet_type,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::test_common;

    use super::*;

    #[test]
    fn start_parse_and_serialize() {
        let stream = test_common::setup();
        let type_byte = PktType::START;
        let original_bytes: &[u8; 1] = &[0x06];
        // Create a packet with known bytes, excluding the type byte
        let packet = Packet::new(&stream, type_byte, &original_bytes[1..]);

        // Deserialize the packet into a PktStart
        let message = PktStart::decode(packet);

        // Assert the fields were parsed correctly
        assert_eq!(message.packet_type, PktType::START);

        // Serialize the message back into bytes
        let mut buffer: Vec<u8> = Vec::new();
        message.write_to(&mut buffer).expect("Encoding failed");

        // Assert that the serialized bytes match the original
        assert_eq!(buffer, original_bytes);
        assert_eq!(buffer[0], u8::from(type_byte));
    }

    /// Default constructor should set correct packet type.
    #[test]
    fn start_default() {
        let start = PktStart::default();
        assert_eq!(start.packet_type, PktType::START);
    }

    /// Serialized output is exactly 1 byte.
    #[test]
    fn start_serialize_length() {
        let start = PktStart::default();
        let mut buffer: Vec<u8> = Vec::new();
        start.write_to(&mut buffer).expect("Encoding failed");
        assert_eq!(buffer.len(), 1);
        assert_eq!(buffer[0], 0x06);
    }

    /// Roundtrip: serialize then deserialize.
    #[test]
    fn start_roundtrip() {
        let stream = test_common::setup();
        let original = PktStart::default();

        let mut buffer: Vec<u8> = Vec::new();
        original.write_to(&mut buffer).expect("Encoding failed");

        let packet = Packet::new(&stream, PktType::START, &[]);
        let deserialized = PktStart::decode(packet);
        assert_eq!(deserialized.packet_type, PktType::START);
    }

    /// Deserialize with extra body bytes should still work.
    #[test]
    fn start_extra_body_bytes() {
        let stream = test_common::setup();
        let body: &[u8] = &[0xFF, 0xFF];
        let packet = Packet::new(&stream, PktType::START, body);
        let start = PktStart::decode(packet);
        assert_eq!(start.packet_type, PktType::START);
    }

    /// Display/JSON output should be valid JSON.
    #[test]
    fn start_display_valid_json() {
        let start = PktStart::default();
        let json_str = format!("{}", start);
        let parsed: serde_json::Value = serde_json::from_str(&json_str).expect("Invalid JSON");
        assert_eq!(parsed["packet_type"], "START");
    }
}
////////////////////////////////////////////////////////////////////////////////
