use alloc::vec::Vec;
use x86_64::structures::idt::InterruptStackFrame;
use crate::println;


pub extern "x86-interrupt" fn syscall_handler(
    stack_frame: InterruptStackFrame,
) {
    let syscall_number: i32;
    let arg1: i32;
    let arg2: i32;
    let arg3: i32;
    let arg4: i32;
    unsafe {
        //core::arch::asm!("mov {0:e}, ebx", out(reg) args[0]);
        core::arch::asm!(
            "mov {0:e}, ebx",
            "mov {1:e}, edx",
            "mov {2:e}, esi",
            "mov {3:e}, edi",
            "mov {4:e}, ebp",
            out(reg) syscall_number,
            out(reg) arg1,
            out(reg) arg2,
            out(reg) arg3,
            out(reg) arg4,

        );
    }
    let syscall_type = match syscall_number {
        1 => SysCallType::Write,
        _ => {
            panic!(
                "Undefined SysCall: {}\nStack: {:#?}", 
                syscall_number, 
                stack_frame
            );
        }
    };
    let syscall = SysCall {
        syscall_type,
        args: Vec::from([arg1, arg2, arg3, arg4]),
    };
    syscall.execute();
}

pub struct SysCall {
    syscall_type: SysCallType,
    args: Vec<i32>
}

impl SysCall {
    fn execute(&self) {
        match &self.syscall_type {
            SysCallType::Write => {
                println!("{}", self.args[0]);
            }
        }
    }
}

pub enum SysCallType {
    Write,
}

impl SysCallType {
    
}
