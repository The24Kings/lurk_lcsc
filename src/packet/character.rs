use serde::{Deserialize, Serialize};
use std::{io::Write, sync::Arc};

use crate::Packet;
use crate::Parser;
use crate::flags::CharacterFlags;
use crate::packet::PktType;

#[derive(Clone, Serialize, Deserialize)]
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
        if let Err(e) = $crate::send_to($stream.as_ref(), &$player) {
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
    fn write_to<W: Write>(&self, writer: &mut W) -> Result<(), std::io::Error> {
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

    fn decode(packet: Packet) -> Self {
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

#[cfg(test)]
mod tests {
    use crate::test_common;

    use super::*;

    #[test]
    fn character_parse_and_serialize() {
        let stream = test_common::setup();
        let type_byte = PktType::CHARACTER;
        let original_bytes: &[u8; 74] = &[
            0x0a, 0x4c, 0x61, 0x64, 0x61, 0x77, 0x6e, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x64, 0x00, 0x00, 0x00, 0x00, 0x00, 0x64, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x1a, 0x00, 0x41, 0x75, 0x74, 0x6f, 0x2d, 0x67, 0x65, 0x6e,
            0x65, 0x72, 0x61, 0x74, 0x65, 0x64, 0x20, 0x74, 0x65, 0x73, 0x74, 0x20, 0x70, 0x6c,
            0x61, 0x79, 0x65, 0x72,
        ];

        // Create a packet with known bytes, excluding the type byte
        let packet = Packet::new(&stream, type_byte, &original_bytes[1..]);

        // Deserialize the packet into a PktCharacter
        let message = PktCharacter::decode(packet);

        // Assert the fields were parsed correctly
        assert_eq!(message.packet_type, PktType::CHARACTER);
        assert_eq!(message.name.as_ref(), "Ladawn");
        assert!(message.flags.is_empty());
        assert_eq!(message.attack, 100);
        assert_eq!(message.defense, 0);
        assert_eq!(message.regen, 0);
        assert_eq!(message.health, 100);
        assert_eq!(message.gold, 0);
        assert_eq!(message.description_len, 26);
        assert_eq!(message.description.as_ref(), "Auto-generated test player");

        // Serialize the message back into bytes
        let mut buffer: Vec<u8> = Vec::new();
        message.write_to(&mut buffer).expect("Encoding failed");

        // Assert that the serialized bytes match the original
        assert_eq!(buffer, original_bytes);
        assert_eq!(buffer[0], u8::from(type_byte));
    }

    /// Parse a character with ALIVE | BATTLE | READY flags (from trace: Test initial).
    #[test]
    fn character_parse_trace_alive_battle_ready() {
        let stream = test_common::setup();
        // Character "Test" with flags=0xC8 (ALIVE|BATTLE|READY), attack=33, defense=33, regen=34,
        // health=0, gold=0, room=0, desc_len=0, no description
        let mut body: Vec<u8> = Vec::new();
        // Name: "Test" padded to 32 bytes
        let mut name = b"Test".to_vec();
        name.resize(32, 0x00);
        body.extend(&name);
        body.push(0xC8); // flags: ALIVE(0x80) | BATTLE(0x40) | READY(0x08)
        body.extend(33u16.to_le_bytes()); // attack
        body.extend(33u16.to_le_bytes()); // defense
        body.extend(34u16.to_le_bytes()); // regen
        body.extend(0i16.to_le_bytes()); // health
        body.extend(0u16.to_le_bytes()); // gold
        body.extend(0u16.to_le_bytes()); // current_room
        body.extend(0u16.to_le_bytes()); // description_len

        let packet = Packet::new(&stream, PktType::CHARACTER, &body);
        let chr = PktCharacter::decode(packet);

        assert_eq!(chr.name.as_ref(), "Test");
        assert!(chr.flags.contains(CharacterFlags::ALIVE));
        assert!(chr.flags.contains(CharacterFlags::BATTLE));
        assert!(chr.flags.contains(CharacterFlags::READY));
        assert!(!chr.flags.contains(CharacterFlags::STARTED));
        assert!(!chr.flags.contains(CharacterFlags::MONSTER));
        assert_eq!(chr.attack, 33);
        assert_eq!(chr.defense, 33);
        assert_eq!(chr.regen, 34);
        assert_eq!(chr.health, 0);
        assert_eq!(chr.gold, 0);
        assert_eq!(chr.current_room, 0);
        assert_eq!(chr.description_len, 0);
        assert_eq!(chr.description.as_ref(), "");
    }

    /// Parse a character with ALIVE | BATTLE | STARTED | READY flags (from trace: Test started).
    #[test]
    fn character_parse_trace_started() {
        let stream = test_common::setup();
        let mut body: Vec<u8> = Vec::new();
        let mut name = b"Test".to_vec();
        name.resize(32, 0x00);
        body.extend(&name);
        body.push(0xD8); // flags: ALIVE(0x80) | BATTLE(0x40) | STARTED(0x10) | READY(0x08)
        body.extend(33u16.to_le_bytes());
        body.extend(33u16.to_le_bytes());
        body.extend(34u16.to_le_bytes());
        body.extend(100i16.to_le_bytes()); // health = 100
        body.extend(0u16.to_le_bytes());
        body.extend(0u16.to_le_bytes());
        body.extend(0u16.to_le_bytes());

        let packet = Packet::new(&stream, PktType::CHARACTER, &body);
        let chr = PktCharacter::decode(packet);

        assert!(chr.flags.contains(CharacterFlags::ALIVE));
        assert!(chr.flags.contains(CharacterFlags::BATTLE));
        assert!(chr.flags.contains(CharacterFlags::STARTED));
        assert!(chr.flags.contains(CharacterFlags::READY));
        assert_eq!(chr.health, 100);
    }

    /// Parse a monster character from trace bytes (Deku Baba, alive).
    #[test]
    fn character_parse_trace_monster_alive() {
        let stream = test_common::setup();
        let mut body: Vec<u8> = Vec::new();
        let mut name = b"Deku Baba".to_vec();
        name.resize(32, 0x00);
        body.extend(&name);
        body.push(0xE8); // ALIVE(0x80) | BATTLE(0x40) | MONSTER(0x20) | READY(0x08)
        body.extend(3u16.to_le_bytes()); // attack
        body.extend(1u16.to_le_bytes()); // defense
        body.extend(0u16.to_le_bytes()); // regen
        body.extend(8i16.to_le_bytes()); // health
        body.extend(2u16.to_le_bytes()); // gold
        body.extend(0u16.to_le_bytes()); // current_room
        let desc = "A snapping plant that lunges when approached. Can drop Deku sticks or nuts when defeated.";
        body.extend((desc.len() as u16).to_le_bytes());
        body.extend(desc.as_bytes());

        let packet = Packet::new(&stream, PktType::CHARACTER, &body);
        let chr = PktCharacter::decode(packet);

        assert_eq!(chr.name.as_ref(), "Deku Baba");
        assert!(chr.flags.contains(CharacterFlags::ALIVE));
        assert!(chr.flags.contains(CharacterFlags::BATTLE));
        assert!(chr.flags.contains(CharacterFlags::MONSTER));
        assert!(chr.flags.contains(CharacterFlags::READY));
        assert_eq!(chr.attack, 3);
        assert_eq!(chr.defense, 1);
        assert_eq!(chr.regen, 0);
        assert_eq!(chr.health, 8);
        assert_eq!(chr.gold, 2);
        assert_eq!(chr.description.as_ref(), desc);
    }

    /// Parse a dead monster from trace: negative health.
    #[test]
    fn character_parse_trace_monster_dead() {
        let stream = test_common::setup();
        let mut body: Vec<u8> = Vec::new();
        let mut name = b"Deku Baba".to_vec();
        name.resize(32, 0x00);
        body.extend(&name);
        body.push(0x68); // BATTLE(0x40) | MONSTER(0x20) | READY(0x08) -- no ALIVE
        body.extend(3u16.to_le_bytes());
        body.extend(1u16.to_le_bytes());
        body.extend(0u16.to_le_bytes());
        body.extend((-57i16).to_le_bytes()); // negative health
        body.extend(2u16.to_le_bytes());
        body.extend(0u16.to_le_bytes());
        let desc = "A snapping plant that lunges when approached. Can drop Deku sticks or nuts when defeated.";
        body.extend((desc.len() as u16).to_le_bytes());
        body.extend(desc.as_bytes());

        let packet = Packet::new(&stream, PktType::CHARACTER, &body);
        let chr = PktCharacter::decode(packet);

        assert!(!chr.flags.contains(CharacterFlags::ALIVE));
        assert!(chr.flags.contains(CharacterFlags::MONSTER));
        assert_eq!(chr.health, -57);
    }

    /// Character with max stat values.
    #[test]
    fn character_max_stats() {
        let stream = test_common::setup();
        let mut body: Vec<u8> = Vec::new();
        let mut name = b"MaxStats".to_vec();
        name.resize(32, 0x00);
        body.extend(&name);
        body.push(0xFF); // All flag bits set
        body.extend(u16::MAX.to_le_bytes()); // attack
        body.extend(u16::MAX.to_le_bytes()); // defense
        body.extend(u16::MAX.to_le_bytes()); // regen
        body.extend(i16::MAX.to_le_bytes()); // health
        body.extend(u16::MAX.to_le_bytes()); // gold
        body.extend(u16::MAX.to_le_bytes()); // current_room
        body.extend(0u16.to_le_bytes()); // description_len

        let packet = Packet::new(&stream, PktType::CHARACTER, &body);
        let chr = PktCharacter::decode(packet);

        assert_eq!(chr.attack, u16::MAX);
        assert_eq!(chr.defense, u16::MAX);
        assert_eq!(chr.regen, u16::MAX);
        assert_eq!(chr.health, i16::MAX);
        assert_eq!(chr.gold, u16::MAX);
        assert_eq!(chr.current_room, u16::MAX);
    }

    /// Character with minimum (negative) health.
    #[test]
    fn character_min_health() {
        let stream = test_common::setup();
        let mut body: Vec<u8> = Vec::new();
        let mut name = b"MinHP".to_vec();
        name.resize(32, 0x00);
        body.extend(&name);
        body.push(0x00); // no flags
        body.extend(0u16.to_le_bytes());
        body.extend(0u16.to_le_bytes());
        body.extend(0u16.to_le_bytes());
        body.extend(i16::MIN.to_le_bytes()); // -32768
        body.extend(0u16.to_le_bytes());
        body.extend(0u16.to_le_bytes());
        body.extend(0u16.to_le_bytes());

        let packet = Packet::new(&stream, PktType::CHARACTER, &body);
        let chr = PktCharacter::decode(packet);

        assert_eq!(chr.health, i16::MIN);
    }

    /// Character with all zero stats.
    #[test]
    fn character_all_zeros() {
        let stream = test_common::setup();
        let body: Vec<u8> = vec![0x00; 47]; // 32 name + 1 flags + 14 stats = 47

        let packet = Packet::new(&stream, PktType::CHARACTER, &body);
        let chr = PktCharacter::decode(packet);

        assert_eq!(chr.name.as_ref(), "");
        assert!(chr.flags.is_empty());
        assert_eq!(chr.attack, 0);
        assert_eq!(chr.defense, 0);
        assert_eq!(chr.regen, 0);
        assert_eq!(chr.health, 0);
        assert_eq!(chr.gold, 0);
        assert_eq!(chr.current_room, 0);
        assert_eq!(chr.description_len, 0);
    }

    /// Character name that fills all 32 bytes (no null terminator padding).
    #[test]
    fn character_max_length_name() {
        let stream = test_common::setup();
        let long_name = "A".repeat(32);
        let mut body: Vec<u8> = Vec::new();
        body.extend(long_name.as_bytes());
        body.push(0xC8);
        body.extend(10u16.to_le_bytes());
        body.extend(10u16.to_le_bytes());
        body.extend(10u16.to_le_bytes());
        body.extend(100i16.to_le_bytes());
        body.extend(50u16.to_le_bytes());
        body.extend(1u16.to_le_bytes());
        body.extend(0u16.to_le_bytes());

        let packet = Packet::new(&stream, PktType::CHARACTER, &body);
        let chr = PktCharacter::decode(packet);

        assert_eq!(chr.name.as_ref(), &long_name);
    }

    /// Roundtrip: construct, serialize, then deserialize and verify.
    #[test]
    fn character_roundtrip() {
        let stream = test_common::setup();
        let original = PktCharacter {
            packet_type: PktType::CHARACTER,
            name: Arc::from("TestHero"),
            flags: CharacterFlags::ALIVE
                | CharacterFlags::BATTLE
                | CharacterFlags::STARTED
                | CharacterFlags::READY,
            attack: 50,
            defense: 25,
            regen: 25,
            health: 100,
            gold: 42,
            current_room: 7,
            description_len: 11,
            description: Box::from("A test hero"),
        };

        let mut buffer: Vec<u8> = Vec::new();
        original.write_to(&mut buffer).expect("Encoding failed");

        let packet = Packet::new(&stream, PktType::CHARACTER, &buffer[1..]);
        let deserialized = PktCharacter::decode(packet);

        assert_eq!(deserialized.name.as_ref(), "TestHero");
        assert_eq!(
            deserialized.flags,
            CharacterFlags::ALIVE
                | CharacterFlags::BATTLE
                | CharacterFlags::STARTED
                | CharacterFlags::READY
        );
        assert_eq!(deserialized.attack, 50);
        assert_eq!(deserialized.defense, 25);
        assert_eq!(deserialized.regen, 25);
        assert_eq!(deserialized.health, 100);
        assert_eq!(deserialized.gold, 42);
        assert_eq!(deserialized.current_room, 7);
        assert_eq!(deserialized.description.as_ref(), "A test hero");
    }

    /// Non-UTF8 bytes in name should be handled by lossy conversion.
    #[test]
    fn character_non_utf8_name() {
        let stream = test_common::setup();
        let mut body: Vec<u8> = Vec::new();
        let mut name = vec![0xFF, 0xFE, 0xFD, 0xFC];
        name.resize(32, 0x00);
        body.extend(&name);
        body.push(0x00);
        body.extend(vec![0u8; 14]); // stats

        let packet = Packet::new(&stream, PktType::CHARACTER, &body);
        let chr = PktCharacter::decode(packet);

        // Should contain replacement characters
        assert!(chr.name.contains('\u{FFFD}'));
    }

    /// Body that is too short to parse stats should panic.
    #[test]
    #[should_panic]
    fn character_body_too_short_panics() {
        let stream = test_common::setup();
        let body: &[u8] = &[0x41; 20]; // Only 20 bytes, need at least 47
        let packet = Packet::new(&stream, PktType::CHARACTER, body);
        let _ = PktCharacter::decode(packet);
    }

    /// Empty body should panic.
    #[test]
    #[should_panic]
    fn character_empty_body_panics() {
        let stream = test_common::setup();
        let body: &[u8] = &[];
        let packet = Packet::new(&stream, PktType::CHARACTER, body);
        let _ = PktCharacter::decode(packet);
    }

    /// Verify CharacterFlags helper methods work.
    #[test]
    fn character_flags_helpers() {
        let alive = CharacterFlags::alive();
        assert!(alive.is_alive());
        assert!(alive.is_battle());
        assert!(alive.is_ready());
        assert!(!alive.is_started());

        let dead = CharacterFlags::dead();
        assert!(!dead.is_alive());
        assert!(dead.is_battle());
        assert!(dead.is_ready());

        let reset = CharacterFlags::reset();
        assert!(reset.is_alive());
        assert!(reset.is_battle());
        assert!(!reset.is_ready());
        assert!(!reset.is_started());
    }

    /// Verify with_defaults_from resets health, gold, room, and flags correctly.
    #[test]
    fn character_with_defaults_from() {
        let incoming = PktCharacter {
            packet_type: PktType::CHARACTER,
            name: Arc::from("Player1"),
            flags: CharacterFlags::ALIVE
                | CharacterFlags::BATTLE
                | CharacterFlags::STARTED
                | CharacterFlags::READY,
            attack: 50,
            defense: 25,
            regen: 25,
            health: 999,
            gold: 999,
            current_room: 42,
            description_len: 0,
            description: Box::from(""),
        };

        let defaulted = PktCharacter::with_defaults_from(&incoming);
        assert_eq!(defaulted.name.as_ref(), "Player1");
        assert_eq!(defaulted.attack, 50);
        assert_eq!(defaulted.defense, 25);
        assert_eq!(defaulted.regen, 25);
        assert_eq!(defaulted.health, 100);
        assert_eq!(defaulted.gold, 0);
        assert_eq!(defaulted.current_room, 0);
        assert_eq!(defaulted.flags, CharacterFlags::reset());
    }

    /// All 0xFF body should parse with truncated flags.
    #[test]
    fn character_all_ones_body() {
        let stream = test_common::setup();
        let body: Vec<u8> = vec![0xFF; 47];

        let packet = Packet::new(&stream, PktType::CHARACTER, &body);
        let chr = PktCharacter::decode(packet);

        // Flags are truncated to known bits
        assert!(chr.flags.contains(CharacterFlags::ALIVE));
        assert!(chr.flags.contains(CharacterFlags::BATTLE));
        assert!(chr.flags.contains(CharacterFlags::MONSTER));
        assert!(chr.flags.contains(CharacterFlags::STARTED));
        assert!(chr.flags.contains(CharacterFlags::READY));

        assert_eq!(chr.attack, u16::MAX);
        assert_eq!(chr.defense, u16::MAX);
        assert_eq!(chr.regen, u16::MAX);
        assert_eq!(chr.health, -1); // 0xFFFF as i16
        assert_eq!(chr.gold, u16::MAX);
        assert_eq!(chr.current_room, u16::MAX);
    }

    /// Display/JSON output should be valid JSON.
    #[test]
    fn character_display_valid_json() {
        let chr = PktCharacter {
            packet_type: PktType::CHARACTER,
            name: Arc::from("TestChar"),
            flags: CharacterFlags::ALIVE | CharacterFlags::BATTLE,
            attack: 10,
            defense: 10,
            regen: 10,
            health: 100,
            gold: 0,
            current_room: 0,
            description_len: 0,
            description: Box::from(""),
        };
        let json_str = format!("{}", chr);
        let parsed: serde_json::Value = serde_json::from_str(&json_str).expect("Invalid JSON");
        assert_eq!(parsed["name"], "TestChar");
        assert_eq!(parsed["attack"], 10);
    }

    /// Verify health underflow at i16 boundary values.
    #[test]
    fn character_health_boundary_values() {
        let stream = test_common::setup();

        for &health in &[
            0i16,
            1,
            -1,
            100,
            -100,
            i16::MAX,
            i16::MIN,
            i16::MIN + 1,
            i16::MAX - 1,
        ] {
            let mut body: Vec<u8> = Vec::new();
            let mut name = b"BoundaryTest".to_vec();
            name.resize(32, 0x00);
            body.extend(&name);
            body.push(0x00);
            body.extend(0u16.to_le_bytes()); // attack
            body.extend(0u16.to_le_bytes()); // defense
            body.extend(0u16.to_le_bytes()); // regen
            body.extend(health.to_le_bytes());
            body.extend(0u16.to_le_bytes()); // gold
            body.extend(0u16.to_le_bytes()); // room
            body.extend(0u16.to_le_bytes()); // desc_len

            let packet = Packet::new(&stream, PktType::CHARACTER, &body);
            let chr = PktCharacter::decode(packet);
            assert_eq!(chr.health, health, "Failed for health value: {}", health);
        }
    }

    /// Gold at various u16 boundary values.
    #[test]
    fn character_gold_boundary_values() {
        let stream = test_common::setup();

        for &gold in &[0u16, 1, 100, 1000, u16::MAX, u16::MAX - 1] {
            let mut body: Vec<u8> = Vec::new();
            let mut name = b"GoldTest".to_vec();
            name.resize(32, 0x00);
            body.extend(&name);
            body.push(0x00);
            body.extend(0u16.to_le_bytes());
            body.extend(0u16.to_le_bytes());
            body.extend(0u16.to_le_bytes());
            body.extend(0i16.to_le_bytes());
            body.extend(gold.to_le_bytes());
            body.extend(0u16.to_le_bytes());
            body.extend(0u16.to_le_bytes());

            let packet = Packet::new(&stream, PktType::CHARACTER, &body);
            let chr = PktCharacter::decode(packet);
            assert_eq!(chr.gold, gold, "Failed for gold value: {}", gold);
        }
    }
}
////////////////////////////////////////////////////////////////////////////////
