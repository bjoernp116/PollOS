#[repr(u8)]
pub enum Color {
    Black = 0,
    Blue = 1,
    Green = 2,
    Cyan = 3,
    Red = 4,
    Magenta = 5,
    Brown = 6,
    LightGrey = 7,
    DarkGrey = 8,
    BrightBlue = 9,
    BrightGreen = 10,
    BrightCyan = 11,
    BrightRed = 12,
    BrightMagenta = 13,
    Yellow = 14,
    White = 15,
}

#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct ColorCode(u8);
impl ColorCode {
    pub fn new(fg: Color, bg: Color) -> Self {
        ColorCode((fg as u8) << 4 | bg as u8)
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Pixel(pub u8, pub ColorCode);
impl Pixel {
    pub fn new() -> Pixel {
        Pixel(0, ColorCode(0))
    }
}
