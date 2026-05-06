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
        CharacterFlags::BATTLE.union(CharacterFlags::READY)
    }

    /// Bring a character back to life, ready to play.
    pub fn alive() -> Self {
        CharacterFlags::ALIVE
            .union(CharacterFlags::BATTLE)
            .union(CharacterFlags::READY)
    }

    /// Reset a character when first starting or respawning.
    pub fn reset() -> Self {
        CharacterFlags::ALIVE.union(CharacterFlags::BATTLE)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── is_alive ──────────────────────────────────────────────────────
    #[test]
    fn is_alive_true_when_alive_set() {
        let flags = CharacterFlags::ALIVE;
        assert!(flags.is_alive());
    }

    #[test]
    fn is_alive_false_when_alive_not_set() {
        let flags = CharacterFlags::empty();
        assert!(!flags.is_alive());
    }

    // ── is_battle ─────────────────────────────────────────────────────
    #[test]
    fn is_battle_true_when_battle_set() {
        let flags = CharacterFlags::BATTLE;
        assert!(flags.is_battle());
    }

    #[test]
    fn is_battle_false_when_battle_not_set() {
        let flags = CharacterFlags::empty();
        assert!(!flags.is_battle());
    }

    // ── is_started ────────────────────────────────────────────────────
    #[test]
    fn is_started_true_when_started_set() {
        let flags = CharacterFlags::STARTED;
        assert!(flags.is_started());
    }

    #[test]
    fn is_started_false_when_started_not_set() {
        let flags = CharacterFlags::empty();
        assert!(!flags.is_started());
    }

    // ── is_ready ──────────────────────────────────────────────────────
    #[test]
    fn is_ready_true_when_ready_set() {
        let flags = CharacterFlags::READY;
        assert!(flags.is_ready());
    }

    #[test]
    fn is_ready_false_when_ready_not_set() {
        let flags = CharacterFlags::empty();
        assert!(!flags.is_ready());
    }

    // ── dead() ────────────────────────────────────────────────────────
    #[test]
    fn dead_has_battle_flag() {
        assert!(CharacterFlags::dead().contains(CharacterFlags::BATTLE));
    }

    #[test]
    fn dead_has_ready_flag() {
        assert!(CharacterFlags::dead().contains(CharacterFlags::READY));
    }

    #[test]
    fn dead_does_not_have_alive_flag() {
        assert!(!CharacterFlags::dead().contains(CharacterFlags::ALIVE));
    }

    #[test]
    fn dead_exact_bits() {
        // BATTLE (0b0100_0000) | READY (0b0000_1000) = 0b0100_1000 = 0x48
        assert_eq!(CharacterFlags::dead().bits(), 0b0100_1000);
    }

    // ── alive() ───────────────────────────────────────────────────────
    #[test]
    fn alive_has_alive_flag() {
        assert!(CharacterFlags::alive().contains(CharacterFlags::ALIVE));
    }

    #[test]
    fn alive_has_battle_flag() {
        assert!(CharacterFlags::alive().contains(CharacterFlags::BATTLE));
    }

    #[test]
    fn alive_has_ready_flag() {
        assert!(CharacterFlags::alive().contains(CharacterFlags::READY));
    }

    #[test]
    fn alive_exact_bits() {
        // ALIVE (0x80) | BATTLE (0x40) | READY (0x08) = 0xC8
        assert_eq!(CharacterFlags::alive().bits(), 0b1100_1000);
    }

    // ── reset() ───────────────────────────────────────────────────────
    #[test]
    fn reset_has_alive_flag() {
        assert!(CharacterFlags::reset().contains(CharacterFlags::ALIVE));
    }

    #[test]
    fn reset_has_battle_flag() {
        assert!(CharacterFlags::reset().contains(CharacterFlags::BATTLE));
    }

    #[test]
    fn reset_does_not_have_ready_flag() {
        assert!(!CharacterFlags::reset().contains(CharacterFlags::READY));
    }

    #[test]
    fn reset_exact_bits() {
        // ALIVE (0x80) | BATTLE (0x40) = 0xC0
        assert_eq!(CharacterFlags::reset().bits(), 0b1100_0000);
    }
}
