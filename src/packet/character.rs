use std::{io::Write, net::TcpStream, sync::Arc};

use serde::Serialize;

use crate::Packet;
use crate::Parser;
use crate::flags::CharacterFlags;
use crate::packet::PktType;

#[derive(Clone, Serialize)]
/// Sent by both the client and the server.
///
/// - The server will send this message to show the client changes to their player's status, such as in health or gold.
/// - The server will also use this message to show other players or monsters in the room the player is in or elsewhere.
/// - The client should expect to receive character messages at any time, which may be updates to the player or others.
/// - If the player is in a room with another player, and the other player leaves, a `PktType::CHARACTER` message should be sent to indicate this.
///   - In many cases, the appropriate room for the outgoing player is the room they have gone to.
/// - If the player goes to an unknown room, the room number may be set to a room that the player will not encounter (does not have to be part of the map).
///   - This could be accompanied by a narrative message (for example, "Glorfindel vanishes into a puff of smoke"), but this is not required.
/// - The client will use this message to set the name, description, attack, defense, regen, and flags when the character is created.
/// - It can also be used to reprise an abandoned or deceased character.
pub struct PktCharacter {
    #[serde(skip_serializing)]
    /// The TCP stream associated with the author of the packet, if available.
    pub author: Option<Arc<TcpStream>>,
    /// The type of message for the `CHARACTER` packet. Default is 10.
    pub packet_type: PktType,
    /// The name of the character, up to 32 bytes.
    pub name: Arc<str>,
    /// The character's flags, represented as a bitfield.
    pub flags: CharacterFlags,
    /// The character's attack stat.
    pub attack: u16,
    /// The character's defense stat.
    pub defense: u16,
    /// The character's regeneration stat.
    pub regen: u16,
    /// The character's health stat.
    pub health: i16,
    /// The character's gold amount.
    pub gold: u16,
    /// The character's current room.
    pub current_room: u16,
    /// The length of the character's description.
    pub description_len: u16,
    /// The character's description.
    pub description: Box<str>,
}

impl PktCharacter {
    /// Creates a new `PktCharacter` with default values for health, gold, current_room, and flags, cloning other fields from the incoming character.
    pub fn with_defaults_from(incoming: &PktCharacter) -> Self {
        Self {
            health: 100,
            gold: 0,
            current_room: 0,
            flags: CharacterFlags::reset(),
            ..incoming.clone()
        }
    }
}

#[macro_export]
/// Send `PktCharacter` over `TcpStream` to connected user
///
/// ```no_run
/// use lurk_lcsc::{
///     Protocol, PktCharacter, LurkError,
///     PktType, send_character, CharacterFlags,
/// };
/// use std::sync::Arc;
/// use std::net::TcpStream;
///
/// let stream = Arc::new(TcpStream::connect("127.0.0.1:8080").unwrap());
/// let player = PktCharacter {
///     author: None,
///     packet_type: PktType::CHARACTER,
///     name: "Test".into(),
///     flags: CharacterFlags::reset(),
///     attack: 50,
///     defense: 25,
///     regen: 25,
///     health: 100,
///     gold: 0,
///     current_room: 0,
///     description_len: 0,
///     description: "".into(),
/// };
///
/// send_character!(stream.clone(), player)
/// ```
macro_rules! send_character {
    ($stream:expr, $player:expr) => {
        if let Err(e) = $crate::Protocol::Character($stream, $player).send() {
            eprintln!("Failed to send character packet: {}", e);
        }
    };
}

impl std::fmt::Display for PktCharacter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            serde_json::to_string(self)
                .unwrap_or_else(|_| "Failed to serialize Character".to_string())
        )
    }
}

impl Parser<'_> for PktCharacter {
    fn serialize<W: Write>(self, writer: &mut W) -> Result<(), std::io::Error> {
        // Package into a byte array
        let mut packet: Vec<u8> = vec![self.packet_type.into()];

        // Serialize the character name
        let mut name_bytes = self.name.as_bytes().to_vec();
        name_bytes.resize(32, 0x00); // Pad the name to 32 bytes

        packet.extend(name_bytes);

        // Serialize the flags byte
        packet.extend([self.flags.bits()]);

        // Serialize the character stats
        packet.extend(self.attack.to_le_bytes());
        packet.extend(self.defense.to_le_bytes());
        packet.extend(self.regen.to_le_bytes());
        packet.extend(self.health.to_le_bytes());
        packet.extend(self.gold.to_le_bytes());
        packet.extend(self.current_room.to_le_bytes());
        packet.extend(self.description_len.to_le_bytes());
        packet.extend(self.description.as_bytes());

        // Write the packet to the buffer
        writer
            .write_all(&packet)
            .map_err(|_| std::io::Error::other("Failed to write packet to buffer"))?;

        Ok(())
    }

    fn deserialize(packet: Packet) -> Self {
        let name = String::from_utf8_lossy(&packet.body[0..32])
            .split('\0')
            .take(1)
            .collect::<String>();
        let flags = CharacterFlags::from_bits_truncate(packet.body[32]); // Other bits are reserved for future use
        let attack = u16::from_le_bytes([packet.body[33], packet.body[34]]);
        let defense = u16::from_le_bytes([packet.body[35], packet.body[36]]);
        let regen = u16::from_le_bytes([packet.body[37], packet.body[38]]);
        let health = i16::from_le_bytes([packet.body[39], packet.body[40]]);
        let gold = u16::from_le_bytes([packet.body[41], packet.body[42]]);
        let current_room = u16::from_le_bytes([packet.body[43], packet.body[44]]);
        let description_len = u16::from_le_bytes([packet.body[45], packet.body[46]]);
        let description = String::from_utf8_lossy(&packet.body[47..]).into();

        Self {
            author: Some(packet.stream.clone()),
            packet_type: packet.packet_type,
            name: Arc::from(name),
            flags,
            attack,
            defense,
            regen,
            health,
            gold,
            current_room,
            description_len,
            description,
        }
    }
}
