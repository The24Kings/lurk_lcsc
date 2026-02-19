use serde::{Deserialize, Serialize};
use std::io::Write;

use crate::packet::PktType;
use crate::{Packet, Parser};

#[derive(Serialize, Deserialize)]
/// Sent by the server upon initial connection along with `PktType::GAME`.
pub struct PktVersion {
    /// The type of message for the `VERSION` packet. Defaults to 14.
    pub packet_type: PktType,
    /// The major revision number of the server.
    pub major_rev: u8,
    /// The minor revision number of the server.
    pub minor_rev: u8,
    /// The length of the extensions field.
    pub extensions_len: u16,
    /// The extensions field:
    /// - 0-1 Length of the first extension, as an unsigned 16-bit integer.
    /// - 2+ First extension
    ///
    /// At the end of the first extension, if there are more extensions, the length of the second extension will be found, then the second extension, and so on.
    /// The length of the list of extensions must be the same as `extension_len`.
    /// Note that servers and clients are not required to support any extensions at all, and in this case are free to ignore the list.
    pub extensions: Option<Vec<u8>>, // 0-1 length, 2+ extension;
}

#[macro_export]
/// Send `PktVersion` over `TcpStream` to connected user
///
/// ```no_run
/// use lurk_lcsc::{Protocol, PktVersion, PktType, send_version};
/// use std::sync::Arc;
/// use std::net::TcpStream;
///
/// let stream = Arc::new(TcpStream::connect("127.0.0.1:8080").unwrap());
/// let version = PktVersion {
///     packet_type: PktType::VERSION,
///     major_rev: 2,
///     minor_rev: 3,
///     extensions_len: 0,
///     extensions: None,
/// };
///
/// send_version!(stream.clone(), version)
/// ```
macro_rules! send_version {
    ($stream:expr, $pkt_version:expr) => {
        $crate::Protocol::Version($stream, $pkt_version)
            .send()
            .expect("Failed to send version packet");
    };
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
        packet.extend(self.extensions_len.to_le_bytes());

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
            extensions_len: 0,
            extensions: None, // Server currently does not use extensions
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::test_common;

    use super::*;

    #[test]
    fn version_parse_and_serialize() {
        let stream = test_common::setup();
        let type_byte = PktType::VERSION;
        let original_bytes: &[u8; 5] = &[0x0e, 0x02, 0x03, 0x00, 0x00];

        // Create a packet with known bytes, excluding the type byte
        let packet = Packet::new(&stream, type_byte, &original_bytes[1..]);

        // Deserialize the packet into a PktVersion
        let message = <PktVersion as Parser>::deserialize(packet);

        // Assert the fields were parsed correctly
        assert_eq!(message.packet_type, PktType::VERSION);
        assert_eq!(message.major_rev, 2);
        assert_eq!(message.minor_rev, 3);
        assert_eq!(message.extensions_len, 0);
        assert!(message.extensions.is_none());

        // Serialize the message back into bytes
        let mut buffer: Vec<u8> = Vec::new();
        message
            .serialize(&mut buffer)
            .expect("Serialization failed");

        // Assert that the serialized bytes match the original
        assert_eq!(buffer, original_bytes);
        assert_eq!(buffer[0], u8::from(type_byte));
    }
}
////////////////////////////////////////////////////////////////////////////////
