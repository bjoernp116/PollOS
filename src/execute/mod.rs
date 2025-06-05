use x86_64::structures::paging::OffsetPageTable;

use crate::{
    file_system::{File, FileSystem, StorageFormat},
    memory::allocator::BootInfoFrameAllocator,
};

pub mod elf64;

pub trait Executor {
    fn load_executable<'a, T: StorageFormat<'a>>(
        fs: &FileSystem<'a, T>,
        file: &File,
        mapper: &mut OffsetPageTable,
        frame_allocator: &mut BootInfoFrameAllocator,
    ) -> anyhow::Result<()>;
}

pub struct UserContext {
    pub rip: u64,
    pub rsp: u64,
    pub cs: u64,
    pub ss: u64,
    pub rflags: u64,
}

impl UserContext {
    pub fn new(entry_point: u64, user_stack_top: u64) -> Self {
        // Standard GDT selectors for user mode
        let cs = 0x23; // User code segment, RPL=3
        let ss = 0x1b; // User data segment, RPL=3
        let rflags = 0x202; // Interrupts enabled

        Self {
            rip: entry_point,
            rsp: user_stack_top,
            cs,
            ss,
            rflags,
        }
    }
}
