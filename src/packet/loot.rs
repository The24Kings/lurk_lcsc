use serde::Serialize;
use std::io::Write;

use crate::packet::PktType;
use crate::{Packet, Parser};

/// Represents a loot packet containing the message type and target name.
#[derive(Serialize)]
pub struct PktLoot {
    /// The type of the packet message.
    pub packet_type: PktType,
    /// The name of the loot target.
    pub target_name: Box<str>,
}

impl PktLoot {
    /// Create a new PktLoot packet from a given name
    pub fn loot(name: &str) -> Self {
        Self {
            packet_type: PktType::LOOT,
            target_name: Box::from(name),
        }
    }
}

#[macro_export]
/// Send `PktLoot` over `TcpStream` to connected user
///
/// ```no_run
/// use lurk_lcsc::{Protocol, PktLoot, LurkError, send_loot};
/// use std::sync::Arc;
/// use std::net::TcpStream;
///
/// let stream = Arc::new(TcpStream::connect("127.0.0.1:8080").unwrap());
///
/// send_loot!(stream.clone(), PktLoot::loot("Test"))
/// ```
macro_rules! send_loot {
    ($stream:expr, $pkt_loot:expr) => {
        if let Err(e) = $crate::Protocol::Loot($stream, $pkt_loot).send() {
            ::tracing::error!("Failed to send loot packet: {}", e);
        }
    };
}

impl std::fmt::Display for PktLoot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            serde_json::to_string(self).unwrap_or_else(|_| "Failed to serialize Loot".to_string())
        )
    }
}

impl Parser<'_> for PktLoot {
    fn serialize<W: Write>(self, writer: &mut W) -> Result<(), std::io::Error> {
        // Package into a byte array
        let mut packet: Vec<u8> = vec![self.packet_type.into()];

        let mut target_name_bytes = self.target_name.as_bytes().to_vec();
        target_name_bytes.resize(32, 0x00); // Pad the name to 32 bytes
        packet.extend(target_name_bytes);

        // Write the packet to the buffer
        writer
            .write_all(&packet)
            .map_err(|_| std::io::Error::other("Failed to write packet to buffer"))?;

        Ok(())
    }

    fn deserialize(packet: Packet) -> Self {
        let message_type = packet.packet_type;
        let target_name = String::from_utf8_lossy(&packet.body[0..32])
            .trim_end_matches('\0')
            .into();

        Self {
            packet_type: message_type,
            target_name,
        }
    }
}
