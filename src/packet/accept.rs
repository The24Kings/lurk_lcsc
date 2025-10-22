use crate::pkt_type::PktType;
use crate::{Packet, Parser};
use serde::Serialize;
use std::io::Write;

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

#[macro_export]
/// Send `PktAccept` over `TcpStream` to connected user
///
/// ```no_run
/// use lurk_lcsc::{Protocol, PktType, send_accept};
/// use std::sync::Arc;
/// use std::net::TcpStream;
///
/// let stream = Arc::new(TcpStream::connect("127.0.0.1:8080").unwrap());
///
/// send_accept!(stream.clone(), PktType::CHARACTER)
/// ```
macro_rules! send_accept {
    ($stream:expr, $p_type:expr) => {
        if let Err(e) = $crate::Protocol::Accept($stream, $crate::PktAccept::new($p_type)).send() {
            eprintln!("Failed to send 'ACCEPT' packet: {}", e);
        }
    };
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

#[cfg(test)]
mod tests {
    use crate::test_common;

    use super::*;

    #[test]
    fn accept_parse_and_serialize() {
        let stream = test_common::setup();
        let type_byte = PktType::ACCEPT;
        let original_bytes: &[u8; 2] = &[0x08, 0x0a];

        // Create a packet with known bytes, excluding the type byte
        let packet = Packet::new(&stream, type_byte, &original_bytes[1..]);

        // Deserialize the packet into a PktAccept
        let message = PktAccept::deserialize(packet);

        // Assert the fields were parsed correctly
        assert_eq!(message.packet_type, PktType::ACCEPT);
        assert_eq!(message.accept_type, u8::from(PktType::CHARACTER));

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
