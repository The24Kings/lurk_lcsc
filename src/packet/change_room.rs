use serde::{Deserialize, Serialize};
use std::io::Write;

use crate::packet::PktType;
use crate::{Packet, Parser};

/// Sent by the client only, to change rooms.
///
/// If the server changes the room a client is in, it should send an updated room, character, and connection message(s) to explain the new location.
/// If not, for example because the client is not ready to start or specified an inappropriate choice, and error should be sent.
#[derive(Serialize, Deserialize)]
pub struct PktChangeRoom {
    /// The type of message for the `CHANGEROOM` packet. Default is 2.
    pub packet_type: PktType,
    /// Number of the room to change to. The server will send an error if an inappropriate choice is made.
    pub room_number: u16,
}

impl From<u16> for PktChangeRoom {
    // Return PktChangeRoom from provided room number
    fn from(room_number: u16) -> Self {
        Self {
            packet_type: PktType::CHANGEROOM,
            room_number,
        }
    }
}

impl From<PktChangeRoom> for u16 {
    /// Return room number from PktChangeRoom
    fn from(packet: PktChangeRoom) -> Self {
        packet.room_number
    }
}

#[macro_export]
/// Send `PktChangeRoom` over `TcpStream` to connected user
///
/// ```no_run
/// use lurk_protocol::{Protocol, PktChangeRoom, LurkError, send_change_room};
/// use std::sync::Arc;
/// use std::net::TcpStream;
///
/// let stream = Arc::new(TcpStream::connect("127.0.0.1:8080").unwrap());
///
/// send_change_room!(stream.clone(), PktChangeRoom::from(0))
/// ```
macro_rules! send_change_room {
    ($stream:expr, $pkt_chg_rm:expr) => {
        if let Err(e) = $crate::send_to($stream.as_ref(), &$pkt_chg_rm) {
            eprintln!("Failed to send change room packet: {}", e);
        }
    };
}

impl std::fmt::Display for PktChangeRoom {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            serde_json::to_string(self)
                .unwrap_or_else(|_| "Failed to serialize ChangeRoom".to_string())
        )
    }
}

impl Parser<'_> for PktChangeRoom {
    fn write_to<W: Write>(&self, writer: &mut W) -> Result<(), std::io::Error> {
        // Package into a byte array
        let mut packet: Vec<u8> = vec![self.packet_type.into()];

        packet.extend(self.room_number.to_le_bytes());

        // Write the packet to the buffer
        writer
            .write_all(&packet)
            .map_err(|_| std::io::Error::other("Failed to write packet to buffer"))?;

        Ok(())
    }

    fn decode(packet: Packet) -> Self {
        let room_number = u16::from_le_bytes([packet.body[0], packet.body[1]]);

        // Implement deserialization logic here
        Self {
            packet_type: packet.packet_type,
            room_number,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn changeroom_parse_and_serialize() {
        let type_byte = PktType::CHANGEROOM;
        let original_bytes: &[u8; 3] = &[0x02, 0x00, 0x00];

        // Create a packet with known bytes, excluding the type byte
        let packet = Packet::new(type_byte, &original_bytes[1..]);

        // Deserialize the packet into a PktChangeRoom
        let message = PktChangeRoom::decode(packet);

        // Assert the fields were parsed correctly
        assert_eq!(message.packet_type, PktType::CHANGEROOM);
        assert_eq!(message.room_number, 0);

        // Serialize the message back into bytes
        let mut buffer: Vec<u8> = Vec::new();
        message.write_to(&mut buffer).expect("Encoding failed");

        // Assert that the serialized bytes match the original
        assert_eq!(buffer, original_bytes);
        assert_eq!(buffer[0], u8::from(type_byte));
    }

    /// From<u16> constructor.
    #[test]
    fn changeroom_from_u16() {
        let cr = PktChangeRoom::from(42u16);
        assert_eq!(cr.packet_type, PktType::CHANGEROOM);
        assert_eq!(cr.room_number, 42);
    }

    /// Into<u16> conversion.
    #[test]
    fn changeroom_into_u16() {
        let cr = PktChangeRoom::from(99u16);
        let num: u16 = cr.into();
        assert_eq!(num, 99);
    }

    /// Room number 0.
    #[test]
    fn changeroom_room_zero() {
        let body: &[u8] = &[0x00, 0x00];
        let packet = Packet::new(PktType::CHANGEROOM, body);
        let cr = PktChangeRoom::decode(packet);
        assert_eq!(cr.room_number, 0);
    }

    /// Room number 1.
    #[test]
    fn changeroom_room_one() {
        let body: &[u8] = &[0x01, 0x00];
        let packet = Packet::new(PktType::CHANGEROOM, body);
        let cr = PktChangeRoom::decode(packet);
        assert_eq!(cr.room_number, 1);
    }

    /// Max room number.
    #[test]
    fn changeroom_max_room() {
        let body: &[u8] = &[0xFF, 0xFF];
        let packet = Packet::new(PktType::CHANGEROOM, body);
        let cr = PktChangeRoom::decode(packet);
        assert_eq!(cr.room_number, u16::MAX);
    }

    /// Boundary room values roundtrip.
    #[test]
    fn changeroom_boundary_values() {
        for &room in &[0u16, 1, 255, 256, 1000, u16::MAX - 1, u16::MAX] {
            let cr = PktChangeRoom::from(room);
            let mut buffer: Vec<u8> = Vec::new();
            cr.write_to(&mut buffer).expect("Encoding failed");

            let packet = Packet::new(PktType::CHANGEROOM, &buffer[1..]);
            let deserialized = PktChangeRoom::decode(packet);
            assert_eq!(deserialized.room_number, room, "Failed for room: {}", room);
        }
    }

    /// Serialized output is exactly 3 bytes.
    #[test]
    fn changeroom_serialize_length() {
        let cr = PktChangeRoom::from(0u16);
        let mut buffer: Vec<u8> = Vec::new();
        cr.write_to(&mut buffer).expect("Encoding failed");
        assert_eq!(buffer.len(), 3);
    }

    /// Body too short should panic.
    #[test]
    #[should_panic]
    fn changeroom_body_too_short_panics() {
        let body: &[u8] = &[0x00]; // Need 2 bytes
        let packet = Packet::new(PktType::CHANGEROOM, body);
        let _ = PktChangeRoom::decode(packet);
    }

    /// Empty body should panic.
    #[test]
    #[should_panic]
    fn changeroom_empty_body_panics() {
        let body: &[u8] = &[];
        let packet = Packet::new(PktType::CHANGEROOM, body);
        let _ = PktChangeRoom::decode(packet);
    }

    /// Extra trailing bytes should be ignored.
    #[test]
    fn changeroom_extra_trailing_bytes() {
        let body: &[u8] = &[0x05, 0x00, 0xFF, 0xFF];
        let packet = Packet::new(PktType::CHANGEROOM, body);
        let cr = PktChangeRoom::decode(packet);
        assert_eq!(cr.room_number, 5);
    }

    /// Display/JSON output should be valid JSON.
    #[test]
    fn changeroom_display_valid_json() {
        let cr = PktChangeRoom::from(42u16);
        let json_str = format!("{}", cr);
        let parsed: serde_json::Value = serde_json::from_str(&json_str).expect("Invalid JSON");
        assert_eq!(parsed["room_number"], 42);
    }
}
////////////////////////////////////////////////////////////////////////////////
