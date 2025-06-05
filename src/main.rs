#![feature(custom_test_frameworks)]
#![test_runner(pollos::test_runner)]
#![reexport_test_harness_main = "test_main"]
#![feature(asm_experimental_arch)]
#![no_std]
// don't link the Rust standard library
#![no_main] // disable all Rust-level entry points

use alloc::borrow::ToOwned;
use bootloader::{entry_point, BootInfo};
use core::panic::PanicInfo;
use pollos::{
    execute::{elf64::ELF64, Executor},
    file_system::{fat16::FAT16, ATABus, BusDrive, Directory, FileSystem},
    memory::{allocator::BootInfoFrameAllocator, init_heap},
    *,
};
use x86_64::VirtAddr;

extern crate alloc;

entry_point!(kernel_main);

/// Kernel Entry Point
fn kernel_main(boot_info: &'static BootInfo) -> ! {
    println!("Hello, World!");
    pollos::init();

    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { memory::init(phys_mem_offset) };
    let mut frame_allocator =
        unsafe { BootInfoFrameAllocator::new(&boot_info.memory_map) };
    init_heap(&mut mapper, &mut frame_allocator).expect("heap init failed!");

    let ata = ATABus::new(0x1f0, 0x3f6);
    let fs: FileSystem<'_, FAT16> =
        FileSystem::new(&ata, BusDrive::Slave).expect("Fat init failed!");

    let mut root: Directory<_> = fs.root().expect("Expected root!");
    fs.load_file("printer.elf".to_owned(), &mut root).unwrap();
    let file = &root.files[0];

    ELF64::load_executable(&fs, file, &mut mapper, &mut frame_allocator)
        .unwrap();

    hlt_loop();
}

#[test_case]
fn trivial_assertion() {
    print!("trivial assertion... ");
    assert_eq!(1, 1);
    println!("[ok]");
}

/// This function is called on panic.
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    _panic!("{}", info);
    serial_println!("{}", info);
    hlt_loop();
}
