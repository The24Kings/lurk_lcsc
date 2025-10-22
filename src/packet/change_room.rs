use serde::Serialize;
use std::io::Write;

use crate::packet::PktType;
use crate::{Packet, Parser};

/// Sent by the client only, to change rooms.
///
/// If the server changes the room a client is in, it should send an updated room, character, and connection message(s) to explain the new location.
/// If not, for example because the client is not ready to start or specified an inappropriate choice, and error should be sent.
#[derive(Serialize)]
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
/// use lurk_lcsc::{Protocol, PktChangeRoom, LurkError, send_change_room};
/// use std::sync::Arc;
/// use std::net::TcpStream;
///
/// let stream = Arc::new(TcpStream::connect("127.0.0.1:8080").unwrap());
///
/// send_change_room!(stream.clone(), PktChangeRoom::from(0))
/// ```
macro_rules! send_change_room {
    ($stream:expr, $pkt_chg_rm:expr) => {
        if let Err(e) = $crate::Protocol::ChangeRoom($stream, $pkt_chg_rm).send() {
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
    fn serialize<W: Write>(self, writer: &mut W) -> Result<(), std::io::Error> {
        // Package into a byte array
        let mut packet: Vec<u8> = vec![self.packet_type.into()];

        packet.extend(self.room_number.to_le_bytes());

        // Write the packet to the buffer
        writer
            .write_all(&packet)
            .map_err(|_| std::io::Error::other("Failed to write packet to buffer"))?;

        Ok(())
    }

    fn deserialize(packet: Packet) -> Self {
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
    use crate::test_common;

    use super::*;

    #[test]
    fn changeroom_parse_and_serialize() {
        let stream = test_common::setup();
        let type_byte = PktType::CHANGEROOM;
        let original_bytes: &[u8; 3] = &[0x02, 0x00, 0x00];

        // Create a packet with known bytes, excluding the type byte
        let packet = Packet::new(&stream, type_byte, &original_bytes[1..]);

        // Deserialize the packet into a PktChangeRoom
        let message = PktChangeRoom::deserialize(packet);

        // Assert the fields were parsed correctly
        assert_eq!(message.packet_type, PktType::CHANGEROOM);
        assert_eq!(message.room_number, 0);

        // Serialize the message back into bytes
        let mut buffer: Vec<u8> = Vec::new();
        message
            .serialize(&mut buffer)
            .expect("Serialization failed");

        // Assert that the serialized bytes match the original
        assert_eq!(buffer, original_bytes);
        assert_eq!(buffer[0], u8::from(type_byte));
    }
}
////////////////////////////////////////////////////////////////////////////////
