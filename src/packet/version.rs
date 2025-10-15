use serde::Serialize;
use std::io::Write;

use crate::packet::PktType;
use crate::{Packet, Parser};

#[derive(Serialize)]
/// Sent by the server upon initial connection along with `PktType::GAME`.
pub struct PktVersion {
    /// The type of message for the `VERSION` packet. Defaults to 14.
    pub packet_type: PktType,
    /// The major revision number of the server.
    pub major_rev: u8,
    /// The minor revision number of the server.
    pub minor_rev: u8,
    /// The length of the extensions field.
    pub extension_len: u16,
    /// The extensions field:
    /// - 0-1 Length of the first extension, as an unsigned 16-bit integer.
    /// - 2+ First extension
    ///
    /// At the end of the first extension, if there are more extensions, the length of the second extension will be found, then the second extension, and so on.
    /// The length of the list of extensions must be the same as `extension_len`.
    /// Note that servers and clients are not required to support any extensions at all, and in this case are free to ignore the list.
    pub extensions: Option<Vec<u8>>, // 0-1 length, 2+ extension;
}

impl std::fmt::Display for PktVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            serde_json::to_string(self)
                .unwrap_or_else(|_| "Failed to serialize Version".to_string())
        )
    }
}

impl Parser<'_> for PktVersion {
    fn serialize<W: Write>(self, writer: &mut W) -> Result<(), std::io::Error> {
        // Package into a byte array
        let mut packet: Vec<u8> = vec![self.packet_type.into()];

        packet.extend(self.major_rev.to_le_bytes());
        packet.extend(self.minor_rev.to_le_bytes());
        packet.extend(self.extension_len.to_le_bytes());

        if let Some(extensions) = &self.extensions {
            packet.extend(extensions);
        }

        // Write the packet to the buffer
        writer
            .write_all(&packet)
            .map_err(|_| std::io::Error::other("Failed to write packet to buffer"))?;

        Ok(())
    }

    fn deserialize(packet: Packet) -> Self {
        Self {
            packet_type: packet.packet_type,
            major_rev: packet.body[0],
            minor_rev: packet.body[1],
            extension_len: 0,
            extensions: None, // Server currently does not use extensions
        }
    }
}
