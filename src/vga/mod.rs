

mod driver;
mod pixel;

pub use driver::*;
pub use pixel::*;

use lazy_static::lazy_static;
use spin::Mutex;

lazy_static!{
    pub static ref VGA_DRIVER: Mutex<VGADriver> = Mutex::new(VGADriver::new());
}


#[doc(hidden)]
pub fn _print(args: core::fmt::Arguments) {
    use core::fmt::Write;
    use x86_64::instructions::interrupts;
    interrupts::without_interrupts(|| { 
        write!(VGA_DRIVER.lock(), "{}", args).unwrap(); 
    });
}


#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::vga::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => (
        $crate::vga::_print(
            format_args!("{}\n", format_args!($($arg)*))
    ));
}
