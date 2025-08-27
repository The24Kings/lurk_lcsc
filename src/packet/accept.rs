use serde::Serialize;
use std::io::Write;

use crate::{Packet, Parser, PktType};

#[derive(Serialize)]
pub struct PktAccept {
    pub message_type: PktType,
    pub accept_type: u8,
}

impl PktAccept {
    pub fn new(accept_type: PktType) -> Self {
        PktAccept {
            message_type: PktType::ACCEPT,
            accept_type: accept_type.into(),
        }
    }
}

impl std::fmt::Display for PktAccept {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            serde_json::to_string(self)
                .unwrap_or_else(|_| "Failed to serialize Accept".to_string())
        )
    }
}

impl<'a> Parser<'a> for PktAccept {
    fn serialize<W: Write>(self, writer: &mut W) -> Result<(), std::io::Error> {
        // Package into a byte array
        let mut packet: Vec<u8> = Vec::new();

        packet.push(self.message_type.into());
        packet.extend(self.accept_type.to_le_bytes());

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
        Ok(PktAccept {
            message_type: packet.message_type,
            accept_type: packet.body[0],
        })
    }
}
