use serde::{Deserialize, Serialize};
use std::io::Write;

use crate::packet::PktType;
use crate::{Packet, Parser};

#[derive(Serialize, Deserialize)]
/// Used by the server to describe rooms connected to the room the player is in.
///
/// - The client should expect a series of these when changing rooms, but they may be sent at any time.
///   - For example; After a fight, a secret staircase may extend out of the ceiling enabling another connection.
/// - Note that the room description may be an abbreviated version of the description sent when a room is actually entered.
/// - The server may also provide a different room description depending on which room the player is in.
///
/// So a description on the connection could read `A strange whirr is heard through the solid oak door`,
/// and the description attached to the message once the player has entered could read
/// `Servers line the walls, softly lighting the room in a cacophony of red, green, blue, and yellow flashes`.
pub struct PktConnection {
    /// The type of message for the `CONNECTION` packet. Defaults to 13.
    pub packet_type: PktType,
    /// Room number. This is the same room number used for `PktType::CHANGEROOM`
    pub room_number: u16,
    /// The name of the room this connection leads to, up to 32 bytes.
    pub room_name: Box<str>,
    /// The length of the room description.
    pub description_len: u16,
    /// The description of the room this connection leads to.
    pub description: Box<str>,
}

#[macro_export]
/// Send `PktConnection` over `TcpStream` to connected user
///
/// ```no_run
/// use lurk_protocol::{Protocol, PktConnection, PktType, send_connection};
/// use std::sync::Arc;
/// use std::net::TcpStream;
///
/// let stream = Arc::new(TcpStream::connect("127.0.0.1:8080").unwrap());
/// let connection = PktConnection {
///     packet_type: PktType::CONNECTION,
///     room_number: 0,
///     room_name: "Test".into(),
///     description_len: 0,
///     description: "".into(),
/// };
///
/// send_connection!(stream.clone(), connection)
/// ```
macro_rules! send_connection {
    ($stream:expr, $connection:expr) => {
        if let Err(e) = $crate::send_to($stream.as_ref(), &$connection) {
            eprintln!("Failed to send connection packet: {}", e);
        }
    };
}

impl std::fmt::Display for PktConnection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            serde_json::to_string(self)
                .unwrap_or_else(|_| "Failed to serialize Connection".to_string())
        )
    }
}

