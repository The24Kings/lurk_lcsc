use serde::Serialize;
use std::io::Write;

use crate::{LurkError, Packet, Parser, PktType};

#[derive(Serialize)]
pub struct PktError {
    pub message_type: PktType,
    pub error: LurkError,
    pub message_len: u16,
    pub message: Box<str>,
}

impl PktError {
    pub fn new(error: LurkError, message: &str) -> Self {
        PktError {
            message_type: PktType::ERROR,
            error,
            message_len: message.len() as u16,
            message: Box::from(message),
        }
    }
}

impl std::fmt::Display for PktError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            serde_json::to_string(self).unwrap_or_else(|_| "Failed to serialize Error".to_string())
        )
    }
}

impl<'a> Parser<'a> for PktError {
    fn serialize<W: Write>(self, writer: &mut W) -> Result<(), std::io::Error> {
        // Package into a byte array
        let mut packet: Vec<u8> = Vec::new();

        packet.push(self.message_type.into());
        packet.push(self.error.into());
        packet.extend(self.message_len.to_le_bytes());
        packet.extend(self.message.as_bytes());

        // Send the packet to the author
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
        let error = LurkError::from(packet.body[0]);
        let message_len = u16::from_le_bytes([packet.body[1], packet.body[2]]);
        let message = String::from_utf8_lossy(&packet.body[3..])
            .trim_end_matches('\0')
            .into();

        Ok(PktError {
            message_type,
            error,
            message_len,
            message,
        })
    }
}
