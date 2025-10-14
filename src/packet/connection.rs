use serde::Serialize;
use std::io::Write;

use crate::packet::PktType;
use crate::{Packet, Parser};

#[derive(Serialize)]
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
    pub message_type: PktType,
    /// Room number. This is the same room number used for `PktType::CHANGEROOM`
    pub room_number: u16,
    /// The name of the room this connection leads to, up to 32 bytes.
    pub room_name: Box<str>,
    /// The length of the room description.
    pub description_len: u16,
    /// The description of the room this connection leads to.
    pub description: Box<str>,
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
    fn serialize<W: Write>(self, writer: &mut W) -> Result<(), std::io::Error> {
        // Package into a byte array
        let mut packet: Vec<u8> = vec![self.message_type.into()];

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

    fn deserialize(packet: Packet) -> Self {
        let message_type = packet.message_type;
        let room_number = u16::from_le_bytes([packet.body[0], packet.body[1]]);
        let room_name = String::from_utf8_lossy(&packet.body[2..34])
            .trim_end_matches('\0')
            .into();
        let description_len = u16::from_le_bytes([packet.body[34], packet.body[35]]);
        let description = String::from_utf8_lossy(&packet.body[36..]).into();

        Self {
            message_type,
            room_number,
            room_name,
            description_len,
            description,
        }
    }
}
