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
    /// use lurk_protocol::lurk_error::LurkError;
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
    /// use lurk_protocol::lurk_error::LurkError;
    ///
    /// let err = LurkError::from(1u8);
    /// assert_eq!(err, LurkError::BADROOM);
    /// ```
    fn from(value: u8) -> Self {
        match value {
            1 => LurkError::BADROOM,
            2 => LurkError::PLAYEREXISTS,
            3 => LurkError::BADMONSTER,
            4 => LurkError::STATERROR,
            5 => LurkError::NOTREADY,
            6 => LurkError::NOTARGET,
            7 => LurkError::NOFIGHT,
            8 => LurkError::NOPLAYERCOMBAT,
            _ => LurkError::OTHER, // Covers any unknown value
        }
    }
}

impl std::fmt::Display for LurkError {
    /// Formats the `LurkError` enum variant as a human-readable string.
    ///
    /// ```rust
    /// use lurk_protocol::lurk_error::LurkError;
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

#[cfg(test)]
mod tests {
    use super::*;

    // ── From<u8> for LurkError ────────────────────────────────────────
    #[test]
    fn from_u8_0_is_other() {
        assert_eq!(LurkError::from(0u8), LurkError::OTHER);
    }

    #[test]
    fn from_u8_1_is_badroom() {
        assert_eq!(LurkError::from(1u8), LurkError::BADROOM);
    }

    #[test]
    fn from_u8_2_is_playerexists() {
        assert_eq!(LurkError::from(2u8), LurkError::PLAYEREXISTS);
    }

    #[test]
    fn from_u8_3_is_badmonster() {
        assert_eq!(LurkError::from(3u8), LurkError::BADMONSTER);
    }

    #[test]
    fn from_u8_4_is_staterror() {
        assert_eq!(LurkError::from(4u8), LurkError::STATERROR);
    }

    #[test]
    fn from_u8_5_is_notready() {
        assert_eq!(LurkError::from(5u8), LurkError::NOTREADY);
    }

    #[test]
    fn from_u8_6_is_notarget() {
        assert_eq!(LurkError::from(6u8), LurkError::NOTARGET);
    }

    #[test]
    fn from_u8_7_is_nofight() {
        assert_eq!(LurkError::from(7u8), LurkError::NOFIGHT);
    }

    #[test]
    fn from_u8_8_is_noplayercombat() {
        assert_eq!(LurkError::from(8u8), LurkError::NOPLAYERCOMBAT);
    }

    #[test]
    fn from_u8_unknown_defaults_to_other() {
        assert_eq!(LurkError::from(9u8), LurkError::OTHER);
        assert_eq!(LurkError::from(255u8), LurkError::OTHER);
    }

    // ── From<LurkError> for u8 ───────────────────────────────────────
    #[test]
    fn lurkerror_to_u8_roundtrip() {
        for i in 0u8..=8u8 {
            let err = LurkError::from(i);
            let back: u8 = err.into();
            assert_eq!(back, i, "roundtrip failed for {}", i);
        }
    }

    // ── Display ──────────────────────────────────────────────────────
    #[test]
    fn display_all_variants() {
        assert_eq!(format!("{}", LurkError::OTHER), "Other");
        assert_eq!(format!("{}", LurkError::BADROOM), "BadRoom");
        assert_eq!(format!("{}", LurkError::PLAYEREXISTS), "PlayerExists");
        assert_eq!(format!("{}", LurkError::BADMONSTER), "BadMonster");
        assert_eq!(format!("{}", LurkError::STATERROR), "StatError");
        assert_eq!(format!("{}", LurkError::NOTREADY), "NotReady");
        assert_eq!(format!("{}", LurkError::NOTARGET), "NoTarget");
        assert_eq!(format!("{}", LurkError::NOFIGHT), "NoFight");
        assert_eq!(format!("{}", LurkError::NOPLAYERCOMBAT), "NoPlayerCombat");
    }
}
