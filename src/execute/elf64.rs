use crate::file_system::{File, FileSystem, StorageFormat};

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
}

impl ELF64Header {
    fn flip_endianess(&mut self) {
        self._os = u16::from_be(self._os);
        self.object_type = u16::from_be(self.object_type);
        self.arch = u16::from_be(self.arch);
        self.version2 = u32::from_be(self.version2);
        self.instruction_pointer_entry =
            u64::from_be(self.instruction_pointer_entry);
        self.program_header_entry = u64::from_be(self.program_header_entry);
        self.section_header_entry = u64::from_be(self.section_header_entry);
        self.flags = u32::from_be(self.flags);
        self.header_size = u16::from_be(self.header_size);
        self.program_header_size = u16::from_be(self.program_header_size);
        self.program_header_entries = u16::from_be(self.program_header_entries);
        self.section_header_entry_size =
            u16::from_be(self.section_header_entry_size);
        self.section_header_entries = u16::from_be(self.section_header_entries);
        self.section_name_index = u16::from_be(self.section_name_index);
    }
}

impl From<&[u8]> for ELF64Header {
    fn from(value: &[u8]) -> Self {
        let mut header =
            unsafe { core::ptr::read(value.as_ptr() as *const ELF64Header) };
        header.flip_endianess();
        header
    }
}

#[derive(Debug)]
#[repr(C, packed)]
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

impl ELF64ProgramHeader {
    fn flip_endianess(&mut self) {
        self.segment_type = u32::from_be(self.segment_type);
        self.segment_flags = u32::from_be(self.segment_flags);
        self.offset = u64::from_be(self.offset);
        self.virt_addr = u64::from_be(self.virt_addr);
        self.phys_addr = u64::from_be(self.phys_addr);
        self.file_image_size = u64::from_be(self.file_image_size);
        self.memory_size = u64::from_be(self.memory_size);
        self.alignment = u64::from_be(self.alignment);
    }
}

impl From<&[u8; 56]> for ELF64ProgramHeader {
    fn from(section: &[u8; 56]) -> Self {
        let mut header = unsafe {
            core::ptr::read(section.as_ptr() as *const ELF64ProgramHeader)
        };
        header.flip_endianess();
        header
    }
}

pub fn get_elf64<'a, T: StorageFormat<'a>>(
    fs: &FileSystem<'a, T>,
    file: &File,
) -> anyhow::Result<(ELF64Header, ELF64ProgramHeader)> {
    let header_sector = fs.get_content(file);
    let header = ELF64Header::from(&header_sector[..0x40]);

    let mut program_header_buffer = [0u8; size_of::<ELF64ProgramHeader>()];

    fs.storage_format.read_bytes(
        file,
        &mut program_header_buffer,
        header.program_header_entry as usize,
    )?;

    let program_header = ELF64ProgramHeader::from(&program_header_buffer);
    Ok((header, program_header))
}
