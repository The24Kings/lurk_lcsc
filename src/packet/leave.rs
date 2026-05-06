use serde::{Deserialize, Serialize};
use std::io::Write;

use crate::packet::PktType;
use crate::{Packet, Parser};

#[derive(Serialize, Deserialize)]
/// Used by the client to leave the game. This is a graceful way to disconnect. The server never terminates, so it doesn't send `PktType::LEAVE`.
pub struct PktLeave {
    /// The type of message for the `LEAVE` packet. Defaults to 12.
    pub packet_type: PktType,
}

impl Default for PktLeave {
    fn default() -> Self {
        Self {
            packet_type: PktType::LEAVE,
        }
    }
}

#[macro_export]
/// Send `PktLeave` over `TcpStream` to connected user
///
/// ```no_run
/// use lurk_lcsc::{Protocol, PktLeave, LurkError, send_leave};
/// use std::sync::Arc;
/// use std::net::TcpStream;
///
/// let stream = Arc::new(TcpStream::connect("127.0.0.1:8080").unwrap());
///
/// send_leave!(stream.clone(), PktLeave::default())
/// ```
macro_rules! send_leave {
    ($stream:expr, $pkt_leave:expr) => {
        if let Err(e) = $crate::send_to($stream.as_ref(), &$pkt_leave) {
            eprintln!("Failed to send leave packet: {}", e);
        }
    };
}

impl std::fmt::Display for PktLeave {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            serde_json::to_string(self).unwrap_or_else(|_| "Failed to serialize Leave".to_string())
        )
    }
}

impl Parser<'_> for PktLeave {
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
    use super::*;

    #[test]
    fn leave_parse_and_serialize() {
        let type_byte = PktType::LEAVE;
        let original_bytes: &[u8; 1] = &[0x0c];

        // Create a packet with known bytes, excluding the type byte
        let packet = Packet::new(type_byte, &[]);

        // Deserialize the packet into a PktLeave
        let message = PktLeave::decode(packet);

        // Assert the fields were parsed correctly
        assert_eq!(message.packet_type, PktType::LEAVE);

        // Serialize the message back into bytes
        let mut buffer: Vec<u8> = Vec::new();
        message.write_to(&mut buffer).expect("Encoding failed");

        // Assert that the serialized bytes match the original
        assert_eq!(buffer, original_bytes);
        assert_eq!(buffer[0], u8::from(type_byte));
    }

    /// Default constructor should set correct packet type.
    #[test]
    fn leave_default() {
        let leave = PktLeave::default();
        assert_eq!(leave.packet_type, PktType::LEAVE);
    }

    /// Serialized output is exactly 1 byte.
    #[test]
    fn leave_serialize_length() {
        let leave = PktLeave::default();
        let mut buffer: Vec<u8> = Vec::new();
        leave.write_to(&mut buffer).expect("Encoding failed");
        assert_eq!(buffer.len(), 1);
        assert_eq!(buffer[0], 0x0c);
    }

    /// Roundtrip: serialize then deserialize.
    #[test]
    fn leave_roundtrip() {
        let original = PktLeave::default();

        let mut buffer: Vec<u8> = Vec::new();
        original.write_to(&mut buffer).expect("Encoding failed");

        let packet = Packet::new(PktType::LEAVE, &[]);
        let deserialized = PktLeave::decode(packet);
        assert_eq!(deserialized.packet_type, PktType::LEAVE);
    }

    /// Deserialize with extra body bytes should still work.
    #[test]
    fn leave_extra_body_bytes() {
        let body: &[u8] = &[0xFF, 0xFF];
        let packet = Packet::new(PktType::LEAVE, body);
        let leave = PktLeave::decode(packet);
        assert_eq!(leave.packet_type, PktType::LEAVE);
    }

    /// Display/JSON output should be valid JSON.
    #[test]
    fn leave_display_valid_json() {
        let leave = PktLeave::default();
        let json_str = format!("{}", leave);
        let parsed: serde_json::Value = serde_json::from_str(&json_str).expect("Invalid JSON");
        assert_eq!(parsed["packet_type"], "LEAVE");
    }

    /// Decode must use the packet_type from the Packet, not Default.
    #[test]
    fn leave_decode_uses_packet_type() {
        // Pass a non-LEAVE type to verify decode reads from the packet
        let packet = Packet::new(PktType::DEFAULT, &[]);
        let leave = PktLeave::decode(packet);
        assert_eq!(leave.packet_type, PktType::DEFAULT);
    }
}
////////////////////////////////////////////////////////////////////////////////
