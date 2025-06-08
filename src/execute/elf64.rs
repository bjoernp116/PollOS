use alloc::vec::Vec;
use x86_64::{
    structures::paging::{
        page_table::PageTableLevel, FrameAllocator, OffsetPageTable, Page,
        PageTableFlags, Size4KiB,
    },
    VirtAddr,
};

use crate::{
    file_system::{File, FileSystem, StorageFormat},
    memory::allocator::BootInfoFrameAllocator,
    serial_println,
};

use super::{enter_user_mode, Executor, UserContext};

#[derive(Debug, Clone)]
#[repr(C, packed)]
pub struct ELF64Identity {
    _magic_number: u32, // 0x00
    format: u8,         // 0x04
    endianess: u8,      // 0x05
    version1: u8,       // 0x06
    _os: u8,            // 0x08
    _padding: [u8; 7],  // 0x09
}

#[derive(Debug, Clone)]
#[repr(C, packed)]
pub struct ELF64Header {
    identity: [u8; 16],             // 0x00
    object_type: u16,               // 0x10
    arch: u16,                      // 0x12
    version2: u32,                  // 0x14
    instruction_pointer_entry: u64, // 0x18
    program_header_entry: u64,      // 0x20
    section_header_entry: u64,      // 0x28
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
        if value.len() != core::mem::size_of::<ELF64Header>() {
            panic!(
                "Value: {:?}, Size: {:?}",
                value.len(),
                core::mem::size_of::<ELF64Header>()
            );
        }
        let header =
            unsafe { core::ptr::read(value.as_ptr() as *const ELF64Header) };
        //header.flip_endianess();
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

impl From<&[u8]> for ELF64ProgramHeader {
    fn from(section: &[u8]) -> Self {
        let header = unsafe {
            core::ptr::read(section.as_ptr() as *const ELF64ProgramHeader)
        };
        //header.flip_endianess();
        header
    }
}

bitflags::bitflags! {
    #[derive(Debug, Clone)]
    pub struct ELF64SegmentFlags: u32 {
        const EXACUTABLE = 0x1;
        const WRITABLE = 0x2;
        const READABLE = 0x4;
    }
}

pub fn get_elf64<'a, T: StorageFormat<'a>>(
    fs: &FileSystem<'a, T>,
    file: &File,
) -> anyhow::Result<(ELF64Header, Vec<ELF64ProgramHeader>)> {
    let header_sector = fs.get_content(file);
    let header = ELF64Header::from(&header_sector[..0x40]);

    let size = header.program_header_size as usize;
    let mut entry = header.program_header_entry as usize;
    let mut program_headers = Vec::new();

    for _ in 0..header.program_header_entries {
        let program_header =
            ELF64ProgramHeader::from(&header_sector[entry..entry + size]);
        program_headers.push(program_header);
        entry += size;
    }
    Ok((header, program_headers))
}

pub fn map_program_header(
    program_header: &ELF64ProgramHeader,
    mapper: &mut OffsetPageTable,
    frame_allocator: &mut BootInfoFrameAllocator,
) {
    use x86_64::structures::paging::{
        mapper::Mapper, FrameAllocator, Page, PageTableFlags,
    };

    let start_addr = VirtAddr::new(program_header.virt_addr);
    let end_addr = start_addr + program_header.memory_size;
    let start_page = Page::containing_address(start_addr);
    let end_page = Page::containing_address(end_addr - 1);

    for page in Page::range_inclusive(start_page, end_page) {
        let frame = frame_allocator.allocate_frame().expect("no frame");
        let mut flags = PageTableFlags::PRESENT;
        flags |= PageTableFlags::WRITABLE;
        unsafe {
            mapper
                .map_to(page, frame, flags, frame_allocator)
                .expect("map failed")
                .flush();
        }
    }
}

pub fn map_stack(
    mapper: &mut OffsetPageTable,
    frame_allocator: &mut BootInfoFrameAllocator,
) -> anyhow::Result<(VirtAddr, u64, VirtAddr)> {
    use x86_64::{
        structures::paging::{
            mapper::Mapper, FrameAllocator, Page, PageTableFlags,
        },
        VirtAddr,
    };

    let stack_size: u64 = 16 * 1024;
    let stack_top = VirtAddr::new(0x0000_8000_0000);
    let stack_bottom = stack_top - stack_size;
    let start_page = Page::containing_address(stack_bottom);
    let end_page = Page::containing_address(stack_top - 1u64);

    for page in Page::range_inclusive(start_page, end_page) {
        let frame = frame_allocator.allocate_frame().expect("no frame");
        let flags = PageTableFlags::WRITABLE
            | PageTableFlags::PRESENT
            | PageTableFlags::USER_ACCESSIBLE;
        unsafe {
            mapper
                .map_to(page, frame, flags, frame_allocator)
                .expect("map failed")
                .flush();
        }
    }

    Ok((stack_top, stack_size, stack_bottom))
}

pub fn load_program_header(
    program_header: &ELF64ProgramHeader,
    content: Vec<u8>,
) {
    let file_offset = program_header.offset as usize;
    let file_size = program_header.file_image_size as usize;
    let file_bytes = &content[file_offset..file_offset + file_size];

    let mem_ptr = program_header.virt_addr as *mut u8;

    unsafe {
        core::ptr::copy_nonoverlapping(file_bytes.as_ptr(), mem_ptr, file_size);
        let mem_size = program_header.memory_size as usize;
        if mem_size > file_size {
            core::ptr::write_bytes(
                mem_ptr.add(file_size),
                0,
                mem_size - file_size,
            );
        }
    }
}

pub struct ELF64;

impl Executor for ELF64 {
    fn load_executable<'a, T: StorageFormat<'a>>(
        fs: &FileSystem<'a, T>,
        file: &File,
        mapper: &mut OffsetPageTable,
        frame_allocator: &mut BootInfoFrameAllocator,
    ) -> anyhow::Result<()> {
        let (header, program_headers) = get_elf64(fs, file)?;

        for program_header in program_headers {
            map_program_header(&program_header, mapper, frame_allocator);
            load_program_header(&program_header, fs.get_content(file));
        }
        let (stack_top, ..) = map_stack(mapper, frame_allocator)?;
        let rip = header.instruction_pointer_entry;

        let user_context = UserContext::new(rip, stack_top.as_u64());
        serial_println!("{:#x?}", user_context);
        unsafe {
            enter_user_mode(&user_context);
        };
        Ok(())
    }
}

pub fn test_user_stack_setup(
    mapper: &mut OffsetPageTable,
    frame_allocator: &mut BootInfoFrameAllocator,
) -> anyhow::Result<()> {
    // These would come from your stack setup code
    let (stack_top, stack_size, stack_bottom) =
        map_stack(mapper, frame_allocator)?;
    use x86_64::structures::paging::mapper::Mapper;

    // Check that stack pointer is canonical
    //assert!(stack_top.is_canonical(), "stack_top is not canonical");

    // Check all pages in stack range are mapped, present, user, writable
    let start_page: Page<Size4KiB> = Page::containing_address(stack_bottom);
    let end_page: Page<Size4KiB> = Page::containing_address(stack_top - 1u64);

    for page in Page::range_inclusive(start_page, end_page) {
        let frame = mapper.translate_page(page).expect("Page not mapped");
        let index = page.page_table_index(PageTableLevel::One);
        let entry = mapper.level_4_table()[index].clone();
        serial_println!("{:#x?}", entry);
        assert!(
            entry.flags().contains(PageTableFlags::PRESENT),
            "Not present"
        );
        assert!(
            entry.flags().contains(PageTableFlags::USER_ACCESSIBLE),
            "Not user"
        );
        assert!(
            entry.flags().contains(PageTableFlags::WRITABLE),
            "Not writable"
        );
    }
    // Additional checks as needed, e.g., overlap
    Ok(())
}
