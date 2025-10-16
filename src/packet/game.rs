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
