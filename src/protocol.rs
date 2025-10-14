use std::io::Read as _;
use std::io::Write;
use std::io::{Error, ErrorKind};
use std::net::TcpStream;
use std::sync::Arc;

#[cfg(feature = "tracing")]
use crate::pcap::PCap;
#[cfg(feature = "tracing")]
use tracing::{debug, info};

use crate::{
    Packet, Parser, PktAccept, PktChangeRoom, PktCharacter, PktConnection, PktError, PktFight,
    PktGame, PktLeave, PktLoot, PktMessage, PktPVPFight, PktRoom, PktStart, PktType, PktVersion,
};

/// Represents all possible protocol packets exchanged between the client and server.
pub enum Protocol {
    /// Packet containing a message sent between client and server.
    Message(Arc<TcpStream>, PktMessage),
    /// Packet containing a request to change the room.
    ChangeRoom(Arc<TcpStream>, PktChangeRoom),
    /// Packet containing a fight request.
    Fight(Arc<TcpStream>, PktFight),
    /// Packet containing a player-versus-player fight request.
    PVPFight(Arc<TcpStream>, PktPVPFight),
    /// Packet containing a loot request.
    Loot(Arc<TcpStream>, PktLoot),
    /// Packet containing a start request.
    Start(Arc<TcpStream>, PktStart),
    /// Packet containing an error response.
    Error(Arc<TcpStream>, PktError),
    /// Packet containing an acceptance response.
    Accept(Arc<TcpStream>, PktAccept),
    /// Packet containing room information.
    Room(Arc<TcpStream>, PktRoom),
    /// Packet containing character information.
    Character(Arc<TcpStream>, PktCharacter),
    /// Packet containing game information.
    Game(Arc<TcpStream>, PktGame),
    /// Packet containing leave information.
    Leave(Arc<TcpStream>, PktLeave),
    /// Packet containing connection information.
    Connection(Arc<TcpStream>, PktConnection),
    /// Packet containing version information.
    Version(Arc<TcpStream>, PktVersion),
}
impl std::fmt::Display for Protocol {
    /// Formats the `Protocol` enum variant as a human-readable string.
    ///
    /// ```no_run
    /// use lurk_lcsc::{Protocol, PktMessage};
    /// use std::net::TcpStream;
    /// use std::sync::Arc;
    ///
    ///
    /// let stream = Arc::new(TcpStream::connect("127.0.0.1:8080").unwrap());
    /// let pkt_message = PktMessage::server("Recipient", "Hello, server!");
    /// let protocol = Protocol::Message(stream, pkt_message);
    ///
    /// println!("{}", protocol); // Displays the serialized message packet
    /// ```
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Protocol::Message(_, msg) => write!(f, "{}", msg),
            Protocol::ChangeRoom(_, room) => write!(f, "{}", room),
            Protocol::Fight(_, fight) => write!(f, "{}", fight),
            Protocol::PVPFight(_, pvp_fight) => write!(f, "{}", pvp_fight),
            Protocol::Loot(_, loot) => write!(f, "{}", loot),
            Protocol::Start(_, start) => write!(f, "{}", start),
            Protocol::Error(_, error) => write!(f, "{}", error),
            Protocol::Accept(_, accept) => write!(f, "{}", accept),
            Protocol::Room(_, room) => write!(f, "{}", room),
            Protocol::Character(_, character) => write!(f, "{}", character),
            Protocol::Game(_, game) => write!(f, "{}", game),
            Protocol::Leave(_, leave) => write!(f, "{}", leave),
            Protocol::Connection(_, connection) => write!(f, "{}", connection),
            Protocol::Version(_, version) => write!(f, "{}", version),
        }
    }
}

impl Protocol {
    /// Serializes and sends the protocol packet to the server.
    ///
    /// ```no_run
    /// use lurk_lcsc::{Protocol, PktMessage};
    /// use std::net::TcpStream;
    /// use std::sync::Arc;
    ///
    /// let stream = Arc::new(TcpStream::connect("127.0.0.1:8080").unwrap());
    /// let pkt_message = PktMessage::server("Recipient", "Message");
    ///
    /// // Send the packet
    /// Protocol::Message(stream.clone(), pkt_message).send().unwrap();
    /// ```
    pub fn send(self) -> Result<(), std::io::Error> {
        let mut byte_stream: Vec<u8> = Vec::new();

        #[cfg(feature = "tracing")]
        info!("[PROTOCOL] Sending packet: {}", self);

        // Serialize the packet and send it to the server
        let author = match self {
            Protocol::Message(author, content) => {
                content.serialize(&mut byte_stream)?;
                author
            }
            Protocol::ChangeRoom(author, content) => {
                content.serialize(&mut byte_stream)?;
                author
            }
            Protocol::Fight(author, content) => {
                content.serialize(&mut byte_stream)?;
                author
            }
            Protocol::PVPFight(author, content) => {
                content.serialize(&mut byte_stream)?;
                author
            }
            Protocol::Loot(author, content) => {
                content.serialize(&mut byte_stream)?;
                author
            }
            Protocol::Start(author, content) => {
                content.serialize(&mut byte_stream)?;
                author
            }
            Protocol::Error(author, content) => {
                content.serialize(&mut byte_stream)?;
                author
            }
            Protocol::Accept(author, content) => {
                content.serialize(&mut byte_stream)?;
                author
            }
            Protocol::Room(author, content) => {
                content.serialize(&mut byte_stream)?;
                author
            }
            Protocol::Character(author, content) => {
                content.serialize(&mut byte_stream)?;
                author
            }
            Protocol::Game(author, content) => {
                content.serialize(&mut byte_stream)?;
                author
            }
            Protocol::Leave(author, content) => {
                content.serialize(&mut byte_stream)?;
                author
            }
            Protocol::Connection(author, content) => {
                content.serialize(&mut byte_stream)?;
                author
            }
            Protocol::Version(author, content) => {
                content.serialize(&mut byte_stream)?;
                author
            }
        };

        #[cfg(feature = "tracing")]
        debug!("[PROTOCOL] Packet:\n{}", PCap::build(byte_stream.clone()));

        author.as_ref().write_all(&byte_stream)?;

        Ok(())
    }

