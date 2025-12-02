use byteorder::{ByteOrder, LittleEndian};
use std::fs::read;

pub struct Program {
    // TODO: mapp the section header data directly here so it maps 1-1 with the program memory addresses
    data: Vec<u8>,
    /// Where does execution start
    start: u64,
    header: Header,
}

impl Program {
    pub fn new(path: &str, start: u64) -> Self {
        let mut data = read(path).unwrap();
        let header = Header::new(&mut data);
        data.drain(0..(header.header_size as usize * 16));

        Self {
            data,
            start,
            header,
        }
    }

    pub fn start(&self) -> u64 {
        self.start
    }

    pub fn header(&self) -> &Header {
        &self.header
    }
}

pub struct Header {
    last_page_bytes: u16,
    pages_in_file: u16,
    relocations: u16,
    header_size: u16,
    min_allocation: u16,
    max_allocation: u16,
    initial_ss: u16,
    initial_sp: u16,
    checksum: u16,
    initial_ip: u16,
    initial_cs: u16,
    relocation_table: u16,
    overlay: u16,
}

impl Header {
    pub fn new(bytes: &[u8]) -> Header {
        Header {
            last_page_bytes: LittleEndian::read_u16(&bytes[2..4]),
            pages_in_file: LittleEndian::read_u16(&bytes[4..6]),
            relocations: LittleEndian::read_u16(&bytes[6..8]),
            header_size: LittleEndian::read_u16(&bytes[8..10]),
            min_allocation: LittleEndian::read_u16(&bytes[10..12]),
            max_allocation: LittleEndian::read_u16(&bytes[12..14]),
            initial_ss: LittleEndian::read_u16(&bytes[14..16]),
            initial_sp: LittleEndian::read_u16(&bytes[16..18]),
            checksum: LittleEndian::read_u16(&bytes[18..20]),
            initial_ip: LittleEndian::read_u16(&bytes[20..22]),
            initial_cs: LittleEndian::read_u16(&bytes[22..24]),
            relocation_table: LittleEndian::read_u16(&bytes[24..26]),
            overlay: LittleEndian::read_u16(&bytes[26..28]),
        }
    }
}

#[cfg(test)]
mod tests {
    use byteorder::{ByteOrder, LittleEndian};

    use crate::program::Header;

    #[test]
    fn parse_header() {
        let header: [u8; 0x1D] = [
            0x4D, 0x5A, 0x56, 0x00, 0x84, 0x00, 0x00, 0x00, 0x20, 0x00, 0xF9, 0x02, 0xFF, 0xFF, 0x82, 0x10, 
            0x80, 0x00, 0x00, 0x00, 0x10, 0x00, 0x2B, 0x10, 0x1E, 0x00, 0x00, 0x00, 0x01,
        ];

        let header = Header::new(&header);

        assert_eq!(header.last_page_bytes, LittleEndian::read_u16(&[0x56, 0x00]));
        assert_eq!(header.pages_in_file, LittleEndian::read_u16(&[0x84, 0x00]));
        assert_eq!(header.relocations, LittleEndian::read_u16(&[0x00, 0x00]));
        assert_eq!(header.header_size, LittleEndian::read_u16(&[0x20, 0x00]));
        assert_eq!(header.min_allocation, LittleEndian::read_u16(&[0xf9, 0x02]));
        assert_eq!(header.max_allocation, LittleEndian::read_u16(&[0xff, 0xff]));
        assert_eq!(header.initial_ss, LittleEndian::read_u16(&[0x82, 0x10]));
        assert_eq!(header.initial_sp, LittleEndian::read_u16(&[0x80, 0x00]));
        assert_eq!(header.checksum, LittleEndian::read_u16(&[0x00, 0x00]));
        assert_eq!(header.initial_ip, LittleEndian::read_u16(&[0x10, 0x00]));
        assert_eq!(header.initial_cs, LittleEndian::read_u16(&[0x2b, 0x10]));
        assert_eq!(header.relocation_table, LittleEndian::read_u16(&[0x1e, 0x00]));
        assert_eq!(header.overlay, LittleEndian::read_u16(&[0x00, 0x00]));
    }
}
