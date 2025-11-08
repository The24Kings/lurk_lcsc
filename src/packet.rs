use std::io::ErrorKind::UnexpectedEof;
use std::{
    io::{Error, Read, Write},
    net::TcpStream,
    sync::Arc,
};

#[cfg(feature = "tracing")]
use crate::pcap::PCap;
#[cfg(feature = "tracing")]
use tracing::debug;

use crate::pkt_type::PktType;

/// Module for handling accept packets.
pub mod accept;
/// Module for handling change room packets.
pub mod change_room;
/// Module for handling character packets.
pub mod character;
/// Module for handling connection packets.
pub mod connection;
/// Module for handling error packets.
pub mod error;
/// Module for handling fight packets.
pub mod fight;
/// Module for handling game packets.
pub mod game;
/// Module for handling leave packets.
pub mod leave;
/// Module for handling loot packets.
pub mod loot;
/// Module for handling message packets.
pub mod message;
/// Module for handling player-versus-player fight packets.
pub mod pvp_fight;
/// Module for handling room packets.
pub mod room;
/// Module for handling start packets.
pub mod start;
/// Module for handling version packets.
pub mod version;

/// Trait for serializing and deserializing packets.
///
/// ```no_run
/// use lurk_lcsc::{Packet, Parser, PktType};
/// use std::io::{Error, Write};
/// use serde::Serialize;
///
///
/// pub struct PktLoot {
///    pub message_type: PktType,
///    pub target_name: Box<str>,
///}
///
/// impl Parser<'_> for PktLoot {
///     fn serialize<W: Write>(self, writer: &mut W) -> Result<(), Error> {
///         // Package into a byte array
///         let mut packet: Vec<u8> = vec![self.message_type.into()];
///
///         let mut target_name_bytes = self.target_name.as_bytes().to_vec();
///         target_name_bytes.resize(32, 0x00); // Pad the name to 32 bytes
///         packet.extend(target_name_bytes);
///
///         // Write the packet to the buffer
///         writer
///             .write_all(&packet)
///             .map_err(|_| Error::other("Failed to write packet to buffer"))?;
///
///         Ok(())
///     }
///
///     fn deserialize(packet: Packet) -> Self {
///         let message_type = packet.packet_type;
///         let target_name = String::from_utf8_lossy(&packet.body[0..32])
///             .trim_end_matches('\0')
///             .into();
///
///         Self {
///             message_type,
///             target_name,
///         }
///     }
/// }
/// ```
pub trait Parser<'a>: Sized + 'a {
    /// Serializes the packet and writes it to the provided writer.
    ///
    /// ```no_run
    /// use lurk_lcsc::{Parser, PktType};
    /// use lurk_lcsc::PktVersion;
    /// use std::io::Write;
    ///
    /// let packet = PktVersion {
    ///    packet_type: PktType::VERSION,
    ///    major_rev: 2,
    ///    minor_rev: 3,
    ///    extensions_len: 0,
    ///    extensions: None,
    /// };
    ///
    /// let mut buffer: Vec<u8> = Vec::new();
    /// packet.serialize(&mut buffer).unwrap();
    /// ```
    fn serialize<W: Write>(self, writer: &mut W) -> Result<(), Error>;

    /// Deserializes a Packet into the implementing type.
    ///
    /// ```no_run
    /// use lurk_lcsc::{Protocol, PktType, PktMessage, Packet, Parser};
    /// use std::io::{Read, Error, ErrorKind};
    /// use std::sync::{Arc, mpsc};
    /// use std::net::TcpStream;
    ///
    ///
    /// let stream = Arc::new(TcpStream::connect("127.0.0.1:8080").unwrap());
    ///
    /// let mut buffer = [0; 1];
    /// stream.as_ref().read_exact(&mut buffer).unwrap();
    /// let packet_type = PktType::from(&buffer);
    ///
    /// // Match the type of the packet to the enum Type
    /// let packet: Result<Protocol, Error> = match packet_type {
    ///     PktType::MESSAGE => {
    ///        let mut buffer = vec![0; 66];
    ///
    ///        let pkt = Packet::read_extended(&stream, packet_type, &mut buffer, (0, 1)).unwrap();
    ///
    ///        Ok(Protocol::Message(
    ///            stream.clone(),
    ///            PktMessage::deserialize(pkt),
    ///        ))
    ///    },
    ///     _ => todo!("Handle other packet types"),
    ///     PktType::DEFAULT => Err(Error::new(ErrorKind::Unsupported, "Invalid packet type")),
    /// };
    /// ```
    fn deserialize(packet: Packet) -> Self;
}

/// Represents a network packet containing a reference to the TCP stream, packet type, and body.
///
/// Do not use this directly; for internal use only. Needed for testing.
pub struct Packet<'a> {
    /// Reference to the TCP stream associated with this packet.
    pub stream: &'a Arc<TcpStream>,
    /// The type of the packet message.
    pub packet_type: PktType,
    /// The body of the packet containing the raw bytes.
    pub body: &'a [u8],
}

impl<'a> Packet<'a> {
    /// Creates a new `Packet` from the given TCP stream, packet type, and byte slice.
    pub(crate) fn new(stream: &'a Arc<TcpStream>, packet_type: PktType, bytes: &'a [u8]) -> Self {
        Packet {
            stream,
            packet_type,
            body: bytes,
        }
    }

    /// Read the stream into a packet with a fixed length.
    /// This function reads the packet body based on the provided buffer length.
    pub fn read_into<'b>(
        stream: &'b Arc<TcpStream>,
        packet_type: PktType,
        buffer: &'b mut [u8],
    ) -> Result<Packet<'b>, Error> {
        // Read the remaining bytes for the packet
        stream
            .as_ref()
            .read_exact(buffer)
            .map_err(|e| Error::new(UnexpectedEof, format!("Failed to read packet body: {}", e)))?;

        #[cfg(feature = "tracing")]
        debug!("Packet body:\n{}", PCap::build(buffer.to_vec()));

        // Create a new packet with the read bytes
        let packet = Packet::new(stream, packet_type, buffer);

        Ok(packet)
    }

    /// Read the packet with a varied length.
    /// This function reads the packet body and then reads the extended description or data
    /// based on the provided index.
    pub fn read_extended<'b>(
        stream: &'b Arc<TcpStream>,
        packet_type: PktType,
        buffer: &'b mut Vec<u8>,
        index: (usize, usize),
    ) -> Result<Packet<'b>, Error> {
        stream
            .as_ref()
            .read_exact(buffer)
            .map_err(|e| Error::new(UnexpectedEof, format!("Failed to read packet body: {}", e)))?;

        // Get the description length from the buffer
        let length = u16::from_le_bytes([buffer[index.0], buffer[index.1]]) as usize;
        let mut desc = vec![0u8; length];

        #[cfg(feature = "tracing")]
        debug!("Description len {}: ({}, {})", length, index.0, index.1);

        // Read the description from the stream
        stream
            .as_ref()
            .read_exact(&mut desc)
            .map_err(|e| Error::new(UnexpectedEof, format!("Failed to read descriptor: {}", e)))?;

        #[cfg(feature = "tracing")]
        if !desc.is_empty() {
            debug!("Read description: {}", String::from_utf8_lossy(&desc));
        } else {
            debug!("Read empty description");
        }

        // Extend the buffer with the description
        buffer.extend_from_slice(&desc);

        let packet = Packet::new(stream, packet_type, buffer);

        Ok(packet)
    }
}
