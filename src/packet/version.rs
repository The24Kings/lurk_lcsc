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
/// use lurk_protocol::{Protocol, PktVersion, PktType, send_version};
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
        $crate::send_to($stream.as_ref(), &$pkt_version).expect("Failed to send version packet");
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
    fn write_to<W: Write>(&self, writer: &mut W) -> Result<(), std::io::Error> {
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

    fn decode(packet: Packet) -> Self {
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
    use super::*;

    #[test]
    fn version_parse_and_serialize() {
        let type_byte = PktType::VERSION;
        let original_bytes: &[u8; 5] = &[0x0e, 0x02, 0x03, 0x00, 0x00];

        // Create a packet with known bytes, excluding the type byte
        let packet = Packet::new(type_byte, &original_bytes[1..]);

        // Deserialize the packet into a PktVersion
        let message = PktVersion::decode(packet);

        // Assert the fields were parsed correctly
        assert_eq!(message.packet_type, PktType::VERSION);
        assert_eq!(message.major_rev, 2);
        assert_eq!(message.minor_rev, 3);
        assert_eq!(message.extensions_len, 0);
        assert!(message.extensions.is_none());

        // Serialize the message back into bytes
        let mut buffer: Vec<u8> = Vec::new();
        message.write_to(&mut buffer).expect("Encoding failed");

        // Assert that the serialized bytes match the original
        assert_eq!(buffer, original_bytes);
        assert_eq!(buffer[0], u8::from(type_byte));
    }

    /// Verify parsing the exact bytes captured from the ZeldaServer trace output.
    #[test]
    fn version_parse_trace_bytes() {
        // From trace: 0e 02 03 00 00
        let body: &[u8] = &[0x02, 0x03, 0x00, 0x00];
        let packet = Packet::new(PktType::VERSION, body);
        let ver = PktVersion::decode(packet);

        assert_eq!(ver.major_rev, 2);
        assert_eq!(ver.minor_rev, 3);
        assert_eq!(ver.extensions_len, 0);
        assert!(ver.extensions.is_none());
    }

    /// Max u8 values for major and minor revision numbers.
    #[test]
    fn version_max_revisions() {
        let body: &[u8] = &[0xFF, 0xFF, 0x00, 0x00];
        let packet = Packet::new(PktType::VERSION, body);
        let ver = PktVersion::decode(packet);

        assert_eq!(ver.major_rev, 255);
        assert_eq!(ver.minor_rev, 255);
    }

    /// Zero major and minor revisions.
    #[test]
    fn version_zero_revisions() {
        let body: &[u8] = &[0x00, 0x00, 0x00, 0x00];
        let packet = Packet::new(PktType::VERSION, body);
        let ver = PktVersion::decode(packet);

        assert_eq!(ver.major_rev, 0);
        assert_eq!(ver.minor_rev, 0);
    }

    /// Serialize with extensions included and verify output.
    #[test]
    fn version_serialize_with_extensions() {
        let ver = PktVersion {
            packet_type: PktType::VERSION,
            major_rev: 2,
            minor_rev: 3,
            extensions_len: 5,
            extensions: Some(vec![0x05, 0x00, 0x41, 0x42, 0x43]),
        };

        let mut buffer: Vec<u8> = Vec::new();
        ver.write_to(&mut buffer).expect("Encoding failed");

        // Type(1) + major(1) + minor(1) + ext_len(2) + extensions(5) = 10
        assert_eq!(buffer.len(), 10);
        assert_eq!(buffer[0], 0x0e); // VERSION type
        assert_eq!(buffer[1], 0x02);
        assert_eq!(buffer[2], 0x03);
        assert_eq!(buffer[3], 0x05); // extensions_len low byte
        assert_eq!(buffer[4], 0x00); // extensions_len high byte
        assert_eq!(&buffer[5..], &[0x05, 0x00, 0x41, 0x42, 0x43]);
    }

    /// Serialize with no extensions and verify compact output.
    #[test]
    fn version_serialize_no_extensions() {
        let ver = PktVersion {
            packet_type: PktType::VERSION,
            major_rev: 1,
            minor_rev: 0,
            extensions_len: 0,
            extensions: None,
        };

        let mut buffer: Vec<u8> = Vec::new();
        ver.write_to(&mut buffer).expect("Encoding failed");

        assert_eq!(buffer.len(), 5);
        assert_eq!(buffer, &[0x0e, 0x01, 0x00, 0x00, 0x00]);
    }

    /// Roundtrip: construct, serialize, then deserialize and verify equality.
    #[test]
    fn version_roundtrip() {
        let original = PktVersion {
            packet_type: PktType::VERSION,
            major_rev: 10,
            minor_rev: 42,
            extensions_len: 0,
            extensions: None,
        };

        let mut buffer: Vec<u8> = Vec::new();
        original.write_to(&mut buffer).expect("Encoding failed");

        let packet = Packet::new(PktType::VERSION, &buffer[1..]);
        let deserialized = PktVersion::decode(packet);

        assert_eq!(deserialized.major_rev, 10);
        assert_eq!(deserialized.minor_rev, 42);
    }

    /// Body with extra trailing bytes should still parse correctly (only first bytes used).
    #[test]
    fn version_extra_trailing_bytes() {
        let body: &[u8] = &[0x02, 0x03, 0x00, 0x00, 0xFF, 0xFF, 0xFF];
        let packet = Packet::new(PktType::VERSION, body);
        let ver = PktVersion::decode(packet);

        assert_eq!(ver.major_rev, 2);
        assert_eq!(ver.minor_rev, 3);
    }

    /// Too-short body should panic during deserialization (index out of bounds).
    #[test]
    #[should_panic]
    fn version_body_too_short_panics() {
        let body: &[u8] = &[0x02]; // Only 1 byte, need at least 2
        let packet = Packet::new(PktType::VERSION, body);
        let _ = PktVersion::decode(packet);
    }

    /// Empty body should panic during deserialization.
    #[test]
    #[should_panic]
    fn version_empty_body_panics() {
        let body: &[u8] = &[];
        let packet = Packet::new(PktType::VERSION, body);
        let _ = PktVersion::decode(packet);
    }

    /// All 0xFF bytes in body.
    #[test]
    fn version_all_ones_body() {
        let body: &[u8] = &[0xFF, 0xFF, 0xFF, 0xFF];
        let packet = Packet::new(PktType::VERSION, body);
        let ver = PktVersion::decode(packet);

        assert_eq!(ver.major_rev, 255);
        assert_eq!(ver.minor_rev, 255);
    }

    /// All 0x00 bytes in body.
    #[test]
    fn version_all_zeros_body() {
        let body: &[u8] = &[0x00, 0x00, 0x00, 0x00];
        let packet = Packet::new(PktType::VERSION, body);
        let ver = PktVersion::decode(packet);

        assert_eq!(ver.major_rev, 0);
        assert_eq!(ver.minor_rev, 0);
    }

    /// Display/JSON output should be valid JSON.
    #[test]
    fn version_display_valid_json() {
        let ver = PktVersion {
            packet_type: PktType::VERSION,
            major_rev: 2,
            minor_rev: 3,
            extensions_len: 0,
            extensions: None,
        };
        let json_str = format!("{}", ver);
        let parsed: serde_json::Value = serde_json::from_str(&json_str).expect("Invalid JSON");
        assert_eq!(parsed["major_rev"], 2);
        assert_eq!(parsed["minor_rev"], 3);
    }
}
////////////////////////////////////////////////////////////////////////////////
