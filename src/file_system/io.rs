#![allow(private_bounds)]
use core::{arch::asm, marker::PhantomData};


#[derive(Debug)]
pub struct PortReader<T: ReadPort> {
    port: u16,
    _phantom_data: PhantomData<T>
}

impl<T: ReadPort> PortReader<T> {
    pub fn new(port: u16) -> Self {
        Self { port, _phantom_data: PhantomData }
    }
    pub fn read(&self) -> T {
        unsafe { T::read_port(self.port) }
    }
}


#[derive(Debug)]
pub struct PortWriter<T: WritePort> {
    port: u16,
    _phantom_data: PhantomData<T>
}

impl<T: WritePort> PortWriter<T> {
    pub fn new(port: u16) -> Self {
        Self { port, _phantom_data: PhantomData }
    }
    pub fn write(&self, value: T) {
        unsafe { T::write_port(self.port, value) };
    }
}

#[derive(Debug)]
pub struct PortRW<T: ReadPort + WritePort> {
    port: u16,
    _phantom_data: PhantomData<T>
}

impl<T: ReadPort + WritePort> PortRW<T> {
    pub fn new(port: u16) -> Self {
        Self { port, _phantom_data: PhantomData }
    }
    pub fn read(&self) -> T {
        unsafe { T::read_port(self.port) }
    }
    pub fn write(&self, value: T) {
        unsafe { T::write_port(self.port, value) };
    }
}





trait WritePort {
    unsafe fn write_port(port: u16, value: Self);
}

trait ReadPort {
    unsafe fn read_port(port: u16) -> Self;
}

impl WritePort for u8 {
    unsafe fn write_port(port: u16, value: Self) {
        asm!(
            "out dx, al",
            in("dx") port,
            in("al") value,
            options(nomem, nostack)
        );
    } 
}

impl WritePort for u16 {
    unsafe fn write_port(port: u16, value: Self) {
        asm!(
            "out dx, ax",
            in("dx") port,
            in("ax") value,
            options(nomem, nostack)
        );
    } 
}

impl ReadPort for u8 {
    unsafe fn read_port(port: u16) -> Self {
        let output: Self;
        asm!(
            "in ax, dx",
            in("dx") port,
            out("al") output,
            options(nomem, nostack)
        );
        output
    }
}

impl ReadPort for u16 {
    unsafe fn read_port(port: u16) -> Self {
        let output: Self;
        asm!(
            "in ax, dx",
            in("dx") port,
            out("ax") output,
            options(nomem, nostack)
        );
        output
    }
}
