
use core::fmt::{Debug, Display};

use alloc::{borrow::ToOwned, boxed::Box, string::String, vec::Vec};

use crate::{print, println, serial_print, serial_println, utils::DoubleVecIndex, warn};

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

#[derive(Clone, Debug)]
pub struct ParsedDirEntry {
    name: String,
    ext: Option<String>,
    entry: DirEntry
}

impl ParsedDirEntry{
    pub fn name(&self) -> String {
        if let Some(ext) = &self.ext {
            alloc::format!("{}.{}", self.name, ext)
        } else {
            self.name.clone()
        }
    }
}

/// Directory Entry, Size: 32B
#[derive(Debug, Clone)]
#[repr(C, packed)]
pub struct DirEntry {
    _name: [u8; 8],
    _ext: [u8; 3],
    attributes: u8,
    _reserved: u8,
    _creation_time: u8,
    creation_time: u16,
    creation_date: u16,
    accessed_date: u16,
    _zero: u16,
    modification_time: u16,
    modification_date: u16,
    start_cluster: u16,
    file_size: u32,
}


impl DirEntry {
    pub fn timestamp(&self) -> TimeStamp {
        let second =      (0b11111_000000_00000 & self.creation_time) as u8 * 2;
        let minute =    (0b00000_111111_00000 & self.creation_time) as u8;
        let hour =    (0b00000_000000_11111 & self.creation_time) as u8;
        let day =      (0b1111111_0000_00000 & self.creation_date) as u8;
        let month =     (0b0000000_1111_00000 & self.creation_date) as u8;
        let year =       (0b0000000_0000_11111 & self.creation_date) as u8;
        TimeStamp { second, minute, hour, day, month, year }
    }
}

impl From<ParsedDirEntry> for String {
    fn from(entry: ParsedDirEntry) -> Self {
        if let Some(ext) = entry.ext {
            alloc::format!("{}.{}", entry.name, ext)
        } else {
            entry.name
        }
    }
}

impl Display for ParsedDirEntry {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let size = self.entry.file_size;
        writeln!(f, "Name: {:?}\nExt: {:?}", self.name, self.ext)?;
        writeln!(f, "Timestamp: {}, size: {}", self.entry.timestamp(), size)?;
        writeln!(f, "Attributes: {:?}", FatAttributes::from_bits(self.entry.attributes))?;
        Ok(())
    }
}



pub struct FAT16<'a> {
    ata: &'a ATABus,
    drive: BusDrive,
    boot_sector: BootSector,
}

