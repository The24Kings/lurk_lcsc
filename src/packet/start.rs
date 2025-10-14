use serde::Serialize;
use std::io::Write;

use crate::packet::PktType;
use crate::{Packet, Parser};

#[derive(Serialize)]
/// Start playing the game.
///
/// - A client will send a `PktType::CHARACTER` message to the server to explain character stats, which the server may either accept or deny (by use of an `PktType::ERROR` message).
/// - If the stats are accepted, the server will not enter the player into the game world until it has received `PktType::START`.
/// - This is sent by the client.
/// - Generally, the server will reply with a `PktType::ROOM`, a `PktType::CHARACTER` message showing the updated room, and a `PktType::CHARACTER` message for each player in the initial room of the game.
pub struct PktStart {
    /// The type of message for the `START` packet. Defaults to 6.
    pub message_type: PktType,
}

impl Default for PktStart {
    fn default() -> Self {
        PktStart {
            message_type: PktType::START,
        }
    }
}

impl std::fmt::Display for PktStart {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            serde_json::to_string(self).unwrap_or_else(|_| "Failed to serialize Start".to_string())
        )
    }
}

impl Parser<'_> for PktStart {
    fn serialize<W: Write>(self, writer: &mut W) -> Result<(), std::io::Error> {
        // Package into a byte array
        let packet: Vec<u8> = vec![self.message_type.into()];

        // Write the packet to the buffer
        writer
            .write_all(&packet)
            .map_err(|_| std::io::Error::other("Failed to write packet to buffer"))?;

        Ok(())
    }
    fn deserialize(packet: Packet) -> Self {
        Self {
            message_type: packet.message_type,
        }
    }
}
