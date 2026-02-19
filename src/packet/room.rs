use serde::{Deserialize, Serialize};
use std::io::Write;

use crate::packet::PktType;
use crate::{Packet, Parser};

#[derive(Serialize, Deserialize)]
/// Sent by the server to describe the room that the player is in.
///
/// - This should be an expected response to `PktType::CHANGEROOM` or `PktType::START`.
/// - Can be re-sent at any time, for example if the player is teleported or falls through a floor.
/// - Outgoing connections will be specified with a series of `PktType::CONNECTION` messages.
/// - Monsters and players in the room should be listed using a series of `PktType::CHARACTER` messages.
pub struct PktRoom {
    /// The type of message for the `ROOM` packet. Defaults to 9
    pub packet_type: PktType,
    /// The room number the player is currently in. This is the same as the room number used in `PktType::CHANGEROOM`.
    pub room_number: u16,
    /// The name of the room, up to 32 bytes.
    pub room_name: Box<str>,
    /// The length of the room description.
    pub description_len: u16,
    /// The room description.
    pub description: Box<str>,
}

#[macro_export]
/// Send `PktRoom` over `TcpStream` to connected user
///
/// ```no_run
/// use lurk_lcsc::{Protocol, PktRoom, PktType, send_room};
/// use std::sync::Arc;
/// use std::net::TcpStream;
///
/// let stream = Arc::new(TcpStream::connect("127.0.0.1:8080").unwrap());
/// let room = PktRoom {
///     packet_type: PktType::ROOM,
///     room_number: 0,
///     room_name: "Test".into(),
///     description_len: 0,
///     description: "".into(),
/// };
///
/// send_room!(stream.clone(), room)
/// ```
macro_rules! send_room {
    ($stream:expr, $room:expr) => {
        if let Err(e) = $crate::Protocol::Room($stream, $room).send() {
            eprintln!("Failed to send room packet: {}", e);
        }
    };
}

impl std::fmt::Display for PktRoom {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            serde_json::to_string(self).unwrap_or_else(|_| "Failed to serialize Room".to_string())
        )
    }
}

impl Parser<'_> for PktRoom {
    fn serialize<W: Write>(self, writer: &mut W) -> Result<(), std::io::Error> {
        // Package into a byte array
        let mut packet: Vec<u8> = vec![self.packet_type.into()];

        packet.extend(self.room_number.to_le_bytes());

        let mut room_name_bytes = self.room_name.as_bytes().to_vec();
        room_name_bytes.resize(32, 0); // Pad with zeros to 32 bytes
        packet.extend(room_name_bytes);

        packet.extend(self.description_len.to_le_bytes());
        packet.extend(self.description.as_bytes());

        // Write the packet to the buffer
        writer
            .write_all(&packet)
            .map_err(|_| std::io::Error::other("Failed to write packet to buffer"))?;

        Ok(())
    }

    fn deserialize(packet: Packet) -> Self {
        let message_type = packet.packet_type;
        let room_number = u16::from_le_bytes([packet.body[0], packet.body[1]]);
        let room_name = String::from_utf8_lossy(&packet.body[2..34])
            .split('\0')
            .take(1)
            .collect();
        let description_len = u16::from_le_bytes([packet.body[34], packet.body[35]]);
        let description = String::from_utf8_lossy(&packet.body[36..]).into();

        Self {
            packet_type: message_type,
            room_number,
            room_name,
            description_len,
            description,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::test_common;

    use super::*;

    #[test]
    fn room_parse_and_serialize() {
        let stream = test_common::setup();
        let type_byte = PktType::ROOM;
        let original_bytes: &[u8; 69] = &[
            0x09, 0x00, 0x00, 0x54, 0x65, 0x73, 0x74, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x20, 0x00, 0x41, 0x75, 0x74, 0x6f, 0x2d,
            0x67, 0x65, 0x6e, 0x65, 0x72, 0x61, 0x74, 0x65, 0x64, 0x20, 0x72, 0x6f, 0x6f, 0x6d,
            0x20, 0x64, 0x65, 0x73, 0x63, 0x72, 0x69, 0x70, 0x74, 0x69, 0x6f, 0x6e, 0x2e,
        ];

        // Create a packet with known bytes, excluding the type byte
        let packet = Packet::new(&stream, type_byte, &original_bytes[1..]);

        // Deserialize the packet into a PktRoom
        let message = <PktRoom as Parser>::deserialize(packet);

        // Assert the fields were parsed correctly
        assert_eq!(message.packet_type, PktType::ROOM);
        assert_eq!(message.room_number, 0);
        assert_eq!(message.room_name.as_ref(), "Test");
        assert_eq!(message.description_len, 32);
        assert_eq!(
            message.description.as_ref(),
            "Auto-generated room description."
        );

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
