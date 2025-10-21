use serde::Serialize;
use std::io::Write;

use crate::packet::PktType;
use crate::{Packet, Parser};

#[derive(Serialize)]
/// Initiate a fight against another player.
///
/// - The server will determine the results of the fight, and allocate damage and rewards appropriately.
/// - The server may include players with join battle in the fight, on either side.
/// - Monsters may or may not be involved in the fight as well.
/// - This message is sent by the client.
///
/// If the server does not support PVP, it should send `LurkError::NOPLAYERCOMBAT` to the client.
pub struct PktPVPFight {
    /// The type of message for the `PVPFIGHT` packet. Defaults to 4.
    pub packet_type: PktType,
    /// The name of the target player, up to 32 bytes.
    pub target_name: Box<str>,
}

impl PktPVPFight {
    /// Create a new PktPVPFight packet from a given name
    pub fn fight(name: &str) -> Self {
        Self {
            packet_type: PktType::PVPFIGHT,
            target_name: Box::from(name),
        }
    }
}

#[macro_export]
/// Send `PktPVPFight` over `TcpStream` to connected user
///
/// ```no_run
/// use lurk_lcsc::{Protocol, PktPVPFight, LurkError, send_pvp};
/// use std::sync::Arc;
/// use std::net::TcpStream;
///
/// let stream = Arc::new(TcpStream::connect("127.0.0.1:8080").unwrap());
///
/// send_pvp!(stream.clone(), PktPVPFight::fight("Test"))
/// ```
macro_rules! send_pvp {
    ($stream:expr, $pkt_pvp:expr) => {
        if let Err(e) = $crate::Protocol::PVPFight($stream, $pkt_pvp).send() {
            eprintln!("Failed to send pvp fight packet: {}", e);
        }
    };
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

impl Parser<'_> for PktPVPFight {
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
