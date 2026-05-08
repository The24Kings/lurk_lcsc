use serde::{Deserialize, Serialize};

/// Represents the different types of packets used in the application.
#[derive(Default, Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum PktType {
    #[default]
    /// The default packet type, used as a fallback or uninitialized value.
    DEFAULT,
    /// Represents a message packet, typically used for communication between players.
    MESSAGE,
    /// Represents a request to change the room.
    CHANGEROOM,
    /// Represents a fight packet to fight monsters.
    FIGHT,
    /// Represents a player-versus-player fight packet.
    PVPFIGHT,
    /// Represents a loot packet.
    LOOT,
    /// Represents a start packet.
    START,
    /// Represents an error packet.
    ERROR,
    /// Represents an accept packet.
    ACCEPT,
    /// Represents a room packet.
    ROOM,
    /// Represents a character packet.
    CHARACTER,
    /// Represents a game packet.
    GAME,
    /// Represents a leave packet.
    LEAVE,
    /// Represents a connection packet.
    CONNECTION,
    /// Represents a version packet.
    VERSION,
}

impl From<PktType> for u8 {
    /// Converts a `PktType` enum variant into its corresponding `u8` value.
    ///
    /// ```rust
    /// use lurk_protocol::pkt_type::PktType;
    ///
    /// let pkt = PktType::MESSAGE;
    /// let pkt_u8: u8 = pkt.into();
    /// assert_eq!(pkt_u8, 1);
    ///
    /// let pkt2 = PktType::from(1u8);
    /// assert_eq!(pkt2, PktType::MESSAGE);
    /// ```
    fn from(val: PktType) -> Self {
        val as u8
    }
}

impl From<u8> for PktType {
    /// Converts a `u8` value into its corresponding `PktType` enum variant.
    ///
    /// ```rust
    /// use lurk_protocol::pkt_type::PktType;
    ///
    /// let pkt_type = PktType::from(3u8);
    /// assert_eq!(pkt_type, PktType::FIGHT);
    /// ```
    fn from(value: u8) -> Self {
        match value {
            1 => PktType::MESSAGE,
            2 => PktType::CHANGEROOM,
            3 => PktType::FIGHT,
            4 => PktType::PVPFIGHT,
            5 => PktType::LOOT,
            6 => PktType::START,
            7 => PktType::ERROR,
            8 => PktType::ACCEPT,
            9 => PktType::ROOM,
            10 => PktType::CHARACTER,
            11 => PktType::GAME,
            12 => PktType::LEAVE,
            13 => PktType::CONNECTION,
            14 => PktType::VERSION,
            _ => PktType::DEFAULT, // Covers any unknown value
        }
    }
}

impl From<&[u8; 1]> for PktType {
    /// Converts a byte slice into its corresponding `PktType` enum variant.
    ///
    /// ```rust
    /// use lurk_protocol::pkt_type::PktType;
    ///
    /// let bytes = [3u8; 1];
    /// let pkt_type = PktType::from(&bytes);
    /// assert_eq!(pkt_type, PktType::FIGHT);
    /// ```
    fn from(value: &[u8; 1]) -> Self {
        match value[0] {
            1 => PktType::MESSAGE,
            2 => PktType::CHANGEROOM,
            3 => PktType::FIGHT,
            4 => PktType::PVPFIGHT,
            5 => PktType::LOOT,
            6 => PktType::START,
            7 => PktType::ERROR,
            8 => PktType::ACCEPT,
            9 => PktType::ROOM,
            10 => PktType::CHARACTER,
            11 => PktType::GAME,
            12 => PktType::LEAVE,
            13 => PktType::CONNECTION,
            14 => PktType::VERSION,
            _ => PktType::DEFAULT, // Covers 0 and any unknown value
        }
    }
}

