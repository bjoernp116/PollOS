

mod driver;
mod pixel;

pub use driver::*;
pub use pixel::*;

use lazy_static::lazy_static;
use spin::Mutex;

lazy_static!{
    pub static ref VGA_DRIVER: Mutex<VGADriver> = Mutex::new(VGADriver::new());
}

pub enum Intensity {
    Debug,
    Error,
    Warn
}

#[doc(hidden)]
pub fn _print(intesity: Intensity, args: core::fmt::Arguments) {
    use core::fmt::Write;
    use x86_64::instructions::interrupts;
    use Color::*;
    let color = match intesity {
        Intensity::Debug => ColorCode::new(Black, Cyan),
        Intensity::Error => ColorCode::new(Black, Red),
        Intensity::Warn => ColorCode::new(Black, Yellow),
    };
    VGA_DRIVER.lock().set_color(color);
    interrupts::without_interrupts(|| { 
        write!(VGA_DRIVER.lock(), "{}", args).unwrap(); 
    });
}


#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::vga::_print($crate::vga::Intensity::Debug, format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => (
        $crate::vga::_print(
            $crate::vga::Intensity::Debug,
            format_args!("{}\n", format_args!($($arg)*))
    ));
}

#[macro_export]
macro_rules! warn {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => (
        $crate::vga::_print(
            $crate::vga::Intensity::Warn,
            format_args!("{}\n", format_args!($($arg)*))
    ));
}

#[macro_export]
macro_rules! _panic {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => (
        $crate::vga::_print(
            $crate::vga::Intensity::Error,
            format_args!("{}\n", format_args!($($arg)*))
    ));
}
