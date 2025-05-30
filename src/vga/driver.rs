use core::fmt;

use super::*;

static BUFFER_HEIGHT: usize = 25;
static BUFFER_WIDTH: usize = 80;

use volatile::Volatile;

pub struct VGABuffer {
    bytes: [Volatile<Pixel>; BUFFER_HEIGHT * BUFFER_WIDTH],
}

pub struct VGADriver {
    column_position: usize,
    color_code: ColorCode,
    buffer: &'static mut VGABuffer,
}

#[allow(unused)]
impl VGADriver {
    pub fn new() -> Self {
        VGADriver {
            column_position: 0,
            color_code: ColorCode::new(Color::Black, Color::Red),
            buffer: unsafe { &mut *(0xb8000 as *mut VGABuffer) },
        }
    }

    pub fn set_color(&mut self, color_code: ColorCode) {
        self.color_code = color_code;
    }

    pub fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.new_line(),
            b'\t' => self.column_position += 4,
            byte => {
                if self.column_position >= BUFFER_WIDTH {
                    self.new_line()
                }
                let row = BUFFER_HEIGHT - 1;
                let col = self.column_position;
                let color_code = self.color_code;
                self.buffer.bytes[row * BUFFER_WIDTH + col]
                    .write(Pixel(byte, color_code));
                self.column_position += 1;
            }
        }
    }
    pub fn write_string(&mut self, s: &str) {
        for byte in s.bytes() {
            match byte {
                0x20..=0x7e | b'\n' | b'\t' => self.write_byte(byte),
                _ => self.write_byte(0xfe),
            }
        }
    }
    fn new_line(&mut self) {
        for row in 1..BUFFER_HEIGHT {
            for col in 0..BUFFER_WIDTH {
                let charahter =
                    self.buffer.bytes[row * BUFFER_WIDTH + col].read();
                self.buffer.bytes[(row - 1) * BUFFER_WIDTH + col]
                    .write(charahter);
            }
        }
        self.clear_row(BUFFER_HEIGHT - 1);
        self.column_position = 0;
    }
    fn clear_row(&mut self, row: usize) {
        let blank = Pixel::new();
        for col in 0..BUFFER_WIDTH {
            self.buffer.bytes[row * BUFFER_WIDTH + col].write(blank);
        }
    }
    fn draw_row(&mut self, row: usize, pixel: Pixel) {
        for col in 0..BUFFER_WIDTH {
            self.buffer.bytes[row * BUFFER_WIDTH + col].write(pixel);
        }
    }
    fn draw_col(&mut self, col: usize, pixel: Pixel) {
        for row in 0..BUFFER_HEIGHT {
            self.buffer.bytes[row * BUFFER_WIDTH + col].write(pixel);
        }
    }
}

pub fn print_something() {
    let mut writer = VGADriver::new();

    writer.write_byte(b'H');
    writer.write_string("ello ");
    writer.write_string("Wörld!");
}

impl fmt::Write for VGADriver {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}
