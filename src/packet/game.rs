use serde::Serialize;
use std::io::Write;

use crate::packet::PktType;
use crate::{Packet, Parser};

#[derive(Serialize)]
/// Used by the server to describe the game.
///
/// - The initial points is a combination of health, defense, and regen, and cannot be exceeded by the client when defining a new character.
/// - The stat limit is a hard limit for the combination for any player on the server regardless of experience.
/// - If unused, it should be set to `65535`, the limit of the unsigned 16-bit integer.
///
/// This message will be sent upon connecting to the server, and not re-sent.
pub struct PktGame {
    /// The type of message for the `GAME` packet. Defaults to 11.
    pub packet_type: PktType,
    /// The initial points available to a new character.
    pub initial_points: u16,
    /// The maximum stat limit for any character.
    pub stat_limit: u16,
    /// The length of the game description.
    pub description_len: u16,
    /// The description of the game.
    pub description: Box<str>,
}

#[macro_export]
/// Send `PktGame` over `TcpStream` to connected user
///
/// ```no_run
/// use lurk_lcsc::{Protocol, PktGame, PktType, send_game};
/// use std::sync::Arc;
/// use std::net::TcpStream;
///
/// let stream = Arc::new(TcpStream::connect("127.0.0.1:8080").unwrap());
/// let game = PktGame {
///     packet_type: PktType::GAME,
///     initial_points: 100,
///     stat_limit: 65535,
///     description_len: 17,
///     description: Box::from("Test Description."),
/// };
///
/// send_game!(stream.clone(), game)
/// ```
macro_rules! send_game {
    ($stream:expr, $pkt_game:expr) => {
        $crate::Protocol::Game($stream, $pkt_game)
            .send()
            .expect("Failed to send game packet");
    };
}

impl std::fmt::Display for PktGame {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            serde_json::to_string(self).unwrap_or_else(|_| "Failed to serialize Game".to_string())
        )
    }
}

impl Parser<'_> for PktGame {
    fn serialize<W: Write>(self, writer: &mut W) -> Result<(), std::io::Error> {
        // Package into a byte array
        let mut packet: Vec<u8> = vec![self.packet_type.into()];

        packet.extend(self.initial_points.to_le_bytes());
        packet.extend(self.stat_limit.to_le_bytes());
        packet.extend(self.description_len.to_le_bytes());
        packet.extend(self.description.as_bytes());

        // Write the packet to the buffer
        writer
            .write_all(&packet)
            .map_err(|_| std::io::Error::other("Failed to write packet to buffer"))?;

        Ok(())
    }

    fn deserialize(packet: Packet) -> Self {
        let initial_points = u16::from_le_bytes([packet.body[0], packet.body[1]]);
        let stat_limit = u16::from_le_bytes([packet.body[2], packet.body[3]]);
        let description_len = u16::from_le_bytes([packet.body[4], packet.body[5]]);
        let description = String::from_utf8_lossy(&packet.body[6..]).into();

        Self {
            packet_type: packet.packet_type,
            initial_points,
            stat_limit,
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
    fn game_parse_and_serialize() {
        let stream = test_common::setup();
        let type_byte = PktType::GAME;
        let original_bytes: &[u8; 23] = &[
            0x0b, 0x64, 0x00, 0xff, 0xff, 0x10, 0x00, 0x54, 0x68, 0x69, 0x73, 0x20, 0x69, 0x73,
            0x20, 0x61, 0x20, 0x74, 0x65, 0x73, 0x74, 0x21, 0x0a,
        ];

        // Create a packet with known bytes, excluding the type byte
        let packet = Packet::new(&stream, type_byte, &original_bytes[1..]);

        // Deserialize the packet into a PktGame
        let message = PktGame::deserialize(packet);

        // Assert the fields were parsed correctly
        assert_eq!(message.packet_type, PktType::GAME);
        assert_eq!(message.initial_points, 100);
        assert_eq!(message.stat_limit, 65535);
        assert_eq!(message.description_len, 16);
        assert_eq!(message.description.as_ref(), "This is a test!\n");

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
