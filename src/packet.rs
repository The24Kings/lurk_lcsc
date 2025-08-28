use std::{
    io::{Read, Write},
    net::TcpStream,
    sync::Arc,
};

#[cfg(feature = "logging")]
use crate::pcap::PCap;
#[cfg(feature = "logging")]
use tracing::debug;

use crate::PktType;

pub mod accept;
pub mod change_room;
pub mod character;
pub mod connection;
pub mod error;
pub mod fight;
pub mod game;
pub mod leave;
pub mod loot;
pub mod message;
pub mod pvp_fight;
pub mod room;
pub mod start;
pub mod version;

pub trait Parser<'a>: Sized + 'a {
    fn serialize<W: Write>(self, writer: &mut W) -> Result<(), std::io::Error>;
    fn deserialize(packet: Packet) -> Result<Self, std::io::Error>;
}

pub struct Packet<'a> {
    pub stream: &'a Arc<TcpStream>,
    pub message_type: PktType,
    pub body: &'a [u8],
}

impl<'a> Packet<'a> {
    pub fn new(stream: &'a Arc<TcpStream>, message_type: PktType, bytes: &'a [u8]) -> Self {
        Packet {
            stream,
            message_type,
            body: &bytes[0..],
        }
    }

    /// Read the stream into a packet
    pub fn read_into<'b>(
        stream: &'b Arc<TcpStream>,
        message_type: PktType,
        buffer: &'b mut Vec<u8>,
    ) -> Result<Packet<'b>, std::io::Error> {
        // Read the remaining bytes for the packet
        let _bytes_read = stream.as_ref().read_exact(buffer).map_err(|e| {
            std::io::Error::new(
                std::io::ErrorKind::UnexpectedEof,
                format!("Failed to read packet body: {}", e),
            )
        })?;

        #[cfg(feature = "logging")]
        debug!("[DEBUG] Packet body:\n{}", PCap::build(buffer.clone()));

        // Create a new packet with the read bytes
        let packet = Packet::new(stream, message_type, buffer);

        Ok(packet)
    }

    /// Read the packet with a varied length.
    /// This function reads the packet body and then reads the extended description or data
    /// based on the provided index.
    pub fn read_extended<'b>(
        stream: &'b Arc<TcpStream>,
        message_type: PktType,
        buffer: &'b mut Vec<u8>,
        index: (usize, usize),
    ) -> Result<Packet<'b>, std::io::Error> {
        stream.as_ref().read_exact(buffer).map_err(|e| {
            std::io::Error::new(
                std::io::ErrorKind::UnexpectedEof,
                format!("Failed to read packet body: {}", e),
            )
        })?;

        // Get the description length from the buffer
        let length = usize::from_le_bytes([buffer[index.0], buffer[index.1], 0, 0, 0, 0, 0, 0]);
        let mut desc = vec![0u8; length];

        #[cfg(feature = "logging")]
        debug!(
            "[PACKET] Reading description of length {} at index {}, {}",
            length, index.0, index.1
        );

        // Read the description from the stream
        stream.as_ref().read_exact(&mut desc).map_err(|e| {
            std::io::Error::new(
                std::io::ErrorKind::UnexpectedEof,
                format!("Failed to read descriptor: {}", e),
            )
        })?;

        #[cfg(feature = "logging")]
        if !desc.is_empty() {
            debug!(
                "[PACKET] Read description: {}",
                String::from_utf8_lossy(&desc)
            );
        } else {
            debug!("[PACKET] Read empty description");
        }

        // Extend the buffer with the description
        buffer.extend_from_slice(&desc);

        let packet = Packet::new(stream, message_type, buffer);

        Ok(packet)
    }
}
