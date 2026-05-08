use std::io::Read as _;
use std::io::{Error, ErrorKind};
use std::net::TcpStream;
use std::sync::Arc;

#[cfg(feature = "tracing")]
use tracing::info;

use crate::{
    Packet, Parser, PktAccept, PktChangeRoom, PktCharacter, PktConnection, PktError, PktFight,
    PktGame, PktLeave, PktLoot, PktMessage, PktPVPFight, PktRoom, PktStart, PktType, PktVersion,
};

/// Represents all possible protocol packets exchanged between the client and server.
///
/// Each variant wraps the deserialized packet data as plain Rust structs,
/// providing a pure wire-format translation layer with no connection state.
pub enum Protocol {
    /// Packet containing a message sent between client and server.
    Message(PktMessage),
    /// Packet containing a request to change the room.
    ChangeRoom(PktChangeRoom),
    /// Packet containing a fight request.
    Fight(PktFight),
    /// Packet containing a player-versus-player fight request.
    PVPFight(PktPVPFight),
    /// Packet containing a loot request.
    Loot(PktLoot),
    /// Packet containing a start request.
    Start(PktStart),
    /// Packet containing an error response.
    Error(PktError),
    /// Packet containing an acceptance response.
    Accept(PktAccept),
    /// Packet containing room information.
    Room(PktRoom),
    /// Packet containing character information.
    Character(PktCharacter),
    /// Packet containing game information.
    /// Must be sent __second__ on new connections.
    ///
    /// ```
    /// use lurk_protocol::{Protocol, PktGame, PktType};
    ///
    /// let protocol = Protocol::Game(
    ///     PktGame {
    ///         packet_type: PktType::GAME,
    ///         initial_points: 100,
    ///         stat_limit: 65535,
    ///         description_len: 17,
    ///         description: Box::from("Test Description."),
    ///     },
    /// );
    /// ```
    Game(PktGame),
    /// Packet containing leave information.
    Leave(PktLeave),
    /// Packet containing connection information.
    Connection(PktConnection),
    /// Packet containing version information.
    /// Must be sent __first__ on new connections.
    ///
    /// ```
    /// use lurk_protocol::{Protocol, PktVersion, PktType};
    ///
    /// let protocol = Protocol::Version(
    ///     PktVersion {
    ///         packet_type: PktType::VERSION,
    ///         major_rev: 2,
    ///         minor_rev: 3,
    ///         extensions_len: 0,
    ///         extensions: None,
    ///     },
    /// );
    /// ```
    Version(PktVersion),
}

impl std::fmt::Display for Protocol {
    /// Formats the `Protocol` enum variant as a human-readable string.
    ///
    /// ```
    /// use lurk_protocol::{Protocol, PktMessage};
    ///
    /// let pkt_message = PktMessage::server("Recipient", "Hello, server!");
    /// let protocol = Protocol::Message(pkt_message);
    ///
    /// println!("{}", protocol); // Displays the serialized message packet
    /// ```
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Protocol::Message(msg) => write!(f, "{}", msg),
            Protocol::ChangeRoom(room) => write!(f, "{}", room),
            Protocol::Fight(fight) => write!(f, "{}", fight),
            Protocol::PVPFight(pvp_fight) => write!(f, "{}", pvp_fight),
            Protocol::Loot(loot) => write!(f, "{}", loot),
            Protocol::Start(start) => write!(f, "{}", start),
            Protocol::Error(error) => write!(f, "{}", error),
            Protocol::Accept(accept) => write!(f, "{}", accept),
            Protocol::Room(room) => write!(f, "{}", room),
            Protocol::Character(character) => write!(f, "{}", character),
            Protocol::Game(game) => write!(f, "{}", game),
            Protocol::Leave(leave) => write!(f, "{}", leave),
            Protocol::Connection(connection) => write!(f, "{}", connection),
            Protocol::Version(version) => write!(f, "{}", version),
        }
    }
}

