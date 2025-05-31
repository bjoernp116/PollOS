use crate::file_system::SECTOR_SIZE;

#[derive(Debug, Clone)]
#[repr(C, packed)]
pub struct ELF64Header {
    _magic_number: u32,
    format: u8,
    endianess: u8,
    version1: u8,
    _os: u16,
    _padding: [u8; 8],
    object_type: u16,
    arch: u16,
    version2: u32,
    instruction_pointer_entry: u64,
    program_header_entry: u64,
    section_header_entry: u64,
    flags: u32,
    header_size: u16,
    program_header_size: u16,
    program_header_entries: u16,
    section_header_entry_size: u16,
    section_header_entries: u16,
    section_name_index: u16,
    _rest: [u8; SECTOR_SIZE - 64],
}

impl From<&[u8]> for ELF64Header {
    fn from(value: &[u8]) -> Self {
        unsafe { core::ptr::read(value.as_ptr() as *const ELF64Header) }
    }
}

#[derive(Debug)]
pub struct ELF64ProgramHeader {
    segment_type: u32,
    segment_flags: u32,
    offset: u64,
    virt_addr: u64,
    phys_addr: u64,
    file_image_size: u64,
    memory_size: u64,
    alignment: u64,
}

impl From<&ELF64Header> for ELF64ProgramHeader {
    fn from(header: &ELF64Header) -> Self {
        let entry = (header.program_header_entry) as usize;
        let section: &[u8] = &header._rest[0 + entry..56 + entry];
        unsafe {
            core::ptr::read(section.as_ptr() as *const ELF64ProgramHeader)
        }
    }
}

fn filp_endian(value: u64) -> u64 {
    let a = value & 0xFFFF000000000000;
    let b = value & 0x0000FFFF00000000;
    let c = value & 0x00000000FFFF0000;
    let d = value & 0x000000000000FFFF;
    (a >> 48) + (b >> 16) + (c >> 16) + 
}






