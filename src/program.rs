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
        let header = Header::new(&data);
        data.drain(0..(header.header_size as usize * 16));
        for reloc in &header.relocation_table {
            let segment = reloc.segment as u64;
            let offset = reloc.offset as u64;
            let addr = (segment * 16 + offset) as usize;
            let bytes = (start as u16).to_le_bytes();
            data[addr] += bytes[0];
            data[addr + 1] += bytes[1];
        }

        Self {
            data,
            start,
            header,
        }
    }

    pub fn start(&self) -> u64 {
        self.start
    }

    pub fn data(&self) -> &[u8] {
        &self.data
    }

    pub fn header(&self) -> &Header {
        &self.header
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Relocation {
    offset: u16,
    segment: u16,
}

pub struct Header {
    last_page_bytes: u16,
    pages_in_file: u16,
    relocation_rows: u16,
    header_size: u16,
    min_allocation: u16,
    max_allocation: u16,
    pub initial_ss: u16,
    pub initial_sp: u16,
    checksum: u16,
    pub initial_ip: u16,
    pub initial_cs: u16,
    relocation_addr: u16,
    pub relocation_table: Vec<Relocation>,
    overlay: u16,
}

impl Header {
    pub fn new(bytes: &[u8]) -> Header {
        let relocations_count = LittleEndian::read_u16(&bytes[6..8]);
        let mut relocations = vec![];

        // This should really be worked out using relocation_addr, but eh
        for n in 0..(relocations_count as usize) {
            let n = n * 4;
            relocations.push(Relocation {
                offset: LittleEndian::read_u16(&bytes[(28 + n)..(30 + n)]),
                segment: LittleEndian::read_u16(&bytes[(30 + n)..(32 + n)]),
            })
        }

        Header {
            last_page_bytes: LittleEndian::read_u16(&bytes[2..4]),
            pages_in_file: LittleEndian::read_u16(&bytes[4..6]),
            relocation_rows: relocations_count,
            header_size: LittleEndian::read_u16(&bytes[8..10]),
            min_allocation: LittleEndian::read_u16(&bytes[10..12]),
            max_allocation: LittleEndian::read_u16(&bytes[12..14]),
            initial_ss: LittleEndian::read_u16(&bytes[14..16]),
            initial_sp: LittleEndian::read_u16(&bytes[16..18]),
            checksum: LittleEndian::read_u16(&bytes[18..20]),
            initial_ip: LittleEndian::read_u16(&bytes[20..22]),
            initial_cs: LittleEndian::read_u16(&bytes[22..24]),
            relocation_addr: LittleEndian::read_u16(&bytes[24..26]),
            relocation_table: relocations,
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
        let header: [u8; 0xC4] = [
            0x4D, 0x5A, 0x70, 0x00, 0x85, 0x00, 0x2A, 0x00, 0x20, 0x00, 0xF9, 0x02, 0xFF, 0xFF,
            0x3E, 0x11, 0x00, 0x20, 0x00, 0x00, 0xD2, 0xBC, 0x00, 0x00, 0x1C, 0x00, 0x00, 0x00,
            0x34, 0x31, 0x00, 0x00, 0x06, 0x31, 0x00, 0x00, 0xB0, 0x6B, 0x00, 0x00, 0x87, 0x6B,
            0x00, 0x00, 0xBE, 0x78, 0x00, 0x00, 0xAA, 0x83, 0x00, 0x00, 0x68, 0x88, 0x00, 0x00,
            0x3A, 0x88, 0x00, 0x00, 0x5E, 0x8E, 0x00, 0x00, 0xE5, 0x8D, 0x00, 0x00, 0x46, 0x8F,
            0x00, 0x00, 0xF8, 0x8E, 0x00, 0x00, 0x41, 0x92, 0x00, 0x00, 0x73, 0x94, 0x00, 0x00,
            0x40, 0x93, 0x00, 0x00, 0x07, 0x93, 0x00, 0x00, 0xE0, 0x95, 0x00, 0x00, 0x5D, 0x95,
            0x00, 0x00, 0x7B, 0x99, 0x00, 0x00, 0x66, 0x99, 0x00, 0x00, 0x4D, 0x97, 0x00, 0x00,
            0x9A, 0x96, 0x00, 0x00, 0xEA, 0x99, 0x00, 0x00, 0x69, 0x9E, 0x00, 0x00, 0x4E, 0x9E,
            0x00, 0x00, 0xD8, 0xA3, 0x00, 0x00, 0x9F, 0xA3, 0x00, 0x00, 0xDA, 0xA4, 0x00, 0x00,
            0x65, 0xA4, 0x00, 0x00, 0x50, 0xAB, 0x00, 0x00, 0x09, 0xAB, 0x00, 0x00, 0x75, 0xB1,
            0x00, 0x00, 0xA2, 0xB4, 0x00, 0x00, 0x64, 0xB4, 0x00, 0x00, 0xA0, 0xB6, 0x00, 0x00,
            0x21, 0xB6, 0x00, 0x00, 0xDD, 0xB7, 0x00, 0x00, 0xE0, 0xBC, 0x00, 0x00, 0xB9, 0xBD,
            0x00, 0x00, 0xA0, 0xF5, 0x00, 0x00, 0x6E, 0x05, 0x00, 0x10, 0x7D, 0x01, 0x00, 0x10,
        ];

        let header = Header::new(&header);

        assert_eq!(
            header.last_page_bytes,
            LittleEndian::read_u16(&[0x70, 0x00])
        );
        assert_eq!(header.pages_in_file, LittleEndian::read_u16(&[0x85, 0x00]));
        assert_eq!(
            header.relocation_rows,
            LittleEndian::read_u16(&[0x2A, 0x00])
        );
        assert_eq!(header.header_size, LittleEndian::read_u16(&[0x20, 0x00]));
        assert_eq!(header.min_allocation, LittleEndian::read_u16(&[0xf9, 0x02]));
        assert_eq!(header.max_allocation, LittleEndian::read_u16(&[0xff, 0xff]));
        assert_eq!(header.initial_ss, LittleEndian::read_u16(&[0x3E, 0x11]));
        assert_eq!(header.initial_sp, LittleEndian::read_u16(&[0x00, 0x20]));
        assert_eq!(header.checksum, LittleEndian::read_u16(&[0x00, 0x00]));
        assert_eq!(header.initial_ip, LittleEndian::read_u16(&[0xD2, 0xBC]));
        assert_eq!(header.initial_cs, LittleEndian::read_u16(&[0x00, 0x00]));
        assert_eq!(
            header.relocation_addr,
            LittleEndian::read_u16(&[0x1C, 0x00])
        );
        assert_eq!(header.overlay, LittleEndian::read_u16(&[0x00, 0x00]));
        assert!(header.relocation_table.len() == 42);
    }
}

#[repr(C)]
#[repr(packed)]
#[derive(Clone, Copy)]
pub struct PSP {
    // Usually set to INT 0x20 (0xcd20) prog terminate
    exit_interrupt: u16,
    /// "Segment of the first byte beyond the memory allocated to the program"
    /// Yeah that wording sucks. Basically the segment alloc ends
    /// So if we've alloc'd 0x0 -> 0x2000 it'd be 0x2001 /probably/
    alloc_end: u16,
    resv: u8,
    /// Far call instruction to MSDos function dispatcher
    call_disp: u8,
    /// .COM programs bytes available in segment (CP/M)
    com_bytes: u32,
    /// Terminate address used by INT 22, we need to jump to this addr on exit
    /// This forces a child program to return to it's parent program
    term_addr: u32,
    /// The Ctrl-Break exit address, a location of a subroutine for us to run
    /// when we encounter a Ctrl-Break
    ctrl_break_addr: u32,
    /// Similar to the above. If we critically error, run the routine here
    crit_err_addr: u32,
    /// Parent process's segment address
    parent_addr: u16,
    /// File handle array for the process. It's completely undocumented for 2.x+
    /// /probably/ not in use for our case
    file_handle_array: [u8; 20],
    /// Segment address of the environment, or zero
    env_segment_addr: u16,
    stack_save: u32,
    /// File handle array size
    file_handle_size: u16,
    /// File handle array pointer
    file_handle_addr: u32,
    /// Pointer to previous PSP
    prev_psp: u32,
    interim_flag: u8,
    truename_flag: u8,
    nn_flags: u16,
    dos_version: u16,
    spacer: [u8; 14],
    // DOS function dispatcher CDh 21h CBh (Undoc. 3.x+) Ų
    dispatcher: [u8; 3],
    spacer_2: [u8; 9],
    unopened_fcb_1: [u8; 16],
    unopened_fcb_2: [u8; 16],
    spacer_3: [u8; 4],
    cmd_trail_chars: u8,
    cmd_trail: [u8; 127],
}

impl PSP {
    pub fn new(alloc_end: u16, call_disp: u8, cmd: &[u8]) -> Self {
        let mut cmd_trail: [u8; 127] = [0x0; 127];
        let args_length = String::from_utf8_lossy(cmd).chars().count();

        cmd_trail[..cmd.len()].copy_from_slice(cmd);
        cmd_trail[cmd.len() + 2] = 0x0D;

        Self {
            exit_interrupt: 0x20CD,
            alloc_end,
            resv: 0x0,
            call_disp,
            com_bytes: 0x0,
            term_addr: 0xFFFF_FFFF,
            ctrl_break_addr: 0x9999_9999,
            crit_err_addr: 0x9999_9999,
            parent_addr: 0x0,
            file_handle_array: [0; 20],
            env_segment_addr: 0x0,
            file_handle_size: 0x0,
            file_handle_addr: 0x9999_9999,
            prev_psp: 0x0,
            spacer: [0x0; 14],
            dispatcher: [0x0; 3],
            spacer_2: [0x0; 9],
            unopened_fcb_1: [0x0; 16],
            unopened_fcb_2: [0x0; 16],
            cmd_trail_chars: args_length as u8,
            cmd_trail,
            stack_save: 0x0,
            interim_flag: 0x0,
            truename_flag: 0x0,
            nn_flags: 0x0,
            dos_version: 0x0,
            spacer_3: [0x0; 4],
        }
    }
}

unsafe fn any_as_u8_slice<T: Sized>(p: &T) -> &[u8] {
    ::core::slice::from_raw_parts((p as *const T) as *const u8, ::core::mem::size_of::<T>())
}

impl<'a> From<&'a PSP> for &'a [u8] {
    fn from(value: &'a PSP) -> &'a [u8] {
        unsafe { any_as_u8_slice(value) }
    }
}
