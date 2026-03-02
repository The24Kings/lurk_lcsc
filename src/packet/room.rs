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

    /// Parse from trace: room 0, "Outside the Great Deku Tree".
    #[test]
    fn room_parse_trace_deku_tree() {
        let stream = test_common::setup();
        let room_name = "Outside the Great Deku Tree";
        let desc = "The dense forest clears just enough to reveal a towering presence.";
        let mut body: Vec<u8> = Vec::new();
        body.extend(0u16.to_le_bytes()); // room_number = 0
        let mut name_bytes = room_name.as_bytes().to_vec();
        name_bytes.resize(32, 0x00);
        body.extend(&name_bytes);
        body.extend((desc.len() as u16).to_le_bytes());
        body.extend(desc.as_bytes());

        let packet = Packet::new(&stream, PktType::ROOM, &body);
        let room = <PktRoom as Parser>::deserialize(packet);

        assert_eq!(room.room_number, 0);
        assert_eq!(room.room_name.as_ref(), room_name);
        assert_eq!(room.description.as_ref(), desc);
    }

    /// Empty description.
    #[test]
    fn room_empty_description() {
        let stream = test_common::setup();
        let mut body: Vec<u8> = Vec::new();
        body.extend(5u16.to_le_bytes());
        let mut name = b"Empty".to_vec();
        name.resize(32, 0x00);
        body.extend(&name);
        body.extend(0u16.to_le_bytes());

        let packet = Packet::new(&stream, PktType::ROOM, &body);
        let room = <PktRoom as Parser>::deserialize(packet);

        assert_eq!(room.room_number, 5);
        assert_eq!(room.room_name.as_ref(), "Empty");
        assert_eq!(room.description_len, 0);
        assert_eq!(room.description.as_ref(), "");
    }

    /// Max room number.
    #[test]
    fn room_max_room_number() {
        let stream = test_common::setup();
        let mut body: Vec<u8> = Vec::new();
        body.extend(u16::MAX.to_le_bytes());
        let mut name = b"MaxRoom".to_vec();
        name.resize(32, 0x00);
        body.extend(&name);
        body.extend(0u16.to_le_bytes());

        let packet = Packet::new(&stream, PktType::ROOM, &body);
        let room = <PktRoom as Parser>::deserialize(packet);

        assert_eq!(room.room_number, u16::MAX);
    }

    /// Max-length room name (32 bytes).
    #[test]
    fn room_max_length_name() {
        let stream = test_common::setup();
        let long_name = "A".repeat(32);
        let mut body: Vec<u8> = Vec::new();
        body.extend(0u16.to_le_bytes());
        body.extend(long_name.as_bytes());
        body.extend(0u16.to_le_bytes());

        let packet = Packet::new(&stream, PktType::ROOM, &body);
        let room = <PktRoom as Parser>::deserialize(packet);

        assert_eq!(room.room_name.as_ref(), &long_name);
    }

    /// Roundtrip: construct, serialize, deserialize.
    #[test]
    fn room_roundtrip() {
        let stream = test_common::setup();
        let original = PktRoom {
            packet_type: PktType::ROOM,
            room_number: 42,
            room_name: Box::from("Treasure Room"),
            description_len: 18,
            description: Box::from("Glittering jewels!"),
        };

        let mut buffer: Vec<u8> = Vec::new();
        original
            .serialize(&mut buffer)
            .expect("Serialization failed");

        let packet = Packet::new(&stream, PktType::ROOM, &buffer[1..]);
        let deserialized = <PktRoom as Parser>::deserialize(packet);

        assert_eq!(deserialized.room_number, 42);
        assert_eq!(deserialized.room_name.as_ref(), "Treasure Room");
        assert_eq!(deserialized.description.as_ref(), "Glittering jewels!");
    }

    /// Long description.
    #[test]
    fn room_long_description() {
        let stream = test_common::setup();
        let desc = "B".repeat(5000);
        let mut body: Vec<u8> = Vec::new();
        body.extend(0u16.to_le_bytes());
        let mut name = b"Long".to_vec();
        name.resize(32, 0x00);
        body.extend(&name);
        body.extend((desc.len() as u16).to_le_bytes());
        body.extend(desc.as_bytes());

        let packet = Packet::new(&stream, PktType::ROOM, &body);
        let room = <PktRoom as Parser>::deserialize(packet);

        assert_eq!(room.description.len(), 5000);
    }

    /// Non-UTF8 room name and description.
    #[test]
    fn room_non_utf8() {
        let stream = test_common::setup();
        let mut body: Vec<u8> = Vec::new();
        body.extend(0u16.to_le_bytes());
        let mut name = vec![0xFF, 0xFE, 0xFD];
        name.resize(32, 0x00);
        body.extend(&name);
        body.extend(3u16.to_le_bytes());
        body.extend(&[0xFC, 0xFB, 0xFA]);

        let packet = Packet::new(&stream, PktType::ROOM, &body);
        let room = <PktRoom as Parser>::deserialize(packet);

        assert!(room.room_name.contains('\u{FFFD}'));
        assert!(room.description.contains('\u{FFFD}'));
    }

    /// Body too short should panic.
    #[test]
    #[should_panic]
    fn room_body_too_short_panics() {
        let stream = test_common::setup();
        let body: &[u8] = &[0x00, 0x00]; // Need at least 36
        let packet = Packet::new(&stream, PktType::ROOM, body);
        let _ = <PktRoom as Parser>::deserialize(packet);
    }

    /// Empty body should panic.
    #[test]
    #[should_panic]
    fn room_empty_body_panics() {
        let stream = test_common::setup();
        let body: &[u8] = &[];
        let packet = Packet::new(&stream, PktType::ROOM, body);
        let _ = <PktRoom as Parser>::deserialize(packet);
    }

    /// All zeros body (36 bytes min header).
    #[test]
    fn room_all_zeros_body() {
        let stream = test_common::setup();
        let body: Vec<u8> = vec![0x00; 36];
        let packet = Packet::new(&stream, PktType::ROOM, &body);
        let room = <PktRoom as Parser>::deserialize(packet);

        assert_eq!(room.room_number, 0);
        assert_eq!(room.room_name.as_ref(), "");
        assert_eq!(room.description_len, 0);
    }

    /// Display/JSON output should be valid JSON.
    #[test]
    fn room_display_valid_json() {
        let room = PktRoom {
            packet_type: PktType::ROOM,
            room_number: 1,
            room_name: Box::from("Hall"),
            description_len: 5,
            description: Box::from("A hall"),
        };
        let json_str = format!("{}", room);
        let parsed: serde_json::Value = serde_json::from_str(&json_str).expect("Invalid JSON");
        assert_eq!(parsed["room_number"], 1);
        assert_eq!(parsed["room_name"], "Hall");
    }
}
////////////////////////////////////////////////////////////////////////////////
