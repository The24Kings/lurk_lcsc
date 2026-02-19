use serde::{Deserialize, Serialize};

/// Represents possible error codes for the Lurk protocol.
#[derive(Default, Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum LurkError {
    #[default]
    /// Not covered by any other error codes
    OTHER,
    /// Attempt to change to an inappropriate room
    BADROOM,
    /// Attempt to create a player that already exists.
    PLAYEREXISTS,
    /// Attempt to loot a nonexistent or not present monster.
    BADMONSTER,
    /// Caused by setting inappropriate player stats.
    STATERROR,
    /// Caused by attempting an action too early, for example changing rooms before sending `PktType::START` or `PktType::CHARACTER`.
    NOTREADY,
    /// Sent in response to attempts to loot nonexistent players, fight players in different rooms, etc.
    NOTARGET,
    /// Sent if the requested fight cannot happen for other reasons (i.e. no live monsters in room)
    NOFIGHT,
    /// No player vs. player combat on the server. Servers do not have to support player-vs-player combat.
    NOPLAYERCOMBAT,
}
impl From<LurkError> for u8 {
    /// Converts a `LurkError` enum variant into its corresponding `u8` value.
    ///     
    /// ```rust
    /// use lurk_lcsc::lurk_error::LurkError;
    ///
    /// let err = LurkError::BADROOM;
    /// let err_u8: u8 = err.into();
    /// assert_eq!(err_u8, 1);
    /// ```
    fn from(e: LurkError) -> Self {
        e as u8
    }
}

impl From<u8> for LurkError {
    /// Converts a `u8` value into its corresponding `LurkError` enum variant.
    ///
    /// ```rust
    /// use lurk_lcsc::lurk_error::LurkError;
    ///
    /// let err = LurkError::from(1u8);
    /// assert_eq!(err, LurkError::BADROOM);
    /// ```
    fn from(value: u8) -> Self {
        match value {
            0 => LurkError::OTHER,
            1 => LurkError::BADROOM,
            2 => LurkError::PLAYEREXISTS,
            3 => LurkError::BADMONSTER,
            4 => LurkError::STATERROR,
            5 => LurkError::NOTREADY,
            6 => LurkError::NOTARGET,
            7 => LurkError::NOFIGHT,
            8 => LurkError::NOPLAYERCOMBAT,
            _ => LurkError::OTHER, // Default case
        }
    }
}

impl std::fmt::Display for LurkError {
    /// Formats the `LurkError` enum variant as a human-readable string.
    ///
    /// ```rust
    /// use lurk_lcsc::lurk_error::LurkError;
    /// let err = LurkError::BADROOM;
    /// assert_eq!(format!("{}", err), "BadRoom");
    /// ```
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            LurkError::OTHER => write!(f, "Other"),
            LurkError::BADROOM => write!(f, "BadRoom"),
            LurkError::PLAYEREXISTS => write!(f, "PlayerExists"),
            LurkError::BADMONSTER => write!(f, "BadMonster"),
            LurkError::STATERROR => write!(f, "StatError"),
            LurkError::NOTREADY => write!(f, "NotReady"),
            LurkError::NOTARGET => write!(f, "NoTarget"),
            LurkError::NOFIGHT => write!(f, "NoFight"),
            LurkError::NOPLAYERCOMBAT => write!(f, "NoPlayerCombat"),
        }
    }
}
