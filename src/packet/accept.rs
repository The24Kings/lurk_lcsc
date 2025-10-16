use serde::Serialize;
use std::io::Write;

use crate::pkt_type::PktType;
use crate::{Packet, Parser};

/// Sent by the server to acknowledge a non-error-causing action which has no other direct result.
///
/// This is not needed for actions which cause other results, such as changing rooms or beginning a fight.
/// It should be sent in response to clients sending messages, setting character stats, etc.
#[derive(Serialize)]
pub struct PktAccept {
    /// The type of message for the `ACCEPT` packet. Default is 8.
    pub packet_type: PktType,
    /// The type of action accepted.
    pub accept_type: u8,
}

impl PktAccept {
    /// Creates a new `PktAccept` with the specified accept type.
    pub fn new(accept_type: PktType) -> Self {
        Self {
            packet_type: PktType::ACCEPT,
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

impl Parser<'_> for PktAccept {
    fn serialize<W: Write>(self, writer: &mut W) -> Result<(), std::io::Error> {
        // Package into a byte array
        let mut packet: Vec<u8> = Vec::new();

        packet.push(self.packet_type.into());
        packet.extend(self.accept_type.to_le_bytes());

        // Write the packet to the buffer
        writer
            .write_all(&packet)
            .map_err(|_| std::io::Error::other("Failed to write packet to buffer"))?;

        Ok(())
    }

    fn deserialize(packet: Packet) -> Self {
        Self {
            packet_type: packet.packet_type,
            accept_type: packet.body[0],
        }
    }
}
