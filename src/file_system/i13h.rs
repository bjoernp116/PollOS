
pub struct CHS {
    cylinder: u8,
    head: u8,
    sector: u8
}

impl From<LBA> for CHS {
    fn from(lba: LBA) -> Self {
        unsafe {
            core::arch::asm! {
                "mov ah, 8",       

            }
        }
        todo!()
        //let temp = u32::from(lba) /  
    }
}

pub struct Drive {
    drive_number: u8,
    sector_size: usize,
}

impl Drive {
    pub fn new(drive_number: u8, sector_size: usize) -> Self {
        Drive { drive_number, sector_size }
    }

    pub fn read_sectors(&self, sectors_to_read: u8, cylinder: u8, head: u8, sector: u8) {
        let dl = self.drive_number;
        unsafe {
            core::arch::asm! {
                "mov dl, {dl:x}",
                "mov al, {al:x}",
                "mov ch, {cylinder:x}",
                "mov cl, {sector:x}",
                "mov dh, {head:x}",
                "xor bx, bx",
                "mov es, bx",
                "mov bx, 7e00h",
                "int 13h",
                "jmp 7e00h",
                dl = in(reg) dl as u16,
                al = in(reg) sectors_to_read as u16,
                cylinder = in(reg) cylinder as u16,
                head = in(reg) head as u16,
                sector = in(reg) sector as u16,
            }
        }
    }
}





#[derive(Debug)]
pub struct LBA(u8, u8, u8);

impl TryFrom<u32> for LBA {
    type Error = ();
    fn try_from(value: u32) -> Result<Self, Self::Error> {
        if value & 0xFF000000 != 0 { return Err(()) }
        let low  = (value >> 0 ) & 0xff;
        let mid  = (value >> 8 ) & 0xff;
        let high = (value >> 16) & 0xff;
        return Ok(LBA(high as u8, mid as u8, low as u8))
    }
}

impl From<LBA> for u32 {
    fn from(lba: LBA) -> Self {
        ((lba.0 as u32) << 16) + ((lba.1 as u32) << 8) + (lba.0 as u32)
    }
}