impl Protocol {
    /// Receive one packet from the connected TcpStream
    ///
    /// ```no_run
    /// use lurk_protocol::Protocol;
    /// use std::net::TcpStream;
    /// use std::sync::Arc;
    ///
    /// let stream = Arc::new(TcpStream::connect("127.0.0.1:8080").unwrap());
    ///
    /// loop {
    ///     let packet = match Protocol::recv(&stream) {
    ///         Ok(pkt) => pkt,
    ///         Err(e) => {
    ///             eprintln!("Error receiving packet: {}", e);
    ///             continue;
    ///         },
    ///     };
    ///
    ///     match packet {
    ///         Protocol::Leave(_leave) => {
    ///             // Handle leave packet
    ///         },
    ///         _ => {
    ///            // Handle other packet types
    ///        },
    ///     }
    /// }
    /// ```
    pub fn recv(stream: &Arc<TcpStream>) -> Result<Protocol, std::io::Error> {
        let mut buffer = [0; 1];
        stream.as_ref().read_exact(&mut buffer)?;
        let packet_type = PktType::from(&buffer);

        #[cfg(feature = "tracing")]
        info!("Read packet type: {}", packet_type);

        match packet_type {
            PktType::MESSAGE => {
                let mut buffer = vec![0; 66];

                let pkt = Packet::read_extended(stream, packet_type, &mut buffer, (0, 1))?;

                Ok(Protocol::Message(PktMessage::decode(pkt)))
            }
            PktType::CHANGEROOM => {
                let mut buffer = vec![0; 2];

                let packet = Packet::read_into(stream, packet_type, &mut buffer)?;

                Ok(Protocol::ChangeRoom(PktChangeRoom::decode(packet)))
            }
            PktType::FIGHT => Ok(Protocol::Fight(PktFight::default())),
            PktType::PVPFIGHT => {
                let mut buffer = vec![0; 32];

                let packet = Packet::read_into(stream, packet_type, &mut buffer)?;

                Ok(Protocol::PVPFight(PktPVPFight::decode(packet)))
            }
            PktType::LOOT => {
                let mut buffer = vec![0; 32];

                let packet = Packet::read_into(stream, packet_type, &mut buffer)?;

                Ok(Protocol::Loot(PktLoot::decode(packet)))
            }
            PktType::START => Ok(Protocol::Start(PktStart::default())),
            PktType::ERROR => {
                let mut buffer = vec![0; 3];

                let packet = Packet::read_extended(stream, packet_type, &mut buffer, (1, 2))?;

                Ok(Protocol::Error(PktError::decode(packet)))
            }
            PktType::ACCEPT => {
                let mut buffer = vec![0; 1];

                let packet = Packet::read_into(stream, packet_type, &mut buffer)?;

                Ok(Protocol::Accept(PktAccept::decode(packet)))
            }
            PktType::ROOM => {
                let mut buffer = vec![0; 36];

                let packet = Packet::read_extended(stream, packet_type, &mut buffer, (34, 35))?;

                Ok(Protocol::Room(PktRoom::decode(packet)))
            }
            PktType::CHARACTER => {
                let mut buffer = vec![0; 47];

                let packet = Packet::read_extended(stream, packet_type, &mut buffer, (45, 46))?;

                Ok(Protocol::Character(PktCharacter::decode(packet)))
            }
            PktType::GAME => {
                let mut buffer = vec![0; 6];

                let packet = Packet::read_extended(stream, packet_type, &mut buffer, (4, 5))?;

                Ok(Protocol::Game(PktGame::decode(packet)))
            }
            PktType::LEAVE => Ok(Protocol::Leave(PktLeave::default())),
            PktType::CONNECTION => {
                let mut buffer = vec![0; 36];

                let packet = Packet::read_extended(stream, packet_type, &mut buffer, (34, 35))?;

                Ok(Protocol::Connection(PktConnection::decode(packet)))
            }
            PktType::VERSION => {
                let mut buffer = vec![0; 4];

                let packet = Packet::read_extended(stream, packet_type, &mut buffer, (2, 3))?;

                Ok(Protocol::Version(PktVersion::decode(packet)))
            }
            PktType::DEFAULT => Err(Error::new(ErrorKind::Unsupported, "Invalid packet type")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Protocol::Display must produce non-empty output for every variant.
    #[test]
    fn protocol_display_message() {
        let pkt = PktMessage::server("Recipient", "Hello");
        let proto = Protocol::Message(pkt);
        let output = format!("{}", proto);
        assert!(!output.is_empty(), "Display for Message must be non-empty");
    }

    #[test]
    fn protocol_display_changeroom() {
        let pkt = PktChangeRoom::from(1u16);
        let proto = Protocol::ChangeRoom(pkt);
        let output = format!("{}", proto);
        assert!(
            !output.is_empty(),
            "Display for ChangeRoom must be non-empty"
        );
    }

    #[test]
    fn protocol_display_fight() {
        let pkt = PktFight::default();
        let proto = Protocol::Fight(pkt);
        let output = format!("{}", proto);
        assert!(!output.is_empty(), "Display for Fight must be non-empty");
    }

    #[test]
    fn protocol_display_pvpfight() {
        let pkt = PktPVPFight::fight("Target");
        let proto = Protocol::PVPFight(pkt);
        let output = format!("{}", proto);
        assert!(!output.is_empty(), "Display for PVPFight must be non-empty");
    }

    #[test]
    fn protocol_display_loot() {
        let pkt = PktLoot::loot("Monster");
        let proto = Protocol::Loot(pkt);
        let output = format!("{}", proto);
        assert!(!output.is_empty(), "Display for Loot must be non-empty");
    }

    #[test]
    fn protocol_display_start() {
        let pkt = PktStart::default();
        let proto = Protocol::Start(pkt);
        let output = format!("{}", proto);
        assert!(!output.is_empty(), "Display for Start must be non-empty");
    }

    #[test]
    fn protocol_display_error() {
        let pkt = PktError::new(crate::LurkError::OTHER, "test");
        let proto = Protocol::Error(pkt);
        let output = format!("{}", proto);
        assert!(!output.is_empty(), "Display for Error must be non-empty");
    }

    #[test]
    fn protocol_display_accept() {
        let pkt = PktAccept::new(PktType::CHARACTER);
        let proto = Protocol::Accept(pkt);
        let output = format!("{}", proto);
        assert!(!output.is_empty(), "Display for Accept must be non-empty");
    }

    #[test]
    fn protocol_display_room() {
        let pkt = PktRoom {
            packet_type: PktType::ROOM,
            room_number: 1,
            room_name: "Test".into(),
            description_len: 4,
            description: "desc".into(),
        };
        let proto = Protocol::Room(pkt);
        let output = format!("{}", proto);
        assert!(!output.is_empty(), "Display for Room must be non-empty");
    }

    #[test]
    fn protocol_display_character() {
        let pkt = PktCharacter {
            packet_type: PktType::CHARACTER,
            name: "Hero".into(),
            flags: crate::CharacterFlags::reset(),
            attack: 10,
            defense: 10,
            regen: 5,
            health: 100,
            gold: 0,
            current_room: 0,
            description_len: 4,
            description: "desc".into(),
        };
        let proto = Protocol::Character(pkt);
        let output = format!("{}", proto);
        assert!(
            !output.is_empty(),
            "Display for Character must be non-empty"
        );
    }

    #[test]
    fn protocol_display_game() {
        let pkt = PktGame {
            packet_type: PktType::GAME,
            initial_points: 100,
            stat_limit: 65535,
            description_len: 4,
            description: "desc".into(),
        };
        let proto = Protocol::Game(pkt);
        let output = format!("{}", proto);
        assert!(!output.is_empty(), "Display for Game must be non-empty");
    }

    #[test]
    fn protocol_display_leave() {
        let pkt = PktLeave::default();
        let proto = Protocol::Leave(pkt);
        let output = format!("{}", proto);
        assert!(!output.is_empty(), "Display for Leave must be non-empty");
    }

    #[test]
    fn protocol_display_connection() {
        let pkt = PktConnection {
            packet_type: PktType::CONNECTION,
            room_number: 1,
            room_name: "Test".into(),
            description_len: 4,
            description: "desc".into(),
        };
        let proto = Protocol::Connection(pkt);
        let output = format!("{}", proto);
        assert!(
            !output.is_empty(),
            "Display for Connection must be non-empty"
        );
    }

    #[test]
    fn protocol_display_version() {
        let pkt = PktVersion {
            packet_type: PktType::VERSION,
            major_rev: 2,
            minor_rev: 3,
            extensions_len: 0,
            extensions: None,
        };
        let proto = Protocol::Version(pkt);
        let output = format!("{}", proto);
        assert!(!output.is_empty(), "Display for Version must be non-empty");
    }
}
