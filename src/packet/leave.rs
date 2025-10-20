use serde::Serialize;
use std::io::Write;

use crate::packet::PktType;
use crate::{Packet, Parser};

#[derive(Serialize)]
/// Used by the client to leave the game. This is a graceful way to disconnect. The server never terminates, so it doesn't send `PktType::LEAVE`.
pub struct PktLeave {
    /// The type of message for the `LEAVE` packet. Defaults to 12.
    pub packet_type: PktType,
}

impl Default for PktLeave {
    fn default() -> Self {
        Self {
            packet_type: PktType::LEAVE,
        }
    }
}

#[macro_export]
/// Send `PktLeave` over `TcpStream` to connected user
///
/// ```no_run
/// use lurk_lcsc::{Protocol, PktLeave, LurkError, send_leave};
/// use std::sync::Arc;
/// use std::net::TcpStream;
///
/// let stream = Arc::new(TcpStream::connect("127.0.0.1:8080").unwrap());
///
/// send_leave!(stream.clone(), PktLeave::default())
/// ```
macro_rules! send_leave {
    ($stream:expr, $pkt_leave:expr) => {
        if let Err(e) = $crate::Protocol::Leave($stream, $pkt_leave).send() {
            ::tracing::error!("Failed to send leave packet: {}", e);
        }
    };
}

impl std::fmt::Display for PktLeave {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            serde_json::to_string(self).unwrap_or_else(|_| "Failed to serialize Leave".to_string())
        )
    }
}

impl Parser<'_> for PktLeave {
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
