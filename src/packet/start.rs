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
    pub packet_type: PktType,
}

impl Default for PktStart {
    fn default() -> Self {
        Self {
            packet_type: PktType::START,
        }
    }
}

#[macro_export]
/// Send `PktStart` over `TcpStream` to connected user
///
/// ```no_run
/// use lurk_lcsc::{Protocol, PktStart, LurkError, send_start};
/// use std::sync::Arc;
/// use std::net::TcpStream;
///
/// let stream = Arc::new(TcpStream::connect("127.0.0.1:8080").unwrap());
///
/// send_start!(stream.clone(), PktStart::default())
/// ```
macro_rules! send_start {
    ($stream:expr, $pkt_fight:expr) => {
        if let Err(e) = $crate::Protocol::Start($stream, $pkt_fight).send() {
            eprintln!("Failed to send start packet: {}", e);
        }
    };
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
        let packet: Vec<u8> = vec![self.packet_type.into()];

        // Write the packet to the buffer
        writer
            .write_all(&packet)
            .map_err(|_| std::io::Error::other("Failed to write packet to buffer"))?;

        Ok(())
    }

    fn deserialize(packet: Packet) -> Self {
        Self {
            packet_type: packet.packet_type,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::test_common;

    use super::*;

    #[test]
    fn start_parse_and_serialize() {
        let stream = test_common::setup();
        let type_byte = PktType::START;
        let original_bytes: &[u8; 1] = &[0x06];
        // Create a packet with known bytes, excluding the type byte
        let packet = Packet::new(&stream, type_byte, &original_bytes[1..]);

        // Deserialize the packet into a PktStart
        let message = PktStart::deserialize(packet);

        // Assert the fields were parsed correctly
        assert_eq!(message.packet_type, PktType::START);

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
