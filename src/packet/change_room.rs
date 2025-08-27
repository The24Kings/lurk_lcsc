use serde::Serialize;
use std::io::Write;

use crate::{Packet, Parser, PktType};

#[derive(Serialize)]
pub struct PktChangeRoom {
    pub message_type: PktType,
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

impl<'a> Parser<'a> for PktChangeRoom {
    fn serialize<W: Write>(self, writer: &mut W) -> Result<(), std::io::Error> {
        // Package into a byte array
        let mut packet: Vec<u8> = Vec::new();

        packet.push(self.message_type.into());
        packet.extend(self.room_number.to_le_bytes());

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
        let room_number = u16::from_le_bytes([packet.body[0], packet.body[1]]);

        // Implement deserialization logic here
        Ok(PktChangeRoom {
            message_type: packet.message_type,
            room_number,
        })
    }
}
