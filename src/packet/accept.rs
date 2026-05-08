use crate::pkt_type::PktType;
use crate::{Packet, Parser};
use serde::{Deserialize, Serialize};
use std::io::Write;

/// Sent by the server to acknowledge a non-error-causing action which has no other direct result.
///
/// This is not needed for actions which cause other results, such as changing rooms or beginning a fight.
/// It should be sent in response to clients sending messages, setting character stats, etc.
#[derive(Serialize, Deserialize)]
pub struct PktAccept {
    /// The type of message for the `ACCEPT` packet. Default is 8.
    pub packet_type: PktType,
    /// The type of action accepted.
    pub accept_type: u8,
}

impl PktAccept {
    /// Creates a new `PktAccept` with the specified accept type.
    pub fn new(accept_type: PktType) -> Self {
        Self {
            packet_type: PktType::ACCEPT,
            accept_type: accept_type.into(),
        }
    }
}

#[macro_export]
/// Send `PktAccept` over `TcpStream` to connected user
///
/// ```no_run
/// use lurk_protocol::{Protocol, PktType, send_accept};
/// use std::sync::Arc;
/// use std::net::TcpStream;
///
/// let stream = Arc::new(TcpStream::connect("127.0.0.1:8080").unwrap());
///
/// send_accept!(stream.clone(), PktType::CHARACTER)
/// ```
macro_rules! send_accept {
    ($stream:expr, $p_type:expr) => {
        if let Err(e) = $crate::send_to($stream.as_ref(), &$crate::PktAccept::new($p_type)) {
            eprintln!("Failed to send 'ACCEPT' packet: {}", e);
        }
    };
}

impl std::fmt::Display for PktAccept {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            serde_json::to_string(self)
                .unwrap_or_else(|_| "Failed to serialize Accept".to_string())
        )
    }
}

impl Parser<'_> for PktAccept {
    fn write_to<W: Write>(&self, writer: &mut W) -> Result<(), std::io::Error> {
        // Package into a byte array
        let mut packet: Vec<u8> = Vec::new();

        packet.push(self.packet_type.into());
        packet.extend(self.accept_type.to_le_bytes());

        // Write the packet to the buffer
        writer
            .write_all(&packet)
            .map_err(|_| std::io::Error::other("Failed to write packet to buffer"))?;

        Ok(())
    }

    fn decode(packet: Packet) -> Self {
        Self {
            packet_type: packet.packet_type,
            accept_type: packet.body[0],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accept_parse_and_serialize() {
        let type_byte = PktType::ACCEPT;
        let original_bytes: &[u8; 2] = &[0x08, 0x0a];

        // Create a packet with known bytes, excluding the type byte
        let packet = Packet::new(type_byte, &original_bytes[1..]);

        // Deserialize the packet into a PktAccept
        let message = PktAccept::decode(packet);

        // Assert the fields were parsed correctly
        assert_eq!(message.packet_type, PktType::ACCEPT);
        assert_eq!(message.accept_type, u8::from(PktType::CHARACTER));

        // Serialize the message back into bytes
        let mut buffer: Vec<u8> = Vec::new();
        message.write_to(&mut buffer).expect("Encoding failed");

        // Assert that the serialized bytes match the original
        assert_eq!(buffer, original_bytes);
        assert_eq!(buffer[0], u8::from(type_byte));
    }

    /// Parse from trace: ACCEPT for CHARACTER type (0x0a = 10).
    #[test]
    fn accept_parse_trace_character() {
        let body: &[u8] = &[0x0a]; // CHARACTER = 10
        let packet = Packet::new(PktType::ACCEPT, body);
        let acc = PktAccept::decode(packet);

        assert_eq!(acc.accept_type, 10);
        assert_eq!(acc.accept_type, u8::from(PktType::CHARACTER));
    }

    /// Accept for each known packet type.
    #[test]
    fn accept_all_packet_types() {
        let types = [
            PktType::MESSAGE,
            PktType::CHANGEROOM,
            PktType::FIGHT,
            PktType::PVPFIGHT,
            PktType::LOOT,
            PktType::START,
            PktType::ERROR,
            PktType::ACCEPT,
            PktType::ROOM,
            PktType::CHARACTER,
            PktType::GAME,
            PktType::LEAVE,
            PktType::CONNECTION,
            PktType::VERSION,
        ];

        for pkt_type in types {
            let acc = PktAccept::new(pkt_type);
            assert_eq!(acc.packet_type, PktType::ACCEPT);
            assert_eq!(acc.accept_type, u8::from(pkt_type));

            // Serialize and deserialize roundtrip
            let mut buffer: Vec<u8> = Vec::new();
            acc.write_to(&mut buffer).expect("Encoding failed");

            let packet = Packet::new(PktType::ACCEPT, &buffer[1..]);
            let deserialized = PktAccept::decode(packet);
            assert_eq!(deserialized.accept_type, u8::from(pkt_type));
        }
    }

    /// Accept with max u8 value.
    #[test]
    fn accept_max_value() {
        let body: &[u8] = &[0xFF];
        let packet = Packet::new(PktType::ACCEPT, body);
        let acc = PktAccept::decode(packet);

        assert_eq!(acc.accept_type, 255);
    }

    /// Accept with zero value.
    #[test]
    fn accept_zero_value() {
        let body: &[u8] = &[0x00];
        let packet = Packet::new(PktType::ACCEPT, body);
        let acc = PktAccept::decode(packet);

        assert_eq!(acc.accept_type, 0);
    }

    /// Roundtrip via PktAccept::new().
    #[test]
    fn accept_roundtrip() {
        let original = PktAccept::new(PktType::MESSAGE);

        let mut buffer: Vec<u8> = Vec::new();
        original.write_to(&mut buffer).expect("Encoding failed");

        assert_eq!(buffer.len(), 2);
        assert_eq!(buffer[0], u8::from(PktType::ACCEPT));
        assert_eq!(buffer[1], u8::from(PktType::MESSAGE));

        let packet = Packet::new(PktType::ACCEPT, &buffer[1..]);
        let deserialized = PktAccept::decode(packet);
        assert_eq!(deserialized.accept_type, u8::from(PktType::MESSAGE));
    }

    /// Empty body should panic.
    #[test]
    #[should_panic]
    fn accept_empty_body_panics() {
        let body: &[u8] = &[];
        let packet = Packet::new(PktType::ACCEPT, body);
        let _ = PktAccept::decode(packet);
    }

    /// Extra trailing bytes should be ignored.
    #[test]
    fn accept_extra_trailing_bytes() {
        let body: &[u8] = &[0x0a, 0xFF, 0xFF, 0xFF];
        let packet = Packet::new(PktType::ACCEPT, body);
        let acc = PktAccept::decode(packet);

        assert_eq!(acc.accept_type, 10);
    }

    /// Display/JSON output should be valid JSON.
    #[test]
    fn accept_display_valid_json() {
        let acc = PktAccept::new(PktType::CHARACTER);
        let json_str = format!("{}", acc);
        let parsed: serde_json::Value = serde_json::from_str(&json_str).expect("Invalid JSON");
        assert_eq!(parsed["accept_type"], 10);
    }
}
////////////////////////////////////////////////////////////////////////////////
