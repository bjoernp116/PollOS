use x86_64::{
    registers::control::Cr3,
    structures::paging::{
        page_table::FrameError, FrameAllocator, Mapper, OffsetPageTable, Page,
        PageTable, PhysFrame, Size4KiB,
    },
    PhysAddr, VirtAddr,
};

pub unsafe fn init(
    physical_memory_offset: VirtAddr,
) -> OffsetPageTable<'static> {
    unsafe {
        let level_4_table = active_level_4_table(physical_memory_offset);
        OffsetPageTable::new(level_4_table, physical_memory_offset)
    }
}

pub fn create_example_mapping(
    page: Page,
    mapper: &mut OffsetPageTable,
    frame_allocator: &mut impl FrameAllocator<Size4KiB>,
) {
    use x86_64::structures::paging::PageTableFlags as Flags;

    let frame: PhysFrame<Size4KiB> =
        PhysFrame::containing_address(PhysAddr::new(0xb8000));
    let flags = Flags::PRESENT | Flags::WRITABLE;

    let map_to_result =
        unsafe { mapper.map_to(page, frame, flags, frame_allocator) };
    map_to_result.expect("map_to failed").flush();
}

unsafe fn active_level_4_table(
    physical_memory_offset: VirtAddr,
) -> &'static mut PageTable {
    let (root_table, _) = Cr3::read();

    let phys = root_table.start_address();
    let virt = physical_memory_offset + phys.as_u64();
    let page_table_ptr: *mut PageTable = virt.as_mut_ptr();

    unsafe { &mut *page_table_ptr }
}

pub unsafe fn translate_addr(
    addr: VirtAddr,
    physical_memory_offset: VirtAddr,
) -> Option<PhysAddr> {
    translate_addr_inner(addr, physical_memory_offset)
}

fn translate_addr_inner(
    addr: VirtAddr,
    physical_memory_offset: VirtAddr,
) -> Option<PhysAddr> {
    let (level_4_table_frame, _) = Cr3::read();
    let table_indexes = [
        addr.p4_index(),
        addr.p3_index(),
        addr.p2_index(),
        addr.p1_index(),
    ];
    let mut frame = level_4_table_frame;

    for &index in &table_indexes {
        let virt = physical_memory_offset + frame.start_address().as_u64();
        let table_ptr: *const PageTable = virt.as_ptr();
        let table = unsafe { &*table_ptr };

        let entry = &table[index];
        frame = match entry.frame() {
            Ok(frame) => frame,
            Err(FrameError::FrameNotPresent) => return None,
            Err(FrameError::HugeFrame) => panic!("huge pages not supported"),
        };
    }

    Some(frame.start_address() + u64::from(addr.page_offset()))
}
