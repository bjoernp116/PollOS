
use core::fmt::{Debug, Display};

use alloc::{borrow::ToOwned, boxed::Box, string::String, vec::Vec};

use crate::{print, println, serial_print, serial_println, utils::DoubleVecIndex};

use super::{ATABus, BusDrive, Directory, File, StorageEntry, StorageFormat, TimeStamp, SECTOR_SIZE};
use anyhow::anyhow;


bitflags::bitflags! {
    #[derive(Debug, Clone)]
    pub struct FatAttributes: u8 {
        const READ_ONLY = 0x01;
        const HIDDEN = 0x02;
        const SYSTEM = 0x04;
        const VOLUME_ID = 0x08;
        const DIRECTORY = 0x10;
        const ARCHIVE = 0x20;
        const DEVICE = 0x40;
        const RESERVED = 0x80;
        const LONG_FILE_NAME = 0b00001111;
    }
}


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


/// Directory Entry, Size: 
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
    file_size: u32,
}


impl DirEntry {
    pub fn name(&self) -> Format83 {
        let name = core::str::from_utf8(&self.name).unwrap().trim_end();
        let ext = core::str::from_utf8(&self.ext).unwrap().trim_end();
        let ext = if ext.len() == 0 {
            None
        } else {
            Some(ext.to_owned())
        };
        Format83(name.to_owned(), ext) 
    }
    pub fn timestamp(&self) -> TimeStamp {
        let second =      (0b11111_000000_00000 & self.time) as u8 * 2;
        let minute =    (0b00000_111111_00000 & self.time) as u8;
        let hour =    (0b00000_000000_11111 & self.time) as u8;
        let day =      (0b1111111_0000_00000 & self.date) as u8;
        let month =     (0b0000000_1111_00000 & self.date) as u8;
        let year =       (0b0000000_0000_11111 & self.date) as u8;
        TimeStamp { second, minute, hour, day, month, year }
    }
}

impl From<DirEntry> for Format83 {
    fn from(entry: DirEntry) -> Self {
        entry.name()
    }
}

impl Display for DirEntry {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let size = self.file_size;
        writeln!(f, "Name: {:?}\nExt: {:?}", self.name().0, self.name().1)?;
        writeln!(f, "Timestamp: {}, size: {}", self.timestamp(), size)?;
        writeln!(f, "Attributes: {:?}", FatAttributes::from_bits(self.attributes))?;
        Ok(())
    }
}



pub struct FAT16<'a> {
    ata: &'a ATABus,
    drive: BusDrive,
    boot_sector: BootSector,
}

impl<'a> FAT16<'a> {
    pub fn parse_root_dir(&self) -> Option<Vec<DirEntry>> {
        let (root_sector, _root_sectors) = self.boot_sector.calculate_root_dir_offset();
        let mut buf = [0u8; SECTOR_SIZE];
        println!("Reading: drive: {:?}, sector: {}", self.drive, root_sector);
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
    pub fn get_children(&self, entry: &DirEntry) -> anyhow::Result<Vec<DirEntry>> {
        if !Self::is_directory(&entry) {
            return Err(anyhow!("Entry is not a directory!"));
        } 
        let cluster = entry.start_cluster;
        let start_sector = self.boot_sector.cluster_to_sector(cluster);
        let mut entries = Vec::new();

        for i in 0..self.boot_sector.sectors_per_cluster {
            let mut buf = [0u8; SECTOR_SIZE];
            self.ata.read(&mut buf, self.drive, start_sector + i as usize, 1)?;

            for chunk in buf.chunks_exact(32) {
                let dir_entry: DirEntry = unsafe { 
                    core::ptr::read(chunk.as_ptr() as *const DirEntry)
                };
                if dir_entry.name[0] == 0x00 {
                    break;
                }
                if dir_entry.name[0] == 0xE5 {
                    continue;
                }
                entries.push(dir_entry)
            }
        }

        Ok(entries)
    }
} 


impl BootSector {
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
    fn cluster_to_sector(&self, cluster: u16) -> usize {
        let (_, root_dir_sectors) = self.calculate_root_dir_offset();
        let data_start = 
            (self.reserved_sectors as usize) +
            (self.fat_count as usize) *
            (self.fat_size_sectors as usize) +
            root_dir_sectors;
        data_start + (cluster as usize - 2) * self.sectors_per_cluster as usize
    }
}

impl StorageEntry for DirEntry {}

impl<'a> StorageFormat<'a> for FAT16<'a> {
    type Entry = DirEntry;
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
    fn get_root(&self) -> anyhow::Result<Directory<DirEntry>> {
        let entries = self.parse_root_dir().ok_or(anyhow!("Could'nt parse root!"))?;
        for entry in &entries {
            serial_println!("{}", entry);
        }
        Ok(Directory {
            contents: DoubleVecIndex::new(entries),
            files: Vec::new(),
            directories: Vec::new(),
            name: Format83::new("ROOT".to_owned(), None), 
            time_stamp: TimeStamp::default()
        })
    }
    fn boot_sector(&self) -> BootSector {
        self.boot_sector.clone()
    }
    fn load_child(&self, child: Format83, directory: &mut Directory<DirEntry>) -> anyhow::Result<()> {
        let entry = directory.take(child.clone())
            .ok_or(anyhow!("Entry {} does not exist!", child))?;
        if Self::is_directory(&entry) {
            let children = self.get_children(&entry)?;
            let dir = Directory {
                contents: DoubleVecIndex::new(children),
                files: Vec::new(),
                name: entry.name(),
                directories: Vec::new(),
                time_stamp: entry.timestamp()
            };
            directory.directories.push(Box::new(dir));
        } else {
            let file = File {
                start_sector: entry.start_cluster as u32,
                size: entry.file_size as u32,
                time_stamp: entry.timestamp(),
                name: Format83::from(entry),
            };
            directory.files.push(file);
        }
        Ok(())
    }
    fn is_directory(entry: &DirEntry) -> bool {
        FatAttributes::from_bits_retain(entry.attributes).intersects(FatAttributes::DIRECTORY)
    }
}

#[allow(unused)]
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



#[derive(Clone)]
pub struct Format83(String, Option<String>);

impl Format83 {
    pub fn new(name: String, ext: Option<String>) -> Self {
        Self(name, ext)
    }
}


impl Display for Format83 {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self.1.clone() {
            Some(ext) => write!(f, "{}.{}", self.0, ext),
            None => write!(f, "{}/", self.0)
        }
    }
}

impl Debug for Format83 {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self)
    }
}

impl core::hash::Hash for Format83 {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state);
        self.1.hash(state);
    } 
}
