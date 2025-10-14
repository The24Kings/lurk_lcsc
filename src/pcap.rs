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
    /// use lurk_lcsc::pcap::PCap;
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
