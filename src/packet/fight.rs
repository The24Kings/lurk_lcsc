use serde::Serialize;
use std::io::Write;

use crate::packet::PktType;
use crate::{Packet, Parser};

#[derive(Serialize)]
/// Initiate a fight against monsters.
///
/// - This will start a fight in the current room against the monsters which are presently in the room.
/// - Players with the join battle flag set, who are in the same room, will automatically join in the fight.
/// - The server will allocate damage and rewards after the battle, and inform clients appropriately.
///   - Clients should expect a slew of messages after starting a fight, especially in a crowded room.
/// - This message is sent by the client.
///   - If a fight should ensue in the room the player is in, the server should notify the client, but not by use of this message.
///   - Instead, the players not initiating the fight should receive an updated `PktType::CHARACTER` message for each entity in the room.
/// - If the server wishes to send additional narrative text, this can be sent as a `PktType::MESSAGE`.
///
/// Note that this is not the only way a fight against monsters can be initiated. The server can initiate a fight at any time.
pub struct PktFight {
    /// The type of message for the `FIGHT` packet. Defaults to 3.
    pub packet_type: PktType,
}

impl Default for PktFight {
    fn default() -> Self {
        Self {
            packet_type: PktType::FIGHT,
        }
    }
}

#[macro_export]
/// Send `PktFight` over `TcpStream` to connected user
///
/// ```no_run
/// use lurk_lcsc::{Protocol, PktFight, LurkError, send_fight};
/// use std::sync::Arc;
/// use std::net::TcpStream;
///
/// let stream = Arc::new(TcpStream::connect("127.0.0.1:8080").unwrap());
///
/// send_fight!(stream.clone(), PktFight::default())
/// ```
macro_rules! send_fight {
    ($stream:expr, $pkt_fight:expr) => {
        if let Err(e) = $crate::Protocol::Fight($stream, $pkt_fight).send() {
            eprintln!("Failed to send fight packet: {}", e);
        }
    };
}

impl std::fmt::Display for PktFight {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            serde_json::to_string(self).unwrap_or_else(|_| "Failed to serialize Fight".to_string())
        )
    }
}

impl Parser<'_> for PktFight {
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
    fn fight_parse_and_serialize() {
        let stream = test_common::setup();
        let type_byte = PktType::FIGHT;
        let original_bytes: &[u8; 1] = &[0x03];

        // Create a packet with known bytes, excluding the type byte
        let packet = Packet::new(&stream, type_byte, &[]);

        // Deserialize the packet into a PktFight
        let message = PktFight::deserialize(packet);

        // Assert the fields were parsed correctly
        assert_eq!(message.packet_type, PktType::FIGHT);

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
