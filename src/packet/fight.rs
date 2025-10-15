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
