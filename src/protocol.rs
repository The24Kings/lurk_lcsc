use std::io::Write;
use std::net::TcpStream;
use std::sync::Arc;

#[cfg(feature = "tracing")]
use crate::pcap::PCap;
#[cfg(feature = "tracing")]
use tracing::{debug, info};

use crate::{
    Parser, PktAccept, PktChangeRoom, PktCharacter, PktConnection, PktError, PktFight, PktGame,
    PktLeave, PktLoot, PktMessage, PktPVPFight, PktRoom, PktStart, PktVersion,
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
    /// use std::net::TcpStream;
    /// use std::sync::Arc;
    ///
    /// use lurk_lcsc::{Protocol, PktMessage};
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
    /// use std::net::TcpStream;
    /// use std::sync::Arc;
    /// use lurk_lcsc::{Protocol, PktMessage};
    ///
    /// // Assume you have a TcpStream and a PktMessage
    /// let stream = Arc::new(TcpStream::connect("127.0.0.1:8080").unwrap());
    /// let pkt_message = PktMessage::server("Recipient", "Hello, server!");
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
}
