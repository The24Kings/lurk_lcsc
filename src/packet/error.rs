use serde::{Deserialize, Serialize};
use std::io::Write;
#[cfg(feature = "tracing")]
use tracing::error;

use crate::lurk_error::LurkError;
use crate::packet::PktType;
use crate::{Packet, Parser};

/// Notify the client of an error.
///
/// This is used to indicate stat violations, inappropriate room connections, attempts to loot nonexistent or living players, attempts to attack players or monsters in different rooms, etc.
#[derive(Serialize, Deserialize)]
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
        error!("{}: {}", error, message);

        Self {
            packet_type: PktType::ERROR,
            error,
            message_len: message.len() as u16,
            message: Box::from(message),
        }
    }
}

#[macro_export]
/// Send `PktError` over `TcpStream` to connected user
///
/// ```no_run
/// use lurk_lcsc::{Protocol, PktError, LurkError, send_error};
/// use std::sync::Arc;
/// use std::net::TcpStream;
///
/// let stream = Arc::new(TcpStream::connect("127.0.0.1:8080").unwrap());
///
/// send_error!(stream.clone(), PktError::new(LurkError::NOTREADY, "Start the game first!"))
/// ```
macro_rules! send_error {
    ($stream:expr, $pkt_error:expr) => {
        if let Err(e) = $crate::Protocol::Error($stream, $pkt_error).send() {
            eprintln!("Failed to send error packet: {}", e);
        }
    };
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
            .split('\0')
            .take(1)
            .collect();

        Self {
            packet_type: message_type,
            error,
            message_len,
            message,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::test_common;

    use super::*;

    #[test]
    fn error_parse_and_serialize() {
        let stream = test_common::setup();
        let type_byte = PktType::ERROR;
        let original_bytes: &[u8; 17] = &[
            0x07, 0x04, 0x0d, 0x00, 0x49, 0x6e, 0x76, 0x61, 0x6c, 0x69, 0x64, 0x20, 0x73, 0x74,
            0x61, 0x74, 0x73,
        ];

        // Create a packet with known bytes, excluding the type byte
        let packet = Packet::new(&stream, type_byte, &original_bytes[1..]);

        // Deserialize the packet into a PktError
        let message = <PktError as Parser>::deserialize(packet);

        // Assert the fields were parsed correctly
        assert_eq!(message.packet_type, PktType::ERROR);
        assert_eq!(message.error, LurkError::STATERROR);
        assert_eq!(message.message_len, 13);
        assert_eq!(message.message.as_ref(), "Invalid stats");

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
