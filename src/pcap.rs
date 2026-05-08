use tabled::{
    Table, Tabled,
    settings::{Remove, Style, object::Rows},
};

#[derive(Tabled, Debug, Clone)]
#[doc(hidden)]
struct PCapLine {
    address: String,
    hex: String,
    ascii: String,
}

impl PCapLine {
    #[doc(hidden)]
    fn new(address: String, bytes: &[u8]) -> Self {
        let hex: String = bytes
            .iter()
            .enumerate()
            .map(|(i, b)| {
                if i < 8 {
                    format!("{:02x} ", b)
                } else {
                    format!(" {:02x}", b)
                }
            })
            .collect();

        let ascii = bytes
            .iter()
            .map(|&b| {
                if (32..=126).contains(&b) {
                    b as char
                } else {
                    '.'
                }
            })
            .collect::<String>();

        PCapLine {
            address,
            hex,
            ascii,
        }
    }
}

#[derive(Debug, Clone)]
/// A utility struct for formatting and displaying a vector of bytes.
pub struct PCap;

impl PCap {
    /// Builds a formatted string representation of the provided byte vector.
    ///
    /// ```no_run
    /// use lurk_protocol::pcap::PCap;
    ///
    /// let data = vec![0x48, 0x65, 0x6c, 0x6c, 0x6f, 0x2c, 0x20, 0x57, 0x6f, 0x72, 0x6c, 0x64, 0x21]; // "Hello, World!"
    /// let formatted = PCap::build(data);
    ///
    /// println!("{}", formatted);
    /// ```
    pub fn build(data: Vec<u8>) -> String {
        let mut lines = Vec::new();
        let chunks = data.chunks(16);

        chunks.enumerate().for_each(|(i, bytes)| {
            let address = format!("{:08x}", i * 16);
            let line = PCapLine::new(address, bytes);

            lines.push(line);
        });

        Table::new(lines)
            .with(Remove::row(Rows::first()))
            .with(Style::blank())
            .to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_nonempty_for_nonempty_data() {
        let data = vec![0x48, 0x65, 0x6c, 0x6c, 0x6f]; // "Hello"
        let result = PCap::build(data);
        assert!(
            !result.is_empty(),
            "PCap::build must return non-empty string"
        );
    }

    #[test]
    fn build_contains_hex_bytes() {
        let data = vec![0xDE, 0xAD];
        let result = PCap::build(data);
        // Result must contain the hex representation of the bytes
        assert!(
            result.contains("de"),
            "output should contain hex byte 'de': {}",
            result
        );
        assert!(
            result.contains("ad"),
            "output should contain hex byte 'ad': {}",
            result
        );
    }

    #[test]
    fn build_empty_data() {
        let data = vec![];
        let result = PCap::build(data);
        // Empty input should still produce a string (possibly empty table)
        // Just verify it doesn't panic and is a valid string
        let _ = result;
    }

    #[test]
    fn build_contains_ascii_printable() {
        let data = vec![0x41, 0x42, 0x43]; // "ABC"
        let result = PCap::build(data);
        assert!(
            result.contains("ABC"),
            "output should contain ASCII 'ABC': {}",
            result
        );
    }

    #[test]
    fn build_non_printable_shown_as_dot() {
        let data = vec![0x00, 0x01, 0x02];
        let result = PCap::build(data);
        assert!(
            result.contains("..."),
            "non-printable bytes should be shown as dots: {}",
            result
        );
    }

    #[test]
    fn build_address_starts_at_zero() {
        let data = vec![0xFF];
        let result = PCap::build(data);
        assert!(
            result.contains("00000000"),
            "address should start at 00000000: {}",
            result
        );
    }

    #[test]
    fn build_multiple_lines_for_large_data() {
        // 17 bytes = 2 chunks of 16, so 2 address lines
        let data = vec![0x41; 17];
        let result = PCap::build(data);
        assert!(result.contains("00000000"), "should have first address");
        assert!(
            result.contains("00000010"),
            "should have second address for offset 16"
        );
    }
}
