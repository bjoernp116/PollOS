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
        let cs = 0x23; //GDT.1.user_code_selector.0 as u64; // User code segment, RPL=3
        let ss = 0x1b; //GDT.1.user_data_selector.0 as u64; // User data segment, RPL=3
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

pub unsafe fn enter_user_mode(ctx: &UserContext) -> ! {
    core::arch::asm!(
        "mov rsp, {0}",
        "push {1}",        // SS
        "push {0}",        // RSP
        "push {2}",        // RFLAGS
        "push {3}",        // CS
        "push {4}",        // RIP
        "iretq",
        in(reg) ctx.rsp,
        in(reg) ctx.ss,
        in(reg) ctx.rflags,
        in(reg) ctx.cs,
        in(reg) ctx.rip,
        options(noreturn)
    );
}