impl Parser<'_> for PktConnection {
    fn write_to<W: Write>(&self, writer: &mut W) -> Result<(), std::io::Error> {
        // Package into a byte array
        let mut packet: Vec<u8> = vec![self.packet_type.into()];

        packet.extend(self.room_number.to_le_bytes());

        let mut room_name_bytes = self.room_name.as_bytes().to_vec();
        room_name_bytes.resize(32, 0x00); // Pad the name to 32 bytes
        packet.extend(room_name_bytes);

        packet.extend(self.description_len.to_le_bytes());
        packet.extend(self.description.as_bytes());

        // Write the packet to the buffer
        writer
            .write_all(&packet)
            .map_err(|_| std::io::Error::other("Failed to write packet to buffer"))?;

        Ok(())
    }

    fn decode(packet: Packet) -> Self {
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
    use super::*;

    #[test]
    fn connection_parse_and_serialize() {
        let type_byte = PktType::CONNECTION;
        let original_bytes: &[u8; 75] = &[
            0x0d, 0x01, 0x00, 0x54, 0x65, 0x73, 0x74, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x26, 0x00, 0x41, 0x75, 0x74, 0x6f, 0x2d,
            0x67, 0x65, 0x6e, 0x65, 0x72, 0x61, 0x74, 0x65, 0x64, 0x20, 0x63, 0x6f, 0x6e, 0x6e,
            0x65, 0x63, 0x74, 0x69, 0x6f, 0x6e, 0x20, 0x64, 0x65, 0x73, 0x63, 0x72, 0x69, 0x70,
            0x74, 0x69, 0x6f, 0x6e, 0x2e,
        ];

        // Create a packet with known bytes, excluding the type byte
        let packet = Packet::new(type_byte, &original_bytes[1..]);

        // Deserialize the packet into a PktConnection
        let message = PktConnection::decode(packet);

        // Assert the fields were parsed correctly
        assert_eq!(message.packet_type, PktType::CONNECTION);
        assert_eq!(message.room_number, 1);
        assert_eq!(message.room_name.as_ref(), "Test");
        assert_eq!(message.description_len, 38);
        assert_eq!(
            message.description.as_ref(),
            "Auto-generated connection description."
        );

        // Serialize the message back into bytes
        let mut buffer: Vec<u8> = Vec::new();
        message.write_to(&mut buffer).expect("Encoding failed");

        // Assert that the serialized bytes match the original
        assert_eq!(buffer, original_bytes);
        assert_eq!(buffer[0], u8::from(type_byte));
    }

    /// Parse from trace: room 1, "F1 - Main Hall".
    #[test]
    fn connection_parse_trace_main_hall() {
        let room_name = "F1 - Main Hall";
        let desc = "You cannot see what lies beyond the darkness of the Deku Tree's maw, but his pleas for help must not go unanswered!";
        let mut body: Vec<u8> = Vec::new();
        body.extend(1u16.to_le_bytes());
        let mut name_bytes = room_name.as_bytes().to_vec();
        name_bytes.resize(32, 0x00);
        body.extend(&name_bytes);
        body.extend((desc.len() as u16).to_le_bytes());
        body.extend(desc.as_bytes());

        let packet = Packet::new(PktType::CONNECTION, &body);
        let conn = PktConnection::decode(packet);

        assert_eq!(conn.room_number, 1);
        assert_eq!(conn.room_name.as_ref(), room_name);
        assert_eq!(conn.description.as_ref(), desc);
    }

    /// Empty description.
    #[test]
    fn connection_empty_description() {
        let mut body: Vec<u8> = Vec::new();
        body.extend(0u16.to_le_bytes());
        let mut name = b"Empty".to_vec();
        name.resize(32, 0x00);
        body.extend(&name);
        body.extend(0u16.to_le_bytes());

        let packet = Packet::new(PktType::CONNECTION, &body);
        let conn = PktConnection::decode(packet);

        assert_eq!(conn.description_len, 0);
        assert_eq!(conn.description.as_ref(), "");
    }

    /// Max room number.
    #[test]
    fn connection_max_room_number() {
        let mut body: Vec<u8> = Vec::new();
        body.extend(u16::MAX.to_le_bytes());
        let mut name = b"MaxConn".to_vec();
        name.resize(32, 0x00);
        body.extend(&name);
        body.extend(0u16.to_le_bytes());

        let packet = Packet::new(PktType::CONNECTION, &body);
        let conn = PktConnection::decode(packet);

        assert_eq!(conn.room_number, u16::MAX);
    }

    /// Roundtrip.
    #[test]
    fn connection_roundtrip() {
        let original = PktConnection {
            packet_type: PktType::CONNECTION,
            room_number: 7,
            room_name: Box::from("Secret Door"),
            description_len: 20,
            description: Box::from("A hidden passageway."),
        };

        let mut buffer: Vec<u8> = Vec::new();
        original.write_to(&mut buffer).expect("Encoding failed");

        let packet = Packet::new(PktType::CONNECTION, &buffer[1..]);
        let deserialized = PktConnection::decode(packet);

        assert_eq!(deserialized.room_number, 7);
        assert_eq!(deserialized.room_name.as_ref(), "Secret Door");
        assert_eq!(deserialized.description.as_ref(), "A hidden passageway.");
    }

    /// Long description.
    #[test]
    fn connection_long_description() {
        let desc = "C".repeat(5000);
        let mut body: Vec<u8> = Vec::new();
        body.extend(0u16.to_le_bytes());
        let mut name = b"Long".to_vec();
        name.resize(32, 0x00);
        body.extend(&name);
        body.extend((desc.len() as u16).to_le_bytes());
        body.extend(desc.as_bytes());

        let packet = Packet::new(PktType::CONNECTION, &body);
        let conn = PktConnection::decode(packet);

        assert_eq!(conn.description.len(), 5000);
    }

    /// Non-UTF8 data.
    #[test]
    fn connection_non_utf8() {
        let mut body: Vec<u8> = Vec::new();
        body.extend(0u16.to_le_bytes());
        let mut name = vec![0xFF, 0xFE, 0xFD];
        name.resize(32, 0x00);
        body.extend(&name);
        body.extend(2u16.to_le_bytes());
        body.extend(&[0xFC, 0xFB]);

        let packet = Packet::new(PktType::CONNECTION, &body);
        let conn = PktConnection::decode(packet);

        assert!(conn.room_name.contains('\u{FFFD}'));
        assert!(conn.description.contains('\u{FFFD}'));
    }

    /// Body too short should panic.
    #[test]
    #[should_panic]
    fn connection_body_too_short_panics() {
        let body: &[u8] = &[0x00, 0x00, 0x41]; // Need at least 36
        let packet = Packet::new(PktType::CONNECTION, body);
        let _ = PktConnection::decode(packet);
    }

    /// Empty body should panic.
    #[test]
    #[should_panic]
    fn connection_empty_body_panics() {
        let body: &[u8] = &[];
        let packet = Packet::new(PktType::CONNECTION, body);
        let _ = PktConnection::decode(packet);
    }

    /// All zeros body.
    #[test]
    fn connection_all_zeros_body() {
        let body: Vec<u8> = vec![0x00; 36];
        let packet = Packet::new(PktType::CONNECTION, &body);
        let conn = PktConnection::decode(packet);

        assert_eq!(conn.room_number, 0);
        assert_eq!(conn.room_name.as_ref(), "");
        assert_eq!(conn.description_len, 0);
    }

    /// Display/JSON output should be valid JSON.
    #[test]
    fn connection_display_valid_json() {
        let conn = PktConnection {
            packet_type: PktType::CONNECTION,
            room_number: 3,
            room_name: Box::from("Dungeon"),
            description_len: 6,
            description: Box::from("Dark.."),
        };
        let json_str = format!("{}", conn);
        let parsed: serde_json::Value = serde_json::from_str(&json_str).expect("Invalid JSON");
        assert_eq!(parsed["room_number"], 3);
        assert_eq!(parsed["room_name"], "Dungeon");
    }
}
////////////////////////////////////////////////////////////////////////////////
