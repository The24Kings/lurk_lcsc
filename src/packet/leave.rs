use serde::Serialize;
use std::io::Write;

use crate::packet::PktType;
use crate::{Packet, Parser};

#[derive(Serialize)]
/// Used by the client to leave the game. This is a graceful way to disconnect. The server never terminates, so it doesn't send `PktType::LEAVE`.
pub struct PktLeave {
    /// The type of message for the `LEAVE` packet. Defaults to 12.
    pub message_type: PktType,
}

impl Default for PktLeave {
    fn default() -> Self {
        Self {
            message_type: PktType::LEAVE,
        }
    }
}

impl std::fmt::Display for PktLeave {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            serde_json::to_string(self).unwrap_or_else(|_| "Failed to serialize Leave".to_string())
        )
    }
}

impl Parser<'_> for PktLeave {
    fn serialize<W: Write>(self, writer: &mut W) -> Result<(), std::io::Error> {
        // Package into a byte array
        let packet: Vec<u8> = vec![self.message_type.into()];

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
        Ok(PktLeave {
            message_type: packet.message_type,
        })
    }
}
