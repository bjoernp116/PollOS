#![feature(custom_test_frameworks)]
#![test_runner(pollos::test_runner)]
#![reexport_test_harness_main = "test_main"]
#![feature(asm_experimental_arch)]
#![no_std] // don't link the Rust standard library
#![no_main] // disable all Rust-level entry points

use core::{panic::PanicInfo};
use bootloader::{entry_point, BootInfo};
use pollos::{file_system::read_boot_sector, memory::{allocator::BootInfoFrameAllocator, init_heap}, *};
use x86_64::{instructions::port::Port, VirtAddr};

extern crate alloc;

entry_point!(kernel_main);

fn kernel_main(boot_info: &'static BootInfo) -> ! {
    println!("Hello, World!");
    pollos::init();


    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { memory::init(phys_mem_offset) };
    let mut frame_allocator = unsafe { 
        BootInfoFrameAllocator::new(&boot_info.memory_map)
    };
    init_heap(&mut mapper, &mut frame_allocator).expect("heap init failed!");


    let boot_sector = unsafe { read_boot_sector() }; 
    //let root_dir = file_system::read_root_directory(&boot_sector);

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
    println!("{}", info);
    serial_println!("{}", info);
    hlt_loop();
}

