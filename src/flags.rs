use bitflags::bitflags;
use serde::{Deserialize, Serialize};

bitflags! {
    #[derive(Default, Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash)]
    /// Flags representing the state of a character in the game.
    ///
    /// When a client uses [`PktType::CHARACTER`] to describe a new player, the server may (should) ignore the client's initial specification for flags, health, gold, etc.
    /// using [`CharacterFlags::reset()`].
    /// > Since the character packet is shared between players and monsters, the server is responsible for setting these values correctly.
    pub struct CharacterFlags: u8 {
        /// The character is alive.
        const ALIVE = 0b1000_0000;
        /// The character will automatically join battles in the room they are in.
        const BATTLE = 0b0100_0000;
        /// The character is a monster.
        const MONSTER = 0b0010_0000;
        /// The character has started.
        const STARTED = 0b0001_0000;
        /// The character is ready.
        const READY = 0b0000_1000;
    }
}

impl CharacterFlags {
    /// Check if the character is alive.
    pub fn is_alive(&self) -> bool {
        self.contains(CharacterFlags::ALIVE)
    }

    /// Check if the character is a monster.
    pub fn is_battle(&self) -> bool {
        self.contains(CharacterFlags::BATTLE)
    }

    /// Check if the character is a monster.
    pub fn is_started(&self) -> bool {
        self.contains(CharacterFlags::STARTED)
    }

    /// Check if the character is ready.
    pub fn is_ready(&self) -> bool {
        self.contains(CharacterFlags::READY)
    }

    /// Kill a character, making them unplayable until they rejoin.
    pub fn dead() -> Self {
        CharacterFlags::BATTLE | CharacterFlags::READY
    }

    /// Bring a character back to life, ready to play.
    pub fn alive() -> Self {
        CharacterFlags::ALIVE | CharacterFlags::BATTLE | CharacterFlags::READY
    }

    /// Reset a character when first starting or respawning.
    pub fn reset() -> Self {
        CharacterFlags::ALIVE | CharacterFlags::BATTLE
    }
}
