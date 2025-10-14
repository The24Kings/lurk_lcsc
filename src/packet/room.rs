use serde::Serialize;
use std::io::Write;

use crate::packet::PktType;
use crate::{Packet, Parser};

#[derive(Serialize)]
/// Sent by the server to describe the room that the player is in.
///
/// - This should be an expected response to `PktType::CHANGEROOM` or `PktType::START`.
/// - Can be re-sent at any time, for example if the player is teleported or falls through a floor.
/// - Outgoing connections will be specified with a series of `PktType::CONNECTION` messages.
/// - Monsters and players in the room should be listed using a series of `PktType::CHARACTER` messages.
pub struct PktRoom {
    /// The type of message for the `ROOM` packet. Defaults to 9
    pub message_type: PktType,
    /// The room number the player is currently in. This is the same as the room number used in `PktType::CHANGEROOM`.
    pub room_number: u16,
    /// The name of the room, up to 32 bytes.
    pub room_name: Box<str>,
    /// The length of the room description.
    pub description_len: u16,
    /// The room description.
    pub description: Box<str>,
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
        let mut packet: Vec<u8> = vec![self.message_type.into()];

        packet.extend(self.room_number.to_le_bytes());

        let mut room_name_bytes = self.room_name.as_bytes().to_vec();
        room_name_bytes.resize(32, 0); // Pad with zeros to 32 bytes
        packet.extend(room_name_bytes);

        packet.extend(self.description_len.to_le_bytes());
        packet.extend(self.description.as_bytes());

        // Write the packet to the buffer
        writer.write_all(&packet).map_err(|_| {
            std::io::Error::new(
                std::io::ErrorKind::Other,
                "Failed to write packet to buffer",
            )
        })?;

        Ok(())
    }

    fn deserialize(packet: Packet) -> Result<Self, std::io::Error> {
        let message_type = packet.message_type;
        let room_number = u16::from_le_bytes([packet.body[0], packet.body[1]]);
        let room_name = String::from_utf8_lossy(&packet.body[2..34])
            .trim_end_matches('\0')
            .into();
        let description_len = u16::from_le_bytes([packet.body[34], packet.body[35]]);
        let description = String::from_utf8_lossy(&packet.body[36..]).into();

        Ok(PktRoom {
            message_type,
            room_number,
            room_name,
            description_len,
            description,
        })
    }
}
