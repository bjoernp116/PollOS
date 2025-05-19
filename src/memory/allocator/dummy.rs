use core::{alloc::GlobalAlloc, ptr::null_mut};

use bootloader::bootinfo::{MemoryMap, MemoryRegionType};
use x86_64::{structures::paging::{FrameAllocator, PhysFrame, Size4KiB}, PhysAddr};



pub struct Dummy;

unsafe impl GlobalAlloc for Dummy {
    unsafe fn alloc(&self, layout: core::alloc::Layout) -> *mut u8 {
        null_mut() 
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: core::alloc::Layout) {
        panic!("Dealloc should never be called!")
    }
}


