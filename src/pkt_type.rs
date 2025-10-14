use serde::Serialize;

/// Represents the different types of packets used in the application.
#[derive(Default, Serialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
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
    /// use lurk_lcsc::pkt_type::PktType;
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
    /// # Example
    /// ```rust
    /// use lurk_lcsc::pkt_type::PktType;
    ///
    /// let pkt_type = PktType::from(3u8);
    /// assert_eq!(pkt_type, PktType::FIGHT);
    /// ```
    fn from(value: u8) -> Self {
        match value {
            0 => PktType::DEFAULT,
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
            _ => PktType::DEFAULT,
        }
    }
}

impl std::fmt::Display for PktType {
    /// Formats the `PktType` enum variant as a human-readable string.
    /// # Example
    /// ```rust
    /// use lurk_lcsc::pkt_type::PktType;
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
