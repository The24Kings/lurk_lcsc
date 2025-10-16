use serde::Serialize;
use std::io::Write;
#[cfg(feature = "tracing")]
use tracing::error;

use crate::lurk_error::LurkError;
use crate::packet::PktType;
use crate::{Packet, Parser};

/// Notify the client of an error.
///
/// This is used to indicate stat violations, inappropriate room connections, attempts to loot nonexistent or living players, attempts to attack players or monsters in different rooms, etc.
#[derive(Serialize)]
pub struct PktError {
    /// The type of message for the `ERROR` packet. Defaults to 7.
    pub packet_type: PktType,
    /// The specific error code.
    pub error: LurkError,
    /// The length of the error message.
    pub message_len: u16,
    /// The error message.
    pub message: Box<str>,
}

impl PktError {
    /// Create a new `PktError` with the specified error code and message.
    pub fn new(error: LurkError, message: &str) -> Self {
        #[cfg(feature = "tracing")]
        error!("[SERVER] {}: {}", error, message);

        Self {
            packet_type: PktType::ERROR,
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

impl Parser<'_> for PktError {
    fn serialize<W: Write>(self, writer: &mut W) -> Result<(), std::io::Error> {
        // Package into a byte array
        let mut packet: Vec<u8> = vec![self.packet_type.into()];

        packet.push(self.error.into());
        packet.extend(self.message_len.to_le_bytes());
        packet.extend(self.message.as_bytes());

        // Write the packet to the buffer
        writer
            .write_all(&packet)
            .map_err(|_| std::io::Error::other("Failed to write packet to buffer"))?;

        Ok(())
    }

    fn deserialize(packet: Packet) -> Self {
        let message_type = packet.packet_type;
        let error = LurkError::from(packet.body[0]);
        let message_len = u16::from_le_bytes([packet.body[1], packet.body[2]]);
        let message = String::from_utf8_lossy(&packet.body[3..])
            .trim_end_matches('\0')
            .into();

        Self {
            packet_type: message_type,
            error,
            message_len,
            message,
        }
    }
}
