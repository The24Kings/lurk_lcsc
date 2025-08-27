use std::io::Write;
use std::net::TcpStream;
use std::sync::Arc;

#[cfg(feature = "logging")]
use tracing::{debug, info};

#[cfg(feature = "logging")]
use crate::pcap::PCap;

#[cfg(feature = "custom-cmds")]
use crate::commands::Action;

use crate::{
    Parser, PktAccept, PktChangeRoom, PktCharacter, PktConnection, PktError, PktFight, PktGame,
    PktLeave, PktLoot, PktMessage, PktPVPFight, PktRoom, PktStart, PktVersion,
};

pub enum Protocol {
    Message(Arc<TcpStream>, PktMessage),
    ChangeRoom(Arc<TcpStream>, PktChangeRoom),
    Fight(Arc<TcpStream>, PktFight),
    PVPFight(Arc<TcpStream>, PktPVPFight),
    Loot(Arc<TcpStream>, PktLoot),
    Start(Arc<TcpStream>, PktStart),
    Error(Arc<TcpStream>, PktError),
    Accept(Arc<TcpStream>, PktAccept),
    Room(Arc<TcpStream>, PktRoom),
    Character(Arc<TcpStream>, PktCharacter),
    Game(Arc<TcpStream>, PktGame),
    Leave(Arc<TcpStream>, PktLeave),
    Connection(Arc<TcpStream>, PktConnection),
    Version(Arc<TcpStream>, PktVersion),
    #[cfg(feature = "custom-cmds")]
    Command(Action),
}

impl std::fmt::Display for Protocol {
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
            #[cfg(feature = "custom-cmds")]
            Protocol::Command(action) => write!(f, "{}", action),
        }
    }
}

impl Protocol {
    pub fn send(self) -> Result<(), std::io::Error> {
        let mut byte_stream: Vec<u8> = Vec::new();

        #[cfg(feature = "logging")]
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
            #[cfg(feature = "custom-cmds")]
            _ => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    "Cannot send this Protocol type",
                ));
            }
        };

        #[cfg(feature = "logging")]
        debug!("[PROTOCOL] Packet:\n{}", PCap::build(byte_stream.clone()));

        author.as_ref().write_all(&byte_stream)?;

        Ok(())
    }
}
