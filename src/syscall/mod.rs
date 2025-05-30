use crate::{print, println};
use x86_64::structures::idt::InterruptStackFrame;

pub extern "x86-interrupt" fn syscall_handler(
    stack_frame: InterruptStackFrame,
) {
    let syscall_number: i32;
    let arga: u32;
    let argb: u32;
    let argc: u32;
    let argd: u32;
    unsafe {
        //core::arch::asm!("mov {0:e}, ebx", out(reg) args[0]);
        core::arch::asm!(
            "mov {0:e}, ebx",
            "mov {1:e}, ecx",
            "mov {2:e}, esi",
            "mov {3:e}, edi",
            "mov {4:e}, ebp",
            out(reg) syscall_number,
            out(reg) arga,
            out(reg) argb,
            out(reg) argc,
            out(reg) argd,

        );
    }
    println!("{} {} {} {} {}", syscall_number, arga, argb, argc, argd,);
    let syscall_type = match syscall_number {
        1 => SysCallType::Write,
        _ => {
            panic!(
                "Undefined SysCall: {}\nStack: {:#?}",
                syscall_number, stack_frame
            );
        }
    };
    let syscall = SysCall {
        syscall_type,
        arga,
        argb,
        argc,
        argd,
    };
    syscall.execute();
}

pub struct SysCall {
    syscall_type: SysCallType,
    arga: u32,
    argb: u32,
    argc: u32,
    argd: u32,
}

impl SysCall {
    fn execute(&self) {
        match &self.syscall_type {
            SysCallType::Write => {
                let buffer = unsafe {
                    core::slice::from_raw_parts(
                        self.arga as *const u8,
                        self.argb as usize,
                    )
                };
                for &byte in buffer {
                    print!("{}", byte as char);
                }
            }
        }
    }
}

pub enum SysCallType {
    Write,
}

impl SysCallType {}
