use serde::Serialize;
use std::io::Write;

use crate::packet::PktType;
use crate::{Packet, Parser};

#[derive(Serialize)]
/// Sent by the client to message other players.
///
/// - Can also be used by the server to send "presentable" information to the client (information that can be displayed to the user with no further processing).
/// - Clients should expect to receive this type of message at any time, and servers should expect to relay messages for clients at any time.
/// - If using this to send game information, the server should mark the message as narration.
pub struct PktMessage {
    /// The type of message for the `MESSAGE` packet. Defaults to 1.
    pub message_type: PktType,
    /// The length of the message.
    pub message_len: u16,
    /// The recipient of the message, up to 32 bytes.
    pub recipient: Box<str>,
    /// The sender of the message, up to 30 bytes.
    pub sender: Box<str>,
    /// Whether the message is narration (from the narrator) or not (from a player or the server).
    pub narration: bool,
    /// The message content. Length was specified in `message_len`.
    pub message: Box<str>,
}

impl PktMessage {
    /// Create a new `PktMessage` from the server to a specific recipient.
    /// The sender will be "Server" and the narration flag will be false.
    /// This is used for system messages, such as "You have been disconnected" or "Welcome to the game".
    pub fn server(recipient: &str, message: &str) -> Self {
        Self {
            message_type: PktType::MESSAGE,
            message_len: message.len() as u16,
            recipient: Box::from(recipient),
            sender: Box::from("Server"),
            narration: false,
            message: Box::from(message),
        }
    }

    /// Create a new `PktMessage` from the narrator to a specific recipient.
    /// The sender will be "Narrator" and the narration flag will be true.
    /// This is used for room descriptions and other narrative messages.
    pub fn narrator(recipient: &str, message: &str) -> Self {
        Self {
            message_type: PktType::MESSAGE,
            message_len: message.len() as u16,
            recipient: Box::from(recipient),
            sender: Box::from("Narrator"),
            narration: true,
            message: Box::from(message),
        }
    }
}

impl std::fmt::Display for PktMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            serde_json::to_string(self)
                .unwrap_or_else(|_| "Failed to serialize Message".to_string())
        )
    }
}

impl Parser<'_> for PktMessage {
    fn serialize<W: Write>(self, writer: &mut W) -> Result<(), std::io::Error> {
        // Package into a byte array
        let mut packet: Vec<u8> = vec![self.message_type.into()];

        packet.extend(self.message_len.to_le_bytes());

        let mut r_bytes = self.recipient.as_bytes().to_vec();
        let mut s_bytes = self.sender.as_bytes().to_vec();

        // Pad the recipient and sender names to 32 bytes
        r_bytes.resize(32, 0x00);
        s_bytes.resize(30, 0x00);

        // If the sender is a narrator, append 0x00 0x01 to the end of the sender name
        if self.narration {
            s_bytes.extend_from_slice(&[0x00, 0x01]);
        } else {
            s_bytes.resize(32, 0x00);
        }
        packet.extend(r_bytes);
        packet.extend(s_bytes);

        // Append the message
        packet.extend(self.message.as_bytes());

        // Write the packet to the buffer
        writer
            .write_all(&packet)
            .map_err(|_| std::io::Error::other("Failed to write packet to buffer"))?;

        Ok(())
    }

    fn deserialize(packet: Packet) -> Self {
        let message_len = u16::from_le_bytes([packet.body[0], packet.body[1]]);

        // Process the names for recipient and sender
        let r_bytes = packet.body[2..34].to_vec();
        let mut s_bytes = packet.body[34..66].to_vec();

        // If the last 2 bytes of the sender is 0x00 0x01, it means the sender is a narrator
        let narration = match s_bytes.get(32..34) {
            Some(&[0x00, 0x01]) => {
                s_bytes.truncate(32); // Remove the last 2 bytes
                true
            }
            _ => false,
        };

        let recipient = String::from_utf8_lossy(&r_bytes)
            .split('\0')
            .take(1)
            .collect();
        let sender = String::from_utf8_lossy(&s_bytes)
            .split('\0')
            .take(1)
            .collect();
        let message = String::from_utf8_lossy(&packet.body[66..]).into();

        Self {
            message_type: packet.message_type,
            message_len,
            recipient,
            sender,
            narration,
            message,
        }
    }
}