impl std::fmt::Display for PktType {
    /// Formats the `PktType` enum variant as a human-readable string.
    /// # Example
    /// ```rust
    /// use lurk_protocol::pkt_type::PktType;
    ///
    /// let pkt = PktType::FIGHT;
    /// assert_eq!(format!("{}", pkt), "Fight");
    /// ```
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PktType::DEFAULT => write!(f, "Default"),
            PktType::MESSAGE => write!(f, "Message"),
            PktType::CHANGEROOM => write!(f, "ChangeRoom"),
            PktType::FIGHT => write!(f, "Fight"),
            PktType::PVPFIGHT => write!(f, "PVPFight"),
            PktType::LOOT => write!(f, "Loot"),
            PktType::START => write!(f, "Start"),
            PktType::ERROR => write!(f, "Error"),
            PktType::ACCEPT => write!(f, "Accept"),
            PktType::ROOM => write!(f, "Room"),
            PktType::CHARACTER => write!(f, "Character"),
            PktType::GAME => write!(f, "Game"),
            PktType::LEAVE => write!(f, "Leave"),
            PktType::CONNECTION => write!(f, "Connection"),
            PktType::VERSION => write!(f, "Version"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── From<PktType> for u8 ─────────────────────────────────────────
    #[test]
    fn pkttype_to_u8_all_variants() {
        assert_eq!(u8::from(PktType::DEFAULT), 0);
        assert_eq!(u8::from(PktType::MESSAGE), 1);
        assert_eq!(u8::from(PktType::CHANGEROOM), 2);
        assert_eq!(u8::from(PktType::FIGHT), 3);
        assert_eq!(u8::from(PktType::PVPFIGHT), 4);
        assert_eq!(u8::from(PktType::LOOT), 5);
        assert_eq!(u8::from(PktType::START), 6);
        assert_eq!(u8::from(PktType::ERROR), 7);
        assert_eq!(u8::from(PktType::ACCEPT), 8);
        assert_eq!(u8::from(PktType::ROOM), 9);
        assert_eq!(u8::from(PktType::CHARACTER), 10);
        assert_eq!(u8::from(PktType::GAME), 11);
        assert_eq!(u8::from(PktType::LEAVE), 12);
        assert_eq!(u8::from(PktType::CONNECTION), 13);
        assert_eq!(u8::from(PktType::VERSION), 14);
    }

    // ── From<u8> for PktType ─────────────────────────────────────────
    #[test]
    fn from_u8_0_is_default() {
        assert_eq!(PktType::from(0u8), PktType::DEFAULT);
    }

    #[test]
    fn from_u8_1_is_message() {
        assert_eq!(PktType::from(1u8), PktType::MESSAGE);
    }

    #[test]
    fn from_u8_2_is_changeroom() {
        assert_eq!(PktType::from(2u8), PktType::CHANGEROOM);
    }

    #[test]
    fn from_u8_3_is_fight() {
        assert_eq!(PktType::from(3u8), PktType::FIGHT);
    }

    #[test]
    fn from_u8_4_is_pvpfight() {
        assert_eq!(PktType::from(4u8), PktType::PVPFIGHT);
    }

    #[test]
    fn from_u8_5_is_loot() {
        assert_eq!(PktType::from(5u8), PktType::LOOT);
    }

    #[test]
    fn from_u8_6_is_start() {
        assert_eq!(PktType::from(6u8), PktType::START);
    }

    #[test]
    fn from_u8_7_is_error() {
        assert_eq!(PktType::from(7u8), PktType::ERROR);
    }

    #[test]
    fn from_u8_8_is_accept() {
        assert_eq!(PktType::from(8u8), PktType::ACCEPT);
    }

    #[test]
    fn from_u8_9_is_room() {
        assert_eq!(PktType::from(9u8), PktType::ROOM);
    }

    #[test]
    fn from_u8_10_is_character() {
        assert_eq!(PktType::from(10u8), PktType::CHARACTER);
    }

    #[test]
    fn from_u8_11_is_game() {
        assert_eq!(PktType::from(11u8), PktType::GAME);
    }

    #[test]
    fn from_u8_12_is_leave() {
        assert_eq!(PktType::from(12u8), PktType::LEAVE);
    }

    #[test]
    fn from_u8_13_is_connection() {
        assert_eq!(PktType::from(13u8), PktType::CONNECTION);
    }

    #[test]
    fn from_u8_14_is_version() {
        assert_eq!(PktType::from(14u8), PktType::VERSION);
    }

    #[test]
    fn from_u8_unknown_defaults() {
        assert_eq!(PktType::from(15u8), PktType::DEFAULT);
        assert_eq!(PktType::from(255u8), PktType::DEFAULT);
    }

    // ── From<&[u8; 1]> for PktType ──────────────────────────────────
    #[test]
    fn from_byte_array_0_is_default() {
        assert_eq!(PktType::from(&[0u8]), PktType::DEFAULT);
    }

    #[test]
    fn from_byte_array_1_is_message() {
        assert_eq!(PktType::from(&[1u8]), PktType::MESSAGE);
    }

    #[test]
    fn from_byte_array_2_is_changeroom() {
        assert_eq!(PktType::from(&[2u8]), PktType::CHANGEROOM);
    }

    #[test]
    fn from_byte_array_3_is_fight() {
        assert_eq!(PktType::from(&[3u8]), PktType::FIGHT);
    }

    #[test]
    fn from_byte_array_4_is_pvpfight() {
        assert_eq!(PktType::from(&[4u8]), PktType::PVPFIGHT);
    }

    #[test]
    fn from_byte_array_5_is_loot() {
        assert_eq!(PktType::from(&[5u8]), PktType::LOOT);
    }

    #[test]
    fn from_byte_array_6_is_start() {
        assert_eq!(PktType::from(&[6u8]), PktType::START);
    }

    #[test]
    fn from_byte_array_7_is_error() {
        assert_eq!(PktType::from(&[7u8]), PktType::ERROR);
    }

    #[test]
    fn from_byte_array_8_is_accept() {
        assert_eq!(PktType::from(&[8u8]), PktType::ACCEPT);
    }

    #[test]
    fn from_byte_array_9_is_room() {
        assert_eq!(PktType::from(&[9u8]), PktType::ROOM);
    }

    #[test]
    fn from_byte_array_10_is_character() {
        assert_eq!(PktType::from(&[10u8]), PktType::CHARACTER);
    }

    #[test]
    fn from_byte_array_11_is_game() {
        assert_eq!(PktType::from(&[11u8]), PktType::GAME);
    }

    #[test]
    fn from_byte_array_12_is_leave() {
        assert_eq!(PktType::from(&[12u8]), PktType::LEAVE);
    }

    #[test]
    fn from_byte_array_13_is_connection() {
        assert_eq!(PktType::from(&[13u8]), PktType::CONNECTION);
    }

    #[test]
    fn from_byte_array_14_is_version() {
        assert_eq!(PktType::from(&[14u8]), PktType::VERSION);
    }

    #[test]
    fn from_byte_array_unknown_defaults() {
        assert_eq!(PktType::from(&[15u8]), PktType::DEFAULT);
        assert_eq!(PktType::from(&[255u8]), PktType::DEFAULT);
    }

    // ── roundtrip u8 ↔ PktType ──────────────────────────────────────
    #[test]
    fn roundtrip_u8_all() {
        for i in 0u8..=14u8 {
            let pkt = PktType::from(i);
            let back: u8 = pkt.into();
            assert_eq!(back, i, "roundtrip failed for {}", i);
        }
    }

    // ── roundtrip &[u8;1] ↔ PktType ────────────────────────────────
    #[test]
    fn roundtrip_byte_array_all() {
        for i in 0u8..=14u8 {
            let arr = [i];
            let pkt = PktType::from(&arr);
            let back: u8 = pkt.into();
            assert_eq!(back, i, "byte-array roundtrip failed for {}", i);
        }
    }

    // ── Display ─────────────────────────────────────────────────────
    #[test]
    fn display_all_variants() {
        assert_eq!(format!("{}", PktType::DEFAULT), "Default");
        assert_eq!(format!("{}", PktType::MESSAGE), "Message");
        assert_eq!(format!("{}", PktType::CHANGEROOM), "ChangeRoom");
        assert_eq!(format!("{}", PktType::FIGHT), "Fight");
        assert_eq!(format!("{}", PktType::PVPFIGHT), "PVPFight");
        assert_eq!(format!("{}", PktType::LOOT), "Loot");
        assert_eq!(format!("{}", PktType::START), "Start");
        assert_eq!(format!("{}", PktType::ERROR), "Error");
        assert_eq!(format!("{}", PktType::ACCEPT), "Accept");
        assert_eq!(format!("{}", PktType::ROOM), "Room");
        assert_eq!(format!("{}", PktType::CHARACTER), "Character");
        assert_eq!(format!("{}", PktType::GAME), "Game");
        assert_eq!(format!("{}", PktType::LEAVE), "Leave");
        assert_eq!(format!("{}", PktType::CONNECTION), "Connection");
        assert_eq!(format!("{}", PktType::VERSION), "Version");
    }
}
