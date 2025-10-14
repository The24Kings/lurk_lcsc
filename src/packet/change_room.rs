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
    pub message_type: PktType,
    /// Number of the room to change to. The server will send an error if an inappropriate choice is made.
    pub room_number: u16,
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
        let mut packet: Vec<u8> = vec![self.message_type.into()];

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
            message_type: packet.message_type,
            room_number,
        }
    }
}
