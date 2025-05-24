
use core::fmt::{Debug, Display};

use alloc::{format, string::String, vec::Vec};

use crate::{print, println, serial_print, utils::DoubleVecIndex};

use super::{ATABus, BusDrive, Directory, StorageFormat, SECTOR_SIZE};
use anyhow::anyhow;




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

#[derive(Debug, Clone)]
#[repr(C, packed)]
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
impl DirEntry {
    pub fn name(&self) -> String {
        self.name.iter()
            .take_while(|&&byte| byte != 0x20)
            .map(|&byte| byte as char)
            .collect()
    }
    pub fn ext(&self) -> String {
        self.ext.iter()
            .filter(|&&byte| byte != 0x20)
            .map(|&byte| byte as char)
            .collect()
    }
    pub fn identifier(&self) -> String {
        format!("{}.{}", self.name(), self.ext())
    }
    pub fn is_dir(&self) -> bool {
        self.file_size == 0 && self.ext[0] + self.ext[1] + self.ext[2] == 0
    }
}

impl From<DirEntry> for String {
    fn from(entry: DirEntry) -> Self {
        entry.name()
    }
}



pub struct FAT16<'a> {
    ata: &'a ATABus,
    drive: BusDrive,
    boot_sector: BootSector,
}

impl<'a> FAT16<'a> {
    pub fn parse_root_dir(&self) -> Option<Vec<DirEntry>> {
        let (root_sector, root_sectors) = self.boot_sector.calculate_root_dir_offset();
        let mut buf = [0u8; SECTOR_SIZE];
        self.ata.read(&mut buf, self.drive, root_sector, 1).ok()?;
        let mut out: Vec<DirEntry> = Vec::new();
        for chunk in buf.chunks_exact(32) {
            let de = unsafe { core::ptr::read(chunk.as_ptr() as *const DirEntry)};
            if de.name[0] == 0x00 { break; } // End
            if de.name[0] == 0xE5 { continue; } // Entry Deleted
            out.push(de);
        }
        Some(out)
    }
    pub fn get_children(&self, entry: &DirEntry) -> Vec<DirEntry> {
       todo!() 
    }
    fn convert_entries(&self, entry: &DirEntry) -> Directory<'a> {
        let children = self.get_children(entry);
        Directory {
            contents: DoubleVecIndex::new(children),
            files: Vec::new(),
            name: entry.name(),
            directories: Vec::new()
        }
    }
} 


impl BootSector {
    fn _flip_endianess(&mut self) {
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


impl<'a> StorageFormat<'a> for FAT16<'a> {
    fn new(ata: &'a ATABus, drive: BusDrive) -> anyhow::Result<Self> {
        let mut buf = [0u8; SECTOR_SIZE];
        ata.read(&mut buf, drive, 0, 1)?;
        let bs: BootSector = unsafe { core::ptr::read(buf.as_ptr() as *const BootSector) };
        Ok(
            Self {
                ata,
                drive,
                boot_sector: bs,
            }
        )
    }
    fn get_root(&self) -> anyhow::Result<Directory> {
        let entries = self.parse_root_dir().ok_or(anyhow!("Could'nt parse root!"))?;
        Ok(Directory {
            contents: DoubleVecIndex::new(entries),
            files: Vec::new(),
            directories: Vec::new(),
            name: String::from(""),
        })
    }
    fn boot_sector(&self) -> BootSector {
        self.boot_sector.clone()
    }
}

fn print_buffer(buffer: &[u8; SECTOR_SIZE]) {
    for (i, byte) in buffer.iter().enumerate() {
        if i % 16 == 0 { print!("\n{:04x}: ", i); }
        print!("{:02x} ", byte);
    }
    println!();
    for (i, byte) in buffer.iter().enumerate() {
        if i % 16 == 0 { serial_print!("\n{:04x}: ", i); }
        serial_print!("{:02x} ", byte);
    }
    println!();
}



pub struct Format83(String, String);

impl Format83 {
    fn new(entry: &DirEntry) -> Self {
        Self(entry.name(), entry.identifier())
    }
}

impl Display for Format83 {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Debug for Format83 {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}.{}", self.0, self.1)
    }
}
