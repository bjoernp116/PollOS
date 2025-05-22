use core::fmt::Display;

use crate::{print, println, serial_print};

use super::{ATABus, BusDrive, StorageFormat, SECTOR_SIZE};




#[allow(unused)]
#[derive(Debug, Clone)]
#[repr(C, packed)]
pub struct BootSector {
    _jump: [u8; 3],
    _oem_name: [u8; 8],
    bytes_per_sector: u16,
    sectors_per_cluster: u8,
    reserved_sectors: u16,
    fat_count: u8,
    root_dir_entries: u16,
    total_sectors_short: u16,
    _media: u8,
    fat_size_sectors: u16,
}

pub struct DirEntry {
    name: [u8; 8],
    ext: [u8; 3],
    attributes: u8,
    _reserved: [u8; 10],
    time: u16,
    date: u16,
    start_cluster: u16,
    file_size: u16,
}

pub struct FAT16<'a> {
    ata: &'a ATABus,
    drive: BusDrive,
    boot_sector: BootSector,
}

impl<'a> FAT16<'a> {
    pub fn new(ata: &'a ATABus, drive: BusDrive) -> Option<Self> {
        let mut buf = [0u8; SECTOR_SIZE];
        ata.read(&mut buf, drive, 0, 1).ok()?;
        let bs: BootSector = unsafe { core::ptr::read(buf.as_ptr() as *const BootSector) };
        Some(Self {
            ata,
            drive,
            boot_sector: bs
        })
    }
    pub fn parse_root_dir(&self) -> Option<()> {
        let (root_sector, root_sectors) = self.boot_sector.calculate_root_dir_offset();
        let mut buf = [0u8; SECTOR_SIZE];
        self.ata.read(&mut buf, self.drive, root_sector, 1).ok()?;
        print_buffer(&buf);


        Some(())
    }
} 


impl BootSector {
    fn flip_endianess(&mut self) {
        self._jump = [
            u8::from_le(self._jump[0]), 
            u8::from_le(self._jump[1]), 
            u8::from_le(self._jump[2])];
        self._oem_name = [ 
            u8::from_le(self._oem_name[0]), 
            u8::from_le(self._oem_name[1]), 
            u8::from_le(self._oem_name[2]),
            u8::from_le(self._oem_name[3]), 
            u8::from_le(self._oem_name[4]), 
            u8::from_le(self._oem_name[5]),
            u8::from_le(self._oem_name[6]), 
            u8::from_le(self._oem_name[7])];
        self.bytes_per_sector = u16::from_le(self.bytes_per_sector);  
        self.sectors_per_cluster = u8::from_le(self.sectors_per_cluster);  
        self.reserved_sectors = u16::from_le(self.reserved_sectors); 
        self.fat_count = u8::from_le(self.fat_count); 
        self.root_dir_entries = u16::from_le(self.root_dir_entries); 
        self.total_sectors_short = u16::from_le(self.total_sectors_short); 
        self._media = u8::from_le(self._media); 
        self.fat_size_sectors = u16::from_le(self.fat_size_sectors); 

    }
    fn calculate_root_dir_offset(&self) -> (usize, usize) {
        let root_dir_sector = 
            self.reserved_sectors as usize + 
            (self.fat_count as usize * self.fat_size_sectors as usize);
        let root_dir_sectors =
            ((self.root_dir_entries as usize * 32) + 
            (self.bytes_per_sector as usize - 1)) /
            self.bytes_per_sector as usize;
        (root_dir_sector, root_dir_sectors)
    }
}


impl<'a> StorageFormat for FAT16<'a> {
    fn boot_sector(&self) -> BootSector {
        self.boot_sector.clone()
    }
}

fn print_buffer(buffer: &[u8; SECTOR_SIZE]) {
    for (i, byte) in buffer.iter().enumerate() {
        if i % 16 == 0 { print!("\n{:04x}: ", i); }
        print!("{:02x} ", byte);
    }
}