    /// Receive one packet from the connected TcpStream
    ///
    /// ```no_run
    /// use lurk_lcsc::{Protocol, PktLeave};
    /// use std::io::{Error, ErrorKind};
    /// use std::net::TcpStream;
    /// use std::sync::Arc;
    ///
    /// let stream = Arc::new(TcpStream::connect("127.0.0.1:8080").unwrap());
    ///
    /// loop {
    ///     let packet = match Protocol::recv(&stream) {
    ///         Ok(pkt) => pkt,
    ///         Err(e) => todo!("Handle any errors"),
    ///     };
    ///
    ///     todo!("Send packet to server")
    /// }
    /// ```
    pub fn recv(stream: &Arc<TcpStream>) -> Result<Protocol, std::io::Error> {
        let mut buffer = [0; 1];
        let bytes_read = stream.as_ref().read(&mut buffer)?;
        let packet_type = buffer[0].into();

        if bytes_read != 1 {
            return Err(Error::new(ErrorKind::UnexpectedEof, "Connection closed"));
        }

        #[cfg(feature = "tracing")]
        info!("[PROTOCOL] Read packet type: {}", packet_type);

        match packet_type {
            PktType::MESSAGE => {
                let mut buffer = vec![0; 66];

                let pkt = Packet::read_extended(stream, packet_type, &mut buffer, (0, 1))?;

                Ok(Protocol::Message(
                    stream.clone(),
                    PktMessage::deserialize(pkt),
                ))
            }
            PktType::CHANGEROOM => {
                let mut buffer = vec![0; 2];

                let packet = Packet::read_into(stream, packet_type, &mut buffer)?;

                Ok(Protocol::ChangeRoom(
                    stream.clone(),
                    PktChangeRoom::deserialize(packet),
                ))
            }
            PktType::FIGHT => Ok(Protocol::Fight(stream.clone(), PktFight::default())),
            PktType::PVPFIGHT => {
                let mut buffer = vec![0; 32];

                let packet = Packet::read_into(stream, packet_type, &mut buffer)?;

                Ok(Protocol::PVPFight(
                    stream.clone(),
                    PktPVPFight::deserialize(packet),
                ))
            }
            PktType::LOOT => {
                let mut buffer = vec![0; 32];

                let packet = Packet::read_into(stream, packet_type, &mut buffer)?;

                Ok(Protocol::Loot(stream.clone(), PktLoot::deserialize(packet)))
            }
            PktType::START => Ok(Protocol::Start(stream.clone(), PktStart::default())),
            PktType::ERROR => {
                let mut buffer = vec![0; 3];

                let packet = Packet::read_extended(stream, packet_type, &mut buffer, (1, 2))?;

                Ok(Protocol::Error(
                    stream.clone(),
                    PktError::deserialize(packet),
                ))
            }
            PktType::ACCEPT => {
                let mut buffer = vec![0; 1];

                let packet = Packet::read_into(stream, packet_type, &mut buffer)?;

                Ok(Protocol::Accept(
                    stream.clone(),
                    PktAccept::deserialize(packet),
                ))
            }
            PktType::ROOM => {
                let mut buffer = vec![0; 36];

                let packet = Packet::read_extended(stream, packet_type, &mut buffer, (34, 35))?;

                Ok(Protocol::Room(stream.clone(), PktRoom::deserialize(packet)))
            }
            PktType::CHARACTER => {
                let mut buffer = vec![0; 47];

                let packet = Packet::read_extended(stream, packet_type, &mut buffer, (45, 46))?;

                Ok(Protocol::Character(
                    stream.clone(),
                    PktCharacter::deserialize(packet),
                ))
            }
            PktType::GAME => {
                let mut buffer = vec![0; 6];

                let packet = Packet::read_extended(stream, packet_type, &mut buffer, (4, 5))?;

                Ok(Protocol::Game(stream.clone(), PktGame::deserialize(packet)))
            }
            PktType::LEAVE => Ok(Protocol::Leave(stream.clone(), PktLeave::default())),
            PktType::CONNECTION => {
                let mut buffer = vec![0; 36];

                let packet = Packet::read_extended(stream, packet_type, &mut buffer, (34, 35))?;

                Ok(Protocol::Connection(
                    stream.clone(),
                    PktConnection::deserialize(packet),
                ))
            }
            PktType::VERSION => {
                let mut buffer = vec![0; 4];

                let packet = Packet::read_extended(stream, packet_type, &mut buffer, (2, 3))?;

                Ok(Protocol::Version(
                    stream.clone(),
                    PktVersion::deserialize(packet),
                ))
            }
            PktType::DEFAULT => Err(Error::new(ErrorKind::Unsupported, "Invalid packet type")),
        }
    }
}
