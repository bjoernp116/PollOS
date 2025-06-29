#![allow(static_mut_refs)]
use lazy_static::lazy_static;
use x86_64::instructions::tables::load_tss;
use x86_64::registers::segmentation::{Segment, CS};
use x86_64::structures::tss::TaskStateSegment;
use x86_64::{structures::gdt::SegmentSelector, VirtAddr};

pub const DOUBLE_FAULT_IST_INDEX: u16 = 0;

lazy_static! {
    static ref TSS: TaskStateSegment = {
        let mut tss = TaskStateSegment::new();

        const STACK_SIZE: usize = 4096 * 5;
        static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];

        // SAFETY: `STACK` is static and only assigned once here
        let stack_start = VirtAddr::from_ptr(unsafe { STACK.as_ptr() });
        let stack_end = stack_start + STACK_SIZE as u64;

        tss.privilege_stack_table[0] = stack_end;

        tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX as usize] = stack_end;
        tss
    };
}

use x86_64::structures::gdt::{Descriptor, GlobalDescriptorTable};

use crate::println;
lazy_static! {
    pub static ref GDT: (GlobalDescriptorTable, Selectors) = {
        let mut gdt = GlobalDescriptorTable::new();
        let kernel_code_selector =
            gdt.append(Descriptor::kernel_code_segment());
        let kernel_data_selector =
            gdt.append(Descriptor::kernel_data_segment());
        let tss_selector = gdt.append(Descriptor::tss_segment(&TSS));
        let user_code_selector = gdt.append(Descriptor::user_code_segment());
        let user_data_selector = gdt.append(Descriptor::user_data_segment());
        (
            gdt,
            Selectors {
                kernel_code_selector,
                kernel_data_selector,
                tss_selector,
                user_code_selector,
                user_data_selector,
            },
        )
    };
}

pub struct Selectors {
    pub kernel_code_selector: SegmentSelector,
    pub kernel_data_selector: SegmentSelector,
    pub tss_selector: SegmentSelector,
    pub user_code_selector: SegmentSelector,
    pub user_data_selector: SegmentSelector,
}
pub fn init() {
    GDT.0.load();
    println!("code_selector = {:#X}", GDT.1.user_code_selector.0);
    println!("tss_selector = {:#X}", GDT.1.user_data_selector.0);
    unsafe {
        CS::set_reg(GDT.1.kernel_code_selector);
        load_tss(GDT.1.tss_selector);
    }
}