impl<'a> FAT16<'a> {
    pub fn parse_root_dir(&self) -> Option<Vec<ParsedDirEntry>> {
        assert_eq!(size_of::<DirEntry>(), 32);
        let (root_sector, _root_sectors) = self.boot_sector.calculate_root_dir_offset();
        let mut buf = [0u8; SECTOR_SIZE];
        self.ata.read(&mut buf, self.drive, root_sector, 1).ok()?;
        self.parse_buffer(&buf)
    }
    pub fn parse_buffer(&self, buffer: &[u8; SECTOR_SIZE]) -> Option<Vec<ParsedDirEntry>> {
        let mut out: Vec<ParsedDirEntry> = Vec::new();
        let mut lfn_chunks = Vec::new();
        for chunk in buffer.chunks_exact(32) {
            if chunk[0] == 0x00 { break; }
            if chunk[0] == 0xE5 { continue; }

            if chunk[11] == 0x0F {
                lfn_chunks.push(chunk.to_owned());
                continue;
            }

            let (name, ext): (String, Option<String>) = if !lfn_chunks.is_empty() {
                lfn_chunks.reverse();
                let mut name = Vec::new();
                
                for lfn in &lfn_chunks {
                    name.extend_from_slice(&lfn[1..11]);
                    name.extend_from_slice(&lfn[14..26]);
                    name.extend_from_slice(&lfn[28..32]);
                }

                let words: Vec<u16> = name
                    .chunks(2)
                    .map(|b| u16::from_le_bytes([b[0], b[1]]))
                    .take_while(|&c| c != 0x0000 && c != 0xFFFF)
                    .collect();

                let filename = String::from_utf16(&words).unwrap_or_else(|_| "<invalid>".to_owned());
                let (name, ext) = filename
                    .as_str()
                    .split_once('.')
                    .unzip();
                let name = name.unwrap_or(filename.as_str()).to_owned(); 
                let ext = if let Some(mut ext) = ext {
                    Some(ext.to_owned())
                } else {
                    None
                };
                (name, ext)
            } else if chunk[0] == 46 {
                if chunk[1] == 46 {
                    ("..".to_owned(), None)
                } else {
                    (".".to_owned(), None)
                }
            } else {
                let format = Format83::from_bytes(&buffer[0..11]);
                (format.0, format.1)
            };
            lfn_chunks.clear();
            let de = unsafe { core::ptr::read(chunk.as_ptr() as *const DirEntry)};
            let parsed = ParsedDirEntry {
                name: name,
                ext: ext.to_owned(),
                entry: de
            };
            out.push(parsed);
        }
        Some(out)
    }
    pub fn get_children(
        &self, 
        entry: &ParsedDirEntry
    ) -> anyhow::Result<Vec<ParsedDirEntry>> {
        if !Self::is_directory(entry) {
            return Err(anyhow!("Entry is not a directory!"));
        } 
        let cluster = entry.entry.start_cluster;
        let start_sector = self.boot_sector.cluster_to_sector(cluster);
        let mut entries = Vec::new();

        for i in 0..self.boot_sector.sectors_per_cluster {
            let mut buf = [0u8; SECTOR_SIZE];
            self.ata.read(&mut buf, self.drive, start_sector + i as usize, 1)?;
            let mut parsed_entries = self.parse_buffer(&buf).ok_or_else(
                || anyhow!("Couldnt parse entry")
            )?;
            serial_println!("{:#?}", parsed_entries);
            entries.append(&mut parsed_entries);
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

impl StorageEntry for ParsedDirEntry {}

impl<'a> StorageFormat<'a> for FAT16<'a> {
    type Entry = ParsedDirEntry;
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
    fn get_root(&self) -> anyhow::Result<Directory<Self::Entry>> {
        let entries = self.parse_root_dir().ok_or(anyhow!("Could'nt parse root!"))?;
        Ok(Directory {
            contents: DoubleVecIndex::new(entries),
            files: Vec::new(),
            directories: Vec::new(),
            name: "ROOT".to_owned(), 
            time_stamp: TimeStamp::default()
        })
    }
    fn boot_sector(&self) -> BootSector {
        self.boot_sector.clone()
    }
    fn load_child(&self, child: String, directory: &mut Directory<Self::Entry>) -> anyhow::Result<()> {
        let entry = directory.take(child.clone())
            .ok_or(anyhow!("Entry {} does not exist!", child))?;
        if Self::is_directory(&entry) {
            let children = self.get_children(&entry)?;
            let dir = Directory {
                contents: DoubleVecIndex::new(children),
                files: Vec::new(),
                name: entry.name(),
                directories: Vec::new(),
                time_stamp: entry.entry.timestamp()
            };
            directory.directories.push(Box::new(dir));
        } else {
            let file = File {
                start_sector: entry.entry.start_cluster as u32,
                size: entry.entry.file_size as u32,
                time_stamp: entry.entry.timestamp(),
                name: entry.name,
                ext: entry.ext.unwrap_or("".to_owned()),
            };
            directory.files.push(file);
        }
        Ok(())
    }
    fn is_directory(entry: &Self::Entry) -> bool {
        FatAttributes::from_bits_retain(entry.entry.attributes).intersects(FatAttributes::DIRECTORY)
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
    pub fn from_bytes(bytes: &[u8]) -> Self {
        let name = core::str::from_utf8(&bytes[0..8]).unwrap().trim_end();
        let ext = core::str::from_utf8(&bytes[8..11]).unwrap().trim_end();
        let ext = if ext.len() == 0 {
            None
        } else {
            Some(ext.to_owned())
        };
        Format83(name.to_owned(), ext) 
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
