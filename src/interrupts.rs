use pic8259::ChainedPics;
use spin;
use x86_64::{
    registers::control::Cr2,
    structures::idt::{
        InterruptDescriptorTable, InterruptStackFrame, PageFaultErrorCode,
    },
};

pub const PIC_1_OFFSET: u8 = 32;
pub const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;

pub static PICS: spin::Mutex<ChainedPics> =
    spin::Mutex::new(unsafe { ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET) });

use crate::*;
use lazy_static::lazy_static;

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt[0x80].set_handler_fn(crate::syscall::syscall_handler);
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        unsafe {
            idt.double_fault
                .set_handler_fn(double_fault_handler)
                .set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX);
        }
        idt.divide_error.set_handler_fn(divide_errror_handler);
        idt.debug.set_handler_fn(debug_handler);
        idt.non_maskable_interrupt
            .set_handler_fn(non_maskable_handler);
        idt.overflow.set_handler_fn(overflow_handler);
        idt.bound_range_exceeded
            .set_handler_fn(bound_range_exceeded_handler);
        idt.invalid_opcode.set_handler_fn(invalid_opcode_handler);
        idt.device_not_available
            .set_handler_fn(device_not_available_handler);

        idt.invalid_tss.set_handler_fn(invalid_tss_handler);
        idt.segment_not_present
            .set_handler_fn(segment_not_present_handler);
        idt.stack_segment_fault
            .set_handler_fn(stack_segment_handler);
        idt.general_protection_fault
            .set_handler_fn(general_protection_handler);

        idt.x87_floating_point.set_handler_fn(x87_floating_handler);
        idt.alignment_check.set_handler_fn(alignment_check_handler);
        idt.machine_check.set_handler_fn(machine_check_handler);
        idt.simd_floating_point
            .set_handler_fn(simd_floating_handler);
        idt.virtualization.set_handler_fn(virtualization_handler);
        idt.cp_protection_exception
            .set_handler_fn(cp_protection_handler);
        idt.hv_injection_exception
            .set_handler_fn(hv_injection_handler);
        idt.vmm_communication_exception
            .set_handler_fn(vmm_communication_handler);
        idt.security_exception.set_handler_fn(security_handler);

        idt.page_fault.set_handler_fn(page_fault_handler);
        idt[InterruptIndex::Timer as u8]
            .set_handler_fn(timer_interrupt_handler);
        idt[InterruptIndex::Keyboard as u8]
            .set_handler_fn(keyboard_interrupt_handler);

        idt
    };
}

pub fn init() {
    IDT.load();
}

extern "x86-interrupt" fn page_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: PageFaultErrorCode,
) {
    println!("EXCEPTION: PAGE FAULT");
    println!("Accessed Address: {:?}", Cr2::read());
    println!("Error Code: {:?}", error_code);
    println!("{:#?}", stack_frame);
    hlt_loop();
}

extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    println!("EXCEPTION: BREAKPOINT\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn divide_errror_handler(
    stack_frame: InterruptStackFrame,
) {
    println!("EXCEPTION: DIVIDE ERROR\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn debug_handler(stack_frame: InterruptStackFrame) {
    println!("EXCEPTION: DEBUG\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn non_maskable_handler(
    stack_frame: InterruptStackFrame,
) {
    println!("EXCEPTION: NON MASKABLE INTERRUPT\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn overflow_handler(stack_frame: InterruptStackFrame) {
    println!("EXCEPTION: OVERFLOW\n{:#?}", stack_frame);
}
extern "x86-interrupt" fn bound_range_exceeded_handler(
    stack_frame: InterruptStackFrame,
) {
    println!("EXCEPTION: BOUND RANGE EXCEEDED\n{:#?}", stack_frame);
}
extern "x86-interrupt" fn invalid_opcode_handler(
    stack_frame: InterruptStackFrame,
) {
    panic!("EXCEPTION: INVALID OPCODE\n{:#?}", stack_frame);
}
extern "x86-interrupt" fn device_not_available_handler(
    stack_frame: InterruptStackFrame,
) {
    println!("EXCEPTION: DEVICE NOT AVAILABLE\n{:#?}", stack_frame);
}
extern "x86-interrupt" fn invalid_tss_handler(
    stack_frame: InterruptStackFrame,
    err_code: u64,
) {
    panic!(
        "EXCEPTION: INVALID TSS\n{:#?}\nERR_CODE: {}",
        stack_frame, err_code
    );
}
#[allow(unused)]
extern "x86-interrupt" fn segment_not_present_handler(
    stack_frame: InterruptStackFrame,
    err_code: u64,
) {
}
extern "x86-interrupt" fn stack_segment_handler(
    stack_frame: InterruptStackFrame,
    err_code: u64,
) {
    panic!(
        "EXCEPTION: STACK SEGMENT FAULT\n{:#?}\nERR_CODE: {}",
        stack_frame, err_code
    );
}
extern "x86-interrupt" fn general_protection_handler(
    stack_frame: InterruptStackFrame,
    err_code: u64,
) {
    panic!(
        "EXCEPTION: GENERAL PROTECTION FAULT\n{:#?}\nERR_CODE: {}",
        stack_frame, err_code
    );
}

extern "x86-interrupt" fn x87_floating_handler(
    stack_frame: InterruptStackFrame,
) {
    println!(
        "EXCEPTION: x87 FLOATING POINT EXCEPTION\n{:#?}",
        stack_frame
    );
}

extern "x86-interrupt" fn alignment_check_handler(
    stack_frame: InterruptStackFrame,
    err_code: u64,
) {
    println!(
        "EXCEPTION: ALIGNMENT CHECK\n{:#?}\nERR_CODE: {}",
        stack_frame, err_code
    );
}

extern "x86-interrupt" fn machine_check_handler(
    stack_frame: InterruptStackFrame,
) -> ! {
    panic!("EXCEPTION: MACHINE CHECK\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn simd_floating_handler(
    stack_frame: InterruptStackFrame,
) {
    println!("EXCEPTION: SIMD FLOATING POINT\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn virtualization_handler(
    stack_frame: InterruptStackFrame,
) {
    println!("EXCEPTION: VIRTUALIZATION\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn cp_protection_handler(
    stack_frame: InterruptStackFrame,
    err_code: u64,
) {
    println!(
        "EXCEPTION: CP PROTECTION EXCEPTION\n{:#?}\nERR_CODE: {}",
        stack_frame, err_code
    );
}

extern "x86-interrupt" fn hv_injection_handler(
    stack_frame: InterruptStackFrame,
) {
    println!("EXCEPTION: HV INJECTION EXCEPTION\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn vmm_communication_handler(
    stack_frame: InterruptStackFrame,
    err_code: u64,
) {
    println!(
        "EXCEPTION: VMM COMMUNICATION EXCEPTION\n{:#?}\nERR_CODE: {}",
        stack_frame, err_code
    );
}

extern "x86-interrupt" fn security_handler(
    stack_frame: InterruptStackFrame,
    err_code: u64,
) {
    println!(
        "EXCEPTION: SECURITY EXCEPTION\n{:#?}\nERR_CODE: {}",
        stack_frame, err_code
    );
}

extern "x86-interrupt" fn double_fault_handler(
    stack_frame: InterruptStackFrame,
    _err_code: u64,
) -> ! {
    panic!(
        "EXCEPTION: DOUBLE FAULT\n{:#?}\nERR_CODE: {}",
        stack_frame, _err_code
    );
}
#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum InterruptIndex {
    Timer = PIC_1_OFFSET,
    Keyboard,
}

impl InterruptIndex {
    fn as_u8(self) -> u8 {
        self as u8
    }
}

extern "x86-interrupt" fn timer_interrupt_handler(
    _stack_frame: InterruptStackFrame,
) {
    unsafe {
        PICS.lock()
            .notify_end_of_interrupt(InterruptIndex::Timer.as_u8());
    }
}

extern "x86-interrupt" fn keyboard_interrupt_handler(
    _stack_frame: InterruptStackFrame,
) {
    use pc_keyboard::{
        layouts, DecodedKey, HandleControl, Keyboard, ScancodeSet1,
    };
    use spin::Mutex;
    use x86_64::instructions::port::Port;

    lazy_static! {
        static ref KEYBOARD: Mutex<Keyboard<layouts::No105Key, ScancodeSet1>> = {
            Mutex::new(Keyboard::new(
                ScancodeSet1::new(),
                layouts::No105Key,
                HandleControl::Ignore,
            ))
        };
    }

    let mut keyboard = KEYBOARD.lock();
    let mut port = Port::new(0x60);

    let scancode: u8 = unsafe { port.read() };
    if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
        if let Some(key) = keyboard.process_keyevent(key_event) {
            match key {
                DecodedKey::Unicode(charachter) => print!("{}", charachter),
                DecodedKey::RawKey(key) => print!("{:?}", key),
            }
        }
    }

    unsafe {
        PICS.lock()
            .notify_end_of_interrupt(InterruptIndex::Keyboard.as_u8());
    }
}
