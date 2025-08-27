use serde::Serialize;
use std::io::Write;

use crate::{Packet, Parser, PktType};

#[derive(Serialize)]
pub struct PktPVPFight {
    pub message_type: PktType,
    pub target_name: String,
}

impl std::fmt::Display for PktPVPFight {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            serde_json::to_string(self)
                .unwrap_or_else(|_| "Failed to serialize PVPFight".to_string())
        )
    }
}

impl<'a> Parser<'a> for PktPVPFight {
    fn serialize<W: Write>(self, writer: &mut W) -> Result<(), std::io::Error> {
        // Package into a byte array
        let mut packet: Vec<u8> = Vec::new();

        packet.push(self.message_type.into());

        let mut target_name_bytes = self.target_name.as_bytes().to_vec();
        target_name_bytes.resize(32, 0x00); // Pad the name to 32 bytes
        packet.extend(target_name_bytes);

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
        let target_name = String::from_utf8_lossy(&packet.body[0..32])
            .trim_end_matches('\0')
            .to_string();

        Ok(PktPVPFight {
            message_type,
            target_name,
        })
    }
}
