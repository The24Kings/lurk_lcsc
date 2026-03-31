use serde::{Deserialize, Serialize};
use std::io::Write;

use crate::packet::PktType;
use crate::{Packet, Parser};

#[derive(Serialize, Deserialize)]
/// Sent by the client to message other players.
///
/// - Can also be used by the server to send "presentable" information to the client (information that can be displayed to the user with no further processing).
/// - Clients should expect to receive this type of message at any time, and servers should expect to relay messages for clients at any time.
/// - If using this to send game information, the server should mark the message as narration.
pub struct PktMessage {
    /// The type of message for the `MESSAGE` packet. Defaults to 1.
    pub packet_type: PktType,
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
            packet_type: PktType::MESSAGE,
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
            packet_type: PktType::MESSAGE,
            message_len: message.len() as u16,
            recipient: Box::from(recipient),
            sender: Box::from("Narrator"),
            narration: true,
            message: Box::from(message),
        }
    }
}

#[macro_export]
/// Send `PktMessage` over `TcpStream` to connected user
///
/// ```no_run
/// use lurk_lcsc::{Protocol, PktMessage, PktType, send_message};
/// use std::sync::Arc;
/// use std::net::TcpStream;
///
/// let stream = Arc::new(TcpStream::connect("127.0.0.1:8080").unwrap());
/// let msg = PktMessage {
///     packet_type: PktType::MESSAGE,
///     message_len: 13 as u16,
///     recipient: Box::from("Test"),
///     sender: Box::from("Server"),
///     narration: false,
///     message: Box::from("Hello, World!"),
/// };
///
/// send_message!(stream.clone(), msg)
/// ```
macro_rules! send_message {
    ($stream:expr, $msg:expr) => {
        if let Err(e) = $crate::send_to($stream.as_ref(), &$msg) {
            eprintln!("Failed to send message packet: {}", e);
        }
    };
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
    fn write_to<W: Write>(&self, writer: &mut W) -> Result<(), std::io::Error> {
        // Package into a byte array
        let mut packet: Vec<u8> = vec![self.packet_type.into()];

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

    fn decode(packet: Packet) -> Self {
        let message_len = u16::from_le_bytes([packet.body[0], packet.body[1]]);

        // Process the names for recipient and sender
        let r_bytes = packet.body[2..34].to_vec();
        let mut s_bytes = packet.body[34..66].to_vec();

        // If the last 2 bytes of the sender is 0x00 0x01, it means the sender is a narrator
        let narration = match s_bytes.get(30..32) {
            Some(&[0x00, 0x01]) => {
                s_bytes.truncate(30); // Remove the narration marker bytes
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
            packet_type: packet.packet_type,
            message_len,
            recipient,
            sender,
            narration,
            message,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::test_common;

    use super::*;

    #[test]
    fn message_parse_and_serialize() {
        let stream = test_common::setup();
        let type_byte = PktType::MESSAGE;
        let original_bytes: &[u8; 80] = &[
            0x01, 0x0d, 0x00, 0x54, 0x65, 0x73, 0x74, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x53, 0x65, 0x72, 0x76, 0x65, 0x72, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x48, 0x65, 0x6c,
            0x6c, 0x6f, 0x2c, 0x20, 0x57, 0x6f, 0x72, 0x6c, 0x64, 0x21,
        ];

        // Create a packet with known bytes, excluding the type byte
        let packet = Packet::new(&stream, type_byte, &original_bytes[1..]);

        // Deserialize the packet into a PktMessage
        let message = PktMessage::decode(packet);

        // Assert the fields were parsed correctly
        assert_eq!(message.packet_type, PktType::MESSAGE);
        assert_eq!(message.message_len, 13);
        assert_eq!(message.recipient.as_ref(), "Test");
        assert_eq!(message.sender.as_ref(), "Server");
        assert!(!message.narration);
        assert_eq!(message.message.as_ref(), "Hello, World!");

        // Serialize the message back into bytes
        let mut buffer: Vec<u8> = Vec::new();
        message.write_to(&mut buffer).expect("Encoding failed");

        // Assert that the serialized bytes match the original
        assert_eq!(buffer, original_bytes);
        assert_eq!(buffer[0], u8::from(type_byte));
    }

    /// Parse trace message: Player2 -> Player1 "Sup".
    #[test]
    fn message_parse_trace_player2_to_player1() {
        let stream = test_common::setup();
        let mut body: Vec<u8> = Vec::new();
        body.extend(3u16.to_le_bytes()); // message_len = 3
        let mut recipient = b"Player1".to_vec();
        recipient.resize(32, 0x00);
        body.extend(&recipient);
        let mut sender = b"Player2".to_vec();
        sender.resize(32, 0x00);
        body.extend(&sender);
        body.extend(b"Sup");

        let packet = Packet::new(&stream, PktType::MESSAGE, &body);
        let msg = PktMessage::decode(packet);

        assert_eq!(msg.message_len, 3);
        assert_eq!(msg.recipient.as_ref(), "Player1");
        assert_eq!(msg.sender.as_ref(), "Player2");
        assert!(!msg.narration);
        assert_eq!(msg.message.as_ref(), "Sup");
    }

    /// Parse trace message: Server -> Player1 "Player1 has started the game!".
    #[test]
    fn message_parse_trace_server_notification() {
        let stream = test_common::setup();
        let msg_text = "Player1 has started the game!";
        let mut body: Vec<u8> = Vec::new();
        body.extend((msg_text.len() as u16).to_le_bytes());
        let mut recipient = b"Player1".to_vec();
        recipient.resize(32, 0x00);
        body.extend(&recipient);
        let mut sender = b"Server".to_vec();
        sender.resize(32, 0x00);
        body.extend(&sender);
        body.extend(msg_text.as_bytes());

        let packet = Packet::new(&stream, PktType::MESSAGE, &body);
        let msg = PktMessage::decode(packet);

        assert_eq!(msg.recipient.as_ref(), "Player1");
        assert_eq!(msg.sender.as_ref(), "Server");
        assert!(!msg.narration);
        assert_eq!(msg.message.as_ref(), msg_text);
    }

    /// PktMessage::server helper constructs correctly.
    #[test]
    fn message_server_helper() {
        let msg = PktMessage::server("Player1", "Welcome!");
        assert_eq!(msg.packet_type, PktType::MESSAGE);
        assert_eq!(msg.recipient.as_ref(), "Player1");
        assert_eq!(msg.sender.as_ref(), "Server");
        assert!(!msg.narration);
        assert_eq!(msg.message.as_ref(), "Welcome!");
        assert_eq!(msg.message_len, 8);
    }

    /// PktMessage::narrator helper constructs correctly.
    #[test]
    fn message_narrator_helper() {
        let msg = PktMessage::narrator("Player1", "You enter a dark cave.");
        assert_eq!(msg.packet_type, PktType::MESSAGE);
        assert_eq!(msg.recipient.as_ref(), "Player1");
        assert_eq!(msg.sender.as_ref(), "Narrator");
        assert!(msg.narration);
        assert_eq!(msg.message.as_ref(), "You enter a dark cave.");
    }

    /// Narration flag: serialize with narration=true, verify the 0x00 0x01 marker.
    #[test]
    fn message_narration_roundtrip() {
        let stream = test_common::setup();
        let msg = PktMessage::narrator("Player1", "A tale of old.");

        let mut buffer: Vec<u8> = Vec::new();
        msg.write_to(&mut buffer).expect("Encoding failed");

        // Check the narration marker bytes at end of sender field (bytes 35..67, sender is at 35+30=65,66)
        // Sender occupies bytes 35..67 in the full packet (including type byte at 0)
        // In the buffer: type(1) + msg_len(2) + recipient(32) + sender(30+2) + message
        // Check that the sender region has 0x00, 0x01 at positions 65, 66 (0-indexed from buffer start)
        assert_eq!(buffer[65], 0x00);
        assert_eq!(buffer[66], 0x01);

        // Deserialize and verify
        let packet = Packet::new(&stream, PktType::MESSAGE, &buffer[1..]);
        let deserialized = PktMessage::decode(packet);
        assert!(deserialized.narration);
        assert_eq!(deserialized.sender.as_ref(), "Narrator");
    }

    /// Non-narration message should not have the marker.
    #[test]
    fn message_non_narration_roundtrip() {
        let stream = test_common::setup();
        let msg = PktMessage::server("Player1", "Hello.");

        let mut buffer: Vec<u8> = Vec::new();
        msg.write_to(&mut buffer).expect("Encoding failed");

        let packet = Packet::new(&stream, PktType::MESSAGE, &buffer[1..]);
        let deserialized = PktMessage::decode(packet);
        assert!(!deserialized.narration);
        assert_eq!(deserialized.sender.as_ref(), "Server");
        assert_eq!(deserialized.message.as_ref(), "Hello.");
    }

    /// Empty message text.
    #[test]
    fn message_empty_text() {
        let stream = test_common::setup();
        let msg = PktMessage::server("Player1", "");

        let mut buffer: Vec<u8> = Vec::new();
        msg.write_to(&mut buffer).expect("Encoding failed");

        let packet = Packet::new(&stream, PktType::MESSAGE, &buffer[1..]);
        let deserialized = PktMessage::decode(packet);
        assert_eq!(deserialized.message_len, 0);
        assert_eq!(deserialized.message.as_ref(), "");
    }

    /// Long message text.
    #[test]
    fn message_long_text() {
        let stream = test_common::setup();
        let long_text = "X".repeat(5000);
        let msg = PktMessage::server("Player1", &long_text);

        let mut buffer: Vec<u8> = Vec::new();
        msg.write_to(&mut buffer).expect("Encoding failed");

        let packet = Packet::new(&stream, PktType::MESSAGE, &buffer[1..]);
        let deserialized = PktMessage::decode(packet);
        assert_eq!(deserialized.message_len, 5000);
        assert_eq!(deserialized.message.len(), 5000);
    }

    /// Max-length recipient name (32 bytes, no padding).
    #[test]
    fn message_max_length_recipient() {
        let stream = test_common::setup();
        let long_name = "R".repeat(32);
        let msg = PktMessage {
            packet_type: PktType::MESSAGE,
            message_len: 2,
            recipient: Box::from(long_name.as_str()),
            sender: Box::from("Server"),
            narration: false,
            message: Box::from("Hi"),
        };

        let mut buffer: Vec<u8> = Vec::new();
        msg.write_to(&mut buffer).expect("Encoding failed");

        let packet = Packet::new(&stream, PktType::MESSAGE, &buffer[1..]);
        let deserialized = PktMessage::decode(packet);
        assert_eq!(deserialized.recipient.as_ref(), &long_name);
    }

    /// Body too short should panic.
    #[test]
    #[should_panic]
    fn message_body_too_short_panics() {
        let stream = test_common::setup();
        let body: &[u8] = &[0x00, 0x00, 0x41]; // Only 3 bytes, need at least 66
        let packet = Packet::new(&stream, PktType::MESSAGE, body);
        let _ = PktMessage::decode(packet);
    }

    /// Empty body should panic.
    #[test]
    #[should_panic]
    fn message_empty_body_panics() {
        let stream = test_common::setup();
        let body: &[u8] = &[];
        let packet = Packet::new(&stream, PktType::MESSAGE, body);
        let _ = PktMessage::decode(packet);
    }

    /// All-zero 66-byte body should parse without panic.
    #[test]
    fn message_all_zeros_body() {
        let stream = test_common::setup();
        let body: Vec<u8> = vec![0x00; 66];
        let packet = Packet::new(&stream, PktType::MESSAGE, &body);
        let msg = PktMessage::decode(packet);

        assert_eq!(msg.message_len, 0);
        assert_eq!(msg.recipient.as_ref(), "");
        assert_eq!(msg.sender.as_ref(), "");
        assert!(!msg.narration);
    }

    /// All-0xFF 66-byte body should parse without panic.
    #[test]
    fn message_all_ones_body() {
        let stream = test_common::setup();
        let body: Vec<u8> = vec![0xFF; 66];
        let packet = Packet::new(&stream, PktType::MESSAGE, &body);
        let msg = PktMessage::decode(packet);

        assert_eq!(msg.message_len, u16::MAX);
        // Recipient and sender will contain replacement chars for invalid UTF-8
        assert!(!msg.recipient.is_empty());
        assert!(!msg.sender.is_empty());
    }

    /// Non-UTF8 bytes in recipient/sender should use lossy conversion.
    #[test]
    fn message_non_utf8_names() {
        let stream = test_common::setup();
        let mut body: Vec<u8> = Vec::new();
        body.extend(0u16.to_le_bytes()); // message_len
        let mut recipient = vec![0xFF, 0xFE, 0xFD];
        recipient.resize(32, 0x00);
        body.extend(&recipient);
        let mut sender = vec![0xFC, 0xFB, 0xFA];
        sender.resize(32, 0x00);
        body.extend(&sender);

        let packet = Packet::new(&stream, PktType::MESSAGE, &body);
        let msg = PktMessage::decode(packet);

        assert!(msg.recipient.contains('\u{FFFD}'));
        assert!(msg.sender.contains('\u{FFFD}'));
    }

    /// Display/JSON output should be valid JSON.
    #[test]
    fn message_display_valid_json() {
        let msg = PktMessage::server("Player1", "Hello!");
        let json_str = format!("{}", msg);
        let parsed: serde_json::Value = serde_json::from_str(&json_str).expect("Invalid JSON");
        assert_eq!(parsed["recipient"], "Player1");
        assert_eq!(parsed["sender"], "Server");
        assert_eq!(parsed["message"], "Hello!");
    }
}
////////////////////////////////////////////////////////////////////////////////
